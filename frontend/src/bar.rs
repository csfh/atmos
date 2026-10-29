//! The Bar page's rows Atmos draws itself (`pages/BarPage.qml`): the tray's hidden and pinned
//! icons, the shown indicators, the agents' usage refresh and sync names, the clock's formats,
//! week start, birth year and life expectancy, and the plugins. Each write is what the QML page's
//! setter sends (`Omarchy.setTrayHidden`, `setIndicatorsItems`, `setAgentsRefreshIntervalSec`,
//! ...); `tests/catalog_setters.rs` runs each, and `tests/catalog_bar.rs` the page's own code,
//! under Node against this.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::{js, protocol, settings};

/// The indicators (`indicatorOptions`), by id and label.
pub const INDICATORS: &[(&str, &str)] = &[
    ("Dictation", "Dictation"),
    ("ScreenRecording", "Screen recording"),
    ("Reminder", "Reminder"),
    ("NightLight", "Night light"),
    ("Dnd", "Do not disturb"),
    ("StayAwake", "Stay awake"),
];

pub const HORIZONTAL: &[(&str, &str)] = &[
    ("dddd HH:mm", "Monday 14:05"),
    ("dddd h:mm AP", "Monday 2:05 PM"),
    ("HH:mm", "14:05"),
    ("h:mm AP", "2:05 PM"),
    ("ddd d MMM HH:mm", "Mon 1 Sep 14:05"),
    ("ddd d MMM h:mm AP", "Mon 1 Sep 2:05 PM"),
    ("d MMMM 'W'ww yyyy", "1 September W36 2026"),
    ("yyyy-MM-dd HH:mm", "2026-09-01 14:05"),
];

pub const VERTICAL: &[(&str, &str)] = &[
    ("HH\n—\nmm", "14 — 05"),
    ("h\n—\nmm\nAP", "2 — 05 PM"),
    ("dd\nMMM\n'W'ww\n''yy", "01 Sep W36 '26"),
    ("HH\nmm", "14 05"),
];

pub const DAYS: &[(&str, &str)] = &[
    ("sunday", "Sunday"),
    ("monday", "Monday"),
    ("tuesday", "Tuesday"),
    ("wednesday", "Wednesday"),
    ("thursday", "Thursday"),
    ("friday", "Friday"),
    ("saturday", "Saturday"),
];

fn strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(js::string)
        .collect()
}

/// `normalizedStringIds`: each id once, none empty.
pub fn string_ids(list: Option<&Value>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in list.and_then(Value::as_array).into_iter().flatten() {
        let id = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        if !id.is_empty() && !out.contains(&id) {
            out.push(id);
        }
    }
    out
}

/// `normalizedIndicatorItems`: the known indicators the list names, in their order.
pub fn indicator_items(list: Option<&Value>) -> Vec<String> {
    let names = strings(list);
    if !list.is_some_and(Value::is_array) {
        return Vec::new();
    }
    INDICATORS
        .iter()
        .map(|(id, _)| (*id).to_owned())
        .filter(|id| names.contains(id))
        .collect()
}

/// `trayIdLabel(id)`: what follows the last slash.
pub fn tray_label(id: &str) -> &str {
    id.rsplit('/').next().unwrap_or(id)
}

pub fn hidden_says(list: Option<&Value>) -> String {
    let labels: Vec<String> = strings(list)
        .iter()
        .map(|i| tray_label(i).to_owned())
        .collect();
    format!(
        "Hidden right now: {}. Show all brings them back.",
        labels.join(", ")
    )
}

pub fn pinned_says(list: Option<&Value>) -> String {
    let labels: Vec<String> = strings(list)
        .iter()
        .map(|i| tray_label(i).to_owned())
        .collect();
    format!(
        "Always visible: {}. Unpin all lets the tray manage them again.",
        labels.join(", ")
    )
}

/// `setTrayHidden([])` or `setTrayPinned([])`: none when it is empty already.
pub fn clear_tray(key: &str, current: Option<&Value>) -> Option<Value> {
    (!string_ids(current).is_empty()).then(|| settings::write(key, &json!([])))
}

