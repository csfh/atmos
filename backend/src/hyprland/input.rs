//! The `atmos:input` block in input.lua, and the workspace-gesture rules.

use super::lua::*;
use serde_json::{json, Map, Value};

pub const BEGIN: &str = "-- atmos:input begin";
pub const END: &str = "-- atmos:input end";
pub const LEGACY_BEGIN: &str = "-- omarchy-prefs:input begin";
pub const LEGACY_END: &str = "-- omarchy-prefs:input end";

fn layouts(raw: Option<&Value>) -> String {
    let text = str_or_empty(raw).trim().to_lowercase();
    if text.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    for part in &parts {
        if !re!(r"^[a-z0-9]{1,8}$").is_match(part) {
            return String::new();
        }
    }
    parts.join(",")
}

fn variants(raw: Option<&Value>, layout_count: usize) -> String {
    let text = str_or_empty(raw).trim().to_string();
    if text.is_empty() {
        return String::new();
    }
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    if layout_count > 0 && parts.len() != layout_count {
        return String::new();
    }
    if parts
        .iter()
        .any(|p| !p.is_empty() && !re!(r"^[A-Za-z0-9_-]{1,32}$").is_match(p))
    {
        return String::new();
    }
    parts.join(",")
}

pub fn serialize(src: &Map<String, Value>, existing: &str) -> String {
    let get = |k: &str| src.get(k);
    let accel = match get("accelProfile") {
        Some(Value::String(s)) if s == "flat" || s == "adaptive" => s.clone(),
        _ => String::new(),
    };
    let layout = layouts(get("kbLayoutOverride"));
    let count = if layout.is_empty() {
        0
    } else {
        layout.split(',').count()
    };
    let variant = variants(get("kbVariantOverride"), count);
    let flag = |k: &str, fb: bool| as_bool(get(k), fb);
    let mut input = vec![
        format!(
            "    sensitivity = {},",
            lua_number(clamp_float(get("sensitivity"), -1.0, 1.0, 0.0))
        ),
        format!(
            "    repeat_rate = {},",
            lua_number(clamp_int(get("repeatRate"), 10, 100, 40) as f64)
        ),
        format!(
            "    repeat_delay = {},",
            lua_number(clamp_int(get("repeatDelay"), 100, 1000, 250) as f64)
        ),
        format!(
            "    numlock_by_default = {},",
            lua_bool(flag("numlock", true))
        ),
        format!(
            "    follow_mouse = {},",
            lua_number(clamp_int(get("followMouse"), 0, 3, 1) as f64)
        ),
    ];
    if !accel.is_empty() {
        input.insert(1, format!("    accel_profile = {},", lua_string(&accel)));
    }
    input.push(format!(
        "    emulate_discrete_scroll = {},",
        lua_number(clamp_int(get("emulateDiscreteScroll"), 0, 2, 1) as f64)
    ));
    if !layout.is_empty() {
        input.push(format!("    kb_layout = {},", lua_string(&layout)));
        if !variant.is_empty() {
            input.push(format!("    kb_variant = {},", lua_string(&variant)));
        }
        let mut options = "compose:caps,shift:both_capslock_cancel".to_string();
        if flag("kbGroupToggle", false) {
            options.push_str(",grp:alts_toggle");
        }
        input.push(format!("    kb_options = {},", lua_string(&options)));
    }
    input.extend([
        "    touchpad = {".into(),
        format!(
            "      natural_scroll = {},",
            lua_bool(flag("naturalScroll", false))
        ),
        format!(
            "      clickfinger_behavior = {},",
            lua_bool(flag("clickfinger", true))
        ),
        format!(
            "      scroll_factor = {},",
            lua_number(clamp_float(get("scrollFactor"), 0.1, 3.0, 0.4))
        ),
        format!(
            "      disable_while_typing = {},",
            lua_bool(flag("disableWhileTyping", true))
        ),
        format!(
            "      drag_3fg = {},",
            lua_number(clamp_int(get("drag3fg"), 0, 1, 0) as f64)
        ),
        "    },".into(),
    ]);
    let mut lines: Vec<String> = vec![BEGIN.into(), "hl.config({".into(), "  input = {".into()];
    lines.extend(input);
    lines.extend([
        "  },".into(),
        "  misc = {".into(),
        format!(
            "    key_press_enables_dpms = {},",
            lua_bool(flag("keyPressDpms", true))
        ),
        format!(
            "    mouse_move_enables_dpms = {},",
            lua_bool(flag("mouseMoveDpms", true))
        ),
        "  },".into(),
        "})".into(),
    ]);
    // A live unmanaged hl.gesture already owns HORIZONTAL. Writing ours would
    // make Hyprland reject the second as "Previous HORIZONTAL shadows new
    // HORIZONTAL". Atmos defers and never comments the user line.
    if flag("workspaceGesture", false) && !has_unmanaged_workspace_gesture(existing) {
        lines.push(
            r#"hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })"#.into(),
        );
    }
    lines.push(END.into());
    lines.join("\n")
}

