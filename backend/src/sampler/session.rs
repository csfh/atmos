//! Sessions: the long-lived connection a window's `serve` keeps to the daemon.
//!
//! A client opens one with `{"op":"session"}`. After the daemon's `ready`
//! frame the line protocol is typed frames, ids multiplexed:
//!
//! - client: `{"t":"sub","topics":{"live":{"intervalMs":2000}}}` replaces what
//!   it holds; `{"t":"unsub"}` drops all; `{"t":"watch","paths":[...],"chrome":
//!   true,"accounts":{"user":..,"home":..}}` replaces what files it watches, and
//!   the daemon answers with `{"t":"event","topic":"stamp"|"chrome"|"accounts",
//!   "value":..}` when they change (see `watch`); `{"t":"req","id":7,"body":{...}}`
//!   asks any op the one-shot socket answers.
//! - daemon: `{"t":"event","topic":K,"sampledAt":MS,"tookMs":MS,"value":V}`,
//!   `{"t":"event","topic":K,"stale":true,"error":"..."}` and
//!   `{"t":"res","id":7,"body":{...}}`.
//!
//! A held kind that is not in the cache, or older than its policy allows, is
//! collected when the window holds it, so a page does not wait for the schedule.
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
use std::sync::{Arc, Mutex, MutexGuard};
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
        if !super::is_topic(kind) {
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

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

/// Write one frame to the session's socket. False when it is gone.
fn write_frame(wire: &Mutex<UnixStream>, frame: &Value) -> bool {
    let guard = lock(wire);
    let mut out = &*guard;
    writeln!(out, "{frame}").is_ok()
}

/// `{"t":"job","cid":..,"name":..,"args":{..},"env":{..},"stdin":".."}`: run a
/// registered job in the daemon with the window's environment. A name or
/// arguments the registry refuses ends at once, with the reason.
fn start_job(daemon: &Arc<Daemon>, owner: &Arc<super::bus::Subscriber>, frame: &Value) {
    let cid = frame
        .get("cid")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let name = frame.get("name").and_then(Value::as_str).unwrap_or("");
    let label = frame
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or(name)
        .to_string();
    let args = frame.get("args").cloned().unwrap_or(Value::Null);
    let env: Vec<(String, String)> = super::client::or_own_env(
        frame
            .get("env")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default(),
    );
    match crate::command::jobs::build(name, &args) {
        Ok(spec) => {
            // What the job reads comes from its entry, never from the frame,
            // so a password is only ever an argument of the entry that takes it.
            let stdin = spec.stdin.clone();
            daemon
                .jobs
                .start(&daemon.commands, owner, cid, label, name, spec, env, stdin);
        }
        Err(err) => {
            owner.send(json!({
                "t": "job.exit", "cid": cid, "job": 0, "code": 2,
                "summary": {"ok": false, "code": 2, "message": err.message, "warnings": []},
            }));
        }
    }
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
        "proto": super::PROTOCOL,
        "pid": std::process::id(),
    });
    if writeln!(out, "{ready}").is_err() {
        return;
    }
    let (subscriber, inbox) = daemon.bus.attach();
    // Events and replies share the socket under one lock. A reply is written
    // directly, never queued behind events, so a window that falls behind on
    // events loses its subscription and never a reply it is waiting for.
    let Ok(wire) = stream.try_clone().map(|clone| Arc::new(Mutex::new(clone))) else {
        return;
    };
    let writer = {
        let wire = Arc::clone(&wire);
        std::thread::spawn(move || {
            for frame in inbox {
                if !write_frame(&wire, &frame) {
                    break;
                }
            }
            let _ = lock(&wire).shutdown(Shutdown::Both);
        })
    };

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
                // A cached settings topic is only as good as its files: they
                // are looked at before anything is sent, and a topic whose
                // files moved, held or not, is dropped and read again
                // (docs/adr/0015). The window is sent the new sample, never
                // the old one first.
                if kinds.iter().any(|kind| super::is_settings(kind)) {
                    daemon.look_at_files();
                }
                // What the daemon already holds goes out at once; `live` is
                // taken by the pump, which samples as soon as it is held.
                for kind in kinds.iter().filter(|kind| !is_pumped(kind)) {
                    let held = daemon.held(kind);
                    // A settings topic is as good as its files, just looked at,
                    // and its last invalidation.
                    let fresh = held.as_ref().is_some_and(|sample| {
                        super::is_settings(kind) || sample.age() <= super::policy::of(kind).max_age
                    });
                    if let Some(sample) = &held {
                        subscriber.admit_and_send(kind, sample);
                    }
                    // Nothing held, or nothing recent enough: a window that
                    // holds a kind is not made to wait for the schedule. The
                    // fresh sample is published to every holder.
                    if !fresh && !daemon.settings.recently_failed(kind) {
                        let (daemon, kind) = (Arc::clone(daemon), kind.clone());
                        std::thread::spawn(move || {
                            let _ = daemon.sample(&kind, true);
                        });
                    }
                }
                daemon.bus.poke();
            }
            Some("env") => {
                // The newest window's environment is what a settings topic is
                // collected with.
                let env = frame
                    .get("env")
                    .and_then(Value::as_object)
                    .map(|map| {
                        map.iter()
                            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                daemon.settings.set_env(env);
            }
            Some("watch") => {
                daemon
                    .watches
                    .set(&subscriber, super::watch::Spec::from_frame(&frame));
            }
            Some("unsub") => {
                subscriber.set(HashMap::new());
                daemon.bus.poke();
            }
            Some("job") => start_job(daemon, &subscriber, &frame),
            Some("input") => {
                daemon.jobs.input(
                    frame.get("job").and_then(Value::as_u64).unwrap_or(0),
                    frame.get("data").and_then(Value::as_str).unwrap_or(""),
                    frame.get("close").and_then(Value::as_bool) == Some(true),
                );
            }
            Some("cancel") => {
                daemon
                    .jobs
                    .cancel(frame.get("job").and_then(Value::as_u64).unwrap_or(0));
            }
            Some("attach") => {
                let job = frame.get("job").and_then(Value::as_u64).unwrap_or(0);
                let from = frame.get("from").and_then(Value::as_u64).unwrap_or(0);
                if !daemon.jobs.attach(job, from, &subscriber) {
                    subscriber.send(json!({"t": "job.gone", "job": job}));
                }
            }
            Some("req") => {
                let id = frame.get("id").cloned().unwrap_or(Value::Null);
                let body = frame.get("body").cloned().unwrap_or(Value::Null);
                let (daemon, wire) = (daemon.clone(), Arc::clone(&wire));
                std::thread::spawn(move || {
                    daemon.note_request_frame();
                    let reply = daemon.answer(&body);
                    write_frame(&wire, &json!({"t": "res", "id": id, "body": reply}));
                });
            }
            _ => {}
        }
    }
    daemon.bus.detach(&subscriber);
    daemon.watches.remove(&subscriber);
    // Its jobs carry on with nobody listening; a window that comes back attaches.
    daemon.jobs.forget_owner(&subscriber);
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
pub(super) fn exit_when_stale(dir: PathBuf, jobs: Arc<super::jobs::Jobs>) {
    let exe = std::env::current_exe().ok();
    let started = exe.as_deref().and_then(super::launch::identity);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(5));
        // A job in flight is not cut off for a rebuild or a vanished directory.
        if jobs.running() > 0 {
            continue;
        }
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
pub(crate) mod tests {
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
    fn a_kind_that_is_not_cached_is_collected_when_a_window_holds_it() {
        let (dir, _daemon, calls) = serve("coldhold", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(json!({"t": "sub", "topics": {"hardware": {}, "disks": {}}}));
        let mut heard = Vec::new();
        for _ in 0..2 {
            heard.push(client.next()["topic"].as_str().unwrap().to_string());
        }
        heard.sort();
        assert_eq!(
            heard,
            ["disks", "hardware"],
            "pushed without waiting for the schedule"
        );
        assert!(calls.load(std::sync::atomic::Ordering::SeqCst) >= 2);
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
    fn a_watch_frame_is_answered_with_stamp_events_and_ends_with_the_session() {
        let (dir, daemon, _calls) = serve("watching", Duration::ZERO);
        let file = dir.join("watched.txt");
        std::fs::write(&file, "one").unwrap();
        let mut client = Client::open(&dir);
        client.send(json!({"t": "watch", "paths": [file.display().to_string()]}));
        let first = client.next();
        assert_eq!(first["topic"], "stamp");
        assert_eq!(first["value"]["items"][0]["text"], "one");
        std::thread::sleep(Duration::from_millis(10));
        // The signature is size and mtime in whole seconds, so the size changes.
        std::fs::write(&file, "three").unwrap();
        let changed = client.next();
        assert_eq!(changed["value"]["items"][0]["text"], "three");
        assert_eq!(daemon.watches.windows(), 1);
        drop(client);
        for _ in 0..50 {
            if daemon.watches.windows() == 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(
            daemon.watches.windows(),
            0,
            "the watch ended with the session"
        );
    }

    fn job_frame(cid: &str, script: &str, stdin: Option<&str>) -> Value {
        let mut frame = json!({
            "t": "job", "cid": cid, "name": format!("test.sh:{script}"),
            "env": {"PATH": "/usr/bin:/bin"},
        });
        if let Some(text) = stdin {
            frame["args"] = json!({ "stdin": text });
        }
        frame
    }

    fn next_until(client: &mut Client, kind: &str) -> Vec<Value> {
        let mut seen = Vec::new();
        loop {
            let frame = client.next();
            let done = frame["t"] == kind;
            seen.push(frame);
            if done {
                return seen;
            }
        }
    }

    #[test]
    fn a_job_runs_over_the_session_with_input_and_ends_with_its_summary() {
        let (dir, _daemon, _calls) = serve("jobframes", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(job_frame(
            "w1",
            "read a; echo got-$a; echo bad >&2; exit 4",
            Some("x\n"),
        ));
        let frames = next_until(&mut client, "job.exit");
        assert_eq!(frames[0]["t"], "job.started");
        assert_eq!(frames[0]["cid"], "w1");
        assert!(frames
            .iter()
            .any(|f| f["t"] == "stream" && f["line"] == "got-x"));
        let end = frames.last().unwrap();
        assert_eq!(end["code"], 4);
        assert_eq!(end["summary"]["message"], "bad");
    }

    #[test]
    fn an_unknown_job_ends_at_once_with_the_reason() {
        let (dir, _daemon, _calls) = serve("jobunknown", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(json!({"t": "job", "cid": "w", "name": "no.such.job", "args": {}}));
        let end = client.next();
        assert_eq!(end["t"], "job.exit");
        assert_eq!(end["cid"], "w");
        assert!(end["summary"]["message"]
            .as_str()
            .unwrap()
            .contains("unknown job"));
    }

    #[test]
    fn a_job_is_cancelled_by_a_frame_and_listed_by_a_request() {
        let (dir, _daemon, _calls) = serve("jobcancel", Duration::ZERO);
        let mut client = Client::open(&dir);
        client.send(job_frame("w", "echo up; sleep 30", None));
        let started = client.next();
        let job = started["job"].as_u64().unwrap();
        loop {
            if client.next()["line"] == "up" {
                break;
            }
        }
        client.send(json!({"t": "req", "id": 5, "body": {"op": "jobs"}}));
        let listed = loop {
            let frame = client.next();
            if frame["t"] == "res" {
                break frame;
            }
        };
        assert_eq!(listed["body"]["jobs"][0]["state"], "running");
        client.send(json!({"t": "cancel", "job": job}));
        let end = next_until(&mut client, "job.exit");
        assert_ne!(end.last().unwrap()["code"], 0);
    }

    #[test]
    fn a_job_survives_its_window_and_the_daemon_waits_for_it() {
        let (dir, daemon, _calls) = serve("jobsurvive", Duration::ZERO);
        let job = {
            let mut client = Client::open(&dir);
            client.send(job_frame("w", "echo before; sleep 0.6; echo after", None));
            let started = client.next();
            loop {
                if client.next()["line"] == "before" {
                    break;
                }
            }
            started["job"].as_u64().unwrap()
        };
        // The window is gone; the job is not.
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(daemon.jobs.running(), 1);
        let mut again = Client::open(&dir);
        again.send(json!({"t": "attach", "job": job, "from": 0}));
        let frames = next_until(&mut again, "job.exit");
        let lines: Vec<&str> = frames
            .iter()
            .filter(|f| f["t"] == "stream")
            .map(|f| f["line"].as_str().unwrap())
            .collect();
        assert_eq!(lines, ["before", "after"]);
        again.send(json!({"t": "attach", "job": 999, "from": 0}));
        assert_eq!(again.next()["t"], "job.gone");
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
