//! Sessions: the long-lived connection a window's `serve` keeps to the daemon.
//!
//! A client opens one with `{"op":"session"}`. After the daemon's `ready`
//! frame the line protocol is typed frames, ids multiplexed:
//!
//! - client: `{"t":"sub","topics":{"live":{"intervalMs":2000}}}` replaces what
//!   it holds; `{"t":"unsub"}` drops all; `{"t":"req","id":7,"body":{...}}`
//!   asks any op the one-shot socket answers.
//! - daemon: `{"t":"event","topic":K,"sampledAt":MS,"tookMs":MS,"value":V}`,
//!   `{"t":"event","topic":K,"stale":true,"error":"..."}` and
//!   `{"t":"res","id":7,"body":{...}}`.
//!
//! `live` is sampled here, in the daemon at normal priority, only while a
//! session holds it and no faster than the quickest holder asks. That is work
//! a window is waiting on, so it is never background work.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::bus::is_pumped;
use super::cache;
use super::daemon::Daemon;

/// The quickest a pumped kind may be asked for, and the slowest it is clamped to.
const MIN_INTERVAL_MS: u64 = 250;
const MAX_INTERVAL_MS: u64 = 60_000;

/// How long an idle pump sleeps before it looks again.
const IDLE: Duration = Duration::from_secs(1);

fn topics_of(frame: &Value) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    let Some(map) = frame.get("topics").and_then(Value::as_object) else {
        return out;
    };
    for (kind, spec) in map {
        if !crate::display::is_kind(kind) {
            continue;
        }
        let asked = spec.get("intervalMs").and_then(Value::as_u64).unwrap_or(0);
        let interval = if is_pumped(kind) {
            asked.clamp(MIN_INTERVAL_MS, MAX_INTERVAL_MS)
        } else {
            0
        };
        out.insert(kind.clone(), interval);
    }
    out
}

/// Serve one session until the client goes away. Blocks.
pub(super) fn open(daemon: &Arc<Daemon>, stream: UnixStream) {
    let _ = stream.set_read_timeout(None);
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let Ok(reader) = stream.try_clone() else {
        return;
    };
    let mut out = &stream;
    let ready = json!({
        "ok": true,
        "t": "ready",
        "version": crate::VERSION,
        "build": super::launch::build_id(),
        "pid": std::process::id(),
    });
    if writeln!(out, "{ready}").is_err() {
        return;
    }
    let (subscriber, inbox) = daemon.bus.attach();
    let writer = stream.try_clone();
    let writer = std::thread::spawn(move || {
        let Ok(stream) = writer else { return };
        let mut out = &stream;
        for frame in inbox {
            if writeln!(out, "{frame}").is_err() {
                break;
            }
        }
        let _ = stream.shutdown(Shutdown::Both);
    });

    for line in BufReader::new(reader).lines() {
        let Ok(line) = line else { break };
        let Ok(frame) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match frame.get("t").and_then(Value::as_str) {
            Some("sub") => {
                let topics = topics_of(&frame);
                let kinds: Vec<String> = topics.keys().cloned().collect();
                subscriber.set(topics);
                // What the daemon already holds goes out at once; `live` is
                // taken by the pump, which samples as soon as it is held.
                for kind in kinds.iter().filter(|kind| !is_pumped(kind)) {
                    if let Some(sample) = daemon.held(kind) {
                        subscriber.admit_and_send(kind, &sample);
                    }
                }
                daemon.bus.poke();
            }
            Some("unsub") => {
                subscriber.set(HashMap::new());
                daemon.bus.poke();
            }
            Some("req") => {
                let id = frame.get("id").cloned().unwrap_or(Value::Null);
                let body = frame.get("body").cloned().unwrap_or(Value::Null);
                let (daemon, subscriber) = (daemon.clone(), subscriber.clone());
                std::thread::spawn(move || {
                    let reply = daemon.answer(&body);
                    subscriber.send(json!({"t": "res", "id": id, "body": reply}));
                });
            }
            _ => {}
        }
    }
    daemon.bus.detach(&subscriber);
    let _ = stream.shutdown(Shutdown::Both);
    drop(subscriber);
    let _ = writer.join();
}

