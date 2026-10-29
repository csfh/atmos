//! The Window rules page's logic (`pages/windows/RulesPage.qml`, `services/WindowRules.js`,
//! `RowText.js`): the rules it lists, what each says, the checks Add makes, and the
//! `windowRules` write `Omarchy.addWindowRule` and `removeWindowRule` send.
//! `tests/catalog_custom.rs` runs the page's code, and `tests/catalog_setters.rs` the setters,
//! under Node against this.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::js;

pub const RULES: Field = Field::Key("windowRules");
pub const FOCUSED: Field = Field::Key("focusedClass");

/// The placements Add offers.
pub const PLACEMENTS: &[(&str, &str)] = &[("", "Leave"), ("float", "Float"), ("tile", "Tile")];

/// The size a field allows, and its step.
pub const SIZE: (f64, f64, f64) = (0.0, 4000.0, 10.0);

pub const NO_MATCH: &str = "Enter a window class, or a class regex.";
pub const BAD_WORKSPACE: &str = "Workspace can use letters, digits, colon, underscore, or hyphen.";
pub const BAD_SIZE: &str = "Size needs both width and height, 100 to 4000.";
pub const NOTHING: &str = "Pick float or tile, turn on center, set a size, or set a workspace.";

fn text(v: Option<&Value>) -> String {
    v.filter(|v| js::truthy(Some(v)))
        .map_or_else(String::new, js::string)
}

/// `ruleRows`: the rules with a match, less this window's own when it is not Atmos's.
pub fn rows(rules: Option<&Value>) -> Vec<Value> {
    rules
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|r| js::truthy(Some(r)) && js::truthy(r.get("match")))
        .filter(|r| r.get("managed") == Some(&Value::Bool(true)) || r["match"] != "dev.csfh.atmos")
        .cloned()
        .collect()
}

/// `RuleJs.describe(row)`.
pub fn describe(row: &Value) -> String {
    let mut bits = Vec::new();
    match row.get("placement").and_then(Value::as_str) {
        Some("float") => bits.push("float".to_owned()),
        Some("tile") => bits.push("tile".to_owned()),
        _ => {}
    }
    if js::truthy(row.get("center")) {
        bits.push("center".into());
    }
    if js::truthy(row.get("width")) && js::truthy(row.get("height")) {
        bits.push(format!(
            "{}\u{d7}{}",
            text(row.get("width")),
            text(row.get("height"))
        ));
    }
    for (name, said) in [("workspace", "workspace "), ("title", "title ")] {
        let t = text(row.get(name));
        if !t.is_empty() {
            bits.push(format!("{said}{t}"));
        }
    }
    if js::truthy(row.get("pin")) {
        bits.push("pin".into());
    }
    if js::truthy(row.get("fullscreen")) {
        bits.push("fullscreen".into());
    }
    let opacity = text(row.get("opacity"));
    if !opacity.is_empty() {
        bits.push(format!("opacity {opacity}"));
    }
    bits.join(" \u{b7} ")
}

/// A rule's description.
pub fn description(row: &Value) -> String {
    let said = describe(row);
    let said = if said.is_empty() {
        "A window rule.".to_owned()
    } else {
        said
    };
    if js::truthy(row.get("managed")) {
        said
    } else {
        format!("{said} Remove stays off — this line is outside the Atmos block.")
    }
}

/// A rule's label.
pub fn label(row: &Value) -> String {
    let m = text(row.get("match"));
    if m.is_empty() { "class".into() } else { m }
}

/// `sanitizeMatch`.
pub fn sanitize_match(raw: &str) -> String {
    if raw.contains(['\n', '\r']) {
        return String::new();
    }
    let t = raw.trim();
    if t.is_empty() || t.encode_utf16().count() > 128 || t.contains("]]") {
        return String::new();
    }
    t.to_owned()
}

/// `sanitizeWorkspace`.
pub fn sanitize_workspace(raw: &str) -> String {
    let t = raw.trim();
    let ok = !t.is_empty()
        && t.len() <= 32
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '_' | '-'));
    if ok { t.to_owned() } else { String::new() }
}

