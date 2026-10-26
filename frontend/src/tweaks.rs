//! The Tweaks hub's logic, as `services/Tweaks.js` has it: the catalog and its groups, whether a
//! tweak is on from the snapshot (`currentOn`), and the setting a toggle or a Reset writes
//! (`writeFor`, `resetWrite`); Reset puts back the tweak's default, what Omarchy sets.
//! `tests/tweaks_parity.rs` runs the JavaScript under Node against this, so a change to either
//! side fails until the other follows.
//!
//! The page reads two snapshot keys, `hyprInput` and `tweaks`, which the daemon holds in two
//! settings topics (docs/adr/0011). [`Snapshot`] keeps what the topics and the writes' read-backs
//! said of them, and a key a producer could not read is a reason, never a default
//! (docs/adr/0006).

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::protocol;

/// A snapshot key the page reads, the settings topic that holds it (`backend/src/snapshot/keys.rs`;
/// `tests/pins.rs` holds the pair to it), and what the page says when it cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Held {
    pub key: &'static str,
    pub topic: &'static str,
    pub unread: &'static str,
}

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

/// Whether `topic` holds a key the page reads.
pub fn holds(topic: &str) -> bool {
    HELD.iter().any(|h| h.topic == topic)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A field of Hyprland's input block, through the same writer as the Input page.
    HyprInput,
    /// A file `ratmos task set-tweaks` writes.
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

    /// The snapshot keys a write reads back: what `Settings.js` `commandFor` names for the
    /// domain, so the reply carries them as the machine reports them (docs/adr/0005).
    pub fn readback(self) -> &'static [&'static str] {
        match self {
            Kind::HyprInput => &["hyprInput", "hyprInputManaged"],
            Kind::Tweak => &["tweaks"],
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
        protocol::settings_set(self.key, &self.value(on), self.kind.readback())
    }

    /// The `settings.set` Reset sends: one write of the default, with the keys it reads back.
    pub fn reset(&self) -> Value {
        protocol::settings_set(self.key, &self.reset_value(), self.kind.readback())
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

/// The two snapshot keys as the topics and the read-backs last said them.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    values: HashMap<&'static str, Value>,
    /// Why a key could not be read, until a sample or a read-back carries it again. A key keeps
    /// the value it had.
    failed: HashMap<&'static str, String>,
    /// The last sample taken in, by topic: one that is not after it is not taken.
    last: HashMap<String, u64>,
}

impl Snapshot {
    /// A sample of a settings topic. False when it is not after the last of that topic, or the
    /// topic holds nothing the page reads.
    pub fn adopt(&mut self, topic: &str, sampled_at: u64, doc: &Value) -> bool {
        if !holds(topic) || self.last.get(topic).is_some_and(|last| sampled_at <= *last) {
            return false;
        }
        self.last.insert(topic.to_owned(), sampled_at);
        let failed = doc.get("_failed").and_then(Value::as_object);
        for held in HELD.iter().filter(|h| h.topic == topic) {
            if let Some(value) = doc.get(held.key) {
                self.values.insert(held.key, value.clone());
                self.failed.remove(held.key);
            } else if let Some(why) = failed.and_then(|f| f.get(held.key)) {
                self.failed.insert(held.key, reason(why));
            }
        }
        true
    }

    /// The topic could not be collected: its keys keep what they had, and say why.
    pub fn topic_failed(&mut self, topic: &str, why: &str) {
        for held in HELD.iter().filter(|h| h.topic == topic) {
            self.failed.insert(held.key, why.to_owned());
        }
    }

    /// A write's reply: the keys it read back, as the machine reports them, and the ones it could
    /// not read.
    pub fn adopt_reply(&mut self, result: &Value) {
        let read = result.get("snapshot").and_then(Value::as_object);
        let failed = result.get("failed").and_then(Value::as_object);
        for held in HELD {
            if let Some(value) = read.and_then(|r| r.get(held.key)) {
                self.values.insert(held.key, value.clone());
                self.failed.remove(held.key);
            } else if let Some(why) = failed.and_then(|f| f.get(held.key)) {
                self.failed.insert(held.key, reason(why));
            }
        }
    }

    pub fn reading(&self, tweak: &Tweak) -> Reading {
        let key = tweak.snapshot_key();
        match (self.values.get(key), self.failed.get(key)) {
            (Some(value), _) => Reading::On(tweak.is_on(value)),
            (None, Some(why)) => Reading::Unknown(why.clone()),
            (None, None) => Reading::Waiting,
        }
    }

    /// The keys that could not be read, and why, in the order of [`HELD`].
    pub fn failures(&self) -> Vec<(&'static Held, &str)> {
        HELD.iter()
            .filter_map(|h| self.failed.get(h.key).map(|why| (h, why.as_str())))
            .collect()
    }
}

/// A producer's reason as text: it is a string, and anything else is shown as it came.
fn reason(why: &Value) -> String {
    match why {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("message")
            .and_then(Value::as_str)
            .map_or_else(|| why.to_string(), str::to_owned),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    /// A topic document with these keys and none failed.
    fn doc(keys: &[(&str, Value)]) -> Value {
        let mut map: Map<String, Value> = keys
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect();
        map.insert("_failed".into(), json!({}));
        Value::Object(map)
    }

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

    #[test]
    fn a_sample_is_taken_once_and_a_failed_key_keeps_its_value_and_says_why() {
        let mut s = Snapshot::default();
        let paste = tweak("middlePaste");
        assert_eq!(s.reading(paste), Reading::Waiting);
        assert!(
            !s.adopt("settings.network", 5, &doc(&[])),
            "not a topic it reads"
        );
        assert!(s.adopt(
            "settings.system",
            5,
            &doc(&[("tweaks", json!({"middlePaste": true}))])
        ));
        assert!(
            !s.adopt("settings.system", 5, &doc(&[("tweaks", json!({}))])),
            "the same one"
        );
        assert_eq!(s.reading(paste), Reading::On(true));

        let failed = json!({"_failed": {"tweaks": "no gtk file"}});
        assert!(s.adopt("settings.system", 6, &failed));
        assert_eq!(s.reading(paste), Reading::On(true), "it keeps what it had");
        assert_eq!(s.failures().len(), 1);

        let accel = tweak("accelFlat");
        s.topic_failed("settings.look", "timeout");
        assert_eq!(s.reading(accel), Reading::Unknown("timeout".into()));
        s.adopt_reply(&json!({"snapshot": {"hyprInput": {"accelProfile": "flat"}}}));
        assert_eq!(s.reading(accel), Reading::On(true));
        s.adopt_reply(&json!({"snapshot": {"tweaks": {"middlePaste": false}}}));
        assert!(s.failures().is_empty(), "both read again");
    }
}