impl Daemon {
    /// A held sample of `kind`: the one in the cache, however old.
    pub(super) fn held(&self, kind: &str) -> Option<cache::Sample> {
        cache::read(&self.dir, kind)
    }

    /// Sample the pumped kinds for as long as someone holds them.
    pub fn pump(&self, stop: &AtomicBool) {
        let mut began: Option<Instant> = None;
        while !stop.load(Ordering::SeqCst) {
            let Some(interval) = self.bus.interval("live") else {
                began = None;
                self.bus.wait(IDLE);
                continue;
            };
            let now = Instant::now();
            let due = began.map_or(now, |at| at + interval);
            if due > now {
                self.bus.wait((due - now).min(IDLE));
                continue;
            }
            began = Some(now);
            // A failure is published by `sample` itself.
            let _ = self.sample("live", true);
        }
    }
}

/// A daemon that is no longer worth running leaves, and the next window that
/// holds a topic starts a current one. That is the case when its runtime dir is
/// gone (a test or a logout removed it, and a second daemon may have started)
/// and when the binary it runs has been replaced (a rebuild or an update; the
/// version string does not change between dev builds).
pub(super) fn exit_when_stale(dir: PathBuf) {
    let exe = std::env::current_exe().ok();
    let started = exe.as_deref().and_then(super::launch::identity);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(5));
        if !dir.join("daemon.lock").exists() {
            eprintln!("ratmos daemon: {} is gone; leaving", dir.display());
            std::process::exit(0);
        }
        if let (Some(exe), Some(started)) = (&exe, &started) {
            if super::launch::identity(exe).as_ref() != Some(started) {
                eprintln!("ratmos daemon: {} was replaced; leaving", exe.display());
                std::process::exit(0);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::cache;
    use crate::sampler::daemon::tests::serve;

    /// A client of the session, as `serve` is.
    pub struct Client {
        stream: UnixStream,
        lines: std::io::Lines<BufReader<UnixStream>>,
    }

    impl Client {
        pub fn open(dir: &std::path::Path) -> Client {
            let stream = UnixStream::connect(cache::socket(dir)).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            writeln!(&stream, "{}", json!({"op": "session"})).unwrap();
            let mut lines = BufReader::new(stream.try_clone().unwrap()).lines();
            let ready: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
            assert_eq!(ready["t"], "ready");
            Client { stream, lines }
        }

        pub fn send(&self, frame: Value) {
            writeln!(&self.stream, "{frame}").unwrap();
        }

        pub fn next(&mut self) -> Value {
            serde_json::from_str(&self.lines.next().unwrap().unwrap()).unwrap()
        }

        /// The next frame, or None when nothing comes within `wait`.
        pub fn next_within(&mut self, wait: Duration) -> Option<Value> {
            self.stream.set_read_timeout(Some(wait)).unwrap();
            let line = self.lines.next()?.ok();
            self.stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            serde_json::from_str(&line?).ok()
        }
    }

    #[test]
    fn a_held_live_topic_is_sampled_on_the_holders_clock() {
        let (dir, _daemon, calls) = serve("pump", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(json!({"t": "sub", "topics": {"live": {"intervalMs": 300}}}));
        let first = client.next();
        let second = client.next();
        assert_eq!(first["topic"], "live");
        assert_eq!(first["value"]["from"], "daemon");
        let gap = second["sampledAt"].as_u64().unwrap() - first["sampledAt"].as_u64().unwrap();
        assert!((250..700).contains(&gap), "gap {gap}");
        assert!(calls.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    }

    #[test]
    fn nothing_is_sampled_once_nobody_holds_live() {
        let (dir, _daemon, calls) = serve("unsub", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(json!({"t": "sub", "topics": {"live": {"intervalMs": 250}}}));
        client.next();
        client.send(json!({"t": "unsub"}));
        std::thread::sleep(Duration::from_millis(400));
        let settled = calls.load(std::sync::atomic::Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(800));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), settled);
    }

    #[test]
    fn a_closed_session_stops_the_pump() {
        let (dir, _daemon, calls) = serve("hangup", Duration::ZERO);
        {
            let mut client = Client::open(&dir);
            client.send(json!({"t": "sub", "topics": {"live": {"intervalMs": 250}}}));
            client.next();
        }
        std::thread::sleep(Duration::from_millis(500));
        let settled = calls.load(std::sync::atomic::Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(800));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), settled);
    }

    #[test]
    fn two_windows_share_samples_and_keep_their_own_rate() {
        let (dir, _daemon, _calls) = serve("two", Duration::ZERO);
        let mut fast = Client::open(&dir);
        let mut slow = Client::open(&dir);
        fast.send(json!({"t": "sub", "topics": {"live": {"intervalMs": 250}}}));
        slow.send(json!({"t": "sub", "topics": {"live": {"intervalMs": 750}}}));
        let began = Instant::now();
        let mut fast_n = 0;
        let mut slow_n = 0;
        while began.elapsed() < Duration::from_millis(2200) {
            while fast.next_within(Duration::from_millis(20)).is_some() {
                fast_n += 1;
            }
            while slow.next_within(Duration::from_millis(20)).is_some() {
                slow_n += 1;
            }
        }
        assert!(fast_n >= 2 * slow_n - 1, "fast {fast_n} slow {slow_n}");
        assert!(slow_n >= 2, "slow {slow_n}");
    }

    #[test]
    fn a_slow_kind_is_pushed_when_it_is_sampled_and_on_subscribe() {
        let (dir, daemon, _calls) = serve("slowkind", Duration::ZERO);
        daemon.sample("services", true).unwrap();
        let mut client = Client::open(&dir);
        client.send(json!({"t": "sub", "topics": {"services": {}}}));
        let held = client.next();
        assert_eq!(held["topic"], "services");
        // Samples a millisecond apart are two samples; the same millisecond is one.
        std::thread::sleep(Duration::from_millis(5));
        daemon.sample("services", true).unwrap();
        let later = client.next();
        assert!(later["sampledAt"].as_u64() >= held["sampledAt"].as_u64());
        assert_ne!(later["value"]["n"], held["value"]["n"]);
    }

    #[test]
    fn a_request_frame_is_answered_with_its_id() {
        let (dir, _daemon, _calls) = serve("reqframe", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(json!({"t": "req", "id": 41, "body": {"op": "ping"}}));
        let reply = client.next();
        assert_eq!(reply["t"], "res");
        assert_eq!(reply["id"], 41);
        assert_eq!(reply["body"]["ok"], true);
    }

    #[test]
    fn an_unknown_kind_and_a_garbled_frame_are_ignored() {
        let (dir, _daemon, _calls) = serve("garbage", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(json!({"t": "sub", "topics": {"../x": {}}}));
        writeln!(&client.stream, "not json").unwrap();
        client.send(json!({"t": "req", "id": 1, "body": {"op": "ping"}}));
        assert_eq!(client.next()["id"], 1);
    }

    #[test]
    fn intervals_are_clamped() {
        let topics = topics_of(&json!({"topics": {
            "live": {"intervalMs": 1}, "disks": {"intervalMs": 99999}, "x": {}}}));
        assert_eq!(topics["live"], MIN_INTERVAL_MS);
        assert_eq!(topics["disks"], 0);
        assert!(!topics.contains_key("x"));
        let far = topics_of(&json!({"topics": {"live": {"intervalMs": 10_000_000}}}));
        assert_eq!(far["live"], MAX_INTERVAL_MS);
    }
}
