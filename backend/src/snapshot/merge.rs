//! Settings snapshot assembly.
//!
//! `capture` reads the live machine. The request document fills in from the
//! domain files `settings.set` writes only where the machine has no reading
//! (docs/adr/0016). A fixture, the plain backend, and `ratmos snapshot` read
//! those files alone. Both callers come through here so
//! the merge rules stay in one place.

use crate::error::Result;
use crate::platform::Backend;
use serde_json::{Map, Value};
use std::path::Path;

/// The `settings.snapshot` document.
///
/// Live Omarchy is what the producers read, with a domain file's value only
/// where a producer has no reading (`stored`). A null file value never erases
/// a captured field. Dotted domains fold into the
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

/// Live Omarchy reads are the topics the group names, each read by its own
/// producers, with the files `settings.set` writes where they have no reading. A key whose
/// tool failed is left out and named under `_failed`.
fn live(group: &str, keys: Option<&[String]>) -> Result<Map<String, Value>> {
    if !group.is_empty() && !super::GROUPS.contains(&group) {
        return Err(format!("snapshot group must be one of {}", super::GROUPS.join(", ")).into());
    }
    let names = super::topics::for_group(group);
    let docs: Vec<Map<String, Value>> = std::thread::scope(|scope| {
        let handles: Vec<_> = names
            .iter()
            .map(|name| scope.spawn(|| super::topics::collect(name)))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or_default())
            .collect()
    });
    let mut snapshot = Map::new();
    let mut failed = Map::new();
    for doc in docs {
        for (key, value) in doc {
            if key == super::topics::FAILED {
                if let Value::Object(map) = value {
                    failed.extend(map.into_iter().filter(|(k, _)| wanted(k, keys)));
                }
            } else if wanted(&key, keys) {
                snapshot.insert(key, value);
            }
        }
    }
    let group_name = if group.is_empty() { "all" } else { group };
    snapshot.insert("group".into(), Value::String(group_name.into()));
    if !failed.is_empty() {
        snapshot.insert(super::topics::FAILED.into(), Value::Object(failed));
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

/// How a domain's stored value stands against what its producer reads
/// (docs/adr/0016).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stored {
    /// The producer's reading is what is in effect and wins; the stored value
    /// is shown only where the producer has no reading (its tool failed, or it
    /// reads nothing for the key).
    Fallback,
    /// Live command status: the tool's answer is the value, and a file value
    /// never stands in for it, not even when the tool failed.
    Never,
    /// The producer reads nothing real for the key, so the stored value is it.
    Wins,
}

pub(super) fn stored(key: &str) -> Stored {
    match key {
        "nightlight" | "audioOutputMuted" | "audioInputMuted" | "barVisible"
        | "screensaverEnabled" | "stayAwake" | "touchpadEnabled" | "touchscreenEnabled"
        | "doNotDisturb" => Stored::Never,
        // The Atmos producer reports the constant "stable"; the channel file is the setting.
        "atmosChannel" => Stored::Wins,
        _ => Stored::Fallback,
    }
}

/// Whether the producer said nothing of `key`: the key, or for `head.tail`
/// the field of the object `head`, is absent or null.
pub(super) fn unread(doc: &Map<String, Value>, key: &str) -> bool {
    let found = match key.split_once('.') {
        Some((head, tail)) => doc
            .get(head)
            .and_then(Value::as_object)
            .and_then(|object| object.get(tail)),
        None => doc.get(key),
    };
    found.is_none_or(Value::is_null)
}

