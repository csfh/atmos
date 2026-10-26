//! What `serve` pushes to its window besides replies: sampled topics.
//!
//! The window says what it holds with `sub.set`. This module keeps one session
//! to the daemon (see `sampler::session`) and turns the daemon's events into
//! `{"event":"sample",...}` lines, and says whether the daemon is there
//! (`{"event":"backend",...}`). There is no sampling here in its place; only a
//! `--root` fixture, which has no daemon, samples its own files (docs/adr/0010).
//!
//! A held topic that goes quiet for six of its intervals gets an error event.
//! That is the guarantee the window's old stall timer gave, kept where the
//! clock now lives.

use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::sampler::{cache, client, launch};
use crate::serve::{send, Config, Out};

/// A held topic that is quiet this many intervals is reported stalled, and never
/// sooner than `STALL_FLOOR`, since a loaded machine may take a while to walk /proc.
const STALL_FACTOR: u32 = 6;
const STALL_FLOOR: Duration = Duration::from_secs(15);

/// The longest pause between looks for a daemon that is not there.
const MAX_BACKOFF: Duration = Duration::from_secs(5);

/// The least time between two attempts to start a daemon.
const LAUNCH_EVERY: Duration = Duration::from_secs(5);

/// A session that lasts this long counts as the daemon having been there.
const STABLE: Duration = Duration::from_secs(5);

/// The slowest a topic that changes rarely is looked at in a fixture.
const SLOW_LOCAL: Duration = Duration::from_secs(30);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

#[derive(Default)]
struct State {
    topics: BTreeMap<String, u64>,
    /// The last `watch.set` the window sent, as the daemon's `watch` frame.
    watch: Value,
    generation: u64,
    stop: bool,
}

#[derive(Default)]
pub struct Subs {
    state: Mutex<State>,
    changed: Condvar,
}

impl Subs {
    pub fn set(&self, topics: BTreeMap<String, u64>) {
        let mut state = lock(&self.state);
        state.topics = topics;
        state.generation += 1;
        self.changed.notify_all();
    }

    /// What the window watches: `{"paths": [...], "chrome": bool, "accounts": ..}`.
    pub fn set_watch(&self, spec: Value) {
        let mut state = lock(&self.state);
        state.watch = spec;
        state.generation += 1;
        self.changed.notify_all();
    }

    fn watch_frame(&self) -> Option<Value> {
        let state = lock(&self.state);
        let spec = state.watch.as_object()?;
        let mut frame = spec.clone();
        frame.insert("t".into(), json!("watch"));
        Some(Value::Object(frame))
    }

    pub fn stop(&self) {
        lock(&self.state).stop = true;
        self.changed.notify_all();
    }

    /// The topics held now and a number that moves when they change; None once
    /// stopped.
    fn view(&self) -> Option<(BTreeMap<String, u64>, u64)> {
        let state = lock(&self.state);
        (!state.stop).then(|| (state.topics.clone(), state.generation))
    }

    /// Sleep until the held set differs from `seen`, a stop, or `timeout`.
    fn wait(&self, seen: u64, timeout: Duration) {
        let state = lock(&self.state);
        let _ = self.changed.wait_timeout_while(state, timeout, |state| {
            !state.stop && state.generation == seen
        });
    }

    /// Sleep until the held set differs from `seen` (the generation the caller
    /// looked at, so a change made since is not slept through), a stop, or
    /// `timeout`. False once stopped.
    fn idle(&self, seen: u64, timeout: Duration) -> bool {
        let state = lock(&self.state);
        let (state, _) = self
            .changed
            .wait_timeout_while(state, timeout, |state| {
                !state.stop && state.generation == seen
            })
            .unwrap_or_else(|err| err.into_inner());
        !state.stop
    }

    /// Sleep for `timeout` or until stopped; a change to the held set does not
    /// cut it short.
    fn sleep(&self, timeout: Duration) {
        let state = lock(&self.state);
        let _ = self
            .changed
            .wait_timeout_while(state, timeout, |state| !state.stop);
    }

