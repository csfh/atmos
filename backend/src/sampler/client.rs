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
    if let Some(sample) = cache::read(dir, kind) {
        if sample.age() <= policy::of(kind).max_age {
            return Some(sample.value);
        }
    }
    ask(dir, kind, policy::of(kind).wait).map(|sample| sample.value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::cache::tests::scratch;
    use crate::sampler::daemon::tests::serve;
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
