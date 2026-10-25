//! `ratmos daemon`: owns the topics, answers readers that want a kind now, and
//! schedules background sampling.
//!
//! Foreground work (a reader's miss) runs in this process at normal priority,
//! one thread per connection, so a window never waits behind a scheduled run.
//! Background work goes only to the sampler process (see `worker`), and only
//! when the governor allows it: with no sampler connected, nothing is sampled
//! in the background at all.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::bus::Bus;
use super::cache::{self, Sample};
use super::commands::Commands;
use super::flight::{Flights, Outcome};
use super::governor::{self, Governor, Limits, Probe, Verdict};
use super::owned::OwnedSampler;
use super::schedule::{self, Schedule};
use super::topics::Topics;
use super::worker::Workers;
use crate::error::{Error, Result};

pub type Collect = dyn Fn(&str) -> std::result::Result<Value, String> + Send + Sync;

/// Runs one command: the backend id, the request body and the caller's
/// environment in, the envelope out.
pub type RunCommand = dyn Fn(&str, &Value, &[(String, String)]) -> Value + Send + Sync;

pub struct Config {
    /// The least time between a kind's samples when a mutation drops it.
    pub floor: Duration,
    pub limits: Limits,
    pub probe: Box<dyn Probe + Send + Sync>,
    /// How long a background collection may take before it is given up on.
    pub job_timeout: Duration,
    pub run_command: Box<RunCommand>,
    pub job_runner: super::jobs::Runner,
}

impl Config {
    pub fn production() -> Config {
        Config {
            floor: schedule::DIRTY_FLOOR,
            limits: Limits::default(),
            probe: Box::new(governor::ProcProbe),
            job_timeout: Duration::from_secs(120),
            run_command: Box::new(crate::command::run_child),
            job_runner: super::jobs::Runner::detect(),
        }
    }
}

pub struct Daemon {
    pub(super) dir: PathBuf,
    flights: Flights,
    collect: Box<Collect>,
    workers: Arc<Workers>,
    topics: Topics,
    pub(super) bus: Bus,
    pub(super) commands: Arc<Commands>,
    pub(super) watches: Arc<super::watch::Watches>,
    pub(super) jobs: Arc<super::jobs::Jobs>,
    pub(super) run_command: Arc<RunCommand>,
    pub(super) settings: super::settings::Settings,
    request_frames: std::sync::atomic::AtomicU64,
    /// Requests a window's `serve` has handed to the daemon to run.
    pub(super) commands_run: std::sync::atomic::AtomicU64,
    pub(super) read_slots: Arc<super::ops::Slots>,
    governor: Mutex<Governor>,
    probe: Box<dyn Probe + Send + Sync>,
    floor: Duration,
    job_timeout: Duration,
    last: Mutex<HashMap<String, Instant>>,
}

impl Daemon {
    /// `collect` is the foreground collector, run in this process.
    pub fn new(dir: PathBuf, collect: Box<Collect>, config: Config) -> Arc<Daemon> {
        let topics = Topics::default();
        for kind in crate::display::KINDS {
            if let Some(sample) = cache::read(&dir, kind) {
                topics.seed(kind, &sample);
            }
        }
        // What a settings topic held when the last daemon left says nothing
        // about the machine now.
        cache::drop_kinds(&dir, crate::snapshot::topics::NAMES);
        let settings = super::settings::Settings::default();
        for topic in crate::snapshot::topics::NAMES {
            settings.stamp(topic);
        }
        Arc::new_cyclic(|me: &std::sync::Weak<Daemon>| {
            // A finished command or lease drops the display kinds it touched,
            // so the next read collects them again, and invalidates the
            // settings topics it named.
            let (invalidate, daemon) = (dir.clone(), me.clone());
            let commands = Commands::new(Box::new(move |affects| {
                let kinds: Vec<&str> = affects
                    .iter()
                    .map(String::as_str)
                    .filter(|topic| crate::display::is_kind(topic))
                    .collect();
                if !kinds.is_empty() {
                    cache::drop_kinds(&invalidate, &kinds);
                }
                let named: Vec<&str> = affects
                    .iter()
                    .map(String::as_str)
                    .filter(|topic| super::is_settings(topic))
                    .collect();
                if let (false, Some(daemon)) = (named.is_empty(), daemon.upgrade()) {
                    daemon.invalidate(&named);
                }
            }));
            Daemon {
                dir,
                flights: Flights::default(),
                collect,
                workers: Arc::new(Workers::default()),
                topics,
                bus: Bus::default(),
                commands,
                watches: Arc::new(super::watch::Watches::default()),
                jobs: Arc::new(super::jobs::Jobs::new(config.job_runner)),
                run_command: Arc::from(config.run_command),
                settings,
                request_frames: std::sync::atomic::AtomicU64::new(0),
                commands_run: std::sync::atomic::AtomicU64::new(0),
                read_slots: Arc::default(),
                governor: Mutex::new(Governor::new(config.limits)),
                probe: config.probe,
                floor: config.floor,
                job_timeout: config.job_timeout,
                last: Mutex::new(HashMap::new()),
            }
        })
    }

