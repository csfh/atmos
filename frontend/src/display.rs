//! The Displays page's logic (`pages/DisplaysPage.qml`, `RichUi.js`'s monitor helpers): each
//! monitor's modes, rule and texts, and the writes its rows send (`monitors.rs` patches a rule
//! and makes a layout's). A rule write (`Omarchy.writeMonitorRules`), the focused monitor's scale
//! and turning the touchpad off are guarded: the bar puts them back unless kept. `tests/catalog_display.rs` runs the QML
//! page's and the helpers' code, and `tests/catalog_setters.rs` the setters, under Node.

use serde_json::{Value, json};

use crate::catalog::Field;
use crate::js;

/// What Hyprland reports of each output.
pub const MONITORS: Field = Field::Key("monitors");
/// The rules Atmos writes in `monitors.lua`.
pub const RULES: Field = Field::Key("monitorRules");

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

fn n(v: &Value, k: &str) -> f64 {
    js::number(v.get(k))
}

fn list(v: Option<&Value>) -> Vec<Value> {
    v.and_then(Value::as_array).cloned().unwrap_or_default()
}

/// `parseMonitorMode`: `W x H @ R Hz`.
pub fn parse_mode(raw: &str) -> Option<(f64, f64, f64)> {
    let t = raw.trim();
    let t = t
        .strip_suffix("Hz")
        .or_else(|| t.strip_suffix("hz"))
        .or_else(|| t.strip_suffix("HZ"))?
        .trim_end();
    let (size, rate) = t.split_once('@')?;
    let (w, h) = size.trim().split_once(['x', '×'])?;
    let digits = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
    let (w, h, rate) = (w.trim(), h.trim(), rate.trim());
    if !digits(w)
        || !digits(h)
        || rate.is_empty()
        || !rate.chars().all(|c| c.is_ascii_digit() || c == '.')
    {
        return None;
    }
    let (w, h, r): (f64, f64, f64) = (w.parse().ok()?, h.parse().ok()?, rate.parse().ok()?);
    (w > 0.0 && h > 0.0 && r >= 0.0).then_some((w, h, r))
}

/// `formatMonitorHz`.
pub fn format_hz(refresh: f64) -> String {
    if !refresh.is_finite() || refresh < 0.0 {
        return String::new();
    }
    let r = js::round(refresh);
    if (refresh - r).abs() < 0.05 {
        return js::string(&js::json_number(r));
    }
    js::string(&js::json_number(js::round(refresh * 100.0) / 100.0))
}

/// `formatMonitorMode`.
pub fn format_mode(raw: &str) -> String {
    let Some((w, h, r)) = parse_mode(raw) else {
        return String::new();
    };
    let hz = format_hz(r);
    let size = format!(
        "{}×{}",
        js::string(&js::json_number(w)),
        js::string(&js::json_number(h))
    );
    if hz.is_empty() {
        size
    } else {
        format!("{size} @ {hz} Hz")
    }
}

/// `currentMonitorModeValue`: the reported mode nearest the live one.
pub fn current_mode(m: &Value) -> String {
    let (w, h, r) = (n(m, "width"), n(m, "height"), n(m, "refresh"));
    let mut best = String::new();
    let mut best_diff = 1e9;
    for item in list(m.get("availableModes")) {
        let raw = js::string(&item);
        let Some((pw, ph, pr)) = parse_mode(&raw) else {
            continue;
        };
        if pw != w || ph != h {
            continue;
        }
        let diff = (pr - r).abs();
        if diff < best_diff {
            best_diff = diff;
            best = raw;
        }
    }
    if !best.is_empty() {
        return best;
    }
    if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
        let r = if r.is_finite() && r > 0.0 { r } else { 0.0 };
        let num = |x: f64| js::string(&js::json_number(x));
        return format!("{}x{}@{}Hz", num(w), num(h), num(r));
    }
    String::new()
}

/// `monitorModeCopyText`.
pub fn copy_text(m: &Value) -> String {
    let label = format_mode(&current_mode(m));
    let name = s(m, "name");
    match (label.is_empty(), name.is_empty()) {
        (false, false) => format!("{name} {label}"),
        (false, true) => label,
        _ => name,
    }
}

