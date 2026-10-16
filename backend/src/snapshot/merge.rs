//! Settings snapshot assembly.
//!
//! `capture` reads the live machine. The request document merges that with the
//! domain files `settings.set` writes. A fixture, the plain backend, and
//! `ratmos snapshot` read those files alone. Both callers come through here so
//! the merge rules stay in one place.

use crate::error::Result;
use crate::platform::Backend;
use serde_json::{Map, Value};
use std::path::Path;

/// The `settings.snapshot` document.
///
/// Live Omarchy merges `capture` with the domain files. A null file value does
/// not erase a captured field, and a file value does not cover a live command
/// status. Anything else nests the domain files. Dotted domains fold into the
/// object the Quickshell snapshot already uses. An unset value is absent.
pub fn settings(
    backend: Backend,
    root: Option<&Path>,
    group: &str,
    keys: Option<&[String]>,
) -> Result<Map<String, Value>> {
    let mut doc = if backend.is_live(root) {
        live(group, keys)?
    } else {
        nested(backend, root, keys)?
    };
    doc.retain(|_, value| !value.is_null());
    Ok(doc)
}

/// Every domain, flat, nulls included. `ratmos snapshot` embeds this.
pub fn settings_snapshot(backend: Backend, root: Option<&Path>) -> Result<Map<String, Value>> {
    let mut snapshot = Map::new();
    for spec in crate::domain::specs() {
        snapshot.insert(
            spec.key.to_string(),
            crate::read_domain(backend, root, spec.key)?,
        );
    }
    Ok(snapshot)
}

/// Live Omarchy reads merge `capture` with the files `settings.set` writes.
/// `--root` never reaches this path, so fixture launches stay file-only.
fn live(group: &str, keys: Option<&[String]>) -> Result<Map<String, Value>> {
    let mut snapshot = Map::new();
    if let Ok(Value::Object(map)) = super::capture(group) {
        snapshot = map;
    }
    let group_name = if group.is_empty() { "all" } else { group };
    snapshot.insert("group".into(), Value::String(group_name.into()));
    for spec in crate::domain::specs() {
        if !wanted(spec.key, keys) {
            continue;
        }
        // One unreadable system file must not drop the rest of the page.
        if let Ok(value) = crate::read_domain(Backend::Omarchy, None, spec.key) {
            overlay_domain(&mut snapshot, spec.key, value);
        }
    }
    Ok(snapshot)
}

fn nested(
    backend: Backend,
    root: Option<&Path>,
    keys: Option<&[String]>,
) -> Result<Map<String, Value>> {
    let mut doc = Map::new();
    for spec in crate::domain::specs() {
        if !wanted(spec.key, keys) {
            continue;
        }
        let value = crate::read_domain(backend, root, spec.key)?;
        if !value.is_null() {
            nest_domain(&mut doc, spec.key, value);
        }
    }
    Ok(doc)
}

fn overlay_domain(doc: &mut Map<String, Value>, key: &str, value: Value) {
    if value.is_null() {
        return;
    }
    // These switches are live command status. A file value must not cover them.
    if matches!(
        key,
        "nightlight"
            | "audioOutputMuted"
            | "audioInputMuted"
            | "barVisible"
            | "screensaverEnabled"
            | "stayAwake"
            | "touchpadEnabled"
            | "touchscreenEnabled"
            | "doNotDisturb"
    ) {
        return;
    }
    nest_domain(doc, key, value);
}

