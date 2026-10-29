//! The Applications page's logic (`pages/ApplicationsPage.qml`): the launchers you added, what
//! Add asks and checks (`submitAdd`), and what Add and Remove… send, as `Omarchy.installDesktopApp`,
//! `installTui`, `installWebApp`, `removeDesktopApp`, `removeTui` and `removeWebApp` make them.
//! `tests/catalog_launchers.rs` runs the page's code, and `tests/catalog_setters.rs` the setters,
//! under Node against this.

use serde_json::{Map, Value, json};

use crate::catalog::control::Field;
use crate::jobs::Job;
use crate::protocol;

/// The kinds of launcher, by what the page calls each: `desktop`, `tui` and `web`.
pub const KINDS: &[&str] = &["desktop", "tui", "web"];

/// The list a kind's launchers are in.
pub fn list(kind: &str) -> Field {
    Field::Key(match kind {
        "web" => "webApps",
        "tui" => "tuiApps",
        _ => "desktopApps",
    })
}

/// The window styles a terminal app takes.
pub const STYLES: &[(&str, &str)] = &[("tile", "Tiled"), ("float", "Floating")];

/// `kindLabel(kind)`.
pub fn kind_label(kind: &str) -> &'static str {
    match kind {
        "web" => "web app",
        "tui" => "terminal app",
        _ => "desktop app",
    }
}

/// `addTitle()`.
pub fn add_title(kind: &str) -> &'static str {
    match kind {
        "web" => "Add a web app",
        "tui" => "Add a terminal app",
        _ => "Add a desktop app",
    }
}

/// `addBlurb()`, with whether a launcher is being made.
pub fn add_blurb(kind: &str, making: bool) -> &'static str {
    match kind {
        _ if making => "Creating the launcher…",
        "web" => "A site in its own window. Leave the icon blank to use the site's favicon.",
        "tui" => "Opens in your default terminal. Icon can be a name, a PNG path, or a URL.",
        _ => "A launcher for a command on this machine. Icon can be a name, a PNG path, or a URL.",
    }
}

/// The kinds of job that make a launcher.
pub const MAKING: &[&str] = &["desktop-install", "tui-install", "webapp-install"];

/// What the icon field starts with (`openAdd`).
pub fn icon_default(kind: &str) -> &'static str {
    if kind == "tui" {
        "utilities-terminal"
    } else {
        ""
    }
}

/// `parseLauncherName`.
pub fn parse_name(raw: &str) -> String {
    let t = raw.trim();
    let ok = !t.is_empty()
        && t.encode_utf16().count() <= 80
        && !t.starts_with('-')
        && !t.contains(['/', '\n', '\r']);
    if ok { t.to_owned() } else { String::new() }
}

/// `parseWebAppUrl`.
pub fn parse_url(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() || t.chars().any(char::is_whitespace) {
        return String::new();
    }
    let scheme = t.find(':').is_some_and(|c| {
        let s = &t[..c];
        s.chars().next().is_some_and(|f| f.is_ascii_alphabetic())
            && s.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || "+.-".contains(ch))
    });
    let url = if scheme {
        t.to_owned()
    } else {
        format!("https://{t}")
    };
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        url
    } else {
        String::new()
    }
}

/// What Add's form holds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Draft {
    pub kind: String,
    pub name: String,
    pub command: String,
    pub url: String,
    pub style: String,
    pub icon: String,
}

/// `submitAdd`: the job that makes the launcher, or what is wrong.
pub fn submit(d: &Draft) -> Result<Job, &'static str> {
    let name = parse_name(&d.name);
    if name.is_empty() {
        return Err("Name cannot be empty or contain a slash.");
    }
    if d.kind == "web" {
        let url = parse_url(&d.url);
        if url.is_empty() {
            return Err("Enter an http or https URL.");
        }
        let mut args = Map::new();
        args.insert("name".into(), Value::from(name));
        args.insert("url".into(), Value::from(url));
        if !d.icon.is_empty() {
            args.insert("icon".into(), Value::from(d.icon.clone()));
        }
        return Ok(Job::new(
            "app.webapp-install",
            Value::Object(args),
            "webapp-install",
            "The web app",
        ));
    }
    let command = d.command.trim();
    if command.is_empty() {
        return Err("Enter the command to run.");
    }
    let icon = d.icon.trim();
    if d.kind == "tui" {
        let icon = if icon.is_empty() {
            "utilities-terminal"
        } else {
            icon
        };
        if !matches!(d.style.as_str(), "float" | "tile") {
            return Err("Pick tiled or floating.");
        }
        let args = json!({ "name": name, "command": command, "style": d.style, "icon": icon });
        return Ok(Job::new(
            "app.tui-install",
            args,
            "tui-install",
            "The terminal app",
        ));
    }
    let icon = if icon.is_empty() {
        "application-x-executable"
    } else {
        icon
    };
    let args = json!({ "name": name, "command": command, "icon": icon });
    Ok(Job::new(
        "app.desktop-install",
        args,
        "desktop-install",
        "The desktop app",
    ))
}

/// Remove of a launcher: the write, and whether it runs as root (`removeDesktopApp`).
pub fn remove(kind: &str, id: &str, name: &str) -> Option<(Value, bool)> {
    if id.is_empty() {
        return None;
    }
    Some(match kind {
        "web" => (
            protocol::write("webapp.remove", &json!({ "name": id })),
            false,
        ),
        "tui" => (protocol::write("tui.remove", &json!({ "name": id })), false),
        _ => {
            let name = if name.is_empty() { id } else { name };
            let args = json!({ "id": id, "name": name });
            (protocol::write("app.remove-desktop", &args), true)
        }
    })
}

/// The Remove confirm's title and message.
pub fn remove_confirm(kind: &str, name: &str) -> (String, String) {
    (
        format!("Remove {}", kind_label(kind)),
        format!(
            "Remove {name} from the launcher list? The desktop file under ~/.local/share/applications goes with it."
        ),
    )
}

fn text(row: &Value, name: &str) -> String {
    row.get(name)
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

/// A launcher's row: its label, what it says, its hint, and whether Remove… can be used.
pub fn row(kind: &str, r: &Value) -> (String, String, &'static str, bool) {
    let (label, says, hint) = match kind {
        "web" => (
            "Web app",
            "Opens this site in its own window.",
            "omarchy webapp remove",
        ),
        "tui" => (
            "TUI",
            "Opens in your default terminal.",
            "omarchy tui remove",
        ),
        _ => (
            "App",
            "Desktop launcher you added.",
            "omarchy remove launcher entry",
        ),
    };
    let or = |n: &str, d: &str| match text(r, n) {
        t if t.is_empty() => d.to_owned(),
        t => t,
    };
    let usable = !text(r, if kind == "desktop" { "id" } else { "name" }).is_empty();
    (or("name", label), or("detail", says), hint, usable)
}

/// The empty row's label and description, by kind.
pub fn empty(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "web" => ("Web apps", "No web apps."),
        "tui" => ("Terminal launchers", "No terminal launchers."),
        _ => ("Desktop launchers", "No desktop launchers."),
    }
}

/// A job as the setters' tests record it.
pub fn job_body(job: &Job) -> Value {
    json!({
        "op": "job", "name": job.name, "args": job.args, "kind": job.kind,
        "sudo": job.sudo, "interactive": job.interactive,
    })
}
