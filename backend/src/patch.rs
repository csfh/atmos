//! In-place edits of the documents Omarchy already reads.
//!
//! shell.json keeps `bar.position` and the bar widgets. Hyprland sentinels keep
//! `hl.config` assignments such as `gaps_in`. Line files such as pacman.conf keep
//! every other line and only replace the matching assignment.

use serde_json::{Map, Value};

use crate::domain::{self, LuaForm, Ty};

pub fn shell_get(doc: &Value, key: &str) -> Value {
    match slot(key) {
        Slot::Path(path) => get_path(doc, path),
        Slot::Widget { id, field } => widget_field(doc, id, field),
        Slot::Clock { alt } => {
            let field = clock_field(doc, alt);
            let value = widget_field(doc, "omarchy.clock", field);
            if value.is_null() {
                widget_field(doc, "omarchy.clock", clock_field_other(alt, field))
            } else {
                value
            }
        }
        Slot::Sync => match widget_field(doc, "omarchy.agents", "syncMode") {
            Value::Bool(on) => Value::Bool(on),
            Value::String(text) => Value::Bool(text == "On" || text == "true"),
            _ => Value::Null,
        },
        Slot::Extra => get_path(doc, &["atmos", "extra"]),
    }
}

pub fn shell_set(doc: &mut Value, key: &str, value: &Value) {
    if !doc.is_object() {
        *doc = Value::Object(Map::new());
    }
    match slot(key) {
        Slot::Path(path) => set_path(doc, path, value.clone()),
        Slot::Widget { id, field } => set_widget(doc, id, field, value.clone()),
        Slot::Clock { alt } => {
            let field = clock_field(doc, alt).to_string();
            set_widget(doc, "omarchy.clock", &field, value.clone());
        }
        Slot::Sync => {
            let text = if value.as_bool() == Some(true) {
                "On"
            } else {
                "Off"
            };
            set_widget(
                doc,
                "omarchy.agents",
                "syncMode",
                Value::String(text.into()),
            );
        }
        Slot::Extra => set_path(doc, &["atmos", "extra"], value.clone()),
    }
}

pub fn nested_get(doc: &Value, path: &[&str]) -> Value {
    get_path(doc, path)
}

pub fn nested_set(doc: &mut Value, path: &[&str], value: &Value) {
    if !doc.is_object() {
        *doc = Value::Object(Map::new());
    }
    set_path(doc, path, value.clone());
}

pub fn lua_read(text: &str, begin: &str, end: &str, key: &str, ty: Ty) -> Result<Value, String> {
    let Some(bind) = domain::lua_bind(key) else {
        return Ok(Value::Null);
    };
    let Some(body) = sentinel_body(text, begin, end) else {
        return Ok(Value::Null);
    };
    let value = match bind.form {
        LuaForm::Value => assign_value(&body, bind.name),
        LuaForm::Enabled => enabled_value(&body, bind.name),
        LuaForm::Cursor => cursor_value(&body),
        LuaForm::Gesture => Value::Bool(
            body.contains("action = \"workspace\"") || body.contains("action=\"workspace\""),
        ),
        LuaForm::KbToggle => Value::Bool(body.contains("grp:alts_toggle")),
    };
    coerce(value, ty)
}

pub fn lua_write(
    text: &str,
    begin: &str,
    end: &str,
    key: &str,
    value: &Value,
) -> Result<String, String> {
    let bind = domain::lua_bind(key).ok_or_else(|| format!("no lua field for {key}"))?;
    let mut text = ensure_sentinel(text, begin, end);
    let rendered = lua_token(&bind.form, value);
    let replaced = match bind.form {
        LuaForm::Value | LuaForm::KbToggle => {
            replace_assign(&text, begin, end, bind.name, &rendered)
        }
        LuaForm::Enabled => {
            replace_enabled(&text, begin, end, bind.name, value.as_bool() == Some(true))
        }
        LuaForm::Cursor => replace_cursor(&text, begin, end, &rendered),
        LuaForm::Gesture => replace_gesture(&text, begin, end, value.as_bool() == Some(true)),
    };
    if let Some(next) = replaced {
        return Ok(next);
    }
    let line = match bind.form {
        LuaForm::Enabled => format!("    {} = {{ enabled = {} }},", bind.name, rendered),
        LuaForm::Cursor => format!(
            "    hl.env(\"HYPRCURSOR_SIZE\", \"{rendered}\")\n    hl.env(\"XCURSOR_SIZE\", \"{rendered}\")"
        ),
        LuaForm::Gesture => "    hl.gesture({ direction = \"horizontal\", fingers = 3, action = \"workspace\" })".into(),
        LuaForm::KbToggle => format!("    {} = {},", bind.name, rendered),
        LuaForm::Value => format!("    {} = {},", bind.name, rendered),
    };
    text = insert_in_sentinel(&text, begin, end, &line);
    Ok(text)
}

