//! `ratmos serve`: one long-lived process per window instead of one fork per
//! call.
//!
//! The wire format is newline-delimited JSON on stdin and stdout.
//!
//! Requests: `{"id": 7, "op": "settings.get", ...}`. The reply is the usual
//! envelope with the same `id` added. Each request runs on its own thread, so a
//! slow one never holds up a fast one, and replies may arrive in any order.
//!
//! `watch.set` is the one op `serve` answers itself. It tells the server what
//! to watch: `{"op": "watch.set", "paths": [...], "chrome": true,
//! "accounts": {"user": "...", "home": "..."}}`. From then on the server pushes
//! `{"event": "stamp"|"chrome"|"accounts", "result": ...}` whenever the thing
//! it watches changes, and says nothing while it does not. That replaces the
//! once-a-second polls the frontend used to run, each of which forked a
//! process.
//!
//! The server stops when stdin closes, which is how a window going away looks.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

use crate::error::Error;
use crate::platform::Backend;
use crate::request::{Accounts, Paths};
use crate::{error_envelope, handle, host};

pub struct Config {
    pub backend: Backend,
    pub root: Option<PathBuf>,
    pub sampler: Option<PathBuf>,
}

type Out = Arc<Mutex<Box<dyn Write + Send>>>;

#[derive(Default)]
struct Watch {
    paths: Vec<String>,
    chrome: bool,
    accounts: Option<(String, String)>,
    // A new subscription starts from nothing, so the first tick reports.
    reset: bool,
}

pub fn serve(config: Config, input: impl BufRead, out: Box<dyn Write + Send>) {
    let config = Arc::new(config);
    let out: Out = Arc::new(Mutex::new(out));
    let watch = Arc::new(Mutex::new(Watch::default()));
    let stop = Arc::new(AtomicBool::new(false));
    // A new subscription or a shutdown wakes the watcher at once instead of
    // letting it finish its second of sleep.
    let (wake, woken) = mpsc::channel::<()>();

    let watcher = {
        let (config, out, watch, stop) = (
            Arc::clone(&config),
            Arc::clone(&out),
            Arc::clone(&watch),
            Arc::clone(&stop),
        );
        thread::spawn(move || watch_loop(&config, &out, &watch, &stop, &woken))
    };

    for line in input.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                let error = Error::bad_request(format!("request: {err}"));
                send(&out, &with_id(error_envelope(config.backend, &error), None));
                continue;
            }
        };
        let id = value.get("id").cloned();
        if value.get("op").and_then(Value::as_str) == Some("watch.set") {
            let reply = set_watch(&config, &watch, &value, &wake);
            send(&out, &with_id(reply, id));
            continue;
        }
        let (config, out) = (Arc::clone(&config), Arc::clone(&out));
        thread::spawn(move || {
            let envelope = match handle(
                config.backend,
                config.root.as_deref(),
                &value,
                config.sampler.as_deref(),
            ) {
                Ok(envelope) => envelope,
                Err(err) => error_envelope(config.backend, &err),
            };
            send(&out, &with_id(envelope, id));
        });
    }

    stop.store(true, Ordering::SeqCst);
    let _ = wake.send(());
    let _ = watcher.join();
}

fn set_watch(config: &Config, watch: &Mutex<Watch>, request: &Value, wake: &Sender<()>) -> Value {
    let parsed = (|| -> Result<Watch, Error> {
        let paths = match request.get("paths") {
            None | Some(Value::Null) => Vec::new(),
            Some(list) => serde_json::from_value::<Vec<String>>(list.clone())
                .map_err(|err| Error::bad_request(format!("paths: {err}")).with_context("paths"))?,
        };
        let accounts = match request.get("accounts") {
            None | Some(Value::Null) => None,
            Some(entry) => {
                let user = entry.get("user").and_then(Value::as_str).unwrap_or("");
                let home = entry.get("home").and_then(Value::as_str).unwrap_or("");
                Some((user.to_string(), home.to_string()))
            }
        };
        Ok(Watch {
            paths,
            chrome: request.get("chrome").and_then(Value::as_bool) == Some(true),
            accounts,
            reset: true,
        })
    })();
    match parsed {
        Ok(next) => {
            if let Ok(mut slot) = watch.lock() {
                *slot = next;
            }
            let _ = wake.send(());
            crate::ok_envelope(config.backend, json!({ "watching": true }))
        }
        Err(err) => error_envelope(config.backend, &err),
    }
}

