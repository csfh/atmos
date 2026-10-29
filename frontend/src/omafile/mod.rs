//! The Omafile: the Markdown export and import of an Omarchy system's settings, as
//! `services/Settings.js` writes, reads and plans it for the Export page (`pages/ExportPage.qml`).
//! The catalog of what travels is `table.rs`, copied from the JavaScript; the file's writer
//! and reader are `markdown.rs` and `toml.rs`; the planner that turns a file and this machine's
//! snapshot into a reviewable list of changes is `plan.rs`, with the lists it checks row by row
//! in `canon.rs`; and what the page says of a plan is `lines.rs`. Documents and plans are the
//! JavaScript's own JSON shapes, so `tests/omafile_parity.rs` holds each function to its twin.

pub mod canon;
pub mod lines;
pub mod markdown;
pub mod plan;
pub mod table;
pub mod toml;

use serde_json::Value;

pub use table::{CATALOG, Item, SECTIONS};

/// `SETTINGS_SCHEMA`.
pub const SCHEMA: f64 = 1.0;

/// The catalog's item for a key (`catalogByKey()[key]`).
pub fn item(key: &str) -> Option<&'static Item> {
    CATALOG.iter().rev().find(|i| i.key == key)
}

/// JavaScript's `\s`.
pub fn space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// `.replace(/^\s+|\s+$/g, "")`.
pub fn trim(s: &str) -> &str {
    s.trim_matches(space)
}

/// A string's length as JavaScript counts it.
pub fn units(s: &str) -> usize {
    s.encode_utf16().count()
}

/// `String(v || "")`.
pub fn text_or_empty(v: Option<&Value>) -> String {
    if crate::js::truthy(v) {
        v.map_or_else(String::new, crate::js::string)
    } else {
        String::new()
    }
}

/// `String(n)` for a number, as JavaScript writes it.
pub fn number_text(n: f64) -> String {
    if !n.is_finite() {
        return if n.is_nan() {
            "NaN".into()
        } else if n > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        };
    }
    if n == 0.0 {
        return "0".into();
    }
    let a = n.abs();
    if (1e-6..1e21).contains(&a) {
        if n.fract() == 0.0 {
            return format!("{n:.0}");
        }
        return n.to_string();
    }
    let e = format!("{n:e}");
    match e.split_once('e') {
        Some((m, x)) if !x.starts_with('-') => format!("{m}e+{x}"),
        _ => e,
    }
}

/// `JSON.stringify(v)`: keys in the order they were set, numbers as JavaScript writes them.
pub fn stringify(v: &Value) -> String {
    match v {
        Value::Number(n) => n
            .as_f64()
            .filter(|f| f.is_finite())
            .map_or_else(|| "null".into(), number_text),
        Value::String(_) | Value::Null | Value::Bool(_) => v.to_string(),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(stringify).collect::<Vec<_>>().join(",")
        ),
        Value::Object(map) => format!(
            "{{{}}}",
            map.iter()
                .map(|(k, v)| format!("{}:{}", Value::from(k.as_str()), stringify(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

/// `===` between a value and one that may be `undefined`: objects and lists are never equal,
/// as two of them are never the same object here.
pub fn strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(Value::Number(x)), Some(Value::Number(y))) => x.as_f64() == y.as_f64(),
        (Some(Value::String(x)), Some(Value::String(y))) => x == y,
        (Some(Value::Bool(x)), Some(Value::Bool(y))) => x == y,
        (Some(Value::Null), Some(Value::Null)) => true,
        _ => false,
    }
}

/// `readValue(source, key)`: a dot walks into a nested object; none when a step is missing.
pub fn read_value<'a>(source: &'a Value, key: &str) -> Option<&'a Value> {
    let mut node = source;
    for part in key.split('.') {
        match node {
            Value::Object(m) => node = m.get(part)?,
            Value::Array(a) => node = part.parse::<usize>().ok().and_then(|i| a.get(i))?,
            _ => return None,
        }
    }
    Some(node)
}

/// `countLabel(n, one, many)`.
pub fn count_label(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// `displayValue(value)`.
pub fn display_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "on".into(),
        Value::Bool(false) => "off".into(),
        Value::Array(a) => count_label(a.len(), "entry", "entries"),
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), number_text),
        _ => crate::js::string(v),
    }
}

/// `shownValue(value)`: "not set" for nothing.
pub fn shown_value(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => "not set".into(),
        Some(Value::String(s)) if s.is_empty() => "not set".into(),
        Some(v) => display_value(v),
    }
}

/// `sectionKeys(section)`: the importable keys in one section.
pub fn section_keys(section: &str) -> Vec<&'static str> {
    CATALOG
        .iter()
        .filter(|i| i.section == section && i.importable)
        .map(|i| i.key)
        .collect()
}

/// A section the page offers: its id, heading, note, how many settings, and whether it starts
/// chosen (`selectableSections()`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    pub id: &'static str,
    pub title: &'static str,
    pub note: &'static str,
    pub count: usize,
    pub by_default: bool,
}

/// `selectableSections()`: the sections that hold something you can carry.
pub fn selectable_sections() -> Vec<Offered> {
    SECTIONS
        .iter()
        .filter_map(|(id, title, note)| {
            let count = section_keys(id).len();
            (count > 0).then(|| Offered {
                id,
                title,
                note,
                count,
                by_default: *id != "system",
            })
        })
        .collect()
}

/// `keysForSections(ids)`: the importable keys of the sections chosen, in catalog order.
pub fn keys_for_sections(ids: &[&str]) -> Vec<&'static str> {
    CATALOG
        .iter()
        .filter(|i| i.importable && ids.contains(&i.section))
        .map(|i| i.key)
        .collect()
}

/// `exportFileName()`: an Omafile is called one, the way a Brewfile is.
pub fn file_name() -> &'static str {
    "Omafile.md"
}
