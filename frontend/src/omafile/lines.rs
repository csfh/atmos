//! What the Export page says of a file and a plan (`Settings.js`): the file's own summary, the
//! changes with each list's rows added, changed and removed, the warnings, what is blocked, the
//! forecast and the two confirms, the plan as `atmosd task settings-apply` reads it, and what
//! the apply printed.

use std::collections::HashSet;

use serde_json::{Map, Value, json};

use super::canon::{normalize_row, number_word};
use super::{SCHEMA, count_label, item, shown_value, stringify, text_or_empty, trim};
use crate::js;

fn changes(plan: &Value) -> &[Value] {
    plan["changes"].as_array().map_or(&[], Vec::as_slice)
}

/// `passwordCount(plan)`: how many changes raise privileges.
pub fn password_count(plan: &Value) -> usize {
    changes(plan)
        .iter()
        .filter(|c| {
            c["key"]
                .as_str()
                .and_then(item)
                .is_some_and(|i| i.needs_root)
        })
        .count()
}

/// `commandImportLines(plan)`: the commands a file would install as shortcuts or startup
/// programs, each once.
pub fn command_lines(plan: &Value) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for change in changes(plan) {
        if !change["key"]
            .as_str()
            .and_then(item)
            .is_some_and(|i| i.extra_confirm)
        {
            continue;
        }
        for row in change["to"].as_array().into_iter().flatten() {
            let cmd = match row {
                Value::String(s) => s.clone(),
                Value::Object(_) | Value::Array(_) => text_or_empty(row.get("command")),
                _ => String::new(),
            };
            let cmd = trim(&cmd).to_owned();
            if !cmd.is_empty() && seen.insert(cmd.clone()) {
                out.push(cmd);
            }
        }
    }
    out
}

/// `hasCommandImport(plan)`.
pub fn has_commands(plan: &Value) -> bool {
    !command_lines(plan).is_empty()
}

/// `applyForecast(plan)`: how much, how long, and whether it will interrupt you.
pub fn forecast(plan: &Value) -> String {
    let n = changes(plan).len();
    if n == 0 {
        return String::new();
    }
    let mut lines = vec![format!(
        "{}, a few seconds.",
        count_label(n, "change", "changes")
    )];
    let asks = password_count(plan);
    if asks > 0 {
        lines.push(format!(
            "{} root. Atmos asks for sudo mode once and the rest run without asking again.",
            count_label(asks, "change needs", "changes need")
        ));
    }
    if has_commands(plan) {
        lines.push("This file replaces keybindings or startup programs. Read every command above before you apply.".into());
    }
    lines.join("\n")
}

/// `applyConfirmMessage(plan)`.
pub fn apply_confirm(plan: &Value) -> String {
    match forecast(plan) {
        f if f.is_empty() => f,
        f => format!(
            "{f}\n\nAtmos writes a way back first, and Put it back restores every one of them."
        ),
    }
}

/// `commandConfirmMessage(plan)`.
pub fn command_confirm(plan: &Value) -> String {
    let mut lines =
        vec!["These commands will be installed as shortcuts or startup programs:".to_owned()];
    lines.extend(command_lines(plan).into_iter().map(|c| format!("• {c}")));
    lines.push(String::new());
    lines.push("A shared file can put a download on Super+Return or run it at login. Only apply if you trust every line.".into());
    lines.join("\n")
}

/// `parseApplyResult(text)`: the last line of the apply's output that is a JSON object.
pub fn apply_result(text: &str) -> Value {
    text.split('\n')
        .rev()
        .map(trim)
        .filter(|l| l.starts_with('{'))
        .find_map(|l| {
            serde_json::from_str::<Value>(l)
                .ok()
                .filter(Value::is_object)
        })
        .unwrap_or_else(|| json!({ "backup": "", "results": [] }))
}

