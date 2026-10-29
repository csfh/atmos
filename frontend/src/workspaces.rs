//! The Workspaces page's logic (`pages/WorkspacesPage.qml`, `services/Workspaces.js`): the
//! numbered and scratch workspaces it lists, the state its writes start from (`clampState`), the
//! choices its selects offer, and the writes: `workspaces` as `Omarchy.writeWorkspaces` sends it,
//! the bar count, and each switch. `tests/catalog_custom.rs` runs the page's code, and
//! `tests/catalog_setters.rs` the setters, under Node against this.
//!
//! The QML page's Wrap switching and Super + mouse wheel call `writeWorkspaces` with the switch,
//! but the switch only reaches the script's input there: the `settings.set` carries the items,
//! and `atmosd` keeps the file's own switches when it writes them. This window sends the switch's
//! own setting (`workspaceWrapSwitch`, `workspaceWheelSwitch`), which `atmosd` writes.

use std::cmp::Ordering;

use serde_json::{Map, Value, json};

use crate::catalog::control::Field;
use crate::{js, settings};

pub const ITEMS: Field = Field::Key("workspaces");
pub const WRAP: Field = Field::Key("workspaceWrapSwitch");
pub const WHEEL: Field = Field::Key("workspaceWheelSwitch");
pub const COUNT: Field = Field::Key("workspaceBarCount");
pub const MONITORS: Field = Field::Key("monitors");

/// The bar count's range (`PrefsSlider`'s `from`, `to` and `stepSize`).
pub const SHOWN: (f64, f64, f64) = (1.0, 10.0, 1.0);

pub const BAD_SCRATCH: &str = "Use letters, digits, underscore, or hyphen.";
pub const SCRATCH_TAKEN: &str = "That scratch workspace already exists.";
pub const SCRATCH_SAYS: &str = "Name it special:notes or similar.";

fn rows(list: Option<&Value>) -> impl Iterator<Item = &Value> {
    list.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|r| js::truthy(Some(r)))
}

/// `numbered`: the workspaces that are not scratch ones.
pub fn numbered(list: Option<&Value>) -> Vec<Value> {
    rows(list)
        .filter(|r| !js::truthy(r.get("special")))
        .cloned()
        .collect()
}

/// `specials`: the scratch workspaces.
pub fn specials(list: Option<&Value>) -> Vec<Value> {
    rows(list)
        .filter(|r| js::truthy(r.get("special")))
        .cloned()
        .collect()
}

/// `String(raw || "")`, trimmed.
fn trimmed(raw: Option<&Value>) -> String {
    raw.filter(|v| js::truthy(Some(v)))
        .map_or_else(String::new, js::string)
        .trim()
        .to_owned()
}

/// `sanitizeId`: `special:` and up to 24 letters, digits, `_` or `-`, or 1 to 10.
pub fn sanitize_id(raw: Option<&Value>) -> String {
    let text = trimmed(raw);
    if let Some(name) = text.strip_prefix("special:") {
        let ok = (1..=24).contains(&name.len())
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'));
        return if ok { text } else { String::new() };
    }
    let digits = text.chars().all(|c| c.is_ascii_digit());
    let ok = digits
        && !text.starts_with('0')
        && (1..=2).contains(&text.len())
        && text.parse::<u8>().is_ok_and(|n| n <= 10);
    if ok { text } else { String::new() }
}

/// `sanitizeName`: one line of at most 32 units, of letters, digits, space, `_`, `.` or `-`.
pub fn sanitize_name(raw: Option<&Value>) -> String {
    let text = raw
        .filter(|v| js::truthy(Some(v)))
        .map_or_else(String::new, js::string);
    if text.contains(['\n', '\r']) {
        return String::new();
    }
    let text = text.trim();
    let ok = text.encode_utf16().count() <= 32
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '.' | '-'));
    if ok { text.to_owned() } else { String::new() }
}

/// `sanitizeMonitor`: letters, digits, `.`, `_` or `-`.
pub fn sanitize_monitor(raw: Option<&Value>) -> String {
    let text = trimmed(raw);
    let ok = text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if ok { text } else { String::new() }
}

