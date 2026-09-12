#!/usr/bin/env python3
"""Apply or strip Atmos sentinel blocks in Hyprland Lua files."""

from __future__ import annotations

import fcntl
import json
import os
import re
import sys
import tempfile
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
WORKSPACES_BEGIN = "-- atmos:workspaces begin"
WORKSPACES_END = "-- atmos:workspaces end"
MONITORS_BEGIN = "-- atmos:monitors begin"
MONITORS_END = "-- atmos:monitors end"
REQUIRE_LINE = 'require("hypr.atmos")'
LAYOUT_REQUIRE = 'require("hypr.atmos_layout")'
OMARCHY_LINE = 'require("default.hypr.omarchy")'
TOGGLES_LINE = 'require("default.hypr.toggles")'
WINDOW_CLASS = "dev.csfh.atmos"
PREFS_WINDOW_SEED = "\n".join(
    [
        "-- Tile the Atmos window like other apps.",
        f'o.window("{WINDOW_CLASS}", {{ tile = true }})',
    ]
)
FLOAT_SEED_LINES = {
    f'o.window("{WINDOW_CLASS}", {{ float = true }})',
    f'o.window("{WINDOW_CLASS}", {{ center = true }})',
    f'o.window("{WINDOW_CLASS}", {{ size = {{ 960, 680 }} }})',
}
FLOAT_SEED_COMMENT = "-- Float and center the Atmos window."
TILE_SEED_COMMENT = "-- Tile the Atmos window like other apps."
TILE_SEED_LINE = f'o.window("{WINDOW_CLASS}", {{ tile = true }})'
# The seed above used to float and center the window. Existing installs
# still carry those exact lines, either as the bare seed or under the
# packaging header comments; rewrite only those verbatim lines so a
# customized rule is never touched.
FLOAT_WINDOW_SEED = "\n".join(
    [
        "-- Float and center the Atmos window.",
        f'o.window("{WINDOW_CLASS}", {{ float = true }})',
        f'o.window("{WINDOW_CLASS}", {{ center = true }})',
        f'o.window("{WINDOW_CLASS}", {{ size = {{ 960, 680 }} }})',
    ]
)


def normalize_prefs_window_seed(text: str) -> str:
    src = text or ""
    if FLOAT_WINDOW_SEED in src:
        return src.replace(FLOAT_WINDOW_SEED, PREFS_WINDOW_SEED)
    if TILE_SEED_LINE in src:
        return src
    if FLOAT_SEED_COMMENT not in src:
        return src
    lines = src.split("\n")
    if not any(ln.strip() in FLOAT_SEED_LINES for ln in lines):
        return src
    kept = [ln for ln in lines if ln.strip() not in FLOAT_SEED_LINES]
    out = []
    inserted = False
    for ln in kept:
        out.append(ln)
        if not inserted and ln.strip() == FLOAT_SEED_COMMENT:
            out.append(TILE_SEED_LINE)
            inserted = True
    src = "\n".join(out)
    if not inserted:
        src = TILE_SEED_LINE + "\n" + src
    return src.replace(FLOAT_SEED_COMMENT, TILE_SEED_COMMENT)


def lua_number(n: float | int) -> str:
    value = float(n)
    if abs(value - round(value)) < 1e-9:
        return str(int(round(value)))
    text = f"{round(value * 1000) / 1000:.3f}".rstrip("0").rstrip(".")
    return text or "0"


def lua_bool(v: bool) -> str:
    return "true" if v else "false"


def unescape_lua(raw: str) -> str:
    src = raw or ""
    out = []
    i = 0
    while i < len(src):
        if src[i] == "\\" and i + 1 < len(src):
            nxt = src[i + 1]
            if nxt == "n":
                out.append("\n")
            elif nxt == "t":
                out.append("\t")
            elif nxt == "r":
                out.append("\r")
            else:
                out.append(nxt)
            i += 2
            continue
        out.append(src[i])
        i += 1
    return "".join(out)


