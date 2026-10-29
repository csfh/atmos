//! What JavaScript makes of a JSON value, for the QML code the window restates: `Number(v)`,
//! `Math.round`, `String(v)` and whether a value is truthy. A missing value is `undefined`.
//! `tests/catalog_controls.rs` runs the QML's own functions under Node against what is built on
//! these.

use serde_json::Value;

/// `Number(v)`: NaN for `undefined`, an object, or text that is not a number.
pub fn number(v: Option<&Value>) -> f64 {
    match v {
        None => f64::NAN,
        Some(Value::Null) => 0.0,
        Some(Value::Bool(b)) => f64::from(u8::from(*b)),
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::String(s)) => text_number(s),
        // An array is its text: `[]` is "", `[5]` is "5", and two items are not a number.
        Some(Value::Array(items)) => match items.as_slice() {
            [] => 0.0,
            [one] => text_number(&string(one)),
            _ => f64::NAN,
        },
        Some(Value::Object(_)) => f64::NAN,
    }
}

/// `Number` of a string: blank is 0, a decimal or `Infinity`, or a hexadecimal, octal or binary
/// integer with its prefix.
fn text_number(s: &str) -> f64 {
    let t = s.trim();
    if t.is_empty() {
        return 0.0;
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(digits) = t.strip_prefix(prefix) {
            return u64::from_str_radix(digits, radix).map_or(f64::NAN, |n| n as f64);
        }
    }
    let unsigned = t.strip_prefix(['+', '-']).unwrap_or(t);
    if unsigned == "Infinity" {
        return if t.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    // Rust takes `inf` and `nan`, which JavaScript does not.
    if !unsigned.starts_with(|c: char| c.is_ascii_digit() || c == '.')
        || unsigned.contains(|c: char| c.is_ascii_alphabetic() && c != 'e' && c != 'E')
    {
        return f64::NAN;
    }
    t.parse().unwrap_or(f64::NAN)
}

/// `Math.round`: halves go up.
pub fn round(n: f64) -> f64 {
    (n + 0.5).floor()
}

/// `String(v)` for the values a snapshot holds: an integer without a point, other numbers as
/// Rust writes them, and an array its items joined by commas.
pub fn string(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => match n.as_f64() {
            Some(f) if f.fract() == 0.0 && f.abs() < 1e21 => format!("{}", f as i64),
            Some(f) => f.to_string(),
            None => n.to_string(),
        },
        Value::Array(items) => items
            .iter()
            .map(|i| {
                if i.is_null() {
                    String::new()
                } else {
                    string(i)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}

/// Whether `v` is truthy: not `undefined`, `null`, `false`, 0, NaN or "".
pub fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// A number as JSON: an integer when it has no fraction, as JavaScript's JSON writes it.
pub fn json_number(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9.0e15 {
        Value::from(n as i64)
    } else {
        serde_json::Number::from_f64(n).map_or(Value::Null, Value::Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_are_what_javascript_makes_of_them() {
        let n = |v: Value| number(Some(&v));
        assert!(number(None).is_nan());
        assert_eq!(n(json!(null)), 0.0);
        assert_eq!(n(json!("")), 0.0);
        assert_eq!(n(json!(" 12 ")), 12.0);
        assert_eq!(n(json!("0x10")), 16.0);
        assert_eq!(n(json!("-Infinity")), f64::NEG_INFINITY);
        assert!(n(json!("inf")).is_nan());
        assert!(n(json!("12px")).is_nan());
        assert_eq!(n(json!([])), 0.0);
        assert_eq!(n(json!(["7"])), 7.0);
        assert!(n(json!([1, 2])).is_nan());
        assert!(n(json!({})).is_nan());
        assert_eq!(round(2.5), 3.0);
        assert_eq!(round(-2.5), -2.0);
        assert_eq!(string(&json!(3.0)), "3");
        assert_eq!(string(&json!([1, null, "a"])), "1,,a");
        assert_eq!(json_number(4.0), json!(4));
        assert_eq!(json_number(0.25), json!(0.25));
    }
}
