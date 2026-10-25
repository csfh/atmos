//! The reader side: what `display::load` calls.
//!
//! A fresh enough sample comes straight from the cache. Otherwise the daemon
//! is asked for the kind with interactive priority and the reply carries the
//! sample. `None` means there is no daemon to ask: a terminal command collects,
//! and a window's `serve` refuses (`require`).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
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

/// The window's one session to the daemon, as `serve` keeps it. A request goes
/// over it as a `req` frame and its answer comes back as a `res` frame with
/// the same id, so commands and barriers share a connection that is already
/// open instead of making one each.
pub struct Bridge {
    wire: Mutex<UnixStream>,
    pending: Mutex<HashMap<u64, Sender<Value>>>,
    next: AtomicU64,
}

/// Why a bridge call has no answer.
#[derive(Debug, PartialEq)]
pub enum Gone {
    /// The frame never left: the caller may ask another way.
    NotSent,
    /// It was sent and nothing came back. It may still be carried out.
    Lost,
}

impl Bridge {
    pub fn new(wire: UnixStream) -> Arc<Bridge> {
        Arc::new(Bridge {
            wire: Mutex::new(wire),
            pending: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
        })
    }

    /// Write one frame as a single line. Every writer of the session goes
    /// through here, so two frames never mix.
    pub fn send(&self, frame: &Value) -> bool {
        let line = format!("{frame}\n");
        let guard = lock(&self.wire);
        let mut out = &*guard;
        out.write_all(line.as_bytes()).is_ok()
    }

    pub fn call(&self, body: &Value, wait: Duration) -> std::result::Result<Value, Gone> {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = channel();
        lock(&self.pending).insert(id, tx);
        let frame = json!({"t": "req", "id": id, "body": body});
        if !self.send(&frame) {
            lock(&self.pending).remove(&id);
            return Err(Gone::NotSent);
        }
        let answer = rx.recv_timeout(wait);
        lock(&self.pending).remove(&id);
        answer.map_err(|_| Gone::Lost)
    }

    /// A `res` frame arrived.
    pub fn deliver(&self, id: u64, body: Value) {
        if let Some(waiter) = lock(&self.pending).remove(&id) {
            let _ = waiter.send(body);
        }
    }

