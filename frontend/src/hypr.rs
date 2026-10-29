//! The Hyprland look and input the window shows, as `services/HyprPrefs.js` makes them of the
//! snapshot's `hyprLook` and `hyprInput`: each field clamped to its range, or its default when it
//! is not a value of its kind (`clampLook`, `clampInput`). `tests/catalog_controls.rs` runs both
//! under Node against these.

use serde_json::Value;

use crate::js;

/// What a field holds and what it is when the snapshot's value is not one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// `asBool`.
    Bool(bool),
    /// `clampInt(min, max, default)`.
    Int(i64, i64, i64),
    /// `clampFloat(min, max, default)`, to three places.
    Float(f64, f64, f64),
    /// `dwindle` unless it says `scrolling`.
    Layout,
    /// `flat`, `adaptive`, or nothing.
    Accel,
    /// One line of at most 128 characters, trimmed (`sanitizeSwallowRegex`).
    Swallow,
}

pub const LOOK: &[(&str, Kind)] = &[
    ("gapsIn", Kind::Int(0, 64, 5)),
    ("gapsOut", Kind::Int(0, 64, 10)),
    ("borderSize", Kind::Int(0, 16, 2)),
    ("rounding", Kind::Int(0, 32, 0)),
    ("blur", Kind::Bool(false)),
    ("shadow", Kind::Bool(false)),
    ("layout", Kind::Layout),
    ("columnWidth", Kind::Float(0.2, 1.0, 0.49)),
    ("dimInactive", Kind::Bool(false)),
    ("dimStrength", Kind::Float(0.0, 1.0, 0.15)),
    ("animations", Kind::Bool(true)),
    ("cursorHideOnKey", Kind::Bool(true)),
    ("cursorWarp", Kind::Bool(true)),
    ("cursorSize", Kind::Int(8, 64, 24)),
    ("allowTearing", Kind::Bool(false)),
    ("resizeOnBorder", Kind::Bool(false)),
    ("activeOpacity", Kind::Float(0.2, 1.0, 1.0)),
    ("inactiveOpacity", Kind::Float(0.2, 1.0, 1.0)),
    ("preserveSplit", Kind::Bool(false)),
    ("focusOnActivate", Kind::Bool(false)),
    ("enableSwallow", Kind::Bool(false)),
    ("swallowRegex", Kind::Swallow),
    ("onFocusUnderFullscreen", Kind::Int(0, 2, 1)),
];

/// The input fields a row reads or writes; the keyboard layout overrides are not among them.
pub const INPUT: &[(&str, Kind)] = &[
    ("sensitivity", Kind::Float(-1.0, 1.0, 0.0)),
    ("accelProfile", Kind::Accel),
    ("emulateDiscreteScroll", Kind::Int(0, 2, 1)),
    ("naturalScroll", Kind::Bool(false)),
    ("scrollFactor", Kind::Float(0.1, 3.0, 0.4)),
    ("clickfinger", Kind::Bool(true)),
    ("disableWhileTyping", Kind::Bool(true)),
    ("drag3fg", Kind::Int(0, 1, 0)),
    ("repeatRate", Kind::Int(10, 100, 40)),
    ("repeatDelay", Kind::Int(100, 1000, 250)),
    ("numlock", Kind::Bool(true)),
    ("followMouse", Kind::Int(0, 3, 1)),
    ("keyPressDpms", Kind::Bool(true)),
    ("mouseMoveDpms", Kind::Bool(true)),
    ("workspaceGesture", Kind::Bool(false)),
];

pub fn kind(table: &[(&str, Kind)], field: &str) -> Option<Kind> {
    table.iter().find(|(f, _)| *f == field).map(|(_, k)| *k)
}

/// The field of `raw`, a snapshot's `hyprLook` or `hyprInput`, as the clamp makes it.
pub fn clamp(kind: Kind, raw: &Value, field: &str) -> Value {
    let src = raw.as_object().and_then(|o| o.get(field));
    match kind {
        Kind::Bool(default) => Value::Bool(as_bool(src, default)),
        Kind::Int(min, max, default) => {
            let mut n = js::round(js::number(src));
            if !n.is_finite() {
                n = default as f64;
            }
            js::json_number(n.clamp(min as f64, max as f64))
        }
        Kind::Float(min, max, default) => {
            let mut n = js::number(src);
            if !n.is_finite() {
                n = default;
            }
            js::json_number(js::round(n.clamp(min, max) * 1000.0) / 1000.0)
        }
        Kind::Layout => {
            let text = if js::truthy(src) {
                js::string(src.unwrap_or(&Value::Null))
            } else {
                String::new()
            };
            Value::from(if text == "scrolling" {
                "scrolling"
            } else {
                "dwindle"
            })
        }
        Kind::Accel => {
            let text = if js::truthy(src) {
                js::string(src.unwrap_or(&Value::Null))
            } else {
                String::new()
            };
            Value::from(if matches!(text.as_str(), "flat" | "adaptive") {
                text
            } else {
                String::new()
            })
        }
        Kind::Swallow => {
            let text = if js::truthy(src) {
                js::string(src.unwrap_or(&Value::Null))
            } else {
                String::new()
            };
            let ok = !text.contains(['\n', '\r']);
            let text = text.trim();
            Value::from(if ok && text.chars().count() <= 128 {
                text
            } else {
                ""
            })
        }
    }
}

/// `asBool`: a boolean, or one of the words and numbers that mean one, else `default`.
fn as_bool(v: Option<&Value>, default: bool) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) if matches!(s.as_str(), "true" | "on" | "1") => true,
        Some(Value::String(s)) if matches!(s.as_str(), "false" | "off" | "0") => false,
        Some(Value::Number(n)) if n.as_f64() == Some(1.0) => true,
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => false,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_field_is_clamped_or_its_default() {
        let look = json!({"gapsIn": "100", "blur": "on", "columnWidth": 0.12345, "layout": 3});
        assert_eq!(clamp(Kind::Int(0, 64, 5), &look, "gapsIn"), json!(64));
        assert_eq!(clamp(Kind::Int(0, 64, 5), &look, "gapsOut"), json!(5));
        assert_eq!(clamp(Kind::Bool(false), &look, "blur"), json!(true));
        assert_eq!(
            clamp(Kind::Float(0.2, 1.0, 0.49), &look, "columnWidth"),
            json!(0.2)
        );
        assert_eq!(clamp(Kind::Layout, &look, "layout"), json!("dwindle"));
        assert_eq!(
            clamp(Kind::Bool(true), &json!("x"), "animations"),
            json!(true)
        );
    }
}
