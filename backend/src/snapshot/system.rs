//! Host settings read from /etc and timedatectl: hostname, time, locale,
//! keyboard, pacman, and the environment and tweak files Atmos manages.

use super::sh::{file_has, first_line, is_uint, lines, ok, out, present, read};
use regex_lite::Regex;
use serde_json::{json, Map, Value};
use std::path::Path;

fn host_label_ok(name: &str) -> bool {
    let label = r"[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?";
    !name.starts_with('-')
        && name.chars().count() <= 253
        && Regex::new(&format!(r"^{label}(\.{label})*$")).is_ok_and(|re| re.is_match(name))
}

pub fn hostname() -> String {
    let mut name = read(Path::new("/etc/hostname"))
        .lines()
        .next()
        .unwrap_or("")
        .split(' ')
        .next()
        .unwrap_or("")
        .to_string();
    if name.is_empty() && present("hostnamectl") {
        name = out("hostnamectl", &["--static"]);
    }
    if host_label_ok(&name) {
        name
    } else {
        String::new()
    }
}

pub fn full_name() -> String {
    let user = out("id", &["-un"]);
    let ok_user = Regex::new(r"^[a-z_][a-z0-9_-]*$").is_ok_and(|re| re.is_match(&user));
    let mut name = String::new();
    if ok_user {
        let entry = out("getent", &["passwd", &user]);
        name = entry
            .split(':')
            .nth(4)
            .unwrap_or("")
            .split(',')
            .next()
            .unwrap_or("")
            .to_string();
    }
    let name = name.lines().next().unwrap_or("").to_string();
    if name.starts_with('-') || name.chars().count() > 256 || name.contains(':') {
        String::new()
    } else {
        name
    }
}

fn tz_ok(tz: &str) -> bool {
    !tz.contains("..")
        && !tz.is_empty()
        && tz
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "/_+-".contains(c))
}

pub fn time() -> Map<String, Value> {
    let mut f = Map::new();
    let (mut timezone, mut zones) = (String::new(), json!([]));
    if present("timedatectl") {
        timezone = out("timedatectl", &["show", "-p", "Timezone", "--value"]);
        if !tz_ok(&timezone) {
            timezone.clear();
        }
        zones = json!(lines(&out("timedatectl", &["list-timezones"]))
            .into_iter()
            .filter(|z| z.as_str().is_some_and(tz_ok))
            .collect::<Vec<_>>());
    }
    let (mut ntp, mut available, mut synced) = (false, false, false);
    if present("timedatectl") {
        let show = |prop: &str| out("timedatectl", &["show", "-p", prop, "--value"]) == "yes";
        available = show("CanNTP");
        ntp = show("NTP");
        synced = show("NTPSynchronized");
    }
    f.insert("timezone".into(), json!(timezone));
    f.insert("timezones".into(), zones);
    f.insert("ntp".into(), json!(ntp));
    f.insert("ntpAvailable".into(), json!(available));
    f.insert("ntpSynchronized".into(), json!(synced));
    f
}

fn locale_ok(text: &str) -> bool {
    text == "C.UTF-8"
        || Regex::new(r"^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$")
            .is_ok_and(|re| re.is_match(text))
}

pub fn locale() -> (String, Value) {
    let conf = read(Path::new("/etc/locale.conf"));
    // `. /etc/locale.conf; echo $LANG`: the last LANG= assignment, unquoted.
    let lang = conf
        .lines()
        .filter_map(|l| l.trim().strip_prefix("LANG="))
        .next_back()
        .map(|v| v.trim_matches('"').trim_matches('\'').to_string())
        .unwrap_or_default();
    let lang = if locale_ok(&lang) {
        lang
    } else {
        String::new()
    };
    let supported = read(Path::new("/usr/share/i18n/SUPPORTED"));
    let list: Vec<Value> = supported
        .lines()
        .filter_map(|l| {
            let mut cols = l.split_whitespace();
            let (id, charset) = (cols.next()?, cols.next()?);
            (charset == "UTF-8" && locale_ok(id)).then(|| json!(id))
        })
        .collect();
    (lang, json!(list))
}

pub fn parallel_downloads() -> u64 {
    let conf = read(Path::new("/etc/pacman.conf"));
    let found = conf.lines().find_map(|l| {
        let t = l.trim_start();
        let rest = t
            .strip_prefix("ParallelDownloads")?
            .trim_start()
            .strip_prefix('=')?;
        let digits: String = rest
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        Some(digits)
    });
    let n = found
        .filter(|d| is_uint(d))
        .and_then(|d| d.parse::<u64>().ok())
        .unwrap_or(5);
    if n < 1 {
        5
    } else {
        n.min(20)
    }
}

