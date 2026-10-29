//! A row whose QML control calls a setter of `Omarchy.qml` of its own (`setChargeLimit`,
//! `setPowerProfile`, `toggleAudioOutputMute`): the setter's name, what the row passes it, the
//! checks the row and the setter make before anything is sent, and what the setter sends: a
//! setting (`dispatchSetting`), a typed write, or a field of the Hyprland input.
//! `tests/catalog_setters.rs` runs the setter itself, from `Omarchy.qml`, under Node, and holds
//! what is written here to what it sends for every value the row can give it.

use serde_json::{Map, Value, json};

use super::control::{Field, When};
use crate::js;
use crate::protocol;
use crate::settings::{self, Settings};

/// What the row passes the setter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Args {
    /// Nothing: `toggleAudioOutputMute()`.
    Nothing,
    /// The control's value: `setChargeLimit(next)`.
    Given,
    /// The control's value, then what a field reads: `setIdle(next, Omarchy.idleLock)`.
    GivenAnd(Field),
    /// A fixed first argument, then the control's value: `setTweak("swappiness", on)`.
    Lead(&'static str),
}

/// What the setter sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sends {
    /// `dispatchSetting(key, value)`: a `settings.set` of the key.
    Setting(&'static str),
    /// `Requests.write(name, { arg: value })`, or `{}` with no argument.
    Write(&'static str, Option<&'static str>),
    /// `writeHyprInput({ field: value })`.
    HyprInput(&'static str),
    /// `Requests.write(name, { arg: flag })`, with a fixed boolean: `bar.spacer`'s `on`.
    WriteBool(&'static str, &'static str, bool),
    /// `Requests.write(name, { ... })`, with fixed text: `restart`'s `{ what: "shell" }`.
    WriteText(&'static str, &'static [(&'static str, &'static str)]),
    /// `runJob` or `runInteractive` of `Requests.job(name, args)`: a job the window's one queue
    /// runs (`jobs.rs`).
    Job(Task),
    /// `Requests.write(name, { id, name })` for the device of the list whose name is the value,
    /// nothing when none is (`setAudioSink`).
    Device(&'static str, Field),
    /// Nothing: the setter opens the sudo prompt (`requestSudoMode`), which `jobs.rs` draws.
    SudoMode,
}

/// A job a setter starts, with fixed arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Task {
    /// The registry entry.
    pub name: &'static str,
    pub args: &'static [(&'static str, &'static str)],
    /// The label the daemon lists it by (`jobKind`).
    pub kind: &'static str,
    /// What a person calls it, for the toast that says it failed.
    pub what: &'static str,
    /// The entry runs as root, so the job waits for sudo mode (`tests/pins.rs` holds it to the
    /// entry).
    pub sudo: bool,
    /// The argument the value is passed as (`{ key: key }`), besides the fixed ones.
    pub arg: Option<&'static str>,
    /// The setter starts it with `runInteractive`: the entry is `Interactive`, and the window
    /// does not count it as busy (`jobBusy`).
    pub interactive: bool,
}

pub const fn task(name: &'static str, kind: &'static str, what: &'static str) -> Task {
    Task {
        name,
        args: &[],
        kind,
        what,
        sudo: false,
        arg: None,
        interactive: false,
    }
}

impl Task {
    /// The same job, waiting for sudo mode.
    pub const fn sudo(self) -> Task {
        Task { sudo: true, ..self }
    }

    /// The same job, with the value as its argument `name`.
    pub const fn with_arg(self, name: &'static str) -> Task {
        Task {
            arg: Some(name),
            ..self
        }
    }

    /// The same job, started as an interactive tool.
    pub const fn interactive(self) -> Task {
        Task {
            interactive: true,
            ..self
        }
    }

    pub fn args(&self) -> Value {
        let args: Map<String, Value> = self
            .args
            .iter()
            .map(|(k, v)| ((*k).to_owned(), Value::from(*v)))
            .collect();
        Value::Object(args)
    }
}

/// What is checked before the setter sends, by the row and by the setter, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Check {
    /// The row skips the value its field shows: `if (next !== Omarchy.chargeLimit)`.
    pub row_same: bool,
    /// The value is `Math.round(Number(value))`, and is refused when it is not finite.
    pub round: bool,
    /// The least and greatest value taken.
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// The values taken, as text.
    pub one_of: &'static [&'static str],
    /// An empty value is refused: `if (!name ...) return`.
    pub non_empty: bool,
    /// Values refused besides: `name === "default"`.
    pub refuse: &'static [&'static str],
    /// The value is trimmed first.
    pub trim: bool,
    /// The value is `String(value || "")` first.
    pub text: bool,
    /// The value must be one the field's own read keeps as it is: the setter checks it with the
    /// pattern the snapshot is read with (`setLocale`, `setTimezone`).
    pub valid: bool,
    /// The value is `value === true`.
    pub boolean: bool,
    /// The setter skips the value its field shows.
    pub same: bool,
    /// A field that, while it reads true, lets the value shown be sent again: the volume while
    /// it is muted (`percent === audioOutputVolume && !audioOutputMuted`).
    pub unless: Option<Field>,
    /// A field that, while it reads true, refuses everything:
    /// `if (hyprWorkspaceGestureUnmanaged) return`.
    pub blocked: Option<Field>,
    /// A field that must read true, or everything is refused: `if (!spacerPresent) return`.
    pub needs: Option<Field>,
    /// A condition that must hold, or everything is refused:
    /// `if (!sshdEnabled && !sshdActive) return`.
    pub only: When,
    /// The value is checked again, and what the check keeps is sent: `setupSshd` parses its key.
    pub parse: Option<Parse>,
}

