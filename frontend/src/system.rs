//! The System page's rows Atmos draws itself (`pages/SystemPage.qml`): the hostname, the Omarchy
//! and Atmos versions, channels and updates, Hyprland integration, printers, the About logo, and
//! the weather location, with the jobs and writes each sends (`Omarchy.setHostname`,
//! `setOmarchyChannel`, `runOmarchyUpdate`, `setAboutBranding`, `setWeatherLocation`, ...).
//! `tests/catalog_setters.rs` runs each setter, and `tests/catalog_system.rs` the page's own
//! code, under Node against this.

use serde_json::{Value, json};

use crate::jobs::Job;
use crate::{js, protocol, settings};

/// `isHostname`: dot-separated labels of letters, digits and inner hyphens.
pub fn is_hostname(text: &str) -> bool {
    let label = |l: &str| {
        let b = l.as_bytes();
        !b.is_empty()
            && b.len() <= 63
            && b[0].is_ascii_alphanumeric()
            && b[b.len() - 1].is_ascii_alphanumeric()
            && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-')
    };
    !text.is_empty() && text.len() <= 253 && !text.starts_with('-') && text.split('.').all(label)
}

/// `parseHostname`.
pub fn parse_hostname(raw: &str) -> String {
    let t = raw.trim();
    if is_hostname(t) {
        t.to_owned()
    } else {
        String::new()
    }
}

/// `parseWeatherLocation`.
pub fn parse_location(raw: &str) -> String {
    let t = raw.trim();
    let ok = !t.is_empty()
        && t.encode_utf16().count() <= 128
        && !t.starts_with('-')
        && !t.contains(['\r', '\n']);
    if ok { t.to_owned() } else { String::new() }
}

/// `parseWeatherCoords`: `lat,lon` in range, a leading `+` dropped.
pub fn parse_coords(raw: &str) -> String {
    let t = raw.trim();
    let Some((a, b)) = t.split_once(',') else {
        return String::new();
    };
    let (a, b) = (a.trim_end(), b.trim_start());
    let number = |s: &str| {
        let body = s.strip_prefix(['+', '-']).unwrap_or(s);
        let (int, frac) = body
            .split_once('.')
            .map_or((body, None), |(i, f)| (i, Some(f)));
        !int.is_empty()
            && int.chars().all(|c| c.is_ascii_digit())
            && frac.is_none_or(|f| !f.is_empty() && f.chars().all(|c| c.is_ascii_digit()))
    };
    if !number(a)
        || !number(b)
        || a.contains(char::is_whitespace)
        || b.contains(char::is_whitespace)
    {
        return String::new();
    }
    let (lat, lon): (f64, f64) = (a.parse().unwrap_or(f64::NAN), b.parse().unwrap_or(f64::NAN));
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return String::new();
    }
    let token = |s: &str| s.strip_prefix('+').unwrap_or(s).to_owned();
    format!("{},{}", token(a), token(b))
}

pub fn hostname_says(ok: bool) -> &'static str {
    if ok {
        "How this computer shows up on the network and in your shell prompt."
    } else {
        "Letters, digits, and hyphens only (labels 1–63 characters). Set stays off until the name is valid."
    }
}

pub fn location_says(ok: bool, auto: bool) -> &'static str {
    match (ok, auto) {
        (true, true) => "Guessing the city from your IP for the weather widget and notifications.",
        (true, false) => "The city the weather widget and notifications use.",
        _ => "A city name, not a flag or a blank line. Set stays off until the name is valid.",
    }
}

pub fn coords_says(ok: bool) -> &'static str {
    if ok {
        "Optional latitude and longitude if the city name is ambiguous. Use lat,lon."
    } else {
        "Latitude -90 to 90 and longitude -180 to 180, as lat,lon. Spaces around the comma are fine. Set stays off until the pair is valid."
    }
}

/// `setHostname(name)`.
pub fn hostname_write(raw: &str, current: &str) -> Option<Value> {
    let name = parse_hostname(raw);
    (!name.is_empty() && name != current).then(|| settings::write("hostname", &Value::from(name)))
}

/// `setKeyboardLayout(name)`: the first of a list, a short lower-case name; with the guard that
/// puts `current` back.
pub fn layout_write(raw: &str, current: &str) -> Option<(Value, Value)> {
    let name = raw.trim();
    let name = name.split(',').next().unwrap_or_default();
    let ok = !name.is_empty()
        && name.len() <= 8
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    (ok && name != current).then(|| {
        let body = settings::write("keyboardLayout", &Value::from(name));
        (body, json!({ "kind": "keyboardLayout", "name": current }))
    })
}

/// `setWeatherLocation(name)`.
pub fn location_write(raw: &str, current: &str, auto: bool) -> Option<Value> {
    let name = parse_location(raw);
    (!name.is_empty() && (auto || name != current))
        .then(|| settings::write("weatherLocation", &Value::from(name)))
}