    fn last_sample(&self, kind: &str) -> Option<Instant> {
        self.last.lock().ok()?.get(kind).copied()
    }

    /// Collect one kind and cache it. Interactive runs in this process; a
    /// background run goes to the sampler. A failure marks the topic stale and
    /// leaves the last good sample where it is.
    pub fn sample(&self, kind: &str, interactive: bool) -> Outcome {
        if !super::is_topic(kind) {
            return Err(format!("unknown topic {kind}"));
        }
        let settings = super::is_settings(kind);
        let outcome = self.flights.run(kind, interactive, || {
            let mut tries = 0;
            let sample = loop {
                let generation = self.settings.generation(kind);
                let at_ms = if settings {
                    self.settings.stamp(kind)
                } else {
                    cache::now_ms()
                };
                let began = Instant::now();
                let value = if settings {
                    self.collect_topic(kind)?
                } else if interactive {
                    (self.collect)(kind)?
                } else {
                    self.collect_in_background(kind)?
                };
                // Something invalidated the topic while it was being read: what
                // was read may be from before it, so it is read again.
                if settings
                    && generation != self.settings.generation(kind)
                    && tries < super::settings::RETRIES
                {
                    tries += 1;
                    continue;
                }
                break Sample {
                    at_ms,
                    took_ms: began.elapsed().as_millis() as u64,
                    value,
                };
            };
            if let Err(err) = cache::write(&self.dir, kind, &sample) {
                eprintln!("ratmos daemon: cannot cache {kind}: {err}");
            }
            if let Ok(mut last) = self.last.lock() {
                last.insert(kind.to_string(), Instant::now());
            }
            self.topics.success(kind, &sample);
            self.bus.publish(kind, &sample);
            Ok(sample)
        });
        if settings {
            self.settings.failed(kind, outcome.is_err());
        }
        if let Err(message) = &outcome {
            self.topics.failure(kind, message);
            self.bus.fail(kind, message);
        }
        outcome
    }

    /// Hand a sample to whoever holds the topic, and note it.
    pub(super) fn publish_sample(&self, kind: &str, sample: &Sample) {
        self.topics.success(kind, sample);
        self.bus.publish(kind, sample);
    }

    /// Ask the sampler for `kind` and charge what it cost to the governor.
    fn collect_in_background(&self, kind: &str) -> std::result::Result<Value, String> {
        let pid = self.workers.pid();
        let before = pid.and_then(governor::process_cpu_secs);
        let reply = self.workers.run(kind, self.job_timeout);
        if let (Some(pid), Some(before)) = (pid, before) {
            if let Some(after) = governor::process_cpu_secs(pid) {
                if let Ok(mut governor) = self.governor.lock() {
                    governor.spent(Instant::now(), after - before);
                }
            }
        }
        reply
    }

    /// Counted so `status` can show that windows reach the daemon over sessions.
    pub(super) fn note_request_frame(&self) {
        self.request_frames.fetch_add(1, Ordering::SeqCst);
    }

    pub(super) fn status(&self) -> Value {
        json!({
            "ok": true,
            "version": crate::VERSION,
            "pid": std::process::id(),
            "sampler": {"connected": self.workers.available(), "pid": self.workers.pid()},
            "topics": self.topics.status(),
            "sessions": self.bus.count(),
            "watching": self.watches.windows(),
            "jobsRunning": self.jobs.running(),
            "requestFrames": self.request_frames.load(Ordering::SeqCst),
            "commandsRun": self.commands_run.load(Ordering::SeqCst),
        })
    }

