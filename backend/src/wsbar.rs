//! The named workspace bar. Omarchy's stock workspaces widget paints numbers, so
//! Atmos installs a clone that paints the names and the shown count, points the
//! bar at it, and tells the running shell.

use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text;
use crate::runner::Run;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const WIDGET: &str = include_str!("../../packaging/workspace-bar/Workspaces.qml");
const STOCK: &str = "omarchy.workspaces";
const SIDES: [&str; 3] = ["left", "center", "right"];

pub fn is_key(key: &str) -> bool {
    matches!(key, "workspaceBarNames" | "workspaceBarCount")
}

pub fn clone_id() -> String {
    let user = ["USER", "LOGNAME"]
        .iter()
        .find_map(|var| std::env::var(var).ok().filter(|v| !v.is_empty()))
        .or_else(|| {
            Run::new("id")
                .arg("-un")
                .output()
                .ok()
                .map(|out| out.stdout_text().trim().to_string())
        })
        .unwrap_or_else(|| "user".into());
    format!("{user}.workspaces")
}

fn entry_id(entry: &Value) -> String {
    match entry {
        Value::Object(map) => match map.get("id") {
            Some(Value::String(id)) => id.clone(),
            Some(Value::Null) | None => String::new(),
            Some(other) => other.to_string(),
        },
        Value::String(id) => id.clone(),
        other => other.to_string(),
    }
}

fn entries(doc: &Value) -> impl Iterator<Item = &Value> {
    SIDES
        .iter()
        .filter_map(|side| doc.pointer(&format!("/bar/layout/{side}")))
        .filter_map(Value::as_array)
        .flatten()
}

/// What the clone entry says now: shown count and whether names are on.
fn current(doc: &Value, id: &str) -> (Value, bool) {
    let hit = entries(doc).find(|w| entry_id(w) == id);
    let names = hit.is_none_or(|w| w.get("showNames") != Some(&Value::Bool(false)));
    let count = hit
        .and_then(|w| w.get("count"))
        .filter(|c| !c.is_null())
        .cloned()
        .unwrap_or_else(|| json!(5));
    (count, names)
}

/// The field as the bar entry holds it; `Null` when there is no workspaces
/// widget or it does not set the field.
pub fn get(doc: &Value, field: &str) -> Value {
    let id = clone_id();
    entries(doc)
        .find(|w| {
            let found = entry_id(w);
            found == id || found == STOCK
        })
        .and_then(|w| w.get(field))
        .cloned()
        .unwrap_or(Value::Null)
}

/// Point every stock workspaces entry at the clone, then write the count and
/// names on the clone. The field not being set keeps the clone's current value.
/// A bar with no workspaces widget gets the clone on its left.
pub fn set(doc: &mut Value, field: &str, value: &Value) {
    let id = clone_id();
    let (mut count_value, mut names) = current(doc, &id);
    if field == "count" {
        count_value = value.clone();
    } else if let Some(on) = value.as_bool() {
        names = on;
    }
    for side in SIDES {
        let Some(items) = doc
            .pointer_mut(&format!("/bar/layout/{side}"))
            .and_then(Value::as_array_mut)
        else {
            continue;
        };
        for item in items.iter_mut().filter(|i| entry_id(i) == STOCK) {
            *item = json!({ "id": id });
        }
        let Some(item) = items.iter_mut().find(|i| entry_id(i) == id) else {
            continue;
        };
        if !item.is_object() {
            *item = json!({ "id": id });
        }
        if let Some(map) = item.as_object_mut() {
            map.insert("count".into(), count_value.clone());
            map.insert("showNames".into(), json!(names));
        }
    }
    if entries(doc).any(|w| entry_id(w) == id) {
        return;
    }
    if let Some(left) = left_mut(doc) {
        left.push(json!({ "id": id, "count": count_value, "showNames": names }));
    }
}

fn left_mut(doc: &mut Value) -> Option<&mut Vec<Value>> {
    fn child<'a>(value: &'a mut Value, key: &str, empty: Value) -> Option<&'a mut Value> {
        Some(value.as_object_mut()?.entry(key).or_insert(empty))
    }
    let bar = child(doc, "bar", json!({}))?;
    let layout = child(bar, "layout", json!({}))?;
    child(layout, "left", json!([]))?.as_array_mut()
}

fn base(root: Option<&Path>) -> PathBuf {
    root.map(Path::to_path_buf).unwrap_or_else(|| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"))
    })
}

