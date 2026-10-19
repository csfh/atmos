//! The live Hyprland surface: option values, monitors, input devices, layout,
//! and the managed-block markers.

use super::sh::{
    self, alt, file_has, first_line, is_uint, jq_text, ok, omarchy, out, present, read,
};
use crate::hyprland::{self, Kind};
use regex_lite::Regex;
use serde_json::{json, Map, Value};
use std::path::Path;

/// `hyprctl -j getoption NAME`, or {} when it is not JSON.
fn option(name: &str) -> Value {
    let raw = out("hyprctl", &["-j", "getoption", name]);
    serde_json::from_str::<Value>(&raw)
        .ok()
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

/// jq `num(o; fb)`: int, float, or the first word of css, else the fallback.
fn num(o: &Value, fallback: Value, css: bool) -> Value {
    if let Some(v) = alt_nonnull(o.get("int")) {
        return v.clone();
    }
    if let Some(v) = alt_nonnull(o.get("float")) {
        return v.clone();
    }
    if css {
        if let Some(Value::String(s)) = o.get("css") {
            if let Some(n) = s.split(' ').next().and_then(|w| w.parse::<f64>().ok()) {
                return number(n);
            }
        }
    }
    fallback
}

fn alt_nonnull(v: Option<&Value>) -> Option<&Value> {
    v.filter(|v| !v.is_null())
}

fn number(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9e15 {
        json!(n as i64)
    } else {
        json!(n)
    }
}

fn flag(o: &Value, fallback: bool) -> Value {
    json!(o.get("bool").and_then(Value::as_bool).unwrap_or(fallback))
}

fn txt(o: &Value, fallback: &str) -> Value {
    match o.get("str") {
        Some(Value::String(s)) if s != "[[EMPTY]]" => json!(s),
        _ => json!(fallback),
    }
}

pub fn look() -> Value {
    let raw = std::env::var("HYPRCURSOR_SIZE")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var("XCURSOR_SIZE").ok().filter(|v| !v.is_empty()))
        .unwrap_or_else(|| "24".into());
    let size: i64 = if is_uint(&raw) {
        raw.parse().unwrap_or(24)
    } else {
        24
    }
    .clamp(8, 64);
    let warp = num(&option("cursor:warp_on_change_workspace"), json!(1), true);
    json!({
        "gapsIn": num(&option("general:gaps_in"), json!(5), true),
        "gapsOut": num(&option("general:gaps_out"), json!(10), true),
        "borderSize": num(&option("general:border_size"), json!(2), true),
        "rounding": num(&option("decoration:rounding"), json!(0), true),
        "blur": flag(&option("decoration:blur:enabled"), false),
        "shadow": flag(&option("decoration:shadow:enabled"), false),
        "layout": txt(&option("general:layout"), "dwindle"),
        "columnWidth": num(&option("scrolling:column_width"), json!(0.49), true),
        "dimInactive": flag(&option("decoration:dim_inactive"), false),
        "dimStrength": num(&option("decoration:dim_strength"), json!(0.15), true),
        "animations": flag(&option("animations:enabled"), true),
        "cursorHideOnKey": flag(&option("cursor:hide_on_key_press"), true),
        "cursorWarp": warp.as_f64().is_none_or(|n| n != 0.0),
        "cursorSize": size,
        "allowTearing": flag(&option("general:allow_tearing"), false),
        "resizeOnBorder": flag(&option("general:resize_on_border"), false),
        "activeOpacity": num(&option("decoration:active_opacity"), json!(1), true),
        "inactiveOpacity": num(&option("decoration:inactive_opacity"), json!(1), true),
        "preserveSplit": flag(&option("dwindle:preserve_split"), false),
        "focusOnActivate": flag(&option("misc:focus_on_activate"), false),
        "enableSwallow": flag(&option("misc:enable_swallow"), false),
        "swallowRegex": txt(&option("misc:swallow_regex"), ""),
        "onFocusUnderFullscreen": num(&option("misc:on_focus_under_fullscreen"), json!(1), true),
    })
}

