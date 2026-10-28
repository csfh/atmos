//! The five inventories as the window knows them (`DisplayFeed.qml`): `hardware`, `disks`,
//! `services`, `software` and `diagnostics`, each pushed by the daemon when it is collected and
//! when a page first holds it. A failed collection leaves what is shown and says why
//! (docs/adr/0006).

use std::collections::HashMap;

use serde_json::Value;

use crate::settings::Known;

pub const KINDS: &[&str] = &["hardware", "disks", "services", "software", "diagnostics"];

pub fn is_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

#[derive(Debug, Clone, Default)]
pub struct Inventories {
    docs: HashMap<String, Value>,
    failed: HashMap<String, String>,
    last: HashMap<String, u64>,
}

impl Inventories {
    /// A collection of `kind`, without the stamp the daemon puts on it (`unstamp`). False when it
    /// is not after the last of that kind, or is not an inventory.
    pub fn adopt(&mut self, kind: &str, sampled_at: u64, doc: &Value) -> bool {
        if !is_kind(kind) || self.last.get(kind).is_some_and(|last| sampled_at <= *last) {
            return false;
        }
        self.last.insert(kind.to_owned(), sampled_at);
        let mut doc = doc.clone();
        if let Some(map) = doc.as_object_mut() {
            map.remove("platform");
            map.remove("collector");
        }
        self.docs.insert(kind.to_owned(), doc);
        self.failed.remove(kind);
        true
    }

    /// A collection that failed: what is shown stays, and says why.
    pub fn failed(&mut self, kind: &str, why: &str) {
        if is_kind(kind) {
            self.failed.insert(kind.to_owned(), why.to_owned());
        }
    }

    pub fn known(&self, kind: &str) -> Known<'_> {
        match (self.docs.get(kind), self.failed.get(kind)) {
            (Some(doc), _) => Known::Value(doc),
            (None, Some(why)) => Known::Unknown(why),
            (None, None) => Known::Waiting,
        }
    }

    /// Why the last collection of `kind` failed, while it has not been collected since.
    pub fn failure(&self, kind: &str) -> Option<&str> {
        self.failed.get(kind).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_collection_is_taken_once_without_its_stamp_and_a_failure_keeps_it() {
        let mut inv = Inventories::default();
        assert_eq!(inv.known("hardware"), Known::Waiting);
        assert!(!inv.adopt("live", 1, &json!({})), "not an inventory");
        let doc = json!({"gpus": [], "platform": "omarchy", "collector": "x"});
        assert!(inv.adopt("hardware", 2, &doc));
        assert!(!inv.adopt("hardware", 2, &doc), "the same one");
        assert_eq!(inv.known("hardware"), Known::Value(&json!({"gpus": []})));
        inv.failed("hardware", "lspci failed");
        assert_eq!(inv.known("hardware"), Known::Value(&json!({"gpus": []})));
        assert_eq!(inv.failure("hardware"), Some("lspci failed"));
        inv.failed("disks", "lsblk failed");
        assert_eq!(inv.known("disks"), Known::Unknown("lsblk failed"));
    }
}