pub fn keyboard() -> (String, Value) {
    let conf = read(Path::new("/etc/vconsole.conf"));
    let layout = conf
        .lines()
        .find_map(|l| {
            let (key, value) = l.split_once('=')?;
            (key == "XKBLAYOUT").then(|| {
                value
                    .chars()
                    .filter(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
                    .collect::<String>()
            })
        })
        .unwrap_or_default();
    let layout = layout.split(',').next().unwrap_or("").to_string();
    let layout = if Regex::new(r"^[a-z0-9]{1,8}$").is_ok_and(|re| re.is_match(&layout)) {
        layout
    } else {
        String::new()
    };
    let mut layouts = Vec::new();
    let lst = read(Path::new("/usr/share/X11/xkb/rules/evdev.lst"));
    let mut inside = false;
    let id_re = Regex::new(r"^[a-z0-9]{1,8}$").expect("regex");
    for line in lst.lines() {
        if line.trim_end() == "! layout" {
            inside = true;
            continue;
        }
        if inside && line.starts_with("! ") {
            break;
        }
        if !inside {
            continue;
        }
        let mut words = line.split_whitespace();
        let Some(id) = words.next().filter(|w| id_re.is_match(w)) else {
            continue;
        };
        let label: Vec<&str> = words.collect();
        if !label.is_empty() {
            layouts.push(json!({"value": id, "label": label.join(" ")}));
        }
    }
    (layout, json!(layouts))
}

pub fn crash_capture() -> bool {
    !ok("omarchy", &["toggle", "enabled", "crash-capture-off"])
}

pub struct Control {
    pub fields: Map<String, Value>,
}

/// Workspaces, monitor rules, presentation, env, tweaks, and power extras.
pub fn control(home: &str, scope_all: bool) -> Control {
    use crate::hyprland::{self, Kind};
    let windows_file = crate::host::env_path(
        "ATMOS_WINDOWS_FILE",
        format!("{home}/.config/hypr/atmos.lua"),
    );
    let monitors_file = crate::host::env_path(
        "ATMOS_MONITORS_FILE",
        format!("{home}/.config/hypr/monitors.lua"),
    );
    let env_file = crate::host::env_path(
        "ATMOS_ENV_FILE",
        format!("{home}/.config/environment.d/10-atmos.conf"),
    );
    let gtk_file = crate::host::env_path(
        "ATMOS_GTK4_FILE",
        format!("{home}/.config/gtk-4.0/settings.ini"),
    );
    let presentation = crate::host::env_path(
        "ATMOS_PRESENTATION_FILE",
        format!("{home}/.local/state/omarchy/atmos-presentation.json"),
    );
    let list = |kind, file: &Path| hyprland::list_file(kind, file).unwrap_or(json!([]));
    let mut f = Map::new();
    f.insert("workspaces".into(), list(Kind::Workspaces, &windows_file));
    f.insert("monitorRules".into(), list(Kind::Monitors, &monitors_file));
    f.insert(
        "workspacesManaged".into(),
        json!(file_has(&windows_file, Kind::Workspaces.begin())),
    );
    f.insert(
        "workspaceWrapSwitch".into(),
        json!(!file_has(&windows_file, "-- atmos:wrapSwitch = false")),
    );
    f.insert(
        "workspaceWheelSwitch".into(),
        json!(!file_has(&windows_file, "-- atmos:wheelSwitch = false")),
    );
    f.insert(
        "monitorRulesManaged".into(),
        json!(file_has(&monitors_file, Kind::Monitors.begin())),
    );
    let on = serde_json::from_str::<Value>(&read(&presentation))
        .ok()
        .and_then(|v| v.get("on").cloned())
        .is_some_and(|v| v == Value::Bool(true));
    f.insert("presentationMode".into(), json!(on));
    f.insert("envVars".into(), json!([]));
    f.insert("envPathPrepend".into(), json!(""));
    f.insert("envDetected".into(), json!({}));
    f.insert("tweaks".into(), json!({"middlePaste": false, "electronWayland": true, "forceZeroScaling": true, "swappiness": false}));
    f.insert("systemdUnits".into(), json!([]));
    f.insert("powerGovernor".into(), json!(""));
    f.insert("amdPstate".into(), json!(""));
    f.insert("chargeLimit".into(), json!(0));
    f.insert("chargeLimitAvailable".into(), json!(false));
    f.insert("netGateway".into(), json!(""));
    f.insert("netDnsServers".into(), json!([]));
    if scope_all {
        let env_text = read(&env_file);
        f.insert("envPathPrepend".into(), json!(env_path_prepend(&env_text)));
        f.insert("envVars".into(), env_vars(&env_text));
        f.insert("envDetected".into(), env_detected());
        let gtk = read(&gtk_file);
        f.insert(
            "tweaks".into(),
            json!({
                "middlePaste": gtk.contains("gtk-enable-primary-paste=false"),
                "electronWayland": !env_text.lines().any(|l| l.starts_with("ELECTRON_OZONE_PLATFORM_HINT=auto")),
                "forceZeroScaling": !env_text.lines().any(|l| l.starts_with("ATMOS_XWAYLAND_ZERO_SCALING=0")),
                "swappiness": Path::new("/etc/sysctl.d/99-atmos-swappiness.conf").is_file(),
            }),
        );
        if present("systemctl") {
            f.insert("systemdUnits".into(), crate::inventory::services::collect());
        }
        f.insert(
            "powerGovernor".into(),
            json!(first_line(Path::new(
                "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"
            ))),
        );
        f.insert(
            "amdPstate".into(),
            json!(first_line(Path::new(
                "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference"
            ))),
        );
        let (limit, available) = charge_limit();
        f.insert("chargeLimit".into(), json!(limit));
        f.insert("chargeLimitAvailable".into(), json!(available));
        if present("nmcli") {
            f.insert(
                "netGateway".into(),
                json!(out("nmcli", &["-g", "IP4.GATEWAY", "device", "show"])
                    .lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("")),
            );
            f.insert(
                "netDnsServers".into(),
                json!(out("nmcli", &["-g", "IP4.DNS", "device", "show"])
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .collect::<Vec<_>>()),
            );
        }
    }
    Control { fields: f }
}

fn env_path_prepend(text: &str) -> String {
    text.lines()
        .find_map(|l| {
            let value = l.strip_prefix("PATH=")?;
            let value = value.split('=').next().unwrap_or(value);
            let value = value
                .strip_suffix(":$PATH")
                .or_else(|| value.strip_suffix(":${PATH}"))
                .unwrap_or(value);
            Some(value.to_string())
        })
        .unwrap_or_default()
}

fn env_vars(text: &str) -> Value {
    let name = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").expect("regex");
    json!(text
        .lines()
        .filter(|l| !l.starts_with('#'))
        .filter_map(|l| {
            let (key, value) = l.split_once('=')?;
            (key != "PATH" && name.is_match(key)).then(|| json!({"key": key, "value": value}))
        })
        .collect::<Vec<_>>())
}

fn env_detected() -> Value {
    let get = |name: &str| std::env::var(name).unwrap_or_default();
    json!({
        "sessionType": get("XDG_SESSION_TYPE"),
        "desktop": get("XDG_CURRENT_DESKTOP"),
        "path": get("PATH"),
        "xdgHome": get("XDG_DATA_HOME"),
        "xdgConfig": get("XDG_CONFIG_HOME"),
        "xdgData": get("XDG_DATA_HOME"),
        "editor": get("EDITOR"),
        "browser": get("BROWSER"),
        "terminal": get("TERMINAL"),
        "shell": get("SHELL"),
    })
}

/// Sysfs charge end-threshold. Dell exposes the stored value in Adaptive, where
/// firmware ignores it. The kernel marks the active type in [brackets], so
/// " Custom " means Custom is offered but not selected; "[Custom]" is left alone.
pub fn charge_limit() -> (u64, bool) {
    let dir = std::env::var("ATMOS_POWER_SUPPLY_DIR")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/sys/class/power_supply".into());
    charge_limit_at(Path::new(&dir))
}

fn charge_limit_at(dir: &Path) -> (u64, bool) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    entries.sort();
    for supply in entries {
        let threshold = supply.join("charge_control_end_threshold");
        let Ok(raw) = std::fs::read_to_string(&threshold) else {
            continue;
        };
        let raw = raw.trim_end_matches('\n');
        let mut limit = if is_uint(raw) {
            raw.parse().unwrap_or(0)
        } else {
            0
        };
        if let Ok(types) = std::fs::read_to_string(supply.join("charge_types")) {
            if format!(" {} ", types.trim_end_matches('\n')).contains(" Custom ") {
                limit = 0;
            }
        }
        return (limit, true);
    }
    (0, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supply(name: &str, threshold: Option<&str>, types: Option<&str>) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ratmos-charge-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let bat = dir.join("BAT0");
        std::fs::create_dir_all(&bat).unwrap();
        if let Some(t) = threshold {
            std::fs::write(bat.join("charge_control_end_threshold"), t).unwrap();
        }
        if let Some(t) = types {
            std::fs::write(bat.join("charge_types"), t).unwrap();
        }
        dir
    }

    #[test]
    fn an_adaptive_dell_hides_its_stored_threshold() {
        let dir = supply(
            "adaptive",
            Some("80\n"),
            Some("Trickle Fast Standard [Adaptive] Custom\n"),
        );
        assert_eq!(charge_limit_at(&dir), (0, true));
    }

    #[test]
    fn other_batteries_report_the_limit_they_hold() {
        for (name, types) in [
            ("custom-active", Some("Standard Adaptive [Custom]\n")),
            ("custom-first", Some("[Custom] Adaptive\n")),
            ("no-custom", Some("[Standard] Fast\n")),
            ("threshold-only", None),
        ] {
            let dir = supply(name, Some("80\n"), types);
            assert_eq!(charge_limit_at(&dir), (80, true), "{name}");
        }
    }

    #[test]
    fn a_missing_or_garbled_threshold_is_no_limit() {
        assert_eq!(
            charge_limit_at(&supply("garbled", Some("not-a-number\n"), None)),
            (0, true)
        );
        assert_eq!(charge_limit_at(&supply("absent", None, None)), (0, false));
    }
}
