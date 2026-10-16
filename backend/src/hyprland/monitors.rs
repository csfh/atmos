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

fn number_text(table: &Map<String, Value>, key: &str) -> String {
    match table.get(key) {
        Some(Value::Number(number)) => {
            if let Some(whole) = number.as_i64() {
                whole.to_string()
            } else {
                number.as_f64().unwrap_or(0.0).to_string()
            }
        }
        _ => String::new(),
    }
}

fn monitor_row(arg: Option<&Value>) -> Option<Value> {
    let table = arg?.as_object()?;
    let output = string_field(table, "output");
    if output.is_empty() {
        return None;
    }
    let int =
        |raw: String, fallback: i64| raw.parse::<f64>().map_or(fallback, |f| f.trunc() as i64);
    let scale = number_text(table, "scale").parse::<f64>().unwrap_or(1.0);
    let or = |key: &str, default: &str| {
        let value = string_field(table, key);
        if value.is_empty() {
            default.to_string()
        } else {
            value
        }
    };
    Some(json!({
        "output": output,
        "mode": or("mode", "preferred"),
        "position": or("position", "auto"),
        "scale": scale,
        "transform": int(number_text(table, "transform"), 0),
        "disabled": table.get("disabled") == Some(&Value::Bool(true)),
        "mirror": string_field(table, "mirror"),
        "vrr": int(number_text(table, "vrr"), 0),
        "bitdepth": int(number_text(table, "bitdepth"), 8),
        "cm": string_field(table, "cm"),
    }))
}

pub fn parse(text: &str) -> Value {
    let body = match sentinel_bounds(text, BEGIN, END) {
        Some((start, stop)) => &text[start..stop],
        None => "",
    };
    let mut out = Vec::new();
    live_calls(body, "hl.monitor(", |args| {
        if let Some(row) = monitor_row(args.first()) {
            out.push(row);
        }
    });
    Value::Array(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_earlier_end_marker_does_not_hide_the_block() {
        let text = "-- atmos:monitors end\n-- atmos:monitors begin\nhl.monitor({ output = \"DP-1\" })\n-- atmos:monitors end\n";
        let rows = parse(text);
        assert_eq!(rows[0]["output"], "DP-1");
    }
}