/// No check at all; the builders below add one each.
pub const ANY: Check = Check {
    row_same: false,
    round: false,
    min: None,
    max: None,
    one_of: &[],
    non_empty: false,
    refuse: &[],
    trim: false,
    text: false,
    valid: false,
    boolean: false,
    same: false,
    unless: None,
    blocked: None,
    needs: None,
    only: When::Always,
    parse: None,
};

impl Check {
    pub const fn row_same(self) -> Check {
        Check {
            row_same: true,
            ..self
        }
    }

    /// A whole number, as `Math.round(Number(n))`, refused when it is not finite.
    pub const fn whole(self) -> Check {
        Check {
            round: true,
            ..self
        }
    }

    /// A whole number in `min..=max`, as `Math.round(Number(n))` with its bounds.
    pub const fn range(self, min: f64, max: f64) -> Check {
        Check {
            round: true,
            min: Some(min),
            max: Some(max),
            ..self
        }
    }

    /// A whole number of at least `min`: `if (!(n >= min)) return`.
    pub const fn at_least(self, min: f64) -> Check {
        Check {
            round: true,
            min: Some(min),
            ..self
        }
    }

    pub const fn one_of(self, values: &'static [&'static str]) -> Check {
        Check {
            one_of: values,
            ..self
        }
    }

    pub const fn non_empty(self) -> Check {
        Check {
            non_empty: true,
            ..self
        }
    }

    pub const fn refuse(self, values: &'static [&'static str]) -> Check {
        Check {
            refuse: values,
            ..self
        }
    }

    pub const fn text(self) -> Check {
        Check { text: true, ..self }
    }

    pub const fn trim(self) -> Check {
        Check { trim: true, ..self }
    }

    pub const fn valid(self) -> Check {
        Check {
            valid: true,
            ..self
        }
    }

    pub const fn boolean(self) -> Check {
        Check {
            boolean: true,
            ..self
        }
    }

    pub const fn same(self) -> Check {
        Check { same: true, ..self }
    }

    pub const fn unless(self, field: Field) -> Check {
        Check {
            unless: Some(field),
            ..self
        }
    }

    pub const fn needs(self, field: Field) -> Check {
        Check {
            needs: Some(field),
            ..self
        }
    }

    pub const fn only(self, when: When) -> Check {
        Check { only: when, ..self }
    }

    pub const fn parse(self, parse: Parse) -> Check {
        Check {
            parse: Some(parse),
            ..self
        }
    }

    pub const fn blocked(self, field: Field) -> Check {
        Check {
            blocked: Some(field),
            ..self
        }
    }
}