pub fn input(gesture: bool) -> Value {
    let layout = txt(&option("input:kb_layout"), "");
    let variant = txt(&option("input:kb_variant"), "");
    let options = txt(&option("input:kb_options"), "");
    let toggle = options
        .as_str()
        .is_some_and(|o| o.contains("grp:alts_toggle"));
    json!({
        "sensitivity": num(&option("input:sensitivity"), json!(0), false),
        "accelProfile": txt(&option("input:accel_profile"), ""),
        "emulateDiscreteScroll": num(&option("input:emulate_discrete_scroll"), json!(1), false),
        "naturalScroll": flag(&option("input:touchpad:natural_scroll"), false),
        "scrollFactor": num(&option("input:touchpad:scroll_factor"), json!(0.4), false),
        "clickfinger": flag(&option("input:touchpad:clickfinger_behavior"), true),
        "disableWhileTyping": flag(&option("input:touchpad:disable_while_typing"), true),
        "drag3fg": num(&option("input:touchpad:drag_3fg"), json!(0), false),
        "repeatRate": num(&option("input:repeat_rate"), json!(40), false),
        "repeatDelay": num(&option("input:repeat_delay"), json!(250), false),
        "numlock": flag(&option("input:numlock_by_default"), true),
        "followMouse": num(&option("input:follow_mouse"), json!(1), false),
        "keyPressDpms": flag(&option("misc:key_press_enables_dpms"), true),
        "mouseMoveDpms": flag(&option("misc:mouse_move_enables_dpms"), true),
        "kbLayout": layout,
        "kbVariant": variant,
        "kbOptions": options,
        "kbLayoutOverride": layout,
        "kbVariantOverride": variant,
        "kbGroupToggle": toggle,
        "workspaceGesture": gesture,
    })
}

/// The tiled layout of the active workspace: scrolling or dwindle.
pub fn workspace_layout() -> String {
    let raw = out("hyprctl", &["activeworkspace", "-j"]);
    let layout = serde_json::from_str::<Value>(&raw)
        .ok()
        .and_then(|v| alt(v.get("tiledLayout")).map(jq_text))
        .unwrap_or_else(|| "dwindle".into());
    if layout == "scrolling" {
        layout
    } else {
        "dwindle".into()
    }
}

fn monitor_row(m: &Value) -> Value {
    let name = alt(m.get("name")).map(jq_text).unwrap_or_default();
    let floor = |key: &str| {
        m.get(key)
            .and_then(Value::as_f64)
            .map_or(0, |f| f.floor() as i64)
    };
    let mirror = match alt(m.get("mirrorOf")) {
        Some(Value::String(s)) if s == "none" => String::new(),
        Some(v) => jq_text(v),
        None => String::new(),
    };
    let modes: Vec<Value> = alt(m.get("availableModes"))
        .and_then(Value::as_array)
        .map(|a| a.iter().take(80).map(|v| json!(jq_text(v))).collect())
        .unwrap_or_default();
    json!({
        "name": name,
        "description": alt(m.get("description")).map(jq_text).unwrap_or_default(),
        "focused": m.get("focused") == Some(&Value::Bool(true)),
        "enabled": m.get("disabled") != Some(&Value::Bool(true)),
        "internal": Regex::new(r"^(eDP|LVDS|DSI)-").is_ok_and(|re| re.is_match(&name)),
        "width": floor("width"),
        "height": floor("height"),
        "refresh": floor("refreshRate"),
        "scale": alt(m.get("scale")).cloned().unwrap_or(json!(1)),
        "x": floor("x"),
        "y": floor("y"),
        "mirrorOf": mirror,
        "availableModes": modes,
        "brightness": 0,
        "brightnessAvailable": false,
    })
}

/// The monitors Hyprland reports. `None` means hyprctl failed, which is not
/// the same as no monitors: the caller leaves the last list alone and the
/// reason goes to stderr.
pub fn monitors() -> Option<Vec<Value>> {
    if !present("hyprctl") {
        return Some(Vec::new());
    }
    let args = ["monitors", "all", "-j"];
    let parsed = match sh::capture("hyprctl", &args) {
        Ok(output) => {
            let raw = output.stdout_text();
            match serde_json::from_str::<Value>(&raw) {
                Ok(Value::Array(rows)) => Ok(rows),
                _ => Err(format!(
                    "exit {}, stderr {:?}, stdout {:?}",
                    output.code,
                    output
                        .stderr_text()
                        .trim()
                        .chars()
                        .take(200)
                        .collect::<String>(),
                    raw.chars().take(80).collect::<String>(),
                )),
            }
        }
        Err(err) => Err(err.to_string()),
    };
    let rows = match parsed {
        Ok(rows) => rows,
        Err(why) => {
            eprintln!("ratmos: hyprctl monitors all -j gave no monitor list ({why}); keeping the last one");
            return None;
        }
    };
    let mut rows: Vec<Value> = rows.iter().map(monitor_row).collect();
    for row in &mut rows {
        let name = row["name"].as_str().unwrap_or("").to_string();
        if !sh::is_ident(&name) {
            continue;
        }
        let shown = out(
            "omarchy",
            &["brightness", "display", "--no-osd", "--monitor", &name],
        );
        let first = shown
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().next())
            .unwrap_or("");
        if is_uint(first) {
            row["brightness"] = json!(first.parse::<u64>().unwrap_or(0).min(100));
            row["brightnessAvailable"] = json!(true);
        }
    }
    Some(rows)
}

