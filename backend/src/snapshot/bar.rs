//! The bar and idle surface read from shell.json: position, clock, indicators,
//! agents, spacer, tray, power, weather, and the workspace bar.

use super::sh::{alt, is_uint, jq_text, jq_uint};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

pub struct Bar {
    pub fields: Map<String, Value>,
}

fn shell_file(home: &str, omarchy_path: &str) -> PathBuf {
    let user = PathBuf::from(home).join(".config/omarchy/shell.json");
    if user.metadata().is_ok_and(|m| m.len() > 0) {
        user
    } else {
        PathBuf::from(omarchy_path).join("config/omarchy/shell.json")
    }
}

/// `[.bar.layout.left // [], .center // [], .right // []] | add`
fn layout(doc: &Value) -> Vec<&Value> {
    ["left", "center", "right"]
        .iter()
        .filter_map(|side| alt(doc.pointer(&format!("/bar/layout/{side}"))))
        .filter_map(Value::as_array)
        .flatten()
        .collect()
}

fn widget<'a>(items: &[&'a Value], id: &str) -> Option<&'a Value> {
    items
        .iter()
        .copied()
        .find(|w| w.get("id").and_then(Value::as_str) == Some(id))
}

fn text_or(v: &Value, key: &str, default: &str) -> String {
    alt(v.get(key)).map_or_else(|| default.to_string(), jq_text)
}

fn string_list(v: Option<&Value>) -> Value {
    Value::Array(
        v.and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|s| s.as_str().is_some_and(|s| !s.is_empty()))
            .cloned()
            .collect(),
    )
}

