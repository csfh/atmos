//! The Appearance page's logic (`pages/AppearancePage.qml`): the themes it offers, the extra
//! themes and what Add, Update all and Remove… do with them, and the night light schedule.
//! `tests/catalog_appearance.rs` runs the page's code, and `tests/catalog_setters.rs` the
//! setters, under Node against this.
//!
//! The QML page sets the whole schedule through `setNightlightSchedule(day, night, nightOn)`,
//! but only the day reaches `atmosd` (`settings.set` carries one key, `nightlightDay`), and
//! `atmosd` keeps the night and the switch it finds in `hyprsunset.conf`. This window writes each
//! time and the switch as its own setting, which `atmosd` writes.

use serde_json::Value;

use crate::catalog::control::Field;
use crate::catalog::setter::{ANY, Args, Parse, Sends, Setter, setter, task};
use crate::{js, settings};

pub const THEME: Field = Field::Key("theme");
pub const THEMES: Field = Field::Key("themes");
pub const EXTRAS: Field = Field::Key("extraThemes");
pub const DAY: Field = Field::Key("nightlightDay");
pub const NIGHT: Field = Field::Key("nightlightNight");
pub const NIGHT_ON: Field = Field::Key("nightlightNightOn");

/// `setTheme(name)`: none for an empty name or the theme in use.
pub const SET_THEME: Setter = setter(
    "setTheme",
    Args::Given,
    ANY.text().non_empty().same(),
    Sends::Setting("theme"),
);
/// `installTheme(url)`: a URL `parseGitUrl` keeps.
pub const INSTALL_THEME: Setter = setter(
    "installTheme",
    Args::Given,
    ANY.parse(Parse::GitUrl),
    Sends::Job(task("theme.install", "theme-install", "The theme install").with_arg("url")),
);
/// `updateThemes()`.
pub const UPDATE_THEMES: Setter = setter(
    "updateThemes",
    Args::Nothing,
    ANY,
    Sends::Job(task("theme.update", "theme-update", "The theme update")),
);
/// `removeTheme(name)`: a name with no `/` that does not start with `.`.
pub const REMOVE_THEME: Setter = setter(
    "removeTheme",
    Args::Given,
    ANY.text().trim().parse(Parse::ThemeName),
    Sends::Write("theme.remove", Some("name")),
);

/// Add's dialog: what it says, while the clone runs, when the URL does not parse, and before.
pub const INSTALLING: &str = "Cloning the repository and switching to it…";
pub const BAD_URL: &str = "Need an https, ssh, or git@host:path URL whose last segment can name a theme. Install stays off until it parses.";
pub const ADD_INTRO: &str = "Paste a git URL for an Omarchy theme. Install clones it into ~/.config/omarchy/themes and switches to it.";
pub const ADD_SAYS: &str = "Install a theme from a public git repository.";

/// The transports `parseGitUrl` takes.
const TRANSPORTS: &[&str] = &[
    "ssh", "git", "git+ssh", "ssh+git", "http", "https", "ftp", "ftps", "file",
];

fn scheme_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')
}

/// The scheme at the start of `url` (`^[A-Za-z0-9][A-Za-z0-9+.-]*`), and what follows it.
fn scheme(url: &str) -> Option<(&str, &str)> {
    let first = url.chars().next()?;
    if !first.is_ascii_alphanumeric() {
        return None;
    }
    let end = url.find(|c: char| !scheme_char(c)).unwrap_or(url.len());
    Some((&url[..end], &url[end..]))
}

/// `gitThemeName(url)`: the theme a URL names, "" when it names none.
pub fn git_theme_name(url: &str) -> String {
    let mut path = url;
    if let Some(colon) = path.find(':')
        && !path.contains("://")
        && !path[..colon].contains('/')
    {
        path = &path[colon + 1..];
    }
    let mut base = path.rsplit('/').next().unwrap_or(path);
    if base.len() > 4 && base.ends_with(".git") {
        base = &base[..base.len() - 4];
    }
    base = base.strip_prefix("omarchy-").unwrap_or(base);
    if base.len() >= 6 && base.ends_with("-theme") {
        base = &base[..base.len() - 6];
    }
    let base = base.to_lowercase();
    let mut chars = base.chars();
    let ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && chars.all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '+' | '-')
        });
    if ok { base } else { String::new() }
}

