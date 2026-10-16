//! The `atmos:windows` block in atmos.lua: window rules, the opaque-windows
//! override, and the Atmos window seed.

use super::lua::*;
use serde_json::{json, Map, Value};

pub const BEGIN: &str = "-- atmos:windows begin";
pub const END: &str = "-- atmos:windows end";
pub const WINDOW_CLASS: &str = "dev.csfh.atmos";

pub const OPAQUE_KEY: &str = "windowOpaque";
const OPAQUE_FLAG: &str = "-- atmos:windowOpaque = true";
// Omarchy tags every window `default-opacity` and fades that tag to 0.985/0.96
// in default.hypr.windows, which loads before atmos.lua. Matching the tag
// here overrides it; windows an app rule opted out of keep their own opacity.
const OPAQUE_RULE: &str = r#"o.window({ tag = "default-opacity" }, { opacity = "1 1" })"#;

const TILE_COMMENT: &str = "-- Tile the Atmos window like other apps.";
const FLOAT_COMMENT: &str = "-- Float and center the Atmos window.";

fn tile_line() -> String {
    format!(r#"o.window("{WINDOW_CLASS}", {{ tile = true }})"#)
}

fn float_lines() -> [String; 3] {
    [
        format!(r#"o.window("{WINDOW_CLASS}", {{ float = true }})"#),
        format!(r#"o.window("{WINDOW_CLASS}", {{ center = true }})"#),
        format!(r#"o.window("{WINDOW_CLASS}", {{ size = {{ 960, 680 }} }})"#),
    ]
}

pub fn prefs_seed() -> String {
    format!("{TILE_COMMENT}\n{}", tile_line())
}

/// Rewrite the old float/center seed to the tile seed, only for its verbatim lines.
pub fn normalize_seed(text: &str) -> String {
    let float = float_lines();
    let old = format!("{FLOAT_COMMENT}\n{}\n{}\n{}", float[0], float[1], float[2]);
    if text.contains(&old) {
        return text.replace(&old, &prefs_seed());
    }
    let tile = tile_line();
    if text.contains(&tile) || !text.contains(FLOAT_COMMENT) {
        return text.to_string();
    }
    let lines: Vec<&str> = text.split('\n').collect();
    if !lines.iter().any(|l| float.iter().any(|f| f == l.trim())) {
        return text.to_string();
    }
    let mut out: Vec<&str> = Vec::new();
    let mut inserted = false;
    for line in lines
        .iter()
        .filter(|l| !float.iter().any(|f| f == l.trim()))
    {
        out.push(line);
        if !inserted && line.trim() == FLOAT_COMMENT {
            out.push(&tile);
            inserted = true;
        }
    }
    let mut joined = out.join("\n");
    if !inserted {
        joined = format!("{tile}\n{joined}");
    }
    joined.replace(FLOAT_COMMENT, TILE_COMMENT)
}

pub fn read_opaque(text: &str) -> bool {
    text.lines().any(|l| l.trim() == OPAQUE_FLAG)
}

fn opaque_lines(on: bool) -> Vec<String> {
    if on {
        vec![OPAQUE_FLAG.into(), OPAQUE_RULE.into()]
    } else {
        Vec::new()
    }
}

pub fn sanitize_match(raw: Option<&Value>) -> String {
    let text = str_or_empty(raw);
    if text.contains('\n') || text.contains('\r') {
        return String::new();
    }
    let text = text.trim();
    if text.is_empty() || text.chars().count() > 128 || text.contains("]]") {
        String::new()
    } else {
        text.to_string()
    }
}

fn sanitize_workspace(raw: Option<&Value>) -> String {
    let text = str_or_empty(raw).trim().to_string();
    if re!(r"^[A-Za-z0-9:_-]{1,32}$").is_match(&text) {
        text
    } else {
        String::new()
    }
}

fn clamp_size(raw: Option<&Value>) -> i64 {
    match py_float(raw).map(f64::round_ties_even) {
        Some(n) if (100.0..=4000.0).contains(&n) => n as i64,
        _ => 0,
    }
}

fn sanitize_opacity(raw: Option<&Value>) -> String {
    match raw {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) if s.is_empty() => String::new(),
        // Python counts a bool as the number 1 or 0.
        Some(Value::Bool(b)) => {
            if *b {
                "1.0".into()
            } else {
                String::new()
            }
        }
        Some(Value::Number(n)) => {
            let n = n.as_f64().unwrap_or(0.0);
            if !(0.2..=1.0).contains(&n) {
                return String::new();
            }
            let rounded = (n * 100.0).round_ties_even() / 100.0;
            // Python's str(float): 1.0 stays "1.0", 0.5 is "0.5".
            if rounded.fract() == 0.0 {
                format!("{rounded:.1}")
            } else {
                rounded.to_string()
            }
        }
        Some(other) => {
            let text = str_or_empty(Some(other)).trim().to_string();
            if re!(r"^[0-9.]+( [0-9.]+)?$").is_match(&text) && text.chars().count() <= 16 {
                text
            } else {
                String::new()
            }
        }
    }
}

struct Row {
    matcher: String,
    title: String,
    placement: &'static str,
    center: bool,
    width: i64,
    height: i64,
    workspace: String,
    pin: bool,
    fullscreen: bool,
    opacity: String,
}

fn normalize(row: &Value) -> Option<Row> {
    let map = row.as_object()?;
    let matcher = sanitize_match(map.get("match"));
    if matcher.is_empty() {
        return None;
    }
    let is_true = |k: &str| map.get(k) == Some(&Value::Bool(true));
    let mut placement = match str_or_empty(map.get("placement")).as_str() {
        "float" => "float",
        "tile" => "tile",
        _ => "",
    };
    if is_true("float") {
        placement = "float";
    }
    if is_true("tile") && placement != "float" {
        placement = "tile";
    }
    let (mut width, mut height) = (clamp_size(map.get("width")), clamp_size(map.get("height")));
    if let Some(size) = map
        .get("size")
        .and_then(Value::as_array)
        .filter(|s| s.len() >= 2)
    {
        width = clamp_size(size.first());
        height = clamp_size(size.get(1));
    }
    if width == 0 || height == 0 {
        width = 0;
        height = 0;
    }
    let row = Row {
        matcher,
        title: sanitize_match(map.get("title")),
        placement,
        center: is_true("center"),
        width,
        height,
        workspace: sanitize_workspace(map.get("workspace")),
        pin: is_true("pin"),
        fullscreen: is_true("fullscreen"),
        opacity: sanitize_opacity(map.get("opacity")),
    };
    let empty = row.placement.is_empty()
        && !row.center
        && row.width == 0
        && row.workspace.is_empty()
        && row.title.is_empty()
        && !row.pin
        && !row.fullscreen
        && row.opacity.is_empty();
    (!empty).then_some(row)
}

fn row_json(row: &Row, managed: Option<bool>) -> Value {
    let mut value = json!({
        "match": row.matcher,
        "title": row.title,
        "placement": row.placement,
        "center": row.center,
        "width": row.width,
        "height": row.height,
        "workspace": row.workspace,
        "pin": row.pin,
        "fullscreen": row.fullscreen,
        "opacity": row.opacity,
    });
    if let Some(managed) = managed {
        value["managed"] = json!(managed);
    }
    value
}

fn row_from_args(args: &[Value]) -> Option<Row> {
    let matcher = args.first()?.as_str()?;
    let empty = Map::new();
    let rules = args.get(1).and_then(Value::as_object).unwrap_or(&empty);
    let (mut width, mut height) = (0, 0);
    if let Some(size) = rules
        .get("size")
        .and_then(Value::as_array)
        .filter(|s| s.len() >= 2)
    {
        width = clamp_size(size.first());
        height = clamp_size(size.get(1));
    }
    let flag = |k: &str| json!(rules.get(k) == Some(&Value::Bool(true)));
    normalize(&json!({
        "match": matcher,
        "title": rules.get("title"),
        "float": flag("float"),
        "tile": flag("tile"),
        "center": flag("center"),
        "width": width,
        "height": height,
        "workspace": rules.get("workspace"),
        "pin": flag("pin"),
        "fullscreen": flag("fullscreen"),
        "opacity": rules.get("opacity"),
    }))
}

fn calls(src: &str) -> Vec<Row> {
    let mut out = Vec::new();
    live_calls(src, "o.window(", |args| {
        out.extend(row_from_args(args));
    });
    out
}

pub fn parse(text: &str) -> Value {
    let mut items = Vec::new();
    match sentinel_bounds(text, BEGIN, END) {
        Some((start, stop)) => {
            let outside = format!("{}\n{}", &text[..start], &text[stop..]);
            items.extend(calls(&outside).iter().map(|r| row_json(r, Some(false))));
            items.extend(
                calls(&text[start..stop])
                    .iter()
                    .map(|r| row_json(r, Some(true))),
            );
        }
        None => items.extend(calls(text).iter().map(|r| row_json(r, Some(false)))),
    }
    Value::Array(items)
}

fn serialize_row(row: &Row) -> String {
    let mut parts = Vec::new();
    if !row.title.is_empty() {
        parts.push(format!("title = {}", lua_string(&row.title)));
    }
    match row.placement {
        "float" => parts.push("float = true".into()),
        "tile" => parts.push("tile = true".into()),
        _ => {}
    }
    if row.center {
        parts.push("center = true".into());
    }
    if row.width != 0 && row.height != 0 {
        parts.push(format!("size = {{ {}, {} }}", row.width, row.height));
    }
    if !row.workspace.is_empty() {
        parts.push(format!("workspace = {}", lua_string(&row.workspace)));
    }
    if row.pin {
        parts.push("pin = true".into());
    }
    if row.fullscreen {
        parts.push("fullscreen = true".into());
    }
    if !row.opacity.is_empty() {
        parts.push(format!("opacity = {}", lua_string(&row.opacity)));
    }
    if parts.is_empty() {
        return String::new();
    }
    format!(
        "o.window({}, {{ {} }})",
        lua_string(&row.matcher),
        parts.join(", ")
    )
}

pub fn serialize(src: &Map<String, Value>) -> String {
    let mut lines = vec![BEGIN.to_string()];
    lines.extend(opaque_lines(
        src.get(OPAQUE_KEY) == Some(&Value::Bool(true)),
    ));
    for item in src
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(row) = normalize(item) {
            let line = serialize_row(&row);
            if !line.is_empty() {
                lines.push(line);
            }
        }
    }
    lines.push(END.into());
    lines.join("\n")
}