/// Lays a domain's stored value into a document the producers filled, by the
/// domain's rule. `read` is only called when the value would be used, so a
/// reading that wins costs no read of the stored file.
pub(super) fn lay_stored(
    doc: &mut Map<String, Value>,
    key: &str,
    read: impl FnOnce() -> Option<Value>,
) {
    let wanted = match stored(key) {
        Stored::Never => false,
        Stored::Wins => true,
        Stored::Fallback => unread(doc, key),
    };
    if !wanted {
        return;
    }
    if let Some(value) = read().filter(|value| !value.is_null()) {
        nest_domain(doc, key, value);
    }
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

    fn lay(doc: &mut Map<String, Value>, key: &str, value: Value) {
        lay_stored(doc, key, || Some(value));
    }

    #[test]
    fn a_null_file_value_does_not_erase_a_captured_field() {
        let mut doc = Map::new();
        doc.insert("theme".into(), json!("rose"));
        lay(&mut doc, "theme", Value::Null);
        assert_eq!(doc["theme"], json!("rose"));
    }

    #[test]
    fn live_command_status_is_not_covered_by_a_file() {
        let mut doc = Map::new();
        doc.insert("nightlight".into(), json!(true));
        doc.insert("barVisible".into(), json!(false));
        doc.insert("doNotDisturb".into(), json!(true));
        lay(&mut doc, "nightlight", json!(false));
        lay(&mut doc, "barVisible", json!(true));
        lay(&mut doc, "doNotDisturb", json!(false));
        lay(&mut doc, "stayAwake", json!(true));
        lay(&mut doc, "theme", json!("rose"));
        assert_eq!(doc["nightlight"], json!(true));
        assert_eq!(doc["barVisible"], json!(false));
        assert_eq!(doc["doNotDisturb"], json!(true));
        assert!(
            doc.get("stayAwake").is_none(),
            "not even when the tool said nothing"
        );
        assert_eq!(doc["theme"], json!("rose"), "a reading wins");
    }

    /// Every domain a producer reads, with a reading and a stored value that
    /// differ: the reading is shown. Under the old overlay each showed the
    /// stored value (docs/adr/0016).
    #[test]
    fn what_a_producer_reads_wins_over_what_was_stored() {
        let cases: &[(&str, Value, Value)] = &[
            ("tweaks.middlePaste", json!(false), json!(true)),
            ("tweaks.electronWayland", json!(true), json!(false)),
            ("tweaks.forceZeroScaling", json!(true), json!(false)),
            ("tweaks.swappiness", json!(false), json!(true)),
            ("audioOutputVolume", json!(40), json!(75)),
            ("audioInputVolume", json!(30), json!(90)),
            ("audioTuningOn", json!(false), json!(true)),
            ("suspendEnabled", json!(true), json!(false)),
            ("crashCapture", json!(false), json!(true)),
            ("bluetooth", json!(false), json!(true)),
            ("wifiRadio", json!(true), json!(false)),
            ("wifiBand", json!("5"), json!("2.4")),
            ("dns", json!("DHCP"), json!("Cloudflare")),
            ("textSize", json!(14), json!(12)),
            ("nightlightTemperature", json!(4000), json!(3500)),
            ("theme", json!("tokyo-night"), json!("rose")),
            ("background", json!("b.jpg"), json!("a.jpg")),
            ("font", json!("A"), json!("B")),
            ("plymouth", json!("x"), json!("y")),
            ("hyprLook.gapsIn", json!(5), json!(10)),
            ("hyprInput.accelProfile", json!("flat"), json!("")),
            ("hyprInput.naturalScroll", json!(true), json!(false)),
            ("browser", json!("firefox"), json!("chromium")),
            ("terminal", json!("ghostty"), json!("alacritty")),
            ("editor", json!("nvim"), json!("code")),
            ("agent", json!("a"), json!("b")),
            ("mimePdf", json!("a.desktop"), json!("b.desktop")),
            ("powerProfile", json!("balanced"), json!("performance")),
            ("powerProfileAc", json!("performance"), json!("balanced")),
            (
                "powerProfileBattery",
                json!("power-saver"),
                json!("balanced"),
            ),
            ("timezone", json!("Europe/Copenhagen"), json!("UTC")),
            ("ntp", json!(true), json!(false)),
            ("omarchyChannel", json!("stable"), json!("edge")),
            (
                "avatarPath",
                json!("/home/u/.face"),
                json!("/home/u/Pictures/me.png"),
            ),
            ("weatherLocation", json!("Copenhagen"), json!("Oslo")),
            ("fullName", json!("A B"), json!("C D")),
            ("plugins", json!(["a"]), json!(["b"])),
            ("chargeLimit", json!(80), json!(100)),
        ];
        let mut covered = Vec::new();
        for (key, reading, stored) in cases {
            assert_eq!(super::stored(key), Stored::Fallback, "{key}");
            let mut doc = Map::new();
            nest_domain(&mut doc, key, reading.clone());
            lay(&mut doc, key, stored.clone());
            let shown = match key.split_once('.') {
                Some((head, tail)) => doc[head][tail].clone(),
                None => doc[*key].clone(),
            };
            if &shown != reading {
                covered.push(*key);
            }
        }
        assert!(
            covered.is_empty(),
            "the stored value covered the reading of {covered:?}"
        );
    }

    #[test]
    fn a_stored_value_stands_in_only_where_the_producer_has_no_reading() {
        let mut doc = Map::new();
        // The tool failed: the key is not there, and the stored value is shown.
        lay(&mut doc, "audioOutputVolume", json!(75));
        assert_eq!(doc["audioOutputVolume"], json!(75));
        // The object is there without the field: the field is filled in.
        doc.insert("hyprLook".into(), json!({"gapsIn": 5}));
        lay(&mut doc, "hyprLook.cursorSize", json!(24));
        assert_eq!(doc["hyprLook"], json!({"gapsIn": 5, "cursorSize": 24}));
        // A null reading is no reading.
        doc.insert("textSize".into(), Value::Null);
        lay(&mut doc, "textSize", json!(13));
        assert_eq!(doc["textSize"], json!(13));
    }

    #[test]
    fn the_atmos_channel_is_what_is_stored_since_the_producer_reads_none() {
        let mut doc = Map::new();
        doc.insert("atmosChannel".into(), json!("stable"));
        lay(&mut doc, "atmosChannel", json!("edge"));
        assert_eq!(doc["atmosChannel"], json!("edge"));
    }

    #[test]
    fn a_reading_that_wins_reads_no_stored_file() {
        let mut doc = Map::new();
        doc.insert("theme".into(), json!("rose"));
        lay_stored(&mut doc, "theme", || panic!("the stored file was read"));
        lay_stored(&mut doc, "nightlight", || {
            panic!("the stored file was read")
        });
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
        let root = std::env::temp_dir().join(format!("ratmos-merge-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
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