pub fn line_read(text: &str, prefix: &str, ty: Ty) -> Result<Value, String> {
    if prefix.is_empty() {
        let text = text.trim_end_matches(['\n', '\r']);
        return coerce(Value::String(text.to_string()), ty);
    }
    let key = assignment_key(prefix);
    for line in text.lines() {
        if let Some(raw) = assignment_value(line, key) {
            return coerce(Value::String(raw), ty);
        }
    }
    Ok(Value::Null)
}

pub fn line_write(text: &str, prefix: &str, value: &Value) -> String {
    let rendered = match value {
        Value::String(text) => text.clone(),
        Value::Bool(true) => "true".into(),
        Value::Bool(false) => "false".into(),
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    };
    if prefix.is_empty() {
        return format!("{rendered}\n");
    }
    let key = assignment_key(prefix);
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut found = false;
    for line in &mut lines {
        if assignment_value(line, key).is_some() {
            *line = format!("{prefix}{rendered}");
            found = true;
            break;
        }
    }
    if !found {
        if lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }
        lines.push(format!("{prefix}{rendered}"));
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

enum Slot {
    Path(&'static [&'static str]),
    Widget {
        id: &'static str,
        field: &'static str,
    },
    Clock {
        alt: bool,
    },
    Sync,
    /// Unmapped shell keys stay under `atmos`, never as a sibling of `bar`.
    Extra,
}

fn slot(key: &str) -> Slot {
    match key {
        "barPosition" => Slot::Path(&["bar", "position"]),
        "barTransparent" => Slot::Path(&["bar", "transparent"]),
        "barVisible" => Slot::Path(&["bar", "visible"]),
        "idleScreensaver" => Slot::Path(&["idle", "screensaver"]),
        "idleLock" => Slot::Path(&["idle", "lock"]),
        "stayAwake" => Slot::Path(&["idle", "stayAwake"]),
        "screensaverEnabled" => Slot::Path(&["idle", "screensaverEnabled"]),
        "clockFormat" => Slot::Clock { alt: false },
        "clockFormatAlt" => Slot::Clock { alt: true },
        "clockWeekStart" => Slot::Widget {
            id: "omarchy.clock",
            field: "weekStartDay",
        },
        "clockBirthYear" => Slot::Widget {
            id: "omarchy.clock",
            field: "birthYear",
        },
        "clockLifeExpectancy" => Slot::Widget {
            id: "omarchy.clock",
            field: "lifeExpectancy",
        },
        "indicatorsAlwaysShow" => Slot::Widget {
            id: "omarchy.indicators",
            field: "alwaysShow",
        },
        "indicatorsItems" => Slot::Widget {
            id: "omarchy.indicators",
            field: "items",
        },
        "powerShowPercentage" => Slot::Widget {
            id: "omarchy.power",
            field: "showPercentage",
        },
        "spacerSize" => Slot::Widget {
            id: "omarchy.spacer",
            field: "size",
        },
        "weatherLocation" => Slot::Widget {
            id: "omarchy.weather",
            field: "location",
        },
        "weatherUnit" => Slot::Widget {
            id: "omarchy.weather",
            field: "unit",
        },
        "weatherRefreshMinutes" => Slot::Widget {
            id: "omarchy.weather",
            field: "refreshMinutes",
        },
        "agentsRefreshIntervalSec" => Slot::Widget {
            id: "omarchy.agents",
            field: "refreshIntervalSec",
        },
        "agentsSync" => Slot::Sync,
        "agentsSyncDir" => Slot::Widget {
            id: "omarchy.agents",
            field: "syncDir",
        },
        "agentsSyncFileName" => Slot::Widget {
            id: "omarchy.agents",
            field: "syncFileName",
        },
        "agentsSyncDeviceId" => Slot::Widget {
            id: "omarchy.agents",
            field: "syncDeviceId",
        },
        "trayHidden" => Slot::Widget {
            id: "omarchy.tray",
            field: "hidden",
        },
        "trayPinned" => Slot::Widget {
            id: "omarchy.tray",
            field: "pinned",
        },
        "workspaceBarNames" => Slot::Widget {
            id: "omarchy.workspaces",
            field: "showNames",
        },
        "workspaceBarCount" => Slot::Widget {
            id: "omarchy.workspaces",
            field: "count",
        },
        _ => Slot::Extra,
    }
}

