//! The Keybindings page's logic (`pages/windows/BindingsPage.qml`, `services/Bindings.js`,
//! `RowText.js`): the overrides Atmos wrote, what Hyprland has bound and its filter, the Add
//! dialog's checks and the line it shows, and the writes Add and Remove send
//! (`Omarchy.addBinding`, `removeBinding`). `tests/catalog_bindings.rs` runs them under Node.

use serde_json::{Value, json};

use crate::catalog::Field;
use crate::js;

/// The overrides in Atmos's block of `bindings.lua`, and the lines outside it.
pub const BINDINGS: Field = Field::Key("bindings");
/// What Hyprland has bound (`omarchy menu keybindings --print`).
pub const KEYBINDINGS: Field = Field::Key("keybindings");
/// How many bound chords the list draws until Show all.
pub const LIMIT: usize = 100;

fn text(v: Option<&Value>) -> String {
    v.filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

fn units(s: &str) -> usize {
    s.encode_utf16().count()
}

fn list(v: Option<&Value>) -> Vec<Value> {
    v.and_then(Value::as_array).cloned().unwrap_or_default()
}

/// `sanitizeKeys`: a chord of letters, digits and `_ +.:,-`, spaces folded, at most 64.
pub fn sanitize_keys(raw: &str) -> String {
    if raw.contains(['\n', '\r']) {
        return String::new();
    }
    let text = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let ok = !text.is_empty()
        && units(&text) <= 64
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_ +.:,-".contains(c));
    if ok { text } else { String::new() }
}

/// `sanitizeLabel`: one line of at most 64.
pub fn sanitize_label(raw: &str) -> String {
    if raw.contains(['\n', '\r']) {
        return String::new();
    }
    let text = raw.trim();
    if text.is_empty() || units(text) > 64 {
        String::new()
    } else {
        text.to_owned()
    }
}

/// `sanitizeCommand`: one line of at most 256.
pub fn sanitize_command(raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() || units(text) > 256 || text.contains(['\n', '\r']) {
        String::new()
    } else {
        text.to_owned()
    }
}

/// `luaString`.
fn lua_string(v: &str) -> String {
    let mut out = String::from("\"");
    for c in v.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '"' => out.push_str("\\\""),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `categoryFromAction`.
pub fn category(action: &str) -> &'static str {
    let t = action.to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| t.contains(w));
    if any(&["workspace", "scratch"]) {
        "Workspace"
    } else if any(&[
        "launch",
        "terminal",
        "browser",
        "file manager",
        "menu",
        "app",
        "webapp",
        "tui",
    ]) {
        "Launcher"
    } else if any(&[
        "volume",
        "mute",
        "media",
        "play",
        "track",
        "brightness",
        "audio",
    ]) {
        "Media"
    } else if any(&["screenshot", "record", "capture", "ocr", "color picker"]) {
        "Capture"
    } else if any(&[
        "lock", "suspend", "power", "system", "idle", "shutdown", "reboot",
    ]) {
        "System"
    } else if any(&[
        "window",
        "float",
        "tile",
        "focus",
        "split",
        "fullscreen",
        "close",
        "swap",
    ]) {
        "Window"
    } else {
        "Other"
    }
}

/// `catalogConflict(catalog, keys)`: what the chord runs now, or "".
pub fn conflict(catalog: Option<&Value>, keys: &str) -> String {
    let chord = sanitize_keys(keys);
    if chord.is_empty() {
        return String::new();
    }
    list(catalog)
        .iter()
        .find(|r| sanitize_keys(&text(r.get("keys"))) == chord)
        .map_or_else(String::new, |r| text(r.get("action")))
}

/// `generatedBindText(row)`: the Hyprland lines Add writes, or "" for a row that is not one.
pub fn generated(keys: &str, label: &str, command: &str, unbind: bool) -> String {
    let keys = sanitize_keys(keys);
    let (label, command) = (sanitize_label(label), sanitize_command(command));
    if keys.is_empty() || (command.is_empty() && !unbind) {
        return String::new();
    }
    let mut lines = Vec::new();
    if unbind {
        lines.push(format!("hl.unbind({})", lua_string(&keys)));
    }
    if !command.is_empty() {
        let label = if label.is_empty() {
            "nil".to_owned()
        } else {
            lua_string(&label)
        };
        lines.push(format!(
            "o.bind({}, {label}, {})",
            lua_string(&keys),
            lua_string(&command)
        ));
    }
    lines.join("\n")
}