def lua_string(v: str) -> str:
    return (
        '"'
        + str(v)
        .replace("\\", "\\\\")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t")
        .replace('"', '\\"')
        + '"'
    )


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
        "inactiveOpacity": clamp_float(src.get("inactiveOpacity"), 0.2, 1, 1),
        "preserveSplit": as_bool(src.get("preserveSplit"), False),
        "focusOnActivate": as_bool(src.get("focusOnActivate"), False),
        "enableSwallow": as_bool(src.get("enableSwallow"), False),
        "swallowRegex": sanitize_swallow_regex(src.get("swallowRegex")),
        "onFocusUnderFullscreen": clamp_int(src.get("onFocusUnderFullscreen"), 0, 2, 1),
    }
    misc = [
        f"    focus_on_activate = {lua_bool(s['focusOnActivate'])},",
        f"    enable_swallow = {lua_bool(s['enableSwallow'])},",
    ]
    if s["swallowRegex"]:
        misc.append(f"    swallow_regex = {lua_string(s['swallowRegex'])},")
    misc.append(f"    on_focus_under_fullscreen = {lua_number(s['onFocusUnderFullscreen'])},")
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
            f"    inactive_opacity = {lua_number(s['inactiveOpacity'])},",
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
            *misc,
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


def sanitize_swallow_regex(raw) -> str:
    text = str(raw or "")
    if "\n" in text or "\r" in text:
        return ""
    text = text.strip()
    if len(text) > 128:
        return ""
    return text


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


def serialize_input(raw: dict, existing: str = "") -> str:
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
    # A live unmanaged hl.gesture already owns HORIZONTAL. Writing ours
    # would make Hyprland reject the second as "Previous HORIZONTAL shadows
    # new HORIZONTAL". Option 2: Atmos defers and never comments the user line.
    if s["workspaceGesture"] and not input_has_unmanaged_workspace_gesture(existing):
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


def lock_path_for(dest: Path) -> Path:
    return dest.parent / (dest.name + ".atmos.lock")


def atomic_write_text(dest: Path, text: str) -> None:
    body = text if text.endswith("\n") or not text else text + "\n"
    dest.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix="." + dest.name + ".", dir=str(dest.parent))
    try:
        with os.fdopen(fd, "w") as fh:
            fh.write(body)
            fh.flush()
            try:
                os.fsync(fh.fileno())
            except OSError:
                pass
        os.replace(tmp, dest)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _lua_bool_field(body: str, name: str) -> bool | None:
    m = re.search(rf"{re.escape(name)}\s*=\s*(true|false)", body)
    if not m:
        return None
    return m.group(1) == "true"


def _lua_num_field(block: str, name: str) -> str:
    m = re.search(rf"{re.escape(name)}\s*=\s*(-?[0-9]+(?:\.[0-9]+)?)", block)
    return m.group(1) if m else ""


def _lua_str_field(block: str, name: str) -> str:
    m = re.search(rf'{re.escape(name)}\s*=\s*"((?:\\.|[^"\\])*)"', block)
    if not m:
        return ""
    return unescape_lua(m.group(1))


def parse_look_block(text: str) -> dict:
    bounds = sentinel_bounds(text or "", LOOK_BEGIN, LOOK_END)
    if not bounds:
        return {}
    body = text[bounds[0] : bounds[1]]
    out: dict = {}
    num_keys = (
        ("gaps_in", "gapsIn"),
        ("gaps_out", "gapsOut"),
        ("border_size", "borderSize"),
        ("rounding", "rounding"),
        ("dim_strength", "dimStrength"),
        ("active_opacity", "activeOpacity"),
        ("inactive_opacity", "inactiveOpacity"),
        ("column_width", "columnWidth"),
        ("on_focus_under_fullscreen", "onFocusUnderFullscreen"),
    )
    for lua_name, key in num_keys:
        raw = _lua_num_field(body, lua_name)
        if raw == "":
            continue
        try:
            out[key] = float(raw) if "." in raw else int(raw)
        except ValueError:
            continue
    bool_keys = (
        ("allow_tearing", "allowTearing"),
        ("resize_on_border", "resizeOnBorder"),
        ("dim_inactive", "dimInactive"),
        ("focus_on_activate", "focusOnActivate"),
        ("enable_swallow", "enableSwallow"),
        ("hide_on_key_press", "cursorHideOnKey"),
        ("preserve_split", "preserveSplit"),
    )
    for lua_name, key in bool_keys:
        val = _lua_bool_field(body, lua_name)
        if val is not None:
            out[key] = val
    for section, key in (("shadow", "shadow"), ("blur", "blur")):
        m = re.search(rf"{section}\s*=\s*\{{\s*enabled\s*=\s*(true|false)", body)
        if m:
            out[key] = m.group(1) == "true"
    m = re.search(r"animations\s*=\s*\{\s*enabled\s*=\s*(true|false)", body)
    if m:
        out["animations"] = m.group(1) == "true"
    layout = _lua_str_field(body, "layout")
    if layout:
        out["layout"] = layout
    swallow = _lua_str_field(body, "swallow_regex")
    if swallow or 'swallow_regex' in body:
        out["swallowRegex"] = swallow
    warp = re.search(r"warp_on_change_workspace\s*=\s*(true|false|1|0)", body)
    if warp:
        out["cursorWarp"] = warp.group(1) in ("true", "1")
    m = re.search(r'hl\.env\(\s*"HYPRCURSOR_SIZE"\s*,\s*"([^"]*)"\s*\)', body)
    if not m:
        m = re.search(r'hl\.env\(\s*"XCURSOR_SIZE"\s*,\s*"([^"]*)"\s*\)', body)
    if m:
        try:
            out["cursorSize"] = int(float(m.group(1)))
        except ValueError:
            pass
    return out