/// `clearWeatherLocation()`.
pub fn auto_write(auto: bool) -> Option<Value> {
    (!auto).then(|| settings::write("weatherLocation", &json!("")))
}

/// `setWeatherCoordinates(coords)`.
pub fn coords_write(raw: &str, location: &str, auto: bool, current: &str) -> Option<Value> {
    let coords = parse_coords(raw);
    let name = parse_location(location);
    (!coords.is_empty() && !name.is_empty() && !auto && coords != current).then(|| {
        protocol::write(
            "weather.coordinates",
            &json!({ "name": name, "coords": coords }),
        )
    })
}

pub const CHANNELS: &[(&str, &str)] = &[
    ("stable", "Stable"),
    ("rc", "RC"),
    ("edge", "Edge"),
    ("dev", "Dev"),
];

/// The channel confirm's message.
pub fn channel_message(name: &str) -> String {
    if name == "dev" {
        "Dev links Omarchy to a source checkout. That is for people working on Omarchy itself. The machine then upgrades against that tree.".into()
    } else {
        format!(
            "Switch the package channel to {name}. That rewrites pacman mirrors and upgrades the system."
        )
    }
}

/// `setOmarchyChannel(name)`.
pub fn channel_job(name: &str, current: &str) -> Option<Job> {
    (CHANNELS.iter().any(|(c, _)| *c == name) && name != current).then(|| {
        Job::new(
            "channel.set",
            json!({ "name": name }),
            "channel-set",
            "The channel switch",
        )
        .sudo()
    })
}

pub fn omarchy_check() -> Job {
    Job::new(
        "update.omarchy-check",
        json!({}),
        "update-check",
        "The update check",
    )
}

pub fn omarchy_update() -> Job {
    Job::new("update.omarchy", json!({}), "omarchy-update", "The update").sudo()
}

pub fn atmos_check() -> Job {
    Job::new(
        "update.atmos-check",
        json!({}),
        "atmos-update-check",
        "The Atmos update check",
    )
}

pub fn atmos_update() -> Job {
    Job::new(
        "update.atmos-apply",
        json!({}),
        "atmos-update",
        "The Atmos update",
    )
}

/// `setAtmosChannel(name)`.
pub fn atmos_channel_write(name: &str, current: &str) -> Option<Value> {
    (matches!(name.trim(), "stable" | "alpha") && name != current)
        .then(|| protocol::write("atmos.channel", &json!({ "channel": "stable" })))
}

/// `openPrinters()`: the printer window when it is installed, else the CUPS page.
pub fn printers_write(setup: bool) -> Value {
    if setup {
        protocol::write("printers.setup", &json!({}))
    } else {
        cups_write()
    }
}

pub fn cups_write() -> Value {
    protocol::write("cups.admin", &json!({}))
}

/// `setAboutBranding(action)` or `setScreensaverBranding(action)`: the interactive tool, or
/// Reset's write.
pub enum Branding {
    Job(Job),
    Write(Value),
}

/// The logo `which` (`about`, `screensaver`) for `action`, while `branded` says it is custom.
pub fn branding(which: &'static str, action: &str, branded: bool) -> Option<Branding> {
    let (name, kind, label) = match which {
        "about" => ("branding.about", "about-branding", "The About logo"),
        "screensaver" => (
            "branding.screensaver",
            "screensaver-branding",
            "The screensaver logo",
        ),
        _ => return None,
    };
    match action {
        "reset" => branded.then(|| {
            Branding::Write(protocol::write(
                "branding.reset",
                &json!({ "which": which }),
            ))
        }),
        "image" | "text" => Some(Branding::Job(
            Job::new(name, json!({ "action": action }), kind, label).interactive(),
        )),
        _ => None,
    }
}

/// What the Idle page's Lock slider sends at `next`: `setIdle(Omarchy.idleScreensaver, next)`,
/// which writes the screensaver's timeout again and not the lock's, so the lock is never
/// written (a bug of the QML window's, reported; this window does what it does). None when
/// `next` is what the lock is.
pub fn idle_lock_write(
    screensaver: Option<&serde_json::Value>,
    lock: Option<&serde_json::Value>,
    next: f64,
) -> Option<serde_json::Value> {
    if lock.is_some_and(|l| l.is_number() && crate::js::number(Some(l)) == next) {
        return None;
    }
    let saver = crate::js::round(crate::js::number(screensaver));
    let saver = if saver.is_finite() { saver } else { 0.0 };
    Some(crate::protocol::settings_set(
        "idleScreensaver",
        &crate::js::json_number(saver),
        &["idleScreensaver", "idleLock"],
    ))
}

/// The Screensaver logo row's words (Idle).
pub fn screensaver_says(branded: bool) -> &'static str {
    if branded {
        "You are using custom ASCII art on the screensaver."
    } else {
        "The stock Omarchy logo. Choose a picture to turn into ASCII. Edit opens the text file."
    }
}