const STANDARD: &[(u32, u32)] = &[
    (5120, 1440),
    (3440, 1440),
    (3840, 2160),
    (2560, 1600),
    (2560, 1440),
    (2560, 1080),
    (1920, 1200),
    (1920, 1080),
    (1680, 1050),
    (1600, 1200),
    (1600, 900),
    (1440, 900),
    (1400, 1050),
    (1366, 768),
    (1280, 1024),
    (1280, 960),
    (1280, 800),
    (1280, 720),
    (1152, 864),
    (1024, 768),
    (960, 540),
    (854, 480),
    (800, 600),
    (640, 480),
    (640, 360),
];

fn live_size(m: &Value) -> Option<(f64, f64)> {
    let (w, h) = (js::round(n(m, "width")), js::round(n(m, "height")));
    (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0).then_some((w, h))
}

pub(crate) fn key(w: f64, h: f64) -> String {
    format!(
        "{}x{}",
        js::string(&js::json_number(w)),
        js::string(&js::json_number(h))
    )
}

/// `monitorResolutions(monitor)`: every size, largest first.
pub fn resolutions(m: &Value) -> Vec<(String, String)> {
    let mut found: Vec<(f64, f64)> = Vec::new();
    for item in list(m.get("availableModes")) {
        if let Some((w, h, _)) = parse_mode(&js::string(&item))
            && !found.contains(&(w, h))
        {
            found.push((w, h));
        }
    }
    if let Some(live) = live_size(m)
        && !found.contains(&live)
    {
        found.push(live);
    }
    let (mw, mh) = found
        .iter()
        .fold((0.0f64, 0.0f64), |(a, b), (w, h)| (a.max(*w), b.max(*h)));
    if mw > 0.0 && mh > 0.0 {
        for (w, h) in STANDARD {
            let (w, h) = (f64::from(*w), f64::from(*h));
            if !found.contains(&(w, h)) && w <= mw && h <= mh {
                found.push((w, h));
            }
        }
    }
    found.sort_by(|a, b| {
        (b.0 * b.1)
            .partial_cmp(&(a.0 * a.1))
            .unwrap()
            .then(b.0.partial_cmp(&a.0).unwrap())
    });
    found
        .into_iter()
        .map(|(w, h)| {
            (
                key(w, h),
                format!(
                    "{}×{}",
                    js::string(&js::json_number(w)),
                    js::string(&js::json_number(h))
                ),
            )
        })
        .collect()
}

/// `currentMonitorResolutionValue`.
pub fn current_resolution(m: &Value) -> String {
    if let Some((w, h)) = live_size(m) {
        return key(w, h);
    }
    parse_mode(&current_mode(m)).map_or_else(String::new, |(w, h, _)| key(w, h))
}

fn rate_value(r: f64) -> String {
    if !r.is_finite() || r <= 0.0 {
        String::new()
    } else {
        js::string(&js::json_number(js::round(r * 100.0) / 100.0))
    }
}

fn parse_resolution(raw: &str) -> Option<(f64, f64)> {
    let (w, h) = raw.trim().split_once(['x', '×'])?;
    let digits = |p: &str| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit());
    let (w, h) = (w.trim(), h.trim());
    if !digits(w) || !digits(h) {
        return None;
    }
    let (w, h): (f64, f64) = (w.parse().ok()?, h.parse().ok()?);
    (w > 0.0 && h > 0.0).then_some((w, h))
}

/// `monitorRefreshRates(monitor, resolution)`: fastest first, 60 Hz when none is known.
pub fn refresh_rates(m: &Value, resolution: &str) -> Vec<(String, String)> {
    let Some((rw, rh)) = parse_resolution(resolution) else {
        return Vec::new();
    };
    let mut found: Vec<(String, f64)> = Vec::new();
    for item in list(m.get("availableModes")) {
        if let Some((w, h, r)) = parse_mode(&js::string(&item))
            && w == rw
            && h == rh
        {
            let v = rate_value(r);
            if !v.is_empty() && !found.iter().any(|(x, _)| *x == v) {
                found.push((v, r));
            }
        }
    }
    let live = rate_value(n(m, "refresh"));
    if !live.is_empty() && !found.iter().any(|(x, _)| *x == live) {
        found.push((live, n(m, "refresh")));
    }
    found.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let out: Vec<(String, String)> = found
        .into_iter()
        .map(|(v, r)| (v, format!("{} Hz", format_hz(r))))
        .collect();
    if out.is_empty() {
        vec![("60".into(), "60 Hz".into())]
    } else {
        out
    }
}

