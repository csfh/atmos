//! The `atmos:workspaces` block in atmos.lua.

use super::lua::*;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

pub const BEGIN: &str = "-- atmos:workspaces begin";
pub const END: &str = "-- atmos:workspaces end";

pub fn serialize(src: &Map<String, Value>) -> String {
    let items = src.get("items").and_then(Value::as_array);
    let count = 10;
    let wrap = src.get("wrapSwitch") != Some(&Value::Bool(false));
    let wheel = src.get("wheelSwitch") != Some(&Value::Bool(false));
    let (wrap_next, wrap_prev) = if wrap { ("e+1", "e-1") } else { ("r+1", "r-1") };
    let mut lines = vec![
        BEGIN.to_string(),
        format!("-- atmos:wrapSwitch = {}", lua_bool(wrap)),
        format!("-- atmos:wheelSwitch = {}", lua_bool(wheel)),
    ];
    let mut seen: HashSet<String> = HashSet::new();
    let mut rows: Vec<Value> = Vec::new();
    for item in items.into_iter().flatten() {
        let Some(map) = item.as_object() else {
            continue;
        };
        let id = {
            let first = str_or_empty(map.get("id"));
            if first.is_empty() {
                str_or_empty(map.get("workspace"))
            } else {
                first
            }
        };
        let id = id.trim().to_string();
        if id.is_empty() || !seen.insert(id) {
            continue;
        }
        rows.push(item.clone());
    }
    for i in 1..=count {
        let sid = i.to_string();
        if seen.insert(sid.clone()) {
            rows.push(json!({"id": sid, "persistent": true}));
        }
    }
    for item in &rows {
        let wid = str_or_empty(item.get("id")).trim().to_string();
        if re!(r"^(?:[1-9]|10)$").is_match(&wid) && wid.parse::<i64>().unwrap_or(0) > count {
            continue;
        }
        let mut parts = vec![format!("workspace = {}", lua_string(&wid))];
        if item.get("persistent") != Some(&Value::Bool(false)) {
            parts.push("persistent = true".into());
        }
        let name = str_or_empty(item.get("name")).trim().to_string();
        if !name.is_empty() {
            parts.push(format!("default_name = {}", lua_string(&name)));
        }
        let monitor = str_or_empty(item.get("monitor")).trim().to_string();
        if !monitor.is_empty() {
            parts.push(format!("monitor = {}", lua_string(&monitor)));
        }
        if item.get("isDefault") == Some(&Value::Bool(true))
            || item.get("default") == Some(&Value::Bool(true))
        {
            parts.push("default = true".into());
        }
        let empty = {
            let first = str_or_empty(item.get("onCreatedEmpty"));
            if first.is_empty() {
                str_or_empty(item.get("on_created_empty"))
            } else {
                first
            }
        };
        let empty = empty.trim();
        if !empty.is_empty() {
            parts.push(format!("on_created_empty = {}", lua_string(empty)));
        }
        lines.push(format!("hl.workspace_rule({{ {} }})", parts.join(", ")));
    }
    lines.push(r#"hl.unbind("SUPER + TAB")"#.into());
    lines.push(r#"hl.unbind("SUPER + SHIFT + TAB")"#.into());
    lines.push(format!(
        r#"o.bind("SUPER + TAB", "Next workspace", hl.dsp.focus({{ workspace = "{wrap_next}" }}))"#
    ));
    lines.push(format!(
        r#"o.bind("SUPER + SHIFT + TAB", "Previous workspace", hl.dsp.focus({{ workspace = "{wrap_prev}" }}))"#
    ));
    lines.push(r#"hl.unbind("SUPER + mouse_down")"#.into());
    lines.push(r#"hl.unbind("SUPER + mouse_up")"#.into());
    if wheel {
        lines.push(format!(
            r#"o.bind("SUPER + mouse_down", "Scroll active workspace forward", hl.dsp.focus({{ workspace = "{wrap_next}" }}))"#
        ));
        lines.push(format!(
            r#"o.bind("SUPER + mouse_up", "Scroll active workspace backward", hl.dsp.focus({{ workspace = "{wrap_prev}" }}))"#
        ));
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
    for m in re!(r"hl\.workspace_rule\(\s*\{([^}]*)\}\s*\)").captures_iter(body) {
        let chunk = &m[1];
        let id = lua_str_field(chunk, "workspace");
        if id.is_empty() {
            continue;
        }
        out.push(json!({
            "id": id,
            "name": lua_str_field(chunk, "default_name"),
            "monitor": lua_str_field(chunk, "monitor"),
            "persistent": re!(r"persistent\s*=\s*true").is_match(chunk),
            "isDefault": re!(r"default\s*=\s*true").is_match(chunk),
            "onCreatedEmpty": lua_str_field(chunk, "on_created_empty"),
            "special": id.starts_with("special:"),
        }));
    }
    Value::Array(out)
}
