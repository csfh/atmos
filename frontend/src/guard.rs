//! Keep or revert (`services/Guard.js`): a write that can blank a display or strand the keyboard
//! applies at once, and a bar asks to keep it; unless kept within twelve seconds, or when
//! Revert is pressed, the values from when the bar opened are written back, newest first. A later
//! guarded write resets the deadline and keeps that first baseline. `Settings` holds the one
//! guard of the window and `pages/guard.rs` draws its bar. `tests/catalog_guard.rs` runs
//! `Guard.js` under Node against this.

use serde_json::{Value, json};

/// How long the bar waits.
pub const WINDOW_MS: u64 = 12_000;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Guard {
    pub open: bool,
    pub deadline: u64,
    /// Each guarded change once, by id, with what puts it back.
    pub reverts: Vec<(String, Value)>,
}

impl Guard {
    /// `open(state, change, now)`: false when the change has no revert.
    pub fn open(&mut self, id: &str, revert: Value, now: u64) -> bool {
        if id.is_empty() || !revert.is_object() {
            return false;
        }
        if !self.open {
            self.reverts.clear();
        }
        if !self.reverts.iter().any(|(i, _)| i == id) {
            self.reverts.push((id.to_owned(), revert));
        }
        self.open = true;
        self.deadline = now + WINDOW_MS;
        true
    }

    /// `keep()`.
    pub fn keep(&mut self) {
        *self = Guard::default();
    }

    fn fire(&mut self) -> Vec<Value> {
        let out = self.reverts.iter().rev().map(|(_, r)| r.clone()).collect();
        *self = Guard::default();
        out
    }

    /// `revert(state)`: what to write back, newest first.
    pub fn revert(&mut self) -> Vec<Value> {
        if !self.open {
            *self = Guard::default();
            return Vec::new();
        }
        self.fire()
    }

    /// `tick(state, now)`: what to write back once the deadline has passed.
    pub fn tick(&mut self, now: u64) -> Vec<Value> {
        if !self.open || now < self.deadline {
            return Vec::new();
        }
        self.fire()
    }

    /// `secondsLeft(state, now)`.
    pub fn seconds_left(&self, now: u64) -> u64 {
        if !self.open || now >= self.deadline {
            return 0;
        }
        (self.deadline - now).div_ceil(1000)
    }
}

/// `dispatchRevert(desc)`: the key it reads back and the write, given what the window shows of a
/// key (the monitor rules, `hyprInput`).
pub fn revert_write(
    desc: &Value,
    shown: &dyn Fn(&str) -> Option<Value>,
) -> Option<(&'static str, Value)> {
    match desc.get("kind").and_then(Value::as_str)? {
        "keyboardLayout" => {
            let name = desc
                .get("name")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(String::new, crate::js::string);
            Some((
                "keyboardLayout",
                crate::settings::write("keyboardLayout", &Value::from(name)),
            ))
        }
        "touchpad" => Some((
            "touchpadEnabled",
            crate::settings::write("touchpadEnabled", &json!(true)),
        )),
        "monitorScale" => {
            let scale = desc
                .get("scale")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(String::new, crate::js::string);
            let n: f64 = scale.parse().unwrap_or(f64::NAN);
            (!scale.is_empty() && n.is_finite() && n > 0.0).then(|| {
                (
                    "monitorScale",
                    json!({ "op": "display.scale", "scale": scale }),
                )
            })
        }
        "monitorRules" => {
            let rules = desc
                .get("rules")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            Some((
                "monitorRules",
                crate::lists::write("monitorRules", shown("monitorRules").as_ref(), rules),
            ))
        }
        "kbOverride" => {
            let text = |k: &str| {
                desc.get(k)
                    .filter(|v| crate::js::truthy(Some(v)))
                    .map_or_else(String::new, crate::js::string)
            };
            let input = crate::keyboard::input(shown("hyprInput").as_ref());
            let toggle = desc.get("groupToggle") == Some(&Value::Bool(true));
            let unmanaged = shown("hyprWorkspaceGestureUnmanaged") == Some(Value::Bool(true));
            let (body, _) = crate::keyboard::write(
                &input,
                (&text("layouts"), &text("variants"), toggle),
                unmanaged,
            );
            Some(("hyprInput", body))
        }
        _ => None,
    }
}

/// What the bar says.
pub fn says(seconds: u64) -> String {
    format!("These settings revert in {seconds}s unless you keep them.")
}