/// A setter of `Omarchy.qml` a row calls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setter {
    pub name: &'static str,
    pub args: Args,
    pub check: Check,
    pub sends: Sends,
}

pub const fn setter(name: &'static str, args: Args, check: Check, sends: Sends) -> Setter {
    Setter {
        name,
        args,
        check,
        sends,
    }
}

impl Setter {
    /// What the setter sends when the row gives it `value` and its field shows `shown`; none
    /// when the row or the setter would send nothing, or the field has not been read. A button
    /// has no field. A job is sent as `Requests.job` has it: `jobs.rs` starts it.
    pub fn body(&self, field: Option<Field>, value: Value, settings: &Settings) -> Option<Value> {
        let shown = match field {
            Some(f) => f.read(settings).value()?.clone(),
            None => Value::Null,
        };
        let reads_true = |f: Option<Field>| {
            f.and_then(|f| f.read(settings).value().cloned()) == Some(Value::Bool(true))
        };
        let c = &self.check;
        let excused = reads_true(c.unless);
        if c.row_same && value == shown && !excused {
            return None;
        }
        if reads_true(c.blocked)
            || c.needs.is_some_and(|f| !reads_true(Some(f)))
            || !c.only.holds(settings)
        {
            return None;
        }
        let value = if c.round {
            let n = js::round(js::number(Some(&value)));
            // `!(n >= min)` in the setter, which is `n < min` for the finite `n` taken here.
            let low = c.min.is_some_and(|m| n < m);
            let high = c.max.is_some_and(|m| n > m);
            if !n.is_finite() || low || high {
                return None;
            }
            js::json_number(n)
        } else if c.boolean {
            Value::Bool(value == Value::Bool(true))
        } else {
            let value = match c.text {
                true if js::truthy(Some(&value)) => Value::from(js::string(&value)),
                true => Value::from(""),
                false => value,
            };
            match c.trim {
                true => Value::from(js::string(&value).trim()),
                false => value,
            }
        };
        if !c.one_of.is_empty() && !value.as_str().is_some_and(|v| c.one_of.contains(&v)) {
            return None;
        }
        let value = match c.parse {
            Some(p) => Value::from(p.parse(&js::string(&value))),
            None => value,
        };
        if (c.non_empty || c.parse.is_some()) && !js::truthy(Some(&value)) {
            return None;
        }
        if value.as_str().is_some_and(|v| c.refuse.contains(&v)) {
            return None;
        }
        if c.valid && field.is_some_and(|f| f.normalize(&value) != value) {
            return None;
        }
        if c.same && value == shown && !excused {
            return None;
        }
        Some(match self.sends {
            Sends::Setting(key) => settings::write(key, &value),
            Sends::Write(name, arg) => {
                let mut args = Map::new();
                if let Some(arg) = arg {
                    args.insert(arg.to_owned(), value);
                }
                protocol::write(name, &Value::Object(args))
            }
            Sends::HyprInput(f) => return Field::Input(f).send(value, settings),
            Sends::Job(t) => {
                let mut args = t.args();
                if let Some(arg) = t.arg {
                    args[arg] = value;
                }
                json!({
                    "op": "job", "name": t.name, "args": args, "kind": t.kind, "sudo": t.sudo,
                    "interactive": t.interactive,
                })
            }
            Sends::SudoMode => return None,
            Sends::Device(name, list) => {
                let device = list
                    .read(settings)
                    .value()?
                    .as_array()?
                    .iter()
                    .find(|i| {
                        i.get("name")
                            .is_some_and(|n| js::string(n) == js::string(&value))
                    })?
                    .clone();
                protocol::write(
                    name,
                    &json!({ "id": js::string(&device["id"]), "name": value }),
                )
            }
            Sends::WriteBool(name, arg, flag) => protocol::write(name, &json!({ arg: flag })),
            Sends::WriteText(name, args) => {
                let args: Map<String, Value> = args
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), Value::from(*v)))
                    .collect();
                protocol::write(name, &Value::Object(args))
            }
        })
    }

    /// The arguments the row passes, as JSON, for the test that calls the setter itself.
    pub fn call_args(&self, value: &Value, settings: &Settings) -> Value {
        match self.args {
            Args::Nothing => json!([]),
            Args::Given => json!([value]),
            Args::GivenAnd(f) => json!([value, f.read(settings).value()]),
            Args::Lead(first) => json!([first, value]),
        }
    }
}