/// `bindingCopyText(row)`.
pub fn copy_text(row: &Value) -> String {
    let clean = |s: String| s.replace(['\r', '\n', '\0'], " ");
    let keys = clean(text(row.get("keys")));
    let action = ["action", "command", "label"]
        .iter()
        .map(|k| text(row.get(*k)))
        .find(|s| !s.is_empty())
        .unwrap_or_default();
    let action = clean(action);
    if !keys.is_empty() && !action.is_empty() && action != keys {
        format!("{keys} — {action}")
    } else if keys.is_empty() {
        action
    } else {
        keys
    }
}

/// `overrideRows`: the overrides that name a chord.
pub fn overrides(bindings: Option<&Value>) -> Vec<Value> {
    list(bindings)
        .into_iter()
        .filter(|r| js::truthy(r.get("keys")))
        .collect()
}

/// `describeOverride(row)`.
pub fn describe(row: &Value) -> String {
    let command = text(row.get("command"));
    if command.is_empty() {
        return "Unbinds the default for this chord.".into();
    }
    match text(row.get("label")) {
        l if l.is_empty() => command,
        l => format!("{l}. {command}"),
    }
}

/// `catalogRows`: each bound chord with its category, those the filter finds.
pub fn catalog(keybindings: Option<&Value>, filter: &str) -> Vec<Value> {
    let q = filter.to_lowercase();
    list(keybindings)
        .iter()
        .filter(|r| js::truthy(r.get("keys")))
        .filter_map(|r| {
            let action = text(r.get("action"));
            let cat = category(&action);
            let mut row = json!({ "keys": r["keys"], "category": cat });
            if let Some(a) = r.get("action") {
                row["action"] = a.clone();
            }
            let hay = format!("{} {action} {cat}", js::string(&r["keys"]));
            (q.is_empty() || hay.to_lowercase().contains(&q)).then_some(row)
        })
        .collect()
}

/// A bound chord's row: its words.
pub fn catalog_says(row: &Value) -> String {
    let cat = text(row.get("category"));
    let action = text(row.get("action"));
    if cat.is_empty() {
        action
    } else {
        format!("{cat} · {action}")
    }
}

/// The Filter row's words.
pub fn filter_says(shown: usize, total: usize) -> String {
    format!("{shown} of {total} bindings.")
}

/// The More bindings row's words.
pub fn more_says(limit: usize, total: usize) -> String {
    format!(
        "Showing {} of {total} bindings. Refine the filter, or show them all.",
        limit.min(total)
    )
}

/// `managedBindings()`: the overrides Atmos wrote, as it writes them.
fn managed(bindings: Option<&Value>) -> Vec<Value> {
    list(bindings)
        .iter()
        .filter(|r| r.get("managed") == Some(&Value::Bool(true)) && js::truthy(r.get("keys")))
        .map(|r| {
            json!({
                "keys": js::string(&r["keys"]), "label": text(r.get("label")),
                "command": text(r.get("command")), "unbind": r.get("unbind") == Some(&Value::Bool(true)),
            })
        })
        .collect()
}

/// `addBinding(keys, label, command, unbind)`: the write, or none when it is not one.
pub fn add(
    bindings: Option<&Value>,
    keybindings: Option<&Value>,
    (keys, label, command, unbind): (&str, &str, &str, bool),
) -> Option<Value> {
    let keys = keys.split_whitespace().collect::<Vec<_>>().join(" ");
    let (label, command) = (label.trim(), command.trim());
    if keys.is_empty() || units(&keys) > 64 {
        return None;
    }
    if !command.is_empty() && (units(command) > 256 || command.contains('\n')) {
        return None;
    }
    if command.is_empty() && !unbind {
        return None;
    }
    let taken = list(keybindings)
        .iter()
        .any(|r| text(r.get("keys")) == keys);
    let unbind = unbind || (!command.is_empty() && taken);
    let mut next: Vec<Value> = managed(bindings)
        .into_iter()
        .filter(|r| r["keys"] != keys.as_str())
        .collect();
    next.push(json!({ "keys": keys, "label": label, "command": command, "unbind": unbind }));
    Some(crate::lists::write("bindings", bindings, next))
}

/// `removeBinding(keys)`: the write, or none when Atmos wrote no such override.
pub fn remove(bindings: Option<&Value>, keys: &str) -> Option<Value> {
    let current = managed(bindings);
    let next: Vec<Value> = current
        .iter()
        .filter(|r| r["keys"] != keys)
        .cloned()
        .collect();
    (next.len() != current.len()).then(|| crate::lists::write("bindings", bindings, next))
}

/// Remove's confirm.
pub fn remove_message(keys: &str) -> String {
    format!("Remove the Atmos override for {keys}?")
}

