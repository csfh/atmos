//! The Omafile as Markdown (`Settings.js`): a person reads the prose, and only the fenced
//! ` ```toml atmos:<section> ` and ` ```json atmos:<list> ` blocks are read back.

use serde_json::{Map, Value, json};

use super::canon::export_list;
use super::{
    CATALOG, SCHEMA, SECTIONS, display_value, read_value, stringify, text_or_empty, toml, trim,
};

/// What the file says about where it came from (`meta` of `exportMarkdown`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Meta {
    /// When it was written, as `Date.toISOString()` gives it.
    pub exported: String,
    pub hardware: String,
}

fn either(a: &str, b: Option<&Value>) -> String {
    if a.is_empty() {
        text_or_empty(b)
    } else {
        a.to_owned()
    }
}

/// `jsonBlock(list)`: a row a line.
fn json_block(list: &[Value]) -> String {
    if list.is_empty() {
        return "[]".into();
    }
    let rows: Vec<String> = list.iter().map(|r| format!("  {}", stringify(r))).collect();
    format!("[\n{}\n]", rows.join(",\n"))
}

/// `exportMarkdown(snapshot, keys, meta)`: the chosen settings, a section each, with what is
/// only reported listed in words.
pub fn export(snap: &Value, keys: &[&str], meta: &Meta) -> String {
    let mut lines: Vec<String> = Vec::new();
    let host = either("", snap.get("hostname"));
    lines.push("# Omafile".into());
    lines.push(String::new());
    lines.push(format!(
        "Omarchy configuration from {}.",
        if host.is_empty() {
            "an Omarchy machine"
        } else {
            host.as_str()
        }
    ));
    lines.push("This file is how you share a whole Omarchy desktop. Read it before you apply it. Atmos shows every change first.".into());
    lines.push(String::new());
    let atmos = ["atmosRevision", "atmosVersion"]
        .iter()
        .map(|k| text_or_empty(snap.get(*k)))
        .find(|s| !s.is_empty())
        .unwrap_or_default();
    lines.push("```toml atmos:meta".into());
    lines.push(toml::line("schema", &crate::js::json_number(SCHEMA)));
    lines.push(toml::line("exported", &Value::from(meta.exported.as_str())));
    lines.push(toml::line("hostname", &Value::from(host.as_str())));
    lines.push(toml::line("atmos", &Value::from(atmos)));
    lines.push(toml::line(
        "omarchy",
        &Value::from(text_or_empty(snap.get("omarchyVersion"))),
    ));
    lines.push(toml::line("hardware", &Value::from(meta.hardware.as_str())));
    lines.push("```".into());
    lines.push(String::new());

    for (id, title, note) in SECTIONS {
        let (mut body, mut reported, mut lists) = (Vec::new(), Vec::new(), Vec::new());
        for item in CATALOG.iter().filter(|i| i.section == *id) {
            let Some(value) = read_value(snap, item.key).filter(|v| !v.is_null()) else {
                continue;
            };
            if !item.importable {
                reported.push(format!("- {}: {}", item.label, display_value(value)));
                continue;
            }
            if !keys.contains(&item.key) {
                continue;
            }
            if item.kind == "list" {
                if let Value::Array(rows) = value {
                    lists.push((item.key, export_list(rows)));
                }
                continue;
            }
            body.push(toml::line(item.key, value));
        }
        if body.is_empty() && reported.is_empty() && lists.is_empty() {
            continue;
        }
        lines.push(format!("## {title}"));
        lines.push(String::new());
        lines.push((*note).to_owned());
        lines.push(String::new());
        if !reported.is_empty() {
            lines.extend(reported);
            lines.push(String::new());
        }
        if !body.is_empty() {
            lines.push(format!("```toml atmos:{id}"));
            lines.extend(body);
            lines.push("```".into());
            lines.push(String::new());
        }
        for (key, rows) in lists {
            lines.push(format!("```json atmos:{key}"));
            lines.push(json_block(&rows));
            lines.push("```".into());
            lines.push(String::new());
        }
    }
    format!("{}\n", lines.join("\n").trim_end_matches('\n'))
}

/// A fence that opens a block: its kind and its name.
fn opening(trimmed: &str) -> Option<(&str, &str)> {
    let rest = trimmed
        .strip_prefix("```")?
        .trim_start_matches(super::space);
    let kind = ["toml", "json"].into_iter().find(|k| rest.starts_with(k))?;
    let rest = &rest[kind.len()..];
    let after = rest.trim_start_matches(super::space);
    if after.len() == rest.len() {
        return None;
    }
    let name = after.strip_prefix("atmos:")?.trim_end_matches(super::space);
    let mut chars = name.chars();
    let ok = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-');
    ok.then_some((kind, name))
}

/// `parseJsonBlock(lines, name, errors)`.
fn json_block_read(lines: &[&str], name: &str, errors: &mut Vec<Value>) -> Map<String, Value> {
    let mut out = Map::new();
    match serde_json::from_str::<Value>(&lines.join("\n")) {
        Ok(v @ Value::Array(_)) => {
            out.insert(name.to_owned(), v);
        }
        Ok(_) => errors.push(Value::from(format!("atmos:{name} is not a list"))),
        Err(_) => errors.push(Value::from(format!("atmos:{name} is not readable JSON"))),
    }
    out
}

/// `parseSettingsMarkdown(text)`: `{ meta, sections: { name: { key: value } }, errors }`.
pub fn parse(text: &str) -> Value {
    let mut sections: Map<String, Value> = Map::new();
    let mut errors: Vec<Value> = Vec::new();
    let mut open: Option<(String, String)> = None;
    let mut buffer: Vec<&str> = Vec::new();
    for line in text.split('\n') {
        let trimmed = trim(line);
        let Some((kind, name)) = open.as_ref() else {
            if let Some((kind, name)) = opening(trimmed) {
                open = Some((kind.to_owned(), name.to_owned()));
                buffer.clear();
            }
            continue;
        };
        if trimmed == "```" {
            let parsed = if kind == "json" {
                json_block_read(&buffer, name, &mut errors)
            } else {
                toml::parse(&buffer, name, &mut errors)
            };
            let section = sections
                .entry(name.clone())
                .or_insert_with(|| Value::Object(Map::new()));
            if let Value::Object(m) = section {
                m.extend(parsed);
            }
            open = None;
            buffer.clear();
            continue;
        }
        buffer.push(line);
    }
    if let Some((_, name)) = open {
        errors.push(Value::from(format!(
            "The atmos:{name} block is never closed"
        )));
    }
    let meta = sections
        .shift_remove("meta")
        .unwrap_or_else(|| Value::Object(Map::new()));
    json!({ "meta": meta, "sections": sections, "errors": errors })
}
