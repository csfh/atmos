#!/usr/bin/env python3
"""Apply or strip Atmos sentinel blocks in Hyprland Lua files."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

LOOK_BEGIN = "-- atmos:look begin"
LOOK_END = "-- atmos:look end"
INPUT_BEGIN = "-- atmos:input begin"
INPUT_END = "-- atmos:input end"
LEGACY_LOOK_BEGIN = "-- omarchy-prefs:look begin"
LEGACY_LOOK_END = "-- omarchy-prefs:look end"
LEGACY_INPUT_BEGIN = "-- omarchy-prefs:input begin"
LEGACY_INPUT_END = "-- omarchy-prefs:input end"
AUTOSTART_BEGIN = "-- atmos:autostart begin"
AUTOSTART_END = "-- atmos:autostart end"
BINDINGS_BEGIN = "-- atmos:bindings begin"
BINDINGS_END = "-- atmos:bindings end"
WINDOWS_BEGIN = "-- atmos:windows begin"
WINDOWS_END = "-- atmos:windows end"
REQUIRE_LINE = 'require("hypr.atmos")'
TOGGLES_LINE = 'require("default.hypr.toggles")'
WINDOW_CLASS = "dev.csfh.atmos"
PREFS_WINDOW_SEED = "\n".join(
    [
        "-- Float and center the Atmos window.",
        f'o.window("{WINDOW_CLASS}", {{ float = true }})',
        f'o.window("{WINDOW_CLASS}", {{ center = true }})',
        f'o.window("{WINDOW_CLASS}", {{ size = {{ 960, 680 }} }})',
    ]
)


def lua_number(n: float | int) -> str:
    value = float(n)
    if abs(value - round(value)) < 1e-9:
        return str(int(round(value)))
    text = f"{round(value * 1000) / 1000:.3f}".rstrip("0").rstrip(".")
    return text or "0"


def lua_bool(v: bool) -> str:
    return "true" if v else "false"


def lua_string(v: str) -> str:
    return '"' + str(v).replace("\\", "\\\\").replace('"', '\\"') + '"'


def clamp_int(raw, lo, hi, fallback):
    try:
        n = int(round(float(raw)))
    except (TypeError, ValueError):
        n = fallback
    return max(lo, min(hi, n))


def clamp_float(raw, lo, hi, fallback):
    try:
        n = float(raw)
    except (TypeError, ValueError):
        n = fallback
    n = max(lo, min(hi, n))
    return round(n * 1000) / 1000


def as_bool(raw, fallback):
    if raw is True or raw is False:
        return raw
    if raw in ("true", "on", 1, "1"):
        return True
    if raw in ("false", "off", 0, "0"):
        return False
    return bool(fallback)


def serialize_look(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    layout = src.get("layout") or "dwindle"
    if layout != "scrolling":
        layout = "dwindle"
    s = {
        "gapsIn": clamp_int(src.get("gapsIn"), 0, 64, 5),
        "gapsOut": clamp_int(src.get("gapsOut"), 0, 64, 10),
        "borderSize": clamp_int(src.get("borderSize"), 0, 16, 2),
        "rounding": clamp_int(src.get("rounding"), 0, 32, 0),
        "blur": as_bool(src.get("blur"), False),
        "shadow": as_bool(src.get("shadow"), False),
        "layout": layout,
        "columnWidth": clamp_float(src.get("columnWidth"), 0.2, 1, 0.49),
        "dimInactive": as_bool(src.get("dimInactive"), False),
        "dimStrength": clamp_float(src.get("dimStrength"), 0, 1, 0.15),
        "animations": as_bool(src.get("animations"), True),
        "cursorHideOnKey": as_bool(src.get("cursorHideOnKey"), True),
        "cursorWarp": as_bool(src.get("cursorWarp"), True),
        "cursorSize": clamp_int(src.get("cursorSize"), 8, 64, 24),
        "allowTearing": as_bool(src.get("allowTearing"), False),
        "resizeOnBorder": as_bool(src.get("resizeOnBorder"), False),
        "activeOpacity": clamp_float(src.get("activeOpacity"), 0.2, 1, 1),
        "preserveSplit": as_bool(src.get("preserveSplit"), False),
        "focusOnActivate": as_bool(src.get("focusOnActivate"), False),
    }
    return "\n".join(
        [
            LOOK_BEGIN,
            "hl.config({",
            "  general = {",
            f"    gaps_in = {lua_number(s['gapsIn'])},",
            f"    gaps_out = {lua_number(s['gapsOut'])},",
            f"    border_size = {lua_number(s['borderSize'])},",
            f"    layout = {lua_string(s['layout'])},",
            f"    allow_tearing = {lua_bool(s['allowTearing'])},",
            f"    resize_on_border = {lua_bool(s['resizeOnBorder'])},",
            "  },",
            "  decoration = {",
            f"    rounding = {lua_number(s['rounding'])},",
            "    shadow = {",
            f"      enabled = {lua_bool(s['shadow'])},",
            "    },",
            "    blur = {",
            f"      enabled = {lua_bool(s['blur'])},",
            "    },",
            f"    dim_inactive = {lua_bool(s['dimInactive'])},",
            f"    dim_strength = {lua_number(s['dimStrength'])},",
            f"    active_opacity = {lua_number(s['activeOpacity'])},",
            "  },",
            "  animations = {",
            f"    enabled = {lua_bool(s['animations'])},",
            "  },",
            "  scrolling = {",
            f"    column_width = {lua_number(s['columnWidth'])},",
            "  },",
            "  dwindle = {",
            f"    preserve_split = {lua_bool(s['preserveSplit'])},",
            "  },",
            "  misc = {",
            f"    focus_on_activate = {lua_bool(s['focusOnActivate'])},",
            "  },",
            "  cursor = {",
            f"    hide_on_key_press = {lua_bool(s['cursorHideOnKey'])},",
            f"    warp_on_change_workspace = {1 if s['cursorWarp'] else 0},",
            "  },",
            "})",
            f'hl.env({lua_string("HYPRCURSOR_SIZE")}, {lua_string(str(s["cursorSize"]))})',
            f'hl.env({lua_string("XCURSOR_SIZE")}, {lua_string(str(s["cursorSize"]))})',
            LOOK_END,
        ]
    )


def sanitize_layouts(raw) -> str:
    text = str(raw or "").strip().lower()
    if not text:
        return ""
    parts = [p.strip() for p in text.split(",")]
    out = []
    for part in parts:
        if not re.fullmatch(r"[a-z0-9]{1,8}", part or ""):
            return ""
        out.append(part)
    return ",".join(out)


def sanitize_variants(raw, layout_count: int) -> str:
    text = str(raw or "").strip()
    if not text:
        return ""
    parts = [p.strip() for p in text.split(",")]
    if layout_count and len(parts) != layout_count:
        return ""
    for part in parts:
        if part and not re.fullmatch(r"[A-Za-z0-9_-]{1,32}", part):
            return ""
    return ",".join(parts)


def serialize_input(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    accel = str(src.get("accelProfile") or "")
    if accel not in ("flat", "adaptive"):
        accel = ""
    layouts = sanitize_layouts(src.get("kbLayoutOverride"))
    variants = sanitize_variants(src.get("kbVariantOverride"), len(layouts.split(",")) if layouts else 0)
    s = {
        "sensitivity": clamp_float(src.get("sensitivity"), -1, 1, 0),
        "accelProfile": accel,
        "emulateDiscreteScroll": clamp_int(src.get("emulateDiscreteScroll"), 0, 2, 1),
        "naturalScroll": as_bool(src.get("naturalScroll"), False),
        "scrollFactor": clamp_float(src.get("scrollFactor"), 0.1, 3, 0.4),
        "clickfinger": as_bool(src.get("clickfinger"), True),
        "disableWhileTyping": as_bool(src.get("disableWhileTyping"), True),
        "drag3fg": clamp_int(src.get("drag3fg"), 0, 1, 0),
        "repeatRate": clamp_int(src.get("repeatRate"), 10, 100, 40),
        "repeatDelay": clamp_int(src.get("repeatDelay"), 100, 1000, 250),
        "numlock": as_bool(src.get("numlock"), True),
        "followMouse": clamp_int(src.get("followMouse"), 0, 3, 1),
        "keyPressDpms": as_bool(src.get("keyPressDpms"), True),
        "mouseMoveDpms": as_bool(src.get("mouseMoveDpms"), True),
        "kbLayoutOverride": layouts,
        "kbVariantOverride": variants,
        "kbGroupToggle": as_bool(src.get("kbGroupToggle"), False),
        "workspaceGesture": as_bool(src.get("workspaceGesture"), False),
    }
    input_lines = [
        f"    sensitivity = {lua_number(s['sensitivity'])},",
        f"    repeat_rate = {lua_number(s['repeatRate'])},",
        f"    repeat_delay = {lua_number(s['repeatDelay'])},",
        f"    numlock_by_default = {lua_bool(s['numlock'])},",
        f"    follow_mouse = {lua_number(s['followMouse'])},",
    ]
    if s["accelProfile"]:
        input_lines.insert(1, f"    accel_profile = {lua_string(s['accelProfile'])},")
    input_lines.append(f"    emulate_discrete_scroll = {lua_number(s['emulateDiscreteScroll'])},")
    if s["kbLayoutOverride"]:
        input_lines.append(f"    kb_layout = {lua_string(s['kbLayoutOverride'])},")
        if s["kbVariantOverride"]:
            input_lines.append(f"    kb_variant = {lua_string(s['kbVariantOverride'])},")
        options = "compose:caps,shift:both_capslock_cancel"
        if s["kbGroupToggle"]:
            options += ",grp:alts_toggle"
        input_lines.append(f"    kb_options = {lua_string(options)},")
    input_lines.extend(
        [
            "    touchpad = {",
            f"      natural_scroll = {lua_bool(s['naturalScroll'])},",
            f"      clickfinger_behavior = {lua_bool(s['clickfinger'])},",
            f"      scroll_factor = {lua_number(s['scrollFactor'])},",
            f"      disable_while_typing = {lua_bool(s['disableWhileTyping'])},",
            f"      drag_3fg = {lua_number(s['drag3fg'])},",
            "    },",
        ]
    )
    lines = [
        INPUT_BEGIN,
        "hl.config({",
        "  input = {",
        *input_lines,
        "  },",
        "  misc = {",
        f"    key_press_enables_dpms = {lua_bool(s['keyPressDpms'])},",
        f"    mouse_move_enables_dpms = {lua_bool(s['mouseMoveDpms'])},",
        "  },",
        "})",
    ]
    if s["workspaceGesture"]:
        lines.append('hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })')
    lines.append(INPUT_END)
    return "\n".join(lines)


def sentinel_bounds(text: str, begin: str, end: str):
    start = text.find(begin)
    if start < 0:
        return None
    stop = text.find(end, start + len(begin))
    if stop < 0:
        return None
    return start, stop + len(end)


def strip_sentinel(text: str, begin: str, end: str) -> str:
    bounds = sentinel_bounds(text, begin, end)
    if not bounds:
        return text
    start, stop = bounds
    before = text[:start].rstrip()
    after = text[stop:].lstrip()
    if before and after:
        return before + "\n\n" + after
    return before or after


def replace_sentinel(text: str, begin: str, end: str, block: str) -> str:
    body = block.rstrip()
    bounds = sentinel_bounds(text, begin, end)
    if not bounds:
        trimmed = text.rstrip()
        return (trimmed + "\n\n" + body + "\n") if trimmed else body + "\n"
    start, stop = bounds
    return text[:start] + body + text[stop:]


def sanitize_command(raw) -> str:
    text = str(raw or "").strip()
    if not text or len(text) > 256:
        return ""
    if "\n" in text or "\r" in text:
        return ""
    return text


def parse_launch_calls(text: str) -> list[str]:
    out = []
    for match in re.finditer(r'o\.launch_on_start\(\s*"((?:\\.|[^"\\])*)"\s*\)', text or ""):
        cmd = sanitize_command(match.group(1).replace('\\"', '"').replace("\\\\", "\\"))
        if cmd:
            out.append(cmd)
    return out


def serialize_autostart(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    commands = src.get("commands")
    if not isinstance(commands, list):
        commands = []
    lines = [AUTOSTART_BEGIN]
    for item in commands:
        cmd = sanitize_command(item)
        if cmd:
            lines.append(f"o.launch_on_start({lua_string(cmd)})")
    lines.append(AUTOSTART_END)
    return "\n".join(lines)


def parse_autostart(text: str) -> list[dict]:
    bounds = sentinel_bounds(text, AUTOSTART_BEGIN, AUTOSTART_END)
    if bounds:
        start, stop = bounds
        managed = parse_launch_calls(text[start:stop])
        unmanaged = parse_launch_calls(text[:start] + "\n" + text[stop:])
    else:
        managed = []
        unmanaged = parse_launch_calls(text)
    items = [{"command": cmd, "managed": False} for cmd in unmanaged]
    items.extend({"command": cmd, "managed": True} for cmd in managed)
    return items


def skip_ws(text: str, i: int) -> int:
    while i < len(text) and text[i] in " \t\r\n":
        i += 1
    return i


def ident_cont(text: str, i: int) -> bool:
    return i < len(text) and (text[i].isalnum() or text[i] == "_")


def parse_lua_string(text: str, i: int):
    if i >= len(text) or text[i] != '"':
        return None
    i += 1
    out = []
    while i < len(text):
        ch = text[i]
        if ch == "\\":
            if i + 1 >= len(text):
                return None
            out.append(text[i + 1])
            i += 2
            continue
        if ch == '"':
            return "".join(out), i + 1
        out.append(ch)
        i += 1
    return None


def parse_lua_value(text: str, i: int):
    i = skip_ws(text, i)
    if i >= len(text):
        return None
    ch = text[i]
    if ch == '"':
        parsed = parse_lua_string(text, i)
        return parsed
    if ch == "{":
        return parse_lua_table(text, i)
    if text.startswith("nil", i) and not ident_cont(text, i + 3):
        return None, i + 3
    if text.startswith("true", i) and not ident_cont(text, i + 4):
        return True, i + 4
    if text.startswith("false", i) and not ident_cont(text, i + 5):
        return False, i + 5
    match = re.match(r"-?\d+(?:\.\d+)?", text[i:])
    if match:
        raw = match.group(0)
        value = float(raw) if "." in raw else int(raw)
        return value, i + len(raw)
    return None


def parse_lua_table(text: str, i: int):
    i = skip_ws(text, i)
    if i >= len(text) or text[i] != "{":
        return None
    i = skip_ws(text, i + 1)
    obj = {}
    arr = []
    while i < len(text) and text[i] != "}":
        i = skip_ws(text, i)
        if i < len(text) and text[i] == "}":
            break
        ident = re.match(r"[A-Za-z_][A-Za-z0-9_]*", text[i:])
        j = skip_ws(text, i + (len(ident.group(0)) if ident else 0))
        if ident and j < len(text) and text[j] == "=":
            parsed = parse_lua_value(text, j + 1)
            if not parsed:
                return None
            obj[ident.group(0)] = parsed[0]
            i = parsed[1]
        else:
            parsed = parse_lua_value(text, i)
            if not parsed:
                return None
            arr.append(parsed[0])
            i = parsed[1]
        i = skip_ws(text, i)
        if i < len(text) and text[i] == ",":
            i += 1
    if i >= len(text) or text[i] != "}":
        return None
    if arr and not obj:
        return arr, i + 1
    return obj, i + 1


def parse_call_args(text: str, i: int):
    args = []
    i = skip_ws(text, i)
    while i < len(text) and text[i] != ")":
        parsed = parse_lua_value(text, i)
        if not parsed:
            return None
        args.append(parsed[0])
        i = skip_ws(text, parsed[1])
        if i < len(text) and text[i] == ",":
            i = skip_ws(text, i + 1)
    if i >= len(text) or text[i] != ")":
        return None
    return args, i + 1


def sanitize_keys(raw) -> str:
    text = str(raw or "")
    if "\n" in text or "\r" in text:
        return ""
    text = re.sub(r"\s+", " ", text.strip())
    if not text or len(text) > 64:
        return ""
    if not re.fullmatch(r"[A-Za-z0-9_ +.:-]+", text):
        return ""
    return text


def sanitize_label(raw) -> str:
    text = str(raw or "")
    if "\n" in text or "\r" in text:
        return ""
    text = text.strip()
    if not text or len(text) > 64:
        return ""
    return text


def command_from_arg(arg) -> str:
    if isinstance(arg, str):
        return sanitize_command(arg)
    if isinstance(arg, dict) and arg.get("launch"):
        return sanitize_command(arg.get("launch"))
    return ""


def normalize_binding(row) -> dict | None:
    if not isinstance(row, dict):
        return None
    keys = sanitize_keys(row.get("keys"))
    if not keys:
        return None
    label = sanitize_label(row.get("label"))
    command = sanitize_command(row.get("command"))
    unbind = row.get("unbind") is True
    if not command and not unbind:
        return None
    return {"keys": keys, "label": label, "command": command, "unbind": unbind}


def parse_binding_events(text: str) -> list[dict]:
    src = text or ""
    events = []
    i = 0
    while i < len(src):
        unbind_at = src.find("hl.unbind(", i)
        bind_at = src.find("o.bind(", i)
        if unbind_at < 0 and bind_at < 0:
            break
        if unbind_at >= 0 and (bind_at < 0 or unbind_at < bind_at):
            parsed = parse_call_args(src, unbind_at + 10)
            if not parsed:
                i = unbind_at + 10
                continue
            keys = sanitize_keys(parsed[0][0] if parsed[0] else "")
            if keys:
                events.append({"kind": "unbind", "keys": keys})
            i = parsed[1]
        else:
            parsed = parse_call_args(src, bind_at + 7)
            if not parsed:
                i = bind_at + 7
                continue
            args = parsed[0]
            keys = sanitize_keys(args[0] if args else "")
            label = "" if (len(args) < 2 or args[1] is None) else sanitize_label(args[1])
            command = command_from_arg(args[2] if len(args) > 2 else "")
            if keys:
                events.append({"kind": "bind", "keys": keys, "label": label, "command": command})
            i = parsed[1]
    return events


def fold_binding_events(events: list[dict]) -> list[dict]:
    out = []
    i = 0
    while i < len(events):
        ev = events[i]
        if ev.get("kind") == "unbind":
            nxt = events[i + 1] if i + 1 < len(events) else None
            if nxt and nxt.get("kind") == "bind" and nxt.get("keys") == ev.get("keys"):
                out.append(
                    {
                        "keys": ev["keys"],
                        "label": nxt.get("label") or "",
                        "command": nxt.get("command") or "",
                        "unbind": True,
                    }
                )
                i += 2
                continue
            out.append({"keys": ev["keys"], "label": "", "command": "", "unbind": True})
            i += 1
            continue
        out.append(
            {
                "keys": ev.get("keys") or "",
                "label": ev.get("label") or "",
                "command": ev.get("command") or "",
                "unbind": False,
            }
        )
        i += 1
    return out


def parse_binding_calls(text: str) -> list[dict]:
    return fold_binding_events(parse_binding_events(text))


def parse_bindings(text: str) -> list[dict]:
    bounds = sentinel_bounds(text, BINDINGS_BEGIN, BINDINGS_END)
    if bounds:
        start, stop = bounds
        managed = parse_binding_calls(text[start:stop])
        unmanaged = parse_binding_calls(text[:start] + "\n" + text[stop:])
    else:
        managed = []
        unmanaged = parse_binding_calls(text)
    items = [{**row, "managed": False} for row in unmanaged]
    items.extend({**row, "managed": True} for row in managed)
    return items


def serialize_bindings(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    items = src.get("items")
    if not isinstance(items, list):
        items = []
    lines = [BINDINGS_BEGIN]
    for item in items:
        row = normalize_binding(item)
        if not row:
            continue
        if row["unbind"]:
            lines.append(f'hl.unbind({lua_string(row["keys"])})')
        if row["command"]:
            label_arg = lua_string(row["label"]) if row["label"] else "nil"
            lines.append(f'o.bind({lua_string(row["keys"])}, {label_arg}, {lua_string(row["command"])})')
    lines.append(BINDINGS_END)
    return "\n".join(lines)


def sanitize_match(raw) -> str:
    text = str(raw or "")
    if "\n" in text or "\r" in text:
        return ""
    text = text.strip()
    if not text or len(text) > 128 or "]]" in text:
        return ""
    return text


def sanitize_workspace(raw) -> str:
    text = str(raw or "").strip()
    if not text:
        return ""
    if not re.fullmatch(r"[A-Za-z0-9:_-]{1,32}", text):
        return ""
    return text


def clamp_size(raw) -> int:
    try:
        n = int(round(float(raw)))
    except (TypeError, ValueError):
        return 0
    if n < 100 or n > 4000:
        return 0
    return n


def normalize_window(row) -> dict | None:
    if not isinstance(row, dict):
        return None
    match = sanitize_match(row.get("match"))
    if not match:
        return None
    placement = str(row.get("placement") or "")
    if placement not in ("float", "tile"):
        placement = ""
    if row.get("float") is True:
        placement = "float"
    if row.get("tile") is True and placement != "float":
        placement = "tile"
    width = clamp_size(row.get("width"))
    height = clamp_size(row.get("height"))
    size = row.get("size")
    if isinstance(size, list) and len(size) >= 2:
        width = clamp_size(size[0])
        height = clamp_size(size[1])
    if not (width and height):
        width = 0
        height = 0
    workspace = sanitize_workspace(row.get("workspace"))
    center = row.get("center") is True
    if not placement and not center and not width and not workspace:
        return None
    return {
        "match": match,
        "placement": placement,
        "center": center,
        "width": width,
        "height": height,
        "workspace": workspace,
    }


def row_from_window_args(args) -> dict | None:
    if not args or not isinstance(args[0], str):
        return None
    rules = args[1] if len(args) > 1 and isinstance(args[1], dict) else {}
    width = 0
    height = 0
    size = rules.get("size")
    if isinstance(size, list) and len(size) >= 2:
        width = clamp_size(size[0])
        height = clamp_size(size[1])
    return normalize_window(
        {
            "match": args[0],
            "float": rules.get("float") is True,
            "tile": rules.get("tile") is True,
            "center": rules.get("center") is True,
            "width": width,
            "height": height,
            "workspace": rules.get("workspace"),
        }
    )


def parse_window_calls(text: str) -> list[dict]:
    src = text or ""
    out = []
    i = 0
    while i < len(src):
        at = src.find("o.window(", i)
        if at < 0:
            break
        parsed = parse_call_args(src, at + 9)
        if not parsed:
            i = at + 9
            continue
        row = row_from_window_args(parsed[0])
        if row:
            out.append(row)
        i = parsed[1]
    return out


def parse_windows(text: str) -> list[dict]:
    bounds = sentinel_bounds(text, WINDOWS_BEGIN, WINDOWS_END)
    if bounds:
        start, stop = bounds
        managed = parse_window_calls(text[start:stop])
        unmanaged = parse_window_calls(text[:start] + "\n" + text[stop:])
    else:
        managed = []
        unmanaged = parse_window_calls(text)
    items = [{**row, "managed": False} for row in unmanaged]
    items.extend({**row, "managed": True} for row in managed)
    return items


def serialize_window(row: dict) -> str:
    parts = []
    if row["placement"] == "float":
        parts.append("float = true")
    if row["placement"] == "tile":
        parts.append("tile = true")
    if row["center"]:
        parts.append("center = true")
    if row["width"] and row["height"]:
        parts.append(f"size = {{ {row['width']}, {row['height']} }}")
    if row["workspace"]:
        parts.append(f"workspace = {lua_string(row['workspace'])}")
    if not parts:
        return ""
    return f'o.window({lua_string(row["match"])}, {{ {", ".join(parts)} }})'


def serialize_windows(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    items = src.get("items")
    if not isinstance(items, list):
        items = []
    lines = [WINDOWS_BEGIN]
    for item in items:
        row = normalize_window(item)
        if not row:
            continue
        line = serialize_window(row)
        if line:
            lines.append(line)
    lines.append(WINDOWS_END)
    return "\n".join(lines)


def ensure_atmos_require(text: str) -> str:
    if "hypr.atmos" in text:
        return text
    if not text.strip():
        return text
    at = text.find(TOGGLES_LINE)
    if at >= 0:
        return text[:at] + REQUIRE_LINE + "\n" + text[at:]
    return text.rstrip() + "\n\n" + REQUIRE_LINE + "\n"


def apply(kind: str, path: Path, payload: dict | None, reset: bool) -> str:
    text = path.read_text() if path.exists() else ""
    if kind == "windows" and not text.strip():
        text = PREFS_WINDOW_SEED + "\n"
    if kind == "look":
        text = strip_sentinel(text, LEGACY_LOOK_BEGIN, LEGACY_LOOK_END)
        begin, end, serialize = LOOK_BEGIN, LOOK_END, serialize_look
    elif kind == "autostart":
        begin, end, serialize = AUTOSTART_BEGIN, AUTOSTART_END, serialize_autostart
    elif kind == "bindings":
        begin, end, serialize = BINDINGS_BEGIN, BINDINGS_END, serialize_bindings
    elif kind == "windows":
        begin, end, serialize = WINDOWS_BEGIN, WINDOWS_END, serialize_windows
    else:
        text = strip_sentinel(text, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END)
        begin, end, serialize = INPUT_BEGIN, INPUT_END, serialize_input
    if reset:
        return strip_sentinel(text, begin, end)
    return replace_sentinel(text, begin, end, serialize(payload or {}))


def main() -> int:
    if len(sys.argv) < 4:
        print(
            "Usage: hypr-sentinel.py look|input|autostart|bindings|windows|require apply|reset|list <file> [json]",
            file=sys.stderr,
        )
        return 2
    kind, action, dest = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    kinds = ("look", "input", "autostart", "bindings", "windows", "require")
    if kind not in kinds or action not in ("apply", "reset", "list"):
        print(
            "hypr-sentinel.py: kind must be look|input|autostart|bindings|windows|require and action apply|reset|list",
            file=sys.stderr,
        )
        return 2
    if kind == "require":
        if action != "apply":
            print("hypr-sentinel.py: require only supports apply", file=sys.stderr)
            return 2
        if not dest.exists():
            return 0
        text = dest.read_text()
        if not text.strip():
            return 0
        updated = ensure_atmos_require(text)
        dest.write_text(updated if updated.endswith("\n") or not updated else updated + "\n")
        return 0
    if action == "list":
        text = dest.read_text() if dest.exists() else ""
        if kind == "bindings":
            print(json.dumps(parse_bindings(text)))
        elif kind == "windows":
            print(json.dumps(parse_windows(text)))
        else:
            print(json.dumps(parse_autostart(text)))
        return 0
    payload = {}
    if action == "apply":
        raw = sys.argv[4] if len(sys.argv) > 4 else sys.stdin.read()
        try:
            payload = json.loads(raw or "{}")
        except json.JSONDecodeError as exc:
            print(f"hypr-sentinel.py: invalid JSON: {exc}", file=sys.stderr)
            return 2
        if not isinstance(payload, dict):
            print("hypr-sentinel.py: JSON object required", file=sys.stderr)
            return 2
    dest.parent.mkdir(parents=True, exist_ok=True)
    updated = apply(kind, dest, payload, reset=action == "reset")
    dest.write_text(updated if updated.endswith("\n") or not updated else updated + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
