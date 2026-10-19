//! Background collection runs in its own process, `ratmos sampler`, under its
//! own systemd unit with idle scheduling and a CPU quota. The daemon decides
//! what to collect and when; the worker only collects what it is told.
//!
//! The worker connects to the daemon's socket and says hello. The daemon then
//! sends one job per line, `{"job":N,"kind":K}`, and gets one reply per line,
//! `{"job":N,"ok":true,"value":...}` or `{"job":N,"ok":false,"message":...}`.
//! With no worker connected the daemon does no background sampling at all.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{json, Value};

use super::cache;
use crate::error::Result;

pub type Reply = std::result::Result<Value, String>;

struct Conn {
    id: u64,
    pid: u32,
    writer: Mutex<UnixStream>,
    waiting: Mutex<HashMap<u64, Sender<Reply>>>,
    /// A job is out and unanswered. A job given up on stays out until the worker replies.
    busy: AtomicBool,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

/// The daemon's side: at most one connected worker.
#[derive(Default)]
pub struct Workers {
    current: Mutex<Option<Arc<Conn>>>,
    conns: AtomicU64,
    jobs: AtomicU64,
}

impl Workers {
    pub fn available(&self) -> bool {
        lock(&self.current).is_some()
    }

    /// Connected, and not still working on an earlier job.
    pub fn idle(&self) -> bool {
        lock(&self.current)
            .as_ref()
            .is_some_and(|conn| !conn.busy.load(Ordering::SeqCst))
    }

    pub fn pid(&self) -> Option<u32> {
        lock(&self.current).as_ref().map(|conn| conn.pid)
    }

    /// Adopt a connection that said hello. A newer worker replaces an older one.
    pub fn attach(self: &Arc<Self>, stream: UnixStream, pid: u32) {
        // The greeting was read under a timeout; a worker may sit idle for hours.
        let _ = stream.set_read_timeout(None);
        let Ok(reader) = stream.try_clone() else {
            return;
        };
        let conn = Arc::new(Conn {
            id: self.conns.fetch_add(1, Ordering::SeqCst) + 1,
            pid,
            writer: Mutex::new(stream),
            waiting: Mutex::new(HashMap::new()),
            busy: AtomicBool::new(false),
        });
        *lock(&self.current) = Some(conn.clone());
        let this = self.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(reader).lines().map_while(|line| line.ok()) {
                let Ok(reply) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                let Some(job) = reply.get("job").and_then(Value::as_u64) else {
                    continue;
                };
                let outcome = if reply.get("ok").and_then(Value::as_bool) == Some(true) {
                    Ok(reply.get("value").cloned().unwrap_or(Value::Null))
                } else {
                    Err(reply
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("the sampler failed")
                        .to_string())
                };
                let waiter = lock(&conn.waiting).remove(&job);
                // Answered, even if the caller stopped waiting. Free before the
                // answer goes out: a caller that asks again at once must not
                // find the worker still marked busy.
                conn.busy.store(false, Ordering::SeqCst);
                if let Some(tx) = waiter {
                    let _ = tx.send(outcome);
                }
            }
            // The worker went away: whoever waits is told, and nobody is current.
            for (_, tx) in lock(&conn.waiting).drain() {
                let _ = tx.send(Err("the sampler went away".into()));
            }
            let mut current = lock(&this.current);
            if current.as_ref().is_some_and(|c| c.id == conn.id) {
                *current = None;
            }
        });
    }

    /// Ask the worker to collect `kind`.
    pub fn run(&self, kind: &str, timeout: Duration) -> Reply {
        let conn = lock(&self.current)
            .clone()
            .ok_or_else(|| "no sampler is connected".to_string())?;
        // One job at a time: a worker still crawling through an earlier one gets no more.
        if conn.busy.swap(true, Ordering::SeqCst) {
            return Err("the sampler is still busy with an earlier job".to_string());
        }
        let job = self.jobs.fetch_add(1, Ordering::SeqCst) + 1;
        let (tx, rx) = channel();
        lock(&conn.waiting).insert(job, tx);
        let sent = {
            let mut writer = lock(&conn.writer);
            writeln!(writer, "{}", json!({"job": job, "kind": kind})).and_then(|()| writer.flush())
        };
        if let Err(err) = sent {
            lock(&conn.waiting).remove(&job);
            conn.busy.store(false, Ordering::SeqCst);
            return Err(format!("the sampler did not take the job: {err}"));
        }
        match rx.recv_timeout(timeout) {
            Ok(outcome) => outcome,
            Err(_) => {
                lock(&conn.waiting).remove(&job);
                Err(format!("the sampler did not answer for {kind} in time"))
            }
        }
    }
}