fn clock_field(doc: &Value, alt: bool) -> &'static str {
    let side = doc
        .get("bar")
        .and_then(|bar| bar.get("position"))
        .and_then(Value::as_str);
    let vertical = matches!(side, Some("left") | Some("right"));
    match (alt, vertical) {
        (false, false) => "format",
        (false, true) => "verticalFormat",
        (true, false) => "formatAlt",
        (true, true) => "verticalFormatAlt",
    }
}

fn clock_field_other(alt: bool, field: &str) -> &'static str {
    if alt {
        if field == "verticalFormatAlt" {
            "formatAlt"
        } else {
            "verticalFormatAlt"
        }
    } else if field == "verticalFormat" {
        "format"
    } else {
        "verticalFormat"
    }
}

fn get_path(doc: &Value, path: &[&str]) -> Value {
    let mut cur = doc;
    for key in path {
        match cur.get(*key) {
            Some(next) => cur = next,
            None => return Value::Null,
        }
    }
    cur.clone()
}

fn set_path(doc: &mut Value, path: &[&str], value: Value) {
    if path.is_empty() {
        return;
    }
    let mut cur = doc;
    for (index, key) in path.iter().enumerate() {
        let map = match cur {
            Value::Object(map) => map,
            other => {
                *other = Value::Object(Map::new());
                match other {
                    Value::Object(map) => map,
                    _ => return,
                }
            }
        };
        if index + 1 == path.len() {
            map.insert((*key).to_string(), value);
            return;
        }
        if !map.get(*key).is_some_and(Value::is_object) {
            map.insert((*key).to_string(), Value::Object(Map::new()));
        }
        cur = map.get_mut(*key).unwrap();
    }
}

fn widget_matches(id: &str, canonical: &str) -> bool {
    if id == canonical {
        return true;
    }
    canonical
        .strip_prefix("omarchy")
        .is_some_and(|suffix| id.ends_with(suffix))
}

fn widget_field(doc: &Value, canonical: &str, field: &str) -> Value {
    let Some(item) = find_widget(doc, canonical) else {
        return Value::Null;
    };
    item.get(field).cloned().unwrap_or(Value::Null)
}

fn find_widget<'a>(doc: &'a Value, canonical: &str) -> Option<&'a Map<String, Value>> {
    let layout = doc.get("bar")?.get("layout")?;
    for side in ["left", "center", "right"] {
        let Some(items) = layout.get(side).and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            let Some(map) = item.as_object() else {
                continue;
            };
            let Some(id) = map.get("id").and_then(Value::as_str) else {
                continue;
            };
            if widget_matches(id, canonical) {
                return Some(map);
            }
        }
    }
    None
}

fn set_widget(doc: &mut Value, canonical: &str, field: &str, value: Value) {
    ensure_layout(doc);
    if let Some((side, index)) = find_widget_index(doc, canonical) {
        if let Some(map) = widget_mut(doc, side, index) {
            map.insert(field.to_string(), value);
        }
        return;
    }
    let side = if canonical.ends_with(".workspaces") {
        "left"
    } else {
        "center"
    };
    let mut item = Map::new();
    item.insert("id".into(), Value::String(canonical.into()));
    item.insert(field.to_string(), value);
    if let Some(items) = layout_array_mut(doc, side) {
        items.push(Value::Object(item));
    }
}