    pub(super) fn answer(self: &Arc<Self>, request: &Value) -> Value {
        match request.get("op").and_then(Value::as_str) {
            Some("ping") => {
                json!({"ok": true, "version": crate::VERSION, "pid": std::process::id()})
            }
            Some("status") => self.status(),
            Some("jobs") => json!({"ok": true, "jobs": self.jobs.list()}),
            Some("command") => self.command_op(request),
            Some("barrier") => self.barrier_op(request),
            Some("invalidate") => {
                // What a refresh button asks: these topics are read again.
                let named: Vec<String> = request
                    .get("topics")
                    .and_then(Value::as_array)
                    .map(|list| {
                        list.iter()
                            .filter_map(|topic| topic.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                let topics: Vec<&str> = if named.is_empty() {
                    crate::snapshot::topics::NAMES.to_vec()
                } else {
                    named.iter().map(String::as_str).collect()
                };
                if let Some(bad) = topics.iter().find(|topic| !super::is_topic(topic)) {
                    return json!({"ok": false, "message": format!("not a topic: {bad}")});
                }
                self.invalidate(&topics);
                json!({"ok": true})
            }
            Some("sample") => {
                let kind = request.get("kind").and_then(Value::as_str).unwrap_or("");
                match self.sample(kind, true) {
                    Ok(sample) => json!({"ok": true, "sample": sample.to_json()}),
                    Err(message) => json!({"ok": false, "message": message}),
                }
            }
            _ => json!({"ok": false, "message": "unknown op"}),
        }
    }

    fn converse(self: &Arc<Self>, stream: UnixStream) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut line = String::new();
        {
            let mut reader = BufReader::new(&stream).take(REQUEST_LIMIT);
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                return;
            }
        }
        let request: Value = match serde_json::from_str(line.trim()) {
            Ok(request) => request,
            Err(err) => {
                let mut out = &stream;
                let _ = writeln!(
                    out,
                    "{}",
                    json!({"ok": false, "message": format!("bad request: {err}")})
                );
                return;
            }
        };
        if request.get("op").and_then(Value::as_str) == Some("sampler.hello") {
            let pid = request.get("pid").and_then(Value::as_u64).unwrap_or(0) as u32;
            let mut out = &stream;
            if writeln!(out, "{}", json!({"ok": true})).is_ok() {
                self.workers.attach(stream, pid);
            }
            return;
        }
        if request.get("op").and_then(Value::as_str) == Some("lease") {
            super::ops::hold_lease(self, stream, &request);
            return;
        }
        if request.get("op").and_then(Value::as_str) == Some("session") {
            super::session::open(self, stream);
            return;
        }
        let reply = self.answer(&request);
        let mut out = &stream;
        let _ = writeln!(out, "{reply}");
    }

    /// The interactive lane: one thread per connection until `listener` closes.
    pub fn accept(self: &Arc<Self>, listener: UnixListener) {
        for stream in listener.incoming().flatten() {
            let daemon = self.clone();
            std::thread::spawn(move || daemon.converse(stream));
        }
    }

    /// The scheduled lane. Returns when `stop` is set.
    #[cfg(test)]
    pub fn schedule(&self, stop: &AtomicBool) {
        self.schedule_with(stop, None);
    }

    /// [`Daemon::schedule`], and keeping a sampler of its own alive when there is a keeper
    /// (`ratmos daemon --with-sampler`).
    pub(super) fn schedule_with(&self, stop: &AtomicBool, mut keeper: Option<OwnedSampler>) {
        let mut schedule = Schedule::new(Instant::now(), |kind| {
            cache::read(&self.dir, kind).map(|sample| sample.age())
        });
        let mut held_until = Instant::now();
        let mut told = "";
        let mut sampler = SamplerWatch::default();
        while !stop.load(Ordering::SeqCst) {
            let (kind, due) = schedule.next();
            let now = Instant::now();
            // A kind dropped by a mutation is resampled soon, not at its interval.
            for known in crate::display::KINDS {
                if cache::is_dirty(&self.dir, known) {
                    let due = schedule::dirty_due(now, self.last_sample(known), self.floor);
                    schedule.bring_forward(known, due);
                }
            }
            match keeper.as_mut() {
                // The daemon's own: it says so when it starts one, so no "sampling is off".
                Some(keeper) => keeper.keep(self.workers.available(), now),
                None => sampler.look(self.workers.available(), now),
            }
            let ready = due <= now
                && now >= held_until
                && self.flights.interactive_active() == 0
                && self.workers.idle();
            if !ready {
                let wait = if due > now { due - now } else { WAKE };
                std::thread::sleep(wait.min(WAKE));
                continue;
            }
            let verdict = match self.governor.lock() {
                Ok(mut governor) => governor.verdict(now, &*self.probe),
                Err(_) => Verdict::Go,
            };
            if let Verdict::Wait { wait, why } = verdict {
                held_until = now + wait;
                if why != told {
                    eprintln!("ratmos daemon: background sampling waits: {why}");
                    told = why;
                }
                continue;
            }
            told = "";
            if let Err(message) = self.sample(kind, false) {
                eprintln!("ratmos daemon: {kind}: {message}");
            }
            schedule.finished(kind, Instant::now(), schedule::jitter());
        }
    }
}

/// Says so, once, when background sampling is off for want of a sampler, and
/// again when one arrives. Without it a hand-started daemon just idles.
#[derive(Default)]
struct SamplerWatch {
    absent_since: Option<Instant>,
    told_off: bool,
}

impl SamplerWatch {
    /// A sampler normally connects seconds after the daemon starts.
    const GRACE: Duration = Duration::from_secs(10);