/// What the Add dialog holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Draft {
    pub keys: String,
    pub label: String,
    pub command: String,
    pub unbind: bool,
}

impl Draft {
    /// `submitAdd()`: what Add hands `addBinding`, or what is wrong.
    pub fn submit(
        &self,
        keybindings: Option<&Value>,
    ) -> Result<(String, String, String, bool), &'static str> {
        let keys = sanitize_keys(&self.keys);
        if keys.is_empty() {
            return Err("Enter a chord such as SUPER + F.");
        }
        let (label, command) = (sanitize_label(&self.label), sanitize_command(&self.command));
        if !self.unbind && command.is_empty() {
            return Err("Enter a command, or turn on Unbind only.");
        }
        let unbind =
            self.unbind || (!command.is_empty() && !conflict(keybindings, &keys).is_empty());
        Ok((keys, label, command, unbind))
    }

    /// Whether Add can be used.
    pub fn ready(&self) -> bool {
        !self.keys.is_empty() && (self.unbind || !self.command.is_empty())
    }

    /// The dialog's first line: the chord's conflict, or what to do: press the shortcut while
    /// Record listens, else enter one.
    pub fn says(&self, keybindings: Option<&Value>, listening: bool) -> String {
        let taken = conflict(keybindings, &self.keys);
        if taken.is_empty() && listening {
            "Press the shortcut now. Super, Ctrl, Alt, and Shift count as modifiers.".into()
        } else if taken.is_empty() {
            "Use the same chord form as bindings.lua, or press Record shortcut.".into()
        } else {
            format!(
                "{} already runs “{taken}”. Add will unbind that first.",
                self.keys
            )
        }
    }

    /// The Hyprland lines it would write.
    pub fn line(&self, keybindings: Option<&Value>) -> String {
        let unbind = self.unbind || !conflict(keybindings, &self.keys).is_empty();
        match generated(&self.keys, &self.label, &self.command, unbind) {
            l if l.is_empty() => "The Hyprland line appears here once the chord is valid.".into(),
            l => l,
        }
    }
}

/// `recordChord(parts)`: a chord's parts in Hyprland's words, joined by ` + `, as a chord the
/// writer takes (`sanitizeKeys`), or "" when it is not one.
pub fn record_chord(parts: &[&str]) -> String {
    let words: Vec<String> = parts
        .iter()
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|p| !p.is_empty())
        .map(|p| match p.as_str() {
            "Meta" | "Super_L" | "Super_R" | "Super" => "SUPER".into(),
            "Control" | "Ctrl" => "CTRL".into(),
            "Alt" | "Alt_L" | "Alt_R" => "ALT".into(),
            "Shift" => "SHIFT".into(),
            "," => "comma".into(),
            "." => "period".into(),
            _ => p,
        })
        .collect();
    sanitize_keys(&words.join(" + "))
}

/// `keyToken(key, text)` for a key as ODK names it (`Chord::key`): a named key in Hyprland's
/// word, a letter or digit in capitals, or "" for a key a binding cannot take.
pub fn key_token(key: &str) -> String {
    let named = match key {
        "space" => "SPACE",
        "," => "comma",
        "." => "period",
        "tab" => "TAB",
        "escape" => "ESCAPE",
        "enter" => "RETURN",
        "backspace" => "BACKSPACE",
        "delete" => "DELETE",
        "left" => "LEFT",
        "up" => "UP",
        "right" => "RIGHT",
        "down" => "DOWN",
        _ => "",
    };
    if !named.is_empty() {
        return named.into();
    }
    if let Some(n) = key.strip_prefix('f').and_then(|n| n.parse::<u8>().ok())
        && (1..=12).contains(&n)
    {
        return format!("F{n}");
    }
    let mut chars = key.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphanumeric() => c.to_ascii_uppercase().to_string(),
        _ => String::new(),
    }
}

/// `recordKeyEvent(evt)`: the chord a key press makes, Super, Ctrl, Alt and Shift first; "" for
/// a key a binding cannot take.
pub fn recorded(key: &str, platform: bool, control: bool, alt: bool, shift: bool) -> String {
    let token = key_token(key);
    if token.is_empty() {
        return String::new();
    }
    let mut parts: Vec<&str> = Vec::new();
    for (on, word) in [
        (platform, "SUPER"),
        (control, "CTRL"),
        (alt, "ALT"),
        (shift, "SHIFT"),
    ] {
        if on {
            parts.push(word);
        }
    }
    parts.push(&token);
    record_chord(&parts)
}
