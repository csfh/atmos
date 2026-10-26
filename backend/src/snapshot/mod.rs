//! The settings snapshot: one JSON document of everything the Quickshell pages
//! show, read from Omarchy, Hyprland, systemd and the files under /etc and ~.

mod bar;
mod host;
pub(crate) mod hypr;
pub(crate) mod keys;
mod merge;
mod net;
pub(crate) mod sh;
mod system;
pub(crate) mod topics;

pub use merge::{settings, settings_snapshot};
pub(crate) use system::crash_capture;

use serde_json::{json, Map, Value};
use sh::{lines, omarchy, out};

pub(super) struct Env {
    home: String,
    omarchy_path: String,
    user: String,
}

impl Env {
    fn read() -> Env {
        let home = std::env::var("HOME").unwrap_or_default();
        let omarchy_path = std::env::var("OMARCHY_PATH")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "/usr/share/omarchy".into());
        let user = std::env::var("USER")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| out("id", &["-un"]));
        Env {
            home,
            omarchy_path,
            user,
        }
    }
}

pub const GROUPS: &[&str] = &[
    "look", "rest", "all", "network", "disks", "accounts", "system",
];

pub(super) fn text(args: &[&str]) -> Value {
    json!(omarchy(args))
}

/// An `omarchy` answer that is the value: a failure is reported, not shown as "".
fn text_must(args: &[&str]) -> Value {
    json!(sh::omarchy_must(args).unwrap_or_default())
}

pub(super) fn theme_fields(f: &mut Map<String, Value>, full: bool) {
    if full {
        let must = |args: &[&str]| sh::omarchy_must(args).unwrap_or_default();
        f.insert("theme".into(), text_must(&["theme", "current"]));
        f.insert("background".into(), text_must(&["theme", "bg", "current"]));
        f.insert("font".into(), text_must(&["font", "current"]));
        f.insert("themes".into(), json!(lines(&must(&["theme", "list"]))));
        let extras: Vec<Value> = must(&["theme", "extras"])
            .lines()
            .filter_map(|l| {
                l.rsplit('/')
                    .next()
                    .filter(|n| !n.is_empty())
                    .map(|n| json!(n))
            })
            .collect();
        f.insert("extraThemes".into(), json!(extras));
        f.insert("fonts".into(), json!(lines(&must(&["font", "list"]))));
        f.insert("plymouth".into(), text_must(&["plymouth", "current"]));
        f.insert(
            "plymouthThemes".into(),
            json!(lines(&must(&["plymouth", "list"]))),
        );
    }
}

pub(super) fn live_toggles(f: &mut Map<String, Value>, full: bool) {
    let mut text_size = 12u64;
    let (mut stay, mut night, mut temp) = (false, false, 0u64);
    if full {
        let size = sh::omarchy_must(&["display", "text", "size"]).unwrap_or_default();
        let found = size.lines().find_map(|l| {
            l.contains("text size:")
                .then(|| l.split_whitespace().nth(2).unwrap_or("").to_string())
        });
        if let Some(n) = found.filter(|n| sh::is_uint(n)) {
            text_size = n.parse().unwrap_or(12);
        }
        let idle: Value = serde_json::from_str(
            &sh::omarchy_must(&["toggle", "idle", "status"]).unwrap_or_default(),
        )
        .unwrap_or(Value::Null);
        stay = sh::alt(idle.get("enabled")).is_some_and(|v| sh::jq_text(v) == "true");
        let status: Value = serde_json::from_str(
            &sh::omarchy_must(&["toggle", "nightlight", "--status"]).unwrap_or_default(),
        )
        .unwrap_or(Value::Null);
        night = sh::alt(status.get("enabled")).is_some_and(|v| sh::jq_text(v) == "true");
        let raw = sh::alt(status.get("temperature"))
            .map(sh::jq_text)
            .unwrap_or_default();
        temp = if sh::is_uint(&raw) {
            raw.parse().unwrap_or(0)
        } else {
            0
        };
    }
    f.insert("textSize".into(), json!(text_size));
    f.insert("stayAwake".into(), json!(stay));
    f.insert("nightlight".into(), json!(night));
    f.insert("nightlightTemperature".into(), json!(temp));
}

pub(super) fn toggle_off(name: &str) -> bool {
    !sh::ok("omarchy", &["toggle", "enabled", name])
}

/// Hyprland input and the managed workspace gesture, which the full snapshot adds.
pub(super) fn input_surface(env: &Env, f: &mut Map<String, Value>) {
    let input_file = crate::host::env_path(
        "ATMOS_INPUT_FILE",
        format!("{}/.config/hypr/input.lua", env.home),
    );
    let (gesture, managed, unmanaged) = hypr::gesture(&input_file);
    f.insert("hyprInput".into(), hypr::input(gesture));
    f.insert(
        "hyprInputManaged".into(),
        json!(sh::file_has(
            &input_file,
            crate::hyprland::Kind::Input.begin()
        )),
    );
    f.insert("hyprWorkspaceGesture".into(), json!(gesture));
    f.insert("hyprWorkspaceGestureManaged".into(), json!(managed));
    f.insert("hyprWorkspaceGestureUnmanaged".into(), json!(unmanaged));
}
