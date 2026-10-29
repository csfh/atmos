//! The Environment page's logic (`pages/system/EnvironmentPage.qml`, `services/EnvPrefs.js`):
//! what the session is detected to use, the variables and PATH prepend Atmos writes, and the
//! writes `Omarchy.setEnvVars` sends for them. `tests/catalog_custom.rs` runs the page's own code
//! under Node against this, and `tests/catalog_setters.rs` runs `setEnvVars`.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::js;
use crate::settings;

pub const DETECTED: Field = Field::Key("envDetected");
pub const VARS: Field = Field::Key("envVars");
pub const PREPEND: Field = Field::Key("envPathPrepend");

/// What Add says before anything went wrong, and when the name is not one.
pub const ADD: &str = "A name and a value. PATH is the prepend field above.";
pub const BAD_KEY: &str = "Use a shell variable name. PATH is set with PATH prepend.";

/// `detected(raw)`'s field `name`: its text on one line, at most `max` units.
pub fn detected(raw: Option<&Value>, name: &str, max: usize) -> String {
    let v = raw.filter(|r| r.is_object()).and_then(|r| r.get(name));
    let text = if js::truthy(v) {
        js::string(v.unwrap_or(&Value::Null))
    } else {
        String::new()
    };
    let mut out = String::new();
    let mut gap = false;
    for c in text.chars() {
        if matches!(c, '\r' | '\n') {
            gap = true;
            continue;
        }
        if gap {
            out.push(' ');
            gap = false;
        }
        out.push(c);
    }
    if gap {
        out.push(' ');
    }
    let mut units = 0;
    out.chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= max
        })
        .collect()
}

/// The Session row: its type and desktop.
pub fn session(raw: Option<&Value>) -> String {
    let kind = detected(raw, "sessionType", 32);
    let desktop = detected(raw, "desktop", 64);
    let kind = if kind.is_empty() {
        "unknown".to_owned()
    } else {
        kind
    };
    if desktop.is_empty() {
        kind
    } else {
        format!("{kind} · {desktop}")
    }
}

/// The Shell row.
pub fn shell(raw: Option<&Value>) -> String {
    let shell = detected(raw, "shell", 64);
    if shell.is_empty() {
        "not set".to_owned()
    } else {
        shell
    }
}

/// The Editor / terminal / browser row.
pub fn tools(raw: Option<&Value>) -> String {
    let set: Vec<String> = [("editor", 64), ("terminal", 64), ("browser", 64)]
        .iter()
        .map(|(n, max)| detected(raw, n, *max))
        .filter(|t| !t.is_empty())
        .collect();
    if set.is_empty() {
        "not set".to_owned()
    } else {
        set.join(" · ")
    }
}

/// The PATH row's description.
pub fn path(raw: Option<&Value>) -> String {
    let path = detected(raw, "path", 1024);
    if path.is_empty() {
        "not set".to_owned()
    } else {
        path
    }
}

/// `sanitizeKey`: a shell variable name that is not PATH, else "".
pub fn sanitize_key(raw: &str) -> String {
    let text = raw.trim();
    let mut chars = text.chars();
    let first = chars.next();
    let ok = first.is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && text.len() <= 64
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if ok && text != "PATH" {
        text.to_owned()
    } else {
        String::new()
    }
}

/// `sanitizeValue`: one line of at most 512 units, else "".
pub fn sanitize_value(raw: &str) -> String {
    if raw.contains(['\n', '\r', '\0']) || raw.encode_utf16().count() > 512 {
        String::new()
    } else {
        raw.to_owned()
    }
}

/// `addVar`: the variables with this one set, or what is wrong with its name.
pub fn add_var(vars: &Value, key: &str, value: &str) -> Result<Vec<Value>, &'static str> {
    let key = sanitize_key(key);
    if key.is_empty() {
        return Err(BAD_KEY);
    }
    let mut out: Vec<Value> = vars
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v.get("key").and_then(Value::as_str) != Some(key.as_str()))
        .cloned()
        .collect();
    out.push(json!({ "key": key, "value": sanitize_value(value) }));
    Ok(out)
}

/// Remove: the variables, less any whose key is the row's (`list[i] && list[i].key !==
/// modelData.key`).
pub fn remove_var(vars: &Value, key: Option<&Value>) -> Vec<Value> {
    vars.as_array()
        .into_iter()
        .flatten()
        .filter(|v| js::truthy(Some(v)) && v.get("key") != key)
        .cloned()
        .collect()
}

/// `setEnvVars(vars, pathPrepend)`: the variables, when they are a list, then the prepend.
pub fn set_env(vars: &Value, prepend: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    if vars.is_array() {
        out.push(settings::write("envVars", vars));
    }
    if !prepend.is_null() {
        out.push(settings::write(
            "envPathPrepend",
            &Value::from(js::string(prepend)),
        ));
    }
    out
}

/// A variable's row: its name (`VAR` when it has none) and its value.
pub fn var_label(row: &Value) -> String {
    let key = row.get("key").filter(|k| js::truthy(Some(k)));
    key.map_or_else(|| "VAR".to_owned(), js::string)
}

pub fn var_value(row: &Value) -> String {
    let value = row.get("value").filter(|v| js::truthy(Some(v)));
    value.map_or_else(String::new, js::string)
}
