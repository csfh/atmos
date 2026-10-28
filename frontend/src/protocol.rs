//! The wire format of `atmosd serve` (docs/protocol.md): the bodies the window sends, one JSON
//! line each, and what comes back, a reply to a request or an event the server pushes.
//!
//! The window builds no command line. It names an op, and the backend checks the arguments.
//! `tests/ops.rs` pins every op named here to the backend's source.

use serde_json::{Map, Value, json};

/// The ops this window sends. Each is a `Request` variant in `backend/src/request.rs` or one of
/// the two ops `serve` answers itself (`backend/src/serve.rs`).
pub const OPS: &[&str] = &[
    "version",
    "platform",
    "sub.set",
    "snapshot.invalidate",
    "job.start",
    "write",
    "settings.get",
    "settings.set",
    "snapshot.keys",
];

/// The server restarts after this long at the first failure, doubling to the ceiling. A server that
/// dies sooner than `STABLE_MS` after it started counts as a failed start (BackendProtocol.js).
pub const STABLE_MS: u64 = 5000;
const RESTART_CEILING_MS: u64 = 30_000;

pub fn restart_delay_ms(failures: u32) -> u64 {
    let n = failures.max(1);
    (1000u64 << (n - 1).min(5)).min(RESTART_CEILING_MS)
}

/// `error.code` of a failed reply (docs/protocol.md). `Other` keeps a code a newer backend adds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Code {
    BadRequest,
    NotFound,
    Denied,
    Io,
    Command,
    Timeout,
    Failed,
    Unavailable,
    Other(String),
}

impl Code {
    fn parse(code: &str) -> Self {
        match code {
            "bad_request" => Self::BadRequest,
            "not_found" => Self::NotFound,
            "denied" => Self::Denied,
            "io" => Self::Io,
            "command" => Self::Command,
            "timeout" => Self::Timeout,
            "failed" => Self::Failed,
            "unavailable" => Self::Unavailable,
            other => Self::Other(other.into()),
        }
    }
}

/// A failed reply. A failure is never shown as an empty value (docs/adr/0006).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub code: Code,
    pub message: String,
    pub context: Option<String>,
}

impl Failure {
    /// What the window makes up when the server cannot answer, so a caller handles one shape.
    pub fn unavailable(message: &str) -> Self {
        Self {
            code: Code::Unavailable,
            message: message.into(),
            context: None,
        }
    }
}

/// A reply envelope: the result, or why there is none.
pub type Reply = Result<Value, Failure>;

