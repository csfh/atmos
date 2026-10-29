//! `planImport` (`Settings.js`): one computation serves the review and the apply, so the plan
//! read is the plan that runs. A plan is the JavaScript's own shape: `changes` (`key`,
//! `section`, `label`, `tier`, `from`, `to`, `consequence`), `unchanged` keys, `warnings`
//! (`key`, `message`), `blocked` (`key`, `reason`) and a `summary`.

use serde_json::{Value, json};

use super::canon::{self, export_list};
use super::{
    SCHEMA, count_label, display_value, item, number_text, read_value, shown_value, strict_eq,
    stringify,
};

/// What the page knows besides the snapshot: this machine's hardware, and whether a live scan
/// found the workspace gesture written by hand.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub hardware: String,
    pub workspace_gesture_unmanaged: bool,
}

/// `sameValue(a, b)`.
pub fn same_value(a: Option<&Value>, b: &Value) -> bool {
    match (a, b) {
        (Some(Value::Number(x)), Value::Number(y)) => {
            (x.as_f64().unwrap_or(f64::NAN) - y.as_f64().unwrap_or(f64::NAN)).abs() < 1e-9
        }
        (Some(Value::Array(x)), Value::Array(y)) => {
            stringify(&Value::Array(export_list(x))) == stringify(&Value::Array(export_list(y)))
        }
        _ => strict_eq(a, Some(b)),
    }
}

/// `typeMatches(type, value)`.
fn type_matches(ty: &str, v: &Value) -> bool {
    match ty {
        "list" => v.is_array(),
        "boolean" => v.is_boolean(),
        "integer" => v.as_f64().is_some_and(|n| n.is_finite() && n.floor() == n),
        "number" => v.as_f64().is_some_and(f64::is_finite),
        "string" => v.is_string(),
        _ => false,
    }
}

/// `describe(value)`.
fn describe(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::Array(_) => "a list",
        _ => "text",
    }
}

/// `allowedContains(list, value)`: the value, or an option whose `value` it is.
fn allowed_contains(list: &[Value], v: &Value) -> bool {
    list.iter().any(|o| {
        strict_eq(Some(o), Some(v)) || (o.is_object() && strict_eq(o.get("value"), Some(v)))
    })
}