/// `appliedCountFromResult(result)`.
pub fn applied_count(result: &Value) -> usize {
    result["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| r.get("status") == Some(&Value::from("applied")))
        .count()
}

/// `backupDirFromResult(result)`.
pub fn backup_dir(result: &Value) -> String {
    text_or_empty(result.get("backup"))
}

/// The time an Omafile says it was exported, as a person reads it.
fn exported_words(when: &str) -> String {
    let when = when.replacen('T', " ", 1);
    // `.replace(/\..*$/, "")`: from the first point whose rest is one line.
    let cut = when
        .char_indices()
        .filter(|(_, c)| *c == '.')
        .map(|(i, _)| i)
        .find(|i| !when[*i..].contains(['\n', '\r', '\u{2028}', '\u{2029}']));
    let when = cut.map_or(when.as_str(), |i| &when[..i]);
    when.replacen('Z', " UTC", 1)
}

/// `fileSummary(doc, plan)`: what the file says about itself.
pub fn file_summary(doc: &Value, plan: Option<&Value>) -> String {
    let meta = &doc["meta"];
    let mut lines = Vec::new();
    let host = text_or_empty(meta.get("hostname"));
    let when = text_or_empty(meta.get("exported"));
    if !host.is_empty() || !when.is_empty() {
        let from = if host.is_empty() {
            "an unnamed machine"
        } else {
            &host
        };
        let at = if when.is_empty() {
            String::new()
        } else {
            format!(", exported {}", exported_words(&when))
        };
        lines.push(format!("From {from}{at}"));
    }
    let hardware = text_or_empty(meta.get("hardware"));
    if !hardware.is_empty() {
        lines.push(format!("Hardware: {hardware}"));
    }
    let mut names: Vec<&String> = doc["sections"]
        .as_object()
        .map(|m| m.keys().collect())
        .unwrap_or_default();
    names.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    if !names.is_empty() {
        lines.push(format!(
            "{}: {}",
            count_label(names.len(), "section", "sections"),
            names
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(plan) = plan {
        let settled = plan["unchanged"].as_array().map_or(0, Vec::len);
        if settled > 0 {
            lines.push(format!("{settled} of these already match this machine."));
        }
    }
    lines.join("\n")
}

fn field(row: &Value, k: &str) -> String {
    text_or_empty(row.get(k))
}

/// `listRowId(row, key)`: what makes a row the same row on both sides.
fn row_id(row: &Value, key: &str) -> String {
    match row {
        Value::Null => String::new(),
        Value::Object(_) | Value::Array(_) => match key {
            "bindings" => format!(
                "k:{}\0u:{}",
                field(row, "keys"),
                if row.get("unbind") == Some(&Value::Bool(true)) {
                    "1"
                } else {
                    "0"
                }
            ),
            "autostart" => format!("c:{}", field(row, "command")),
            "windowRules" => format!(
                "m:{}\0p:{}\0w:{}",
                field(row, "match"),
                field(row, "placement"),
                field(row, "workspace")
            ),
            _ if js::truthy(row.get("command")) => format!("c:{}", field(row, "command")),
            _ if js::truthy(row.get("keys")) => format!("k:{}", field(row, "keys")),
            _ => stringify(row),
        },
        v => format!("\0{}", plain(v)),
    }
}

/// `String(v)`, numbers as JavaScript writes them.
fn plain(v: &Value) -> String {
    match v {
        Value::Number(_) => number_word(v),
        v => js::string(v),
    }
}

/// `shownListRow(row, key)`: a row as the review reads it.
fn shown_row(row: &Value, key: &str) -> String {
    match row {
        Value::Null => "(empty)".into(),
        Value::Object(_) | Value::Array(_) => {
            if key == "bindings" || js::truthy(row.get("keys")) {
                let keys = match field(row, "keys") {
                    k if k.is_empty() => "?".to_owned(),
                    k => k,
                };
                if js::truthy(row.get("unbind")) {
                    return format!("{keys} unbound");
                }
                let cmd = match field(row, "command") {
                    c if c.is_empty() => "(no command)".to_owned(),
                    c => c,
                };
                let label = field(row, "label");
                return if label.is_empty() {
                    format!("{keys} → {cmd}")
                } else {
                    format!("{keys} → {cmd}  ({label})")
                };
            }
            if key == "autostart" || js::truthy(row.get("command")) {
                return match field(row, "command") {
                    c if c.is_empty() => "(no command)".into(),
                    c => c,
                };
            }
            if key == "windowRules" || js::truthy(row.get("match")) {
                let mut bits = vec![match field(row, "match") {
                    m if m.is_empty() => "?".to_owned(),
                    m => m,
                }];
                if js::truthy(row.get("placement")) {
                    bits.push(field(row, "placement"));
                }
                if js::truthy(row.get("workspace")) {
                    bits.push(format!("workspace {}", plain(&row["workspace"])));
                }
                if js::truthy(row.get("width")) && js::truthy(row.get("height")) {
                    bits.push(format!(
                        "{}×{}",
                        plain(&row["width"]),
                        plain(&row["height"])
                    ));
                }
                return bits.join(", ");
            }
            stringify(row)
        }
        v => plain(v),
    }
}

/// `listDiffLines(from, to, key)`: rows added (+), changed (~) and removed (-).
fn diff_lines(from: &Value, to: &Value, key: &str) -> Vec<String> {
    let before = from.as_array().map_or(&[][..], Vec::as_slice);
    let after = to.as_array().map_or(&[][..], Vec::as_slice);
    let mut old: Map<String, Value> = Map::new();
    for r in before {
        old.insert(row_id(r, key), r.clone());
    }
    let new: HashSet<String> = after.iter().map(|r| row_id(r, key)).collect();
    let mut lines = Vec::new();
    for r in after {
        match old.get(&row_id(r, key)) {
            None => lines.push(format!("    + {}", shown_row(r, key))),
            Some(o) if stringify(&normalize_row(o)) != stringify(&normalize_row(r)) => {
                lines.push(format!("    ~ {}", shown_row(r, key)))
            }
            Some(_) => {}
        }
    }
    for r in before {
        if !new.contains(&row_id(r, key)) {
            lines.push(format!("    - {}", shown_row(r, key)));
        }
    }
    lines
}

/// `changeLines(plan)`.
pub fn change_lines(plan: &Value) -> String {
    changes(plan)
        .iter()
        .map(|c| {
            let mut line = format!(
                "• {}: {} → {}",
                js::string(&c["label"]),
                shown_value(c.get("from")),
                shown_value(c.get("to"))
            );
            if c["from"].is_array() || c["to"].is_array() {
                let diffs = diff_lines(&c["from"], &c["to"], c["key"].as_str().unwrap_or_default());
                if !diffs.is_empty() {
                    line.push('\n');
                    line.push_str(&diffs.join("\n"));
                }
            }
            let consequence = text_or_empty(c.get("consequence"));
            if !consequence.is_empty() {
                line.push_str("\n    ");
                line.push_str(&consequence);
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bullets(plan: &Value, list: &str, field: &str) -> String {
    plan[list]
        .as_array()
        .into_iter()
        .flatten()
        .map(|w| format!("• {}", plain(&w[field])))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `warningLines(plan)`.
pub fn warning_lines(plan: &Value) -> String {
    bullets(plan, "warnings", "message")
}

/// `blockedLines(plan)`.
pub fn blocked_lines(plan: &Value) -> String {
    bullets(plan, "blocked", "reason")
}

/// `planToJson(plan)`: each change's key, its new value and its old one, as the apply reads
/// them.
pub fn plan_json(plan: &Value) -> String {
    let out: Vec<Value> = changes(plan)
        .iter()
        .map(|c| json!({ "key": c["key"], "value": c["to"], "from": c["from"] }))
        .collect();
    stringify(&json!({ "schema": js::json_number(SCHEMA), "changes": out }))
}
