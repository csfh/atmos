//! The Services page's logic (`pages/ServicesPage.qml`, `services/Systemd.js`): the units the
//! snapshot lists, the search and the state filter, the summary, what each unit's row says and
//! offers, and the writes its actions send (`Omarchy.systemdAction`). `tests/catalog_units.rs`
//! runs `Systemd.js` and the page's code under Node against this.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::protocol;

pub const UNITS: Field = Field::Key("systemdUnits");

/// The units Atmos may start, stop, or enable, by scope (`allowlist()`).
pub const ALLOWED: &[(&str, &str)] = &[
    ("pipewire.service", "user"),
    ("pipewire-pulse.service", "user"),
    ("wireplumber.service", "user"),
    ("xdg-desktop-portal.service", "user"),
    ("xdg-desktop-portal-hyprland.service", "user"),
    ("bluetooth.service", "system"),
    ("NetworkManager.service", "system"),
    ("cups.service", "system"),
    ("docker.service", "system"),
    ("tailscaled.service", "system"),
    ("fstrim.timer", "system"),
];

/// The state filters (`filterChips()`).
pub const CHIPS: &[(&str, &str)] = &[
    ("all", "All"),
    ("running", "Running"),
    ("stopped", "Stopped"),
    ("enabled", "Enabled"),
    ("disabled", "Disabled"),
    ("failed", "Failed"),
];

fn text(v: &Value, name: &str) -> String {
    v.get(name)
        .filter(|x| crate::js::truthy(Some(x)))
        .map_or_else(String::new, crate::js::string)
}

/// `unitName`: a unit's name, "" for one that is not.
pub fn unit_name(raw: &str) -> String {
    let s = raw.trim();
    let ok = !s.is_empty()
        && s.encode_utf16().count() <= 128
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || ":_.@\\-".contains(c));
    if ok { s.to_owned() } else { String::new() }
}

/// `normalizeUnit`.
pub fn normalize(row: &Value) -> Option<Value> {
    if !row.is_object() {
        return None;
    }
    let unit = unit_name(&text(row, "unit"));
    if unit.is_empty() {
        return None;
    }
    let scope = if row.get("scope") == Some(&Value::from("user")) {
        "user"
    } else {
        "system"
    };
    let state = row
        .get("unitFileState")
        .filter(|v| crate::js::truthy(Some(v)))
        .or_else(|| row.get("enabled"))
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string);
    // `replace(/[\r\n]+/g, " ")`.
    let mut description = String::new();
    let mut run = false;
    for c in text(row, "description").chars() {
        if matches!(c, '\r' | '\n') {
            if !run {
                description.push(' ');
            }
            run = true;
        } else {
            description.push(c);
            run = false;
        }
    }
    let mut units = 0;
    let description: String = description
        .chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= 160
        })
        .collect();
    let allowed = ALLOWED.iter().any(|(u, s)| *u == unit && *s == scope);
    Some(json!({
        "unit": unit, "scope": scope, "load": text(row, "load"), "active": text(row, "active"),
        "sub": text(row, "sub"), "unitFileState": state.trim().to_lowercase(),
        "description": description, "allowed": allowed,
    }))
}

fn low(row: &Value, name: &str) -> String {
    text(row, name).to_lowercase()
}

pub fn is_failed(row: &Value) -> bool {
    low(row, "active") == "failed" || low(row, "sub") == "failed"
}

pub fn is_running(row: &Value) -> bool {
    !is_failed(row) && low(row, "active") == "active"
}

/// `runtimeState`: its mark and label.
fn runtime(row: &Value) -> (&'static str, &'static str) {
    match low(row, "active").as_str() {
        _ if is_failed(row) => ("×", "Failed"),
        "active" => ("●", "Running"),
        "activating" => ("●", "Starting"),
        "deactivating" => ("○", "Stopping"),
        _ => ("○", "Stopped"),
    }
}

/// `startupKey`.
fn startup_key(row: &Value) -> String {
    match low(row, "unitFileState").trim() {
        "enabled" | "enabled-runtime" => "enabled".into(),
        k @ ("disabled" | "static" | "masked" | "alias" | "generated" | "transient"
        | "indirect") => k.into(),
        _ => String::new(),
    }
}

/// `startupState(row).label`.
fn startup_label(row: &Value) -> String {
    let key = startup_key(row);
    let mut chars = key.chars();
    chars.next().map_or_else(String::new, |c| {
        format!("{}{}", c.to_ascii_uppercase(), chars.as_str())
    })
}

/// `statusLine`.
pub fn status_line(row: &Value) -> String {
    let (mark, label) = runtime(row);
    let start = startup_label(row);
    let mut parts = vec![format!("{mark} {label}")];
    if !start.is_empty() {
        parts.push(start);
    }
    parts.join("   ")
}

