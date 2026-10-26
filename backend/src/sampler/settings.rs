//! The settings topics in the daemon (docs/adr/0011).
//!
//! They are not sampled on a clock. A topic is collected when a window first
//! holds it and nothing is cached, and again when something invalidates it
//! while it is held. Each invalidation bumps the topic's generation, and a
//! collection that began under an older one is thrown away and run again, so a
//! sample taken before a write never lands after it.
//!
//! A cached topic is only as good as the files it is read from (docs/adr/0015).
//! The daemon keeps their signatures from its last look, also while nothing
//! holds a topic, looks again before it sends a cached topic to a window that
//! holds it, and once a second while one is held; a topic whose files moved is
//! invalidated, whoever moved them.
//!
//! A collection needs the session's environment (`hyprctl`, `omarchy`), which a
//! daemon that outlived a login does not have: a window's session tells the
//! daemon its own (`env` frame), and the topic is collected in a child process
//! that carries the newest one, as a command is.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{json, Map, Value};

use super::cache::{self, Sample};
use super::daemon::Daemon;

/// How many times a collection that an invalidation overtook is run again
/// before the topic is left dirty for the next read to pick up.
pub(super) const RETRIES: usize = 4;

#[derive(Default)]
pub(super) struct Settings {
    generations: Mutex<HashMap<String, u64>>,
    stamped: Mutex<HashMap<String, u64>>,
    /// When a topic last failed to collect, so a window that keeps sending
    /// `sub` frames does not start the same failing children again and again.
    failed: Mutex<HashMap<String, std::time::Instant>>,
    env: Mutex<Vec<(String, String)>>,
    files: Mutex<Files>,
}