/// `isClockTime(value)`: a 24-hour time such as 07:00.
fn clock_time(v: &Value) -> bool {
    let Some((h, m)) = v.as_str().and_then(|s| s.split_once(':')) else {
        return false;
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let hour = digits(h)
        && match h.len() {
            1 => true,
            2 => h.as_bytes()[0] <= b'1' || (h.as_bytes()[0] == b'2' && h.as_bytes()[1] <= b'3'),
            _ => false,
        };
    hour && m.len() == 2 && digits(m) && m.as_bytes()[0] <= b'5'
}

/// `unmanagedCount(value)`: the rows the snapshot says live outside the Atmos block.
fn unmanaged(v: Option<&Value>) -> usize {
    v.and_then(Value::as_array).map_or(0, |rows| {
        rows.iter()
            .filter(|r| r.is_object() && r.get("managed") == Some(&Value::Bool(false)))
            .count()
    })
}

/// `managedRows(value)`: the rows of the Atmos block.
fn managed(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .filter(|r| !(r.is_object() && r.get("managed") == Some(&Value::Bool(false))))
        .cloned()
        .collect()
}

/// `planSummary(...)`.
fn summary(changes: usize, unchanged: usize, warnings: usize, blocked: usize) -> String {
    let mut parts = vec![count_label(changes, "change", "changes")];
    if unchanged > 0 {
        parts.push(format!("{unchanged} already match"));
    }
    if warnings > 0 {
        parts.push(count_label(warnings, "warning", "warnings"));
    }
    if blocked > 0 {
        parts.push(format!("{blocked} blocked"));
    }
    parts.join(", ")
}

/// `result(...)`.
fn result(
    changes: Vec<Value>,
    unchanged: Vec<String>,
    warnings: Vec<Value>,
    blocked: Vec<Value>,
) -> Value {
    let summary = summary(
        changes.len(),
        unchanged.len(),
        warnings.len(),
        blocked.len(),
    );
    json!({ "changes": changes, "unchanged": unchanged, "warnings": warnings, "blocked": blocked, "summary": summary })
}

fn warn(key: &str, message: impl Into<String>) -> Value {
    json!({ "key": key, "message": message.into() })
}

fn block(key: &str, reason: impl Into<String>) -> Value {
    json!({ "key": key, "reason": reason.into() })
}

/// `planImport(doc, snapshot, null, options)`: what applying the file would change here.
pub fn plan(doc: &Value, snap: &Value, opts: &Options) -> Value {
    let (mut changes, mut unchanged, mut warnings, mut blocked) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for e in doc["errors"].as_array().into_iter().flatten() {
        blocked.push(json!({ "key": "", "reason": e }));
    }
    let meta = &doc["meta"];
    let schema = match meta.get("schema") {
        None => f64::NAN,
        some => crate::js::number(some),
    };
    if !schema.is_finite() || schema <= 0.0 {
        warnings.push(warn(
            "",
            format!(
                "No schema version in this file. Reading it as version {}.",
                number_text(SCHEMA)
            ),
        ));
    } else if schema > SCHEMA {
        blocked.push(block(
            "",
            format!(
                "This file is schema {} and this Atmos reads {}. Update Atmos first.",
                number_text(schema),
                number_text(SCHEMA)
            ),
        ));
        return result(changes, unchanged, warnings, blocked);
    }
    let file_hardware = super::text_or_empty(meta.get("hardware"));
    let different =
        !file_hardware.is_empty() && !opts.hardware.is_empty() && file_hardware != opts.hardware;
    if different {
        warnings.push(warn(
            "",
            "This file came from different hardware. Device-specific settings are held back.",
        ));
    }
    let gesture = opts.workspace_gesture_unmanaged
        || snap.get("hyprWorkspaceGestureUnmanaged") == Some(&Value::Bool(true));

    let sections = doc["sections"].as_object().cloned().unwrap_or_default();
    for (name, values) in &sections {
        let Some(values) = values.as_object() else {
            continue;
        };
        for (key, to) in values {
            let Some(item) = item(key) else {
                warnings.push(warn(key, "Atmos does not know this setting. Skipped."));
                continue;
            };
            if item.section != name {
                warnings.push(warn(
                    key,
                    format!(
                        "Found under {name} but it belongs to {}. Skipped.",
                        item.section
                    ),
                ));
                continue;
            }
            if !item.importable {
                blocked.push(block(
                    key,
                    format!(
                        "{} is reported only. Atmos never imports security settings.",
                        item.label
                    ),
                ));
                continue;
            }
            let from = read_value(snap, key);
            if key == "hyprInput.workspaceGesture" && gesture {
                warnings.push(warn(key, "Workspace gesture is written by hand in ~/.config/hypr/input.lua outside the Atmos block. Atmos leaves that line alone, so this setting is skipped."));
                continue;
            }
            if same_value(from, to) {
                unchanged.push(key.clone());
                continue;
            }
            if !type_matches(item.ty, to) {
                blocked.push(block(
                    key,
                    format!(
                        "{} expects {} but the file has {}.",
                        item.label,
                        item.ty,
                        describe(to)
                    ),
                ));
                continue;
            }
            let mut to = to.clone();
            if item.kind == "list" {
                match canon::list(item, &to) {
                    Err(e) => {
                        blocked.push(block(key, e));
                        continue;
                    }
                    Ok(v) => to = Value::Array(v),
                }
                let from_cmp = match from {
                    Some(f @ Value::Array(_)) => canon::list(item, f).map(Value::Array).ok(),
                    _ => None,
                };
                if same_value(from_cmp.as_ref().or(from), &to) {
                    unchanged.push(key.clone());
                    continue;
                }
            }
            if !item.options.is_empty() {
                match read_value(snap, item.options).and_then(Value::as_array) {
                    Some(allowed) if !allowed.is_empty() => {
                        if !allowed_contains(allowed, &to) {
                            blocked.push(block(
                                key,
                                format!("{} is not available on this machine.", display_value(&to)),
                            ));
                            continue;
                        }
                    }
                    _ => warnings.push(warn(
                        key,
                        format!(
                            "This machine did not report which {} values it has, so {} could not be checked. It may fail when applied.",
                            item.label.to_lowercase(),
                            display_value(&to)
                        ),
                    )),
                }
            }
            if let Some(choices) = item.choices.filter(|c| !c.is_empty()) {
                let choices: Vec<Value> = choices.iter().map(|c| Value::from(*c)).collect();
                if !allowed_contains(&choices, &to) {
                    let label = if item.label.is_empty() {
                        "value"
                    } else {
                        item.label
                    };
                    blocked.push(block(
                        key,
                        format!(
                            "{} is not a valid {}.",
                            shown_value(Some(&to)),
                            label.to_lowercase()
                        ),
                    ));
                    continue;
                }
            }
            if item.format == "time" && !clock_time(&to) {
                blocked.push(block(
                    key,
                    format!("{} expects a 24-hour time such as 07:00.", item.label),
                ));
                continue;
            }
            if item.host_bound && different {
                warnings.push(warn(
                    key,
                    format!(
                        "{} is tied to this machine's hardware. Held back.",
                        item.label
                    ),
                ));
                continue;
            }
            let hand = unmanaged(from);
            if hand > 0 {
                warnings.push(warn(
                    key,
                    format!(
                        "{} of your current {} are written by hand in the config file. Atmos writes its own block and leaves those lines alone, so they stay in the file with the imported ones taking effect.",
                        count_label(hand, "row", "rows"),
                        item.label.to_lowercase()
                    ),
                ));
            }
            let plan_from = match from {
                Some(Value::Array(rows)) if item.kind == "list" => Value::Array(managed(rows)),
                Some(v) => v.clone(),
                None => Value::Null,
            };
            changes.push(json!({
                "key": key, "section": item.section, "label": item.label, "tier": item.tier,
                "from": plan_from, "to": to, "consequence": item.consequence,
            }));
        }
    }
    changes.sort_by(|a, b| {
        let k = |v: &Value| {
            v["key"]
                .as_str()
                .unwrap_or_default()
                .encode_utf16()
                .collect::<Vec<_>>()
        };
        k(a).cmp(&k(b))
    });
    unchanged.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    result(changes, unchanged, warnings, blocked)
}
