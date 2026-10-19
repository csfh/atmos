//! The reader side: what `display::load` calls.
//!
//! A fresh enough sample comes straight from the cache. Otherwise the daemon
//! is asked for the kind with interactive priority and the reply carries the
//! sample. `None` means there is no daemon to ask, and the caller collects.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

use super::cache::{self, Sample};
use super::policy;

pub fn fetch(kind: &str) -> Option<Value> {
    fetch_in(&cache::dir_from_env()?, kind)
}

pub fn fetch_in(dir: &Path, kind: &str) -> Option<Value> {
    fetch_sample_in(dir, kind).map(|sample| sample.value)
}

/// The same read, with the time the sample was collected at.
pub fn fetch_sample(kind: &str) -> Option<Sample> {
    fetch_sample_in(&cache::dir_from_env()?, kind)
}

pub fn fetch_sample_in(dir: &Path, kind: &str) -> Option<Sample> {
    if let Some(sample) = cache::read(dir, kind) {
        if sample.age() <= policy::of(kind).max_age {
            return Some(sample);
        }
    }
    ask(dir, kind, policy::of(kind).wait)
}

fn ask(dir: &Path, kind: &str, wait: Duration) -> Option<Sample> {
    let stream = UnixStream::connect(cache::socket(dir)).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .ok()?;
    stream.set_read_timeout(Some(wait)).ok()?;
    let mut writer = &stream;
    writeln!(writer, "{}", json!({"op": "sample", "kind": kind})).ok()?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).ok()?;
    let reply: Value = serde_json::from_str(line.trim()).ok()?;
    if reply.get("ok").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    Sample::from_json(reply.get("sample")?)
}

/// What came of asking the daemon to run a command.
#[derive(Debug)]
pub enum Command {
    /// Nothing is listening, or the daemon predates commands. Nothing was
    /// queued, so the caller may run the command itself.
    Unavailable,
    /// It ran, in order on its lane. `coalesced` says a newer command of the
    /// same key ran in its place and `envelope` is that one's answer.
    Done { envelope: Value, coalesced: bool },
    /// The daemon took it and did not answer. It may still run, so the caller
    /// must not run it again.
    Lost(String),
}

/// Ask the daemon to run the command in `body` on its lane, with this
/// process's environment.
pub fn command(body: &Value, backend_id: &str) -> Command {
    let Some(dir) = cache::dir_from_env() else {
        return Command::Unavailable;
    };
    command_in(&dir, body, backend_id)
}

pub fn command_in(dir: &Path, body: &Value, backend_id: &str) -> Command {
    let Ok(stream) = UnixStream::connect(cache::socket(dir)) else {
        return Command::Unavailable;
    };
    let env: serde_json::Map<String, Value> = std::env::vars()
        .map(|(key, value)| (key, Value::String(value)))
        .collect();
    let ask = json!({"op": "command", "request": body, "backend": backend_id, "env": env});
    if stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .is_err()
        || stream.set_read_timeout(Some(COMMAND_WAIT)).is_err()
    {
        return Command::Unavailable;
    }
    let mut writer = &stream;
    // The request line is read with a limit; a large environment must fit.
    if writeln!(writer, "{ask}").is_err() {
        return Command::Unavailable;
    }
    let mut line = String::new();
    if BufReader::new(&stream).read_line(&mut line).is_err() || line.is_empty() {
        return Command::Lost("the daemon did not answer".into());
    }
    let reply: Value = match serde_json::from_str(line.trim()) {
        Ok(reply) => reply,
        Err(err) => return Command::Lost(format!("the daemon's answer was garbled: {err}")),
    };
    if reply.get("ok").and_then(Value::as_bool) == Some(true) {
        return Command::Done {
            envelope: reply.get("envelope").cloned().unwrap_or(Value::Null),
            coalesced: reply.get("coalesced").and_then(Value::as_bool) == Some(true),
        };
    }
    match reply.get("message").and_then(Value::as_str) {
        // An older daemon: it queued nothing.
        Some("unknown op") => Command::Unavailable,
        // An older daemon cut the line at 4 KiB, and a command carries the
        // environment: it read half of it, so nothing was queued.
        Some(message) if message.starts_with("bad request") => Command::Unavailable,
        Some(message) => Command::Lost(message.to_string()),
        None => Command::Lost("the daemon refused the command".into()),
    }
}

/// How long a command may take, queue included.
const COMMAND_WAIT: Duration = Duration::from_secs(160);

/// Wait until no command or leased job ahead of a read touches `topics`, for
/// at most `wait`. With no daemon there is nothing to wait for.
pub fn barrier(topics: &[String], wait: Duration) {
    if topics.is_empty() {
        return;
    }
    let Some(dir) = cache::dir_from_env() else {
        return;
    };
    barrier_in(&dir, topics, wait);
}

pub fn barrier_in(dir: &Path, topics: &[String], wait: Duration) {
    let Ok(stream) = UnixStream::connect(cache::socket(dir)) else {
        return;
    };
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_read_timeout(Some(wait + Duration::from_secs(1)));
    let mut writer = &stream;
    let ask = json!({"op": "barrier", "topics": topics, "waitMs": wait.as_millis() as u64});
    if writeln!(writer, "{ask}").is_err() {
        return;
    }
    let mut line = String::new();
    let _ = BufReader::new(&stream).read_line(&mut line);
}