    fn stopped(&self) -> bool {
        lock(&self.state).stop
    }
}

/// Parse `sub.set`: `{"topics": {"live": {"intervalMs": 2000}}}`.
pub fn parse_topics(request: &Value) -> Result<BTreeMap<String, u64>, String> {
    let mut out = BTreeMap::new();
    match request.get("topics") {
        None | Some(Value::Null) => {}
        Some(Value::Object(map)) => {
            for (kind, spec) in map {
                if !crate::sampler::is_topic(kind) {
                    return Err(format!("topics: unknown topic {kind}"));
                }
                out.insert(
                    kind.clone(),
                    spec.get("intervalMs").and_then(Value::as_u64).unwrap_or(0),
                );
            }
        }
        Some(_) => return Err("topics: expected an object".into()),
    }
    Ok(out)
}

pub fn sample_event(kind: &str, sampled_at: u64, took_ms: u64, value: Value) -> Value {
    json!({"event": "sample", "kind": kind, "sampledAt": sampled_at, "tookMs": took_ms, "result": value})
}

fn error_event(kind: &str, message: &str) -> Value {
    json!({"event": "sample", "kind": kind, "stale": true, "error": message})
}

/// A daemon frame as the window sees it.
fn from_daemon(frame: &Value) -> Option<Value> {
    // A job's frames come back as `{"event":"job","kind":..}` with the window's cid.
    if let Some(kind) = frame.get("t").and_then(Value::as_str).and_then(|t| {
        t.strip_prefix("job.")
            .or((t == "stream").then_some("stream"))
    }) {
        let mut event = frame.as_object()?.clone();
        event.remove("t");
        event.insert("event".into(), json!("job"));
        event.insert("kind".into(), json!(kind));
        return Some(Value::Object(event));
    }
    if frame.get("t").and_then(Value::as_str) != Some("event") {
        return None;
    }
    let kind = frame.get("topic")?.as_str()?;
    // The files the window watches come back under the names it already knows.
    if matches!(kind, "stamp" | "chrome" | "accounts") {
        return Some(json!({"event": kind, "result": frame.get("value")?.clone()}));
    }
    if let Some(message) = frame.get("error").and_then(Value::as_str) {
        return Some(error_event(kind, message));
    }
    Some(sample_event(
        kind,
        frame.get("sampledAt")?.as_u64()?,
        frame.get("tookMs").and_then(Value::as_u64).unwrap_or(0),
        frame.get("value")?.clone(),
    ))
}

/// Notices held topics that have gone quiet.
#[derive(Default)]
struct Watchdog {
    quiet_since: HashMap<String, Instant>,
    told: HashMap<String, bool>,
}

impl Watchdog {
    fn heard(&mut self, kind: &str, now: Instant) {
        self.quiet_since.insert(kind.to_string(), now);
        self.told.insert(kind.to_string(), false);
    }

    /// Topics held at `interval_ms > 0` that are overdue and not yet reported.
    fn overdue(&mut self, topics: &BTreeMap<String, u64>, now: Instant) -> Vec<String> {
        self.quiet_since.retain(|kind, _| topics.contains_key(kind));
        self.told.retain(|kind, _| topics.contains_key(kind));
        let mut late = Vec::new();
        for (kind, interval_ms) in topics {
            if *interval_ms == 0 {
                continue;
            }
            let since = *self.quiet_since.entry(kind.clone()).or_insert(now);
            let limit = (Duration::from_millis(*interval_ms) * STALL_FACTOR).max(STALL_FLOOR);
            let told = self.told.entry(kind.clone()).or_insert(false);
            if now.duration_since(since) >= limit && !*told {
                *told = true;
                late.push(kind.clone());
            }
        }
        late
    }
}

fn report_stalls(out: &Out, dog: &mut Watchdog, topics: &BTreeMap<String, u64>) {
    for kind in dog.overdue(topics, Instant::now()) {
        send(
            out,
            &error_event(&kind, &format!("The {kind} sampler stopped responding.")),
        );
    }
}

