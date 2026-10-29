//! The Omafile's TOML-lite (`Settings.js`): one `key = value` a line, where a value is a string,
//! a boolean, a number or a flat list of them, and `#` starts a comment after one.

use serde_json::{Map, Value};

use super::{number_text, space, trim};

/// `tomlLine(key, value)`.
pub fn line(key: &str, value: &Value) -> String {
    format!("{key} = {}", value_text(value))
}

/// `tomlValue(value)`.
pub fn value_text(value: &Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => match n.as_f64() {
            Some(f) if f.is_finite() => number_text(f),
            _ => "0".into(),
        },
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(value_text).collect::<Vec<_>>().join(", ")
        ),
        Value::Null => "\"\"".into(),
        v => format!("\"{}\"", escape(&crate::js::string(v))),
    }
}

/// `escapeTomlString(value)`: one line, whatever it holds.
pub fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// What is left after a closing quote or bracket is nothing, or a comment.
fn tail_ok(rest: &str) -> bool {
    let rest = rest.trim_start_matches(space);
    rest.is_empty() || rest.starts_with('#')
}

/// `parseTomlString(text)`: a quoted string with its escapes, none when it is not one.
fn string(text: &str) -> Option<Value> {
    let mut out = String::new();
    let mut chars = text.char_indices().skip(1);
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return tail_ok(&text[i + 1..]).then(|| Value::String(out)),
            '\\' => {
                let (_, n) = chars.next()?;
                out.push(match n {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    _ => return None,
                });
            }
            '\n' | '\r' => return None,
            c => out.push(c),
        }
    }
    None
}

/// `parseTomlArray(text)`: a flat list, split on the commas outside quotes.
fn array(text: &str) -> Option<Value> {
    if !text.starts_with('[') {
        return None;
    }
    let mut items = Vec::new();
    let mut buf = String::new();
    let (mut in_string, mut escaped) = (false, false);
    for (i, c) in text.char_indices().skip(1) {
        if in_string {
            buf.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                buf.push(c);
            }
            ',' => {
                let piece = trim(&buf);
                if piece.is_empty() {
                    return None;
                }
                items.push(value(piece)?);
                buf.clear();
            }
            ']' => {
                let last = trim(&buf);
                if !last.is_empty() {
                    items.push(value(last)?);
                }
                return tail_ok(&text[i + 1..]).then_some(Value::Array(items));
            }
            c => buf.push(c),
        }
    }
    None
}

fn is_int(t: &str) -> bool {
    let d = t.strip_prefix('-').unwrap_or(t);
    !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit())
}

fn is_decimal(t: &str) -> bool {
    let d = t.strip_prefix('-').unwrap_or(t);
    match d.split_once('.') {
        Some((whole, frac)) => {
            whole.bytes().all(|b| b.is_ascii_digit())
                && !frac.is_empty()
                && frac.bytes().all(|b| b.is_ascii_digit())
        }
        None => false,
    }
}

/// `parseTomlValue(raw)`: none when it is not a value.
pub fn value(raw: &str) -> Option<Value> {
    let mut text = raw.trim_end_matches(space);
    if text.is_empty() {
        return None;
    }
    if text.starts_with('"') {
        return string(text);
    }
    if text.starts_with('[') {
        return array(text);
    }
    if let Some(hash) = text.find('#') {
        text = text[..hash].trim_end_matches(space);
        if text.is_empty() {
            return None;
        }
    }
    match text {
        "true" => return Some(Value::Bool(true)),
        "false" => return Some(Value::Bool(false)),
        _ => {}
    }
    if is_int(text) || is_decimal(text) {
        let (sign, digits) = match text.strip_prefix('-') {
            Some(d) => (-1.0, d),
            None => (1.0, text),
        };
        let digits = if digits.starts_with('.') {
            format!("0{digits}")
        } else {
            digits.to_owned()
        };
        return digits
            .parse::<f64>()
            .ok()
            .map(|n| crate::js::json_number(sign * n));
    }
    None
}

fn key_ok(key: &str) -> bool {
    key.split('.').all(|part| {
        let mut chars = part.chars();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric())
    })
}

/// `parseTomlLite(lines, section, errors)`: the settings a block sets, and why a line was not
/// one.
pub fn parse(lines: &[&str], section: &str, errors: &mut Vec<Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for (i, raw) in lines.iter().enumerate() {
        let raw = trim(raw);
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let eq = match raw.find('=') {
            Some(eq) if eq >= 1 => eq,
            _ => {
                errors.push(Value::from(format!(
                    "atmos:{section} line {} is not a setting: {raw}",
                    i + 1
                )));
                continue;
            }
        };
        let key = raw[..eq].trim_end_matches(space);
        let rest = raw[eq + 1..].trim_start_matches(space);
        if !key_ok(key) {
            errors.push(Value::from(format!(
                "atmos:{section} has a bad setting name: {key}"
            )));
            continue;
        }
        match value(rest) {
            Some(v) => {
                out.insert(key.to_owned(), v);
            }
            None => errors.push(Value::from(format!(
                "atmos:{section} could not read a value for {key}: {rest}"
            ))),
        }
    }
    out
}
