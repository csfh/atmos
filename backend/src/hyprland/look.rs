//! The `atmos:look` block in looknfeel.lua.

use super::lua::*;
use serde_json::{json, Map, Value};

pub const BEGIN: &str = "-- atmos:look begin";
pub const END: &str = "-- atmos:look end";
pub const LEGACY_BEGIN: &str = "-- omarchy-prefs:look begin";
pub const LEGACY_END: &str = "-- omarchy-prefs:look end";

fn swallow_regex(raw: Option<&Value>) -> String {
    let text = str_or_empty(raw);
    if text.contains('\n') || text.contains('\r') {
        return String::new();
    }
    let text = text.trim();
    if text.chars().count() > 128 {
        String::new()
    } else {
        text.to_string()
    }
}

pub fn serialize(src: &Map<String, Value>) -> String {
    let get = |k: &str| src.get(k);
    let layout = match get("layout") {
        Some(Value::String(s)) if s == "scrolling" => "scrolling",
        _ => "dwindle",
    };
    let gaps_in = clamp_int(get("gapsIn"), 0, 64, 5);
    let gaps_out = clamp_int(get("gapsOut"), 0, 64, 10);
    let border = clamp_int(get("borderSize"), 0, 16, 2);
    let rounding = clamp_int(get("rounding"), 0, 32, 0);
    let flag = |k: &str, fb: bool| as_bool(get(k), fb);
    let column = clamp_float(get("columnWidth"), 0.2, 1.0, 0.49);
    let dim_strength = clamp_float(get("dimStrength"), 0.0, 1.0, 0.15);
    let cursor_size = clamp_int(get("cursorSize"), 8, 64, 24);
    let active = clamp_float(get("activeOpacity"), 0.2, 1.0, 1.0);
    let inactive = clamp_float(get("inactiveOpacity"), 0.2, 1.0, 1.0);
    let regex = swallow_regex(get("swallowRegex"));
    let fullscreen = clamp_int(get("onFocusUnderFullscreen"), 0, 2, 1);
    let mut misc = vec![
        format!(
            "    focus_on_activate = {},",
            lua_bool(flag("focusOnActivate", false))
        ),
        format!(
            "    enable_swallow = {},",
            lua_bool(flag("enableSwallow", false))
        ),
    ];
    if !regex.is_empty() {
        misc.push(format!("    swallow_regex = {},", lua_string(&regex)));
    }
    misc.push(format!(
        "    on_focus_under_fullscreen = {},",
        lua_number(fullscreen as f64)
    ));
    let mut lines: Vec<String> = vec![
        BEGIN.into(),
        "hl.config({".into(),
        "  general = {".into(),
        format!("    gaps_in = {},", lua_number(gaps_in as f64)),
        format!("    gaps_out = {},", lua_number(gaps_out as f64)),
        format!("    border_size = {},", lua_number(border as f64)),
        format!("    layout = {},", lua_string(layout)),
        format!(
            "    allow_tearing = {},",
            lua_bool(flag("allowTearing", false))
        ),
        format!(
            "    resize_on_border = {},",
            lua_bool(flag("resizeOnBorder", false))
        ),
        "  },".into(),
        "  decoration = {".into(),
        format!("    rounding = {},", lua_number(rounding as f64)),
        "    shadow = {".into(),
        format!("      enabled = {},", lua_bool(flag("shadow", false))),
        "    },".into(),
        "    blur = {".into(),
        format!("      enabled = {},", lua_bool(flag("blur", false))),
        "    },".into(),
        format!(
            "    dim_inactive = {},",
            lua_bool(flag("dimInactive", false))
        ),
        format!("    dim_strength = {},", lua_number(dim_strength)),
        format!("    active_opacity = {},", lua_number(active)),
        format!("    inactive_opacity = {},", lua_number(inactive)),
        "  },".into(),
        "  animations = {".into(),
        format!("    enabled = {},", lua_bool(flag("animations", true))),
        "  },".into(),
        "  scrolling = {".into(),
        format!("    column_width = {},", lua_number(column)),
        "  },".into(),
        "  dwindle = {".into(),
        format!(
            "    preserve_split = {},",
            lua_bool(flag("preserveSplit", false))
        ),
        "  },".into(),
        "  misc = {".into(),
    ];
    lines.extend(misc);
    lines.extend([
        "  },".into(),
        "  cursor = {".into(),
        format!(
            "    hide_on_key_press = {},",
            lua_bool(flag("cursorHideOnKey", true))
        ),
        format!(
            "    warp_on_change_workspace = {},",
            u8::from(flag("cursorWarp", true))
        ),
        "  },".into(),
        "})".into(),
        format!(
            "hl.env({}, {})",
            lua_string("HYPRCURSOR_SIZE"),
            lua_string(&cursor_size.to_string())
        ),
        format!(
            "hl.env({}, {})",
            lua_string("XCURSOR_SIZE"),
            lua_string(&cursor_size.to_string())
        ),
        END.into(),
    ]);
    lines.join("\n")
}

fn section<'a>(map: &'a Map<String, Value>, key: &str) -> Option<&'a Map<String, Value>> {
    map.get(key).and_then(Value::as_object)
}