    /// The session ended: everyone waiting is let go without an answer.
    pub fn close(&self) {
        lock(&self.pending).clear();
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

static BRIDGE: Mutex<Option<Arc<Bridge>>> = Mutex::new(None);
static CHANGED: Condvar = Condvar::new();
/// The window is looking for a daemon: starting one, or connecting to it.
static TRYING: AtomicBool = AtomicBool::new(false);
/// This process serves a window, which does nothing without the daemon
/// (docs/adr/0010): a read or write that finds none is refused, not run here.
static REQUIRED: AtomicBool = AtomicBool::new(false);

pub fn require(now: bool) {
    REQUIRED.store(now, Ordering::SeqCst);
}

pub fn required() -> bool {
    REQUIRED.load(Ordering::SeqCst)
}

/// Make `bridge` the session this process talks to the daemon over.
pub fn install(bridge: Arc<Bridge>) {
    *lock(&BRIDGE) = Some(bridge);
    CHANGED.notify_all();
}

/// The session ended; calls go back to the one-shot socket.
pub fn uninstall() {
    if let Some(bridge) = lock(&BRIDGE).take() {
        bridge.close();
    }
    CHANGED.notify_all();
}

/// Say whether this process is looking for a daemon right now.
pub fn trying(now: bool) {
    let held = lock(&BRIDGE);
    TRYING.store(now, Ordering::SeqCst);
    drop(held);
    CHANGED.notify_all();
}

/// Wait for the session to the daemon while the window is still looking for
/// one: a window that has just opened has no session for a moment, and a job
/// started in that moment is not to be refused. Returns at once when there is
/// a session, or when nobody is looking for a daemon.
pub fn wait_for_session(limit: Duration) -> bool {
    let until = std::time::Instant::now() + limit;
    let mut held = lock(&BRIDGE);
    loop {
        if held.is_some() {
            return true;
        }
        let left = until.saturating_duration_since(std::time::Instant::now());
        if !TRYING.load(Ordering::SeqCst) || left.is_zero() {
            return false;
        }
        held = CHANGED
            .wait_timeout(held, left)
            .map(|(guard, _)| guard)
            .unwrap_or_else(|err| err.into_inner().0);
    }
}

/// Ask the daemon one request over the window's session, if there is one.
pub fn call(body: &Value, wait: Duration) -> Option<Value> {
    bridge()?.call(body, wait).ok()
}

/// Send one frame over the window's session, if there is one.
pub fn send_frame(frame: &Value) -> bool {
    bridge().is_some_and(|bridge| bridge.send(frame))
}

fn bridge() -> Option<Arc<Bridge>> {
    lock(&BRIDGE).clone()
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
    let ask = command_ask(body, backend_id);
    if let Some(bridge) = bridge() {
        match bridge.call(&ask, COMMAND_WAIT) {
            Ok(reply) => return command_reply(&reply),
            Err(Gone::Lost) => return Command::Lost("the daemon did not answer".into()),
            // Nothing was sent: try the daemon's one-shot socket instead.
            Err(Gone::NotSent) => {}
        }
    }
    let Some(dir) = cache::dir_from_env() else {
        return Command::Unavailable;
    };
    command_in(&dir, body, backend_id)
}

/// This process's environment as the daemon is to run a child in it. The daemon
/// may be another copy of the program, so the app root is named even when
/// nothing set it: a read that looks at the app's own files (`search.query`)
/// means this window's.
///
/// A window that says its session is the daemon's (`ATMOS_SESSION=daemon`, which a window in a
/// sandbox sets, since it sees only a few variables and none of the desktop's) sends none: the
/// daemon runs what it asks in the daemon's own environment ([`or_own_env`]), which the
/// application's `oma` gave it from the real session (docs/adr/0014).
pub fn window_env() -> serde_json::Map<String, Value> {
    sent_env(session_is_the_daemons(
        std::env::var(SESSION).ok().as_deref(),
    ))
}

/// The variable that says a window has no session of its own, and the value that says whose it
/// has instead.
pub const SESSION: &str = "ATMOS_SESSION";
const DAEMONS: &str = "daemon";

fn session_is_the_daemons(value: Option<&str>) -> bool {
    value == Some(DAEMONS)
}

fn sent_env(daemons_session: bool) -> serde_json::Map<String, Value> {
    if daemons_session {
        serde_json::Map::new()
    } else {
        own_env()
    }
}

fn own_env() -> serde_json::Map<String, Value> {
    let mut env: serde_json::Map<String, Value> = std::env::vars()
        .map(|(key, value)| (key, Value::String(value)))
        .collect();
    env.entry("ATMOS_ROOT")
        .or_insert_with(|| Value::String(crate::platform::app_root().display().to_string()));
    env
}

/// What a command, a job or a settings topic runs with: the environment a window sent, or this
/// process's own when it sent none. A child is always given an environment, never the lack of
/// one, since the child runs with a cleared one (`PATH` is its first need).
pub fn or_own_env(sent: Vec<(String, String)>) -> Vec<(String, String)> {
    if !sent.is_empty() {
        return sent;
    }
    own_env()
        .into_iter()
        .filter_map(|(key, value)| Some((key, value.as_str()?.to_owned())))
        .collect()
}

fn command_ask(body: &Value, backend_id: &str) -> Value {
    json!({"op": "command", "request": body, "backend": backend_id, "env": window_env()})
}

pub fn command_in(dir: &Path, body: &Value, backend_id: &str) -> Command {
    let Ok(stream) = UnixStream::connect(cache::socket(dir)) else {
        return Command::Unavailable;
    };
    let ask = command_ask(body, backend_id);
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
    command_reply(&reply)
}

/// What the daemon's answer to a `command` op means for the caller.
fn command_reply(reply: &Value) -> Command {
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
        // A daemon that does not know this command yet, or does not treat it as
        // one: it refused before queueing anything.
        Some(message) if message.starts_with("bad command") || message == "not a command" => {
            Command::Unavailable
        }
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
    if let Some(bridge) = bridge() {
        let ask = json!({"op": "barrier", "topics": topics, "waitMs": wait.as_millis() as u64});
        match bridge.call(&ask, wait + Duration::from_secs(1)) {
            Ok(_) | Err(Gone::Lost) => return,
            Err(Gone::NotSent) => {}
        }
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
    fn the_environment_a_window_sends_names_its_app_root() {
        let env = window_env();
        // The daemon may be another copy of the program: the root is the
        // window's, whether or not something set it.
        let expected = std::env::var("ATMOS_ROOT")
            .unwrap_or_else(|_| crate::platform::app_root().display().to_string());
        assert_eq!(env["ATMOS_ROOT"], expected);
        assert!(
            env.contains_key("PATH"),
            "the rest of the environment goes too"
        );
    }

    #[test]
    fn a_child_is_given_what_a_window_sent_or_else_the_daemons_own_environment() {
        let sent = vec![("PATH".to_owned(), "/window/bin".to_owned())];
        assert_eq!(or_own_env(sent.clone()), sent, "a window's word is kept");
        let own = or_own_env(Vec::new());
        assert!(
            own.iter().any(|(k, _)| k == "PATH"),
            "the daemon's own PATH, not none: a child runs with a cleared environment"
        );
        assert!(own.iter().any(|(k, _)| k == "ATMOS_ROOT"));
    }

    #[test]
    fn a_window_whose_session_is_the_daemons_sends_no_environment() {
        assert!(session_is_the_daemons(Some("daemon")));
        for other in [None, Some(""), Some("window"), Some("Daemon")] {
            assert!(!session_is_the_daemons(other), "{other:?}");
        }
        assert!(sent_env(true).is_empty(), "nothing is sent");
        assert!(
            sent_env(false).contains_key("PATH"),
            "any other value keeps the old way"
        );
    }

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
    fn a_daemon_that_refuses_a_command_before_queueing_it_leaves_it_to_the_caller() {
        for refusal in [
            "bad command: unknown variant `write`, expected one of `version`",
            "not a command",
            "unknown op",
            "bad request: EOF while parsing",
        ] {
            assert!(
                matches!(
                    command_reply(&json!({"ok": false, "message": refusal})),
                    Command::Unavailable
                ),
                "{refusal}"
            );
        }
        // A command that was taken and then failed is not run again.
        assert!(matches!(
            command_reply(&json!({"ok": false, "message": "the command did not finish in time"})),
            Command::Lost(_)
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
