//! An Omafile's lists in the shape the writer applies (`canonicalizeList` and its rows in
//! `Settings.js`, with `RowText.js`'s sanitizers): extra fields, key order and indicator order
//! are not settings, and a row the writer would drop blocks the list, so the review cannot show
//! a list that will not land.

use serde_json::{Map, Value, json};

use super::{Item, number_text, text_or_empty, trim, units};
use crate::bindings::{sanitize_command, sanitize_keys, sanitize_label};
use crate::js;
use crate::rules::{sanitize_match, sanitize_workspace};

/// `normalizeListRow(row)`: an object's keys sorted, without `managed`; anything else as it is.
pub fn normalize_row(row: &Value) -> Value {
    let Value::Object(m) = row else {
        return row.clone();
    };
    let mut keys: Vec<&String> = m.keys().filter(|k| *k != "managed").collect();
    keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    Value::Object(
        keys.into_iter()
            .map(|k| (k.clone(), m[k].clone()))
            .collect(),
    )
}

/// `exportList(list)`: each row normalized.
pub fn export_list(list: &[Value]) -> Vec<Value> {
    list.iter().map(normalize_row).collect()
}

/// `indicatorIds()`.
pub const INDICATORS: [&str; 6] = [
    "Dictation",
    "ScreenRecording",
    "Reminder",
    "NightLight",
    "Dnd",
    "StayAwake",
];

type Row = Result<Value, String>;

fn get<'a>(row: &'a Map<String, Value>, k: &str) -> Option<&'a Value> {
    row.get(k)
}

/// `v != null`.
fn given(v: Option<&Value>) -> bool {
    v.is_some_and(|v| !v.is_null())
}

/// `v != null && v !== ""`.
fn set(v: Option<&Value>) -> bool {
    given(v) && v != Some(&Value::from(""))
}

/// `v != null && v !== true && v !== false`: a flag that is not one.
fn bad_flag(v: Option<&Value>) -> bool {
    given(v) && !v.is_some_and(Value::is_boolean)
}

fn is(v: Option<&Value>, b: bool) -> bool {
    v == Some(&Value::Bool(b))
}

/// `String(v)` of a value that is there.
fn text(v: Option<&Value>) -> String {
    v.map_or_else(|| "undefined".into(), js::string)
}

/// `Math.round(Number(v))`.
fn rounded(v: Option<&Value>) -> f64 {
    js::round(js::number(v))
}

fn object<'a>(row: &'a Value, prefix: &str, what: &str) -> Result<&'a Map<String, Value>, String> {
    match row {
        Value::Object(m) => Ok(m),
        _ => Err(format!("{prefix}is not {what}.")),
    }
}

fn binding(row: &Value, prefix: &str) -> Row {
    let row = object(row, prefix, "a binding")?;
    let keys = sanitize_keys(&text_or_empty(get(row, "keys")));
    if keys.is_empty() {
        return Err(format!("{prefix}is not a valid chord."));
    }
    if bad_flag(get(row, "unbind")) {
        return Err(format!("{prefix}has a bad unbind flag."));
    }
    let unbind = is(get(row, "unbind"), true);
    let mut command = String::new();
    if set(get(row, "command")) {
        let Some(Value::String(c)) = get(row, "command") else {
            return Err(format!("{prefix}has a bad command."));
        };
        command = sanitize_command(c);
        if command.is_empty() {
            return Err(format!("{prefix}has a command Atmos will not write."));
        }
    }
    if command.is_empty() && !unbind {
        return Err(format!("{prefix}needs a command or an unbind."));
    }
    let mut label = String::new();
    if set(get(row, "label")) {
        let Some(Value::String(l)) = get(row, "label") else {
            return Err(format!("{prefix}has a bad label."));
        };
        label = sanitize_label(l);
        if !trim(l).is_empty() && label.is_empty() {
            return Err(format!("{prefix}has a label Atmos will not write."));
        }
    }
    Ok(normalize_row(
        &json!({"keys": keys, "label": label, "command": command, "unbind": unbind}),
    ))
}