/// The hint of a unit's row.
pub fn hint(row: &Value) -> &'static str {
    match (
        crate::js::truthy(row.get("allowed")),
        text(row, "scope").as_str(),
    ) {
        (true, "user") => "systemctl --user",
        (true, _) => "systemctl",
        _ => "status and logs only",
    }
}

fn haystack(row: &Value) -> String {
    let allowed = if crate::js::truthy(row.get("allowed")) {
        "allowlist"
    } else {
        "logs status"
    };
    [
        text(row, "unit"),
        text(row, "description"),
        text(row, "scope"),
        runtime(row).1.to_owned(),
        startup_label(row),
        allowed.to_owned(),
    ]
    .join(" ")
}

fn matches(row: &Value, query: &str, filter: &str) -> bool {
    let q = query.trim().to_lowercase();
    let found = q.is_empty() || haystack(row).to_lowercase().contains(&q);
    let kept = match filter {
        "running" => is_running(row),
        "stopped" => !is_failed(row) && !is_running(row),
        "failed" => is_failed(row),
        "enabled" | "disabled" => startup_key(row) == filter,
        _ => true,
    };
    found && kept
}

/// `allRows`: each unit as `normalizeUnit` keeps it.
pub fn all(list: Option<&Value>) -> Vec<Value> {
    list.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(normalize)
        .collect()
}

/// `listUnits(rows, query, filter)`.
pub fn shown(rows: &[Value], query: &str, filter: &str) -> Vec<Value> {
    rows.iter()
        .filter_map(normalize)
        .filter(|r| matches(r, query, filter))
        .collect()
}

/// `summaryParts(summarize(rows))`: each part's id, its words, and whether it is the failures.
pub fn summary(rows: &[Value]) -> Vec<(&'static str, String, bool)> {
    let (mut running, mut stopped, mut enabled, mut failed) = (0, 0, 0, 0);
    for r in rows {
        if is_failed(r) {
            failed += 1;
        } else if is_running(r) {
            running += 1;
        } else {
            stopped += 1;
        }
        if startup_key(r) == "enabled" {
            enabled += 1;
        }
    }
    let failures = if failed > 0 {
        format!("{failed} failed")
    } else {
        "✓ No failures".into()
    };
    vec![
        ("running", format!("{running} running"), false),
        ("stopped", format!("{stopped} stopped"), false),
        ("enabled", format!("{enabled} enabled"), false),
        ("failed", failures, failed > 0),
    ]
}

/// `primaryAction`.
pub fn primary(row: &Value) -> Option<(&'static str, &'static str)> {
    if !crate::js::truthy(row.get("allowed")) {
        return None;
    }
    Some(if is_failed(row) {
        ("restart", "Restart")
    } else if is_running(row) {
        ("stop", "Stop")
    } else {
        ("start", "Start")
    })
}

/// `overflowActions`.
pub fn overflow(row: &Value) -> Vec<(&'static str, &'static str)> {
    let mut items = Vec::new();
    let allowed = crate::js::truthy(row.get("allowed"));
    let primary = primary(row).map(|(id, _)| id);
    if allowed && is_running(row) && primary != Some("restart") {
        items.push(("restart", "Restart"));
    }
    let start = startup_key(row);
    if allowed && !matches!(start.as_str(), "static" | "masked" | "") {
        items.push(if start == "enabled" {
            ("disable", "Disable at startup")
        } else {
            ("enable", "Enable at startup")
        });
    }
    items.extend([
        ("status", "View status"),
        ("logs", "View logs"),
        ("copy", "Copy unit name"),
    ]);
    items
}

/// What an empty list says.
pub fn empty_says(any: bool) -> &'static str {
    if any {
        "Nothing matches that search or filter."
    } else {
        "No units were reported for this session."
    }
}

/// `systemdAction(action, unit, scope)`: the write, and whether it runs as root.
pub fn action(action: &str, unit: &str, scope: &str) -> Option<(Value, bool)> {
    let known = matches!(action, "start" | "stop" | "restart" | "enable" | "disable");
    let ok = !unit.is_empty()
        && unit
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ":_.@\\-".contains(c));
    if !known || !ok {
        return None;
    }
    let user = scope == "user";
    let name = if user {
        "systemd.user"
    } else {
        "systemd.system"
    };
    let body = protocol::write(name, &json!({ "action": action, "unit": unit }));
    Some((body, !user))
}

/// The output dialog's title, for status or logs.
pub fn output_title(kind: &str, unit: &str) -> String {
    let what = if kind == "logs" { "Logs" } else { "Status" };
    format!("{what} · {unit}")
}

/// What the dialog shows of a reply: its text, or why there is none.
pub fn output_text(reply: Result<&Value, &str>) -> String {
    match reply {
        Ok(result) => {
            let text = text(result, "text");
            if text.trim().is_empty() {
                "No output.".into()
            } else {
                text
            }
        }
        Err("") => "Could not read that unit.".into(),
        Err(why) => why.to_owned(),
    }
}