def parse_input_block(text: str) -> dict:
    bounds = sentinel_bounds(text or "", INPUT_BEGIN, INPUT_END)
    if not bounds:
        return {}
    body = text[bounds[0] : bounds[1]]
    out: dict = {}
    raw = _lua_num_field(body, "sensitivity")
    if raw != "":
        try:
            out["sensitivity"] = float(raw)
        except ValueError:
            pass
    accel = _lua_str_field(body, "accel_profile")
    if accel or "accel_profile" in body:
        out["accelProfile"] = accel
    for lua_name, key in (
        ("emulate_discrete_scroll", "emulateDiscreteScroll"),
        ("drag_3fg", "drag3fg"),
        ("repeat_rate", "repeatRate"),
        ("repeat_delay", "repeatDelay"),
        ("follow_mouse", "followMouse"),
    ):
        raw = _lua_num_field(body, lua_name)
        if raw == "":
            continue
        try:
            out[key] = float(raw) if "." in raw else int(raw)
        except ValueError:
            continue
    for lua_name, key in (
        ("natural_scroll", "naturalScroll"),
        ("clickfinger_behavior", "clickfinger"),
        ("disable_while_typing", "disableWhileTyping"),
        ("numlock_by_default", "numlock"),
        ("key_press_enables_dpms", "keyPressDpms"),
        ("mouse_move_enables_dpms", "mouseMoveDpms"),
    ):
        val = _lua_bool_field(body, lua_name)
        if val is not None:
            out[key] = val
    scroll = _lua_num_field(body, "scroll_factor")
    if scroll != "":
        try:
            out["scrollFactor"] = float(scroll)
        except ValueError:
            pass
    layout = _lua_str_field(body, "kb_layout")
    if layout or "kb_layout" in body:
        out["kbLayoutOverride"] = layout
    variant = _lua_str_field(body, "kb_variant")
    if variant or "kb_variant" in body:
        out["kbVariantOverride"] = variant
    options = _lua_str_field(body, "kb_options")
    if options or "kb_options" in body:
        out["kbGroupToggle"] = "grp:alts_toggle" in options
    if re.search(r"""action\s*=\s*["']workspace["']""", body):
        out["workspaceGesture"] = True
    return out


def split_patch(payload: dict | None) -> dict | None:
    if not isinstance(payload, dict):
        return None
    patch = payload.get("_patch")
    if isinstance(patch, dict):
        return {k: v for k, v in patch.items() if not str(k).startswith("_")}
    return None


def sanitize_command(raw) -> str:
    text = str(raw or "").strip()
    if not text or len(text) > 256:
        return ""
    if "\n" in text or "\r" in text:
        return ""
    return text


def in_line_comment(src: str, at: int) -> bool:
    line_start = src.rfind("\n", 0, at) + 1
    i = line_start
    in_str = False
    while i < at:
        ch = src[i]
        if in_str:
            if ch == "\\":
                i += 2
                continue
            if ch == '"':
                in_str = False
            i += 1
            continue
        if ch == '"':
            in_str = True
            i += 1
            continue
        if ch == "-" and i + 1 < len(src) and src[i + 1] == "-":
            return True
        i += 1
    return False


def sentinel_has_workspace_gesture(src: str) -> bool:
    text = src or ""
    i = 0
    while i < len(text):
        at = text.find("hl.gesture(", i)
        if at < 0:
            return False
        if in_line_comment(text, at):
            i = at + 11
            continue
        end = text.find(")", at)
        body = text[at : end + 1 if end >= 0 else len(text)]
        # Any live workspace action means defer. Hyprland clashes on
        # HORIZONTAL, so a vertical/left/right workspace line over-defers
        # and blocks a non-shadowing Atmos swipe. That is the rule: do not
        # emit a second workspace gesture when one already exists.
        if re.search(r"""action\s*=\s*["']workspace["']""", body):
            return True
        i = at + 11
    return False


