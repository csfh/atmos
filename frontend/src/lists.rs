//! A list setting Atmos manages part of (`autostart`, `windowRules`): what a write of the managed
//! part sends, as `Settings.commandFor` makes it, with the lines the user wrote kept
//! (`mergeUnmanaged`).

use serde_json::Value;

use crate::settings;

/// `mergeUnmanaged(current, next, key)`: the user's items, then these, each marked managed; a
/// text is a command for `autostart`, and kept as it is otherwise.
pub fn merge(current: Option<&Value>, next: Vec<Value>, key: &str) -> Value {
    let mut out: Vec<Value> = current
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|r| r.get("managed") == Some(&Value::Bool(false)))
        .cloned()
        .collect();
    for item in next {
        match item {
            Value::Null => {}
            Value::String(text) if key == "autostart" => {
                out.push(serde_json::json!({ "command": text, "managed": true }));
            }
            Value::String(text) => out.push(Value::String(text)),
            Value::Object(mut row) => {
                row.insert("managed".into(), Value::Bool(true));
                out.push(Value::Object(row));
            }
            _ => {}
        }
    }
    Value::Array(out)
}

/// The `settings.set` of `key`'s managed items `next`, over the snapshot's `current`.
pub fn write(key: &str, current: Option<&Value>, next: Vec<Value>) -> Value {
    settings::write(key, &merge(current, next, key))
}
