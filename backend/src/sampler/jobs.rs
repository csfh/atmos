//! Jobs: work that runs for a while, streams output, may read input and may
//! need the user's session. A job runs in the daemon, as a child with the
//! caller's environment, on a lane of the command bus, and it outlives the
//! window that started it.
//!
//! Frames, all carrying the window's own `cid`:
//!
//! - window to daemon: `{"t":"job","cid":C,"name":N,"args":{..},"env":{..},"stdin":".."}`,
//!   `{"t":"input","job":J,"data":"..","close":bool}`, `{"t":"cancel","job":J}`,
//!   `{"t":"attach","job":J,"from":SEQ}`;
//! - daemon to window: `{"t":"job.started","cid":C,"job":J}`,
//!   `{"t":"stream","cid":C,"job":J,"seq":K,"fd":"out"|"err","line":".."}` and
//!   `{"t":"job.exit","cid":C,"job":J,"code":N,"summary":{ok,code,message,warnings}}`.
//!
//! The daemon keeps each job's output and its end, so a window that reopens
//! finds the job in `list` and takes the rest with `attach`.

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::process::{ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use super::bus::Subscriber;
use super::commands::Commands;

/// Lines of output a job keeps for a window that comes back.
const LOG_CAP: usize = 5000;

/// Finished jobs kept for a window that comes back.
const KEEP_FINISHED: usize = 50;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// How a job's process is started.
///
/// `Systemd` runs it as a transient user unit (`systemd-run --user --pipe
/// --wait`), so it has the priority and the cgroup of an ordinary program of
/// the user's, not the daemon's: a recording or a picker the user is waiting
/// on must not run at the daemon's `Nice=15`, and `systemctl restart
/// atmosd.service` must not kill a job or an app it started. `Direct` is a
/// child of the daemon, for machines with no user manager and for tests,
/// which must never reach the real one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Runner {
    #[default]
    Direct,
    Systemd,
}

impl Runner {
    /// A transient unit where a user manager can be reached, a child otherwise.
    pub fn detect() -> Runner {
        let bus = std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some()
            || std::env::var_os("XDG_RUNTIME_DIR")
                .is_some_and(|dir| std::path::Path::new(&dir).join("bus").exists());
        if bus && crate::tasks::present("systemd-run") {
            Runner::Systemd
        } else {
            Runner::Direct
        }
    }
}

/// Environment names handed on to a transient unit. The unit starts from the
/// user manager's own environment, which has the session's; these are the
/// window's values for the names that differ between sessions and windows.
const PASSED: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "LANG",
    "LANGUAGE",
    "TERM",
    "DISPLAY",
    "WAYLAND_DISPLAY",
    "DBUS_SESSION_BUS_ADDRESS",
    "SSH_AUTH_SOCK",
    "SUDO_ASKPASS",
    "OMARCHY_PATH",
    "HYPRLAND_INSTANCE_SIGNATURE",
];