/// What Add's form holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub matches: String,
    pub placement: String,
    pub center: bool,
    pub width: f64,
    pub height: f64,
    pub workspace: String,
}

/// `submitAdd`: the rule to add, as `addWindowRule`'s arguments, or what is wrong.
pub fn submit(d: &Draft) -> Result<(String, String, bool, f64, f64, String), &'static str> {
    let m = sanitize_match(&d.matches);
    if m.is_empty() {
        return Err(NO_MATCH);
    }
    let ws = sanitize_workspace(&d.workspace);
    if !d.workspace.trim().is_empty() && ws.is_empty() {
        return Err(BAD_WORKSPACE);
    }
    let (w, h) = (d.width, d.height);
    if (w > 0.0 || h > 0.0) && !(w >= 100.0 && h >= 100.0) {
        return Err(BAD_SIZE);
    }
    if d.placement.is_empty() && !d.center && !(w != 0.0 && h != 0.0) && ws.is_empty() {
        return Err(NOTHING);
    }
    Ok((m, d.placement.clone(), d.center, w, h, ws))
}

/// `managedWindowRules`.
fn managed(rules: Option<&Value>) -> Vec<Value> {
    let num = |v: Option<&Value>| {
        let n = js::round(js::number(v));
        js::json_number(if n == 0.0 || n.is_nan() { 0.0 } else { n })
    };
    rules
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|r| r.get("managed") == Some(&Value::Bool(true)) && js::truthy(r.get("match")))
        .map(|r| {
            json!({
                "match": js::string(&r["match"]),
                "placement": text(r.get("placement")),
                "center": r.get("center") == Some(&Value::Bool(true)),
                "width": num(r.get("width")),
                "height": num(r.get("height")),
                "workspace": text(r.get("workspace")),
                "title": text(r.get("title")),
                "pin": r.get("pin") == Some(&Value::Bool(true)),
                "fullscreen": r.get("fullscreen") == Some(&Value::Bool(true)),
                "opacity": text(r.get("opacity")),
            })
        })
        .collect()
}

/// `addWindowRule(match, placement, center, width, height, workspace)`.
pub fn add(
    rules: Option<&Value>,
    m: &str,
    placement: &str,
    center: bool,
    width: &Value,
    height: &Value,
    workspace: &str,
) -> Option<Value> {
    let m = m.trim();
    let placement = if matches!(placement, "float" | "tile") {
        placement
    } else {
        ""
    };
    let workspace = workspace.trim();
    let round = |v: &Value| {
        let n = js::round(js::number(Some(v)));
        if n == 0.0 || n.is_nan() { 0.0 } else { n }
    };
    let (mut w, mut h) = (round(width), round(height));
    if m.is_empty() || m.encode_utf16().count() > 128 || m.contains('\n') || m.contains("]]") {
        return None;
    }
    if !(w >= 100.0 && h >= 100.0) {
        w = 0.0;
        h = 0.0;
    }
    if placement.is_empty() && !center && w == 0.0 && workspace.is_empty() {
        return None;
    }
    let mut next: Vec<Value> = managed(rules)
        .into_iter()
        .filter(|r| r["match"] != m)
        .collect();
    next.push(json!({
        "match": m, "title": "", "placement": placement, "center": center,
        "width": js::json_number(w), "height": js::json_number(h), "workspace": workspace,
        "pin": false, "fullscreen": false, "opacity": "",
    }));
    Some(crate::lists::write("windowRules", rules, next))
}

/// `removeWindowRule(match)`.
pub fn remove(rules: Option<&Value>, m: &str) -> Option<Value> {
    let cur = managed(rules);
    let next: Vec<Value> = cur.iter().filter(|r| r["match"] != m).cloned().collect();
    (next.len() != cur.len()).then(|| crate::lists::write("windowRules", rules, next))
}

/// The Remove confirm's message.
pub fn remove_message(m: &str) -> String {
    format!("Remove the Atmos rule for {m}?")
}