/// The files the topics are read from (`snapshot::topics::watched`), each with
/// the topics it belongs to, and their signatures at the daemon's last look.
#[derive(Default)]
struct Files {
    table: Vec<(PathBuf, Vec<&'static str>)>,
    seen: Option<HashMap<PathBuf, String>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

impl Settings {
    pub(super) fn generation(&self, topic: &str) -> u64 {
        lock(&self.generations).get(topic).copied().unwrap_or(0)
    }

    /// A time for a sample of `topic` that is later than the last one it
    /// had. Two different documents of one topic are never "the same sample",
    /// even within a millisecond.
    pub(super) fn stamp(&self, topic: &str) -> u64 {
        let mut stamped = lock(&self.stamped);
        let last = stamped.entry(topic.to_string()).or_insert(0);
        *last = cache::now_ms().max(*last + 1);
        *last
    }

    pub(super) fn failed(&self, topic: &str, failed: bool) {
        let mut map = lock(&self.failed);
        if failed {
            map.insert(topic.to_string(), std::time::Instant::now());
        } else {
            map.remove(topic);
        }
    }

    /// Failed to collect a moment ago: not tried again for a few seconds.
    pub(super) fn recently_failed(&self, topic: &str) -> bool {
        lock(&self.failed)
            .get(topic)
            .is_some_and(|at| at.elapsed() < std::time::Duration::from_secs(5))
    }

    /// The files to look at, as they are now. Called once, when the daemon
    /// starts, before any topic is collected: what a later look finds moved
    /// moved after that.
    pub(super) fn watch(&self, table: Vec<(PathBuf, Vec<&'static str>)>) {
        let mut files = lock(&self.files);
        files.table = table;
        files.seen = None;
        let Files { table, seen } = &mut *files;
        moved(table, seen);
    }

    pub(super) fn bump(&self, topic: &str) {
        *lock(&self.generations)
            .entry(topic.to_string())
            .or_insert(0) += 1;
    }

    pub(super) fn set_env(&self, env: Vec<(String, String)>) {
        if !env.is_empty() {
            *lock(&self.env) = env;
        }
    }

    fn env(&self) -> Vec<(String, String)> {
        lock(&self.env).clone()
    }
}

/// The topics whose files moved since `last`, which is then brought up to date.
/// The first look only remembers, so starting does not invalidate everything.
pub(super) fn moved(
    table: &[(std::path::PathBuf, Vec<&'static str>)],
    last: &mut Option<HashMap<std::path::PathBuf, String>>,
) -> Vec<&'static str> {
    let now: HashMap<std::path::PathBuf, String> = table
        .iter()
        .map(|(path, _)| (path.clone(), crate::host::signature(path)))
        .collect();
    let mut topics: Vec<&'static str> = Vec::new();
    if let Some(before) = last.as_ref() {
        for (path, topics_of) in table {
            if before.get(path) != now.get(path) {
                for topic in topics_of {
                    if !topics.contains(topic) {
                        topics.push(topic);
                    }
                }
            }
        }
    }
    *last = Some(now);
    topics
}

impl Daemon {
    /// Looks at the files the topics are read from and invalidates the topics
    /// whose files moved since the last look, whoever moved them: a write the
    /// daemon ran is read again too, which costs a collection and never shows
    /// an old value. The lock is held through the invalidation, so a hold that
    /// looks next finds the moved topics already dropped from the cache.
    pub(super) fn look_at_files(self: &Arc<Self>) -> Vec<&'static str> {
        let mut files = lock(&self.settings.files);
        let Files { table, seen } = &mut *files;
        let changed = moved(table, seen);
        if !changed.is_empty() {
            self.invalidate(&changed);
        }
        changed
    }

    /// Looks once a second, while a window holds a settings topic, at the
    /// files the topics are read from. The daemon knows the files, so no
    /// window has to say. While nothing is held it does nothing, and keeps
    /// what it last saw: the next hold looks before anything is sent.
    pub fn watch_settings(self: &Arc<Self>, stop: &std::sync::atomic::AtomicBool) {
        while !stop.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let held = crate::snapshot::topics::NAMES
                .iter()
                .any(|topic| self.bus.holds(topic));
            if held {
                self.look_at_files();
            }
        }
    }

    /// Read one settings topic from the machine, in a child that carries the
    /// newest window's environment.
    pub(super) fn collect_topic(&self, topic: &str) -> std::result::Result<Value, String> {
        // The newest window's, or the daemon's own when no window has one to tell: a window
        // whose session is the daemon's tells it nothing (docs/adr/0014).
        let env = super::client::or_own_env(self.settings.env());
        let body = json!({"op": "snapshot.topic", "topic": topic});
        let envelope = (self.run_command)("omarchy", &body, &env);
        if envelope["ok"].as_bool() == Some(true) {
            Ok(envelope["result"].clone())
        } else {
            Err(envelope["error"]["message"]
                .as_str()
                .unwrap_or("the snapshot could not be read")
                .to_string())
        }
    }

    /// Something changed what these topics show. Each is marked dirty, and one
    /// that a window holds is collected again at once. An inventory is dropped
    /// the same way; `live` is never old, so it is left alone.
    pub fn invalidate(self: &Arc<Self>, topics: &[&str]) {
        for topic in topics
            .iter()
            .filter(|topic| super::is_topic(topic) && **topic != "live")
        {
            if super::is_settings(topic) {
                self.settings.bump(topic);
            }
            cache::drop_kinds(&self.dir, &[topic]);
            // A sample of the topic is later than the mark just written.
            self.settings.stamp(topic);
            if self.bus.holds(topic) {
                let (daemon, topic) = (Arc::clone(self), topic.to_string());
                std::thread::spawn(move || {
                    let _ = daemon.sample(&topic, true);
                });
            }
        }
    }

    /// After a write: the topics that hold the keys it names are made current
    /// (docs/adr/0016). A key it read back once the machine had settled is
    /// patched into the cached topic, without a second collection. A topic
    /// that holds a key it named and could not show (the write failed, the
    /// key's tool failed, the machine had not settled, the key is not in what
    /// came back, or nothing was cached to patch) is invalidated, so it is
    /// read again and nothing stays as it was before the write.
    pub(super) fn after_write(self: &Arc<Self>, named: &[String], envelope: &Value) {
        let ok = envelope["ok"].as_bool() == Some(true);
        let result = &envelope["result"];
        let read = result
            .get("snapshot")
            .and_then(Value::as_object)
            .filter(|_| ok);
        let settled = result.get("settled").and_then(Value::as_bool) != Some(false);
        let mut by_topic: HashMap<&'static str, Map<String, Value>> = HashMap::new();
        for (key, value) in read.into_iter().flatten() {
            if let Some(topic) = crate::snapshot::keys::topic_of_key(key) {
                by_topic
                    .entry(topic)
                    .or_default()
                    .insert(key.clone(), value.clone());
            }
        }
        let mut stale: Vec<&'static str> = Vec::new();
        for (topic, patch) in by_topic {
            if !(settled && self.patch_topic(topic, patch)) {
                stale.push(topic);
            }
        }
        let failed = result
            .get("failed")
            .and_then(Value::as_object)
            .filter(|_| ok);
        let unread = named.iter().filter(|key| {
            let shown = read.is_some_and(|read| read.contains_key(key.as_str()));
            !shown || failed.is_some_and(|failed| failed.contains_key(key.as_str()))
        });
        let failed_keys = failed.into_iter().flat_map(|failed| failed.keys());
        for key in unread.chain(failed_keys) {
            if let Some(topic) = crate::snapshot::keys::topic_of_key(key) {
                stale.push(topic);
            }
        }
        // A topic that was patched and still holds a key that was not shown is read again too.
        stale.sort_unstable();
        stale.dedup();
        if !stale.is_empty() {
            self.invalidate(&stale);
        }
    }

    /// False when nothing was cached to patch.
    fn patch_topic(&self, topic: &str, patch: Map<String, Value>) -> bool {
        // A topic nobody collected yet is collected whole when it is held.
        let Some(held) = cache::read(&self.dir, topic) else {
            return false;
        };
        // A collection that began before this write read the machine from
        // before it: it must not land over the patch.
        self.settings.bump(topic);
        let mut value = held.value;
        if let Some(doc) = value.as_object_mut() {
            for (key, new) in patch {
                if let Some(failed) = doc.get_mut(crate::snapshot::topics::FAILED) {
                    if let Some(map) = failed.as_object_mut() {
                        map.remove(&key);
                    }
                }
                doc.insert(key, new);
            }
        }
        let sample = Sample {
            at_ms: self.settings.stamp(topic),
            took_ms: 0,
            value,
        };
        let _ = cache::write(&self.dir, topic, &sample);
        self.publish_sample(topic, &sample);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::daemon::tests::serve_commands;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    fn env() -> Vec<(String, String)> {
        vec![("HYPRLAND_INSTANCE_SIGNATURE".into(), "abc".into())]
    }

    /// A daemon whose snapshot child answers `{"n": <collection number>}`.
    fn counting(name: &str) -> (std::path::PathBuf, Arc<Daemon>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let (dir, daemon) = serve_commands(
            name,
            Box::new(move |_, body, env| {
                assert_eq!(body["op"], "snapshot.topic");
                assert!(env.iter().any(|(k, _)| k == "HYPRLAND_INSTANCE_SIGNATURE"));
                let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                json!({"ok": true, "result": {"n": n, "a": 1, "b": 2}})
            }),
        );
        (dir, daemon, calls)
    }

    #[test]
    fn a_changed_file_names_the_topics_it_belongs_to_and_the_first_look_names_none() {
        let dir = crate::sampler::cache::tests::scratch("snap-moved");
        let (a, b) = (dir.join("a.conf"), dir.join("b.conf"));
        std::fs::write(&a, "1").unwrap();
        std::fs::write(&b, "1").unwrap();
        let table = vec![
            (a.clone(), vec!["settings.look"]),
            (b.clone(), vec!["settings.network", "settings.look"]),
        ];
        let mut last = None;
        assert!(
            moved(&table, &mut last).is_empty(),
            "the first look remembers"
        );
        assert!(moved(&table, &mut last).is_empty(), "nothing moved");
        std::fs::write(&b, "two").unwrap();
        assert_eq!(
            moved(&table, &mut last),
            ["settings.network", "settings.look"]
        );
        std::fs::remove_file(&a).unwrap();
        assert_eq!(
            moved(&table, &mut last),
            ["settings.look"],
            "a removed file moved"
        );
    }

    /// A topic is collected with the newest window's environment, and with the daemon's own when
    /// no window has told it one: a window whose session is the daemon's tells it nothing
    /// (docs/adr/0014), and a child is never run with an environment of nothing.
    #[test]
    fn a_topic_is_collected_with_the_newest_windows_environment_or_else_the_daemons_own() {
        type Seen = Arc<std::sync::Mutex<Vec<Vec<(String, String)>>>>;
        let seen: Seen = Arc::default();
        let record = seen.clone();
        let (_dir, daemon) = serve_commands(
            "snap-ownenv",
            Box::new(move |_, _, env| {
                record.lock().unwrap().push(env.to_vec());
                json!({"ok": true, "result": {"n": 1}})
            }),
        );
        daemon.sample("settings.look", true).unwrap();
        daemon.settings.set_env(env());
        daemon.sample("settings.network", true).unwrap();
        let seen = seen.lock().unwrap();
        assert!(
            seen[0].iter().any(|(k, _)| k == "PATH"),
            "no window said: the daemon's own, which has a PATH"
        );
        assert_eq!(seen[1], env(), "a window said: exactly that");
    }

    #[test]
    fn what_an_earlier_daemon_cached_is_not_trusted() {
        let dir = cache::tests::scratch("snap-old");
        let old = Sample {
            at_ms: cache::now_ms(),
            took_ms: 1,
            value: json!({"theme": "old"}),
        };
        cache::write(&dir, "settings.look", &old).unwrap();
        cache::write(&dir, "hardware", &old).unwrap();
        std::thread::sleep(Duration::from_millis(3));
        let _daemon = Daemon::new(
            dir.clone(),
            Box::new(|_| Ok(json!({}))),
            crate::sampler::daemon::tests::test_config(&Arc::new(
                crate::sampler::daemon::tests::Dial(Default::default()),
            )),
        );
        assert!(cache::read(&dir, "settings.look").is_none());
        assert!(
            cache::read(&dir, "hardware").is_some(),
            "an inventory is kept"
        );
    }

    #[test]
    fn a_held_topic_is_collected_again_when_it_is_invalidated() {
        let (_dir, daemon, calls) = counting("snap-held");
        daemon.settings.set_env(env());
        let (subscriber, inbox) = daemon.bus.attach();
        subscriber.set(HashMap::from([("settings.look".to_string(), 0)]));
        daemon.sample("settings.look", true).unwrap();
        assert_eq!(
            inbox.recv_timeout(Duration::from_secs(2)).unwrap()["value"]["n"],
            1
        );
        daemon.invalidate(&["settings.look"]);
        let pushed = inbox.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(pushed["value"]["n"], 2, "{pushed}");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        // A topic nobody holds is only marked, not collected.
        daemon.invalidate(&["settings.network"]);
        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert!(cache::read(&daemon.dir, "settings.network").is_none());
    }

    #[test]
    fn a_collection_that_an_invalidation_overtook_never_lands() {
        let (go, wait) = std::sync::mpsc::channel::<()>();
        let (began, started) = std::sync::mpsc::channel::<usize>();
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let wait = Mutex::new(wait);
        let began = Mutex::new(began);
        let (_dir, daemon) = serve_commands(
            "snap-race",
            Box::new(move |_, _, _| {
                let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                began.lock().unwrap().send(n).unwrap();
                if n == 1 {
                    // The read the write overtook: it saw the machine from before.
                    wait.lock().unwrap().recv().unwrap();
                    return json!({"ok": true, "result": {"n": 1, "theme": "before"}});
                }
                json!({"ok": true, "result": {"n": n, "theme": "after"}})
            }),
        );
        daemon.settings.set_env(env());
        let (subscriber, inbox) = daemon.bus.attach();
        subscriber.set(HashMap::from([("settings.look".to_string(), 0)]));
        let reader = {
            let daemon = daemon.clone();
            std::thread::spawn(move || daemon.sample("settings.look", true))
        };
        assert_eq!(started.recv_timeout(Duration::from_secs(2)).unwrap(), 1);
        daemon.invalidate(&["settings.look"]);
        go.send(()).unwrap();
        let sample = reader.join().unwrap().unwrap();
        assert_eq!(
            sample.value["theme"], "after",
            "the old read was thrown away"
        );
        let pushed: Vec<Value> = inbox.try_iter().collect();
        assert!(
            pushed
                .iter()
                .all(|frame| frame["value"]["theme"] == "after"),
            "nothing from before the write was published: {pushed:?}"
        );
        assert_eq!(
            cache::read(&daemon.dir, "settings.look").unwrap().value["theme"],
            "after"
        );
    }

    #[test]
    fn a_collection_that_a_patch_overtook_never_lands_over_it() {
        let (go, wait) = std::sync::mpsc::channel::<()>();
        let (began, started) = std::sync::mpsc::channel::<usize>();
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let (wait, began) = (Mutex::new(wait), Mutex::new(began));
        let (_dir, daemon) = serve_commands(
            "snap-patchrace",
            Box::new(move |_, _, _| {
                let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                began.lock().unwrap().send(n).unwrap();
                if n == 2 {
                    // A read the write overtook: the machine as it was before.
                    wait.lock().unwrap().recv().unwrap();
                    return json!({"ok": true, "result": {"hyprLook": {"gapsIn": 1}}});
                }
                // Before the write the first read saw 1; once it is written, 7.
                let seen = if n == 1 { 1 } else { 7 };
                json!({"ok": true, "result": {"hyprLook": {"gapsIn": seen}}})
            }),
        );
        daemon.settings.set_env(env());
        let (subscriber, inbox) = daemon.bus.attach();
        subscriber.set(HashMap::from([("settings.look".to_string(), 0)]));
        daemon.sample("settings.look", true).unwrap();
        inbox.recv_timeout(Duration::from_secs(2)).unwrap();
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        // A file change starts a collection ...
        let reader = {
            let daemon = daemon.clone();
            std::thread::spawn(move || daemon.sample("settings.look", true))
        };
        assert_eq!(started.recv_timeout(Duration::from_secs(2)).unwrap(), 2);
        // ... and a write's read-back lands while it is still reading.
        daemon.after_write(&[], &json!({"ok": true, "result": {"snapshot": {"hyprLook": {"gapsIn": 7}}, "settled": true}}));
        go.send(()).unwrap();
        let sample = reader.join().unwrap().unwrap();
        assert_eq!(
            sample.value["hyprLook"]["gapsIn"], 7,
            "the old read was thrown away"
        );
        assert_eq!(
            cache::read(&daemon.dir, "settings.look").unwrap().value["hyprLook"]["gapsIn"],
            7,
            "the cache never held the value from before the write"
        );
        let pushed: Vec<Value> = inbox.try_iter().collect();
        assert!(
            pushed
                .iter()
                .all(|frame| frame["value"]["hyprLook"]["gapsIn"] == 7),
            "nothing from before the write was published after it: {pushed:?}"
        );
    }

    #[test]
    fn a_topic_that_failed_a_moment_ago_is_not_collected_again_by_every_sub_frame() {
        let healthy = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let ok_calls = Arc::new(AtomicUsize::new(0));
        let (up, counted) = (healthy.clone(), ok_calls.clone());
        let (_dir, daemon) = serve_commands(
            "snap-backoff",
            Box::new(move |_, _, _| {
                if up.load(Ordering::SeqCst) {
                    let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
                    json!({"ok": true, "result": {"n": n}})
                } else {
                    json!({"ok": false, "error": {"code": "command", "message": "hyprctl failed"}})
                }
            }),
        );
        // The tool fails: every collection does.
        for _ in 0..3 {
            let _ = daemon.sample("settings.look", true);
        }
        assert!(daemon.settings.recently_failed("settings.look"));
        healthy.store(true, Ordering::SeqCst);
        daemon.sample("settings.look", true).unwrap();
        assert!(
            !daemon.settings.recently_failed("settings.look"),
            "a success clears it"
        );
        assert_eq!(ok_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_write_that_read_keys_back_patches_the_cached_topic() {
        let (_dir, daemon, calls) = counting("snap-patch");
        daemon.settings.set_env(env());
        let (subscriber, inbox) = daemon.bus.attach();
        subscriber.set(HashMap::from([("settings.look".to_string(), 0)]));
        let first = daemon.sample("settings.look", true).unwrap();
        inbox.recv_timeout(Duration::from_secs(2)).unwrap();
        // `hyprLook` is a look key; `hostname` belongs to another topic.
        daemon.after_write(
            &[],
            &json!({"ok": true, "result": {
                "snapshot": {"hyprLook": {"gapsIn": 7}, "hostname": "box"},
                "settled": true,
            }}),
        );
        let held = cache::read(&daemon.dir, "settings.look").unwrap();
        assert_eq!(held.value["hyprLook"]["gapsIn"], 7);
        assert_eq!(held.value["b"], 2, "the rest of the topic is kept");
        assert!(held.at_ms > first.at_ms);
        assert_eq!(
            inbox.recv_timeout(Duration::from_secs(2)).unwrap()["value"]["hyprLook"]["gapsIn"],
            7
        );
        assert!(cache::read(&daemon.dir, "settings.system").is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1, "no second collection");
        // A key the machine had not settled on is read again.
        daemon.after_write(
            &[],
            &json!({"ok": true, "result": {"snapshot": {"hyprLook": {}}, "settled": false}}),
        );
        let again = inbox.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(again["value"]["n"], 2, "{again}");
    }
}