fn passed(name: &str) -> bool {
    PASSED.contains(&name)
        || [
            "XDG_",
            "LC_",
            "ATMOS_",
            "HYPRLAND_",
            "XCURSOR_",
            "QT_",
            "GTK_",
        ]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// The command line that runs `argv` as a transient user unit named `unit`.
pub fn unit_argv(
    unit: &str,
    describe: &str,
    argv: &[String],
    env: &[(String, String)],
    wait: bool,
) -> Vec<String> {
    let mut out: Vec<String> = ["systemd-run", "--user", "--collect", "--quiet"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if wait {
        out.extend(["--pipe".to_string(), "--wait".to_string()]);
        // What a tool leaves running when it ends is not the job's to kill: a
        // screen recorder, a clipboard holder (`wl-copy`) and a switcher all
        // start a background process and exit. The default would end them
        // with the unit. A cancel signals the whole unit instead (`cancel`).
        out.push("--property=KillMode=process".to_string());
    }
    out.push(format!("--unit={unit}"));
    out.push(format!("--description={describe}"));
    for (name, _) in env {
        if passed(name) {
            out.push(format!("--setenv={name}"));
        }
    }
    out.push("--".into());
    out.extend(argv.iter().cloned());
    out
}

/// What a registered job runs.
pub struct Spec {
    pub argv: Vec<String>,
    pub lane: String,
    /// Topics the job changes: held back from readers while it runs, dropped when it ends.
    pub affects: Vec<String>,
    /// The job reads its input after the first write, so stdin stays open.
    pub keep_stdin: bool,
    /// What it reads first: a password, a plan, a report.
    pub stdin: Option<String>,
}

#[derive(Clone)]
enum State {
    Queued,
    Running,
    Done { code: i32, summary: Value },
}

struct Line {
    seq: u64,
    fd: &'static str,
    text: String,
}

pub struct Job {
    id: u64,
    cid: String,
    name: String,
    started_at: u64,
    state: Mutex<State>,
    log: Mutex<VecDeque<Line>>,
    next_seq: AtomicU64,
    stdin: Mutex<Option<ChildStdin>>,
    pid: Mutex<Option<u32>>,
    /// The transient unit the job runs in, when it has one.
    unit: Mutex<Option<String>>,
    cancelled: std::sync::atomic::AtomicBool,
    /// The window's own name for the kind of job, so a window that comes back
    /// can tell what it is looking at.
    label: String,
    owner: Mutex<Option<Arc<Subscriber>>>,
    /// Runs once as the job ends, before the window is told: what it changed
    /// is dropped from the caches first, so a read after the exit is fresh.
    on_end: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}

impl Job {
    fn send(&self, frame: Value) {
        if let Some(owner) = lock(&self.owner).as_ref() {
            owner.send(frame);
        }
    }

    fn frame(&self, kind: &str) -> Value {
        json!({"t": kind, "cid": self.cid, "job": self.id})
    }

    fn push(&self, fd: &'static str, text: String) {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let mut frame = self.frame("stream");
        frame["seq"] = json!(seq);
        frame["fd"] = json!(fd);
        frame["line"] = json!(text);
        {
            let mut log = lock(&self.log);
            if log.len() >= LOG_CAP {
                log.pop_front();
            }
            log.push_back(Line { seq, fd, text });
        }
        self.send(frame);
    }

    fn exit_frame(&self) -> Option<Value> {
        match &*lock(&self.state) {
            State::Done { code, summary } => {
                let mut frame = self.frame("job.exit");
                frame["code"] = json!(code);
                frame["summary"] = summary.clone();
                Some(frame)
            }
            _ => None,
        }
    }

    fn describe(&self) -> Value {
        let (state, code) = match &*lock(&self.state) {
            State::Queued => ("queued", None),
            State::Running => ("running", None),
            State::Done { code, .. } => ("done", Some(*code)),
        };
        json!({
            "job": self.id,
            "cid": self.cid,
            "name": self.name,
            "label": self.label,
            "state": state,
            "code": code,
            "startedAt": self.started_at,
            "lines": self.next_seq.load(Ordering::SeqCst),
        })
    }
}

#[derive(Default)]
pub struct Jobs {
    pub runner: Runner,
    next: AtomicU64,
    table: Mutex<HashMap<u64, Arc<Job>>>,
    finished: Mutex<VecDeque<u64>>,
}

impl Jobs {
    pub fn new(runner: Runner) -> Jobs {
        Jobs {
            runner,
            ..Jobs::default()
        }
    }

    /// Queue a job on its lane. It answers `job.started` at once, then streams.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        self: &Arc<Self>,
        commands: &Arc<Commands>,
        owner: &Arc<Subscriber>,
        cid: String,
        label: String,
        name: &str,
        spec: Spec,
        env: Vec<(String, String)>,
        stdin: Option<String>,
    ) -> Arc<Job> {
        let job = Arc::new(Job {
            id: self.next.fetch_add(1, Ordering::SeqCst) + 1,
            cid,
            name: name.to_string(),
            started_at: now_ms(),
            state: Mutex::new(State::Queued),
            log: Mutex::new(VecDeque::new()),
            next_seq: AtomicU64::new(0),
            stdin: Mutex::new(None),
            pid: Mutex::new(None),
            unit: Mutex::new(None),
            cancelled: std::sync::atomic::AtomicBool::new(false),
            label,
            owner: Mutex::new(Some(Arc::clone(owner))),
            on_end: Mutex::new(None),
        });
        lock(&self.table).insert(job.id, Arc::clone(&job));
        job.send(job.frame("job.started"));
        let (jobs, running) = (Arc::clone(self), Arc::clone(&job));
        // The lane orders the job against others. The topics it changes are not
        // on the barrier: a job may run for minutes, and holding every read of
        // the settings for that long would stall the window. They are dropped
        // from the caches as it ends.
        let (bus, affects) = (Arc::clone(commands), spec.affects.clone());
        *lock(&job.on_end) = Some(Box::new(move || bus.invalidate(&affects)));
        let lane = spec.lane.clone();
        let _ = commands.submit(&lane, None, &[], move || {
            run(&running, &spec, env, stdin, jobs.runner);
            jobs.finish(&running);
            json!({"ok": true})
        });
        job
    }

    fn finish(&self, job: &Job) {
        let mut finished = lock(&self.finished);
        finished.push_back(job.id);
        while finished.len() > KEEP_FINISHED {
            if let Some(old) = finished.pop_front() {
                lock(&self.table).remove(&old);
            }
        }
    }

    fn get(&self, id: u64) -> Option<Arc<Job>> {
        lock(&self.table).get(&id).cloned()
    }

    /// Jobs not yet finished.
    pub fn running(&self) -> usize {
        lock(&self.table)
            .values()
            .filter(|job| !matches!(*lock(&job.state), State::Done { .. }))
            .count()
    }

    pub fn list(&self) -> Value {
        let mut jobs: Vec<Arc<Job>> = lock(&self.table).values().cloned().collect();
        jobs.sort_by_key(|job| job.id);
        Value::Array(jobs.iter().map(|job| job.describe()).collect())
    }

    /// Write to a job's stdin, and close it when asked.
    pub fn input(&self, id: u64, data: &str, close: bool) -> bool {
        let Some(job) = self.get(id) else {
            return false;
        };
        let mut slot = lock(&job.stdin);
        let Some(stdin) = slot.as_mut() else {
            return false;
        };
        let ok = stdin
            .write_all(data.as_bytes())
            .and_then(|()| stdin.flush())
            .is_ok();
        if close || !ok {
            *slot = None;
        }
        ok
    }

    /// Ask a job to stop: its unit is stopped, or its process group gets SIGTERM.
    pub fn cancel(&self, id: u64) -> bool {
        let Some(job) = self.get(id) else {
            return false;
        };
        job.cancelled.store(true, Ordering::SeqCst);
        let stopped = if let Some(unit) = lock(&job.unit).clone() {
            crate::runner::Run::new("systemctl")
                .args([
                    "--user",
                    "kill",
                    "--kill-whom=all",
                    "--signal=SIGTERM",
                    &unit,
                ])
                .output()
                .is_ok_and(|out| out.success())
        } else if let Some(pid) = *lock(&job.pid) {
            crate::runner::Run::new("kill")
                .args(["-TERM", "--", &format!("-{pid}")])
                .output()
                .is_ok_and(|out| out.success())
        } else {
            false
        };
        if !stopped {
            job.cancelled.store(false, Ordering::SeqCst);
        }
        stopped
    }

    /// Point a job at a window and send it what it missed since `from`.
    pub fn attach(&self, id: u64, from: u64, to: &Arc<Subscriber>) -> bool {
        let Some(job) = self.get(id) else {
            return false;
        };
        *lock(&job.owner) = Some(Arc::clone(to));
        let missed: Vec<Value> = lock(&job.log)
            .iter()
            .filter(|line| line.seq >= from)
            .map(|line| {
                let mut frame = job.frame("stream");
                frame["seq"] = json!(line.seq);
                frame["fd"] = json!(line.fd);
                frame["line"] = json!(line.text);
                frame
            })
            .collect();
        to.send(job.frame("job.started"));
        for frame in missed {
            to.send(frame);
        }
        if let Some(frame) = job.exit_frame() {
            to.send(frame);
        }
        true
    }

    /// A window going away leaves its jobs running, with nobody listening.
    pub fn forget_owner(&self, owner: &Subscriber) {
        for job in lock(&self.table).values() {
            let mut slot = lock(&job.owner);
            if slot.as_ref().is_some_and(|o| o.id() == owner.id()) {
                *slot = None;
            }
        }
    }
}

/// Jobs a daemon that is gone left running, from `systemctl --user list-units
/// 'atmos-job-*'`: unit name and description, the daemon pid in the name.
fn parse_orphans(listing: &str, own_pid: u32) -> Vec<(String, String)> {
    listing
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let unit = words.next()?;
            let owner = unit
                .strip_prefix("atmos-job-")?
                .split('-')
                .next()?
                .parse::<u32>()
                .ok()?;
            if owner == own_pid {
                return None;
            }
            // load, active, sub, then the description.
            let description: Vec<&str> = words.skip(3).collect();
            Some((unit.to_string(), description.join(" ")))
        })
        .collect()
}

