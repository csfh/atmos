//! The Startup page's logic (`pages/applications/StartupPage.qml`): the commands Hyprland
//! launches at login and the failed units, as the page lists them, and the `autostart` write
//! `Omarchy.addAutostart`, `setAutostartEnabled` and `removeAutostart` send: the managed lines
//! (`managedAutostart`) changed, merged with the lines the user wrote (`Settings.mergeUnmanaged`).
//! `tests/catalog_custom.rs` runs the page's code, and `tests/catalog_setters.rs` the setters,
//! under Node against this.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::js;

pub const AUTOSTART: Field = Field::Key("autostart");

/// The longest delay a command can wait (`PrefsSpinBox`'s `to`, and `addAutostart`'s clamp).
pub const MAX_DELAY: f64 = 600.0;

fn list(v: Option<&Value>) -> Vec<Value> {
    v.and_then(Value::as_array).cloned().unwrap_or_default()
}

/// `rows`: the lines with a command.
pub fn rows(autostart: Option<&Value>) -> Vec<Value> {
    list(autostart)
        .into_iter()
        .filter(|r| js::truthy(r.get("command")))
        .collect()
}

/// `failed`: the units that failed this boot.
pub fn failed(units: Option<&Value>) -> Vec<Value> {
    list(units)
        .into_iter()
        .filter(|u| {
            u.get("active").and_then(Value::as_str) == Some("failed")
                || u.get("sub").and_then(Value::as_str) == Some("failed")
        })
        .collect()
}

/// A line's description.
pub fn description(row: &Value) -> String {
    let off = if row.get("enabled") == Some(&Value::Bool(false)) {
        "Disabled. "
    } else {
        ""
    };
    let delay = row
        .get("delay")
        .filter(|d| js::truthy(Some(d)))
        .map_or_else(String::new, |d| {
            format!("Starts after {}s. ", js::string(d))
        });
    let whose = if js::truthy(row.get("managed")) {
        "Atmos manages this line."
    } else {
        "You wrote this line."
    };
    format!("{off}{delay}{whose}")
}

/// Whether a line is on.
pub fn enabled(row: &Value) -> bool {
    row.get("enabled") != Some(&Value::Bool(false))
}

/// `managedAutostart`: the lines Atmos wrote, as it writes them.
fn managed(autostart: Option<&Value>) -> Vec<Value> {
    list(autostart)
        .into_iter()
        .filter(|r| r.get("managed") == Some(&Value::Bool(true)) && js::truthy(r.get("command")))
        .map(|r| {
            let delay = js::round(js::number(Some(
                r.get("delay")
                    .filter(|d| js::truthy(Some(d)))
                    .unwrap_or(&json!(0)),
            )));
            let delay = if delay == 0.0 || delay.is_nan() {
                0.0
            } else {
                delay
            };
            json!({
                "command": js::string(&r["command"]),
                "delay": js::json_number(delay),
                "enabled": r.get("enabled") != Some(&Value::Bool(false)),
            })
        })
        .collect()
}

/// The `settings.set` of the managed lines `next`.
fn write(autostart: Option<&Value>, next: Vec<Value>) -> Value {
    crate::lists::write("autostart", autostart, next)
}

/// `addAutostart(command, delay)`: none when the command is empty, too long, on more than one
/// line, or already there.
pub fn add(autostart: Option<&Value>, command: &str, delay: &Value) -> Option<Value> {
    let command = command.trim();
    if command.is_empty() || command.encode_utf16().count() > 256 || command.contains('\n') {
        return None;
    }
    let zero = json!(0);
    let n = js::round(js::number(Some(if js::truthy(Some(delay)) {
        delay
    } else {
        &zero
    })));
    let n = if !n.is_finite() || n < 0.0 {
        0.0
    } else {
        n.min(MAX_DELAY)
    };
    let mut next = managed(autostart);
    if next.iter().any(|r| r["command"] == command) {
        return None;
    }
    next.push(json!({ "command": command, "delay": js::json_number(n), "enabled": true }));
    Some(write(autostart, next))
}

/// `setAutostartEnabled(command, on)`: none when Atmos does not manage it.
pub fn set_enabled(autostart: Option<&Value>, command: &str, on: bool) -> Option<Value> {
    let mut found = false;
    let next: Vec<Value> = managed(autostart)
        .into_iter()
        .map(|mut r| {
            if r["command"] == command {
                found = true;
                r["enabled"] = Value::Bool(on);
            }
            r
        })
        .collect();
    found.then(|| write(autostart, next))
}

/// `removeAutostart(command)`: none when Atmos does not manage it.
pub fn remove(autostart: Option<&Value>, command: &str) -> Option<Value> {
    let cur = managed(autostart);
    let next: Vec<Value> = cur
        .iter()
        .filter(|r| r["command"] != command)
        .cloned()
        .collect();
    (next.len() != cur.len()).then(|| write(autostart, next))
}

/// The Remove confirm's message.
pub fn remove_message(command: &str) -> String {
    format!("Remove {command} from the Atmos autostart block?")
}
