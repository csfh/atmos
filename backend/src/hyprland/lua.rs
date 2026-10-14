//! Lua text helpers shared by every sentinel block: quoting, number formatting,
//! a small call-argument parser, comment detection, and the sentinel markers.

use regex_lite::Regex;
use serde_json::{Map, Value};

/// A cached regex, compiled once per call site.
macro_rules! re {
    ($pattern:expr) => {{
        static RE: std::sync::OnceLock<regex_lite::Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex_lite::Regex::new($pattern).expect("valid regex"))
    }};
}
pub(crate) use re;

// ---- Python-flavoured value helpers ----------------------------------------

/// Python truthiness of a JSON value.
pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// `str(value or "")`.
pub fn str_or_empty(value: Option<&Value>) -> String {
    match value {
        Some(v) if truthy(v) => match v {
            Value::String(s) => s.clone(),
            Value::Bool(_) => "True".into(),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i.to_string()
                } else {
                    let f = n.as_f64().unwrap_or(0.0);
                    if f.fract() == 0.0 && f.abs() < 1e16 {
                        format!("{f:.1}")
                    } else {
                        f.to_string()
                    }
                }
            }
            _ => String::new(),
        },
        _ => String::new(),
    }
}

/// `float(value)`: numbers, numeric strings and booleans; None otherwise.
pub fn py_float(value: Option<&Value>) -> Option<f64> {
    let n = match value? {
        Value::Number(n) => n.as_f64()?,
        Value::Bool(b) => f64::from(u8::from(*b)),
        Value::String(s) => s.trim().replace('_', "").parse::<f64>().ok()?,
        _ => return None,
    };
    n.is_finite().then_some(n)
}

pub fn clamp_int(raw: Option<&Value>, lo: i64, hi: i64, fallback: i64) -> i64 {
    let n = py_float(raw).map_or(fallback, |f| f.round_ties_even() as i64);
    n.clamp(lo, hi)
}

pub fn clamp_float(raw: Option<&Value>, lo: f64, hi: f64, fallback: f64) -> f64 {
    let n = py_float(raw).unwrap_or(fallback).clamp(lo, hi);
    (n * 1000.0).round_ties_even() / 1000.0
}

pub fn as_bool(raw: Option<&Value>, fallback: bool) -> bool {
    match raw {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => match s.as_str() {
            "true" | "on" | "1" => true,
            "false" | "off" | "0" => false,
            _ => fallback,
        },
        Some(Value::Number(n)) => match n.as_f64() {
            Some(1.0) => true,
            Some(0.0) => false,
            _ => fallback,
        },
        _ => fallback,
    }
}

// ---- Lua output -------------------------------------------------------------

pub fn lua_number(n: f64) -> String {
    if (n - n.round_ties_even()).abs() < 1e-9 {
        return format!("{}", n.round_ties_even() as i64);
    }
    let rounded = (n * 1000.0).round_ties_even() / 1000.0;
    let text = format!("{rounded:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() {
        "0".into()
    } else {
        text.into()
    }
}

pub fn lua_bool(v: bool) -> &'static str {
    if v {
        "true"
    } else {
        "false"
    }
}