/// `currentMonitorRefreshValue`: the rate nearest the live one.
pub fn current_refresh(m: &Value, resolution: &str) -> String {
    let rates = refresh_rates(m, resolution);
    let Some(first) = rates.first() else {
        return String::new();
    };
    let live = n(m, "refresh");
    if !live.is_finite() || live <= 0.0 {
        return first.0.clone();
    }
    let mut best = first.0.clone();
    let mut best_diff = 1e9;
    for (v, _) in &rates {
        let diff = (v.parse::<f64>().unwrap_or(f64::NAN) - live).abs();
        if diff < best_diff {
            best_diff = diff;
            best = v.clone();
        }
    }
    best
}

/// `monitorTitle`.
pub fn title(m: &Value) -> String {
    let name = match s(m, "name") {
        x if x.is_empty() => "Display".to_owned(),
        x => x,
    };
    let desc = s(m, "description");
    if desc.is_empty() {
        return name;
    }
    let short: String = if desc.contains(' ') && desc.encode_utf16().count() > 28 {
        desc.chars().take(28).collect()
    } else {
        desc
    };
    format!("{name}  {short}")
}

/// `monitorSummary`.
pub fn summary(m: &Value) -> String {
    let mut size = format_mode(&current_mode(m));
    if size.is_empty() {
        size = format!(
            "{}×{}",
            m.get("width").map_or("undefined".into(), js::string),
            m.get("height").map_or("undefined".into(), js::string)
        );
    }
    let mut extra = Vec::new();
    if m.get("focused").is_some_and(|v| js::truthy(Some(v))) {
        extra.push("focused".to_owned());
    }
    if !js::truthy(m.get("enabled")) {
        extra.push("disabled".into());
    }
    let mirror = s(m, "mirrorOf");
    if !mirror.is_empty() {
        extra.push(format!("mirroring {mirror}"));
    }
    let tail = if extra.is_empty() {
        ".".to_owned()
    } else {
        format!(". {}.", extra.join(", "))
    };
    format!("{size}{tail}")
}

/// `resolutionDescription`.
pub fn resolution_says(m: &Value) -> String {
    let count = resolutions(m).len();
    let modes = if count > 1 {
        format!("{count} resolutions on this output. ")
    } else {
        String::new()
    };
    format!(
        "{} {modes}Picking one writes a monitor rule in ~/.config/hypr/monitors.lua. The refresh rate behind it stays.",
        summary(m)
    )
}

/// `ruleFor`: this monitor's rule, if it has one.
pub fn rule_for<'a>(m: &Value, rules: &'a [Value]) -> Option<&'a Value> {
    let name = s(m, "name");
    rules
        .iter()
        .find(|r| r.get("output").map(js::string).unwrap_or_default() == name)
}

pub fn rule_transform(m: &Value, rules: &[Value]) -> String {
    if let Some(t) = rule_for(m, rules)
        .and_then(|r| r.get("transform"))
        .filter(|t| !t.is_null())
    {
        return js::string(t);
    }
    let t = js::round(n(m, "transform"));
    js::string(&js::json_number(if t.is_nan() { 0.0 } else { t }))
}

pub fn rule_disabled(m: &Value, rules: &[Value]) -> bool {
    match rule_for(m, rules) {
        Some(r) => r.get("disabled") == Some(&Value::Bool(true)),
        None => m.get("enabled") == Some(&Value::Bool(false)),
    }
}

fn mirror_source(m: &Value, monitors: &[Value], rules: &[Value]) -> bool {
    let name = s(m, "name");
    !name.is_empty()
        && monitors
            .iter()
            .any(|o| s(o, "mirrorOf") == name && !rule_disabled(o, rules))
}

/// `canDisable`: something else stays on, and nothing mirrors this one.
pub fn can_disable(m: &Value, monitors: &[Value], rules: &[Value]) -> bool {
    if s(m, "name").is_empty() {
        return false;
    }
    if rule_disabled(m, rules) {
        return true;
    }
    if mirror_source(m, monitors, rules) {
        return false;
    }
    monitors.iter().filter(|o| !rule_disabled(o, rules)).count() > 1
}

