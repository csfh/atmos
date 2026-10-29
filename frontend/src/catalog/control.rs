//! What a mapped row of a catalog page reads and writes (docs/adr/0021). A row is mapped when its
//! QML control reads a snapshot value and writes it through one of `Omarchy.qml`'s generic paths,
//! which the window restates here: `writeHyprLook` and `writeHyprInput` for a field of the
//! Hyprland look or input, `set` (or a setter that does no more, such as `setTextSize`) for a key
//! of its own, a typed write with fixed arguments, or a link to another page.
//! `tests/catalog_controls.rs` runs the QML's own path under Node for every mapped row, and
//! checks that the QML row calls the path its control names.

use serde_json::{Map, Value, json};

pub use super::keys::KEYS;
use super::keys::{Reads, entry};
use super::setter::Setter;
use crate::hypr;
use crate::js;
use crate::protocol;
use crate::settings::{self, Held, Known, Settings};

/// Where a row's value is read from and how a change to it is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// A field of `hyprLook` as `HyprPrefs.clampLook` reads it, written by `writeHyprLook`.
    Look(&'static str),
    /// A field of `hyprInput` as `clampInput` reads it, written by `writeHyprInput`.
    Input(&'static str),
    /// A snapshot key as `Snapshot.clampPatchedValue` reads it, written by `Omarchy.set` or a
    /// setter that only skips the value shown.
    Key(&'static str),
    /// A property `Snapshot.js` derives from a list key, by the list and the property's name:
    /// the default device's name (`audioSink` from `audioSinks`, `defaultAudioName`).
    Derived(&'static str, &'static str),
}

/// What the window knows of a field, as the QML reads it.
#[derive(Debug, Clone, PartialEq)]
pub enum Reading {
    Waiting,
    Unknown,
    Value(Value),
}

impl Reading {
    pub fn value(&self) -> Option<&Value> {
        match self {
            Reading::Value(v) => Some(v),
            _ => None,
        }
    }
}

impl Field {
    /// The snapshot key it is read from.
    pub fn key(self) -> &'static str {
        match self {
            Field::Look(_) => "hyprLook",
            Field::Input(_) => "hyprInput",
            Field::Key(key) | Field::Derived(key, _) => key,
        }
    }

    pub fn held(self) -> &'static Held {
        &entry(self.key()).0
    }

    /// The field of a snapshot value of its key, as the QML reads it.
    pub fn normalize(self, raw: &Value) -> Value {
        match self {
            Field::Look(f) => hypr::clamp(look(f), raw, f),
            Field::Input(f) => hypr::clamp(input(f), raw, f),
            // `defaultAudioName`: the name of the first item marked default, or "".
            Field::Derived(..) => Value::from(
                raw.as_array()
                    .into_iter()
                    .flatten()
                    .find(|i| i.get("default").is_some_and(|d| js::truthy(Some(d))))
                    .and_then(|i| i.get("name"))
                    .filter(|n| js::truthy(Some(n)))
                    .map_or_else(String::new, js::string),
            ),
            Field::Key(key) => match entry(key).1 {
                Some(Reads::On) => Value::Bool(raw == &Value::Bool(true)),
                Some(Reads::Off) => Value::Bool(raw != &Value::Bool(false)),
                Some(Reads::Text(default)) => Value::from(if js::truthy(Some(raw)) {
                    js::string(raw)
                } else {
                    default.to_owned()
                }),
                Some(Reads::Number(default)) => {
                    let n = js::number(Some(raw));
                    js::json_number(if n == 0.0 || n.is_nan() { default } else { n })
                }
                Some(Reads::List(default)) if !raw.is_array() => Value::from(default.to_vec()),
                Some(Reads::List(_)) => raw.clone(),
                Some(Reads::Own(rule)) => rule(raw),
                // As it came; a whole number is one, as JavaScript holds it.
                None => match raw.as_f64() {
                    Some(n) => js::json_number(n),
                    None => raw.clone(),
                },
            },
        }
    }

    pub fn read(self, settings: &Settings) -> Reading {
        match settings.known(self.held()) {
            Known::Waiting => Reading::Waiting,
            Known::Unknown(_) => Reading::Unknown,
            Known::Value(raw) => Reading::Value(self.normalize(raw)),
        }
    }

    /// Whether it holds a number, so a choice of it writes one.
    fn numeric(self) -> bool {
        let kind = match self {
            Field::Look(f) => look(f),
            Field::Input(f) => input(f),
            Field::Key(key) => return matches!(entry(key).1, Some(Reads::Number(_))),
            Field::Derived(..) => return false,
        };
        matches!(kind, hypr::Kind::Int(..) | hypr::Kind::Float(..))
    }

    /// The `settings.set` that writes `value`, once the field has been read; none when it is the
    /// value shown, or an empty text for a key of its own, as the row and `set` skip both.
    pub fn write(self, value: Value, settings: &Settings) -> Option<Value> {
        if self.read(settings).value()? == &value {
            return None;
        }
        self.send(value, settings)
    }

    /// The `settings.set` that writes `value` once the field has been read, even when it is the
    /// value shown: a text row's Set, which the QML row sends as it stands.
    pub fn send(self, value: Value, settings: &Settings) -> Option<Value> {
        self.read(settings).value()?;
        match self {
            Field::Look(f) => {
                let value = hypr::clamp(look(f), &json!({ f: value }), f);
                Some(settings::write(&format!("hyprLook.{f}"), &value))
            }
            Field::Input(f) => {
                let value = hypr::clamp(input(f), &json!({ f: value }), f);
                Some(settings::write(&format!("hyprInput.{f}"), &value))
            }
            Field::Key(key) => (value != json!("")).then(|| settings::write(key, &value)),
            Field::Derived(..) => None,
        }
    }
}

