//! The Hooks page's logic (`pages/HooksPage.qml`, `services/Hooks.js`): the events, the scripts
//! each holds, what Add checks, and the writes its buttons send (`Omarchy.installHook`,
//! `createHook`, `removeHook`, `runHook`, `openHookFolder`, `setHookSample`, `editHook`).
//! `tests/catalog_hooks.rs` runs `Hooks.js` and the page's code, and `tests/catalog_setters.rs`
//! the setters, under Node against this.
//!
//! The QML setters keep a script's path to `$HOME/.config/omarchy/hooks/`. Inside the sandbox
//! `$HOME` is the window's own folder, so this window keeps it to a path with that folder in it;
//! the paths are the ones `atmosd` listed, and `atmosd` checks them again.

use serde_json::{Value, json};

use crate::protocol;

/// An event: its id, label, when it runs, what `$1` is, and what Run now passes as `$1`.
pub struct Type {
    pub id: &'static str,
    pub label: &'static str,
    pub when: &'static str,
    pub arg: &'static str,
    pub run_arg: &'static str,
}

pub const TYPES: &[Type] = &[
    Type {
        id: "theme-set",
        label: "Theme set",
        when: "After you switch themes.",
        arg: "the snake-cased theme name that was just set",
        run_arg: "theme",
    },
    Type {
        id: "font-set",
        label: "Font set",
        when: "After you change the monospace font.",
        arg: "the snake-cased font name that was just set",
        run_arg: "font",
    },
    Type {
        id: "post-boot",
        label: "After boot",
        when: "After the desktop starts.",
        arg: "",
        run_arg: "",
    },
    Type {
        id: "post-update",
        label: "After update",
        when: "During omarchy update, after packages and migrations.",
        arg: "",
        run_arg: "",
    },
    Type {
        id: "pre-refresh-pacman",
        label: "Before pacman refresh",
        when: "Before omarchy refresh pacman re-syncs packages.",
        arg: "",
        run_arg: "",
    },
    Type {
        id: "battery-low",
        label: "Low battery",
        when: "When Omarchy sends the low-battery notification.",
        arg: "the current battery percentage",
        run_arg: "",
    },
];

/// `isHookId`: a known event, or a folder name of lower-case letters, digits and hyphens.
pub fn is_hook_id(id: &str) -> bool {
    TYPES.iter().any(|t| t.id == id)
        || id
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `typeInfo(id)`: its label, when it runs, what `$1` is, and Run now's argument.
pub fn info(id: &str) -> Option<(String, &'static str, &'static str, &'static str)> {
    if let Some(t) = TYPES.iter().find(|t| t.id == id) {
        return Some((t.label.into(), t.when, t.arg, t.run_arg));
    }
    is_hook_id(id).then(|| (id.to_owned(), "A hook folder on this machine.", "", ""))
}

/// `eventBlurb(id)`.
pub fn blurb(id: &str) -> String {
    match info(id) {
        None => String::new(),
        Some((_, when, "", _)) => format!("{when} This event has no extra argument."),
        Some((_, when, arg, _)) => format!("{when} $1 is {arg}."),
    }
}

/// `runHint(type)`, with the theme and font in use.
pub fn run_hint(id: &str, theme: &str, font: &str) -> String {
    let arg = run_arg(id, theme, font);
    if !arg.is_empty() {
        return format!("Runs now with $1 set to {arg}.");
    }
    match info(id) {
        Some((_, _, a, _)) if !a.is_empty() => {
            "Runs now. $1 stays empty unless the real event supplied it.".into()
        }
        _ => "Runs every script for this event now.".into(),
    }
}

/// `runArg(type)`: the theme or font in use, for the events that pass it.
pub fn run_arg(id: &str, theme: &str, font: &str) -> String {
    match info(id).map(|i| i.3) {
        Some("theme") => theme.into(),
        Some("font") => font.into(),
        _ => String::new(),
    }
}

/// `sanitizeName`: a script name, `.sh` added when it has no extension.
pub fn sanitize_name(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty()
        || t.encode_utf16().count() > 64
        || t.contains('/')
        || t.contains("..")
        || t.ends_with(".sample")
    {
        return String::new();
    }
    let mut chars = t.chars();
    let ok = chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || "._+-".contains(c));
    if !ok {
        return String::new();
    }
    let has_ext = t
        .rsplit_once('.')
        .is_some_and(|(_, e)| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()));
    if has_ext {
        t.to_owned()
    } else {
        format!("{t}.sh")
    }
}

/// `sanitizeLine`: one line of at most 512 units.
pub fn sanitize_line(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() || t.encode_utf16().count() > 512 || t.contains(['\n', '\r']) {
        String::new()
    } else {
        t.to_owned()
    }
}

/// `destHint(type, name)`.
pub fn dest_hint(id: &str, name: &str) -> String {
    let file = sanitize_name(name);
    if !is_hook_id(id) || file.is_empty() {
        String::new()
    } else {
        format!("~/.config/omarchy/hooks/{id}.d/{file}")
    }
}

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .filter(|x| crate::js::truthy(Some(x)))
        .map_or_else(String::new, crate::js::string)
}

/// `itemsFor(raw, type)`: the scripts of an event, as `sanitizeItem` keeps them.
pub fn items_for(raw: Option<&Value>, id: &str) -> Vec<Value> {
    listing(raw)
        .into_iter()
        .filter(|i| i["type"] == id)
        .collect()
}