fn install(dest: &Path, text: &str) -> Result<()> {
    if std::fs::read_to_string(dest).is_ok_and(|have| have == text) {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic_write_text(dest, text)
}

fn omarchy_path() -> String {
    std::env::var("OMARCHY_PATH")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/usr/share/omarchy".into())
}

/// The stock manifest under the clone's id and name, marked as cloned from it.
fn clone_manifest(stock: &str, id: &str) -> Result<String> {
    let mut manifest: Value = serde_json::from_str(stock)?;
    let Some(map) = manifest.as_object_mut() else {
        return Err(Error::bad_request(
            "the stock workspaces manifest is not an object",
        ));
    };
    map.insert("id".into(), json!(id));
    map.insert("name".into(), json!("My Workspaces"));
    if let Some(widget) = map.get_mut("barWidget").and_then(Value::as_object_mut) {
        widget.insert("displayName".into(), json!("My Workspaces"));
    }
    let mut omarchy = match map.remove("omarchy") {
        Some(Value::Object(o)) => o,
        _ => serde_json::Map::new(),
    };
    omarchy.insert("clonedFrom".into(), json!(STOCK));
    map.insert("omarchy".into(), Value::Object(omarchy));
    Ok(format!("{}\n", serde_json::to_string_pretty(&manifest)?))
}

/// Put the clone under `~/.config/omarchy/plugins`. It is rewritten only when
/// the widget or manifest differs, so a count write also picks up widget fixes.
pub fn install_clone(root: Option<&Path>) -> Result<()> {
    let id = clone_id();
    let dir = base(root).join(".config/omarchy/plugins").join(&id);
    let manifest = dir.join("manifest.json");
    let stock =
        Path::new(&omarchy_path()).join("shell/plugins/bar/widgets/Workspaces.manifest.json");
    if let Ok(text) = std::fs::read_to_string(&stock) {
        install(&manifest, &clone_manifest(&text, &id)?)?;
    } else if !manifest.is_file() && root.is_none() {
        // A fixture root has no Omarchy to copy the manifest from.
        return Err(Error::command(format!("cannot install {id}")));
    }
    install(&dir.join("Workspaces.qml"), WIDGET)
}

/// Before the shell.json write: the bar must not name a clone that is not there.
pub fn before_write(root: Option<&Path>, key: &str) -> Result<()> {
    if is_key(key) {
        install_clone(root)?;
    }
    Ok(())
}

/// After the write: a live desktop may run /usr/share/omarchy/shell instead of
/// `$OMARCHY_PATH/shell`, so ping every candidate and update the one that answers.
/// Fixture roots never touch the session bar.
pub fn after_write(root: Option<&Path>, key: &str) {
    if !is_key(key) || root.is_some() {
        return;
    }
    let id = clone_id();
    let Ok(text) = std::fs::read_to_string(base(None).join(".config/omarchy/shell.json")) else {
        return;
    };
    let Ok(doc) = serde_json::from_str::<Value>(&text) else {
        return;
    };
    let (count, names) = current(&doc, &id);
    let candidates = [
        "/usr/share/omarchy/shell".to_string(),
        format!("{}/shell", omarchy_path()),
    ];
    let live = candidates.iter().find(|shell| {
        Path::new(shell).join("shell.qml").is_file() && ipc(shell, &["ping"]).is_ok_and(|ok| ok)
    });
    let Some(shell) = live else {
        return;
    };
    for (field, value) in [("count", count), ("showNames", json!(names))] {
        let _ = ipc(
            shell,
            &["setBarWidget", &id, field, &value.to_string(), "{}"],
        );
    }
}

fn ipc(shell: &str, call: &[&str]) -> Result<bool> {
    let mut args = vec!["ipc", "-n", "-p", shell, "call", "--", "shell"];
    args.extend_from_slice(call);
    Ok(Run::new("qs").args(args).output()?.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar() -> Value {
        json!({"bar": {"layout": {
            "left": [{"id": "omarchy.menu"}, {"id": "omarchy.workspaces"}],
            "center": [], "right": []}}})
    }

    #[test]
    fn the_first_write_points_the_bar_at_the_clone() {
        let id = clone_id();
        let mut doc = bar();
        set(&mut doc, "showNames", &json!(true));
        assert_eq!(doc["bar"]["layout"]["left"][1]["id"], json!(id));
        assert_eq!(doc["bar"]["layout"]["left"][1]["showNames"], json!(true));
        assert_eq!(doc["bar"]["layout"]["left"][1]["count"], json!(5));
    }

    #[test]
    fn a_count_write_keeps_the_names_choice_and_other_fields() {
        let id = clone_id();
        let mut doc = bar();
        set(&mut doc, "showNames", &json!(false));
        doc["bar"]["layout"]["left"][1]["note"] = json!("keep");
        set(&mut doc, "count", &json!(7));
        let item = &doc["bar"]["layout"]["left"][1];
        assert_eq!(item["id"], json!(id));
        assert_eq!(item["count"], json!(7));
        assert_eq!(item["showNames"], json!(false));
        assert_eq!(item["note"], json!("keep"));
        assert_eq!(get(&doc, "count"), json!(7));
        assert_eq!(get(&doc, "showNames"), json!(false));
    }

    #[test]
    fn a_bar_without_the_widget_gets_the_clone_on_its_left() {
        let mut doc = json!({"bar": {"layout": {"left": [], "center": [], "right": []}}});
        assert_eq!(get(&doc, "count"), Value::Null);
        set(&mut doc, "count", &json!(3));
        assert_eq!(doc["bar"]["layout"]["left"][0]["id"], json!(clone_id()));
        assert_eq!(get(&doc, "count"), json!(3));
    }

    #[test]
    fn the_clone_manifest_takes_the_stock_one_over() {
        let stock = r#"{"id":"omarchy.workspaces","name":"Workspaces","barWidget":{"displayName":"Workspaces"},"omarchy":{"x":1}}"#;
        let out: Value =
            serde_json::from_str(&clone_manifest(stock, "tester.workspaces").unwrap()).unwrap();
        assert_eq!(out["id"], json!("tester.workspaces"));
        assert_eq!(out["name"], json!("My Workspaces"));
        assert_eq!(out["barWidget"]["displayName"], json!("My Workspaces"));
        assert_eq!(out["omarchy"]["clonedFrom"], json!("omarchy.workspaces"));
        assert_eq!(out["omarchy"]["x"], json!(1));
    }
}