pub struct Devices {
    pub fields: Map<String, Value>,
}

pub fn devices(home: &str, rows: &[Value]) -> Devices {
    let state = Path::new(home).join(".local/state/omarchy/toggles/hypr");
    let any = |pred: &dyn Fn(&Value) -> bool| rows.iter().any(pred);
    let mut f = Map::new();
    f.insert(
        "internalPresent".into(),
        json!(any(&|m| m["internal"] == true)),
    );
    f.insert(
        "internalEnabled".into(),
        json!(any(&|m| m["internal"] == true && m["enabled"] == true)),
    );
    f.insert(
        "mirroring".into(),
        json!(any(&|m| m["mirrorOf"]
            .as_str()
            .is_some_and(|s| !s.is_empty()))),
    );
    f.insert(
        "externalPresent".into(),
        json!(ok("omarchy", &["hw", "external", "monitors"])),
    );
    let (mut tp, mut tp_on) = (ok("omarchy", &["hw", "touchpad"]), true);
    if state.join("touchpad-disabled-name").is_file() {
        (tp, tp_on) = (true, false);
    }
    let (mut ts, mut ts_on) = (ok("omarchy", &["hw", "touchscreen"]), true);
    if state.join("touchscreen-disabled-name").is_file() {
        (ts, ts_on) = (true, false);
    }
    f.insert("touchpadPresent".into(), json!(tp));
    f.insert("touchpadEnabled".into(), json!(tp_on));
    f.insert("touchscreenPresent".into(), json!(ts));
    f.insert("touchscreenEnabled".into(), json!(ts_on));
    let (present_kb, brightness) = keyboard_backlight();
    f.insert("keyboardBacklightPresent".into(), json!(present_kb));
    f.insert("keyboardBrightness".into(), json!(brightness));
    f.insert("isLaptop".into(), json!(ok("omarchy", &["hw", "laptop"])));
    f.insert(
        "batteryPresent".into(),
        json!(ok("omarchy", &["battery", "present"])),
    );
    Devices { fields: f }
}

fn keyboard_backlight() -> (bool, u64) {
    let mut dirs: Vec<_> = std::fs::read_dir("/sys/class/leds")
        .map(|rd| rd.flatten().map(|e| e.path()).collect::<Vec<_>>())
        .unwrap_or_default();
    dirs.sort();
    for dir in dirs {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.contains("kbd_backlight")
            || !dir.join("brightness").exists()
            || !dir.join("max_brightness").exists()
        {
            continue;
        }
        let cur = first_line(&dir.join("brightness"));
        let max = first_line(&dir.join("max_brightness"));
        let mut value = 0;
        if is_uint(&cur) && is_uint(&max) && !max.starts_with('0') {
            let (c, m): (u64, u64) = (cur.parse().unwrap_or(0), max.parse().unwrap_or(1));
            value = (c * 100 / m).min(100);
        }
        return (true, value);
    }
    (false, 0)
}

/// Hyprland block markers and toggles for the look surface.
pub fn markers(home: &str) -> Map<String, Value> {
    let hypr = Path::new(home).join(".config/hypr");
    let toggles = Path::new(home).join(".local/state/omarchy/toggles/hypr");
    let mut f = Map::new();
    f.insert(
        "hyprLookManaged".into(),
        json!(file_has(&hypr.join("looknfeel.lua"), Kind::Look.begin())),
    );
    f.insert(
        "hyprNoGaps".into(),
        json!(toggles.join("window-no-gaps.lua").is_file()),
    );
    f.insert(
        "hyprSquareAspect".into(),
        json!(toggles.join("single-window-aspect-ratio.lua").is_file()),
    );
    f.insert("hyprWorkspaceLayout".into(), json!(workspace_layout()));
    f
}

/// The workspace gesture state of an input file.
pub fn gesture(input_file: &Path) -> (bool, bool, bool) {
    if !input_file.is_file() {
        return (false, false, false);
    }
    let state = hyprland::list(Kind::Input, &read(input_file));
    let get = |k: &str| state[k] == Value::Bool(true);
    (
        get("workspaceGesture"),
        get("workspaceGestureManaged"),
        get("workspaceGestureUnmanaged"),
    )
}