fn ensure_layout(doc: &mut Value) {
    set_path(
        doc,
        &["bar", "layout", "left"],
        existing_or_array(doc, &["bar", "layout", "left"]),
    );
    set_path(
        doc,
        &["bar", "layout", "center"],
        existing_or_array(doc, &["bar", "layout", "center"]),
    );
    set_path(
        doc,
        &["bar", "layout", "right"],
        existing_or_array(doc, &["bar", "layout", "right"]),
    );
}

fn existing_or_array(doc: &Value, path: &[&str]) -> Value {
    let current = get_path(doc, path);
    if current.is_array() {
        current
    } else {
        Value::Array(Vec::new())
    }
}

fn find_widget_index(doc: &Value, canonical: &str) -> Option<(&'static str, usize)> {
    let layout = doc.get("bar")?.get("layout")?;
    for side in ["left", "center", "right"] {
        let Some(items) = layout.get(side).and_then(Value::as_array) else {
            continue;
        };
        for (index, item) in items.iter().enumerate() {
            let Some(id) = item.get("id").and_then(Value::as_str) else {
                continue;
            };
            if widget_matches(id, canonical) {
                return Some((side, index));
            }
        }
    }
    None
}

fn widget_mut<'a>(
    doc: &'a mut Value,
    side: &str,
    index: usize,
) -> Option<&'a mut Map<String, Value>> {
    layout_array_mut(doc, side)?.get_mut(index)?.as_object_mut()
}

fn layout_array_mut<'a>(doc: &'a mut Value, side: &str) -> Option<&'a mut Vec<Value>> {
    doc.get_mut("bar")?
        .get_mut("layout")?
        .get_mut(side)?
        .as_array_mut()
}

fn assignment_key(prefix: &str) -> &str {
    prefix.trim().trim_end_matches('=').trim()
}

fn assignment_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim().trim_start_matches('#').trim();
    let rest = trimmed.strip_prefix(key)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?;
    Some(rest.trim().to_string())
}

fn coerce(value: Value, ty: Ty) -> Result<Value, String> {
    if value.is_null() {
        return Ok(value);
    }
    let text = match &value {
        Value::String(text) => text.clone(),
        other => {
            return if ty.accepts(other) {
                Ok(other.clone())
            } else {
                Ok(Value::Null)
            }
        }
    };
    if text.is_empty() && !matches!(ty, Ty::String) {
        return Ok(Value::Null);
    }
    match ty {
        Ty::String => Ok(Value::String(text)),
        Ty::Bool => match text.as_str() {
            "true" | "1" | "on" | "yes" => Ok(Value::Bool(true)),
            "false" | "0" | "off" | "no" => Ok(Value::Bool(false)),
            _ => Ok(Value::Null),
        },
        Ty::Int => {
            let number: i64 = text
                .parse()
                .map_err(|_| format!("expected an integer, got {text}"))?;
            Ok(Value::from(number))
        }
        Ty::Number => {
            let number: serde_json::Number = text
                .parse()
                .map_err(|_| format!("expected a number, got {text}"))?;
            Ok(Value::Number(number))
        }
        Ty::List => Ok(Value::Null),
    }
}

fn sentinel_body<'a>(text: &'a str, begin: &str, end: &str) -> Option<&'a str> {
    let start = line_pos(text, begin)?;
    let after = text[start..]
        .find('\n')
        .map(|at| start + at + 1)
        .unwrap_or(text.len());
    let end_at = line_pos(&text[after..], end)? + after;
    Some(&text[after..end_at])
}

