//! The Notifications page's logic (`pages/NotificationsPage.qml`): a new reminder's write, as
//! `Omarchy.setReminder` makes it, and each waiting reminder's row, as the page's `Repeater` and
//! `RichUi.reminderCopyText` make it. `tests/catalog_custom.rs` runs the JavaScript under Node
//! against this.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::protocol;

/// The reminders the snapshot lists.
pub const REMINDERS: Field = Field::Key("reminders");

/// The minutes a new reminder starts at (`property int reminderMinutes: 5`), and their range.
pub const MINUTES: (f64, f64, f64) = (5.0, 1.0, 1440.0);

/// What the row of a reminder says it runs.
pub const HINT: &str = "omarchy reminder show --json";

const LATER: &str = "Clear all is above — Omarchy cannot drop a single timer.";

/// `setReminder(minutes, message)`: a whole number of minutes from 1, and the message trimmed.
pub fn reminder_write(minutes: &str, message: &str) -> Option<Value> {
    let minutes = minutes.trim();
    let digits = minutes.chars().all(|c| c.is_ascii_digit());
    if minutes.is_empty() || !digits || minutes.starts_with('0') {
        return None;
    }
    let n: u64 = minutes.parse().ok()?;
    Some(protocol::write(
        "reminder.set",
        &json!({ "minutes": n, "message": message.trim() }),
    ))
}

fn text(row: &Value, name: &str) -> String {
    row.get(name)
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

/// A reminder's label: `modelData.label ? modelData.label : "Reminder"`.
pub fn label(row: &Value) -> String {
    let label = text(row, "label");
    if label.is_empty() {
        "Reminder".to_owned()
    } else {
        label
    }
}

/// A reminder's description: when it fires, if the list says.
pub fn description(row: &Value) -> String {
    let remaining = text(row, "remaining");
    if remaining.is_empty() {
        return format!("Waiting. {LATER}");
    }
    let at = text(row, "atTime");
    let at = if at.is_empty() {
        String::new()
    } else {
        format!(" ({at})")
    };
    format!("Fires in {remaining}{at}. {LATER}")
}

/// `RichUi.reminderCopyText`: the label, then when, at most 1024 units.
pub fn copy_text(row: &Value) -> String {
    let clean = |s: String| s.replace(['\r', '\n', '\0'], " ");
    let mut label = text(row, "label");
    if label.is_empty() {
        label = text(row, "message");
    }
    let label = clean(label);
    let mut bits = Vec::new();
    let remaining = text(row, "remaining");
    if !remaining.is_empty() {
        bits.push(format!("in {}", clean(remaining)));
    }
    let at = text(row, "atTime");
    if !at.is_empty() {
        bits.push(format!("at {}", clean(at)));
    }
    let extra = clean(text(row, "message"));
    if !extra.is_empty() && extra != label {
        bits.push(extra);
    }
    let text = if bits.is_empty() {
        label
    } else {
        format!("{label} — {}", bits.join(" · "))
    };
    let mut units = 0;
    text.chars()
        .take_while(|c| {
            units += c.len_utf16();
            units <= 1024
        })
        .collect()
}

/// `showReminders()`.
pub fn show() -> Value {
    protocol::write("reminder.show", &json!({}))
}