/// A dotted domain such as `hyprLook.gaps` lands inside the object `hyprLook`.
fn nest_domain(doc: &mut Map<String, Value>, key: &str, value: Value) {
    if let Some((head, tail)) = key.split_once('.') {
        let entry = doc
            .entry(head.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        if let Some(map) = entry.as_object_mut() {
            map.insert(tail.to_string(), value);
        }
        return;
    }
    doc.insert(key.to_string(), value);
}

/// Whether a domain is in the requested key list. The list names top-level
/// keys, so `hyprLook.gaps` is wanted when `hyprLook` is.
fn wanted(key: &str, keys: Option<&[String]>) -> bool {
    let Some(keys) = keys else { return true };
    let head = key.split_once('.').map_or(key, |(head, _)| head);
    keys.iter().any(|name| name == head)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_dotted_domain_folds_into_its_object() {
        let mut doc = Map::new();
        nest_domain(&mut doc, "hyprLook.gapsIn", json!(8));
        nest_domain(&mut doc, "hyprLook.gapsOut", json!(16));
        nest_domain(&mut doc, "theme", json!("rose"));
        assert_eq!(doc["hyprLook"]["gapsIn"], json!(8));
        assert_eq!(doc["hyprLook"]["gapsOut"], json!(16));
        assert_eq!(doc["theme"], json!("rose"));
    }

    #[test]
    fn a_dotted_domain_replaces_a_scalar_already_stored_there() {
        let mut doc = Map::new();
        doc.insert("hyprLook".into(), json!("nope"));
        nest_domain(&mut doc, "hyprLook.gapsIn", json!(4));
        assert_eq!(doc["hyprLook"]["gapsIn"], json!(4));
    }

    #[test]
    fn a_null_file_value_does_not_erase_a_captured_field() {
        let mut doc = Map::new();
        doc.insert("theme".into(), json!("rose"));
        overlay_domain(&mut doc, "theme", Value::Null);
        assert_eq!(doc["theme"], json!("rose"));
    }

    #[test]
    fn live_command_status_is_not_covered_by_a_file() {
        let mut doc = Map::new();
        doc.insert("nightlight".into(), json!(true));
        doc.insert("barVisible".into(), json!(false));
        doc.insert("doNotDisturb".into(), json!(true));
        overlay_domain(&mut doc, "nightlight", json!(false));
        overlay_domain(&mut doc, "barVisible", json!(true));
        overlay_domain(&mut doc, "doNotDisturb", json!(false));
        overlay_domain(&mut doc, "stayAwake", json!(true));
        overlay_domain(&mut doc, "theme", json!("rose"));
        assert_eq!(doc["nightlight"], json!(true));
        assert_eq!(doc["barVisible"], json!(false));
        assert_eq!(doc["doNotDisturb"], json!(true));
        assert!(doc.get("stayAwake").is_none());
        assert_eq!(doc["theme"], json!("rose"));
    }

    #[test]
    fn wanted_matches_the_top_level_name_of_a_dotted_key() {
        let keys = names(&["hyprLook", "theme"]);
        assert!(wanted("hyprLook.gapsIn", Some(&keys)));
        assert!(wanted("theme", Some(&keys)));
        assert!(!wanted("hyprInput.sensitivity", Some(&keys)));
        assert!(!wanted("hyprLooks", Some(&keys)));
        assert!(!wanted("font", Some(&keys)));
    }

    #[test]
    fn wanted_with_no_list_wants_everything_and_with_an_empty_list_nothing() {
        assert!(wanted("anything.at.all", None));
        assert!(!wanted("theme", Some(&[])));
    }

    #[test]
    fn a_fixture_snapshot_drops_nulls_nests_dots_and_filters_keys() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("ratmos-merge-{nanos}-{}", std::process::id()));
        let plain = root.join(".config/plain");
        std::fs::create_dir_all(&plain).unwrap();
        std::fs::write(
            plain.join("theme.json"),
            "{\"theme\":\"rose\",\"font\":null}\n",
        )
        .unwrap();
        std::fs::write(plain.join("look.json"), "{\"hyprLook.gapsIn\":8}\n").unwrap();

        let filtered = settings(
            Backend::Plain,
            Some(&root),
            "look",
            Some(&names(&["theme"])),
        )
        .unwrap();
        assert_eq!(filtered.get("theme"), Some(&json!("rose")));
        assert!(filtered.get("hyprLook").is_none());
        assert!(filtered.get("font").is_none());
        assert!(filtered.get("group").is_none());

        let all = settings(Backend::Plain, Some(&root), "", None).unwrap();
        assert_eq!(all.get("theme"), Some(&json!("rose")));
        assert_eq!(all["hyprLook"]["gapsIn"], json!(8));
        assert!(all.get("font").is_none());
        assert!(all.get("group").is_none());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