impl Jobs {
    /// Take in the units of jobs an earlier daemon started and did not see end.
    /// Their output went with that daemon, so each is listed and can be
    /// cancelled, and is reported as lost when it ends: never silently dropped.
    pub fn adopt_orphans(self: &Arc<Self>) {
        if self.runner != Runner::Systemd {
            return;
        }
        let Ok(out) = crate::runner::Run::new("systemctl")
            .args([
                "--user",
                "list-units",
                "atmos-job-*",
                "--no-legend",
                "--plain",
                "--state=active,activating",
            ])
            .output()
        else {
            return;
        };
        for (unit, description) in parse_orphans(&out.stdout_text(), std::process::id()) {
            let job = Arc::new(Job {
                id: self.next.fetch_add(1, Ordering::SeqCst) + 1,
                cid: String::new(),
                name: description.trim_start_matches("Atmos job ").to_string(),
                started_at: now_ms(),
                state: Mutex::new(State::Running),
                log: Mutex::new(VecDeque::new()),
                next_seq: AtomicU64::new(0),
                stdin: Mutex::new(None),
                pid: Mutex::new(None),
                unit: Mutex::new(Some(unit.clone())),
                cancelled: std::sync::atomic::AtomicBool::new(false),
                label: "orphaned".into(),
                owner: Mutex::new(None),
                on_end: Mutex::new(None),
            });
            lock(&self.table).insert(job.id, Arc::clone(&job));
            let jobs = Arc::clone(self);
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let active = crate::runner::Run::new("systemctl")
                        .args(["--user", "is-active", "--quiet", &unit])
                        .output()
                        .is_ok_and(|o| o.success());
                    if !active {
                        break;
                    }
                }
                end(
                    &job,
                    if job.cancelled.load(Ordering::SeqCst) {
                        143
                    } else {
                        -1
                    },
                    &[
                        "The daemon restarted while this job ran, so its output was lost"
                            .to_string(),
                    ],
                );
                jobs.finish(&job);
            });
        }
    }
}

