//! What this window changed, and what Preview holds (`pages/system/HistoryPage.qml`,
//! `services/History.js`). Every write the window sends goes through `Settings::send`, which
//! records it here, or holds it while Preview is on; Apply sends what is held, oldest first, and
//! Discard drops it. The log is in memory for this window, newest first, the latest 200.
//!
//! The QML window describes a setting by the command line its scripts once ran; this window
//! sends the setting itself, so it describes each write by its request, as the QML window
//! describes a typed write (`Requests.label`). `tests/catalog_history.rs` runs `History.js` and
//! `Requests.label` under Node against this.

use serde_json::Value;

/// How many changes the log keeps.
pub const MAX: usize = 200;

/// The arguments `Requests.label` leaves out.
const HIDDEN: &[&str] = &[
    "password", "current", "new", "header", "plan", "json", "payload", "text", "stdin",
];

/// One change: made (`you`) or held (`preview`).
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub at: u64,
    pub key: String,
    pub text: String,
    pub file: String,
    pub source: &'static str,
    pub sudo: bool,
}

/// A write Preview held, and what Apply sends.
#[derive(Debug, Clone, PartialEq)]
pub struct Held {
    pub change: Change,
    pub key: &'static str,
    pub body: Value,
    pub refused: String,
    /// It runs as root, behind the sudo prompt.
    pub root: bool,
}

/// The window's log.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Log {
    pub changes: Vec<Change>,
    pub held: Vec<Held>,
    pub preview: bool,
}

/// `Requests.label(body)`: the request's name, then its plain arguments.
pub fn label(body: &Value) -> Vec<String> {
    let name = body
        .get("name")
        .filter(|v| crate::js::truthy(Some(v)))
        .or_else(|| body.get("op"))
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string);
    let nested = body.get("args").filter(|a| a.is_object());
    let args = nested.unwrap_or(body);
    let mut parts = vec![name];
    for (key, value) in args.as_object().into_iter().flatten() {
        if HIDDEN.contains(&key.as_str()) {
            continue;
        }
        if nested.is_none() && matches!(key.as_str(), "op" | "name" | "args") {
            continue;
        }
        if value.is_null() || value.is_object() || value.is_array() {
            continue;
        }
        let mut units = 0;
        let text: String = crate::js::string(value)
            .chars()
            .take_while(|c| {
                units += c.len_utf16();
                units <= 60
            })
            .collect();
        parts.push(text);
    }
    parts
}