pub fn disable_says(m: &Value, monitors: &[Value], rules: &[Value]) -> &'static str {
    if rule_disabled(m, rules) {
        "Off for now. Turn it back on when you need it."
    } else if mirror_source(m, monitors, rules) {
        "Another display is mirroring this one. Unmirror it first."
    } else if !can_disable(m, monitors, rules) {
        "This is the only display on, so it has to stay on."
    } else {
        "Keeps the rule so you can turn it back on. Does not delete the output from the file."
    }
}

pub fn rule_vrr(m: &Value, rules: &[Value]) -> String {
    if let Some(v) = rule_for(m, rules)
        .and_then(|r| r.get("vrr"))
        .filter(|v| !v.is_null())
    {
        return js::string(v);
    }
    let v = js::round(n(m, "vrr"));
    js::string(&js::json_number(if v.is_nan() { 0.0 } else { v }))
}

pub fn rule_bitdepth(m: &Value, rules: &[Value]) -> &'static str {
    if rule_for(m, rules)
        .and_then(|r| r.get("bitdepth"))
        .and_then(Value::as_f64)
        == Some(10.0)
    {
        "10"
    } else {
        "8"
    }
}

pub fn rule_cm(m: &Value, rules: &[Value]) -> String {
    rule_for(m, rules).map_or_else(String::new, |r| s(r, "cm"))
}

pub fn vrr_available(m: &Value) -> bool {
    m.get("vrr") == Some(&Value::Bool(true))
        || n(m, "vrr") > 0.0
        || m.get("availableVrr") == Some(&Value::Bool(true))
        || !list(m.get("availableModes")).is_empty()
}

pub fn hdr_available(m: &Value, rules: &[Value]) -> bool {
    let cm = match s(m, "currentFormat") {
        x if x.is_empty() => s(m, "cm"),
        x => x,
    }
    .to_lowercase();
    cm.contains("hdr")
        || cm.contains("bt2020")
        || cm.contains("10")
        || m.get("hdr") == Some(&Value::Bool(true))
        || rule_cm(m, rules) == "hdr"
}

pub fn color_says(hdr: bool) -> &'static str {
    if hdr {
        "This output can take HDR. Auto leaves Hyprland's default."
    } else {
        "Color profile for this output. HDR stays off unless the panel reports it."
    }
}

pub const SCALES: &[(&str, &str)] = &[
    ("1", "100%"),
    ("1.25", "125%"),
    ("1.6", "160%"),
    ("2", "200%"),
    ("3", "300%"),
    ("4", "400%"),
];
pub const TRANSFORMS: &[(&str, &str)] =
    &[("0", "Normal"), ("1", "90°"), ("2", "180°"), ("3", "270°")];
pub const VRRS: &[(&str, &str)] = &[
    ("0", "Off"),
    ("1", "On"),
    ("2", "Fullscreen"),
    ("3", "Fullscreen games"),
];
pub const DEPTHS: &[(&str, &str)] = &[("8", "8-bit"), ("10", "10-bit")];
pub const COLORS: &[(&str, &str)] = &[
    ("", "Auto"),
    ("srgb", "sRGB"),
    ("wide", "Wide"),
    ("dcip3", "DCI-P3"),
    ("hdr", "HDR"),
    ("hdredid", "HDR EDID"),
];

/// `scaleValue`.
pub fn scale_value(m: &Value) -> String {
    let v = n(m, "scale");
    if !v.is_finite() || v <= 0.0 {
        "1".into()
    } else {
        js::string(&js::json_number(v))
    }
}

/// `scaleOptions`: the presets, and the scale in use when it is none of them.
pub fn scale_options(m: &Value) -> Vec<(String, String)> {
    let current = scale_value(m);
    let mut out: Vec<(String, String)> = SCALES
        .iter()
        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
        .collect();
    if !out.iter().any(|(v, _)| *v == current) {
        out.push((current.clone(), current));
    }
    out
}

/// `writeMonitorRules(items)`: the write, and the guard that puts the rules shown back.
pub fn rules_write(current: Option<&Value>, items: Vec<Value>) -> (Value, Value) {
    let body = crate::lists::write("monitorRules", current, items);
    let revert = json!({ "kind": "monitorRules", "rules": current.cloned().filter(Value::is_array).unwrap_or_else(|| json!([])) });
    (body, revert)
}