fn run(
    job: &Arc<Job>,
    spec: &Spec,
    env: Vec<(String, String)>,
    stdin: Option<String>,
    runner: Runner,
) {
    let argv = match runner {
        Runner::Direct => spec.argv.clone(),
        Runner::Systemd => {
            let unit = format!("atmos-job-{}-{}", std::process::id(), job.id);
            *lock(&job.unit) = Some(format!("{unit}.service"));
            unit_argv(
                &unit,
                &format!("Atmos job {}", job.name),
                &spec.argv,
                &env,
                true,
            )
        }
    };
    let (program, rest) = match argv.split_first() {
        Some(split) => split,
        None => return end(job, 127, &["no command".to_string()]),
    };
    let spawned = Command::new(program)
        .args(rest)
        .env_clear()
        .envs(env)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Its own process group, so a cancel reaches what it started too.
        .process_group(0)
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => return end(job, 127, &[format!("{program}: {err}")]),
    };
    *lock(&job.pid) = Some(child.id());
    *lock(&job.state) = State::Running;
    if let Some(mut pipe) = child.stdin.take() {
        let wrote = match &stdin {
            Some(text) => pipe.write_all(text.as_bytes()).and_then(|()| pipe.flush()),
            None => Ok(()),
        };
        // Closed after the first write unless the job reads more, or a job that
        // reads its input to the end never gets one.
        if wrote.is_ok() && spec.keep_stdin {
            *lock(&job.stdin) = Some(pipe);
        }
    }
    let out = child.stdout.take().map(|pipe| {
        let job = Arc::clone(job);
        std::thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(|l| l.ok()) {
                job.push("out", line);
            }
        })
    });
    let err = child.stderr.take().map(|pipe| {
        let job = Arc::clone(job);
        std::thread::spawn(move || {
            let mut seen = Vec::new();
            for line in BufReader::new(pipe).lines().map_while(|l| l.ok()) {
                job.push("err", line.clone());
                seen.push(line);
            }
            seen
        })
    });
    let code = child
        .wait()
        .map(|status| status.code().unwrap_or(1))
        .unwrap_or(1);
    *lock(&job.stdin) = None;
    if let Some(reader) = out {
        let _ = reader.join();
    }
    let seen = err
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default();
    // Stopping a unit makes its client report success; a stopped job says it was.
    if job.cancelled.load(Ordering::SeqCst) {
        let mut said = seen;
        said.push("cancelled".into());
        return end(job, 143, &said);
    }
    end(job, code, &seen);
}

fn end(job: &Job, code: i32, stderr: &[String]) {
    if let Some(done) = lock(&job.on_end).take() {
        done();
    }
    let (message, warnings) = crate::apply::classify(stderr);
    let summary = json!({"ok": code == 0, "code": code, "message": message, "warnings": warnings});
    *lock(&job.state) = State::Done { code, summary };
    if let Some(frame) = job.exit_frame() {
        job.send(frame);
    }
}
#[cfg(test)]
#[path = "jobs_tests.rs"]
mod tests;