/// `indicatorShown(id)`: every indicator shows while the list is empty.
pub fn indicator_shown(items: Option<&Value>, id: &str) -> bool {
    let list = strings(items);
    list.is_empty() || list.iter().any(|i| i == id)
}

/// `toggleIndicator(id)`, then `setIndicatorsItems(next)`: none when nothing would change.
pub fn toggle_indicator(items: Option<&Value>, id: &str) -> Option<Value> {
    indicators_write(&toggle_next(items, id)?, items)
}

/// `toggleIndicator(id)`: the list it hands `setIndicatorsItems`, none when it would be empty.
pub fn toggle_next(items: Option<&Value>, id: &str) -> Option<Vec<String>> {
    let all: Vec<&str> = INDICATORS.iter().map(|(i, _)| *i).collect();
    let current = strings(items);
    let mut selected: Vec<&str> = all
        .iter()
        .copied()
        .filter(|a| current.iter().any(|c| c == a))
        .collect();
    if current.is_empty() || selected.is_empty() {
        selected = all.clone();
    }
    let on = selected.contains(&id);
    let next: Vec<&str> = all
        .iter()
        .copied()
        .filter(|a| if *a == id { !on } else { selected.contains(a) })
        .collect();
    (!next.is_empty()).then(|| next.iter().map(|s| (*s).to_owned()).collect())
}

/// `setIndicatorsItems(list)`: every indicator is the empty list; none when it is so already.
pub fn indicators_write(list: &[String], current: Option<&Value>) -> Option<Value> {
    let next = indicator_items(Some(&json!(list)));
    let next = if next.len() == INDICATORS.len() {
        Vec::new()
    } else {
        next
    };
    let shown: Vec<Value> = current
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    (json!(next) != json!(shown)).then(|| settings::write("indicatorsItems", &json!(next)))
}

/// The usage refresh in minutes, as the slider shows it.
pub fn refresh_minutes(seconds: Option<&Value>) -> f64 {
    (js::round(js::number(seconds) / 60.0)).max(5.0)
}

/// The slider's `onChanged`, then `setAgentsRefreshIntervalSec`.
pub fn refresh_write(minutes: f64, seconds: Option<&Value>) -> Option<Value> {
    seconds_write(js::round(minutes) * 60.0, seconds)
}

/// `setAgentsRefreshIntervalSec(seconds)`: none under 30, or when it is so already.
pub fn seconds_write(next: f64, seconds: Option<&Value>) -> Option<Value> {
    let next = js::round(next);
    let current = seconds.and_then(Value::as_f64);
    (next >= 30.0 && Some(next) != current)
        .then(|| settings::write("agentsRefreshIntervalSec", &js::json_number(next)))
}

/// `setAgentsSyncDir(path)`.
pub fn sync_dir_write(raw: &str, current: Option<&Value>) -> Option<Value> {
    let path = raw.trim().to_owned();
    (current != Some(&Value::from(path.clone())))
        .then(|| settings::write("agentsSyncDir", &Value::from(path)))
}

/// `setAgentsSyncFileName(name)`: the trimmed name after its last slash.
pub fn sync_file_write(raw: &str, current: Option<&Value>) -> Option<Value> {
    let name = raw.trim().rsplit('/').next().unwrap_or_default().to_owned();
    (current != Some(&Value::from(name.clone())))
        .then(|| settings::write("agentsSyncFileName", &Value::from(name)))
}

/// `setAgentsSyncDeviceId(id)`.
pub fn device_write(raw: &str, current: Option<&Value>) -> Option<Value> {
    let id = raw.trim().to_owned();
    (current != Some(&Value::from(id.clone())))
        .then(|| settings::write("agentsSyncDeviceId", &Value::from(id)))
}

/// `clockFormatOptions` or `clockFormatAltOptions`: the bar's set, and the format in use when it
/// is none of them.
pub fn clock_options(position: &str, current: &str) -> Vec<(String, String)> {
    let vertical = matches!(position, "left" | "right");
    let list = if vertical { VERTICAL } else { HORIZONTAL };
    let mut out: Vec<(String, String)> = list
        .iter()
        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
        .collect();
    if !current.is_empty() && !out.iter().any(|(v, _)| v == current) {
        out.push((current.to_owned(), current.to_owned()));
    }
    out
}