fn line_pos(text: &str, marker: &str) -> Option<usize> {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if line.trim().trim_end_matches(['\n', '\r']) == marker {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn ensure_sentinel(text: &str, begin: &str, end: &str) -> String {
    if line_pos(text, begin).is_some() {
        return text.to_string();
    }
    let mut out = text.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(begin);
    out.push_str("\nhl.config({\n})\n");
    out.push_str(end);
    out.push('\n');
    out
}

fn insert_in_sentinel(text: &str, begin: &str, end: &str, line: &str) -> String {
    let Some(body_start) = sentinel_body_start(text, begin) else {
        return text.to_string();
    };
    let Some(end_at) = line_pos(&text[body_start..], end).map(|at| body_start + at) else {
        return text.to_string();
    };
    let body = &text[body_start..end_at];
    let insert_at = body.rfind("})").map(|at| body_start + at).unwrap_or(end_at);
    let mut out = String::new();
    out.push_str(&text[..insert_at]);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(line);
    out.push('\n');
    out.push_str(&text[insert_at..]);
    out
}

fn sentinel_body_start(text: &str, begin: &str) -> Option<usize> {
    let start = line_pos(text, begin)?;
    Some(
        text[start..]
            .find('\n')
            .map(|at| start + at + 1)
            .unwrap_or(text.len()),
    )
}

fn replace_assign(
    text: &str,
    begin: &str,
    end: &str,
    name: &str,
    rendered: &str,
) -> Option<String> {
    let (body_start, body_end) = body_range(text, begin, end)?;
    let body = &text[body_start..body_end];
    let (value_start, value_end) = find_assign(body, name)?;
    let mut out = String::new();
    out.push_str(&text[..body_start + value_start]);
    out.push_str(rendered);
    out.push_str(&text[body_start + value_end..]);
    Some(out)
}

fn find_assign(body: &str, name: &str) -> Option<(usize, usize)> {
    let mut from = 0;
    while let Some(at) = body[from..].find(name) {
        let abs = from + at;
        if !ident_boundary(body, abs) || comment_line(body, abs) {
            from = abs + name.len();
            continue;
        }
        let after = abs + name.len();
        let rest = body[after..].trim_start();
        if !rest.starts_with('=') {
            from = after;
            continue;
        }
        let eq = body[after..].find('=')? + after;
        let mut value_at = eq + 1;
        while body
            .as_bytes()
            .get(value_at)
            .is_some_and(u8::is_ascii_whitespace)
        {
            value_at += 1;
        }
        return Some((value_at, end_of_lua_value(body, value_at)));
    }
    None
}

fn ident_boundary(text: &str, at: usize) -> bool {
    at == 0 || !text.as_bytes()[at - 1].is_ascii_alphanumeric() && text.as_bytes()[at - 1] != b'_'
}

fn comment_line(text: &str, at: usize) -> bool {
    let line_start = text[..at].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
    text[line_start..at].trim_start().starts_with("--")
}

fn end_of_lua_value(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    if start >= bytes.len() {
        return start;
    }
    if bytes[start] == b'"' {
        let mut index = start + 1;
        while index < bytes.len() {
            if bytes[index] == b'\\' {
                index += 2;
                continue;
            }
            if bytes[index] == b'"' {
                return index + 1;
            }
            index += 1;
        }
        return bytes.len();
    }
    let mut index = start;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-' || byte == b'_' {
            index += 1;
            continue;
        }
        break;
    }
    index
}

fn replace_enabled(text: &str, begin: &str, end: &str, name: &str, on: bool) -> Option<String> {
    let (body_start, body_end) = body_range(text, begin, end)?;
    let body = &text[body_start..body_end];
    let mut from = 0;
    while let Some(at) = body[from..].find(name) {
        let abs = from + at;
        if ident_boundary(body, abs) && !comment_line(body, abs) {
            let after = abs + name.len();
            let window = &body[after..body.len().min(after + 80)];
            if let Some(rel) = window.find("enabled") {
                let enabled_at = after + rel;
                if let Some((value_start, value_end)) = find_assign(&body[enabled_at..], "enabled")
                {
                    let mut out = String::new();
                    out.push_str(&text[..body_start + enabled_at + value_start]);
                    out.push_str(if on { "true" } else { "false" });
                    out.push_str(&text[body_start + enabled_at + value_end..]);
                    return Some(out);
                }
            }
        }
        from = abs + name.len();
    }
    None
}

fn replace_cursor(text: &str, begin: &str, end: &str, rendered: &str) -> Option<String> {
    let mut next = text.to_string();
    let mut found = false;
    for name in ["HYPRCURSOR_SIZE", "XCURSOR_SIZE"] {
        let (body_start, body_end) = body_range(&next, begin, end)?;
        let body = &next[body_start..body_end];
        let Some(at) = body.find(name) else { continue };
        let after_rel = at + name.len();
        let after = &body[after_rel..];
        let Some(close_name) = after.find('"') else {
            continue;
        };
        let rest_at = after_rel + close_name + 1;
        let rest = &body[rest_at..];
        let Some(open_value) = rest.find('"') else {
            continue;
        };
        let start = body_start + rest_at + open_value + 1;
        let Some(end_rel) = next[start..].find('"') else {
            continue;
        };
        let stop = start + end_rel;
        let mut out = String::new();
        out.push_str(&next[..start]);
        out.push_str(rendered);
        out.push_str(&next[stop..]);
        next = out;
        found = true;
    }
    found.then_some(next)
}