fn put_num(out: &mut Map<String, Value>, table: Option<&Map<String, Value>>, lua: &str, key: &str) {
    let Some(Value::Number(number)) = table.and_then(|row| row.get(lua)) else {
        return;
    };
    out.insert(key.into(), Value::Number(number.clone()));
}

fn put_bool(
    out: &mut Map<String, Value>,
    table: Option<&Map<String, Value>>,
    lua: &str,
    key: &str,
) {
    let Some(Value::Bool(value)) = table.and_then(|row| row.get(lua)) else {
        return;
    };
    out.insert(key.into(), json!(*value));
}

fn put_text(
    out: &mut Map<String, Value>,
    table: Option<&Map<String, Value>>,
    lua: &str,
    key: &str,
    keep_empty: bool,
) {
    let Some(Value::String(text)) = table.and_then(|row| row.get(lua)) else {
        return;
    };
    if text.is_empty() && !keep_empty {
        return;
    }
    out.insert(key.into(), json!(text));
}

fn put_warp(out: &mut Map<String, Value>, cursor: Option<&Map<String, Value>>) {
    match cursor.and_then(|row| row.get("warp_on_change_workspace")) {
        Some(Value::Bool(value)) => {
            out.insert("cursorWarp".into(), json!(*value));
        }
        Some(Value::Number(number)) => {
            if let Some(0 | 1) = number.as_i64() {
                out.insert("cursorWarp".into(), json!(number.as_i64() == Some(1)));
            }
        }
        _ => {}
    }
}

fn cursor_size(body: &str) -> Option<i64> {
    let mut hypr = None;
    let mut saw_hypr = false;
    let mut xcursor = None;
    let mut i = 0;
    let mark = "hl.env(";
    while let Some(rel) = body[i..].find(mark) {
        let at = i + rel;
        let Some((args, end)) = parse_call_args(body, at + mark.len()) else {
            i = at + mark.len();
            continue;
        };
        i = end;
        let name = args.first().and_then(Value::as_str);
        let raw = args.get(1).and_then(Value::as_str).unwrap_or("");
        let parsed = raw.trim().parse::<f64>().ok().map(|n| n.trunc() as i64);
        match name {
            Some("HYPRCURSOR_SIZE") => {
                saw_hypr = true;
                if hypr.is_none() {
                    hypr = parsed;
                }
            }
            Some("XCURSOR_SIZE") if xcursor.is_none() => xcursor = parsed,
            _ => {}
        }
    }
    if saw_hypr {
        hypr
    } else {
        xcursor
    }
}

pub fn parse(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let Some((start, stop)) = sentinel_bounds(text, BEGIN, END) else {
        return out;
    };
    let body = &text[start..stop];
    let config = first_table_arg(body, "hl.config(");
    let general = config.as_ref().and_then(|row| section(row, "general"));
    let decoration = config.as_ref().and_then(|row| section(row, "decoration"));
    let shadow = decoration.and_then(|row| section(row, "shadow"));
    let blur = decoration.and_then(|row| section(row, "blur"));
    let animations = config.as_ref().and_then(|row| section(row, "animations"));
    let scrolling = config.as_ref().and_then(|row| section(row, "scrolling"));
    let dwindle = config.as_ref().and_then(|row| section(row, "dwindle"));
    let misc = config.as_ref().and_then(|row| section(row, "misc"));
    let cursor = config.as_ref().and_then(|row| section(row, "cursor"));
    for (lua, key) in [
        ("gaps_in", "gapsIn"),
        ("gaps_out", "gapsOut"),
        ("border_size", "borderSize"),
    ] {
        put_num(&mut out, general, lua, key);
    }
    for (lua, key) in [
        ("rounding", "rounding"),
        ("dim_strength", "dimStrength"),
        ("active_opacity", "activeOpacity"),
        ("inactive_opacity", "inactiveOpacity"),
    ] {
        put_num(&mut out, decoration, lua, key);
    }
    put_num(&mut out, scrolling, "column_width", "columnWidth");
    put_num(
        &mut out,
        misc,
        "on_focus_under_fullscreen",
        "onFocusUnderFullscreen",
    );
    for (lua, key) in [
        ("allow_tearing", "allowTearing"),
        ("resize_on_border", "resizeOnBorder"),
    ] {
        put_bool(&mut out, general, lua, key);
    }
    put_bool(&mut out, decoration, "dim_inactive", "dimInactive");
    put_bool(&mut out, shadow, "enabled", "shadow");
    put_bool(&mut out, blur, "enabled", "blur");
    put_bool(&mut out, animations, "enabled", "animations");
    put_bool(&mut out, misc, "focus_on_activate", "focusOnActivate");
    put_bool(&mut out, misc, "enable_swallow", "enableSwallow");
    put_bool(&mut out, cursor, "hide_on_key_press", "cursorHideOnKey");
    put_bool(&mut out, dwindle, "preserve_split", "preserveSplit");
    put_text(&mut out, general, "layout", "layout", false);
    put_text(&mut out, misc, "swallow_regex", "swallowRegex", true);
    put_warp(&mut out, cursor);
    if let Some(size) = cursor_size(body) {
        out.insert("cursorSize".into(), json!(size));
    }
    out
}