/// A lane held for a job this process runs itself. Dropping it, or the process
/// dying, releases the lane and invalidates what it affects.
pub struct Lease(#[allow(dead_code)] UnixStream);

/// Take `lane` for a job that keeps its own process tree. None when there is
/// no daemon, it is an older one, or the lane stays busy for `wait`; the job
/// then simply runs without ordering.
pub fn lease(lane: &str, affects: &[&str], wait: Duration) -> Option<Lease> {
    lease_in(&cache::dir_from_env()?, lane, affects, wait)
}

pub fn lease_in(dir: &Path, lane: &str, affects: &[&str], wait: Duration) -> Option<Lease> {
    let stream = UnixStream::connect(cache::socket(dir)).ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .ok()?;
    stream
        .set_read_timeout(Some(wait + Duration::from_secs(1)))
        .ok()?;
    let mut writer = &stream;
    let ask =
        json!({"op": "lease", "lane": lane, "affects": affects, "waitMs": wait.as_millis() as u64});
    writeln!(writer, "{ask}").ok()?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).ok()?;
    let reply: Value = serde_json::from_str(line.trim()).ok()?;
    if reply.get("ok").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    stream.set_read_timeout(None).ok()?;
    Some(Lease(stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::cache::tests::scratch;
    use crate::sampler::daemon::tests::serve;
    use std::io::Read;
    use std::sync::atomic::Ordering;

    #[test]
    fn no_daemon_means_the_caller_collects() {
        let dir = scratch("nodaemon");
        assert!(fetch_in(&dir, "services").is_none());
    }

    #[test]
    fn a_cold_read_asks_the_daemon_and_the_next_one_hits_the_cache() {
        let (dir, _daemon, calls) = serve("cold", Duration::ZERO);
        let first = fetch_in(&dir, "services").unwrap();
        assert_eq!(first["kind"], "services");
        let second = fetch_in(&dir, "services").unwrap();
        assert_eq!(second, first);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn live_is_resampled_once_it_is_older_than_a_poll() {
        let (dir, _daemon, calls) = serve("live", Duration::ZERO);
        let first = fetch_in(&dir, "live").unwrap();
        // Within its max age the same sample is fine.
        assert_eq!(fetch_in(&dir, "live").unwrap(), first);
        std::thread::sleep(policy::of("live").max_age + Duration::from_millis(200));
        let later = fetch_in(&dir, "live").unwrap();
        assert_ne!(
            later["n"], first["n"],
            "a poller must not see one sample twice"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_dropped_kind_is_collected_again_on_the_next_read() {
        let (dir, _daemon, calls) = serve("dropped", Duration::ZERO);
        fetch_in(&dir, "services").unwrap();
        std::thread::sleep(Duration::from_millis(5));
        cache::drop_kinds(&dir, &["services"]);
        let again = fetch_in(&dir, "services").unwrap();
        assert_eq!(again["n"], 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn concurrent_cold_readers_cause_one_collection() {
        let (dir, _daemon, calls) = serve("herd", Duration::from_millis(200));
        let readers: Vec<_> = (0..6)
            .map(|_| {
                let dir = dir.clone();
                std::thread::spawn(move || fetch_in(&dir, "disks").unwrap())
            })
            .collect();
        let docs: Vec<Value> = readers.into_iter().map(|r| r.join().unwrap()).collect();
        assert!(docs.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_cold_snapshot_collects_every_kind_side_by_side() {
        let (dir, _daemon, calls) = serve("snapshot", Duration::from_millis(300));
        let started = std::time::Instant::now();
        let readers: Vec<_> = crate::display::KINDS
            .iter()
            .map(|kind| {
                let dir = dir.clone();
                std::thread::spawn(move || fetch_in(&dir, kind).unwrap())
            })
            .collect();
        for reader in readers {
            reader.join().unwrap();
        }
        assert_eq!(calls.load(Ordering::SeqCst), crate::display::KINDS.len());
        assert!(
            started.elapsed() < Duration::from_millis(1200),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn a_daemon_that_never_answers_costs_the_reader_only_its_budget() {
        let dir = scratch("silent");
        let (listener, lock) = crate::sampler::daemon::bind(&dir).unwrap();
        std::mem::forget(lock);
        std::thread::spawn(move || {
            let held: Vec<_> = listener.incoming().flatten().collect();
            std::thread::sleep(Duration::from_secs(5));
            drop(held);
        });
        let started = std::time::Instant::now();
        assert!(ask(&dir, "live", Duration::from_millis(300)).is_none());
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "{:?}",
            started.elapsed()
        );
    }

    #[test]
    fn an_older_daemon_that_cut_the_line_queued_nothing() {
        let dir = scratch("oldcut");
        let (listener, lock) = crate::sampler::daemon::bind(&dir).unwrap();
        std::mem::forget(lock);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut line = String::new();
                let _ = BufReader::new(&stream).take(4096).read_line(&mut line);
                let mut out = &stream;
                let _ = writeln!(
                    out,
                    "{}",
                    json!({"ok": false, "message": "bad request: EOF"})
                );
            }
        });
        let body = json!({"op": "display.brightness", "monitor": "DP-3", "percent": 1});
        assert!(matches!(
            command_in(&dir, &body, "omarchy"),
            Command::Unavailable
        ));
    }

    #[test]
    fn a_garbled_reply_falls_back_to_the_caller() {
        let dir = scratch("garbled");
        let (listener, lock) = crate::sampler::daemon::bind(&dir).unwrap();
        std::mem::forget(lock);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = &stream;
                let _ = writeln!(stream, "not json");
            }
        });
        assert!(fetch_in(&dir, "services").is_none());
    }
}