fn autostart(row: &Value, prefix: &str) -> Row {
    let row = object(row, prefix, "a startup program")?;
    let Some(Value::String(c)) = get(row, "command") else {
        return Err(format!("{prefix}has no command."));
    };
    let command = sanitize_command(c);
    if command.is_empty() {
        return Err(format!("{prefix}has a command Atmos will not write."));
    }
    let mut delay = 0.0;
    if set(get(row, "delay")) {
        delay = rounded(get(row, "delay"));
        if !delay.is_finite() || !(0.0..=600.0).contains(&delay) {
            return Err(format!("{prefix}has a delay Atmos will not write."));
        }
    }
    if bad_flag(get(row, "enabled")) {
        return Err(format!("{prefix}has a bad enabled flag."));
    }
    Ok(
        json!({"command": command, "delay": js::json_number(delay), "enabled": !is(get(row, "enabled"), false)}),
    )
}

/// `clampSize(raw)`: a window size from 100 to 4000, or 0.
fn clamp_size(raw: Option<&Value>) -> f64 {
    let n = rounded(raw);
    if !n.is_finite() || !(100.0..=4000.0).contains(&n) {
        0.0
    } else {
        n
    }
}

fn window_rule(row: &Value, prefix: &str) -> Row {
    let row = object(row, prefix, "a window rule")?;
    let matches = sanitize_match(&text_or_empty(get(row, "match")));
    if matches.is_empty() {
        return Err(format!("{prefix}has no class to match."));
    }
    let mut placement = if given(get(row, "placement")) {
        text(get(row, "placement"))
    } else {
        String::new()
    };
    if is(get(row, "float"), true) {
        placement = "float".into();
    }
    if is(get(row, "tile"), true) && placement != "float" {
        placement = "tile".into();
    }
    if !matches!(placement.as_str(), "" | "float" | "tile") {
        return Err(format!("{prefix}is not a valid placement."));
    }
    if bad_flag(get(row, "center")) {
        return Err(format!("{prefix}has a bad center flag."));
    }
    let center = is(get(row, "center"), true);
    let (mut width, mut height) = (
        clamp_size(get(row, "width")),
        clamp_size(get(row, "height")),
    );
    let pair = get(row, "size")
        .and_then(Value::as_array)
        .filter(|s| s.len() >= 2);
    if let Some(s) = pair {
        width = clamp_size(s.first());
        height = clamp_size(s.get(1));
    }
    let sent = given(get(row, "width")) || given(get(row, "height")) || pair.is_some();
    let sized = width != 0.0 && height != 0.0;
    if !sized {
        (width, height) = (0.0, 0.0);
    }
    if sent && !sized {
        return Err(format!("{prefix}has a size Atmos will not write."));
    }
    let mut workspace = String::new();
    if set(get(row, "workspace")) {
        workspace = sanitize_workspace(&text_or_empty(get(row, "workspace")));
        if workspace.is_empty() {
            return Err(format!("{prefix}is not a valid workspace."));
        }
    }
    let mut title = String::new();
    if set(get(row, "title")) {
        title = sanitize_match(&text_or_empty(get(row, "title")));
        if title.is_empty() {
            return Err(format!("{prefix}has a title Atmos will not write."));
        }
    }
    if bad_flag(get(row, "pin")) {
        return Err(format!("{prefix}has a bad pin flag."));
    }
    if bad_flag(get(row, "fullscreen")) {
        return Err(format!("{prefix}has a bad fullscreen flag."));
    }
    let mut opacity = String::new();
    if set(get(row, "opacity")) {
        opacity = trim(&text(get(row, "opacity"))).to_owned();
        let shape = match opacity.split_once(' ') {
            Some((a, b)) => [a, b]
                .iter()
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit() || c == '.')),
            None => !opacity.is_empty() && opacity.chars().all(|c| c.is_ascii_digit() || c == '.'),
        };
        if !shape || units(&opacity) > 16 {
            return Err(format!("{prefix}has an opacity Atmos will not write."));
        }
    }
    let (pin, fullscreen) = (is(get(row, "pin"), true), is(get(row, "fullscreen"), true));
    if placement.is_empty()
        && !center
        && width == 0.0
        && workspace.is_empty()
        && title.is_empty()
        && !pin
        && !fullscreen
        && opacity.is_empty()
    {
        return Err(format!("{prefix}does nothing."));
    }
    Ok(normalize_row(&json!({
        "match": matches, "title": title, "placement": placement, "center": center,
        "width": js::json_number(width), "height": js::json_number(height), "workspace": workspace,
        "pin": pin, "fullscreen": fullscreen, "opacity": opacity,
    })))
}