fn watch_loop(
    config: &Config,
    out: &Out,
    watch: &Mutex<Watch>,
    stop: &AtomicBool,
    woken: &Receiver<()>,
) {
    let mut last_stamp: Option<Vec<(String, String)>> = None;
    let mut last_chrome: Option<Value> = None;
    let mut last_accounts: Option<Value> = None;
    while !stop.load(Ordering::SeqCst) {
        let snapshot = match watch.lock() {
            Ok(mut slot) => {
                let copy = (
                    slot.paths.clone(),
                    slot.chrome,
                    slot.accounts.clone(),
                    slot.reset,
                );
                slot.reset = false;
                copy
            }
            Err(_) => break,
        };
        let (paths, chrome, accounts, reset) = snapshot;
        if reset {
            last_stamp = None;
            last_chrome = None;
            last_accounts = None;
        }
        let root = config.root.as_deref();

        if !paths.is_empty() {
            if let Ok(doc) = host::stamp(root, &Paths { paths }) {
                let sigs = signatures(&doc);
                if last_stamp.as_ref() != Some(&sigs) {
                    last_stamp = Some(sigs);
                    send(out, &event("stamp", doc));
                }
            }
        }
        if chrome {
            if let Ok(doc) = host::chrome(config.backend, root) {
                if last_chrome.as_ref() != Some(&doc) {
                    last_chrome = Some(doc.clone());
                    send(out, &event("chrome", doc));
                }
            }
        }
        if let Some((user, home)) = accounts {
            if let Ok(doc) = host::accounts(config.backend, root, &Accounts { user, home }) {
                if last_accounts.as_ref() != Some(&doc) {
                    last_accounts = Some(doc.clone());
                    send(out, &event("accounts", doc));
                }
            }
        }

        // One second between looks, cut short by a new subscription or a stop.
        let _ = woken.recv_timeout(Duration::from_secs(1));
    }
}

/// Path and signature pairs. The stamp doc also carries file text, which only
/// matters when the signature moved, so the signature alone decides.
fn signatures(doc: &Value) -> Vec<(String, String)> {
    doc.get("items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    (
                        item.get("path")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        item.get("sig")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn event(name: &str, result: Value) -> Value {
    json!({ "event": name, "result": result })
}

fn with_id(mut envelope: Value, id: Option<Value>) -> Value {
    if let (Some(id), Some(map)) = (id, envelope.as_object_mut()) {
        map.insert("id".into(), id);
    }
    envelope
}

fn send(out: &Out, value: &Value) {
    let Ok(text) = serde_json::to_string(value) else {
        return;
    };
    if let Ok(mut out) = out.lock() {
        // One lock per line, so replies and events never interleave.
        let _ = writeln!(out, "{text}");
        let _ = out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_pair_each_path_with_its_sig() {
        let doc = json!({"items": [
            {"path": "/a", "sig": "1", "text": "ignored"},
            {"path": "/b", "sig": "2"},
        ]});
        assert_eq!(
            signatures(&doc),
            vec![
                ("/a".to_string(), "1".to_string()),
                ("/b".to_string(), "2".to_string())
            ]
        );
    }

    #[test]
    fn signatures_survive_a_malformed_document() {
        assert!(signatures(&json!({})).is_empty());
        assert!(signatures(&json!({"items": "x"})).is_empty());
        assert!(signatures(&Value::Null).is_empty());
        assert_eq!(
            signatures(&json!({"items": [{}]})),
            vec![(String::new(), String::new())]
        );
        assert_eq!(
            signatures(&json!({"items": [{"path": 7, "sig": null}]})),
            vec![(String::new(), String::new())]
        );
    }

    #[test]
    fn an_event_names_itself_and_carries_the_result() {
        assert_eq!(
            event("stamp", json!({"n": 1})),
            json!({"event": "stamp", "result": {"n": 1}})
        );
    }

    #[test]
    fn with_id_echoes_the_id_of_the_request() {
        let reply = with_id(json!({"ok": true}), Some(json!(7)));
        assert_eq!(reply, json!({"ok": true, "id": 7}));
        let string_id = with_id(json!({"ok": true}), Some(json!("abc")));
        assert_eq!(string_id["id"], "abc");
    }

    #[test]
    fn without_an_id_the_envelope_is_unchanged() {
        assert_eq!(with_id(json!({"ok": true}), None), json!({"ok": true}));
    }

    #[test]
    fn an_id_cannot_be_added_to_a_non_object() {
        assert_eq!(with_id(json!([1]), Some(json!(1))), json!([1]));
    }
}