pub fn parse(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let Some((start, stop)) = sentinel_bounds(text, BEGIN, END) else {
        return out;
    };
    let body = &text[start..stop];
    if let Ok(n) = lua_num_field(body, "sensitivity").parse::<f64>() {
        out.insert("sensitivity".into(), json!(n));
    }
    let accel = lua_str_field(body, "accel_profile");
    if !accel.is_empty() || body.contains("accel_profile") {
        out.insert("accelProfile".into(), json!(accel));
    }
    for (lua, key) in [
        ("emulate_discrete_scroll", "emulateDiscreteScroll"),
        ("drag_3fg", "drag3fg"),
        ("repeat_rate", "repeatRate"),
        ("repeat_delay", "repeatDelay"),
        ("follow_mouse", "followMouse"),
    ] {
        if let Some(v) = num_value(&lua_num_field(body, lua)) {
            out.insert(key.into(), v);
        }
    }
    for (lua, key) in [
        ("natural_scroll", "naturalScroll"),
        ("clickfinger_behavior", "clickfinger"),
        ("disable_while_typing", "disableWhileTyping"),
        ("numlock_by_default", "numlock"),
        ("key_press_enables_dpms", "keyPressDpms"),
        ("mouse_move_enables_dpms", "mouseMoveDpms"),
    ] {
        if let Some(v) = lua_bool_field(body, lua) {
            out.insert(key.into(), json!(v));
        }
    }
    if let Ok(n) = lua_num_field(body, "scroll_factor").parse::<f64>() {
        out.insert("scrollFactor".into(), json!(n));
    }
    for (lua, key) in [
        ("kb_layout", "kbLayoutOverride"),
        ("kb_variant", "kbVariantOverride"),
    ] {
        let v = lua_str_field(body, lua);
        if !v.is_empty() || body.contains(lua) {
            out.insert(key.into(), json!(v));
        }
    }
    let options = lua_str_field(body, "kb_options");
    if !options.is_empty() || body.contains("kb_options") {
        out.insert(
            "kbGroupToggle".into(),
            json!(options.contains("grp:alts_toggle")),
        );
    }
    if re!(r#"action\s*=\s*["']workspace["']"#).is_match(body) {
        out.insert("workspaceGesture".into(), json!(true));
    }
    out
}

fn sentinel_has_workspace_gesture(src: &str) -> bool {
    let mut i = 0;
    while i < src.len() {
        let Some(rel) = src[i..].find("hl.gesture(") else {
            return false;
        };
        let at = i + rel;
        if in_line_comment(src, at) {
            i = at + 11;
            continue;
        }
        let end = src[at..].find(')').map_or(src.len(), |e| at + e + 1);
        // Any live workspace action means defer. Hyprland clashes on
        // HORIZONTAL, so a vertical/left/right workspace line over-defers and
        // blocks a non-shadowing Atmos swipe. That is the rule: do not emit a
        // second workspace gesture when one already exists.
        if re!(r#"action\s*=\s*["']workspace["']"#).is_match(&src[at..end]) {
            return true;
        }
        i = at + 11;
    }
    false
}

pub fn has_workspace_gesture(text: &str) -> bool {
    for (begin, end) in [(BEGIN, END), (LEGACY_BEGIN, LEGACY_END)] {
        if let Some((start, stop)) = sentinel_bounds(text, begin, end) {
            return sentinel_has_workspace_gesture(&text[start..stop]);
        }
    }
    false
}

pub fn has_unmanaged_workspace_gesture(text: &str) -> bool {
    let outside = strip_sentinel(&strip_sentinel(text, BEGIN, END), LEGACY_BEGIN, LEGACY_END);
    sentinel_has_workspace_gesture(&outside)
}

pub fn gesture_state(text: &str) -> Value {
    let managed = has_workspace_gesture(text);
    let unmanaged = has_unmanaged_workspace_gesture(text);
    json!({
        "workspaceGesture": managed || unmanaged,
        "workspaceGestureManaged": managed,
        "workspaceGestureUnmanaged": unmanaged,
    })
}