/// `describe(argv)`: the parts, a part with a space in quotes.
pub fn describe(argv: &[String]) -> String {
    argv.iter()
        .map(|p| {
            if p.contains(' ') {
                format!("\"{p}\"")
            } else {
                p.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `targetFile(argv, home)`: the file a command lands in, when its words name one.
pub fn target_file(text: &str, home: &str) -> String {
    let stop = |c: char| c.is_whitespace() || c == '"';
    for (at, _) in text.match_indices('/') {
        // The path runs from the slash to the next space or quote, and must end in one of these.
        let rest = &text[at..];
        let end = rest.find(stop).unwrap_or(rest.len());
        let path = &rest[..end];
        let Some(found) = [".lua", ".toml", ".json", ".conf", ".sh"]
            .iter()
            .find_map(|ext| {
                path.match_indices(ext).find_map(|(i, _)| {
                    let after = path[i + ext.len()..].chars().next();
                    (!after.is_some_and(|c| c.is_ascii_alphanumeric())).then_some(i + ext.len())
                })
            })
        else {
            continue;
        };
        let path = &path[..found];
        // A leading `~` or `/home/user` belongs to it.
        let before = &text[..at];
        let lead = if before.ends_with('~') {
            "~".to_owned()
        } else {
            let start = before.rfind(stop).map_or(0, |i| i + 1);
            let word = &before[start..];
            if word.starts_with("/home/") {
                word.to_owned()
            } else {
                String::new()
            }
        };
        let full = format!("{lead}{path}");
        if !home.is_empty() && (full == home || full.starts_with(&format!("{home}/"))) {
            return format!("~{}", &full[home.len()..]);
        }
        return full;
    }
    String::new()
}

/// `relativeTime(at, now)`.
pub fn relative_time(at: u64, now: u64) -> String {
    if at == 0 {
        return String::new();
    }
    let secs = crate::js::round((now as f64 - at as f64) / 1000.0).max(0.0);
    if secs < 10.0 {
        return "just now".into();
    }
    if secs < 60.0 {
        return format!("{secs}s ago");
    }
    let mins = crate::js::round(secs / 60.0);
    if mins < 60.0 {
        return format!("{mins}m ago");
    }
    let hours = crate::js::round(mins / 60.0);
    if hours < 24.0 {
        return format!("{hours}h ago");
    }
    format!("{}d ago", crate::js::round(hours / 24.0))
}

/// `entry(argv, {key, file, source, sudo})` of a request.
pub fn entry(
    body: &Value,
    key: &str,
    source: &'static str,
    sudo: bool,
    at: u64,
    home: &str,
) -> Change {
    let text = describe(&label(body));
    Change {
        at,
        key: key.into(),
        file: target_file(&text, home),
        text,
        source,
        sudo,
    }
}

fn push<T>(list: &mut Vec<T>, item: T) {
    list.insert(0, item);
    list.truncate(MAX);
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

impl Log {
    /// `recordChange`.
    pub fn record(&mut self, key: &str, body: &Value, sudo: bool) {
        let change = entry(body, key, "you", sudo, crate::settings::now_ms(), &home());
        push(&mut self.changes, change);
    }

    /// `holdChange`.
    pub fn hold(&mut self, key: &'static str, body: &Value, refused: String, root: bool) {
        let change = entry(
            body,
            key,
            "preview",
            root,
            crate::settings::now_ms(),
            &home(),
        );
        push(
            &mut self.held,
            Held {
                change,
                key,
                body: body.clone(),
                refused,
                root,
            },
        );
    }

    /// What Apply sends, oldest first; nothing is held after.
    pub fn take_held(&mut self) -> Vec<Held> {
        let mut held = std::mem::take(&mut self.held);
        held.reverse();
        held
    }

    /// How many changes this window made.
    pub fn by_you(&self) -> usize {
        self.changes.iter().filter(|c| c.source == "you").count()
    }
}

/// A change's row label: its file, else its key, else `or`.
pub fn row_label(c: &Change, or: &str) -> String {
    [&c.file, &c.key]
        .into_iter()
        .find(|t| !t.is_empty())
        .cloned()
        .unwrap_or_else(|| or.to_owned())
}

/// What Held says.
pub fn held_says(n: usize) -> String {
    if n == 1 {
        "1 change is waiting. Nothing has been written.".into()
    } else {
        format!("{n} changes are waiting. Nothing has been written.")
    }
}

/// What Preview changes says.
pub fn preview_says(on: bool) -> &'static str {
    if on {
        "On. Controls will not write. Anything you change is held below."
    } else {
        "Off. Controls write immediately, as normal."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_log_keeps_the_newest_first_and_apply_sends_the_oldest_first() {
        let mut log = Log::default();
        for n in 0..(MAX + 5) {
            log.record(
                "k",
                &json!({"op": "write", "name": "x", "args": {"n": n}}),
                false,
            );
        }
        assert_eq!(log.changes.len(), MAX);
        assert_eq!(log.changes[0].text, format!("x {}", MAX + 4));
        assert_eq!(log.by_you(), MAX);
        log.hold(
            "a",
            &json!({"op": "write", "name": "first"}),
            "no".into(),
            false,
        );
        log.hold(
            "b",
            &json!({"op": "write", "name": "second"}),
            "no".into(),
            true,
        );
        assert_eq!(log.held[0].change.source, "preview");
        let sent: Vec<_> = log
            .take_held()
            .into_iter()
            .map(|h| (h.key, h.root))
            .collect();
        assert_eq!(sent, [("a", false), ("b", true)]);
        assert!(log.held.is_empty());
    }
}
