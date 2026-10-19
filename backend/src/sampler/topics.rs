//! What the daemon knows about each topic besides its value: how many times it
//! has been collected, when, and whether the latest collection failed. A failure
//! marks the topic stale and keeps the last good sample; it never replaces it.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use serde_json::{json, Value};

use super::cache::Sample;
use crate::display::KINDS;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Topic {
    pub seq: u64,
    pub sampled_at: Option<u64>,
    pub took_ms: Option<u64>,
    pub stale: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct Topics(Mutex<HashMap<String, Topic>>);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

impl Topics {
    /// A sample from before this daemon started, found in the cache.
    pub fn seed(&self, kind: &str, sample: &Sample) {
        lock(&self.0).insert(
            kind.to_string(),
            Topic {
                seq: 0,
                sampled_at: Some(sample.at_ms),
                took_ms: Some(sample.took_ms),
                stale: false,
                error: None,
            },
        );
    }

    pub fn success(&self, kind: &str, sample: &Sample) {
        let mut topics = lock(&self.0);
        let topic = topics.entry(kind.to_string()).or_default();
        topic.seq += 1;
        topic.sampled_at = Some(sample.at_ms);
        topic.took_ms = Some(sample.took_ms);
        topic.stale = false;
        topic.error = None;
    }

    /// The last good sample stays; the topic says it is out of date and why.
    pub fn failure(&self, kind: &str, error: &str) {
        let mut topics = lock(&self.0);
        let topic = topics.entry(kind.to_string()).or_default();
        topic.stale = true;
        topic.error = Some(error.to_string());
    }

    #[cfg(test)]
    pub fn get(&self, kind: &str) -> Topic {
        lock(&self.0).get(kind).cloned().unwrap_or_default()
    }

    pub fn status(&self) -> Value {
        let topics = lock(&self.0);
        let mut out = serde_json::Map::new();
        for kind in KINDS {
            let topic = topics.get(*kind).cloned().unwrap_or_default();
            out.insert(
                (*kind).to_string(),
                json!({
                    "seq": topic.seq,
                    "sampledAt": topic.sampled_at,
                    "tookMs": topic.took_ms,
                    "stale": topic.stale,
                    "error": topic.error,
                }),
            );
        }
        Value::Object(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(at_ms: u64) -> Sample {
        Sample {
            at_ms,
            took_ms: 7,
            value: json!({}),
        }
    }

    #[test]
    fn a_new_topic_has_no_history() {
        assert_eq!(Topics::default().get("services"), Topic::default());
    }

    #[test]
    fn each_success_advances_the_sequence() {
        let topics = Topics::default();
        topics.success("services", &sample(10));
        topics.success("services", &sample(20));
        let topic = topics.get("services");
        assert_eq!(
            (topic.seq, topic.sampled_at, topic.stale),
            (2, Some(20), false)
        );
    }

    #[test]
    fn a_failure_marks_the_topic_stale_and_keeps_when_it_last_succeeded() {
        let topics = Topics::default();
        topics.success("disks", &sample(10));
        topics.failure("disks", "df hung");
        let topic = topics.get("disks");
        assert!(topic.stale);
        assert_eq!(topic.error.as_deref(), Some("df hung"));
        assert_eq!(
            (topic.seq, topic.sampled_at),
            (1, Some(10)),
            "the good sample is untouched"
        );
        topics.success("disks", &sample(30));
        let healed = topics.get("disks");
        assert!(
            !healed.stale && healed.error.is_none(),
            "the next success clears it"
        );
    }

    #[test]
    fn seeding_from_the_cache_does_not_count_as_a_collection() {
        let topics = Topics::default();
        topics.seed("hardware", &sample(5));
        let topic = topics.get("hardware");
        assert_eq!((topic.seq, topic.sampled_at), (0, Some(5)));
    }

    #[test]
    fn status_lists_every_kind() {
        let status = Topics::default().status();
        for kind in KINDS {
            assert_eq!(status[*kind]["seq"], 0, "{kind}");
            assert_eq!(status[*kind]["stale"], false, "{kind}");
        }
    }
}