fn listing(raw: Option<&Value>) -> Vec<Value> {
    raw.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|r| r.is_object())
        .filter_map(|r| {
            let (kind, name, path) = (s(r, "type"), s(r, "name"), s(r, "path"));
            let ok = is_hook_id(&kind)
                && !name.is_empty()
                && !name.contains('/')
                && !name.contains("..")
                && path.starts_with('/')
                && !path.contains("..");
            ok.then(|| {
                json!({
                    "type": kind, "name": name, "path": path,
                    "sample": r.get("sample") == Some(&Value::Bool(true)),
                    "flat": r.get("flat") == Some(&Value::Bool(true)),
                })
            })
        })
        .collect()
}

/// `displayTypes(raw)`: the known events, then any other folder the listing names.
pub fn display_types(raw: Option<&Value>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = TYPES
        .iter()
        .map(|t| (t.id.to_owned(), t.label.to_owned()))
        .collect();
    for item in listing(raw) {
        let id = s(&item, "type");
        if !out.iter().any(|(i, _)| *i == id)
            && let Some((label, ..)) = info(&id)
        {
            out.push((id, label));
        }
    }
    out
}

/// `describeItem(item)`.
pub fn describe(item: &Value) -> String {
    if crate::js::truthy(item.get("sample")) {
        "Sample. Enable drops .sample so Omarchy runs it.".into()
    } else if crate::js::truthy(item.get("flat")) {
        "Runs first, before scripts in the .d folder.".into()
    } else {
        s(item, "path")
    }
}

/// A script's main button: Enable, Edit or Disable.
pub fn action(item: &Value) -> &'static str {
    if crate::js::truthy(item.get("sample")) {
        "Enable"
    } else if crate::js::truthy(item.get("flat")) {
        "Edit"
    } else {
        "Disable"
    }
}

fn under_hooks(path: &str) -> bool {
    path.contains("/.config/omarchy/hooks/") && !path.contains("..")
}

/// `installHook(type, path)`.
pub fn install_write(id: &str, path: &str) -> Option<Value> {
    let base = path.rsplit('/').next().unwrap_or_default();
    let ok = is_hook_id(id) && path.starts_with('/') && !path.contains("..") && !base.is_empty();
    ok.then(|| protocol::write("hook.install", &json!({ "type": id, "path": path })))
}

/// `createHook(type, name, command)`.
pub fn create_write(id: &str, name: &str, command: &str) -> Option<Value> {
    let (name, command) = (sanitize_name(name), sanitize_line(command));
    (is_hook_id(id) && !name.is_empty() && !command.is_empty()).then(|| {
        protocol::write(
            "hook.create",
            &json!({ "type": id, "name": name, "command": command }),
        )
    })
}

/// `removeHook(path)`: never a sample.
pub fn remove_write(path: &str) -> Option<Value> {
    (under_hooks(path) && !path.ends_with(".sample"))
        .then(|| protocol::write("hook.remove", &json!({ "path": path })))
}

/// `setHookSample(path, enabled)`.
pub fn sample_write(path: &str, enabled: bool) -> Option<Value> {
    under_hooks(path)
        .then(|| protocol::write("hook.sample", &json!({ "path": path, "enabled": enabled })))
}

/// `editHook(path)`.
pub fn edit_write(path: &str) -> Option<Value> {
    under_hooks(path).then(|| protocol::write("open.path", &json!({ "path": path })))
}

/// `runHook(name, arg)`.
pub fn run_write(id: &str, arg: &str) -> Option<Value> {
    let arg = arg.trim();
    is_hook_id(id).then(|| {
        if arg.is_empty() {
            protocol::write("hook.run", &json!({ "name": id }))
        } else {
            protocol::write("hook.run", &json!({ "name": id, "arg": arg }))
        }
    })
}

/// `openHookFolder(type)`.
pub fn folder_write(id: &str) -> Option<Value> {
    is_hook_id(id).then(|| protocol::write("hook.folder", &json!({ "type": id })))
}

/// Add's dialog.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Draft {
    pub kind: String,
    /// `file` or `command`.
    pub mode: String,
    pub file: String,
    pub name: String,
    pub command: String,
}

/// `submitAdd`: the write, or what is wrong.
pub fn submit(d: &Draft) -> Result<Option<Value>, &'static str> {
    if !is_hook_id(&d.kind) {
        return Err("Pick an event.");
    }
    if d.mode == "file" {
        if d.file.is_empty() {
            return Err("Choose a script file to copy.");
        }
        return Ok(install_write(&d.kind, &d.file));
    }
    if sanitize_name(&d.name).is_empty() {
        return Err("Name the script. Letters, digits, dot, plus, underscore, hyphen.");
    }
    if sanitize_line(&d.command).is_empty() {
        return Err("Enter one command line. $1 is available on theme, font, and battery events.");
    }
    Ok(create_write(&d.kind, &d.name, &d.command))
}

/// `destText()`.
pub fn dest_text(d: &Draft) -> String {
    if d.mode == "command" {
        return match dest_hint(&d.kind, &d.name) {
            h if h.is_empty() => "Name the script to see the install path.".into(),
            h => h,
        };
    }
    if d.file.is_empty() {
        return "Pick a file to copy into the event folder.".into();
    }
    format!(
        "~/.config/omarchy/hooks/{}.d/{}",
        d.kind,
        d.file.rsplit('/').next().unwrap_or_default()
    )
}

/// The Remove confirm's message.
pub fn remove_message(name: &str, kind: &str) -> String {
    format!("Delete {name} from the {kind} hook folder? Sample files stay.")
}
