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

fn put_f64(out: &mut Map<String, Value>, table: Option<&Map<String, Value>>, lua: &str, key: &str) {
    let Some(value) = table.and_then(|row| row.get(lua)).and_then(Value::as_f64) else {
        return;
    };
    out.insert(key.into(), json!(value));
}

fn workspace_gesture(body: &str) -> bool {
    let mut i = 0;
    let mark = "hl.gesture(";
    while let Some(rel) = body[i..].find(mark) {
        let at = i + rel;
        let Some((args, end)) = parse_call_args(body, at + mark.len()) else {
            i = at + mark.len();
            continue;
        };
        i = end;
        let action = args
            .first()
            .and_then(Value::as_object)
            .and_then(|row| row.get("action"))
            .and_then(Value::as_str);
        if action == Some("workspace") {
            return true;
        }
    }
    false
}

pub fn parse(text: &str) -> Map<String, Value> {
    let mut out = Map::new();
    let Some((start, stop)) = sentinel_bounds(text, BEGIN, END) else {
        return out;
    };
    let body = &text[start..stop];
    let config = first_table_arg(body, "hl.config(");
    let input = config.as_ref().and_then(|row| section(row, "input"));
    let touchpad = input.and_then(|row| section(row, "touchpad"));
    let misc = config.as_ref().and_then(|row| section(row, "misc"));
    put_f64(&mut out, input, "sensitivity", "sensitivity");
    put_text(&mut out, input, "accel_profile", "accelProfile", true);
    for (lua, key) in [
        ("emulate_discrete_scroll", "emulateDiscreteScroll"),
        ("repeat_rate", "repeatRate"),
        ("repeat_delay", "repeatDelay"),
        ("follow_mouse", "followMouse"),
    ] {
        put_num(&mut out, input, lua, key);
    }
    put_num(&mut out, touchpad, "drag_3fg", "drag3fg");
    for (lua, key) in [
        ("natural_scroll", "naturalScroll"),
        ("clickfinger_behavior", "clickfinger"),
        ("disable_while_typing", "disableWhileTyping"),
    ] {
        put_bool(&mut out, touchpad, lua, key);
    }
    put_bool(&mut out, input, "numlock_by_default", "numlock");
    put_f64(&mut out, touchpad, "scroll_factor", "scrollFactor");
    put_bool(&mut out, misc, "key_press_enables_dpms", "keyPressDpms");
    put_bool(&mut out, misc, "mouse_move_enables_dpms", "mouseMoveDpms");
    put_text(&mut out, input, "kb_layout", "kbLayoutOverride", true);
    put_text(&mut out, input, "kb_variant", "kbVariantOverride", true);
    if let Some(Value::String(options)) = input.and_then(|row| row.get("kb_options")) {
        out.insert(
            "kbGroupToggle".into(),
            json!(options.contains("grp:alts_toggle")),
        );
    }
    if workspace_gesture(body) {
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
