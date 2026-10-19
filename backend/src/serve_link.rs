//! What `serve` pushes to its window besides replies: sampled topics.
//!
//! The window says what it holds with `sub.set`. This module keeps one session
//! to the daemon (see `sampler::session`) and turns the daemon's events into
//! `{"event":"sample",...}` lines. With no daemon, or one that does not speak
//! sessions, it samples the held topics itself on the same clock, so the window
//! sees the same events either way.
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

/// How long the local sampler runs before it tries for a daemon again.
const RETRY: Duration = Duration::from_secs(5);

/// The slowest a topic that changes rarely is looked at locally.
const SLOW_LOCAL: Duration = Duration::from_secs(30);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

#[derive(Default)]
struct State {
    topics: BTreeMap<String, u64>,
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
                if !crate::display::is_kind(kind) {
                    return Err(format!("topics: unknown display kind {kind}"));
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
    if frame.get("t").and_then(Value::as_str) != Some("event") {
        return None;
    }
    let kind = frame.get("topic")?.as_str()?;
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

/// Run until `subs` stops. With a daemon to be had, this keeps one session to
/// it for as long as the window lives: it carries the held topics and, through
/// `client::Bridge`, the window's commands and barriers. Without one it samples
/// the held topics here.
pub fn run(config: &Config, out: &Out, subs: &Subs) {
    let dir = (config.root.is_none() && config.backend == crate::platform::Backend::Omarchy)
        .then(cache::dir_from_env)
        .flatten();
    let mut told = String::new();
    let mut last_launch: Option<Instant> = None;
    while !subs.stopped() {
        if let Some(dir) = &dir {
            let recently = last_launch.is_some_and(|at| at.elapsed() < RETRY);
            if !recently {
                last_launch = Some(Instant::now());
                launch::ensure(dir);
            }
            match connect(dir) {
                Ok(stream) => {
                    told.clear();
                    let began = Instant::now();
                    session(stream, out, subs);
                    // A daemon that drops sessions at once must not be spun on.
                    if began.elapsed() < Duration::from_secs(1) {
                        let seen = subs.view().map_or(0, |(_, generation)| generation);
                        subs.idle(seen, Duration::from_secs(1));
                    }
                    continue;
                }
                Err(why) => {
                    if told != why {
                        eprintln!("ratmos serve: {why}; sampling in this window instead");
                        told = why;
                    }
                }
            }
        }
        let Some((topics, seen)) = subs.view() else {
            return;
        };
        if topics.is_empty() {
            // Nothing to sample; look for a daemon again after a while. The
            // generation seen with the topics goes into the wait, so a
            // `sub.set` that lands in between wakes it.
            let wait = if dir.is_some() {
                RETRY
            } else {
                Duration::from_secs(3600)
            };
            subs.idle(seen, wait);
        } else {
            local(config, dir.as_deref().map(|_| RETRY), out, subs);
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
    if reply.get("t").and_then(Value::as_str) != Some("ready") {
        return Err(
            "the running daemon is older than this build and has no sessions (restart ratmos.service)"
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

    let mut sent: Option<u64> = None;
    while alive.load(Ordering::SeqCst) {
        let Some((topics, generation)) = subs.view() else {
            break;
        };
        if sent != Some(generation) {
            if writeln!(&stream, "{}", sub_frame(&topics)).is_err() {
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

/// Sample the held topics here, on the window's clock. `retry` bounds how long
/// this runs when a daemon may appear; with none to wait for it runs until the
/// subscriptions stop.
fn local(config: &Config, retry: Option<Duration>, out: &Out, subs: &Subs) {
    let began = Instant::now();
    let mut last: HashMap<String, (Instant, u64)> = HashMap::new();
    let mut dog = Watchdog::default();
    loop {
        let Some((topics, generation)) = subs.view() else {
            return;
        };
        if topics.is_empty() || retry.is_some_and(|limit| began.elapsed() >= limit) {
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
    if config.root.is_none() && config.backend == crate::platform::Backend::Omarchy {
        // A daemon that cannot hold a session may still hold the sample.
        if let Some(sample) = client::fetch_sample(kind) {
            return Ok((sample.at_ms, sample.took_ms, sample.value));
        }
    }
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