fn sub_frame(topics: &BTreeMap<String, u64>) -> Value {
    let map: serde_json::Map<String, Value> = topics
        .iter()
        .map(|(kind, ms)| (kind.clone(), json!({"intervalMs": ms})))
        .collect();
    json!({"t": "sub", "topics": map})
}

/// Whether this window looks for a daemon at all: not with a `--root` fixture
/// and not off Omarchy.
pub(crate) fn daemon_dir(config: &Config) -> Option<std::path::PathBuf> {
    (config.root.is_none() && config.backend == crate::platform::Backend::Omarchy)
        .then(cache::dir_from_env)
        .flatten()
}

/// Called as the window starts, so a request that arrives before `run` has
/// begun still knows a daemon is being looked for.
pub fn expect_daemon(config: &Config) {
    if daemon_dir(config).is_some() {
        client::trying(true);
    }
}

/// The window's word on whether the daemon is there: `{"event":"backend",
/// "state":"up"}`, or `"down"` with the reason, how many looks have failed in a
/// row and when the next one is.
fn backend_event(why: Option<(&str, u32, Duration)>) -> Value {
    match why {
        None => json!({"event": "backend", "state": "up"}),
        Some((why, attempt, retry)) => json!({
            "event": "backend", "state": "down", "why": why,
            "attempt": attempt, "retryMs": retry.as_millis() as u64,
        }),
    }
}

/// Seconds to wait before looking for the daemon again after `failures` looks
/// in a row failed: one, doubling to a ceiling.
fn backoff(failures: u32) -> Duration {
    Duration::from_secs(1u64 << failures.saturating_sub(1).min(4)).min(MAX_BACKOFF)
}

/// Run until `subs` stops. A window keeps one session to the daemon for as long
/// as it lives: it carries the held topics and, through `client::Bridge`, the
/// window's commands and barriers. The daemon is started when it is not there,
/// and looked for again with a growing pause when it goes away; the window is
/// told each time (`backend_event`), and nothing is done here in its place
/// (docs/adr/0010). A fixture root has no daemon, so its topics are sampled
/// here.
pub fn run(config: &Config, out: &Out, subs: &Subs) {
    let Some(dir) = daemon_dir(config) else {
        return run_fixture(config, out, subs);
    };
    let mut told = String::new();
    let mut failures = 0u32;
    let mut last_launch: Option<Instant> = None;
    while !subs.stopped() {
        client::trying(true);
        // Starting one costs a few processes; looking for it does not.
        if last_launch.is_none_or(|at| at.elapsed() >= LAUNCH_EVERY) {
            last_launch = Some(Instant::now());
            launch::ensure(&dir);
        }
        let why = match connect(&dir) {
            Ok(stream) => {
                told.clear();
                let began = Instant::now();
                send(out, &backend_event(None));
                session(stream, out, subs);
                if began.elapsed() >= STABLE {
                    failures = 0;
                }
                "The Atmos daemon stopped answering".to_string()
            }
            Err(why) => why,
        };
        if subs.stopped() {
            return;
        }
        // Nothing is looking now; a request is refused, not held.
        client::trying(false);
        failures += 1;
        let retry = backoff(failures);
        send(out, &backend_event(Some((&why, failures, retry))));
        if told != why {
            eprintln!("atmosd serve: {why}; looking again in {}s", retry.as_secs());
            told = why;
        }
        subs.sleep(retry);
    }
}

/// A fixture root: sample what the window holds, on its clock, until it stops.
fn run_fixture(config: &Config, out: &Out, subs: &Subs) {
    while let Some((topics, seen)) = subs.view() {
        if topics.is_empty() {
            subs.idle(seen, Duration::from_secs(3600));
        } else {
            local(config, out, subs);
        }
    }
}