fn word(s: &str, extra: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || extra.contains(c))
}

fn workspace(row: &Value, prefix: &str) -> Row {
    let row = object(row, prefix, "a workspace")?;
    let raw = if js::truthy(get(row, "id")) {
        get(row, "id")
    } else {
        get(row, "workspace")
    };
    let id = trim(&text_or_empty(raw)).to_owned();
    let special = id
        .strip_prefix("special:")
        .is_some_and(|n| (1..=24).contains(&n.len()) && word(n, "_-"));
    let numbered = matches!(
        id.as_str(),
        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10"
    );
    if !special && !numbered {
        return Err(format!("{prefix}is not a valid workspace id."));
    }
    let mut name = String::new();
    if set(get(row, "name")) {
        name = text(get(row, "name"));
        if name.contains('\n') || units(&name) > 32 || !word(&name, " _.-") {
            return Err(format!("{prefix}has a name Atmos will not write."));
        }
    }
    let mut monitor = String::new();
    if set(get(row, "monitor")) {
        monitor = trim(&text(get(row, "monitor"))).to_owned();
        if !word(&monitor, "._-") {
            return Err(format!("{prefix}has a monitor Atmos will not write."));
        }
    }
    let empty = [get(row, "onCreatedEmpty"), get(row, "on_created_empty")]
        .into_iter()
        .find(|v| js::truthy(*v))
        .flatten();
    Ok(normalize_row(&json!({
        "id": id, "name": name, "persistent": !is(get(row, "persistent"), false), "monitor": monitor,
        "isDefault": is(get(row, "isDefault"), true) || is(get(row, "default"), true),
        "special": id.starts_with("special:"),
        "onCreatedEmpty": sanitize_command(&text_or_empty(empty)),
    })))
}

fn digits(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
}

/// `^[0-9]{3,5}x[0-9]{3,5}(@[0-9]+(\.[0-9]+)?)?$`.
fn mode_ok(mode: &str) -> bool {
    let (size, rate) = match mode.split_once('@') {
        Some((s, r)) => (s, Some(r)),
        None => (mode, None),
    };
    let size_ok = size
        .split_once('x')
        .is_some_and(|(w, h)| digits(w, 3, 5) && digits(h, 3, 5));
    let rate_ok = rate.is_none_or(|r| match r.split_once('.') {
        Some((a, b)) => digits(a, 1, usize::MAX) && digits(b, 1, usize::MAX),
        None => digits(r, 1, usize::MAX),
    });
    size_ok && rate_ok
}

/// `^-?[0-9]+x-?[0-9]+$`.
fn position_ok(p: &str) -> bool {
    let signed = |s: &str| digits(s.strip_prefix('-').unwrap_or(s), 1, usize::MAX);
    p.split_once('x')
        .is_some_and(|(a, b)| signed(a) && signed(b))
}

