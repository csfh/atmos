//! The `atmos:autostart` block in autostart.lua.

use super::lua::*;
use serde_json::{json, Map, Value};

pub const BEGIN: &str = "-- atmos:autostart begin";
pub const END: &str = "-- atmos:autostart end";

pub fn sanitize_command(raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() || text.chars().count() > 256 || text.contains('\n') || text.contains('\r') {
        String::new()
    } else {
        text.to_string()
    }
}

fn clamp_delay(n: i64) -> i64 {
    n.clamp(0, 600)
}

/// `sleep N && cmd` -> (cmd, N)
pub fn split_delay(raw: &str) -> (String, i64) {
    let text = raw.trim();
    match re!(r"^sleep\s+(\d+)\s+&&\s+(.+)$").captures(text) {
        Some(m) => (
            sanitize_command(&m[2]),
            clamp_delay(m[1].parse::<i64>().unwrap_or(i64::MAX)),
        ),
        None => (sanitize_command(text), 0),
    }
}

fn join_delay(command: &str, delay: i64) -> String {
    let cmd = sanitize_command(command);
    let n = clamp_delay(delay);
    if cmd.is_empty() {
        String::new()
    } else if n > 0 {
        format!("sleep {n} && {cmd}")
    } else {
        cmd
    }
}

struct Call {
    command: String,
    delay: i64,
}

fn launch_calls(text: &str, disabled: bool) -> Vec<Call> {
    let mut out = Vec::new();
    let mut i = 0;
    let mark = "o.launch_on_start(";
    while let Some(rel) = text[i..].find(mark) {
        let at = i + rel;
        let Some((args, end)) = parse_call_args(text, at + mark.len()) else {
            i = at + mark.len();
            continue;
        };
        i = end;
        if in_line_comment(text, at) != disabled {
            continue;
        }
        let Some(raw) = args.first().and_then(Value::as_str) else {
            continue;
        };
        let (command, delay) = split_delay(raw);
        if !command.is_empty() {
            out.push(Call { command, delay });
        }
    }
    out
}

pub fn parse(text: &str) -> Value {
    let row = |c: &Call, enabled: bool, managed: bool| json!({"command": c.command, "delay": c.delay, "enabled": enabled, "managed": managed});
    let mut items = Vec::new();
    match sentinel_bounds(text, BEGIN, END) {
        Some((start, stop)) => {
            let outside = format!("{}\n{}", &text[..start], &text[stop..]);
            items.extend(
                launch_calls(&outside, false)
                    .iter()
                    .map(|c| row(c, true, false)),
            );
            items.extend(
                launch_calls(&text[start..stop], false)
                    .iter()
                    .map(|c| row(c, true, true)),
            );
            items.extend(
                launch_calls(&text[start..stop], true)
                    .iter()
                    .map(|c| row(c, false, true)),
            );
        }
        None => items.extend(
            launch_calls(text, false)
                .iter()
                .map(|c| row(c, true, false)),
        ),
    }
    Value::Array(items)
}

fn normalize(item: &Value) -> Option<(String, i64, bool)> {
    match item {
        Value::String(s) => {
            let (command, delay) = split_delay(s);
            (!command.is_empty()).then_some((command, delay, true))
        }
        Value::Object(map) => {
            let command = sanitize_command(&str_or_empty(map.get("command")));
            if command.is_empty() {
                return None;
            }
            let delay = map
                .get("delay")
                .filter(|v| truthy(v))
                .and_then(|v| match v {
                    Value::Number(n) => n.as_f64().map(|f| f.trunc() as i64),
                    Value::String(s) => s.trim().parse::<i64>().ok(),
                    Value::Bool(b) => Some(i64::from(*b)),
                    _ => None,
                })
                .unwrap_or(0);
            Some((
                command,
                clamp_delay(delay),
                map.get("enabled") != Some(&Value::Bool(false)),
            ))
        }
        _ => None,
    }
}

pub fn serialize(src: &Map<String, Value>) -> String {
    let commands = src
        .get("items")
        .and_then(Value::as_array)
        .or_else(|| src.get("commands").and_then(Value::as_array));
    let mut lines = vec![BEGIN.to_string()];
    for item in commands.into_iter().flatten() {
        let Some((command, delay, enabled)) = normalize(item) else {
            continue;
        };
        let cmd = join_delay(&command, delay);
        if cmd.is_empty() {
            continue;
        }
        let line = format!("o.launch_on_start({})", lua_string(&cmd));
        lines.push(if enabled { line } else { format!("-- {line}") });
    }
    lines.push(END.into());
    lines.join("\n")
}
