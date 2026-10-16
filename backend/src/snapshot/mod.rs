//! The settings snapshot: one JSON document of everything the Quickshell pages
//! show, read from Omarchy, Hyprland, systemd and the files under /etc and ~.

mod bar;
mod host;
mod hypr;
mod merge;
mod net;
mod sh;
mod system;

pub use merge::{settings, settings_snapshot};

use crate::error::Result;
use serde_json::{json, Map, Value};
use sh::{lines, omarchy, out};
use std::path::Path;

struct Env {
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

/// What `snapshot.sh GROUP` printed.
pub fn capture(group: &str) -> Result<Value> {
    if !GROUPS.contains(&group) {
        return Err(format!("snapshot group must be one of {}", GROUPS.join(", ")).into());
    }
    let env = Env::read();
    let mut doc = match group {
        "look" => look(&env),
        "network" => net::network(),
        "disks" => {
            let mut f = Map::new();
            host::disks(&mut f);
            host::snapper(&mut f);
            f
        }
        "accounts" => accounts(&env),
        "system" => system_group(&env),
        _ => everything(&env, group == "rest"),
    };
    doc.insert("group".into(), json!(group));
    Ok(Value::Object(doc))
}

fn text(args: &[&str]) -> Value {
    json!(omarchy(args))
}

fn theme_fields(f: &mut Map<String, Value>, full: bool) {
    if full {
        f.insert("theme".into(), text(&["theme", "current"]));
        f.insert("background".into(), text(&["theme", "bg", "current"]));
        f.insert("font".into(), text(&["font", "current"]));
        f.insert("themes".into(), json!(lines(&omarchy(&["theme", "list"]))));
        let extras: Vec<Value> = omarchy(&["theme", "extras"])
            .lines()
            .filter_map(|l| {
                l.rsplit('/')
                    .next()
                    .filter(|n| !n.is_empty())
                    .map(|n| json!(n))
            })
            .collect();
        f.insert("extraThemes".into(), json!(extras));
        f.insert("fonts".into(), json!(lines(&omarchy(&["font", "list"]))));
        f.insert("plymouth".into(), text(&["plymouth", "current"]));
        f.insert(
            "plymouthThemes".into(),
            json!(lines(&omarchy(&["plymouth", "list"]))),
        );
    }
}

fn live_toggles(f: &mut Map<String, Value>, full: bool) {
    let mut text_size = 12u64;
    let (mut stay, mut night, mut temp) = (false, false, 0u64);
    if full {
        let size = out("omarchy", &["display", "text", "size"]);
        let found = size.lines().find_map(|l| {
            l.contains("text size:")
                .then(|| l.split_whitespace().nth(2).unwrap_or("").to_string())
        });
        if let Some(n) = found.filter(|n| sh::is_uint(n)) {
            text_size = n.parse().unwrap_or(12);
        }
        let idle: Value =
            serde_json::from_str(&omarchy(&["toggle", "idle", "status"])).unwrap_or(Value::Null);
        stay = sh::alt(idle.get("enabled")).is_some_and(|v| sh::jq_text(v) == "true");
        let status: Value = serde_json::from_str(&omarchy(&["toggle", "nightlight", "--status"]))
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

fn toggle_off(name: &str) -> bool {
    !sh::ok("omarchy", &["toggle", "enabled", name])
}

/// The bar, idle, Hyprland look, monitors and devices of the fast look snapshot.
fn look_surface(env: &Env, f: &mut Map<String, Value>) {
    let bar = bar::read_bar(&env.home, &env.omarchy_path, &env.user);
    f.extend(bar.fields);
    f.insert("barVisible".into(), json!(toggle_off("bar-off")));
    f.insert(
        "screensaverEnabled".into(),
        json!(toggle_off("screensaver-off")),
    );
    f.insert("suspendEnabled".into(), json!(toggle_off("suspend-off")));
    f.insert("hyprLook".into(), hypr::look());
    f.extend(hypr::markers(&env.home));
    let monitors = hypr::monitors();
    f.extend(hypr::devices(&env.home, &monitors).fields);
    f.insert("monitors".into(), json!(monitors));
    f.insert("doNotDisturb".into(), json!(hypr::dnd(&env.home)));
    let (count, active, list) = hypr::reminders();
    f.insert("reminderCount".into(), json!(count));
    f.insert("reminderActive".into(), json!(active));
    f.insert("reminders".into(), list);
}

/// Hyprland input and the managed workspace gesture, which the full snapshot adds.
fn input_surface(env: &Env, f: &mut Map<String, Value>) {
    let input_file = std::env::var("ATMOS_INPUT_FILE")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{}/.config/hypr/input.lua", env.home));
    let (gesture, managed, unmanaged) = hypr::gesture(Path::new(&input_file));
    f.insert("hyprInput".into(), hypr::input(gesture));
    f.insert(
        "hyprInputManaged".into(),
        json!(sh::file_has(Path::new(&input_file), "-- atmos:input begin")),
    );
    f.insert("hyprWorkspaceGesture".into(), json!(gesture));
    f.insert("hyprWorkspaceGestureManaged".into(), json!(managed));
    f.insert("hyprWorkspaceGestureUnmanaged".into(), json!(unmanaged));
}

fn control_fields(env: &Env, f: &mut Map<String, Value>, all: bool) {
    f.extend(system::control(&env.home, all).fields);
}

fn look(env: &Env) -> Map<String, Value> {
    let mut f = Map::new();
    theme_fields(&mut f, true);
    live_toggles(&mut f, true);
    host::branding(&mut f, &env.home, &env.omarchy_path);
    let (day, night, night_on) = hypr::nightlight_conf(&env.home);
    f.insert("nightlightDay".into(), json!(day));
    f.insert("nightlightNight".into(), json!(night));
    f.insert("nightlightNightOn".into(), json!(night_on));
    look_surface(env, &mut f);
    control_fields(env, &mut f, false);
    // The look group has no power or weather detail, and only these control keys.
    for key in [
        "powerPresent",
        "powerShowPercentage",
        "weatherUnit",
        "weatherRefreshMinutes",
    ] {
        f.remove(key);
    }
    for key in [
        "envVars",
        "envPathPrepend",
        "envDetected",
        "tweaks",
        "systemdUnits",
        "powerGovernor",
        "amdPstate",
        "chargeLimit",
        "chargeLimitAvailable",
        "netGateway",
        "netDnsServers",
    ] {
        f.remove(key);
    }
    f
}

fn accounts(env: &Env) -> Map<String, Value> {
    let inv = crate::inventory::accounts::collect();
    let mut f = Map::new();
    f.insert("fullName".into(), json!(system::full_name()));
    for key in ["currentUser", "avatarPath", "users", "groups"] {
        f.insert(
            key.into(),
            inv.get(key)
                .cloned()
                .unwrap_or(if key.ends_with("User") || key == "avatarPath" {
                    json!("")
                } else {
                    json!([])
                }),
        );
    }
    let _ = env;
    f
}

fn system_group(env: &Env) -> Map<String, Value> {
    let mut f = Map::new();
    f.insert("hostname".into(), json!(system::hostname()));
    f.extend(system::time());
    let (locale, locales) = system::locale();
    f.insert("locale".into(), json!(locale));
    f.insert("locales".into(), locales);
    f.insert(
        "parallelDownloads".into(),
        json!(system::parallel_downloads()),
    );
    let (layout, layouts) = system::keyboard();
    f.insert("keyboardLayout".into(), json!(layout));
    f.insert("keyboardLayouts".into(), layouts);
    f.insert("crashCapture".into(), json!(system::crash_capture()));
    f.insert(
        "diagnostics".into(),
        crate::inventory::diagnostics::collect(),
    );
    let control = system::control(&env.home, true).fields;
    for key in ["envVars", "envPathPrepend", "envDetected"] {
        if let Some(v) = control.get(key) {
            f.insert(key.into(), v.clone());
        }
    }
    f
}

/// The `all` document, or `rest` without the look-only keys.
fn everything(env: &Env, rest: bool) -> Map<String, Value> {
    let mut f = Map::new();
    theme_fields(&mut f, !rest);
    live_toggles(&mut f, !rest);
    f.insert("browser".into(), text(&["default", "browser"]));
    f.insert("terminal".into(), text(&["default", "terminal"]));
    f.insert("editor".into(), text(&["default", "editor"]));
    f.insert("agent".into(), text(&["default", "agent"]));
    f.insert("dns".into(), json!(net::dns()));
    f.insert(
        "screensaverEnabled".into(),
        json!(toggle_off("screensaver-off")),
    );
    f.insert("barVisible".into(), json!(toggle_off("bar-off")));
    f.insert("suspendEnabled".into(), json!(toggle_off("suspend-off")));
    f.insert("crashCapture".into(), json!(system::crash_capture()));
    f.extend(net::network());
    f.extend(net::audio());
    host::disks(&mut f);
    host::snapper(&mut f);
    let accounts = crate::inventory::accounts::collect();
    for key in ["currentUser", "avatarPath", "users", "groups"] {
        f.insert(
            key.into(),
            accounts.get(key).cloned().unwrap_or(Value::Null),
        );
    }
    f.insert("hardware".into(), crate::inventory::hardware::collect());
    f.insert(
        "diagnostics".into(),
        crate::inventory::diagnostics::collect(),
    );
    let apps = crate::inventory::apps::collect();
    f.insert("desktopApps".into(), apps["desktop"].clone());
    f.insert("tuiApps".into(), apps["tui"].clone());
    f.insert("webApps".into(), apps["web"].clone());
    host::branding(&mut f, &env.home, &env.omarchy_path);
    if rest {
        f.insert("screensaverBranded".into(), json!(false));
        f.insert("aboutBranded".into(), json!(false));
    }
    host::weather(&mut f, &env.home);
    host::power(&mut f);
    look_surface(env, &mut f);
    input_surface(env, &mut f);
    f.insert("hostname".into(), json!(system::hostname()));
    f.insert("fullName".into(), json!(system::full_name()));
    f.extend(system::time());
    let (locale, locales) = system::locale();
    f.insert("locale".into(), json!(locale));
    f.insert("locales".into(), locales);
    f.insert(
        "parallelDownloads".into(),
        json!(system::parallel_downloads()),
    );
    let (layout, layouts) = system::keyboard();
    f.insert("keyboardLayout".into(), json!(layout));
    f.insert("keyboardLayouts".into(), layouts);
    host::security(&mut f, &env.user);
    host::omarchy_info(&mut f);
    host::atmos(&mut f, &env.home);
    host::gpu_extras(&mut f);
    host::hardware_flags(&mut f);
    host::boot_and_defaults(&mut f, &env.home);
    let tailscale = f
        .get("tailscaleInstalled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    host::software(&mut f, &env.home, tailscale);
    f.insert("hooks".into(), crate::inventory::hooks::collect());
    host::marker_flags(&mut f, &env.home);
    control_fields(env, &mut f, true);
    f.insert("keybindings".into(), host::keybindings());
    host::focus_and_print(&mut f);
    let (day, night, night_on) = if rest {
        ("07:00".to_string(), "20:00".to_string(), false)
    } else {
        hypr::nightlight_conf(&env.home)
    };
    f.insert("nightlightDay".into(), json!(day));
    f.insert("nightlightNight".into(), json!(night));
    f.insert("nightlightNightOn".into(), json!(night_on));
    if rest {
        for key in [
            "theme",
            "background",
            "font",
            "textSize",
            "themes",
            "extraThemes",
            "fonts",
            "stayAwake",
            "nightlight",
            "nightlightTemperature",
            "screensaverBranded",
            "aboutBranded",
            "plymouth",
            "plymouthThemes",
            "nightlightDay",
            "nightlightNight",
            "nightlightNightOn",
        ] {
            f.remove(key);
        }
    }
    f
}