/// `sanitizeCommand`: one line of at most 256 units.
fn sanitize_command(raw: Option<&Value>) -> String {
    let text = trimmed(raw);
    if text.encode_utf16().count() > 256 || text.contains(['\n', '\r']) {
        String::new()
    } else {
        text
    }
}

/// The value of the first of `names` that is truthy (`row.name || row.default_name`).
fn either<'a>(row: &'a Value, names: [&str; 2]) -> Option<&'a Value> {
    names
        .iter()
        .filter_map(|n| row.get(*n))
        .find(|v| js::truthy(Some(v)))
}

/// `normalizeItem`.
fn normalize(row: &Value) -> Option<Value> {
    if !row.is_object() && !row.is_array() {
        return None;
    }
    let id = sanitize_id(either(row, ["id", "workspace"]));
    if id.is_empty() {
        return None;
    }
    let special = id.starts_with("special:") || row.get("special") == Some(&Value::Bool(true));
    Some(json!({
        "id": id,
        "name": sanitize_name(either(row, ["name", "default_name"])),
        "persistent": special || row.get("persistent") != Some(&Value::Bool(false)),
        "monitor": sanitize_monitor(row.get("monitor")),
        "isDefault": row.get("isDefault") == Some(&Value::Bool(true))
            || row.get("default") == Some(&Value::Bool(true)),
        "special": special,
        "onCreatedEmpty": sanitize_command(either(row, ["onCreatedEmpty", "on_created_empty"])),
    }))
}

fn default_item(id: &str) -> Value {
    json!({
        "id": id, "name": "", "persistent": true, "monitor": "", "isDefault": false,
        "special": false, "onCreatedEmpty": "",
    })
}

fn id(row: &Value) -> &str {
    row["id"].as_str().unwrap_or_default()
}