/// The daemon's state as the server reports it with a `backend` event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Daemon {
    Up,
    Down {
        why: String,
        attempt: u32,
        retry_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Backend(Daemon),
    /// One sample of a held topic. `result` is `None` when `stale`, and `error` says why.
    Sample {
        kind: String,
        sampled_at: Option<u64>,
        stale: bool,
        result: Option<Value>,
        error: Option<String>,
    },
    Stamp(Value),
    Chrome(Value),
    Accounts(Value),
    Job(Value),
    /// An event this window does not know. A newer backend may send more.
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Incoming {
    Reply { id: Option<u64>, reply: Reply },
    Event(Event),
}

pub fn version() -> Value {
    json!({ "op": "version" })
}

pub fn platform() -> Value {
    json!({ "op": "platform" })
}

/// Hold these topics: topic name to an interval in milliseconds, 0 for "when it changes". It
/// replaces the previous set; no topics releases everything.
pub fn sub_set(topics: &[(&str, u64)]) -> Value {
    let mut map = Map::new();
    for (name, interval_ms) in topics {
        map.insert((*name).into(), json!({ "intervalMs": interval_ms }));
    }
    json!({ "op": "sub.set", "topics": map })
}

/// Run a job from the backend's registry in the daemon (`backend/src/command/entries.rs`). `cid`
/// is this window's own id for it; the `job` events that come back carry it. The window sends a
/// name and arguments, never a command line.
pub fn job_start(cid: &str, name: &str, args: &Value, label: &str) -> Value {
    json!({ "op": "job.start", "cid": cid, "name": name, "args": args, "label": label })
}

/// A typed write: a registry entry by `name`, with `args` the entry checks
/// (`backend/src/command/entries.rs`). The window never sends a command line.
pub fn write(name: &str, args: &Value) -> Value {
    json!({ "op": "write", "name": name, "args": args })
}

/// One setting's stored value, `null` when it is unset.
pub fn settings_get(domain: &str) -> Value {
    json!({ "op": "settings.get", "domain": domain })
}

/// Stores a setting. `value` must be what the domain's type is: an integer domain refuses
/// `80.0`, so a number from a field is sent as an integer. `readback` names the snapshot keys the
/// setting changes, which the reply carries as the machine reports them after the write
/// (docs/adr/0005); none named sends no `readback`, as `Requests.settingsSet` does.
pub fn settings_set(domain: &str, value: &Value, readback: &[&str]) -> Value {
    let mut body = json!({ "op": "settings.set", "domain": domain, "value": value });
    if !readback.is_empty() {
        body["readback"] = json!(readback);
    }
    body
}

/// Read these snapshot keys now, and say whether they show `expect` (`Requests.snapshotKeys`).
pub fn snapshot_keys(keys: &[&str], expect: &Value) -> Value {
    json!({ "op": "snapshot.keys", "keys": keys, "expect": expect })
}

/// Read these topics again. None named means the five settings topics. What a Refresh sends.
pub fn snapshot_invalidate(topics: &[&str]) -> Value {
    json!({ "op": "snapshot.invalidate", "topics": topics })
}

/// One request line, newline-terminated. The caller's `id` wins over a stray one in the body.
pub fn encode(id: u64, body: &Value) -> String {
    let mut body = body.clone();
    if let Some(map) = body.as_object_mut() {
        map.insert("id".into(), json!(id));
    }
    let mut line = body.to_string();
    line.push('\n');
    line
}

/// Read one line from the server. `None` for a line that is neither a reply nor an event.
pub fn decode(line: &str) -> Option<Incoming> {
    let value: Value = serde_json::from_str(line.trim()).ok()?;
    let map = value.as_object()?;
    if let Some(name) = map.get("event").and_then(Value::as_str) {
        return Some(Incoming::Event(event(name, map)));
    }
    let ok = map.get("ok")?.as_bool()?;
    let id = map.get("id").and_then(Value::as_u64);
    let reply = if ok {
        Ok(map.get("result").cloned().unwrap_or(Value::Null))
    } else {
        Err(failure(map.get("error")))
    };
    Some(Incoming::Reply { id, reply })
}

fn failure(error: Option<&Value>) -> Failure {
    let text = |key: &str| {
        error
            .and_then(|e| e.get(key))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    Failure {
        code: Code::parse(&text("code").unwrap_or_default()),
        message: text("message").unwrap_or_default(),
        context: text("context"),
    }
}

fn event(name: &str, map: &Map<String, Value>) -> Event {
    let result = || map.get("result").cloned().unwrap_or(Value::Null);
    match name {
        "backend" => Event::Backend(match map.get("state").and_then(Value::as_str) {
            Some("up") => Daemon::Up,
            _ => Daemon::Down {
                why: map
                    .get("why")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .into(),
                attempt: map.get("attempt").and_then(Value::as_u64).unwrap_or(0) as u32,
                retry_ms: map.get("retryMs").and_then(Value::as_u64).unwrap_or(0),
            },
        }),
        "sample" => Event::Sample {
            kind: map
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .into(),
            sampled_at: map.get("sampledAt").and_then(Value::as_u64),
            stale: map.get("stale").and_then(Value::as_bool).unwrap_or(false),
            result: map.get("result").cloned(),
            error: map.get("error").and_then(Value::as_str).map(str::to_owned),
        },
        "stamp" => Event::Stamp(result()),
        "chrome" => Event::Chrome(result()),
        "accounts" => Event::Accounts(result()),
        "job" => Event::Job(Value::Object(map.clone())),
        other => Event::Other(other.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_line_carries_the_callers_id() {
        let line = encode(7, &json!({ "op": "version", "id": 99 }));
        assert!(line.ends_with('\n'));
        let back: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(back["id"], 7);
        assert_eq!(back["op"], "version");
    }

    #[test]
    fn a_reply_keeps_its_id_and_result() {
        let got = decode(r#"{"id":7,"ok":true,"result":"0.4.0"}"#).unwrap();
        assert_eq!(
            got,
            Incoming::Reply {
                id: Some(7),
                reply: Ok(json!("0.4.0"))
            }
        );
    }

    #[test]
    fn a_failure_keeps_its_code_message_and_context() {
        let line =
            r#"{"ok":false,"result":null,"error":{"code":"denied","message":"no","context":"/x"}}"#;
        let Some(Incoming::Reply { id: None, reply }) = decode(line) else {
            panic!("not a reply");
        };
        assert_eq!(
            reply,
            Err(Failure {
                code: Code::Denied,
                message: "no".into(),
                context: Some("/x".into())
            })
        );
    }

    #[test]
    fn a_code_from_a_newer_backend_is_kept() {
        let line = r#"{"id":1,"ok":false,"error":{"code":"later","message":""}}"#;
        let Some(Incoming::Reply { reply: Err(f), .. }) = decode(line) else {
            panic!("not a failure");
        };
        assert_eq!(f.code, Code::Other("later".into()));
    }

    #[test]
    fn backend_events_say_up_and_down() {
        assert_eq!(
            decode(r#"{"event":"backend","state":"up"}"#),
            Some(Incoming::Event(Event::Backend(Daemon::Up)))
        );
        let down =
            r#"{"event":"backend","state":"down","why":"no daemon","attempt":2,"retryMs":2000}"#;
        assert_eq!(
            decode(down),
            Some(Incoming::Event(Event::Backend(Daemon::Down {
                why: "no daemon".into(),
                attempt: 2,
                retry_ms: 2000
            })))
        );
    }

    #[test]
    fn a_sample_carries_its_clock_and_a_stale_one_has_no_result() {
        let sample = r#"{"event":"sample","kind":"live","sampledAt":1760000000123,"tookMs":9,"result":{"a":1}}"#;
        assert_eq!(
            decode(sample),
            Some(Incoming::Event(Event::Sample {
                kind: "live".into(),
                sampled_at: Some(1_760_000_000_123),
                stale: false,
                result: Some(json!({ "a": 1 })),
                error: None,
            }))
        );
        let stale = r#"{"event":"sample","kind":"live","stale":true,"error":"x"}"#;
        assert!(matches!(
            decode(stale),
            Some(Incoming::Event(Event::Sample {
                stale: true,
                result: None,
                error: Some(why),
                ..
            })) if why == "x"
        ));
    }

    #[test]
    fn junk_is_not_a_line() {
        for line in ["", "nonsense", "[1]", r#"{"result":1}"#] {
            assert_eq!(decode(line), None, "{line:?}");
        }
    }

    #[test]
    fn a_subscription_names_each_topic_and_its_interval() {
        let body = sub_set(&[("live", 2000), ("hardware", 0)]);
        assert_eq!(body["op"], "sub.set");
        assert_eq!(body["topics"]["live"]["intervalMs"], 2000);
        assert_eq!(body["topics"]["hardware"]["intervalMs"], 0);
        assert_eq!(sub_set(&[])["topics"], json!({}));
    }

    #[test]
    fn a_write_and_a_setting_name_what_they_are_about() {
        assert_eq!(
            write("process.signal", &json!({"pid": 4242, "signal": "TERM"})),
            json!({"op": "write", "name": "process.signal", "args": {"pid": 4242, "signal": "TERM"}})
        );
        assert_eq!(
            settings_get("sentinelCpuPercent"),
            json!({"op": "settings.get", "domain": "sentinelCpuPercent"})
        );
        let set = settings_set("sentinelCpuPercent", &json!(80), &[]);
        assert_eq!(set["value"], 80);
        assert!(
            set["value"].is_i64(),
            "an integer domain takes an integer, not 80.0"
        );
        assert!(set.get("readback").is_none(), "none named, none sent");
        assert_eq!(
            settings_set("tweaks.middlePaste", &json!(true), &["tweaks"]),
            json!({"op": "settings.set", "domain": "tweaks.middlePaste", "value": true,
                "readback": ["tweaks"]})
        );
    }

    #[test]
    fn restarts_back_off_to_the_ceiling() {
        assert_eq!(restart_delay_ms(0), 1000);
        assert_eq!(restart_delay_ms(1), 1000);
        assert_eq!(restart_delay_ms(3), 4000);
        assert_eq!(restart_delay_ms(50), 30_000);
    }
}