/// `parseGitUrl(raw)`: a URL `omarchy theme install` takes, "" for one it does not.
pub fn parse_git_url(raw: &str) -> String {
    let url = raw.trim();
    if url.is_empty() || url.encode_utf16().count() > 2048 {
        return String::new();
    }
    if url.chars().any(char::is_whitespace) || url.starts_with('-') {
        return String::new();
    }
    let mut url = url.to_owned();
    match scheme(&url) {
        Some((_, rest)) if rest.starts_with("::") => return String::new(),
        Some((s, rest)) if rest.starts_with("://") => {
            let lower = s.to_ascii_lowercase();
            if !TRANSPORTS.contains(&lower.as_str()) {
                return String::new();
            }
            url = format!("{lower}{rest}");
        }
        _ => {
            let colon = url.find(':');
            let slash = url.find('/');
            match colon {
                None | Some(0) => return String::new(),
                Some(c) if slash.is_some_and(|s| s < c) => return String::new(),
                _ => {}
            }
        }
    }
    if git_theme_name(&url).is_empty() {
        String::new()
    } else {
        url
    }
}

/// What `removeTheme` takes of a name: none with a `/`, or that starts with `.`.
pub fn theme_name(raw: &str) -> String {
    let name = raw.trim();
    if name.contains('/') || name.starts_with('.') {
        String::new()
    } else {
        name.to_owned()
    }
}

/// The themes a list holds, as text.
pub fn names(list: Option<&Value>) -> Vec<String> {
    list.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(js::string)
        .collect()
}

/// `extraCountText()`.
pub fn extra_count(n: usize) -> String {
    if n == 1 {
        "One extra theme is installed on this machine.".to_owned()
    } else {
        format!("{n} extra themes are installed on this machine.")
    }
}

/// The Installed themes row's description.
pub fn extras_says(n: usize) -> String {
    format!(
        "{} Pick one to apply or remove, or pull the latest commit on all of them. Hover a name to preview its colors on this window.",
        extra_count(n)
    )
}

/// `syncExtraToRemove`: the one picked, while it is still installed, else the first.
pub fn picked(extras: &[String], draft: &str) -> String {
    if extras.iter().any(|e| e == draft) {
        draft.to_owned()
    } else {
        extras.first().cloned().unwrap_or_default()
    }
}

/// The Remove confirm's message.
pub fn remove_message(name: &str) -> String {
    format!(
        "Delete {name} from your extra themes? The files under ~/.config/omarchy/themes go with it."
    )
}

/// `HyprSunset.parseTime`: `H:MM` or `HH:MM`, hours to 23, padded; "" for anything else.
pub fn parse_time(raw: &str) -> String {
    let text = raw.trim();
    let Some((h, m)) = text.split_once(':') else {
        return String::new();
    };
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    let hour_ok = digits(h)
        && match h.len() {
            1 => true,
            2 => h.starts_with(['0', '1']) || (h.starts_with('2') && h.as_bytes()[1] <= b'3'),
            _ => false,
        };
    let minute_ok = digits(m) && m.len() == 2 && m.as_bytes()[0] <= b'5';
    if !hour_ok || !minute_ok {
        return String::new();
    }
    format!("{:02}:{m}", h.parse::<u8>().unwrap_or(0))
}

/// The schedule rows' descriptions, by whether both times parse.
pub fn schedule_says(valid: bool) -> &'static str {
    if valid {
        "Left is when daylight starts. Right is when night starts."
    } else {
        "Times need to look like 07:00 and 20:00 (hours 0–23)."
    }
}

pub fn use_schedule_says(valid: bool) -> &'static str {
    if valid {
        "Turn the timed amber profile on."
    } else {
        "Needs valid times above."
    }
}

/// A schedule setting's write, as `Omarchy.dispatchSetting(key, value)` makes it.
pub fn schedule_write(key: &str, value: Value) -> Value {
    settings::write(key, &value)
}

/// Background's current wallpaper row: its label and words (`fileBasename`, the path).
pub fn wallpaper_row(path: &str) -> (String, String) {
    if path.is_empty() {
        return (
            "Current wallpaper".into(),
            "No wallpaper is set for this theme yet. Choose a file below, or Next if the theme ships images.".into(),
        );
    }
    (
        path.rsplit('/').next().unwrap_or_default().to_owned(),
        path.to_owned(),
    )
}