def input_has_workspace_gesture(text: str) -> bool:
    src = text or ""
    bounds = sentinel_bounds(src, INPUT_BEGIN, INPUT_END)
    if bounds:
        return sentinel_has_workspace_gesture(src[bounds[0] : bounds[1]])
    bounds = sentinel_bounds(src, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END)
    if bounds:
        return sentinel_has_workspace_gesture(src[bounds[0] : bounds[1]])
    return False


def input_outside_sentinels(text: str) -> str:
    src = text or ""
    src = strip_sentinel(src, INPUT_BEGIN, INPUT_END)
    return strip_sentinel(src, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END)


def input_has_unmanaged_workspace_gesture(text: str) -> bool:
    return sentinel_has_workspace_gesture(input_outside_sentinels(text))


def input_workspace_gesture_state(text: str) -> dict:
    managed = input_has_workspace_gesture(text)
    unmanaged = input_has_unmanaged_workspace_gesture(text)
    return {
        "workspaceGesture": managed or unmanaged,
        "workspaceGestureManaged": managed,
        "workspaceGestureUnmanaged": unmanaged,
    }


def split_delay(raw: str) -> tuple[str, int]:
    text = str(raw or "").strip()
    match = re.fullmatch(r"sleep\s+(\d+)\s+&&\s+(.+)", text)
    if not match:
        return sanitize_command(text), 0
    delay = int(match.group(1))
    if delay < 0:
        delay = 0
    if delay > 600:
        delay = 600
    return sanitize_command(match.group(2)), delay


def join_delay(command: str, delay: int) -> str:
    cmd = sanitize_command(command)
    n = delay if isinstance(delay, int) else 0
    if n < 0:
        n = 0
    if n > 600:
        n = 600
    if not cmd:
        return ""
    if n > 0:
        return f"sleep {n} && {cmd}"
    return cmd


def parse_launch_calls(text: str, disabled: bool = False) -> list[dict]:
    src = text or ""
    out = []
    for match in re.finditer(r'o\.launch_on_start\(\s*"((?:\\.|[^"\\])*)"\s*\)', src):
        commented = in_line_comment(src, match.start())
        if disabled:
            if not commented:
                continue
        elif commented:
            continue
        command, delay = split_delay(unescape_lua(match.group(1)))
        if command:
            out.append({"command": command, "delay": delay, "enabled": not disabled})
    return out


def normalize_autostart_item(item):
    if isinstance(item, str):
        command, delay = split_delay(item)
        if not command:
            return None
        return {"command": command, "delay": delay, "enabled": True}
    if not isinstance(item, dict):
        return None
    command = sanitize_command(item.get("command"))
    if not command:
        return None
    delay = item.get("delay") or 0
    try:
        delay = int(delay)
    except (TypeError, ValueError):
        delay = 0
    if delay < 0:
        delay = 0
    if delay > 600:
        delay = 600
    return {"command": command, "delay": delay, "enabled": item.get("enabled") is not False}