/// `weekStartOptions`: Locale default first while none is set.
pub fn week_options(current: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = DAYS
        .iter()
        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
        .collect();
    if current.is_empty() {
        out.insert(0, (String::new(), "Locale default".into()));
    }
    out
}

/// `setClockWeekStart(day)`.
pub fn week_write(day: &str, current: Option<&Value>) -> Option<Value> {
    let day = day.to_lowercase();
    (DAYS.iter().any(|(d, _)| *d == day) && current != Some(&Value::from(day.clone())))
        .then(|| settings::write("clockWeekStart", &Value::from(day)))
}

/// This year, on the system's clock.
pub fn this_year() -> i64 {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400) as i64;
    // Days to a civil date (Howard Hinnant's algorithm), for the year alone.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    yoe + era * 400 + i64::from(month <= 2)
}

/// A year or span as the setter reads it: a number is rounded, then it is trimmed text.
fn as_text(v: &Value) -> String {
    match v {
        Value::Number(_) => js::string(&js::json_number(js::round(js::number(Some(v))))),
        _ if js::truthy(Some(v)) => js::string(v).trim().to_owned(),
        _ => String::new(),
    }
}

/// `setClockBirthYear(year)`, in `now`.
pub fn birth_write(year: &Value, current: Option<&Value>, now: i64) -> Option<Value> {
    let text = as_text(year);
    let shown = current.and_then(Value::as_f64).unwrap_or(0.0);
    if text.is_empty() || text == "0" {
        return (shown != 0.0).then(|| settings::write("clockBirthYear", &json!(0)));
    }
    if text.len() != 4 || !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let born: i64 = text.parse().ok()?;
    (born >= now - 120 && born <= now && born as f64 != shown)
        .then(|| settings::write("clockBirthYear", &json!(born)))
}

/// `setClockLifeExpectancy(years)`.
pub fn life_write(years: &Value, current: Option<&Value>) -> Option<Value> {
    let text = as_text(years);
    let shown = current.and_then(Value::as_f64).unwrap_or(0.0);
    if text.is_empty() || text == "0" {
        return (shown != 0.0).then(|| settings::write("clockLifeExpectancy", &json!(0)));
    }
    if !text.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let span: i64 = text.parse().ok()?;
    ((1..=150).contains(&span) && span as f64 != shown)
        .then(|| settings::write("clockLifeExpectancy", &json!(span)))
}

/// `setPluginEnabled(id, on)`: none for an id that is not one, or a plugin already so.
pub fn plugin_write(id: &str, on: bool, plugins: Option<&Value>) -> Option<Value> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {
        return None;
    }
    let found = plugins
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|p| p.get("id").is_some_and(|i| js::string(i) == id));
    if found.is_some_and(|p| (p.get("enabled") == Some(&Value::Bool(true))) == on) {
        return None;
    }
    Some(protocol::write(
        "plugin.set",
        &json!({ "id": id, "on": on }),
    ))
}

/// A plugin's row: its label, what it says, and whether it can be turned off.
pub fn plugin_row(p: &Value) -> (String, &'static str, bool) {
    let name = ["name", "id"]
        .iter()
        .filter_map(|k| p.get(*k).filter(|v| js::truthy(Some(v))))
        .map(js::string)
        .next()
        .unwrap_or_else(|| "Plugin".into());
    let fixed = p.get("canDisable") == Some(&Value::Bool(false));
    let says = if fixed {
        "Always on. This widget cannot be disabled here."
    } else if js::truthy(p.get("firstParty")) {
        "A first-party Omarchy plugin."
    } else {
        "A third-party shell plugin."
    };
    (name, says, !fixed)
}

/// What Birth year says.
pub fn birth_says(year: f64) -> &'static str {
    if year > 0.0 {
        "The calendar popup draws a life bar from this year to the expectancy below."
    } else {
        "Optional. Set a year if you want a life bar in the calendar popup."
    }
}