/// The worker's side: say hello, then collect each job the daemon sends until
/// it hangs up.
pub fn serve_jobs(stream: UnixStream, collect: &dyn Fn(&str) -> Reply) -> std::io::Result<()> {
    let mut out = &stream;
    writeln!(
        out,
        "{}",
        json!({"op": "sampler.hello", "pid": std::process::id()})
    )?;
    let mut lines = BufReader::new(&stream).lines();
    // The daemon's acknowledgement.
    lines.next().transpose()?;
    for line in lines {
        let line = line?;
        let Ok(job) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let id = job.get("job").and_then(Value::as_u64).unwrap_or(0);
        let kind = job.get("kind").and_then(Value::as_str).unwrap_or("");
        let outcome = catch_unwind(AssertUnwindSafe(|| collect(kind)))
            .unwrap_or_else(|_| Err(format!("the {kind} collector panicked")));
        let reply = match outcome {
            Ok(value) => json!({"job": id, "ok": true, "value": value}),
            Err(message) => json!({"job": id, "ok": false, "message": message}),
        };
        writeln!(out, "{reply}")?;
    }
    Ok(())
}

/// `ratmos sampler`: stay connected to the daemon and collect for it.
pub fn run() -> Result<i32> {
    let dir = cache::dir_from_env().ok_or("ratmos sampler needs XDG_RUNTIME_DIR")?;
    let socket = cache::socket(&dir);
    eprintln!("ratmos sampler {} waiting for the daemon", crate::VERSION);
    loop {
        if let Ok(stream) = UnixStream::connect(&socket) {
            let result = serve_jobs(stream, &|kind| {
                crate::display::collect(crate::platform::Backend::Omarchy, kind)
                    .map_err(|err| err.to_string())
            });
            if let Err(err) = result {
                eprintln!("ratmos sampler: lost the daemon: {err}");
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::cache::tests::scratch;
    use std::os::unix::net::UnixListener;

    /// A daemon-side listener that hands each hello to `workers`.
    fn listen(name: &str) -> (std::path::PathBuf, Arc<Workers>) {
        let dir = scratch(name);
        let listener = UnixListener::bind(dir.join("w.sock")).unwrap();
        let workers = Arc::new(Workers::default());
        let held = workers.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut line = String::new();
                let mut reader = BufReader::new(&stream);
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    continue;
                }
                let hello: Value = serde_json::from_str(line.trim()).unwrap();
                let mut out = &stream;
                let _ = writeln!(out, "{}", json!({"ok": true}));
                held.attach(
                    stream.try_clone().unwrap(),
                    hello["pid"].as_u64().unwrap() as u32,
                );
            }
        });
        (dir.join("w.sock"), workers)
    }

    fn connect_worker(
        socket: &std::path::Path,
        collect: impl Fn(&str) -> Reply + Send + 'static,
    ) -> std::thread::JoinHandle<()> {
        let stream = UnixStream::connect(socket).unwrap();
        std::thread::spawn(move || {
            let _ = serve_jobs(stream, &collect);
        })
    }

    fn wait_for(workers: &Workers, want: bool) {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while workers.available() != want {
            assert!(
                std::time::Instant::now() < deadline,
                "available never became {want}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn with_no_worker_a_job_fails_at_once() {
        let workers = Workers::default();
        assert!(!workers.available());
        let err = workers.run("live", Duration::from_secs(1)).unwrap_err();
        assert!(err.contains("no sampler"), "{err}");
    }

    #[test]
    fn a_worker_collects_what_it_is_told() {
        let (socket, workers) = listen("collects");
        connect_worker(&socket, |kind| Ok(json!({"kind": kind, "from": "worker"})));
        wait_for(&workers, true);
        assert_eq!(workers.pid(), Some(std::process::id()));
        let value = workers.run("disks", Duration::from_secs(2)).unwrap();
        assert_eq!(value["kind"], "disks");
        assert_eq!(value["from"], "worker");
    }

    #[test]
    fn a_failed_collection_comes_back_as_an_error_not_a_value() {
        let (socket, workers) = listen("fails");
        connect_worker(&socket, |kind| Err(format!("{kind} hung")));
        wait_for(&workers, true);
        assert_eq!(
            workers.run("disks", Duration::from_secs(2)).unwrap_err(),
            "disks hung"
        );
    }

    #[test]
    fn a_panicking_collector_does_not_take_the_worker_down() {
        let (socket, workers) = listen("panics");
        connect_worker(&socket, |kind| {
            if kind == "disks" {
                panic!("boom");
            }
            Ok(json!({"kind": kind}))
        });
        wait_for(&workers, true);
        let err = workers.run("disks", Duration::from_secs(2)).unwrap_err();
        assert!(err.contains("panicked"), "{err}");
        assert!(
            workers.run("live", Duration::from_secs(2)).is_ok(),
            "the worker carries on"
        );
    }

    #[test]
    fn a_slow_worker_costs_the_caller_only_its_timeout() {
        let (socket, workers) = listen("slow");
        connect_worker(&socket, |_| {
            std::thread::sleep(Duration::from_secs(2));
            Ok(json!({}))
        });
        wait_for(&workers, true);
        let began = std::time::Instant::now();
        let err = workers.run("live", Duration::from_millis(200)).unwrap_err();
        assert!(err.contains("in time"), "{err}");
        assert!(began.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn a_job_given_up_on_keeps_the_worker_busy_until_it_answers() {
        let (socket, workers) = listen("busy");
        connect_worker(&socket, |_| {
            std::thread::sleep(Duration::from_millis(600));
            Ok(json!({}))
        });
        wait_for(&workers, true);
        assert!(workers.idle());
        assert!(workers
            .run("live", Duration::from_millis(100))
            .unwrap_err()
            .contains("in time"));
        assert!(!workers.idle(), "still working on the job nobody waits for");
        let refused = workers.run("disks", Duration::from_secs(1)).unwrap_err();
        assert!(refused.contains("still busy"), "{refused}");
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while !workers.idle() {
            assert!(
                std::time::Instant::now() < deadline,
                "the worker never answered"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            workers.run("disks", Duration::from_secs(2)).is_ok(),
            "and then it takes work again"
        );
    }

    #[test]
    fn a_worker_that_hangs_up_mid_job_fails_the_job_and_leaves_nobody_current() {
        let (socket, workers) = listen("dies");
        let stream = UnixStream::connect(&socket).unwrap();
        let hung = stream.try_clone().unwrap();
        std::thread::spawn(move || {
            let _ = serve_jobs(stream, &|_| {
                std::thread::sleep(Duration::from_millis(300));
                Ok(json!({}))
            });
        });
        wait_for(&workers, true);
        let asker = {
            let workers = workers.clone();
            std::thread::spawn(move || workers.run("live", Duration::from_secs(5)))
        };
        std::thread::sleep(Duration::from_millis(100));
        hung.shutdown(std::net::Shutdown::Both).unwrap();
        let err = asker.join().unwrap().unwrap_err();
        assert!(err.contains("went away"), "{err}");
        wait_for(&workers, false);
    }

    #[test]
    fn a_newer_worker_replaces_the_one_before() {
        let (socket, workers) = listen("replace");
        connect_worker(&socket, |_| Ok(json!({"from": "first"})));
        wait_for(&workers, true);
        connect_worker(&socket, |_| Ok(json!({"from": "second"})));
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let who = workers.run("live", Duration::from_secs(2)).unwrap()["from"].clone();
            if who == "second" {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the second worker never took over"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