pub fn read_bar(home: &str, omarchy_path: &str, user: &str) -> Bar {
    let file = shell_file(home, omarchy_path);
    let doc: Option<Value> = std::fs::read_to_string(&file)
        .ok()
        .map(|t| serde_json::from_str(&t).unwrap_or(Value::Null));
    let doc = doc.unwrap_or(Value::Null);
    let has_file = file.is_file();
    let mut f = Map::new();
    let mut position = "top".to_string();
    let mut transparent = false;
    let (mut screensaver, mut lock) = (150u64, 300u64);
    if has_file {
        position = text_or(&doc["bar"], "position", "top");
        transparent = jq_text_is_true(doc.pointer("/bar/transparent"));
        screensaver = jq_uint(alt(doc.pointer("/idle/screensaver"))).unwrap_or(150);
        lock = jq_uint(alt(doc.pointer("/idle/lock"))).unwrap_or(300);
    }
    if !["top", "bottom", "left", "right"].contains(&position.as_str()) {
        position = "top".into();
    }
    f.insert("barPosition".into(), json!(position));
    f.insert("barTransparent".into(), json!(transparent));
    f.insert("idleScreensaver".into(), json!(screensaver));
    f.insert("idleLock".into(), json!(lock));

    let items = if has_file { layout(&doc) } else { Vec::new() };
    let vertical = position == "left" || position == "right";
    let mut clock = (
        false,
        String::new(),
        String::new(),
        String::new(),
        0u64,
        0u64,
    );
    if let Some(w) = widget(&items, "omarchy.clock") {
        let (fmt, alt_fmt) = if vertical {
            (
                text_or(w, "verticalFormat", "HH\n—\nmm"),
                text_or(w, "verticalFormatAlt", "dd\nMMM\n'W'ww\n''yy"),
            )
        } else {
            (
                text_or(w, "format", "dddd HH:mm"),
                text_or(w, "formatAlt", "d MMMM 'W'ww yyyy"),
            )
        };
        let week = w
            .get("weekStartDay")
            .and_then(|v| alt(Some(v)))
            .map(jq_text)
            .unwrap_or_default()
            .to_lowercase();
        let week = if [
            "sunday",
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
        ]
        .contains(&week.as_str())
        {
            week
        } else {
            String::new()
        };
        let birth = jq_uint(alt(w.get("birthYear"))).unwrap_or(0);
        let life = jq_uint(alt(w.get("lifeExpectancy"))).unwrap_or(0);
        clock = (
            true,
            fmt,
            alt_fmt,
            week,
            birth,
            if (1..=150).contains(&life) { life } else { 0 },
        );
    }
    f.insert("clockPresent".into(), json!(clock.0));
    f.insert("clockFormat".into(), json!(clock.1));
    f.insert("clockFormatAlt".into(), json!(clock.2));
    f.insert("clockWeekStart".into(), json!(clock.3));
    f.insert("clockBirthYear".into(), json!(clock.4));
    f.insert("clockLifeExpectancy".into(), json!(clock.5));

    // indicators
    let (mut present, mut always, mut list) = (false, false, json!([]));
    if let Some(w) = widget(&items, "omarchy.indicators") {
        present = true;
        always = jq_text_is_true(w.get("alwaysShow"));
        const KNOWN: &[&str] = &[
            "Dictation",
            "ScreenRecording",
            "Reminder",
            "NightLight",
            "Dnd",
            "StayAwake",
        ];
        let raw = alt(w.get("items")).or_else(|| alt(w.get("indicators")));
        let ids: Vec<Value> = raw
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Object(o) => o.get("id").and_then(Value::as_str).map(String::from),
                _ => None,
            })
            .filter(|id| KNOWN.contains(&id.as_str()))
            .map(Value::String)
            .collect();
        list = Value::Array(ids);
    }
    f.insert("indicatorsPresent".into(), json!(present));
    f.insert("indicatorsAlwaysShow".into(), json!(always));
    f.insert("indicatorsItems".into(), list);

    // agents
    let (mut present, mut refresh, mut sync) = (false, 900u64, false);
    let (mut dir, mut file_name, mut device) = (String::new(), String::new(), String::new());
    if let Some(w) = widget(&items, "omarchy.agents") {
        present = true;
        refresh = jq_uint(alt(w.get("refreshIntervalSec")))
            .unwrap_or(900)
            .max(30);
        sync = text_or(w, "syncMode", "Off") == "On";
        let s = |k: &str| alt(w.get(k)).map(jq_text).unwrap_or_default();
        (dir, file_name, device) = (s("syncDir"), s("syncFileName"), s("syncDeviceId"));
    }
    f.insert("agentsPresent".into(), json!(present));
    f.insert("agentsRefreshIntervalSec".into(), json!(refresh));
    f.insert("agentsSync".into(), json!(sync));
    f.insert("agentsSyncDir".into(), json!(dir));
    f.insert("agentsSyncFileName".into(), json!(file_name));
    f.insert("agentsSyncDeviceId".into(), json!(device));

    // spacer
    let (mut present, mut size) = (false, 12u64);
    if let Some(w) = widget(&items, "omarchy.spacer") {
        present = true;
        size = jq_uint(alt(w.get("size"))).unwrap_or(12).min(64);
    }
    f.insert("spacerPresent".into(), json!(present));
    f.insert("spacerSize".into(), json!(size));

    // tray
    let (mut present, mut hidden, mut pinned) = (false, json!([]), json!([]));
    if let Some(w) = widget(&items, "omarchy.tray") {
        present = true;
        hidden = string_list(alt(w.get("hidden")));
        pinned = string_list(alt(w.get("pinned")));
    }
    f.insert("trayPresent".into(), json!(present));
    f.insert("trayHidden".into(), hidden);
    f.insert("trayPinned".into(), pinned);

    // power
    let (mut present, mut percent) = (false, false);
    if let Some(w) = widget(&items, "omarchy.power") {
        present = true;
        percent = jq_text_is_true(w.get("showPercentage"));
    }
    f.insert("powerPresent".into(), json!(present));
    f.insert("powerShowPercentage".into(), json!(percent));

    // weather
    let (mut present, mut unit, mut refresh) = (false, "auto".to_string(), 15u64);
    if let Some(w) = widget(&items, "omarchy.weather") {
        present = true;
        unit = alt(w.get("unit")).map(jq_text).unwrap_or_default();
        if !["metric", "imperial", "auto"].contains(&unit.as_str()) {
            unit = "auto".into();
        }
        refresh = jq_uint(alt(w.get("refreshMinutes"))).unwrap_or(15).max(1);
    }
    f.insert("weatherPresent".into(), json!(present));
    f.insert("weatherUnit".into(), json!(unit));
    f.insert("weatherRefreshMinutes".into(), json!(refresh));

    // workspace bar
    let (mut names, mut count) = (false, 5u64);
    if has_file {
        let mine = format!("{user}.workspaces");
        let hit = items.iter().copied().find(|w| {
            let id = if w.is_object() {
                w.get("id").and_then(Value::as_str)
            } else {
                w.as_str()
            };
            id == Some(mine.as_str()) || id == Some("omarchy.workspaces")
        });
        if let Some(w) = hit {
            let id = if w.is_object() {
                w.get("id").and_then(Value::as_str)
            } else {
                w.as_str()
            };
            if id == Some(mine.as_str()) {
                names = w.get("showNames") != Some(&Value::Bool(false));
            }
            let raw = alt(w.get("count"))
                .map(jq_text)
                .unwrap_or_else(|| "5".into());
            count = if is_uint(&raw) && matches!(raw.parse::<u64>(), Ok(1..=10)) {
                raw.parse().unwrap_or(5)
            } else {
                5
            };
        }
    }
    f.insert("workspaceBarNames".into(), json!(names));
    f.insert("workspaceBarCount".into(), json!(count));
    Bar { fields: f }
}

fn jq_text_is_true(v: Option<&Value>) -> bool {
    alt(v).is_some_and(|v| jq_text(v) == "true")
}