/// The Omarchy row: its label, what it says, and what Copy copies.
pub fn omarchy_row(version: &str, channel: &str) -> (String, String, String) {
    if version.is_empty() {
        return (
            "Omarchy".into(),
            "Version was not readable. Copy stays disabled until omarchy version works.".into(),
            String::new(),
        );
    }
    let says = if channel.is_empty() {
        "Copy the version string if you need it in a report.".into()
    } else {
        format!("Channel {channel}. Copy the version string if you need it in a report.")
    };
    let copy = if channel.is_empty() {
        format!("Omarchy {version}")
    } else {
        format!("Omarchy {version} ({channel})")
    };
    (format!("Omarchy {version}"), says, copy)
}

/// The Atmos row: its label, what it says, its hint, and what Copy copies.
pub fn atmos_row(
    packaged: bool,
    version: &str,
    revision: &str,
    installed: bool,
) -> (String, &'static str, &'static str, String) {
    let label = if packaged && !version.is_empty() {
        version.to_owned()
    } else if !revision.is_empty() {
        revision.to_owned()
    } else {
        "Atmos".into()
    };
    let says = if packaged {
        "Installed package version. Copy if you need it in a report."
    } else if !revision.is_empty() {
        "Installed revision under ~/.local/share/atmos. Copy if you need it in a report."
    } else if installed {
        "REVISION was not readable. Copy stays disabled until that file exists."
    } else {
        "Not installed in XDG data. Copy stays disabled. Run install.sh from the Atmos source tree."
    };
    let hint = if packaged {
        "/usr/lib/atmos/PACKAGED"
    } else {
        "~/.local/share/atmos/REVISION"
    };
    let copy = if packaged { version } else { revision }.to_owned();
    (label, says, hint, copy)
}

/// What the Omarchy Updates row says.
pub fn updates_says(updating: bool, available: bool, summary: &str) -> String {
    if updating {
        "Updating…".into()
    } else if !summary.is_empty() {
        summary.into()
    } else if available {
        "Updates are available.".into()
    } else {
        "Omarchy is up to date.".into()
    }
}

/// `AtmosUpdate.parseCheckOutput`: whether a newer Atmos is on stable, and what to say.
pub fn parse_check(text: &str) -> (String, String) {
    let (mut status, mut summary) = ("unknown".to_owned(), String::new());
    for line in text.split('\n') {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(' ')
            .map_or((line, ""), |(k, v)| (k, v.trim_start()));
        match key {
            "status" if matches!(value, "current" | "behind" | "fetch-failed" | "packaged") => {
                status = value.into()
            }
            "summary" => {
                let bad = value.contains(['\n', '/'])
                    || value.contains("..")
                    || value.encode_utf16().count() > 200
                    || value.starts_with('-');
                if !bad {
                    summary = value.into();
                }
            }
            _ => {}
        }
    }
    if summary.is_empty() {
        summary = match status.as_str() {
            "behind" => "A newer Atmos is on stable.",
            "current" => "Atmos is up to date.",
            "fetch-failed" => "Could not fetch the stable branch.",
            "packaged" => "Atmos is managed by pacman. Update it with omarchy update.",
            _ => "",
        }
        .into();
    }
    (status, summary)
}

/// What the Atmos Updates row says, by what runs and what the last check said.
pub fn atmos_updates_says(running: Option<&str>, summary: &str, available: bool) -> String {
    match running {
        Some("atmos-update") => "Updating…".into(),
        Some("atmos-update-check") => "Checking…".into(),
        _ if !summary.is_empty() => summary.into(),
        _ if available => "A newer Atmos is on stable.".into(),
        _ => "Check the stable branch for a newer copy.".into(),
    }
}

/// Hyprland integration: what it says, and On or Off.
pub fn integration(on: bool) -> (&'static str, &'static str) {
    if on {
        (
            "The Atmos window rule and layout wrap load from hyprland.lua. Run atmos --setup off in a terminal to remove them.",
            "On",
        )
    } else {
        (
            "Off. Window rules and workspace settings Atmos writes are not loaded until you run atmos --setup in a terminal.",
            "Off",
        )
    }
}

/// What the Printers row says.
pub fn printers_says(active: bool) -> &'static str {
    if active {
        "CUPS is running. Set up opens the printer window. The web UI is http://127.0.0.1:631."
    } else {
        "CUPS is not running. You can still open the admin page if you start the service."
    }
}

/// What the About logo row says.
pub fn about_says(branded: bool) -> &'static str {
    if branded {
        "You are using custom ASCII art on the About screen."
    } else {
        "The stock Omarchy icon. Choose a picture to turn into ASCII. Edit opens the text file."
    }
}

/// `js::string` of a truthy value, else "".
pub fn text(v: Option<&Value>) -> String {
    v.filter(|v| js::truthy(Some(v)))
        .map_or_else(String::new, js::string)
}
