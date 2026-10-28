//! The Tweaks hub's logic, as `services/Tweaks.js` has it: the catalog and its groups, whether a
//! tweak is on from the snapshot (`currentOn`), and the setting a toggle or a Reset writes
//! (`writeFor`, `resetWrite`); Reset puts back the tweak's default, what Omarchy sets.
//! `tests/tweaks_parity.rs` runs the JavaScript under Node against this, so a change to either
//! side fails until the other follows.
//!
//! The page reads two snapshot keys, `hyprInput` and `tweaks`, which the daemon holds in two
//! settings topics (docs/adr/0011), from the window's [`Settings`]; a key a producer could not
//! read is a reason, never a default (docs/adr/0006).

use serde_json::{Value, json};

use crate::search::Row;
use crate::settings::{self, Held, Known, Settings};

pub const HELD: &[Held] = &[
    Held {
        key: "hyprInput",
        topic: "settings.look",
        unread: "Hyprland's input settings could not be read",
    },
    Held {
        key: "tweaks",
        topic: "settings.system",
        unread: "The tweak files could not be read",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A field of Hyprland's input block, through the same writer as the Input page.
    HyprInput,
    /// A file `atmosd task set-tweaks` writes.
    Tweak,
}

impl Kind {
    /// The name `Tweaks.js` gives it.
    pub fn id(self) -> &'static str {
        match self {
            Kind::HyprInput => "hypr-input",
            Kind::Tweak => "tweak",
        }
    }
}

/// A value a tweak writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    Bool(bool),
    Text(&'static str),
}

impl Setting {
    pub fn to_json(self) -> Value {
        match self {
            Setting::Bool(b) => json!(b),
            Setting::Text(t) => json!(t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tweak {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// The file or command it writes, which the row names.
    pub modifies: &'static str,
    /// The settings domain it writes, `<snapshot key>.<field>`.
    pub key: &'static str,
    pub on: Setting,
    pub off: Setting,
    pub kind: Kind,
    /// What Omarchy sets, which Reset writes (`defaultValue`). A tweak file whose value is not a
    /// boolean reads as it, so the two that Omarchy turns on are on until the machine says false.
    pub default: Setting,
}

pub const CATALOG: &[Tweak] = &[
    Tweak {
        id: "accelFlat",
        group: "Pointer",
        label: "Disable mouse acceleration",
        description: "Flat pointer acceleration. Same writer as Input → Acceleration.",
        modifies: "~/.config/hypr/input.lua · input.accel_profile",
        key: "hyprInput.accelProfile",
        on: Setting::Text("flat"),
        off: Setting::Text(""),
        kind: Kind::HyprInput,
        // Omarchy sets no profile (default/hypr/input.lua): Hyprland's adaptive one applies.
        default: Setting::Text(""),
    },
    Tweak {
        id: "naturalScroll",
        group: "Pointer",
        label: "Natural scrolling",
        description: "Content moves with the fingers. Same writer as Input.",
        modifies: "~/.config/hypr/input.lua · input.touchpad.natural_scroll",
        key: "hyprInput.naturalScroll",
        on: Setting::Bool(true),
        off: Setting::Bool(false),
        kind: Kind::HyprInput,
        // default/hypr/input.lua: touchpad.natural_scroll = false.
        default: Setting::Bool(false),
    },
    Tweak {
        id: "middlePaste",
        group: "Pointer",
        label: "Disable middle-click paste",
        description: "Stops the middle button from pasting the primary selection in GTK apps.",
        modifies: "~/.config/gtk-4.0/settings.ini · gtk-enable-primary-paste",
        key: "tweaks.middlePaste",
        on: Setting::Bool(true),
        off: Setting::Bool(false),
        kind: Kind::Tweak,
        // Omarchy turns primary paste on (install/user/first-run/gtk-primary-paste.sh).
        default: Setting::Bool(false),
    },
    Tweak {
        id: "electronWayland",
        group: "GTK/Electron",
        label: "Electron Wayland",
        description: "Hint Electron apps to use Ozone Wayland. Omarchy already sets this; turn off to opt out.",
        modifies: "~/.config/environment.d/10-atmos.conf · ELECTRON_OZONE_PLATFORM_HINT",
        key: "tweaks.electronWayland",
        on: Setting::Bool(true),
        off: Setting::Bool(false),
        kind: Kind::Tweak,
        // default/hypr/envs.lua: ELECTRON_OZONE_PLATFORM_HINT=wayland.
        default: Setting::Bool(true),
    },
    Tweak {
        id: "forceZeroScaling",
        group: "Compatibility",
        label: "XWayland zero scaling",
        description: "XWayland apps stay at 1x and the compositor scales them. Omarchy default is on.",
        modifies: "default/hypr/envs.lua · xwayland.force_zero_scaling",
        key: "tweaks.forceZeroScaling",
        on: Setting::Bool(true),
        off: Setting::Bool(false),
        kind: Kind::Tweak,
        // default/hypr/envs.lua: xwayland.force_zero_scaling = true.
        default: Setting::Bool(true),
    },
];

/// The catalog in sections, one per group in the order a group first appears (`groupedCatalog`).
pub fn groups() -> Vec<(&'static str, Vec<&'static Tweak>)> {
    let mut out: Vec<(&'static str, Vec<&'static Tweak>)> = Vec::new();
    for tweak in CATALOG {
        match out.iter_mut().find(|(name, _)| *name == tweak.group) {
            Some((_, items)) => items.push(tweak),
            None => out.push((tweak.group, vec![tweak])),
        }
    }
    out
}

/// The page's rows as search finds them: each tweak, by what its row says. The groups' headings
/// are the catalog's, not the page's words, and are not searched.
pub fn rows() -> Vec<Row> {
    CATALOG
        .iter()
        .map(|t| {
            Row::setting(
                "tweaks",
                t.label,
                t.description,
                t.modifies,
                &["tweak", "reset"],
            )
        })
        .collect()
}

pub fn by_id(id: &str) -> Option<&'static Tweak> {
    CATALOG.iter().find(|t| t.id == id)
}

impl Tweak {
    /// The snapshot key it is read from: `hyprInput` or `tweaks`.
    pub fn snapshot_key(&self) -> &'static str {
        self.key.split_once('.').map_or(self.key, |(head, _)| head)
    }

    /// The field of that key it is: `accelProfile`, `middlePaste`.
    pub fn field(&self) -> &'static str {
        self.key.split_once('.').map_or("", |(_, field)| field)
    }

    /// Whether it is on, given the snapshot key's value (`currentOn`): a value that is not an
    /// object has no fields.
    pub fn is_on(&self, snapshot: &Value) -> bool {
        let value = snapshot.as_object().and_then(|o| o.get(self.field()));
        match self.kind {
            Kind::HyprInput => value == Some(&self.on.to_json()),
            Kind::Tweak => value.and_then(Value::as_bool).unwrap_or(self.default_on()),
        }
    }

    /// Whether its default is on.
    pub fn default_on(&self) -> bool {
        self.default == self.on
    }

    /// What Reset writes (`resetValue`): the default.
    pub fn reset_value(&self) -> Value {
        self.default.to_json()
    }

    /// The value a toggle to `on` writes (`writeFor`).
    pub fn value(&self, on: bool) -> Value {
        if on { self.on } else { self.off }.to_json()
    }

    /// The `settings.set` a toggle to `on` sends, with the keys it reads back.
    pub fn write(&self, on: bool) -> Value {
        settings::write(self.key, &self.value(on))
    }

    /// The `settings.set` Reset sends: one write of the default, with the keys it reads back.
    pub fn reset(&self) -> Value {
        settings::write(self.key, &self.reset_value())
    }
}

/// What the page knows of a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// Nothing has said yet.
    Waiting,
    On(bool),
    /// The key has never been read, and this is why.
    Unknown(String),
}

