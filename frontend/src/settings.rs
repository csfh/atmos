//! The settings snapshot as the window knows it: what the daemon's settings topics and the writes'
//! read-backs last said of each key, shared by every page (`Omarchy.qml` is the QML window's). The
//! shell holds the topics a page needs while it is shown (`wanted` in `app.rs`) and hands every
//! sample to one [`Settings`]; a page reads its keys from it and gives it what its writes read
//! back.
//!
//! A key a producer could not read keeps the value it had and says why, and a key never read is a
//! reason, never a default (docs/adr/0006).

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use odk::prelude::*;
use serde_json::Value;

use crate::client::Client;
use crate::protocol;

/// A snapshot key a page reads, the settings topic that holds it (`backend/src/snapshot/keys.rs`;
/// `tests/pins.rs` holds the pair to it), and what the page says when it cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    pub key: &'static str,
    pub topic: &'static str,
    pub unread: &'static str,
}

/// The topics `held` is in, each once, held at 0: "when it changes" (docs/adr/0011).
pub fn topics(held: &[Held]) -> Vec<(&'static str, u64)> {
    let mut out: Vec<(&'static str, u64)> = Vec::new();
    for h in held {
        if !out.iter().any(|(topic, _)| *topic == h.topic) {
            out.push((h.topic, 0));
        }
    }
    out
}

/// What a Refresh button reads again: the five settings topics and the five inventories
/// (`SnapshotGroups.topicsForGroups("all")`).
pub const ALL_TOPICS: &[&str] = &[
    "settings.look",
    "settings.network",
    "settings.accounts",
    "settings.system",
    "settings.host",
    "hardware",
    "disks",
    "services",
    "software",
    "diagnostics",
];

/// Whether `topic` is one of the daemon's settings topics.
pub fn is_topic(topic: &str) -> bool {
    topic.starts_with("settings.")
}

/// What the window knows of a key.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Known<'a> {
    /// Nothing has said yet.
    Waiting,
    Value(&'a Value),
    /// The key has never been read, and this is why.
    Unknown(&'a str),
}

#[derive(Debug, Clone, Default)]
pub struct Settings {
    values: HashMap<String, Value>,
    /// Why a key could not be read, until a sample or a read-back carries it again.
    failed: HashMap<String, String>,
    /// Why a topic could not be collected, until a sample of it arrives. It is said of every key
    /// of the topic that has no value of its own.
    topic_failed: HashMap<String, String>,
    /// The keys each topic has carried, so that a failure of the topic is said of them too.
    seen: HashMap<String, HashSet<String>>,
    /// The last sample taken in, by topic: one that is not after it is not taken.
    last: HashMap<String, u64>,
    /// The newest write of each snapshot key: only its reply is taken in.
    sent: HashMap<&'static str, u64>,
    /// When each key was last written, on the daemon's clock: a sample that began before it is
    /// not taken for that key (`Ops.dropStale`).
    wrote: HashMap<String, u64>,
}

/// Now, in milliseconds since the epoch: the clock `sampledAt` is on, since the daemon runs on
/// the same machine.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

impl Settings {
    /// A sample of a settings topic. False when it is not after the last of that topic, or is not
    /// a settings topic.
    pub fn adopt(&mut self, topic: &str, sampled_at: u64, doc: &Value) -> bool {
        if !is_topic(topic) || self.last.get(topic).is_some_and(|last| sampled_at <= *last) {
            return false;
        }
        self.last.insert(topic.to_owned(), sampled_at);
        self.topic_failed.remove(topic);
        let seen = self.seen.entry(topic.to_owned()).or_default();
        let Some(doc) = doc.as_object() else {
            return true;
        };
        let wrote = &self.wrote;
        let fresh = |key: &String| wrote.get(key).is_none_or(|at| *at <= sampled_at);
        for (key, value) in doc
            .iter()
            .filter(|(key, _)| *key != "_failed" && fresh(key))
        {
            seen.insert(key.clone());
            self.values.insert(key.clone(), value.clone());
            self.failed.remove(key);
        }
        if let Some(failed) = doc.get("_failed").and_then(Value::as_object) {
            for (key, why) in failed {
                seen.insert(key.clone());
                self.failed.insert(key.clone(), reason(why));
            }
        }
        true
    }

    /// The topic could not be collected: its keys keep what they had, and say why.
    pub fn topic_failed(&mut self, topic: &str, why: &str) {
        if !is_topic(topic) {
            return;
        }
        self.topic_failed.insert(topic.to_owned(), why.to_owned());
        for key in self.seen.get(topic).into_iter().flatten() {
            self.failed.insert(key.clone(), why.to_owned());
        }
    }

    /// A write's reply: the keys it read back, as the machine reports them, and the ones it could
    /// not read. A setting of a whole key that read nothing back shows the value the backend says
    /// it set (`mutationAnswered`). `sent_at` is when the write was sent: a sample that began
    /// before it is not taken for these keys.
    pub fn adopt_reply(&mut self, result: &Value, sent_at: u64) {
        let mut read = result
            .get("snapshot")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let (Some(domain), Some(value)) = (
            result.get("domain").and_then(Value::as_str),
            result.get("value"),
        ) && !domain.contains('.')
            && !read.contains_key(domain)
            && (value.is_string() || value.is_number() || value.is_boolean())
        {
            read.insert(domain.to_owned(), value.clone());
        }
        for (key, value) in read {
            self.wrote.insert(key.clone(), sent_at);
            self.failed.remove(&key);
            self.values.insert(key, value);
        }
        if let Some(failed) = result.get("failed").and_then(Value::as_object) {
            for (key, why) in failed {
                self.failed.insert(key.clone(), reason(why));
            }
        }
    }

    /// Sends a write that reads back `key`, and takes in what the newest write of the key read
    /// back. A refusal keeps what is shown and says `refused` with the backend's reason.
    pub fn send(
        &mut self,
        client: &Client,
        key: &'static str,
        body: &Value,
        refused: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let seq = self.sent.get(key).copied().unwrap_or(0) + 1;
        self.sent.insert(key, seq);
        let refused = refused.into();
        let sent_at = now_ms();
        let reply = client.request(body);
        cx.spawn_in(window, async move |this, cx| {
            let outcome = reply.await;
            let _ = this.update_in(cx, |settings, window, cx| match outcome {
                // A newer write of the key is on its way; its reply is the one to show.
                Ok(result) if settings.sent.get(key) == Some(&seq) => {
                    settings.adopt_reply(&result, sent_at);
                    cx.notify();
                }
                Ok(_) => {}
                Err(failure) => toast(
                    window,
                    cx,
                    Toast::new(refused)
                        .description(failure.message)
                        .variant(Tone::Danger),
                ),
            });
        })
        .detach();
    }

    pub fn known(&self, held: &Held) -> Known<'_> {
        match (self.values.get(held.key), self.failure(held)) {
            (Some(value), _) => Known::Value(value),
            (None, Some(why)) => Known::Unknown(why),
            (None, None) => Known::Waiting,
        }
    }

    /// The value of a key, when one has been read.
    pub fn value(&self, held: &Held) -> Option<&Value> {
        self.values.get(held.key)
    }

    /// Why a key could not be read, while it could not.
    pub fn failure(&self, held: &Held) -> Option<&str> {
        self.failed
            .get(held.key)
            .or_else(|| {
                (!self.values.contains_key(held.key))
                    .then(|| self.topic_failed.get(held.topic))
                    .flatten()
            })
            .map(String::as_str)
    }

    /// The keys of `held` that could not be read, and why, in its order.
    pub fn failures<'a>(&'a self, held: &'a [Held]) -> Vec<(&'a Held, &'a str)> {
        held.iter()
            .filter_map(|h| self.failure(h).map(|why| (h, why)))
            .collect()
    }
}

