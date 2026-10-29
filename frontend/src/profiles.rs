//! The built-in profiles (`services/Profiles.js`), and what applying one sends: each of its values
//! through `dispatchSetting`, as `Omarchy.applyProfileValues` does. `tests/catalog_setters.rs`
//! runs `applyProfileValues` itself under Node for each, and `tests/catalog_parity.rs` holds the
//! list to `Profiles.builtins()`.

use serde_json::Value;

use crate::settings;

pub struct Profile {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Each settings key and its value, as JSON text.
    pub values: &'static [(&'static str, &'static str)],
}

pub const PROFILES: &[Profile] = &[
    Profile {
        id: "coding",
        title: "Coding",
        description: "Performance power, stay awake, fewer notifications.",
        values: &[
            ("powerProfile", "\"performance\""),
            ("stayAwake", "true"),
            ("doNotDisturb", "false"),
            ("hyprLook.animations", "true"),
        ],
    },
    Profile {
        id: "gaming",
        title: "Gaming",
        description: "Performance, tearing, animations off, notifications silenced, stay awake.",
        values: &[
            ("powerProfile", "\"performance\""),
            ("stayAwake", "true"),
            ("doNotDisturb", "true"),
            ("screensaverEnabled", "false"),
            ("hyprLook.animations", "false"),
            ("hyprLook.allowTearing", "true"),
        ],
    },
    Profile {
        id: "battery",
        title: "Battery",
        description: "Power saver, shorter idle, animations off.",
        values: &[
            ("powerProfile", "\"power-saver\""),
            ("stayAwake", "false"),
            ("hyprLook.animations", "false"),
            ("idleScreensaver", "120"),
            ("idleLock", "300"),
        ],
    },
];

/// What the row says it runs (`hint: "Settings import"`).
pub const HINT: &str = "Settings import";

impl Profile {
    /// Each value, as JSON.
    pub fn values(&self) -> Vec<(&'static str, Value)> {
        self.values
            .iter()
            .map(|(key, value)| (*key, serde_json::from_str(value).expect("a JSON value")))
            .collect()
    }

    /// The `settings.set` of each value, in order, with the keys it reads back.
    pub fn writes(&self) -> Vec<(&'static str, Value)> {
        self.values()
            .into_iter()
            .map(|(key, value)| (key, settings::write(key, &value)))
            .collect()
    }
}