pub fn dnd(home: &str) -> bool {
    let state = out("omarchy-shell", &["notifications", "isDnd"]);
    if state == "on" {
        return true;
    }
    if state.is_empty() {
        let file = Path::new(home).join(".local/state/omarchy/notifications.json");
        if file.is_file() {
            return serde_json::from_str::<Value>(&read(&file))
                .ok()
                .and_then(|v| alt(v.get("dnd")).map(jq_text))
                .is_some_and(|s| s == "true");
        }
    }
    false
}

pub fn reminders() -> (u64, bool, Value) {
    let raw = omarchy(&["reminder", "show", "--json"]);
    let doc: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    if raw.is_empty() {
        return (0, false, json!([]));
    }
    let count = alt(doc.get("count"))
        .map(jq_text)
        .unwrap_or_else(|| "0".into());
    let count = if is_uint(&count) {
        count.parse().unwrap_or(0)
    } else {
        0
    };
    let active = alt(doc.get("active"))
        .map(jq_text)
        .is_some_and(|s| s == "true");
    let list = alt(doc.get("reminders")).cloned().unwrap_or(json!([]));
    (count, active, list)
}

/// The nightlight schedule from hyprsunset.conf. Commented profile blocks
/// (Omarchy ships the night example that way) are not live.
pub fn nightlight_conf(home: &str) -> (String, String, bool) {
    let file = crate::host::env_path(
        "ATMOS_HYPRSUNSET_FILE",
        format!("{home}/.config/hypr/hyprsunset.conf"),
    );
    parse_nightlight_conf(&std::fs::read_to_string(&file).unwrap_or_default())
}

pub fn parse_nightlight_conf(raw: &str) -> (String, String, bool) {
    let (mut day, mut night, mut night_on) = ("07:00".to_string(), "20:00".to_string(), false);
    if raw.is_empty() {
        return (day, night, night_on);
    }
    let text: String = raw
        .lines()
        .map(|l| l.split('#').next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let time_re = Regex::new(r#"time\s*=\s*"?([0-2]?\d:[0-5]\d)"?"#).expect("regex");
    let ident_re = Regex::new(r"\bidentity\s*=\s*true\b").expect("regex");
    let temp_re = Regex::new(r"temperature\s*=\s*([0-9]+)").expect("regex");
    let valid = Regex::new(r"^([01]?\d|2[0-3]):([0-5]\d)$").expect("regex");
    let norm = |raw: &str| {
        valid.captures(raw.trim()).map(|m| {
            format!(
                "{:02}:{:02}",
                m[1].parse::<u32>().unwrap_or(0),
                m[2].parse::<u32>().unwrap_or(0)
            )
        })
    };
    let (mut parsed_day, mut parsed_night, mut parsed_on) = (String::new(), String::new(), false);
    let profile = Regex::new(r"profile\s*\{").expect("regex");
    for block in profile.split(&text).skip(1) {
        let body = block.split('}').next().unwrap_or("");
        let Some(time) = time_re.captures(body).and_then(|m| norm(&m[1])) else {
            continue;
        };
        if ident_re.is_match(body) {
            parsed_day = time;
            continue;
        }
        if temp_re.is_match(body) {
            parsed_night = time;
            parsed_on = true;
        }
    }
    if valid.is_match(&parsed_day) {
        day = parsed_day;
    }
    if valid.is_match(&parsed_night) {
        night = parsed_night;
    }
    if parsed_on {
        night_on = true;
    }
    (day, night, night_on)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OMARCHY_DEFAULT: &str = "# Makes hyprsunset do nothing\nprofile {\n    time = 07:00\n    identity = true\n}\n\n# profile {\n#     time = 20:00\n#     temperature = 4000\n# }\n";

    #[test]
    fn a_commented_night_profile_is_not_live() {
        assert_eq!(
            parse_nightlight_conf(OMARCHY_DEFAULT),
            ("07:00".into(), "20:00".into(), false)
        );
    }

    #[test]
    fn a_live_night_profile_is_read_and_padded() {
        let conf = "profile {\n    time = 6:30\n    identity = true\n}\nprofile {\n    time = 21:00\n    temperature = 3800\n}\n";
        assert_eq!(
            parse_nightlight_conf(conf),
            ("06:30".into(), "21:00".into(), true)
        );
    }

    #[test]
    fn inline_comments_and_missing_files_fall_back_to_the_defaults() {
        assert_eq!(
            parse_nightlight_conf("profile {\n time = 6:30 # sunrise\n identity = true\n}\n").0,
            "06:30"
        );
        assert_eq!(
            parse_nightlight_conf(""),
            ("07:00".into(), "20:00".into(), false)
        );
    }
}
