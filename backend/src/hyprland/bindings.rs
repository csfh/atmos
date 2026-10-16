//! The `atmos:bindings` block in bindings.lua.

use super::lua::*;
use serde_json::{json, Map, Value};

pub const BEGIN: &str = "-- atmos:bindings begin";
pub const END: &str = "-- atmos:bindings end";

fn sanitize_command(raw: &str) -> String {
    super::autostart::sanitize_command(raw)
}

pub fn sanitize_keys(raw: &str) -> String {
    if raw.contains('\n') || raw.contains('\r') {
        return String::new();
    }
    let text = re!(r"\s+").replace_all(raw.trim(), " ").to_string();
    if text.is_empty() || text.chars().count() > 64 || !re!(r"^[A-Za-z0-9_ +.:-]+$").is_match(&text)
    {
        String::new()
    } else {
        text
    }
}

fn sanitize_label(raw: &str) -> String {
    let text = raw.trim();
    if raw.contains('\n') || raw.contains('\r') || text.is_empty() || text.chars().count() > 64 {
        String::new()
    } else {
        text.to_string()
    }
}

fn lua_stringish(value: &Value) -> String {
    match value {
        Value::Bool(true) => "true".into(),
        Value::Bool(false) | Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => str_or_empty(Some(other)),
    }
}

fn shell_quote(value: &Value) -> String {
    format!("'{}'", lua_stringish(value).replace('\'', "'\\''"))
}

fn command_from_arg(arg: Option<&Value>, description: &Value) -> String {
    match arg {
        Some(Value::String(s)) => sanitize_command(s),
        Some(Value::Object(map)) => {
            let get = |k: &str| map.get(k).filter(|v| truthy(v));
            if let Some(v) = get("omarchy") {
                return sanitize_command(&format!("omarchy-launch-{}", lua_stringish(v)));
            }
            let (launch, focus) = (get("launch"), get("focus"));
            if let (Some(launch), Some(focus)) = (launch, focus) {
                let wrapped = Value::String(format!("uwsm-app -- {}", lua_stringish(launch)));
                return sanitize_command(&format!(
                    "omarchy-launch-or-focus {} {}",
                    shell_quote(focus),
                    shell_quote(&wrapped)
                ));
            }
            if let Some(launch) = launch {
                return sanitize_command(&format!("uwsm-app -- {}", lua_stringish(launch)));
            }
            if let Some(webapp) = get("webapp") {
                if focus.is_some() {
                    return sanitize_command(&format!(
                        "omarchy-launch-or-focus-webapp {} {}",
                        shell_quote(description),
                        shell_quote(webapp)
                    ));
                }
                return sanitize_command(&format!("omarchy-launch-webapp {}", shell_quote(webapp)));
            }
            if let Some(tui) = get("tui") {
                let verb = if focus.is_some() {
                    "omarchy-launch-or-focus-tui"
                } else {
                    "omarchy-launch-tui"
                };
                return sanitize_command(&format!("{verb} {}", shell_quote(tui)));
            }
            String::new()
        }
        _ => String::new(),
    }
}

enum Event {
    Unbind(String),
    Bind {
        keys: String,
        label: String,
        command: String,
    },
}

fn events(src: &str) -> Vec<Event> {
    let mut out = Vec::new();
    // `o.bind(` is listed first so a same-index tie stays a bind.
    earliest_live_calls(src, &["o.bind(", "hl.unbind("], |mark, args| {
        let keys = sanitize_keys(&str_or_empty(args.first()));
        if keys.is_empty() {
            return;
        }
        if mark == "hl.unbind(" {
            out.push(Event::Unbind(keys));
            return;
        }
        let raw_label = match args.get(1) {
            Some(v) if !v.is_null() => v.clone(),
            _ => Value::String(String::new()),
        };
        let label = match args.get(1) {
            Some(v) if !v.is_null() => sanitize_label(&str_or_empty(Some(v))),
            _ => String::new(),
        };
        let command = command_from_arg(args.get(2), &raw_label);
        out.push(Event::Bind {
            keys,
            label,
            command,
        });
    });
    out
}

fn fold(events: Vec<Event>) -> Vec<Value> {
    let mut out = Vec::new();
    let mut iter = events.into_iter().peekable();
    while let Some(event) = iter.next() {
        match event {
            Event::Unbind(keys) => {
                if let Some(Event::Bind { keys: next, .. }) = iter.peek() {
                    if *next == keys {
                        if let Some(Event::Bind { label, command, .. }) = iter.next() {
                            out.push(json!({"keys": keys, "label": label, "command": command, "unbind": true}));
                            continue;
                        }
                    }
                }
                out.push(json!({"keys": keys, "label": "", "command": "", "unbind": true}));
            }
            Event::Bind {
                keys,
                label,
                command,
            } => {
                out.push(
                    json!({"keys": keys, "label": label, "command": command, "unbind": false}),
                );
            }
        }
    }
    out
}

fn with_managed(rows: Vec<Value>, managed: bool) -> impl Iterator<Item = Value> {
    rows.into_iter().map(move |mut row| {
        row["managed"] = json!(managed);
        row
    })
}

pub fn parse(text: &str) -> Value {
    let mut items: Vec<Value> = Vec::new();
    match split_sentinel(text, BEGIN, END) {
        Some((inside, outside)) => {
            items.extend(with_managed(fold(events(&outside)), false));
            items.extend(with_managed(fold(events(inside)), true));
        }
        None => items.extend(with_managed(fold(events(text)), false)),
    }
    Value::Array(items)
}

fn normalize(row: &Value) -> Option<(String, String, String, bool)> {
    let map = row.as_object()?;
    let keys = sanitize_keys(&str_or_empty(map.get("keys")));
    if keys.is_empty() {
        return None;
    }
    let label = sanitize_label(&str_or_empty(map.get("label")));
    let command = sanitize_command(&str_or_empty(map.get("command")));
    let unbind = map.get("unbind") == Some(&Value::Bool(true));
    if command.is_empty() && !unbind {
        return None;
    }
    Some((keys, label, command, unbind))
}

pub fn serialize(src: &Map<String, Value>) -> String {
    let mut lines = vec![BEGIN.to_string()];
    for item in src
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some((keys, label, command, unbind)) = normalize(item) else {
            continue;
        };
        if unbind {
            lines.push(format!("hl.unbind({})", lua_string(&keys)));
        }
        if !command.is_empty() {
            let label_arg = if label.is_empty() {
                "nil".to_string()
            } else {
                lua_string(&label)
            };
            lines.push(format!(
                "o.bind({}, {label_arg}, {})",
                lua_string(&keys),
                lua_string(&command)
            ));
        }
    }
    lines.push(END.into());
    lines.join("\n")
}