/// The snapshot key a settings domain is read from: `hyprInput` of `hyprInput.accelProfile`
/// (`snapshotHead` in `Settings.js`).
pub fn head(domain: &str) -> &str {
    domain.split_once('.').map_or(domain, |(head, _)| head)
}

/// The snapshot keys a write of `domain` reads back, as `Settings.commandFor` names them, so the
/// reply carries them as the machine reports them (docs/adr/0005). `tests/settings_parity.rs`
/// holds every domain a page writes to it.
pub fn readback(domain: &str) -> Vec<&str> {
    match head(domain) {
        "hyprInput" => vec!["hyprInput", "hyprInputManaged"],
        "hyprLook" => vec!["hyprLook", "hyprLookManaged"],
        other => vec![other],
    }
}

/// The `settings.set` that writes `value` to `domain`, with the keys it reads back.
pub fn write(domain: &str, value: &Value) -> Value {
    protocol::settings_set(domain, value, &readback(domain))
}

/// A producer's reason as text: it is a string, and anything else is shown as it came.
fn reason(why: &Value) -> String {
    match why {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("message")
            .and_then(Value::as_str)
            .map_or_else(|| why.to_string(), str::to_owned),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const INPUT: Held = Held {
        key: "hyprInput",
        topic: "settings.look",
        unread: "",
    };
    const TWEAKS: Held = Held {
        key: "tweaks",
        topic: "settings.system",
        unread: "",
    };
    const BROWSER: Held = Held {
        key: "browser",
        topic: "settings.system",
        unread: "",
    };

    #[test]
    fn a_sample_is_taken_once_and_a_failed_key_keeps_its_value_and_says_why() {
        let mut s = Settings::default();
        assert_eq!(s.known(&TWEAKS), Known::Waiting);
        assert!(
            !s.adopt("live", 5, &json!({"tweaks": {}})),
            "not a settings topic"
        );
        assert!(s.adopt(
            "settings.system",
            5,
            &json!({"tweaks": {"a": true}, "_failed": {}})
        ));
        assert!(
            !s.adopt("settings.system", 5, &json!({"tweaks": {}})),
            "the same one"
        );
        assert_eq!(s.known(&TWEAKS), Known::Value(&json!({"a": true})));

        assert!(s.adopt(
            "settings.system",
            6,
            &json!({"_failed": {"tweaks": "no gtk file"}})
        ));
        assert_eq!(
            s.known(&TWEAKS),
            Known::Value(&json!({"a": true})),
            "it keeps what it had"
        );
        assert_eq!(s.failures(&[INPUT, TWEAKS]).len(), 1);
        s.adopt_reply(&json!({"snapshot": {"tweaks": {"a": false}}}), 7);
        assert!(s.failures(&[TWEAKS]).is_empty(), "read again");
    }

    #[test]
    fn a_sample_that_began_before_a_write_is_not_taken_for_the_keys_it_wrote() {
        let mut s = Settings::default();
        s.adopt_reply(&json!({"snapshot": {"tweaks": {"a": true}}}), 100);
        assert!(s.adopt(
            "settings.system",
            90,
            &json!({"tweaks": {"a": false}, "browser": "zen"})
        ));
        assert_eq!(
            s.known(&TWEAKS),
            Known::Value(&json!({"a": true})),
            "older than the write"
        );
        assert_eq!(
            s.value(&BROWSER),
            Some(&json!("zen")),
            "a key it did not write is taken"
        );
        assert!(s.adopt("settings.system", 100, &json!({"tweaks": {"a": false}})));
        assert_eq!(
            s.known(&TWEAKS),
            Known::Value(&json!({"a": false})),
            "began with the write"
        );
    }

    #[test]
    fn a_setting_that_read_nothing_back_shows_what_the_backend_set() {
        let mut s = Settings::default();
        s.adopt_reply(
            &json!({"domain": "browser", "value": "zen", "snapshot": {}}),
            1,
        );
        assert_eq!(s.value(&BROWSER), Some(&json!("zen")));
        s.adopt_reply(
            &json!({"domain": "tweaks.a", "value": true, "snapshot": {}}),
            2,
        );
        assert_eq!(s.value(&TWEAKS), None, "a field is not the key");
        s.adopt_reply(
            &json!({"domain": "browser", "value": "x", "snapshot": {"browser": "y"}}),
            3,
        );
        assert_eq!(
            s.value(&BROWSER),
            Some(&json!("y")),
            "what it read back wins"
        );
    }

    #[test]
    fn a_failed_topic_is_said_of_every_key_it_holds_until_one_is_read() {
        let mut s = Settings::default();
        s.topic_failed("settings.look", "timeout");
        assert_eq!(s.known(&INPUT), Known::Unknown("timeout"), "never read");
        s.adopt_reply(&json!({"snapshot": {"hyprInput": {"x": 1}}}), 1);
        assert_eq!(s.known(&INPUT), Known::Value(&json!({"x": 1})));
        assert!(s.failures(&[INPUT]).is_empty());

        assert!(s.adopt("settings.look", 1, &json!({"hyprInput": {"x": 2}})));
        s.topic_failed("settings.look", "hyprctl did not answer");
        assert_eq!(
            s.failures(&[INPUT]),
            [(&INPUT, "hyprctl did not answer")],
            "a key it carried keeps its value and says why"
        );
        assert!(s.adopt("settings.look", 2, &json!({"hyprInput": {"x": 3}})));
        assert!(s.failures(&[INPUT]).is_empty());
    }

    #[test]
    fn a_reply_says_what_it_could_not_read() {
        let mut s = Settings::default();
        s.adopt_reply(
            &json!({"snapshot": {}, "failed": {"tweaks": {"message": "denied"}}}),
            1,
        );
        assert_eq!(s.known(&TWEAKS), Known::Unknown("denied"));
    }

    #[test]
    fn the_topics_of_a_page_are_held_once_each_when_they_change() {
        assert_eq!(
            topics(&[INPUT, TWEAKS, INPUT]),
            [("settings.look", 0), ("settings.system", 0)]
        );
    }

    #[test]
    fn a_write_reads_back_its_head_and_a_hyprland_group_its_managed_flag() {
        assert_eq!(readback("browser"), ["browser"]);
        assert_eq!(readback("tweaks.middlePaste"), ["tweaks"]);
        assert_eq!(
            readback("hyprInput.sensitivity"),
            ["hyprInput", "hyprInputManaged"]
        );
        assert_eq!(
            write("browser", &json!("firefox")),
            json!({"op": "settings.set", "domain": "browser", "value": "firefox",
                "readback": ["browser"]})
        );
    }
}