/// The Scale row of a monitor that is not focused: its rule with the new scale.
pub fn scale_rules(m: &Value, rules: &[Value], value: &str) -> Vec<Value> {
    let name = s(m, "name");
    let scale = js::json_number(value.parse().unwrap_or(f64::NAN));
    let mut found = false;
    let mut out: Vec<Value> = rules
        .iter()
        .map(|r| {
            let mut row = if r.is_object() { r.clone() } else { json!({}) };
            if r.get("output").map(js::string).unwrap_or_default() == name {
                row["scale"] = scale.clone();
                found = true;
            }
            row
        })
        .collect();
    if !found {
        out.push(json!({
            "output": name, "mode": "preferred", "position": "auto", "scale": scale, "transform": 0,
            "disabled": m.get("enabled") == Some(&Value::Bool(false)), "vrr": 0, "bitdepth": 8, "cm": "",
        }));
    }
    out
}

/// `setMonitorScale(scale)`: the focused monitor's, with the guard that puts its scale back.
pub fn scale_write(value: &str, monitors: &[Value]) -> Option<(Value, Option<Value>)> {
    let v: f64 = value.parse().unwrap_or(f64::NAN);
    if value.is_empty() || !v.is_finite() || v <= 0.0 {
        return None;
    }
    let focused = monitors
        .iter()
        .find(|m| m.get("focused") == Some(&Value::Bool(true)));
    if focused.is_some_and(|m| n(m, "scale") == v) {
        return None;
    }
    let previous = focused
        .map(|m| m.get("scale").map_or("undefined".into(), js::string))
        .filter(|p| !p.is_empty());
    let body = json!({ "op": "display.scale", "scale": value });
    Some((
        body,
        previous.map(|p| json!({ "kind": "monitorScale", "scale": p })),
    ))
}

/// `setDisplayBrightness(name, percent)`.
pub fn brightness_write(name: &str, percent: f64) -> Option<Value> {
    let p = js::round(percent);
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        && (0.0..=100.0).contains(&p);
    ok.then(
        || json!({ "op": "display.brightness", "monitor": name, "percent": js::json_number(p) }),
    )
}

/// `setTouchpad(on)`: the write, and the guard that turns it back on when it goes off.
pub fn touchpad_write(on: bool, current: Option<&Value>) -> Option<(Value, Option<Value>)> {
    (current != Some(&Value::Bool(on))).then(|| {
        let body = crate::settings::write("touchpadEnabled", &Value::Bool(on));
        (body, (!on).then(|| json!({ "kind": "touchpad" })))
    })
}

/// `adjustKeyboardBacklight(direction)`.
pub fn backlight_write(direction: &str) -> Option<Value> {
    match direction {
        "restore" => Some(crate::protocol::write(
            "keyboard.backlight-restore",
            &json!({}),
        )),
        "up" | "down" | "off" => Some(crate::protocol::write(
            "keyboard.backlight",
            &json!({ "direction": direction }),
        )),
        _ => None,
    }
}

/// `writeMonitorMode(monitor, resolution, refresh)`'s patch: the mode, and the output on.
pub fn mode_patch(resolution: &str, refresh: &str) -> Option<Value> {
    let mode = crate::monitors::sanitize_mode(&format!("{resolution}@{refresh}"));
    (!mode.is_empty()).then(|| json!({ "mode": mode, "disabled": false }))
}

/// `editMonitorsLua()`: opens `~/.config/hypr/monitors.lua` of `home`.
pub fn edit_write(home: &str) -> Option<Value> {
    let path = format!("{home}/.config/hypr/monitors.lua");
    (!home.is_empty() && !path.contains(".."))
        .then(|| crate::protocol::write("open.path", &json!({ "path": path })))
}

/// The keyboard backlight row's words (`keyboardBrightness > 0`).
pub fn backlight_says(brightness: f64) -> String {
    if brightness > 0.0 {
        format!(
            "The keys are at {}%.",
            js::string(&js::json_number(brightness))
        )
    } else {
        "Step the keyboard LEDs up or down, or turn them off.".into()
    }
}
