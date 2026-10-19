//! `ratmos daemon`: samples every display kind slowly and on repeat, and
//! answers readers that want a kind now.
//!
//! Two lanes share one collector per kind (see `flight`). The scheduled lane
//! is a single thread that walks the schedule and stands aside while any
//! interactive collection runs. The interactive lane is one thread per
//! connection, so a reader never waits behind a scheduled run.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::cache::{self, Sample};
use super::flight::{Flights, Outcome};
use super::schedule::{self, Schedule};
use crate::error::{Error, Result};

pub type Collect = dyn Fn(&str) -> std::result::Result<Value, String> + Send + Sync;

pub struct Daemon {
    dir: PathBuf,
    flights: Flights,
    collect: Box<Collect>,
    floor: Duration,
    last: Mutex<HashMap<String, Instant>>,
}

impl Daemon {
    /// `floor` is the least time between a kind's samples when a mutation drops it.
    pub fn new(dir: PathBuf, collect: Box<Collect>, floor: Duration) -> Arc<Daemon> {
        Arc::new(Daemon {
            dir,
            flights: Flights::default(),
            collect,
            floor,
            last: Mutex::new(HashMap::new()),
        })
    }

    fn last_sample(&self, kind: &str) -> Option<Instant> {
        self.last.lock().ok()?.get(kind).copied()
    }

    /// Collect one kind (or join the run in progress) and cache it.
    pub fn sample(&self, kind: &str, interactive: bool) -> Outcome {
        if !crate::display::is_kind(kind) {
            return Err(format!("unknown display kind {kind}"));
        }
        self.flights.run(kind, interactive, || {
            let at_ms = cache::now_ms();
            let began = Instant::now();
            let value = (self.collect)(kind)?;
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
            Ok(sample)
        })
    }

    fn answer(&self, line: &str) -> Value {
        let request: Value = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(err) => return json!({"ok": false, "message": format!("bad request: {err}")}),
        };
        match request.get("op").and_then(Value::as_str) {
            Some("ping") => {
                json!({"ok": true, "version": crate::VERSION, "pid": std::process::id()})
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

    fn converse(&self, stream: UnixStream) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut line = String::new();
        let mut reader = BufReader::new(&stream).take(4096);
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let reply = self.answer(line.trim());
        let mut stream = &stream;
        let _ = writeln!(stream, "{reply}");
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
            if due > now || self.flights.interactive_active() > 0 {
                std::thread::sleep(due.saturating_duration_since(now).min(WAKE));
                continue;
            }
            if let Err(message) = self.sample(kind, false) {
                eprintln!("ratmos daemon: {kind}: {message}");
            }
            schedule.finished(kind, Instant::now(), schedule::jitter());
        }
    }
}

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
    let daemon = Daemon::new(
        dir,
        Box::new(|kind| {
            crate::display::collect(crate::platform::Backend::Omarchy, kind)
                .map_err(|err| err.to_string())
        }),
        schedule::DIRTY_FLOOR,
    );
    eprintln!("ratmos daemon {} ready", crate::VERSION);
    let lane = daemon.clone();
    std::thread::spawn(move || lane.accept(listener));
    // Only a signal ends the daemon, and the lock goes with the process.
    daemon.schedule(&AtomicBool::new(false));
    Ok(0)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::sampler::cache::tests::scratch;
    use std::sync::atomic::AtomicUsize;

    /// A daemon over a real socket whose collector counts calls.
    pub fn serve(name: &str, delay: Duration) -> (PathBuf, Arc<Daemon>, Arc<AtomicUsize>) {
        let dir = scratch(name);
        let (listener, lock) = bind(&dir).unwrap();
        std::mem::forget(lock);
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let daemon = Daemon::new(
            dir.clone(),
            Box::new(move |kind| {
                let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                std::thread::sleep(delay);
                Ok(json!({"kind": kind, "n": n}))
            }),
            Duration::ZERO,
        );
        let lane = daemon.clone();
        std::thread::spawn(move || lane.accept(listener));
        (dir, daemon, calls)
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
    fn sampling_caches_the_document() {
        let (dir, daemon, _) = serve("caches", Duration::ZERO);
        let sample = daemon.sample("services", false).unwrap();
        assert_eq!(sample.value["kind"], "services");
        assert_eq!(cache::read(&dir, "services").unwrap().value, sample.value);
    }

    #[test]
    fn an_unknown_kind_is_refused() {
        let (_, daemon, calls) = serve("unknown", Duration::ZERO);
        assert!(daemon.sample("../x", true).is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn the_schedule_samples_live_at_once_and_stops() {
        let (dir, daemon, _) = serve("schedule", Duration::ZERO);
        let stop = Arc::new(AtomicBool::new(false));
        let lane = daemon.clone();
        let flag = stop.clone();
        let handle = std::thread::spawn(move || lane.schedule(&flag));
        let deadline = Instant::now() + Duration::from_secs(5);
        while cache::read(&dir, "live").is_none() {
            assert!(Instant::now() < deadline, "live was not sampled");
            std::thread::sleep(Duration::from_millis(50));
        }
        stop.store(true, Ordering::SeqCst);
        handle.join().unwrap();
    }

    #[test]
    fn a_dropped_kind_is_resampled_without_a_reader() {
        let (dir, daemon, _) = serve("redo", Duration::ZERO);
        // Everything is cached and not yet due, so only the drop can wake a kind.
        for kind in crate::display::KINDS {
            daemon.sample(kind, false).unwrap();
        }
        let stop = Arc::new(AtomicBool::new(false));
        let lane = daemon.clone();
        let flag = stop.clone();
        let handle = std::thread::spawn(move || lane.schedule(&flag));
        std::thread::sleep(Duration::from_millis(20));
        cache::drop_kinds(&dir, &["services"]);
        let deadline = Instant::now() + schedule::DIRTY_DELAY + Duration::from_secs(3);
        while cache::read(&dir, "services").is_none() {
            assert!(Instant::now() < deadline, "services was not resampled");
            std::thread::sleep(Duration::from_millis(100));
        }
        stop.store(true, Ordering::SeqCst);
        handle.join().unwrap();
    }

    #[test]
    fn the_schedule_yields_to_an_interactive_request() {
        let (dir, daemon, _) = serve("yield", Duration::from_millis(400));
        let stop = Arc::new(AtomicBool::new(false));
        // Live is due at once, so the scheduled lane is already running it.
        let lane = daemon.clone();
        let flag = stop.clone();
        let handle = std::thread::spawn(move || lane.schedule(&flag));
        std::thread::sleep(Duration::from_millis(100));
        // A different kind, asked for now, must not wait for the scheduled run.
        let started = Instant::now();
        daemon.sample("hardware", true).unwrap();
        assert!(
            started.elapsed() < Duration::from_millis(900),
            "{:?}",
            started.elapsed()
        );
        assert!(cache::read(&dir, "hardware").is_some());
        stop.store(true, Ordering::SeqCst);
        handle.join().unwrap();
    }
}
