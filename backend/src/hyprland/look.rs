//! The `atmos:look` block in looknfeel.lua.

use super::lua::*;
use regex_lite::Regex;
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

pub fn parse(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let Some((start, stop)) = sentinel_bounds(text, BEGIN, END) else {
        return out;
    };
    let body = &text[start..stop];
    for (lua, key) in [
        ("gaps_in", "gapsIn"),
        ("gaps_out", "gapsOut"),
        ("border_size", "borderSize"),
        ("rounding", "rounding"),
        ("dim_strength", "dimStrength"),
        ("active_opacity", "activeOpacity"),
        ("inactive_opacity", "inactiveOpacity"),
        ("column_width", "columnWidth"),
        ("on_focus_under_fullscreen", "onFocusUnderFullscreen"),
    ] {
        if let Some(v) = num_value(&lua_num_field(body, lua)) {
            out.insert(key.into(), v);
        }
    }
    for (lua, key) in [
        ("allow_tearing", "allowTearing"),
        ("resize_on_border", "resizeOnBorder"),
        ("dim_inactive", "dimInactive"),
        ("focus_on_activate", "focusOnActivate"),
        ("enable_swallow", "enableSwallow"),
        ("hide_on_key_press", "cursorHideOnKey"),
        ("preserve_split", "preserveSplit"),
    ] {
        if let Some(v) = lua_bool_field(body, lua) {
            out.insert(key.into(), json!(v));
        }
    }
    for section in ["shadow", "blur"] {
        let re = Regex::new(&format!(
            r"{section}\s*=\s*\{{\s*enabled\s*=\s*(true|false)"
        ))
        .expect("regex");
        if let Some(m) = re.captures(body) {
            out.insert(section.into(), json!(&m[1] == "true"));
        }
    }
    if let Some(m) = re!(r"animations\s*=\s*\{\s*enabled\s*=\s*(true|false)").captures(body) {
        out.insert("animations".into(), json!(&m[1] == "true"));
    }
    let layout = lua_str_field(body, "layout");
    if !layout.is_empty() {
        out.insert("layout".into(), json!(layout));
    }
    let swallow = lua_str_field(body, "swallow_regex");
    if !swallow.is_empty() || body.contains("swallow_regex") {
        out.insert("swallowRegex".into(), json!(swallow));
    }
    if let Some(m) = re!(r"warp_on_change_workspace\s*=\s*(true|false|1|0)").captures(body) {
        out.insert("cursorWarp".into(), json!(&m[1] == "true" || &m[1] == "1"));
    }
    let size = re!(r#"hl\.env\(\s*"HYPRCURSOR_SIZE"\s*,\s*"([^"]*)"\s*\)"#)
        .captures(body)
        .or_else(|| re!(r#"hl\.env\(\s*"XCURSOR_SIZE"\s*,\s*"([^"]*)"\s*\)"#).captures(body));
    if let Some(m) = size {
        if let Ok(n) = m[1].trim().parse::<f64>() {
            out.insert("cursorSize".into(), json!(n.trunc() as i64));
        }
    }
    out
}