/// A button of a row of buttons: its label, the setter it calls with nothing, when it can be
/// used (`enabled:` on the button), and the confirm it asks first, if any (`PrefsConfirm`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Btn {
    pub label: &'static str,
    pub setter: Setter,
    pub when: When,
    pub confirm: Option<Confirm>,
    /// The line of text it asks for first, if any (a `PrefsDialog` with one field).
    pub prompt: Option<Prompt>,
}

/// A dialog that asks for one line of text, checks it, and passes what the check keeps to the
/// setter: its title, the field's placeholder, the button that goes on, and what it says while
/// it waits for text, while the text is not valid, and while the job it starts runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prompt {
    pub title: &'static str,
    pub placeholder: &'static str,
    pub action: &'static str,
    pub intro: &'static str,
    pub invalid: &'static str,
    pub busy: &'static str,
    pub parse: Parse,
}

/// A prompt for a row's data: `ask(title, placeholder, action, intro, invalid, busy, parse)`.
pub const fn ask(
    title: &'static str,
    placeholder: &'static str,
    action: &'static str,
    intro: &'static str,
    invalid: &'static str,
    busy: &'static str,
    parse: Parse,
) -> Prompt {
    Prompt {
        title,
        placeholder,
        action,
        intro,
        invalid,
        busy,
        parse,
    }
}

/// How a prompt's text is checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parse {
    /// `RichUi.parseSshPublicKey`.
    SshKey,
    /// `RichUi.parseGitUrl`.
    GitUrl,
    /// `removeTheme`'s check of a theme's name.
    ThemeName,
    /// An absolute path (`path.charAt(0) !== "/"` refused).
    Absolute,
    /// An absolute path with no `..`.
    SafePath,
}

impl Parse {
    /// What the check keeps of `text`, or "" when it is not valid.
    pub fn parse(self, text: &str) -> String {
        match self {
            Parse::SshKey => crate::security::ssh_public_key(text),
            Parse::GitUrl => crate::appearance::parse_git_url(text),
            Parse::ThemeName => crate::appearance::theme_name(text),
            Parse::Absolute if text.starts_with('/') => text.to_owned(),
            Parse::SafePath if text.starts_with('/') && !text.contains("..") => text.to_owned(),
            Parse::Absolute | Parse::SafePath => String::new(),
        }
    }
}

/// What a confirm says: its title, its message, and the label of the button that goes on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confirm {
    pub title: &'static str,
    pub message: &'static str,
    pub action: &'static str,
}

pub const fn btn(label: &'static str, setter: Setter) -> Btn {
    Btn {
        label,
        setter,
        when: When::Always,
        confirm: None,
        prompt: None,
    }
}

impl Btn {
    pub const fn when(self, when: When) -> Btn {
        Btn { when, ..self }
    }

    pub const fn prompt(self, prompt: Prompt) -> Btn {
        Btn {
            prompt: Some(prompt),
            ..self
        }
    }

    pub const fn confirm(
        self,
        title: &'static str,
        message: &'static str,
        action: &'static str,
    ) -> Btn {
        Btn {
            confirm: Some(Confirm {
                title,
                message,
                action,
            }),
            ..self
        }
    }
}

impl Confirm {
    /// Whether its action destroys something, so it is drawn as danger
    /// (`RichUi.confirmIsDestructive`).
    pub fn destructive(&self) -> bool {
        let t = self.action.trim().to_lowercase();
        matches!(
            t.as_str(),
            "remove" | "forget" | "clear" | "reset" | "undo" | "prune" | "restore" | "turn off"
        ) || t.contains("roll back")
    }
}
