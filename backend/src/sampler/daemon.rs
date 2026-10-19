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
}

impl Config {
    pub fn production() -> Config {
        Config {
            floor: schedule::DIRTY_FLOOR,
            limits: Limits::default(),
            probe: Box::new(governor::ProcProbe),
            job_timeout: Duration::from_secs(120),
            run_command: Box::new(crate::command::run_child),
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
    pub(super) run_command: Arc<RunCommand>,
    request_frames: std::sync::atomic::AtomicU64,
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
        // A finished command or lease drops the display kinds it touched, so
        // the next read collects them again.
        let invalidate = dir.clone();
        let commands = Commands::new(Box::new(move |affects| {
            let kinds: Vec<&str> = affects
                .iter()
                .map(String::as_str)
                .filter(|topic| crate::display::is_kind(topic))
                .collect();
            if !kinds.is_empty() {
                cache::drop_kinds(&invalidate, &kinds);
            }
        }));
        Arc::new(Daemon {
            dir,
            flights: Flights::default(),
            collect,
            workers: Arc::new(Workers::default()),
            topics,
            bus: Bus::default(),
            commands,
            run_command: Arc::from(config.run_command),
            request_frames: std::sync::atomic::AtomicU64::new(0),
            governor: Mutex::new(Governor::new(config.limits)),
            probe: config.probe,
            floor: config.floor,
            job_timeout: config.job_timeout,
            last: Mutex::new(HashMap::new()),
        })
    }

    fn last_sample(&self, kind: &str) -> Option<Instant> {
        self.last.lock().ok()?.get(kind).copied()
    }

    /// Collect one kind and cache it. Interactive runs in this process; a
    /// background run goes to the sampler. A failure marks the topic stale and
    /// leaves the last good sample where it is.
    pub fn sample(&self, kind: &str, interactive: bool) -> Outcome {
        if !crate::display::is_kind(kind) {
            return Err(format!("unknown display kind {kind}"));
        }
        let outcome = self.flights.run(kind, interactive, || {
            let at_ms = cache::now_ms();
            let began = Instant::now();
            let value = if interactive {
                (self.collect)(kind)?
            } else {
                self.collect_in_background(kind)?
            };
            let sample = Sample {
                at_ms,
                took_ms: began.elapsed().as_millis() as u64,
                value,
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
        if let Err(message) = &outcome {
            self.topics.failure(kind, message);
            self.bus.fail(kind, message);
        }
        outcome
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

    fn status(&self) -> Value {
        json!({
            "ok": true,
            "version": crate::VERSION,
            "pid": std::process::id(),
            "sampler": {"connected": self.workers.available(), "pid": self.workers.pid()},
            "topics": self.topics.status(),
            "sessions": self.bus.count(),
            "requestFrames": self.request_frames.load(Ordering::SeqCst),
        })
    }

    pub(super) fn answer(&self, request: &Value) -> Value {
        match request.get("op").and_then(Value::as_str) {
            Some("ping") => {
                json!({"ok": true, "version": crate::VERSION, "pid": std::process::id()})
            }
            Some("status") => self.status(),
            Some("command") => self.command_op(request),
            Some("barrier") => self.barrier_op(request),
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
    pub fn schedule(&self, stop: &AtomicBool) {
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
            sampler.look(self.workers.available(), now);
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

/// Bind the socket, holding the single-instance lock. The returned file keeps
/// the lock for as long as the daemon lives.
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
    let socket = cache::socket(dir);
    // The lock is ours, so a socket file here is a leftover.
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).map_err(|err| Error::io(err, &socket))?;
    let _ = std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600));
    Ok((listener, lock))
}

/// `ratmos daemon`
pub fn run() -> Result<i32> {
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
    super::session::exit_when_stale(watched);
    // Only a signal ends the daemon, and the lock goes with the process.
    daemon.schedule(&AtomicBool::new(false));
    Ok(0)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::sampler::cache::tests::scratch;
    use crate::sampler::worker::serve_jobs;
    use std::sync::atomic::AtomicUsize;

    /// A pressure dial a test can turn.
    pub struct Dial(pub AtomicBool);
    impl Probe for Arc<Dial> {
        fn pressure(&self) -> Option<f64> {
            Some(if self.0.load(Ordering::SeqCst) {
                50.0
            } else {
                0.0
            })
        }
        fn load_per_core(&self) -> Option<f64> {
            None
        }
    }

    pub fn test_config(dial: &Arc<Dial>) -> Config {
        Config {
            floor: Duration::ZERO,
            limits: Limits {
                duty: 1000.0,
                backoff_min: Duration::from_millis(100),
                backoff_max: Duration::from_millis(200),
                ..Limits::default()
            },
            probe: Box::new(dial.clone()),
            job_timeout: Duration::from_secs(5),
            run_command: Box::new(|_, body, _| json!({"ok": true, "result": {"ran": body}})),
        }
    }

    /// A daemon over a real socket. Its foreground collector counts calls and
    /// says it ran in the daemon.
    pub fn serve(name: &str, delay: Duration) -> (PathBuf, Arc<Daemon>, Arc<AtomicUsize>) {
        let (dir, daemon, calls, _) = serve_with_dial(name, delay);
        (dir, daemon, calls)
    }

    pub fn serve_with_dial(
        name: &str,
        delay: Duration,
    ) -> (PathBuf, Arc<Daemon>, Arc<AtomicUsize>, Arc<Dial>) {
        let dir = scratch(name);
        let (listener, lock) = bind(&dir).unwrap();
        std::mem::forget(lock);
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let dial = Arc::new(Dial(AtomicBool::new(false)));
        let daemon = Daemon::new(
            dir.clone(),
            Box::new(move |kind| {
                let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                std::thread::sleep(delay);
                Ok(json!({"kind": kind, "n": n, "from": "daemon"}))
            }),
            test_config(&dial),
        );
        let lane = daemon.clone();
        std::thread::spawn(move || lane.accept(listener));
        let pump = daemon.clone();
        std::thread::spawn(move || pump.pump(&AtomicBool::new(false)));
        (dir, daemon, calls, dial)
    }

    /// A daemon whose commands run `run` instead of a child process.
    pub fn serve_commands(name: &str, run: Box<RunCommand>) -> (PathBuf, Arc<Daemon>) {
        let dir = scratch(name);
        let (listener, lock) = bind(&dir).unwrap();
        std::mem::forget(lock);
        let dial = Arc::new(Dial(AtomicBool::new(false)));
        let mut config = test_config(&dial);
        config.run_command = run;
        let daemon = Daemon::new(
            dir.clone(),
            Box::new(|kind| Ok(json!({"kind": kind}))),
            config,
        );
        let lane = daemon.clone();
        std::thread::spawn(move || lane.accept(listener));
        (dir, daemon)
    }

    /// Connect a sampler process stand-in and wait until the daemon sees it.
    pub fn attach_worker(
        daemon: &Arc<Daemon>,
        dir: &Path,
        collect: impl Fn(&str) -> std::result::Result<Value, String> + Send + 'static,
    ) {
        let stream = UnixStream::connect(cache::socket(dir)).unwrap();
        std::thread::spawn(move || {
            let _ = serve_jobs(stream, &collect);
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while !daemon.workers.available() {
            assert!(Instant::now() < deadline, "the worker never attached");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn worker_doc(kind: &str) -> std::result::Result<Value, String> {
        Ok(json!({"kind": kind, "from": "worker"}))
    }

    fn run_schedule(daemon: &Arc<Daemon>) -> (Arc<AtomicBool>, std::thread::JoinHandle<()>) {
        let stop = Arc::new(AtomicBool::new(false));
        let (lane, flag) = (daemon.clone(), stop.clone());
        (stop, std::thread::spawn(move || lane.schedule(&flag)))
    }

    fn finish(stop: Arc<AtomicBool>, handle: std::thread::JoinHandle<()>) {
        stop.store(true, Ordering::SeqCst);
        handle.join().unwrap();
    }

    fn wait_until(what: &str, secs: u64, mut done: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(secs);
        while !done() {
            assert!(Instant::now() < deadline, "timed out: {what}");
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    #[test]
    fn a_second_daemon_cannot_start() {
        let dir = scratch("single");
        let first = bind(&dir).unwrap();
        let err = bind(&dir).expect_err("second bind should fail");
        assert!(err.to_string().contains("already running"), "{err}");
        drop(first);
        // A child another test forks shares the lock until it execs, so the release can lag.
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match bind(&dir) {
                Ok(_) => break,
                Err(err) if Instant::now() < deadline => {
                    assert!(err.to_string().contains("already running"), "{err}");
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(err) => panic!("the lock goes with the first daemon: {err}"),
            }
        }
    }

    #[test]
    fn the_socket_is_private() {
        let dir = scratch("mode");
        let _held = bind(&dir).unwrap();
        let mode = std::fs::metadata(cache::socket(&dir))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn an_interactive_sample_runs_in_the_daemon_and_is_cached() {
        let (dir, daemon, _) = serve("caches", Duration::ZERO);
        let sample = daemon.sample("services", true).unwrap();
        assert_eq!(sample.value["from"], "daemon");
        assert_eq!(cache::read(&dir, "services").unwrap().value, sample.value);
        assert_eq!(daemon.topics.get("services").seq, 1);
    }

    #[test]
    fn an_unknown_kind_is_refused() {
        let (_, daemon, calls) = serve("unknown", Duration::ZERO);
        assert!(daemon.sample("../x", true).is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn with_no_sampler_nothing_is_sampled_in_the_background() {
        let (dir, daemon, calls) = serve("noworker", Duration::ZERO);
        let (stop, handle) = run_schedule(&daemon);
        std::thread::sleep(Duration::from_millis(1500));
        finish(stop, handle);
        assert!(
            cache::read(&dir, "live").is_none(),
            "live was due at once and still not sampled"
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "the daemon did no background work itself"
        );
        assert!(
            daemon.sample("live", false).is_err(),
            "a background sample needs a sampler"
        );
    }

    #[test]
    fn with_a_sampler_the_schedule_collects_in_the_sampler_and_stops() {
        let (dir, daemon, calls) = serve("worker", Duration::ZERO);
        attach_worker(&daemon, &dir, worker_doc);
        let (stop, handle) = run_schedule(&daemon);
        wait_until("live was not sampled", 5, || {
            cache::read(&dir, "live").is_some()
        });
        finish(stop, handle);
        assert_eq!(cache::read(&dir, "live").unwrap().value["from"], "worker");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "the daemon's own collector was not used"
        );
    }

    #[test]
    fn an_interactive_request_is_answered_in_the_daemon_even_with_a_sampler_attached() {
        let (dir, daemon, _) = serve("fg", Duration::ZERO);
        attach_worker(&daemon, &dir, worker_doc);
        assert_eq!(
            daemon.sample("disks", true).unwrap().value["from"],
            "daemon"
        );
    }

    #[test]
    fn a_slow_background_run_never_holds_up_an_interactive_one() {
        let (dir, daemon, _) = serve("yield", Duration::ZERO);
        attach_worker(&daemon, &dir, |kind| {
            std::thread::sleep(Duration::from_millis(1500));
            worker_doc(kind)
        });
        let (stop, handle) = run_schedule(&daemon);
        // Live is due at once, so the sampler is busy with it.
        std::thread::sleep(Duration::from_millis(300));
        let began = Instant::now();
        let sample = daemon.sample("live", true).unwrap();
        assert!(
            began.elapsed() < Duration::from_millis(600),
            "{:?}",
            began.elapsed()
        );
        assert_eq!(
            sample.value["from"], "daemon",
            "the reader did not join the sampler's run"
        );
        stop.store(true, Ordering::SeqCst);
        handle.join().unwrap();
        assert!(cache::read(&dir, "live").is_some());
    }

    #[test]
    fn a_failed_background_run_marks_the_topic_stale_and_keeps_the_last_good_sample() {
        let (dir, daemon, _) = serve("stale", Duration::ZERO);
        let first = daemon.sample("services", true).unwrap();
        attach_worker(&daemon, &dir, |kind| Err(format!("{kind} hung")));
        let err = daemon.sample("services", false).unwrap_err();
        assert_eq!(err, "services hung");
        let topic = daemon.topics.get("services");
        assert!(topic.stale);
        assert_eq!(topic.error.as_deref(), Some("services hung"));
        assert_eq!(topic.seq, 1, "the failure is not a collection");
        assert_eq!(
            cache::read(&dir, "services").unwrap().value,
            first.value,
            "the good sample is still what readers get"
        );
    }

    #[test]
    fn a_sampler_that_dies_mid_run_is_a_failure_and_the_daemon_carries_on() {
        let (dir, daemon, _) = serve("sampler-dies", Duration::ZERO);
        let stream = UnixStream::connect(cache::socket(&dir)).unwrap();
        let cut = stream.try_clone().unwrap();
        std::thread::spawn(move || {
            let _ = serve_jobs(stream, &|kind| {
                std::thread::sleep(Duration::from_millis(500));
                worker_doc(kind)
            });
        });
        wait_until("the worker attached", 3, || daemon.workers.available());
        let asker = {
            let daemon = daemon.clone();
            std::thread::spawn(move || daemon.sample("disks", false))
        };
        std::thread::sleep(Duration::from_millis(100));
        cut.shutdown(std::net::Shutdown::Both).unwrap();
        assert!(asker.join().unwrap().unwrap_err().contains("went away"));
        assert!(daemon.topics.get("disks").stale);
        assert!(
            daemon.sample("disks", true).is_ok(),
            "readers are still served"
        );
        wait_until("nobody is current", 3, || !daemon.workers.available());
        attach_worker(&daemon, &dir, worker_doc);
        assert!(
            daemon.sample("disks", false).is_ok(),
            "a new sampler takes over"
        );
        assert!(!daemon.topics.get("disks").stale, "and the topic heals");
    }

    #[test]
    fn pressure_holds_background_sampling_and_calm_releases_it() {
        let (dir, daemon, _, dial) = serve_with_dial("pressure", Duration::ZERO);
        attach_worker(&daemon, &dir, worker_doc);
        dial.0.store(true, Ordering::SeqCst);
        let (stop, handle) = run_schedule(&daemon);
        std::thread::sleep(Duration::from_millis(1500));
        assert!(
            cache::read(&dir, "live").is_none(),
            "a busy machine is not sampled"
        );
        dial.0.store(false, Ordering::SeqCst);
        wait_until("live was not sampled once it calmed", 5, || {
            cache::read(&dir, "live").is_some()
        });
        finish(stop, handle);
    }

    #[test]
    fn a_dropped_kind_is_resampled_in_the_background_without_a_reader() {
        let (dir, daemon, _) = serve("redo", Duration::ZERO);
        attach_worker(&daemon, &dir, worker_doc);
        // Everything is cached and not yet due, so only the drop can wake a kind.
        for kind in crate::display::KINDS {
            daemon.sample(kind, true).unwrap();
        }
        let (stop, handle) = run_schedule(&daemon);
        std::thread::sleep(Duration::from_millis(50));
        cache::drop_kinds(&dir, &["services"]);
        wait_until("services was not resampled", 9, || {
            cache::read(&dir, "services").is_some_and(|s| s.value["from"] == "worker")
        });
        finish(stop, handle);
    }

    #[test]
    fn the_sampler_watch_speaks_once_when_it_is_missing_and_once_when_it_returns() {
        let mut watch = SamplerWatch::default();
        let start = Instant::now();
        watch.look(false, start);
        assert!(
            !watch.told_off,
            "a sampler normally takes a few seconds to connect"
        );
        watch.look(false, start + SamplerWatch::GRACE);
        assert!(watch.told_off);
        watch.look(false, start + SamplerWatch::GRACE * 2);
        assert!(watch.told_off, "and it does not repeat itself");
        watch.look(true, start + SamplerWatch::GRACE * 3);
        assert!(
            !watch.told_off && watch.absent_since.is_none(),
            "the next absence starts over"
        );
    }

    #[test]
    fn status_reports_the_sampler_and_every_topic() {
        let (dir, daemon, _) = serve("status", Duration::ZERO);
        let before = daemon.answer(&json!({"op": "status"}));
        assert_eq!(before["sampler"]["connected"], false);
        attach_worker(&daemon, &dir, worker_doc);
        daemon.sample("live", true).unwrap();
        let after = daemon.answer(&json!({"op": "status"}));
        assert_eq!(after["sampler"]["connected"], true);
        assert_eq!(after["topics"]["live"]["seq"], 1);
        assert_eq!(after["topics"]["hardware"]["seq"], 0);
        assert_eq!(after["version"], crate::VERSION);
    }

    #[test]
    fn a_restarted_daemon_knows_what_the_cache_already_holds() {
        let (dir, daemon, _) = serve("seed", Duration::ZERO);
        daemon.sample("software", true).unwrap();
        let again = Daemon::new(
            dir.clone(),
            Box::new(|_| Err("unused".into())),
            test_config(&Arc::new(Dial(AtomicBool::new(false)))),
        );
        let topic = again.topics.get("software");
        assert_eq!(topic.seq, 0);
        assert!(topic.sampled_at.is_some());
    }
}