fn connect(dir: &Path) -> Result<UnixStream, String> {
    let stream =
        UnixStream::connect(cache::socket(dir)).map_err(|err| format!("no daemon ({err})"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| err.to_string())?;
    writeln!(&stream, "{}", json!({"op": "session"})).map_err(|err| err.to_string())?;
    let mut line = String::new();
    BufReader::new(&stream)
        .read_line(&mut line)
        .map_err(|err| format!("the daemon did not answer ({err})"))?;
    let reply: Value = serde_json::from_str(line.trim()).unwrap_or(Value::Null);
    // A daemon that cannot run a window's reads answers them "not a command",
    // so every page would be empty with nothing to say why: it is named here
    // and the window shows its screen (docs/adr/0012).
    if reply.get("t").and_then(Value::as_str) == Some("ready")
        && reply.get("proto").and_then(Value::as_u64) < Some(crate::sampler::PROTOCOL)
    {
        return Err(
            "the running daemon is older than this window and cannot run its reads (restart atmosd.service)"
                .into(),
        );
    }
    if reply.get("t").and_then(Value::as_str) != Some("ready") {
        return Err(
            "the running daemon is older than this build and has no sessions (restart atmosd.service)"
                .into(),
        );
    }
    stream
        .set_read_timeout(None)
        .map_err(|err| err.to_string())?;
    Ok(stream)
}

/// Forward the daemon's events until it goes away or `subs` stops.
fn session(stream: UnixStream, out: &Out, subs: &Subs) {
    let alive = Arc::new(AtomicBool::new(true));
    let heard: Arc<Mutex<Watchdog>> = Arc::new(Mutex::new(Watchdog::default()));
    let reader = match stream.try_clone() {
        Ok(clone) => clone,
        Err(_) => return,
    };
    let bridge: Arc<Mutex<Option<Arc<client::Bridge>>>> = Arc::new(Mutex::new(None));
    if let Ok(wire) = stream.try_clone() {
        let made = client::Bridge::new(wire);
        client::install(Arc::clone(&made));
        *lock(&bridge) = Some(made);
    }
    let forward = {
        let (out, alive, heard) = (Arc::clone(out), Arc::clone(&alive), Arc::clone(&heard));
        let bridge = Arc::clone(&bridge);
        std::thread::spawn(move || {
            for line in BufReader::new(reader).lines() {
                let Ok(line) = line else { break };
                let Ok(frame) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                if frame.get("t").and_then(Value::as_str) == Some("res") {
                    if let (Some(id), Some(body)) =
                        (frame.get("id").and_then(Value::as_u64), frame.get("body"))
                    {
                        if let Some(bridge) = &*lock(&bridge) {
                            bridge.deliver(id, body.clone());
                        }
                    }
                    continue;
                }
                if let Some(event) = from_daemon(&frame) {
                    if event.get("error").is_none() {
                        if let Some(kind) = event["kind"].as_str() {
                            lock(&heard).heard(kind, Instant::now());
                        }
                    }
                    send(&out, &event);
                }
            }
            alive.store(false, Ordering::SeqCst);
            // Let go of anyone waiting on a reply that will not come.
            if let Some(bridge) = lock(&bridge).take() {
                bridge.close();
            }
        })
    };

    let wire = lock(&bridge).clone();
    // The settings topics are collected with the environment of a window that
    // holds them (`hyprctl` and `omarchy` need the session's).
    // A window whose session is the daemon's has none to tell (docs/adr/0014).
    if let Some(wire) = &wire {
        let env = client::window_env();
        if !env.is_empty() {
            wire.send(&json!({"t": "env", "env": env}));
        }
    }
    let mut sent: Option<u64> = None;
    while alive.load(Ordering::SeqCst) {
        let Some((topics, generation)) = subs.view() else {
            break;
        };
        if sent != Some(generation) {
            let Some(wire) = &wire else { break };
            let mut frames = vec![sub_frame(&topics)];
            frames.extend(subs.watch_frame());
            if !frames.iter().all(|frame| wire.send(frame)) {
                break;
            }
            sent = Some(generation);
        }
        report_stalls(out, &mut lock(&heard), &topics);
        subs.wait(generation, Duration::from_secs(1));
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
    let _ = forward.join();
    client::uninstall();
}

/// Sample the held topics here, on the window's clock, until they are all let
/// go or the window stops. Only a fixture root does this.
fn local(config: &Config, out: &Out, subs: &Subs) {
    let mut last: HashMap<String, (Instant, u64)> = HashMap::new();
    let mut dog = Watchdog::default();
    loop {
        let Some((topics, generation)) = subs.view() else {
            return;
        };
        if topics.is_empty() {
            return;
        }
        let now = Instant::now();
        let mut next = now + Duration::from_secs(1);
        for (kind, interval_ms) in &topics {
            let every = if *interval_ms > 0 {
                Duration::from_millis(*interval_ms)
            } else {
                SLOW_LOCAL
            };
            let due = last.get(kind).map_or(now, |(at, _)| *at + every);
            if due > now {
                next = next.min(due);
                continue;
            }
            let after = last.get(kind).map_or(0, |(_, sampled)| *sampled);
            match sample_here(config, kind) {
                Ok((sampled_at, took_ms, value)) => {
                    last.insert(kind.clone(), (Instant::now(), sampled_at));
                    dog.heard(kind, Instant::now());
                    // The same sample twice would read as a rate of zero.
                    if sampled_at > after {
                        send(out, &sample_event(kind, sampled_at, took_ms, value));
                    }
                }
                Err(message) => {
                    last.insert(kind.clone(), (Instant::now(), after));
                    send(out, &error_event(kind, &message));
                }
            }
            next = next.min(Instant::now() + every);
        }
        report_stalls(out, &mut dog, &topics);
        subs.wait(generation, next.saturating_duration_since(Instant::now()));
    }
}

/// One sample with the time its collection began.
fn sample_here(config: &Config, kind: &str) -> Result<(u64, u64, Value), String> {
    let began = Instant::now();
    let at = cache::now_ms();
    let value = crate::display::load(config.root.as_deref(), config.backend, kind)
        .map_err(|err| err.to_string())?;
    Ok((at, began.elapsed().as_millis() as u64, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(pairs: &[(&str, u64)]) -> BTreeMap<String, u64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn sub_set_names_kinds_and_their_intervals() {
        let topics =
            parse_topics(&json!({"topics": {"live": {"intervalMs": 2000}, "disks": {}}})).unwrap();
        assert_eq!(topics, held(&[("live", 2000), ("disks", 0)]));
        assert!(parse_topics(&json!({})).unwrap().is_empty());
        assert!(parse_topics(&json!({"topics": {"nope": {}}})).is_err());
        assert!(parse_topics(&json!({"topics": [1]})).is_err());
    }

    #[test]
    fn a_jobs_frames_become_job_events_with_the_windows_cid() {
        let started = from_daemon(&json!({"t": "job.started", "cid": "w1", "job": 4})).unwrap();
        assert_eq!(
            started,
            json!({"event": "job", "kind": "started", "cid": "w1", "job": 4})
        );
        let line = from_daemon(
            &json!({"t": "stream", "cid": "w1", "job": 4, "seq": 0, "fd": "out", "line": "hi"}),
        )
        .unwrap();
        assert_eq!(line["kind"], "stream");
        assert_eq!(line["line"], "hi");
        let end =
            from_daemon(&json!({"t": "job.exit", "cid": "w1", "job": 4, "code": 1, "summary": {}}))
                .unwrap();
        assert_eq!(end["kind"], "exit");
        assert_eq!(
            from_daemon(&json!({"t": "job.gone", "job": 9})).unwrap()["kind"],
            "gone"
        );
    }

    #[test]
    fn a_daemon_event_becomes_a_window_event() {
        let frame =
            json!({"t": "event", "topic": "live", "sampledAt": 9, "tookMs": 2, "value": {"a": 1}});
        assert_eq!(
            from_daemon(&frame).unwrap(),
            json!({"event": "sample", "kind": "live", "sampledAt": 9, "tookMs": 2, "result": {"a": 1}})
        );
        let failed = json!({"t": "event", "topic": "live", "stale": true, "error": "boom"});
        assert_eq!(from_daemon(&failed).unwrap()["error"], "boom");
        assert!(from_daemon(&json!({"t": "res", "id": 1})).is_none());
        assert!(from_daemon(&json!({"t": "event", "topic": "live"})).is_none());
    }

    #[test]
    fn a_quiet_topic_is_reported_once_after_six_intervals() {
        let mut dog = Watchdog::default();
        let topics = held(&[("live", 5000), ("disks", 0)]);
        let start = Instant::now();
        assert!(dog.overdue(&topics, start).is_empty());
        assert!(dog
            .overdue(&topics, start + Duration::from_secs(29))
            .is_empty());
        assert_eq!(
            dog.overdue(&topics, start + Duration::from_secs(31)),
            ["live"]
        );
        assert!(dog
            .overdue(&topics, start + Duration::from_secs(60))
            .is_empty());
        dog.heard("live", start + Duration::from_secs(61));
        assert_eq!(
            dog.overdue(&topics, start + Duration::from_secs(100)),
            ["live"]
        );
    }

    #[test]
    fn the_stall_limit_never_drops_below_the_floor() {
        let mut dog = Watchdog::default();
        let topics = held(&[("live", 500)]);
        let start = Instant::now();
        dog.overdue(&topics, start);
        assert!(dog
            .overdue(&topics, start + Duration::from_secs(14))
            .is_empty());
        assert_eq!(
            dog.overdue(&topics, start + Duration::from_secs(16)),
            ["live"]
        );
    }

    #[test]
    fn dropping_a_topic_forgets_its_silence() {
        let mut dog = Watchdog::default();
        let start = Instant::now();
        dog.overdue(&held(&[("live", 1000)]), start);
        dog.overdue(&held(&[]), start + Duration::from_secs(20));
        assert!(dog
            .overdue(&held(&[("live", 1000)]), start + Duration::from_secs(21))
            .is_empty());
    }

    #[test]
    fn the_pause_between_looks_for_the_daemon_doubles_to_a_ceiling() {
        let secs: Vec<u64> = (1..=7).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(secs, [1, 2, 4, 5, 5, 5, 5]);
    }

    #[test]
    fn a_backend_event_says_why_and_when_it_looks_again() {
        assert_eq!(
            backend_event(None),
            json!({"event": "backend", "state": "up"})
        );
        let down = backend_event(Some(("no daemon", 2, Duration::from_secs(2))));
        assert_eq!(down["state"], "down");
        assert_eq!(down["retryMs"], 2000);
        assert_eq!(down["attempt"], 2);
    }

    #[test]
    fn a_sleep_ends_at_a_stop_and_not_at_a_new_topic() {
        let subs = Arc::new(Subs::default());
        let waiter = subs.clone();
        let handle = std::thread::spawn(move || {
            let began = Instant::now();
            waiter.sleep(Duration::from_secs(30));
            began.elapsed()
        });
        std::thread::sleep(Duration::from_millis(50));
        subs.set(BTreeMap::from([("live".to_string(), 1000)]));
        std::thread::sleep(Duration::from_millis(100));
        subs.stop();
        let slept = handle.join().unwrap();
        assert!(slept >= Duration::from_millis(140), "{slept:?}");
    }

    #[test]
    fn a_stop_wakes_a_waiter() {
        let subs = Arc::new(Subs::default());
        let waiter = subs.clone();
        let handle = std::thread::spawn(move || waiter.idle(0, Duration::from_secs(30)));
        std::thread::sleep(Duration::from_millis(50));
        subs.stop();
        assert!(!handle.join().unwrap(), "idle reports the stop");
    }

    #[test]
    fn a_change_made_before_the_wait_begins_is_not_slept_through() {
        let subs = Subs::default();
        let (_, seen) = subs.view().unwrap();
        subs.set(BTreeMap::from([("live".to_string(), 1000)]));
        let began = Instant::now();
        assert!(subs.idle(seen, Duration::from_secs(30)));
        assert!(
            began.elapsed() < Duration::from_secs(1),
            "{:?}",
            began.elapsed()
        );
    }
}