fn replace_gesture(text: &str, begin: &str, end: &str, on: bool) -> Option<String> {
    let (body_start, body_end) = body_range(text, begin, end)?;
    let body = &text[body_start..body_end];
    let present = body.contains("action = \"workspace\"") || body.contains("action=\"workspace\"");
    if on && present {
        return Some(text.to_string());
    }
    if !on && present {
        let mut kept = Vec::new();
        for line in text.lines() {
            if line.contains("action = \"workspace\"") || line.contains("action=\"workspace\"") {
                let pos = text.find(line).unwrap_or(0);
                if pos >= body_start && pos < body_end {
                    continue;
                }
            }
            kept.push(line.to_string());
        }
        let mut out = kept.join("\n");
        if text.ends_with('\n') {
            out.push('\n');
        }
        return Some(out);
    }
    if on {
        None
    } else {
        Some(text.to_string())
    }
}

fn body_range(text: &str, begin: &str, end: &str) -> Option<(usize, usize)> {
    let start = sentinel_body_start(text, begin)?;
    let end_at = line_pos(&text[start..], end)? + start;
    Some((start, end_at))
}

fn lua_token(form: &LuaForm, value: &Value) -> String {
    match form {
        LuaForm::KbToggle => {
            if value.as_bool() == Some(true) {
                "\"grp:alts_toggle\"".into()
            } else {
                "\"\"".into()
            }
        }
        LuaForm::Cursor => match value {
            Value::Number(number) => number.to_string(),
            Value::String(text) => text.clone(),
            _ => "24".into(),
        },
        LuaForm::Enabled | LuaForm::Gesture => {
            if value.as_bool() == Some(true) {
                "true".into()
            } else {
                "false".into()
            }
        }
        LuaForm::Value => match value {
            Value::Bool(true) => "true".into(),
            Value::Bool(false) => "false".into(),
            Value::Number(number) => number.to_string(),
            Value::String(text) => {
                format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
            }
            other => other.to_string(),
        },
    }
}

fn assign_value(body: &str, name: &str) -> Value {
    let Some((start, end)) = find_assign(body, name) else {
        return Value::Null;
    };
    parse_lua_scalar(&body[start..end])
}

fn parse_lua_scalar(token: &str) -> Value {
    match token {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "1" => Value::from(1),
        "0" => Value::from(0),
        _ => {
            if let Some(text) = token
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
            {
                return Value::String(text.replace("\\\"", "\"").replace("\\\\", "\\"));
            }
            if let Ok(number) = token.parse::<i64>() {
                return Value::from(number);
            }
            if let Ok(number) = token.parse::<serde_json::Number>() {
                return Value::Number(number);
            }
            Value::Null
        }
    }
}

fn enabled_value(body: &str, name: &str) -> Value {
    let mut from = 0;
    while let Some(at) = body[from..].find(name) {
        let abs = from + at;
        if ident_boundary(body, abs) && !comment_line(body, abs) {
            let after = abs + name.len();
            let window = &body[after..body.len().min(after + 80)];
            if let Some(rel) = window.find("enabled") {
                if let Some((start, end)) = find_assign(&body[after + rel..], "enabled") {
                    return parse_lua_scalar(&body[after + rel + start..after + rel + end]);
                }
            }
        }
        from = abs + name.len();
    }
    Value::Null
}

fn cursor_value(body: &str) -> Value {
    for name in ["HYPRCURSOR_SIZE", "XCURSOR_SIZE"] {
        let Some(at) = body.find(name) else { continue };
        let after = &body[at + name.len()..];
        let Some(close_name) = after.find('"') else {
            continue;
        };
        let rest = &after[close_name + 1..];
        let Some(open_value) = rest.find('"') else {
            continue;
        };
        let start = open_value + 1;
        let Some(end) = rest[start..].find('"') else {
            continue;
        };
        return parse_lua_scalar(&rest[start..start + end]);
    }
    Value::Null
}