fn monitor(row: &Value, prefix: &str) -> Row {
    let row = object(row, prefix, "a monitor rule")?;
    let output = trim(&text_or_empty(get(row, "output"))).to_owned();
    if !word(&output, "._-") || units(&output) > 64 {
        return Err(format!("{prefix}has no output name."));
    }
    let raw_mode = text_or_empty(get(row, "mode"));
    let mode = trim(if raw_mode.is_empty() {
        "preferred"
    } else {
        &raw_mode
    })
    .to_owned();
    if !matches!(mode.as_str(), "preferred" | "highres" | "highrr") && !mode_ok(&mode) {
        return Err(format!("{prefix}has a mode Atmos will not write."));
    }
    let mut scale = js::number(get(row, "scale"));
    if !scale.is_finite() || scale <= 0.0 {
        scale = 1.0;
    }
    scale = js::round(scale.min(4.0) * 1000.0) / 1000.0;
    let or_zero = |k: &str| {
        let v = get(row, k);
        if js::truthy(v) { rounded(v) } else { 0.0 }
    };
    let mut transform = or_zero("transform");
    if !transform.is_finite() || !(0.0..=7.0).contains(&transform) {
        transform = 0.0;
    }
    let mut vrr = or_zero("vrr");
    if !vrr.is_finite() || !(0.0..=3.0).contains(&vrr) {
        vrr = 0.0;
    }
    let depth = if js::truthy(get(row, "bitdepth")) {
        rounded(get(row, "bitdepth"))
    } else {
        8.0
    };
    let bitdepth = if depth == 10.0 { 10 } else { 8 };
    let mut cm = text_or_empty(get(row, "cm"));
    if !["", "auto", "srgb", "dcip3", "wide", "hdr", "hdredid"].contains(&cm.as_str()) {
        cm = String::new();
    }
    let mirror = trim(&text_or_empty(get(row, "mirror"))).to_owned();
    if !mirror.is_empty() && !word(&mirror, "._-") {
        return Err(format!("{prefix}has a mirror output Atmos will not write."));
    }
    let raw_position = text_or_empty(get(row, "position"));
    let mut position = trim(if raw_position.is_empty() {
        "auto"
    } else {
        &raw_position
    })
    .to_owned();
    if position != "auto" && position != "0x0" && !position_ok(&position) {
        position = "auto".into();
    }
    Ok(normalize_row(&json!({
        "output": output, "mode": mode, "position": position, "scale": js::json_number(scale),
        "transform": js::json_number(transform), "disabled": is(get(row, "disabled"), true),
        "mirror": mirror, "vrr": js::json_number(vrr), "bitdepth": bitdepth, "cm": cm,
    })))
}

fn env_var(row: &Value, prefix: &str) -> Row {
    let row = object(row, prefix, "an environment variable")?;
    let key = trim(&text_or_empty(get(row, "key"))).to_owned();
    let mut chars = key.chars();
    let shaped = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && key.len() <= 64;
    if !shaped || key == "PATH" {
        return Err(format!("{prefix}has a name Atmos will not write."));
    }
    let value = if given(get(row, "value")) {
        text(get(row, "value"))
    } else {
        String::new()
    };
    if value.contains(['\n', '\r']) || units(&value) > 512 {
        return Err(format!("{prefix}has a value Atmos will not write."));
    }
    Ok(json!({"key": key, "value": value}))
}

fn indicator(row: &Value, prefix: &str) -> Row {
    match row {
        Value::String(s) if INDICATORS.contains(&s.as_str()) => Ok(row.clone()),
        _ => Err(format!("{prefix}is not a known indicator.")),
    }
}

fn tray_id(row: &Value, prefix: &str) -> Row {
    match row {
        Value::String(s) if !s.is_empty() => Ok(row.clone()),
        _ => Err(format!("{prefix}is not a tray id.")),
    }
}

/// `canonicalizeListRow(item, row, index)`.
fn row(item: &Item, row: &Value, index: usize) -> Row {
    let prefix = format!("{} row {} ", item.label, index + 1);
    match item.key {
        "bindings" => binding(row, &prefix),
        "autostart" => autostart(row, &prefix),
        "windowRules" => window_rule(row, &prefix),
        "workspaces" => workspace(row, &prefix),
        "monitorRules" => monitor(row, &prefix),
        "envVars" => env_var(row, &prefix),
        "indicatorsItems" => indicator(row, &prefix),
        "trayHidden" | "trayPinned" => tray_id(row, &prefix),
        _ => Ok(normalize_row(row)),
    }
}

/// `canonicalizeList(item, value)`: the list as the writer applies it, or why it cannot.
pub fn list(item: &Item, value: &Value) -> Result<Vec<Value>, String> {
    let Value::Array(rows) = value else {
        return Err(format!("{} expects a list.", item.label));
    };
    let mut out = rows
        .iter()
        .enumerate()
        .map(|(i, r)| row(item, r, i))
        .collect::<Result<Vec<_>, _>>()?;
    if item.key == "indicatorsItems" {
        let next: Vec<Value> = INDICATORS
            .iter()
            .filter(|id| out.contains(&Value::from(**id)))
            .map(|id| Value::from(*id))
            .collect();
        out = if next.len() == INDICATORS.len() {
            Vec::new()
        } else {
            next
        };
    }
    Ok(out)
}

/// `String(n)` of a number a row holds, for the words a list's diff shows.
pub fn number_word(v: &Value) -> String {
    v.as_f64().map_or_else(|| js::string(v), number_text)
}