/// What the page shows for a tweak, from what the window knows of its snapshot key.
pub fn reading(settings: &Settings, tweak: &Tweak) -> Reading {
    let held = HELD
        .iter()
        .find(|h| h.key == tweak.snapshot_key())
        .expect("every tweak reads a held key");
    match settings.known(held) {
        Known::Value(value) => Reading::On(tweak.is_on(value)),
        Known::Unknown(why) => Reading::Unknown(why.to_owned()),
        Known::Waiting => Reading::Waiting,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tweak(id: &str) -> &'static Tweak {
        by_id(id).unwrap()
    }

    #[test]
    fn the_groups_keep_the_catalog_order_and_every_row() {
        let names: Vec<_> = groups().iter().map(|(n, _)| *n).collect();
        assert_eq!(names, ["Pointer", "GTK/Electron", "Compatibility"]);
        assert_eq!(
            groups().iter().map(|(_, i)| i.len()).sum::<usize>(),
            CATALOG.len()
        );
    }

    #[test]
    fn a_tweak_omarchy_turns_on_is_on_until_the_snapshot_says_false() {
        let electron = tweak("electronWayland");
        assert!(electron.is_on(&json!({})));
        assert!(electron.is_on(&Value::Null));
        assert!(electron.is_on(&json!({"electronWayland": "false"})));
        assert!(!electron.is_on(&json!({"electronWayland": false})));
        let paste = tweak("middlePaste");
        assert!(!paste.is_on(&json!({})));
        assert!(!paste.is_on(&json!({"middlePaste": 1})));
        assert!(paste.is_on(&json!({"middlePaste": true})));
    }

    #[test]
    fn a_pointer_tweak_is_on_only_at_its_on_value() {
        let accel = tweak("accelFlat");
        assert!(accel.is_on(&json!({"accelProfile": "flat"})));
        assert!(!accel.is_on(&json!({"accelProfile": "adaptive"})));
        assert!(!accel.is_on(&json!({"accelProfile": "FLAT"})));
        assert!(!tweak("naturalScroll").is_on(&json!({"naturalScroll": "true"})));
    }

    #[test]
    fn a_write_names_its_domain_value_and_read_back() {
        assert_eq!(
            tweak("accelFlat").write(true),
            json!({"op": "settings.set", "domain": "hyprInput.accelProfile", "value": "flat",
                "readback": ["hyprInput", "hyprInputManaged"]})
        );
        assert_eq!(
            tweak("electronWayland").reset(),
            json!({"op": "settings.set", "domain": "tweaks.electronWayland", "value": true,
                "readback": ["tweaks"]}),
            "back to what Omarchy sets"
        );
        assert_eq!(tweak("accelFlat").reset()["value"], "");
    }
}