def serialize_autostart(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    commands = src.get("items")
    if not isinstance(commands, list):
        commands = src.get("commands")
    if not isinstance(commands, list):
        commands = []
    lines = [AUTOSTART_BEGIN]
    for item in commands:
        row = normalize_autostart_item(item)
        if not row:
            continue
        cmd = join_delay(row["command"], row["delay"])
        if not cmd:
            continue
        line = f"o.launch_on_start({lua_string(cmd)})"
        if row["enabled"] is False:
            line = "-- " + line
        lines.append(line)
    lines.append(AUTOSTART_END)
    return "\n".join(lines)


def parse_autostart(text: str) -> list[dict]:
    bounds = sentinel_bounds(text, AUTOSTART_BEGIN, AUTOSTART_END)
    if bounds:
        start, stop = bounds
        managed = parse_launch_calls(text[start:stop])
        unmanaged = parse_launch_calls(text[:start] + "\n" + text[stop:])
        disabled = parse_launch_calls(text[start:stop], True)
    else:
        managed = []
        unmanaged = parse_launch_calls(text)
        disabled = []
    items = [
        {"command": row["command"], "delay": row["delay"], "enabled": True, "managed": False}
        for row in unmanaged
    ]
    items.extend(
        {"command": row["command"], "delay": row["delay"], "enabled": True, "managed": True}
        for row in managed
    )
    items.extend(
        {"command": row["command"], "delay": row["delay"], "enabled": False, "managed": True}
        for row in disabled
    )
    return items


def skip_ws(text: str, i: int) -> int:
    # Lua -- comments. Only called outside strings.
    while i < len(text):
        while i < len(text) and text[i] in " \t\r\n":
            i += 1
        if i + 1 >= len(text) or text[i : i + 2] != "--":
            break
        nl = text.find("\n", i + 2)
        i = len(text) if nl < 0 else nl + 1
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
            nxt = text[i + 1]
            if nxt == "n":
                out.append("\n")
            elif nxt == "t":
                out.append("\t")
            elif nxt == "r":
                out.append("\r")
            else:
                out.append(nxt)
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


def lua_stringish(value) -> str:
    if value is True:
        return "true"
    if value is False:
        return "false"
    if value is None:
        return ""
    return str(value)


def shell_quote(value) -> str:
    return "'" + lua_stringish(value).replace("'", "'\\''") + "'"


def command_from_arg(arg, description="") -> str:
    if isinstance(arg, str):
        return sanitize_command(arg)
    if not isinstance(arg, dict):
        return ""
    if arg.get("omarchy"):
        return sanitize_command("omarchy-launch-" + lua_stringish(arg.get("omarchy")))
    launch = arg.get("launch")
    focus = arg.get("focus")
    if launch and focus:
        wrapped = "uwsm-app -- " + lua_stringish(launch)
        return sanitize_command(
            "omarchy-launch-or-focus " + shell_quote(focus) + " " + shell_quote(wrapped)
        )
    if launch:
        return sanitize_command("uwsm-app -- " + lua_stringish(launch))
    webapp = arg.get("webapp")
    if webapp:
        if focus:
            return sanitize_command(
                "omarchy-launch-or-focus-webapp "
                + shell_quote(description)
                + " "
                + shell_quote(webapp)
            )
        return sanitize_command("omarchy-launch-webapp " + shell_quote(webapp))
    tui = arg.get("tui")
    if tui:
        if focus:
            return sanitize_command("omarchy-launch-or-focus-tui " + shell_quote(tui))
        return sanitize_command("omarchy-launch-tui " + shell_quote(tui))
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
            if in_line_comment(src, unbind_at):
                i = unbind_at + 10
                continue
            parsed = parse_call_args(src, unbind_at + 10)
            if not parsed:
                i = unbind_at + 10
                continue
            keys = sanitize_keys(parsed[0][0] if parsed[0] else "")
            if keys:
                events.append({"kind": "unbind", "keys": keys})
            i = parsed[1]
        else:
            if in_line_comment(src, bind_at):
                i = bind_at + 7
                continue
            parsed = parse_call_args(src, bind_at + 7)
            if not parsed:
                i = bind_at + 7
                continue
            args = parsed[0]
            keys = sanitize_keys(args[0] if args else "")
            raw_label = args[1] if len(args) > 1 and args[1] is not None else ""
            label = "" if (len(args) < 2 or args[1] is None) else sanitize_label(args[1])
            command = command_from_arg(args[2] if len(args) > 2 else "", raw_label)
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
    title = sanitize_match(row.get("title"))
    pin = row.get("pin") is True
    fullscreen = row.get("fullscreen") is True
    opacity = sanitize_opacity(row.get("opacity"))
    if (
        not placement
        and not center
        and not width
        and not workspace
        and not title
        and not pin
        and not fullscreen
        and not opacity
    ):
        return None
    return {
        "match": match,
        "title": title,
        "placement": placement,
        "center": center,
        "width": width,
        "height": height,
        "workspace": workspace,
        "pin": pin,
        "fullscreen": fullscreen,
        "opacity": opacity,
    }


def sanitize_opacity(raw) -> str:
    if raw is None or raw == "":
        return ""
    if isinstance(raw, (int, float)):
        n = float(raw)
        if n < 0.2 or n > 1:
            return ""
        return str(round(n * 100) / 100)
    text = str(raw).strip()
    if not re.fullmatch(r"[0-9.]+( [0-9.]+)?", text) or len(text) > 16:
        return ""
    return text


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
            "title": rules.get("title"),
            "float": rules.get("float") is True,
            "tile": rules.get("tile") is True,
            "center": rules.get("center") is True,
            "width": width,
            "height": height,
            "workspace": rules.get("workspace"),
            "pin": rules.get("pin") is True,
            "fullscreen": rules.get("fullscreen") is True,
            "opacity": rules.get("opacity"),
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
        if in_line_comment(src, at):
            i = at + 9
            continue
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
    if row.get("title"):
        parts.append(f"title = {lua_string(row['title'])}")
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
    if row.get("pin"):
        parts.append("pin = true")
    if row.get("fullscreen"):
        parts.append("fullscreen = true")
    if row.get("opacity"):
        parts.append(f"opacity = {lua_string(row['opacity'])}")
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


def layout_lua() -> str:
    path = Path(__file__).resolve().parent.parent / "packaging" / "hypr-atmos-layout.lua"
    return path.read_text()


def ensure_layout_file(dest: Path) -> None:
    body = layout_lua()
    if not body.endswith("\n"):
        body += "\n"
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and dest.read_text() == body:
        return
    atomic_write_text(dest, body)


def ensure_layout_require(text: str) -> str:
    if "hypr.atmos_layout" in text:
        return text
    if not text.strip():
        return text
    at = text.find(OMARCHY_LINE)
    if at >= 0:
        return text[:at] + LAYOUT_REQUIRE + "\n" + text[at:]
    at = text.find(REQUIRE_LINE)
    if at >= 0:
        return text[:at] + LAYOUT_REQUIRE + "\n" + text[at:]
    return text.rstrip() + "\n\n" + LAYOUT_REQUIRE + "\n"


def serialize_workspaces(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    items = src.get("items") if isinstance(src.get("items"), list) else []
    count = 10
    wrap = src.get("wrapSwitch") is not False
    wheel = src.get("wheelSwitch") is not False
    wrap_next = "r+1" if not wrap else "e+1"
    wrap_prev = "r-1" if not wrap else "e-1"
    lines = [
        WORKSPACES_BEGIN,
        "-- atmos:wrapSwitch = " + ("true" if wrap else "false"),
        "-- atmos:wheelSwitch = " + ("true" if wheel else "false"),
    ]
    seen = set()
    rows = []
    for item in items:
        if not isinstance(item, dict):
            continue
        wid = str(item.get("id") or item.get("workspace") or "").strip()
        if not wid or wid in seen:
            continue
        seen.add(wid)
        rows.append(item)
    for i in range(1, count + 1):
        sid = str(i)
        if sid not in seen:
            rows.append({"id": sid, "persistent": True})
            seen.add(sid)
    for item in rows:
        wid = str(item.get("id") or "").strip()
        if re.fullmatch(r"[1-9]|10", wid) and int(wid) > count:
            continue
        parts = [f"workspace = {lua_string(wid)}"]
        if item.get("persistent") is not False:
            parts.append("persistent = true")
        name = str(item.get("name") or "").strip()
        if name:
            parts.append(f"default_name = {lua_string(name)}")
        monitor = str(item.get("monitor") or "").strip()
        if monitor:
            parts.append(f"monitor = {lua_string(monitor)}")
        if item.get("isDefault") is True or item.get("default") is True:
            parts.append("default = true")
        empty = str(item.get("onCreatedEmpty") or item.get("on_created_empty") or "").strip()
        if empty:
            parts.append(f"on_created_empty = {lua_string(empty)}")
        lines.append("hl.workspace_rule({ " + ", ".join(parts) + " })")
    lines.append('hl.unbind("SUPER + TAB")')
    lines.append('hl.unbind("SUPER + SHIFT + TAB")')
    lines.append(
        f'o.bind("SUPER + TAB", "Next workspace", hl.dsp.focus({{ workspace = "{wrap_next}" }}))'
    )
    lines.append(
        f'o.bind("SUPER + SHIFT + TAB", "Previous workspace", hl.dsp.focus({{ workspace = "{wrap_prev}" }}))'
    )
    lines.append('hl.unbind("SUPER + mouse_down")')
    lines.append('hl.unbind("SUPER + mouse_up")')
    if wheel:
        lines.append(
            f'o.bind("SUPER + mouse_down", "Scroll active workspace forward", hl.dsp.focus({{ workspace = "{wrap_next}" }}))'
        )
        lines.append(
            f'o.bind("SUPER + mouse_up", "Scroll active workspace backward", hl.dsp.focus({{ workspace = "{wrap_prev}" }}))'
        )
    lines.append(WORKSPACES_END)
    return "\n".join(lines)


def serialize_monitors(raw: dict) -> str:
    src = raw if isinstance(raw, dict) else {}
    items = src.get("items") if isinstance(src.get("items"), list) else []
    lines = [MONITORS_BEGIN]
    for item in items:
        if not isinstance(item, dict):
            continue
        output = str(item.get("output") or "").strip()
        if not re.fullmatch(r"[A-Za-z0-9._-]+", output or ""):
            continue
        mode = str(item.get("mode") or "preferred")
        position = str(item.get("position") or "auto")
        try:
            scale = float(item.get("scale") or 1)
        except (TypeError, ValueError):
            scale = 1.0
        parts = [
            f"output = {lua_string(output)}",
            f"mode = {lua_string(mode)}",
            f"position = {lua_string(position)}",
            f"scale = {lua_number(scale)}",
        ]
        transform = int(item.get("transform") or 0)
        if transform:
            parts.append(f"transform = {transform}")
        if item.get("disabled") is True:
            parts.append("disabled = true")
        mirror = str(item.get("mirror") or "").strip()
        if mirror:
            parts.append(f"mirror = {lua_string(mirror)}")
        vrr = int(item.get("vrr") or 0)
        if vrr:
            parts.append(f"vrr = {vrr}")
        if int(item.get("bitdepth") or 8) == 10:
            parts.append("bitdepth = 10")
        cm = str(item.get("cm") or "").strip()
        if cm:
            parts.append(f"cm = {lua_string(cm)}")
        lines.append("hl.monitor({ " + ", ".join(parts) + " })")
    lines.append(MONITORS_END)
    return "\n".join(lines)


def _lua_str_field(body: str, name: str) -> str:
    m = re.search(rf'{name}\s*=\s*"((?:\\.|[^"\\])*)"', body)
    return m.group(1) if m else ""


def _lua_num_field(body: str, name: str) -> str:
    m = re.search(rf"{name}\s*=\s*([0-9.]+)", body)
    return m.group(1) if m else ""


def parse_workspaces(text: str) -> list:
    start = text.find(WORKSPACES_BEGIN)
    stop = text.find(WORKSPACES_END)
    body = text[start:stop] if start >= 0 and stop > start else ""
    out = []
    for m in re.finditer(r"hl\.workspace_rule\(\s*\{([^}]*)\}\s*\)", body):
        chunk = m.group(1)
        wid = _lua_str_field(chunk, "workspace")
        if not wid:
            continue
        row = {
            "id": wid,
            "name": _lua_str_field(chunk, "default_name"),
            "monitor": _lua_str_field(chunk, "monitor"),
            "persistent": bool(re.search(r"persistent\s*=\s*true", chunk)),
            "isDefault": bool(re.search(r"default\s*=\s*true", chunk)),
            "onCreatedEmpty": _lua_str_field(chunk, "on_created_empty"),
            "special": wid.startswith("special:"),
        }
        out.append(row)
    return out


def parse_monitors(text: str) -> list:
    start = text.find(MONITORS_BEGIN)
    stop = text.find(MONITORS_END)
    body = text[start:stop] if start >= 0 and stop > start else ""
    out = []
    for m in re.finditer(r"hl\.monitor\(\s*\{([^}]*)\}\s*\)", body):
        chunk = m.group(1)
        output = _lua_str_field(chunk, "output")
        if not output:
            continue
        scale_raw = _lua_num_field(chunk, "scale")
        try:
            scale = float(scale_raw) if scale_raw else 1.0
        except ValueError:
            scale = 1.0
        transform_raw = _lua_num_field(chunk, "transform")
        vrr_raw = _lua_num_field(chunk, "vrr")
        bit_raw = _lua_num_field(chunk, "bitdepth")
        row = {
            "output": output,
            "mode": _lua_str_field(chunk, "mode") or "preferred",
            "position": _lua_str_field(chunk, "position") or "auto",
            "scale": scale,
            "transform": int(float(transform_raw)) if transform_raw else 0,
            "disabled": bool(re.search(r"disabled\s*=\s*true", chunk)),
            "mirror": _lua_str_field(chunk, "mirror"),
            "vrr": int(float(vrr_raw)) if vrr_raw else 0,
            "bitdepth": int(float(bit_raw)) if bit_raw else 8,
            "cm": _lua_str_field(chunk, "cm"),
        }
        out.append(row)
    return out


def apply(kind: str, path: Path, payload: dict | None, reset: bool, _text: str | None = None) -> str:
    text = _text if _text is not None else (path.read_text() if path.exists() else "")
    if kind == "windows" and not text.strip():
        text = PREFS_WINDOW_SEED + "\n"
    if kind == "windows":
        text = normalize_prefs_window_seed(text)
    if kind == "look":
        text = strip_sentinel(text, LEGACY_LOOK_BEGIN, LEGACY_LOOK_END)
        begin, end, serialize = LOOK_BEGIN, LOOK_END, serialize_look
    elif kind == "autostart":
        begin, end, serialize = AUTOSTART_BEGIN, AUTOSTART_END, serialize_autostart
    elif kind == "bindings":
        begin, end, serialize = BINDINGS_BEGIN, BINDINGS_END, serialize_bindings
    elif kind == "windows":
        begin, end, serialize = WINDOWS_BEGIN, WINDOWS_END, serialize_windows
    elif kind == "workspaces":
        begin, end, serialize = WORKSPACES_BEGIN, WORKSPACES_END, serialize_workspaces
    elif kind == "monitors":
        begin, end, serialize = MONITORS_BEGIN, MONITORS_END, serialize_monitors
    else:
        text = strip_sentinel(text, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END)
        begin, end, serialize = INPUT_BEGIN, INPUT_END, serialize_input
    if reset:
        return strip_sentinel(text, begin, end)
    # A _patch payload carries only the fields one write changed. Merge
    # those onto what is on disk right now (under the caller's lock) so two
    # windows editing different fields do not clobber each other. A plain
    # object without _patch is a full managed block, as before.
    if kind == "input":
        patch = split_patch(payload)
        if patch is not None:
            base = parse_input_block(text)
            base.update(patch)
            if "workspaceGesture" not in patch and "workspaceGesture" not in base:
                base["workspaceGesture"] = input_has_workspace_gesture(text)
            return replace_sentinel(text, begin, end, serialize_input(base, text))
        return replace_sentinel(text, begin, end, serialize_input(payload or {}, text))
    if kind == "look":
        patch = split_patch(payload)
        if patch is not None:
            base = parse_look_block(text)
            base.update(patch)
            return replace_sentinel(text, begin, end, serialize_look(base))
    return replace_sentinel(text, begin, end, serialize(payload or {}))


def main() -> int:
    if len(sys.argv) < 4:
        print(
            "Usage: hypr-sentinel.py look|input|autostart|bindings|windows|workspaces|monitors|require apply|reset|list <file> [json]",
            file=sys.stderr,
        )
        return 2
    kind, action, dest = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    kinds = ("look", "input", "autostart", "bindings", "windows", "workspaces", "monitors", "require")
    if kind not in kinds or action not in ("apply", "reset", "list"):
        print(
            "hypr-sentinel.py: kind must be look|input|autostart|bindings|windows|workspaces|monitors|require and action apply|reset|list",
            file=sys.stderr,
        )
        return 2
    if kind == "require":
        if action != "apply":
            print("hypr-sentinel.py: require only supports apply", file=sys.stderr)
            return 2
        if not dest.exists():
            return 0
        dest.parent.mkdir(parents=True, exist_ok=True)
        with open(lock_path_for(dest), "a+") as lock:
            try:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
            except OSError:
                pass
            text = dest.read_text()
            if not text.strip():
                return 0
            ensure_layout_file(dest.parent / "atmos_layout.lua")
            updated = ensure_atmos_require(text)
            updated = ensure_layout_require(updated)
            if updated != text:
                atomic_write_text(dest, updated)
        return 0
    if action == "list":
        text = dest.read_text() if dest.exists() else ""
        if kind == "bindings":
            print(json.dumps(parse_bindings(text)))
        elif kind == "windows":
            print(json.dumps(parse_windows(text)))
        elif kind == "autostart":
            print(json.dumps(parse_autostart(text)))
        elif kind == "workspaces":
            print(json.dumps(parse_workspaces(text)))
        elif kind == "monitors":
            print(json.dumps(parse_monitors(text)))
        elif kind == "input":
            print(json.dumps(input_workspace_gesture_state(text)))
        else:
            print(
                "hypr-sentinel.py: list is for autostart|bindings|windows|workspaces|monitors|input",
                file=sys.stderr,
            )
            return 2
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
    # Serialize concurrent writers from every Atmos window (and imports)
    # through one exclusive lock per destination file, and publish with an
    # atomic rename so readers never see a torn mid-write file.
    with open(lock_path_for(dest), "a+") as lock:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        except OSError:
            pass
        text = dest.read_text() if dest.exists() else ""
        updated = apply(kind, dest, payload, reset=action == "reset", _text=text)
        if updated != text:
            atomic_write_text(dest, updated)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