pub fn lua_string(v: &str) -> String {
    let mut out = String::with_capacity(v.len() + 2);
    out.push('"');
    for c in v.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '"' => out.push_str("\\\""),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn unescape_lua(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            out.push(match chars[i + 1] {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                other => other,
            });
            i += 2;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

// ---- Field lookups inside a block's text --------------------------------------

pub fn lua_bool_field(body: &str, name: &str) -> Option<bool> {
    let re = Regex::new(&format!(r"{}\s*=\s*(true|false)", regex_lite::escape(name))).ok()?;
    re.captures(body).map(|m| &m[1] == "true")
}

/// The matched number text, or "" when absent.
pub fn lua_num_field(block: &str, name: &str) -> String {
    let pattern = format!(
        r"{}\s*=\s*(-?[0-9]+(?:\.[0-9]+)?)",
        regex_lite::escape(name)
    );
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(block).map(|m| m[1].to_string()))
        .unwrap_or_default()
}

pub fn lua_str_field(block: &str, name: &str) -> String {
    let pattern = format!(r#"{}\s*=\s*"((?:\\.|[^"\\])*)""#, regex_lite::escape(name));
    Regex::new(&pattern)
        .ok()
        .and_then(|re| re.captures(block).map(|m| unescape_lua(&m[1])))
        .unwrap_or_default()
}

/// `float(raw) if "." in raw else int(raw)` as a JSON number.
pub fn num_value(raw: &str) -> Option<Value> {
    if raw.contains('.') {
        raw.parse::<f64>().ok().map(Value::from)
    } else {
        raw.parse::<i64>().ok().map(Value::from)
    }
}

// ---- Sentinels --------------------------------------------------------------

pub fn sentinel_bounds(text: &str, begin: &str, end: &str) -> Option<(usize, usize)> {
    let start = text.find(begin)?;
    let stop = text[start + begin.len()..].find(end)? + start + begin.len();
    Some((start, stop + end.len()))
}

pub fn strip_sentinel(text: &str, begin: &str, end: &str) -> String {
    let Some((start, stop)) = sentinel_bounds(text, begin, end) else {
        return text.to_string();
    };
    let before = text[..start].trim_end();
    let after = text[stop..].trim_start();
    match (before.is_empty(), after.is_empty()) {
        (false, false) => format!("{before}\n\n{after}"),
        (false, true) => before.to_string(),
        _ => after.to_string(),
    }
}

pub fn replace_sentinel(text: &str, begin: &str, end: &str, block: &str) -> String {
    let body = block.trim_end();
    match sentinel_bounds(text, begin, end) {
        None => {
            let trimmed = text.trim_end();
            if trimmed.is_empty() {
                format!("{body}\n")
            } else {
                format!("{trimmed}\n\n{body}\n")
            }
        }
        Some((start, stop)) => format!("{}{}{}", &text[..start], body, &text[stop..]),
    }
}

/// True when `at` lies after a `--` that is not inside a string on its line.
pub fn in_line_comment(src: &str, at: usize) -> bool {
    let bytes = src.as_bytes();
    let line_start = src[..at].rfind('\n').map_or(0, |i| i + 1);
    let (mut i, mut in_str) = (line_start, false);
    while i < at {
        let ch = bytes[i];
        if in_str {
            if ch == b'\\' {
                i += 2;
                continue;
            }
            if ch == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if ch == b'"' {
            in_str = true;
            i += 1;
            continue;
        }
        if ch == b'-' && bytes.get(i + 1) == Some(&b'-') {
            return true;
        }
        i += 1;
    }
    false
}

// ---- A small Lua value/call parser ------------------------------------------

/// A parsed Lua value; `Nil` is the `nil` literal.
pub fn skip_ws(text: &str, mut i: usize) -> usize {
    let bytes = text.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r' | b'\n') {
            i += 1;
        }
        if i + 1 >= bytes.len() || &bytes[i..i + 2] != b"--" {
            break;
        }
        i = text[i + 2..]
            .find('\n')
            .map_or(bytes.len(), |nl| i + 2 + nl + 1);
    }
    i
}

fn ident_cont(text: &str, i: usize) -> bool {
    text[i..]
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
}

fn parse_string(text: &str, start: usize) -> Option<(String, usize)> {
    let rest = text.get(start..)?;
    let mut chars = rest.char_indices();
    if chars.next()?.1 != '"' {
        return None;
    }
    let mut out = String::new();
    while let Some((at, ch)) = chars.next() {
        match ch {
            '\\' => {
                let (_, next) = chars.next()?;
                out.push(match next {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    other => other,
                });
            }
            '"' => return Some((out, start + at + 1)),
            c => out.push(c),
        }
    }
    None
}

fn parse_value(text: &str, i: usize) -> Option<(Value, usize)> {
    let i = skip_ws(text, i);
    let rest = text.get(i..)?;
    let ch = rest.chars().next()?;
    if ch == '"' {
        return parse_string(text, i).map(|(s, at)| (Value::String(s), at));
    }
    if ch == '{' {
        return parse_table(text, i);
    }
    if rest.starts_with("nil") && !ident_cont(text, i + 3) {
        return Some((Value::Null, i + 3));
    }
    if rest.starts_with("true") && !ident_cont(text, i + 4) {
        return Some((Value::Bool(true), i + 4));
    }
    if rest.starts_with("false") && !ident_cont(text, i + 5) {
        return Some((Value::Bool(false), i + 5));
    }
    let m = re!(r"^-?\d+(?:\.\d+)?").find(rest)?;
    let raw = m.as_str();
    let value = if raw.contains('.') {
        Value::from(raw.parse::<f64>().ok()?)
    } else {
        Value::from(raw.parse::<i64>().ok()?)
    };
    Some((value, i + raw.len()))
}

fn parse_table(text: &str, i: usize) -> Option<(Value, usize)> {
    let mut i = skip_ws(text, i);
    if text.as_bytes().get(i) != Some(&b'{') {
        return None;
    }
    i = skip_ws(text, i + 1);
    let (mut obj, mut arr) = (Map::new(), Vec::new());
    while i < text.len() && text.as_bytes()[i] != b'}' {
        i = skip_ws(text, i);
        if text.as_bytes().get(i) == Some(&b'}') {
            break;
        }
        let ident = re!(r"^[A-Za-z_][A-Za-z0-9_]*")
            .find(&text[i..])
            .map(|m| m.as_str().to_string());
        let j = skip_ws(text, i + ident.as_ref().map_or(0, String::len));
        if ident.is_some() && text.as_bytes().get(j) == Some(&b'=') {
            let (value, at) = parse_value(text, j + 1)?;
            obj.insert(ident.unwrap_or_default(), value);
            i = at;
        } else {
            let (value, at) = parse_value(text, i)?;
            arr.push(value);
            i = at;
        }
        i = skip_ws(text, i);
        if text.as_bytes().get(i) == Some(&b',') {
            i += 1;
        }
    }
    if text.as_bytes().get(i) != Some(&b'}') {
        return None;
    }
    let value = if !arr.is_empty() && obj.is_empty() {
        Value::Array(arr)
    } else {
        Value::Object(obj)
    };
    Some((value, i + 1))
}

/// The arguments of a call whose `(` was just consumed, and the index after `)`.
pub fn parse_call_args(text: &str, i: usize) -> Option<(Vec<Value>, usize)> {
    let mut args = Vec::new();
    let mut i = skip_ws(text, i);
    while i < text.len() && text.as_bytes()[i] != b')' {
        let (value, at) = parse_value(text, i)?;
        args.push(value);
        i = skip_ws(text, at);
        if text.as_bytes().get(i) == Some(&b',') {
            i = skip_ws(text, i + 1);
        }
    }
    (text.as_bytes().get(i) == Some(&b')')).then_some((args, i + 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_format_like_the_script() {
        assert_eq!(lua_number(5.0), "5");
        assert_eq!(lua_number(0.15), "0.15");
        assert_eq!(lua_number(0.4), "0.4");
        assert_eq!(lua_number(-0.5), "-0.5");
        assert_eq!(lua_number(0.0), "0");
        assert_eq!(lua_number(2.0004), "2");
    }

    #[test]
    fn strings_round_trip_through_lua_quoting() {
        let raw = "a \"quoted\"\\ line\nnext\ttab";
        let quoted = lua_string(raw);
        let (parsed, _) = parse_string(&quoted, 0).unwrap();
        assert_eq!(parsed, raw);
        assert_eq!(unescape_lua(&quoted[1..quoted.len() - 1]), raw);
    }

    #[test]
    fn calls_parse_tables_nil_and_comments() {
        let text = "\"a\", nil, { float = true, size = { 1, 2 } } -- tail\n)";
        let (args, _) = parse_call_args(text, 0).unwrap();
        assert_eq!(args[0], json!("a"));
        assert_eq!(args[1], Value::Null);
        assert_eq!(args[2]["size"], json!([1, 2]));
        assert!(parse_call_args("\"a\"", 0).is_none());
    }

    #[test]
    fn comments_are_found_outside_strings_only() {
        assert!(in_line_comment("x -- o.bind(", 5));
        assert!(!in_line_comment("\"--\" o.bind(", 5));
    }

    #[test]
    fn sentinels_strip_and_replace() {
        let text = "a\n-- b\nx\n-- e\nz\n";
        assert_eq!(strip_sentinel(text, "-- b", "-- e"), "a\n\nz\n");
        assert_eq!(
            replace_sentinel(text, "-- b", "-- e", "-- b\ny\n-- e"),
            "a\n-- b\ny\n-- e\nz\n"
        );
        assert_eq!(
            replace_sentinel("", "-- b", "-- e", "-- b\n-- e"),
            "-- b\n-- e\n"
        );
    }
}