    fn look(&mut self, connected: bool, now: Instant) {
        if connected {
            if self.told_off {
                eprintln!("ratmos daemon: a sampler connected; background sampling is on");
            }
            *self = SamplerWatch::default();
            return;
        }
        let since = *self.absent_since.get_or_insert(now);
        if !self.told_off && now.duration_since(since) >= Self::GRACE {
            eprintln!("ratmos daemon: background sampling is off: no sampler is connected (run `ratmos sampler`, or enable ratmos-sampler.service)");
            self.told_off = true;
        }
    }
}

/// The longest request line a connection may send. A command carries the
/// caller's environment, which is a few kilobytes.
const REQUEST_LIMIT: u64 = 512 * 1024;

/// How often the scheduled lane looks up from sleeping.
const WAKE: Duration = Duration::from_millis(500);

/// The listening socket systemd passes to a socket-activated service: the first
/// descriptor after stderr, when `LISTEN_PID` names this process and
/// `LISTEN_FDS` counts at least one. The descriptor is duplicated so the copy
/// is closed on exec (a command the daemon starts must not hold the socket
/// open) and the inherited one is closed.
pub fn inherited_listener() -> Option<UnixListener> {
    use std::os::fd::FromRawFd;
    let for_us = listen_fds(
        std::env::var("LISTEN_PID").ok().as_deref(),
        std::env::var("LISTEN_FDS").ok().as_deref(),
        std::process::id(),
    )?;
    // SAFETY: systemd handed this descriptor to this process, as the two
    // variables say, and nothing else in the process owns it.
    let passed = unsafe { UnixListener::from_raw_fd(for_us) };
    passed.try_clone().ok()
}

/// The first passed descriptor, if the environment says it is for `pid`.
fn listen_fds(listen_pid: Option<&str>, listen_fds: Option<&str>, pid: u32) -> Option<i32> {
    const FIRST: i32 = 3;
    let named: u32 = listen_pid?.trim().parse().ok()?;
    let count: i32 = listen_fds?.trim().parse().ok()?;
    (named == pid && count >= 1).then_some(FIRST)
}

/// Take the socket and the single-instance lock. The socket is the one systemd
/// passed when it started the daemon on a connection, or a new one at the
/// socket path. The returned file keeps the lock for as long as the daemon lives.
pub fn bind(dir: &Path) -> Result<(UnixListener, std::fs::File)> {
    cache::ensure_dir(dir)?;
    let lock_path = dir.join("daemon.lock");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|err| Error::io(err, &lock_path))?;
    lock.try_lock()
        .map_err(|_| Error::from("ratmos daemon is already running"))?;
    if let Some(listener) = inherited_listener() {
        return Ok((listener, lock));
    }
    let socket = cache::socket(dir);
    // The lock is ours, so a socket file here is a leftover.
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).map_err(|err| Error::io(err, &socket))?;
    let _ = std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600));
    Ok((listener, lock))
}

/// `ratmos daemon [--with-sampler]`: with the flag the daemon starts a sampler of its own and
/// keeps it alive (docs/adr/0014), where otherwise `ratmos-sampler.service` or a hand starts one.
pub fn run(with_sampler: bool) -> Result<i32> {
    let dir = cache::dir_from_env().ok_or("ratmos daemon needs XDG_RUNTIME_DIR")?;
    let (listener, _lock) = bind(&dir)?;
    let watched = dir.clone();
    let daemon = Daemon::new(
        dir,
        Box::new(|kind| {
            crate::display::collect(crate::platform::Backend::Omarchy, kind)
                .map_err(|err| err.to_string())
        }),
        Config::production(),
    );
    eprintln!("ratmos daemon {} ready", crate::VERSION);
    let lane = daemon.clone();
    std::thread::spawn(move || lane.accept(listener));
    let pump = daemon.clone();
    std::thread::spawn(move || pump.pump(&AtomicBool::new(false)));
    let watcher = daemon.clone();
    std::thread::spawn(move || watcher.watches.run(&AtomicBool::new(false)));
    let files = daemon.clone();
    std::thread::spawn(move || files.watch_settings(&AtomicBool::new(false)));
    daemon.jobs.adopt_orphans();
    super::session::exit_when_stale(watched, daemon.jobs.clone());
    // Only a signal ends the daemon, and the lock goes with the process.
    let keeper = with_sampler.then(OwnedSampler::production);
    daemon.schedule_with(&AtomicBool::new(false), keeper);
    Ok(0)
}

#[cfg(test)]
#[path = "daemon_tests.rs"]
pub(crate) mod tests;