/// `clampState(...).items`: each workspace once, 1 to 10 always, scratch ones last, and one
/// default at most.
pub fn state(list: Option<&Value>) -> Vec<Value> {
    let mut items: Vec<Value> = Vec::new();
    for row in list.and_then(Value::as_array).into_iter().flatten() {
        if let Some(row) = normalize(row)
            && !items.iter().any(|r| id(r) == id(&row))
        {
            items.push(row);
        }
    }
    for n in 1..=10 {
        let n = n.to_string();
        if !items.iter().any(|r| id(r) == n) {
            items.push(default_item(&n));
        }
    }
    let special = |r: &Value| r["special"] == Value::Bool(true);
    items.sort_by(|a, b| match (special(a), special(b)) {
        (false, true) => Ordering::Less,
        (true, false) => Ordering::Greater,
        _ => match (id(a).parse::<f64>(), id(b).parse::<f64>()) {
            (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
            _ if id(a) < id(b) => Ordering::Less,
            _ => Ordering::Greater,
        },
    });
    let mut seen = false;
    for row in &mut items {
        if special(row) || seen {
            row["isDefault"] = Value::Bool(false);
        } else if row["isDefault"] == Value::Bool(true) {
            seen = true;
        }
    }
    items
}

/// `String(value)`, which is "undefined" for none.
fn or_undefined(v: Option<&Value>) -> String {
    v.map_or_else(|| "undefined".to_owned(), js::string)
}

/// `defaultId`: the default workspace, "" for Hyprland's.
pub fn default_id(items: &[Value]) -> String {
    items
        .iter()
        .find(|r| js::truthy(r.get("isDefault")) && !js::truthy(r.get("special")))
        .map_or_else(String::new, |r| or_undefined(r.get("id")))
}

/// `withDefault`: the workspaces with `want` the default.
pub fn with_default(items: &[Value], want: &str) -> Vec<Value> {
    items
        .iter()
        .filter(|r| js::truthy(Some(r)))
        .map(|r| {
            let mut copy = r.clone();
            let on = !js::truthy(r.get("special")) && r.get("id") == Some(&Value::from(want));
            copy["isDefault"] = Value::Bool(on && !want.is_empty());
            copy
        })
        .collect()
}

/// `defaultOptions`: Hyprland's default, then each numbered workspace by number and name.
pub fn default_options(items: &[Value]) -> Vec<(String, String)> {
    let mut out = vec![(String::new(), "Hyprland default".to_owned())];
    for r in items.iter().filter(|r| js::truthy(Some(r))) {
        if js::truthy(r.get("special")) {
            continue;
        }
        let id = or_undefined(r.get("id"));
        let label = match r.get("name").filter(|n| js::truthy(Some(n))) {
            Some(name) => format!("{id} · {}", js::string(name)),
            None => id.clone(),
        };
        out.push((id, label));
    }
    out
}

/// `monitorOptions`: Any, each monitor once, and the one pinned when it is not among them.
pub fn monitor_options(monitors: Option<&Value>, current: Option<&Value>) -> Vec<(String, String)> {
    let mut out = vec![(String::new(), "Any".to_owned())];
    for m in monitors.and_then(Value::as_array).into_iter().flatten() {
        let name = either(m, ["name", "id"]).map_or_else(String::new, js::string);
        if !name.is_empty() && !out.iter().any(|(v, _)| *v == name) {
            out.push((name.clone(), name));
        }
    }
    let extra = current
        .filter(|c| js::truthy(Some(c)))
        .map_or_else(String::new, js::string);
    if !extra.is_empty() && !out.iter().any(|(v, _)| *v == extra) {
        out.push((extra.clone(), extra));
    }
    out
}

/// A row's label and description.
pub fn label(row: &Value, fallback: &str) -> String {
    row.get("id")
        .filter(|v| js::truthy(Some(v)))
        .map_or_else(|| fallback.to_owned(), js::string)
}

pub fn description(row: &Value) -> String {
    match row.get("monitor").filter(|v| js::truthy(Some(v))) {
        Some(m) => format!("Pinned to {}.", js::string(m)),
        None => "Number in the bar unless you name it.".to_owned(),
    }
}

/// `clampShown`: a whole number from 1 to 10, 5 for one that is not.
pub fn clamp_shown(raw: f64) -> f64 {
    let n = js::round(raw);
    if !n.is_finite() || n < 1.0 {
        5.0
    } else {
        n.min(10.0)
    }
}

/// The bar count's write, when the count it comes to is not the one shown.
pub fn count_write(shown: Option<&Value>, raw: f64) -> Option<Value> {
    let next = clamp_shown(raw);
    let same = shown.and_then(Value::as_f64) == Some(next);
    (!same).then(|| settings::write("workspaceBarCount", &js::json_number(next)))
}

/// A switch's write, as `Omarchy.dispatchSetting(key, on)` makes it.
pub fn switch_write(key: &str, on: bool) -> Value {
    settings::write(key, &Value::Bool(on))
}

/// `writeWorkspaces(items, ...)`: the items, after the lines the user wrote.
pub fn write(list: Option<&Value>, items: Vec<Value>) -> Value {
    crate::lists::write("workspaces", list, items)
}

/// `patchItem(id, patch)`: the workspaces, with `key` set to `value` on `id`'s.
pub fn patch(list: Option<&Value>, id: &Value, key: &str, value: Value) -> Vec<Value> {
    let mut items = state(list);
    for row in &mut items {
        if row.get("id") == Some(id) {
            row[key] = value.clone();
        }
    }
    items
}

/// `addSpecial`: the workspaces with a new scratch one, or what is wrong with its name.
pub fn add_special(list: Option<&Value>, draft: &str) -> Result<Vec<Value>, &'static str> {
    let raw = Value::from(format!("special:{}", draft.trim()));
    let id = sanitize_id(Some(&raw));
    if !id.starts_with("special:") {
        return Err(BAD_SCRATCH);
    }
    let mut items = state(list);
    if items.iter().any(|r| r["id"] == id.as_str()) {
        return Err(SCRATCH_TAKEN);
    }
    let mut row = Map::new();
    for (k, v) in [
        ("id", Value::from(id)),
        ("name", Value::from("")),
        ("persistent", Value::Bool(true)),
        ("special", Value::Bool(true)),
        ("monitor", Value::from("")),
        ("isDefault", Value::Bool(false)),
        ("onCreatedEmpty", Value::from("")),
    ] {
        row.insert(k.into(), v);
    }
    items.push(Value::Object(row));
    Ok(items)
}

/// Remove: the workspaces less `id`'s.
pub fn remove(list: Option<&Value>, id: &Value) -> Vec<Value> {
    state(list)
        .into_iter()
        .filter(|r| r.get("id") != Some(id))
        .collect()
}