fn look(field: &str) -> hypr::Kind {
    hypr::kind(hypr::LOOK, field).unwrap_or_else(|| panic!("hyprLook has no `{field}`"))
}

fn input(field: &str) -> hypr::Kind {
    hypr::kind(hypr::INPUT, field).unwrap_or_else(|| panic!("hyprInput has no `{field}`"))
}

/// When a row is shown, or its control can be used. Until what it reads has been read, it
/// does not hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Always,
    /// While the field reads true, or 1.
    On(Field),
    /// While the field reads false.
    Off(Field),
    /// While the field reads this text.
    Is(Field, &'static str),
    /// While the field does not read this text (`Omarchy.plymouth !== "default"`).
    IsNot(Field, &'static str),
    /// While the key's object says true of this name (`Omarchy.extras.herdr === true`).
    Flag(&'static str, &'static str),
    /// While every one holds.
    All(&'static [When]),
    /// While one of them holds (`(Omarchy.sshdEnabled || Omarchy.sshdActive)`).
    Any(&'static [When]),
    /// While the field's text or list is not empty (`.length > 0`).
    Has(Field),
    /// While the field's list has more than one item (`.length > 1`).
    Many(Field),
    /// While the field reads a number other than this one (`Omarchy.textSize !== 12`).
    Differs(Field, i64),
    /// While the window runs no job (`!Omarchy.jobBusy`): the page asks its `Jobs`.
    Idle,
}

impl When {
    /// Whether it holds while no job runs.
    pub fn holds(self, settings: &Settings) -> bool {
        self.holds_with(settings, false)
    }

    /// Whether it holds, `busy` saying whether the window runs a job.
    pub fn holds_with(self, settings: &Settings, busy: bool) -> bool {
        let read = |f: Field| f.read(settings);
        match self {
            When::Always => true,
            When::Idle => !busy,
            When::On(f) => read(f).value().is_some_and(on),
            When::Off(f) => read(f).value() == Some(&Value::Bool(false)),
            When::Is(f, text) => read(f).value().is_some_and(|v| js::string(v) == text),
            When::IsNot(f, text) => read(f).value().is_none_or(|v| js::string(v) != text),
            When::Flag(key, name) => {
                settings.value(&entry(key).0).and_then(|o| o.get(name)) == Some(&Value::Bool(true))
            }
            When::All(all) => all.iter().all(|w| w.holds_with(settings, busy)),
            When::Any(any) => any.iter().any(|w| w.holds_with(settings, busy)),
            When::Differs(f, n) => read(f)
                .value()
                .is_some_and(|v| v.as_f64() != Some(n as f64)),
            When::Many(f) => read(f)
                .value()
                .is_some_and(|v| v.as_array().is_some_and(|a| a.len() > 1)),
            When::Has(f) => read(f).value().is_some_and(|v| {
                v.as_str().is_some_and(|s| !s.is_empty())
                    || v.as_array().is_some_and(|a| !a.is_empty())
            }),
        }
    }

    /// The fields it reads.
    pub fn fields(self) -> Vec<Field> {
        match self {
            When::Always | When::Idle => Vec::new(),
            When::On(f)
            | When::Off(f)
            | When::Is(f, _)
            | When::IsNot(f, _)
            | When::Has(f)
            | When::Many(f)
            | When::Differs(f, _) => {
                vec![f]
            }
            When::Flag(key, _) => vec![Field::Key(key)],
            When::All(all) | When::Any(all) => all.iter().flat_map(|w| w.fields()).collect(),
        }
    }
}

/// Whether a switch's value is on: true, or 1 for a field that holds 0 or 1.
pub fn on(v: &Value) -> bool {
    v == &Value::Bool(true) || v.as_f64() == Some(1.0)
}

/// A number field, as the row's slider has it: its range and step, and the places the row rounds
/// what it writes to (`Math.round(value * 100) / 100` is 2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub places: i32,
    /// What the field shows while its value is 0, when the QML shows another
    /// (`value: Omarchy.chargeLimit || 100`).
    pub unset: Option<f64>,
}

/// A range for a row's data: `range(0.0, 32.0, 1.0, 0)`.
pub const fn range(min: f64, max: f64, step: f64, places: i32) -> Range {
    Range {
        min,
        max,
        step,
        places,
        unset: None,
    }
}

impl Range {
    /// The range, showing `shown` while the field's value is 0.
    pub const fn unset(self, shown: f64) -> Range {
        Range {
            unset: Some(shown),
            ..self
        }
    }

    /// The number the field shows for `current`.
    pub fn shown(&self, current: &Value) -> f64 {
        let n = js::number(Some(current));
        match self.unset {
            Some(shown) if n == 0.0 || n.is_nan() => shown,
            _ => n,
        }
    }

    pub fn round(&self, value: f64) -> f64 {
        let scale = 10f64.powi(self.places);
        js::round(value * scale) / scale
    }
}

/// A typed write with fixed arguments: its button, the write, and the `Omarchy` setter the QML
/// row calls, whose body sends it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Act {
    pub label: &'static str,
    pub name: &'static str,
    pub args: &'static [(&'static str, &'static str)],
    pub setter: &'static str,
}

/// A typed write for a row's data: `act("Reset", "hypr.reset", &[("group", "look")], "resetHyprLook")`.
pub const fn act(
    label: &'static str,
    name: &'static str,
    args: &'static [(&'static str, &'static str)],
    setter: &'static str,
) -> Act {
    Act {
        label,
        name,
        args,
        setter,
    }
}

impl Act {
    pub fn body(&self) -> Value {
        let args: Map<String, Value> = self
            .args
            .iter()
            .map(|(k, v)| ((*k).to_owned(), Value::from(*v)))
            .collect();
        protocol::write(self.name, &Value::Object(args))
    }
}

/// What a row changes or shows in this window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Control {
    /// Not in this window yet: the row is drawn without a control.
    Pending,
    /// A switch that writes the other value.
    Switch(Field),
    /// A number typed in the range.
    Number(Field, Range),
    /// One of the choices, by the value each writes and its label.
    Choice(Field, &'static [(&'static str, &'static str)]),
    /// A line of text, written when it is submitted or Set is pressed.
    Text(Field, &'static str),
    /// A button for each typed write.
    Acts(&'static [Act]),
    /// A button that opens another page: its label and the page.
    Go(&'static str, &'static str),
    /// Text that does not change (`valueText: "On"`).
    Says(&'static str),
    /// One of the options, written by the field's own path (`Omarchy.set`).
    Picks(Field, Options),
    /// A control whose change calls a setter of its own.
    Calls(Field, Shape, Setter),
    /// What a field reads, as text (`valueText: Omarchy.powerGovernor`).
    Shows(Field),
    /// A button, by its label, that calls a setter with nothing.
    Press(&'static str, Setter),
    /// Buttons, each with its setter, its condition and its confirm.
    Buttons(&'static [super::setter::Btn]),
    /// The minutes sudo mode stays on once the prompt turns it on (`Omarchy.sudoMinutes`), which
    /// the window holds, not the snapshot.
    Minutes(Range),
    /// A row its page module draws (`custom.rs`).
    Custom(super::custom::Custom),
    /// No control: the row only says something (`SettingRow` with no child).
    Bare,
}

/// The control a row that calls a setter draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// A switch that passes the other value (`setPresentationMode(next)`), or calls a setter
    /// that takes none (`toggleAudioOutputMute()`).
    Toggle,
    Count(Range),
    Pick(Options),
    /// A button, by its label, that passes what the field reads: `openThemeFolder()` writes
    /// `theme`.
    Button(&'static str),
}

/// Where a choice's options come from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Options {
    /// These, by the value each writes and its label.
    Fixed(&'static [(&'static str, &'static str)]),
    /// Each item of a key's list: a `{value, label}`, or a text labelled by the table or by
    /// itself (`profileOptions`, `Omarchy.fonts`).
    Listed(Field, &'static [(&'static str, &'static str)]),
    /// Each device of a key's list with a name, labelled by its description or its name
    /// (`deviceOptions`).
    Devices(Field),
}

impl Options {
    /// The options, by value and label.
    pub fn list(self, settings: &Settings) -> Vec<(String, String)> {
        match self {
            Options::Fixed(all) => all
                .iter()
                .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
                .collect(),
            Options::Devices(field) => field
                .read(settings)
                .value()
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|i| i.get("name").is_some_and(|n| js::truthy(Some(n))))
                .map(|i| {
                    let name = js::string(&i["name"]);
                    let label = i
                        .get("description")
                        .filter(|d| js::truthy(Some(d)))
                        .map_or_else(|| name.clone(), js::string);
                    (name, label)
                })
                .collect(),
            Options::Listed(field, labels) => field
                .read(settings)
                .value()
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|item| {
                    // An item is its text, or `{value, label}` (`RichUi.optionValue`).
                    let part = |name: &str| item.get(name).map_or_else(String::new, js::string);
                    if item.is_object() {
                        return (part("value"), part("label"));
                    }
                    let id = js::string(item);
                    let label = labels
                        .iter()
                        .find(|(v, _)| *v == id)
                        .map_or_else(|| id.clone(), |(_, l)| (*l).to_owned());
                    (id, label)
                })
                .collect(),
        }
    }

    /// The field the options are read from, if any.
    pub fn field(self) -> Option<Field> {
        match self {
            Options::Fixed(_) => None,
            Options::Listed(f, _) | Options::Devices(f) => Some(f),
        }
    }
}

impl Control {
    /// The field it reads and writes, if any.
    pub fn field(self) -> Option<Field> {
        match self {
            Control::Switch(f)
            | Control::Number(f, _)
            | Control::Choice(f, _)
            | Control::Text(f, _)
            | Control::Picks(f, _)
            | Control::Calls(f, ..)
            | Control::Shows(f) => Some(f),
            Control::Pending
            | Control::Acts(_)
            | Control::Go(..)
            | Control::Says(_)
            | Control::Press(..)
            | Control::Buttons(_)
            | Control::Minutes(_)
            | Control::Custom(_)
            | Control::Bare => None,
        }
    }

    /// Every field it reads: its own, its options' and its setter's.
    pub fn fields(self) -> Vec<Field> {
        let mut out: Vec<Field> = self.field().into_iter().collect();
        let options = match self {
            Control::Picks(_, o) | Control::Calls(_, Shape::Pick(o), _) => o.field(),
            _ => None,
        };
        out.extend(options);
        if let Control::Custom(c) = self {
            out.extend(c.fields());
            for s in c.setters() {
                out.extend(Control::Press("", s).fields());
            }
        }
        if let Control::Buttons(all) = self {
            for b in all {
                out.extend(b.when.fields());
                out.extend(Control::Press(b.label, b.setter).fields());
            }
        }
        if let Control::Calls(_, _, s) | Control::Press(_, s) = self {
            if let super::setter::Args::GivenAnd(f) = s.args {
                out.push(f);
            }
            out.extend(s.check.unless);
            out.extend(s.check.blocked);
            out.extend(s.check.needs);
            out.extend(s.check.only.fields());
            if let super::setter::Sends::HyprInput(f) = s.sends {
                out.push(Field::Input(f));
            }
            if let super::setter::Sends::Device(_, list) = s.sends {
                out.push(list);
            }
        }
        out
    }

    /// What pressing a switch at `current` writes: the other boolean, or the other of 0 and 1.
    pub fn flip(f: Field, current: &Value) -> Value {
        if f.numeric() {
            json!(if on(current) { 0 } else { 1 })
        } else {
            Value::Bool(!on(current))
        }
    }

    /// What choosing `chosen` writes: a number for a field that holds one.
    pub fn chosen(f: Field, chosen: &str) -> Value {
        if f.numeric() {
            js::json_number(js::round(js::number(Some(&Value::from(chosen)))))
        } else {
            Value::from(chosen)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(doc: Value) -> Settings {
        let mut s = Settings::default();
        s.adopt("settings.look", 1, &doc);
        s.adopt("settings.host", 1, &doc);
        s
    }

    #[test]
    fn a_field_is_read_as_the_qml_reads_it_and_written_when_it_changes() {
        let s = settings(json!({"hyprLook": {"gapsIn": "7"}, "touchscreenEnabled": null}));
        assert_eq!(Field::Look("gapsIn").read(&s), Reading::Value(json!(7)));
        assert_eq!(Field::Look("blur").read(&s), Reading::Value(json!(false)));
        assert_eq!(
            Field::Key("touchscreenEnabled").read(&s),
            Reading::Value(json!(true))
        );
        assert_eq!(Field::Key("stayAwake").read(&s), Reading::Waiting);
        assert_eq!(Field::Look("gapsIn").write(json!(7), &s), None, "no change");
        assert_eq!(
            Field::Look("gapsIn").write(json!(100), &s),
            Some(
                json!({"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 64,
                "readback": ["hyprLook", "hyprLookManaged"]})
            )
        );
        assert_eq!(
            Field::Key("stayAwake").write(json!(true), &s),
            None,
            "not read"
        );
    }

    #[test]
    fn a_condition_holds_only_once_it_is_read() {
        let s = settings(json!({"hyprLook": {"layout": "scrolling"}, "extras": {"herdr": true}}));
        assert!(When::Is(Field::Look("layout"), "scrolling").holds(&s));
        assert!(When::Flag("extras", "herdr").holds(&s));
        assert!(!When::On(Field::Key("hyprNoGaps")).holds(&s), "not read");
        assert!(!When::Off(Field::Key("hyprNoGaps")).holds(&s), "not read");
        assert_eq!(Control::flip(Field::Input("drag3fg"), &json!(1)), json!(0));
        assert_eq!(Control::chosen(Field::Input("followMouse"), "2"), json!(2));
        assert_eq!(
            Control::chosen(Field::Look("layout"), "scrolling"),
            json!("scrolling")
        );
    }
}
