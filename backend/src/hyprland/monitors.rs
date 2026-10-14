//! The `atmos:monitors` block in monitors.lua.

use super::lua::*;
use serde_json::{json, Map, Value};

pub const BEGIN: &str = "-- atmos:monitors begin";
pub const END: &str = "-- atmos:monitors end";

fn int_of(item: &Value, key: &str, fallback: i64) -> i64 {
    match item.get(key) {
        Some(v) if truthy(v) => py_float(Some(v)).map_or(0, |f| f.trunc() as i64),
        _ => fallback,
    }
}

pub fn serialize(src: &Map<String, Value>) -> String {
    let mut lines = vec![BEGIN.to_string()];
    for item in src
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if !item.is_object() {
            continue;
        }
        let output = str_or_empty(item.get("output")).trim().to_string();
        if !re!(r"^[A-Za-z0-9._-]+$").is_match(&output) {
            continue;
        }
        let or = |key: &str, default: &str| {
            let v = str_or_empty(item.get(key));
            if v.is_empty() {
                default.to_string()
            } else {
                v
            }
        };
        let scale = match item.get("scale") {
            Some(v) if truthy(v) => py_float(Some(v)).unwrap_or(1.0),
            _ => 1.0,
        };
        let mut parts = vec![
            format!("output = {}", lua_string(&output)),
            format!("mode = {}", lua_string(&or("mode", "preferred"))),
            format!("position = {}", lua_string(&or("position", "auto"))),
            format!("scale = {}", lua_number(scale)),
        ];
        let transform = int_of(item, "transform", 0);
        if transform != 0 {
            parts.push(format!("transform = {transform}"));
        }
        if item.get("disabled") == Some(&Value::Bool(true)) {
            parts.push("disabled = true".into());
        }
        let mirror = str_or_empty(item.get("mirror")).trim().to_string();
        if !mirror.is_empty() {
            parts.push(format!("mirror = {}", lua_string(&mirror)));
        }
        let vrr = int_of(item, "vrr", 0);
        if vrr != 0 {
            parts.push(format!("vrr = {vrr}"));
        }
        if int_of(item, "bitdepth", 8) == 10 {
            parts.push("bitdepth = 10".into());
        }
        let cm = str_or_empty(item.get("cm")).trim().to_string();
        if !cm.is_empty() {
            parts.push(format!("cm = {}", lua_string(&cm)));
        }
        lines.push(format!("hl.monitor({{ {} }})", parts.join(", ")));
    }
    lines.push(END.into());
    lines.join("\n")
}

pub fn parse(text: &str) -> Value {
    let body = match (text.find(BEGIN), text.find(END)) {
        (Some(start), Some(stop)) if stop > start => &text[start..stop],
        _ => "",
    };
    let mut out = Vec::new();
    for m in re!(r"hl\.monitor\(\s*\{([^}]*)\}\s*\)").captures_iter(body) {
        let chunk = &m[1];
        let output = lua_str_field(chunk, "output");
        if output.is_empty() {
            continue;
        }
        let num = |key: &str| lua_num_field(chunk, key);
        let int =
            |raw: String, fallback: i64| raw.parse::<f64>().map_or(fallback, |f| f.trunc() as i64);
        let scale = num("scale").parse::<f64>().unwrap_or(1.0);
        let or = |key: &str, default: &str| {
            let v = lua_str_field(chunk, key);
            if v.is_empty() {
                default.to_string()
            } else {
                v
            }
        };
        out.push(json!({
            "output": output,
            "mode": or("mode", "preferred"),
            "position": or("position", "auto"),
            "scale": scale,
            "transform": int(num("transform"), 0),
            "disabled": re!(r"disabled\s*=\s*true").is_match(chunk),
            "mirror": lua_str_field(chunk, "mirror"),
            "vrr": int(num("vrr"), 0),
            "bitdepth": int(num("bitdepth"), 8),
            "cm": lua_str_field(chunk, "cm"),
        }));
    }
    Value::Array(out)
}
