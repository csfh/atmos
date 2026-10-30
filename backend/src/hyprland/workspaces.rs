//! The `atmos:workspaces` block in atmos.lua.

use super::lua::*;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

pub const BEGIN: &str = "-- atmos:workspaces begin";
pub const END: &str = "-- atmos:workspaces end";

fn serialize_created_empty(id: &str, cmd: &str) -> String {
    let is_rule = cmd.starts_with('[');
    let exec_call = if is_rule {
        format!("hl.exec_cmd({})", lua_string(cmd))
    } else {
        format!(
            "hl.exec_cmd({}, {{ workspace = {} }})",
            lua_string(cmd),
            lua_string(id)
        )
    };
    format!(
        "hl.on(\"workspace.created\", function(ws)\n  if ws and (ws.name == {id_lua} or tostring(ws.id) == {id_lua}) and ws.is_empty then\n    {exec_call}\n  end\nend)",
        id_lua = lua_string(id),
        exec_call = exec_call,
    )
}

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
        lines.push(format!("hl.workspace_rule({{ {} }})", parts.join(", ")));
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
            lines.push(serialize_created_empty(&wid, empty));
        }
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

fn workspace_row(arg: Option<&Value>) -> Option<Value> {
    let table = arg?.as_object()?;
    let id = string_field(table, "workspace");
    if id.is_empty() {
        return None;
    }
    let flag = |key: &str| table.get(key) == Some(&Value::Bool(true));
    Some(json!({
        "id": id,
        "name": string_field(table, "default_name"),
        "monitor": string_field(table, "monitor"),
        "persistent": flag("persistent"),
        "isDefault": flag("default"),
        "onCreatedEmpty": string_field(table, "on_created_empty"),
        "special": id.starts_with("special:"),
    }))
}

pub fn parse(text: &str) -> Value {
    let body = match sentinel_bounds(text, BEGIN, END) {
        Some((start, stop)) => &text[start..stop],
        None => "",
    };
    let mut on_created_empty_map = std::collections::HashMap::new();
    let re_handler = re!(
        r#"hl\.on\(\s*"workspace\.created"\s*,\s*function\s*\(\s*ws\s*\)[\s\S]*?(?:ws\.name|tostring\(ws\.id\))\s*==\s*"((?:\\.|[^"\\])*)"[\s\S]*?hl\.exec_cmd\(\s*"((?:\\.|[^"\\])*)"[\s\S]*?end\s*\)"#
    );
    for cap in re_handler.captures_iter(body) {
        on_created_empty_map.insert(cap[1].to_string(), cap[2].to_string());
    }
    let mut out = Vec::new();
    live_calls(body, "hl.workspace_rule(", |args| {
        if let Some(mut row) = workspace_row(args.first()) {
            if let Some(id) = row.get("id").and_then(Value::as_str) {
                if row
                    .get("onCreatedEmpty")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .is_empty()
                {
                    if let Some(empty) = on_created_empty_map.get(id) {
                        row["onCreatedEmpty"] = json!(empty);
                    }
                }
            }
            out.push(row);
        }
    });
    Value::Array(out)
}
