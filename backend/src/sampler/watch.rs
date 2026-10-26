//! What windows watch besides display kinds: files by signature, the theme
//! files, the account files. Each window says what it holds in a `watch` frame
//! and the daemon looks once a second for all of them together, at one `stat`
//! per distinct path however many windows hold it, and pushes an `event` to
//! each window whose view changed. The windows no longer look on their own.
//!
//! A look is a poll, not a notification: the daemon has no way into inotify
//! without a foreign call, which this build does not make.

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{json, Value};

use super::bus::Subscriber;
use crate::platform::Backend;
use crate::request::Accounts;

/// Between two looks.
const TICK: Duration = Duration::from_secs(1);

/// While nobody watches anything.
const IDLE: Duration = Duration::from_secs(30);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Spec {
    pub paths: Vec<String>,
    pub chrome: bool,
    pub accounts: Option<(String, String)>,
}

impl Spec {
    /// `{"paths": [...], "chrome": true, "accounts": {"user", "home"}}`
    pub fn from_frame(frame: &Value) -> Spec {
        let paths = frame
            .get("paths")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(|p| p.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let accounts = frame.get("accounts").filter(|a| a.is_object()).map(|a| {
            let text = |key: &str| a.get(key).and_then(Value::as_str).unwrap_or("").to_string();
            (text("user"), text("home"))
        });
        Spec {
            paths,
            chrome: frame.get("chrome").and_then(Value::as_bool) == Some(true),
            accounts,
        }
    }
}

struct Entry {
    sub: Arc<Subscriber>,
    spec: Spec,
    // A new spec starts from nothing, so its first look reports.
    stamp: Option<Vec<(String, String)>>,
    chrome: Option<Value>,
    accounts: Option<Value>,
}

#[derive(Default)]
pub struct Watches {
    entries: Mutex<HashMap<u64, Entry>>,
    generation: Mutex<u64>,
    changed: Condvar,
    known: Mutex<HashMap<String, (String, String)>>,
}

fn signatures(doc: &Value) -> Vec<(String, String)> {
    doc.get("items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let text = |key: &str| {
                        item.get(key)
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string()
                    };
                    (text("path"), text("sig"))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn event(topic: &str, value: Value) -> Value {
    json!({"t": "event", "topic": topic, "value": value})
}

impl Watches {
    /// Replace what `sub` watches.
    pub fn set(&self, sub: &Arc<Subscriber>, spec: Spec) {
        lock(&self.entries).insert(
            sub.id(),
            Entry {
                sub: Arc::clone(sub),
                spec,
                stamp: None,
                chrome: None,
                accounts: None,
            },
        );
        self.poke();
    }

    pub fn remove(&self, sub: &Subscriber) {
        lock(&self.entries).remove(&sub.id());
        self.poke();
    }

    fn poke(&self) {
        *lock(&self.generation) += 1;
        self.changed.notify_all();
    }

    fn wait(&self, seen: u64, timeout: Duration) {
        let guard = lock(&self.generation);
        let _ = self
            .changed
            .wait_timeout_while(guard, timeout, |now| *now == seen);
    }

    /// Windows watching now.
    pub fn windows(&self) -> usize {
        lock(&self.entries).len()
    }

    /// One look for every window, then what changed goes out.
    pub fn look(&self) {
        let (union, chrome, accounts) = {
            let entries = lock(&self.entries);
            let mut union: Vec<String> = Vec::new();
            let mut chrome = false;
            let mut accounts: Vec<(String, String)> = Vec::new();
            for entry in entries.values() {
                for path in &entry.spec.paths {
                    if !union.contains(path) {
                        union.push(path.clone());
                    }
                }
                chrome |= entry.spec.chrome;
                if let Some(who) = &entry.spec.accounts {
                    if !accounts.contains(who) {
                        accounts.push(who.clone());
                    }
                }
            }
            (union, chrome, accounts)
        };
        if union.is_empty() && !chrome && accounts.is_empty() {
            return;
        }
        // The union is read once. A path one window sent badly would fail it for
        // everyone, so a failure is retried per window below.
        let stamped = crate::host::stamp_with(None, &union, &mut lock(&self.known)).ok();
        let chrome_doc = chrome
            .then(|| crate::host::chrome(Backend::Omarchy, None).ok())
            .flatten();
        let account_docs: HashMap<(String, String), Value> = accounts
            .into_iter()
            .filter_map(|(user, home)| {
                let doc = crate::host::accounts(
                    Backend::Omarchy,
                    None,
                    &Accounts {
                        user: user.clone(),
                        home: home.clone(),
                    },
                )
                .ok()?;
                Some(((user, home), doc))
            })
            .collect();

        let mut gone = Vec::new();
        let mut entries = lock(&self.entries);
        for (id, entry) in entries.iter_mut() {
            let mut out: Vec<Value> = Vec::new();
            if !entry.spec.paths.is_empty() {
                let doc = match &stamped {
                    Some(all) => view_of(all, &entry.spec.paths),
                    None => {
                        crate::host::stamp_with(None, &entry.spec.paths, &mut lock(&self.known))
                            .ok()
                    }
                };
                if let Some(doc) = doc {
                    let sigs = signatures(&doc);
                    if entry.stamp.as_ref() != Some(&sigs) {
                        entry.stamp = Some(sigs);
                        out.push(event("stamp", doc));
                    }
                }
            }
            if entry.spec.chrome {
                if let Some(doc) = &chrome_doc {
                    if entry.chrome.as_ref() != Some(doc) {
                        entry.chrome = Some(doc.clone());
                        out.push(event("chrome", doc.clone()));
                    }
                }
            }
            if let Some(who) = &entry.spec.accounts {
                if let Some(doc) = account_docs.get(who) {
                    if entry.accounts.as_ref() != Some(doc) {
                        entry.accounts = Some(doc.clone());
                        out.push(event("accounts", doc.clone()));
                    }
                }
            }
            for frame in out {
                if !entry.sub.send(frame) {
                    gone.push(*id);
                    break;
                }
            }
        }
        for id in gone {
            entries.remove(&id);
        }
    }

    /// Look once a second while anyone watches, until `stop` says so.
    pub fn run(&self, stop: &std::sync::atomic::AtomicBool) {
        while !stop.load(std::sync::atomic::Ordering::SeqCst) {
            let seen = *lock(&self.generation);
            let any = !lock(&self.entries).is_empty();
            if any {
                self.look();
            }
            self.wait(seen, if any { TICK } else { IDLE });
        }
    }
}

/// The entries of `all` for `paths`, in the order the window listed them.
fn view_of(all: &Value, paths: &[String]) -> Option<Value> {
    let items = all.get("items")?.as_array()?;
    let picked: Vec<Value> = paths
        .iter()
        .filter_map(|path| {
            items
                .iter()
                .find(|item| item.get("path").and_then(Value::as_str) == Some(path))
                .cloned()
        })
        .collect();
    Some(json!({"items": picked}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::bus::Bus;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("atmosd-watch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn spec(paths: &[&std::path::Path]) -> Spec {
        Spec {
            paths: paths.iter().map(|p| p.display().to_string()).collect(),
            ..Spec::default()
        }
    }

    #[test]
    fn a_spec_is_read_from_a_frame() {
        let frame = json!({"paths": ["/a", 7, "/b"], "chrome": true, "accounts": {"user": "me", "home": "/h"}});
        let spec = Spec::from_frame(&frame);
        assert_eq!(spec.paths, ["/a", "/b"]);
        assert!(spec.chrome);
        assert_eq!(spec.accounts, Some(("me".into(), "/h".into())));
        assert_eq!(Spec::from_frame(&json!({})), Spec::default());
        assert_eq!(Spec::from_frame(&json!({"accounts": null})).accounts, None);
    }

    #[test]
    fn a_new_watch_reports_at_once_and_then_only_a_change() {
        let dir = scratch("change");
        let file = dir.join("note.txt");
        std::fs::write(&file, "one").unwrap();
        let bus = Bus::default();
        let (sub, inbox) = bus.attach();
        let watches = Watches::default();
        watches.set(&sub, spec(&[&file]));
        watches.look();
        let first = inbox.try_recv().unwrap();
        assert_eq!(first["topic"], "stamp");
        assert_eq!(first["value"]["items"][0]["text"], "one");
        watches.look();
        assert!(inbox.try_recv().is_err(), "nothing moved");
        std::thread::sleep(Duration::from_millis(5));
        std::fs::write(&file, "three").unwrap();
        watches.look();
        let changed = inbox.try_recv().unwrap();
        assert_eq!(changed["value"]["items"][0]["text"], "three");
        // A new spec starts from nothing.
        watches.set(&sub, spec(&[&file]));
        watches.look();
        assert!(inbox.try_recv().is_ok());
    }

    #[test]
    fn two_windows_share_a_look_and_each_sees_only_its_own_paths() {
        let dir = scratch("share");
        let (a, b) = (dir.join("a"), dir.join("b"));
        std::fs::write(&a, "A").unwrap();
        std::fs::write(&b, "B").unwrap();
        let bus = Bus::default();
        let (one, one_inbox) = bus.attach();
        let (two, two_inbox) = bus.attach();
        let watches = Watches::default();
        watches.set(&one, spec(&[&a, &b]));
        watches.set(&two, spec(&[&b]));
        watches.look();
        let first = one_inbox.try_recv().unwrap();
        assert_eq!(first["value"]["items"].as_array().unwrap().len(), 2);
        let second = two_inbox.try_recv().unwrap();
        assert_eq!(second["value"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(second["value"]["items"][0]["text"], "B");
        std::thread::sleep(Duration::from_millis(5));
        std::fs::write(&a, "A2").unwrap();
        watches.look();
        assert!(one_inbox.try_recv().is_ok());
        assert!(
            two_inbox.try_recv().is_err(),
            "a file it does not watch moved"
        );
    }

    #[test]
    fn a_file_is_read_only_when_its_signature_moved() {
        let dir = scratch("cached");
        let file = dir.join("big");
        std::fs::write(&file, "x").unwrap();
        let mut known = HashMap::new();
        let path = file.display().to_string();
        crate::host::stamp_with(None, std::slice::from_ref(&path), &mut known).unwrap();
        // Same signature, so the cache wins even where the file's text is no longer the same.
        known.get_mut(&path).unwrap().1 = "remembered".into();
        let again = crate::host::stamp_with(None, std::slice::from_ref(&path), &mut known).unwrap();
        assert_eq!(again["items"][0]["text"], "remembered");
    }

    #[test]
    fn a_window_that_stops_listening_is_forgotten_and_a_bad_path_spares_the_others() {
        let dir = scratch("gone");
        let file = dir.join("f");
        std::fs::write(&file, "1").unwrap();
        let bus = Bus::default();
        let (good, good_inbox) = bus.attach();
        let (bad, bad_inbox) = bus.attach();
        let watches = Watches::default();
        watches.set(&good, spec(&[&file]));
        bad.set(HashMap::new());
        watches.set(
            &bad,
            Spec {
                paths: vec![String::new()],
                ..Spec::default()
            },
        );
        watches.look();
        assert!(good_inbox.try_recv().is_ok(), "the good window still hears");
        assert!(bad_inbox.try_recv().is_err());
        drop(good_inbox);
        std::thread::sleep(Duration::from_millis(5));
        std::fs::write(&file, "22").unwrap();
        watches.look();
        assert_eq!(watches.windows(), 1, "the closed window was dropped");
    }
}
