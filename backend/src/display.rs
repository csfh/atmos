//! Live stats and the inventories the Quickshell pages already show.

use crate::error::{Error, Result};
use crate::platform::Backend;
use std::path::Path;

use serde_json::{Map, Value};

pub const KINDS: &[&str] = &[
    "live",
    "hardware",
    "disks",
    "services",
    "software",
    "diagnostics",
];

pub fn is_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

pub fn display_rel(platform: Backend, kind: &str) -> String {
    format!(".local/state/{}/display/{kind}.json", platform.id())
}

pub fn collector(platform: Backend, kind: &str) -> &'static str {
    let (omarchy, plain) = match kind {
        "live" => ("omarchy-sampler", "plain-sampler"),
        "hardware" => ("omarchy-hardware", "plain-hardware"),
        "disks" => ("omarchy-disks", "plain-disks"),
        "services" => ("omarchy-services", "plain-services"),
        "software" => ("omarchy-software", "plain-software"),
        "diagnostics" => ("omarchy-diagnostics", "plain-diagnostics"),
        _ => ("omarchy", "plain"),
    };
    if platform == Backend::Plain {
        plain
    } else {
        omarchy
    }
}

/// Read one display document.
///
/// A fixture file under the config root wins, so tests stay deterministic.
/// Without a root, the Omarchy backend collects the inventory natively. The
/// plain backend has no host inventories.
pub fn load(root: Option<&Path>, platform: Backend, kind: &str) -> Result<Value> {
    if !is_kind(kind) {
        return Err(format!("unknown display kind {kind}").into());
    }
    let named = collector(platform, kind);
    // A fixture root is for tests and never reads the live sample cache.
    if let Some(root) = root {
        let path = root.join(display_rel(platform, kind));
        if path.is_file() {
            let text = std::fs::read_to_string(&path)?;
            let value: Value = serde_json::from_str(&text)?;
            return Ok(stamp(value, platform, named));
        }
        return Ok(stamp(Value::Object(Map::new()), platform, named));
    }
    if platform != Backend::Omarchy {
        return Ok(stamp(Value::Object(Map::new()), platform, named));
    }
    // The sampling daemon keeps these warm; without one the kind is collected here.
    if let Some(doc) = crate::sampler::client::fetch(kind) {
        return Ok(doc);
    }
    collect(platform, kind)
}

/// Collect one inventory now, bypassing the sample cache. The daemon's collector.
pub fn collect(platform: Backend, kind: &str) -> Result<Value> {
    let value = crate::inventory::collect(kind)
        .ok_or_else(|| Error::from(format!("no collector for {kind}")))?;
    Ok(stamp(value, platform, collector(platform, kind)))
}

fn stamp(value: Value, platform: Backend, collector: &str) -> Value {
    let mut map = match value {
        Value::Object(map) => map,
        Value::Array(items) => {
            let mut map = Map::new();
            map.insert("items".into(), Value::Array(items));
            map
        }
        other => {
            let mut map = Map::new();
            map.insert("value".into(), other);
            map
        }
    };
    map.insert("platform".into(), Value::String(platform.id().into()));
    map.insert("collector".into(), Value::String(collector.into()));
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_listed_kinds_are_kinds() {
        for kind in KINDS {
            assert!(is_kind(kind), "{kind}");
        }
        assert!(!is_kind(""));
        assert!(!is_kind("Live"));
        assert!(!is_kind("live "));
        assert!(!is_kind("../live"));
    }

    #[test]
    fn display_documents_live_under_the_platform_state_dir() {
        assert_eq!(
            display_rel(Backend::Omarchy, "live"),
            ".local/state/omarchy/display/live.json"
        );
        assert_eq!(
            display_rel(Backend::Plain, "disks"),
            ".local/state/plain/display/disks.json"
        );
    }

    #[test]
    fn every_kind_has_a_collector_on_every_backend() {
        for kind in KINDS {
            let omarchy = collector(Backend::Omarchy, kind);
            assert!(
                omarchy.starts_with("omarchy-"),
                "{kind} should have a native collector, got {omarchy}"
            );
            assert!(
                collector(Backend::Plain, kind).starts_with("plain-"),
                "{kind}"
            );
        }
    }

    #[test]
    fn an_unknown_kind_falls_back_instead_of_panicking() {
        assert_eq!(collector(Backend::Omarchy, "nope"), "omarchy");
        assert_eq!(collector(Backend::Plain, "nope"), "plain");
    }
}
