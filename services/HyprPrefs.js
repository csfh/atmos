// Managed Hyprland look/input blocks in ~/.config/hypr/{looknfeel,input}.lua.
// Prefs writes only the sentinel region. The rest of the user file stays.

var LOOK_BEGIN = "-- atmos:look begin";
var LOOK_END = "-- atmos:look end";
var INPUT_BEGIN = "-- atmos:input begin";
var INPUT_END = "-- atmos:input end";
var LEGACY_LOOK_BEGIN = "-- omarchy-prefs:look begin";
var LEGACY_LOOK_END = "-- omarchy-prefs:look end";
var LEGACY_INPUT_BEGIN = "-- omarchy-prefs:input begin";
var LEGACY_INPUT_END = "-- omarchy-prefs:input end";

function defaultLook() {
  return {
    gapsIn: 5,
    gapsOut: 10,
    borderSize: 2,
    rounding: 0,
    blur: false,
    shadow: false,
    layout: "dwindle",
    columnWidth: 0.49,
    dimInactive: false,
    dimStrength: 0.15,
    animations: true,
    cursorHideOnKey: true,
    cursorWarp: true,
    cursorSize: 24,
    allowTearing: false,
    resizeOnBorder: false,
    activeOpacity: 1,
    inactiveOpacity: 1,
    preserveSplit: false,
    focusOnActivate: false,
  };
}

function defaultInput() {
  return {
    sensitivity: 0,
    accelProfile: "",
    emulateDiscreteScroll: 1,
    naturalScroll: false,
    scrollFactor: 0.4,
    clickfinger: true,
    disableWhileTyping: true,
    drag3fg: 0,
    repeatRate: 40,
    repeatDelay: 250,
    numlock: true,
    followMouse: 1,
    keyPressDpms: true,
    mouseMoveDpms: true,
    kbLayoutOverride: "",
    kbVariantOverride: "",
    kbGroupToggle: false,
    workspaceGesture: false,
  };
}

function clampInt(raw, min, max, fallback) {
  var n = Math.round(Number(raw));
  if (!isFinite(n)) n = fallback;
  if (n < min) n = min;
  if (n > max) n = max;
  return n;
}

function clampFloat(raw, min, max, fallback) {
  var n = Number(raw);
  if (!isFinite(n)) n = fallback;
  if (n < min) n = min;
  if (n > max) n = max;
  return Math.round(n * 1000) / 1000;
}

function asBool(raw, fallback) {
  if (raw === true || raw === false) return raw;
  if (raw === "true" || raw === "on" || raw === 1 || raw === "1") return true;
  if (raw === "false" || raw === "off" || raw === 0 || raw === "0") return false;
  return fallback === true;
}

function clampLook(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var base = defaultLook();
  var layout = String(src.layout || base.layout);
  if (layout !== "scrolling") layout = "dwindle";
  return {
    gapsIn: clampInt(src.gapsIn, 0, 64, base.gapsIn),
    gapsOut: clampInt(src.gapsOut, 0, 64, base.gapsOut),
    borderSize: clampInt(src.borderSize, 0, 16, base.borderSize),
    rounding: clampInt(src.rounding, 0, 32, base.rounding),
    blur: asBool(src.blur, base.blur),
    shadow: asBool(src.shadow, base.shadow),
    layout: layout,
    columnWidth: clampFloat(src.columnWidth, 0.2, 1, base.columnWidth),
    dimInactive: asBool(src.dimInactive, base.dimInactive),
    dimStrength: clampFloat(src.dimStrength, 0, 1, base.dimStrength),
    animations: asBool(src.animations, base.animations),
    cursorHideOnKey: asBool(src.cursorHideOnKey, base.cursorHideOnKey),
    cursorWarp: asBool(src.cursorWarp, base.cursorWarp),
    cursorSize: clampInt(src.cursorSize, 8, 64, base.cursorSize),
    allowTearing: asBool(src.allowTearing, base.allowTearing),
    resizeOnBorder: asBool(src.resizeOnBorder, base.resizeOnBorder),
    activeOpacity: clampFloat(src.activeOpacity, 0.2, 1, base.activeOpacity),
    inactiveOpacity: clampFloat(src.inactiveOpacity, 0.2, 1, base.inactiveOpacity),
    preserveSplit: asBool(src.preserveSplit, base.preserveSplit),
    focusOnActivate: asBool(src.focusOnActivate, base.focusOnActivate),
  };
}

function sanitizeLayoutList(raw) {
  var text = String(raw || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!text) return "";
  var parts = text.split(",");
  var out = [];
  for (var i = 0; i < parts.length; i++) {
    var id = parts[i].replace(/^\s+|\s+$/g, "");
    if (!/^[a-z0-9]{1,8}$/.test(id)) return "";
    out.push(id);
  }
  return out.join(",");
}

function sanitizeVariantList(raw, layoutCount) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  var parts = text.split(",");
  if (layoutCount > 0 && parts.length !== layoutCount) return "";
  var out = [];
  for (var i = 0; i < parts.length; i++) {
    var id = parts[i].replace(/^\s+|\s+$/g, "");
    if (id && !/^[A-Za-z0-9_-]{1,32}$/.test(id)) return "";
    out.push(id);
  }
  return out.join(",");
}

function clampInput(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var base = defaultInput();
  var accel = String(src.accelProfile || "");
  if (accel !== "flat" && accel !== "adaptive") accel = "";
  var layouts = sanitizeLayoutList(src.kbLayoutOverride);
  var variants = layouts
    ? sanitizeVariantList(src.kbVariantOverride, layouts.split(",").length)
    : "";
  return {
    sensitivity: clampFloat(src.sensitivity, -1, 1, base.sensitivity),
    accelProfile: accel,
    emulateDiscreteScroll: clampInt(src.emulateDiscreteScroll, 0, 2, base.emulateDiscreteScroll),
    naturalScroll: asBool(src.naturalScroll, base.naturalScroll),
    scrollFactor: clampFloat(src.scrollFactor, 0.1, 3, base.scrollFactor),
    clickfinger: asBool(src.clickfinger, base.clickfinger),
    disableWhileTyping: asBool(src.disableWhileTyping, base.disableWhileTyping),
    drag3fg: clampInt(src.drag3fg, 0, 1, base.drag3fg),
    repeatRate: clampInt(src.repeatRate, 10, 100, base.repeatRate),
    repeatDelay: clampInt(src.repeatDelay, 100, 1000, base.repeatDelay),
    numlock: asBool(src.numlock, base.numlock),
    followMouse: clampInt(src.followMouse, 0, 3, base.followMouse),
    keyPressDpms: asBool(src.keyPressDpms, base.keyPressDpms),
    mouseMoveDpms: asBool(src.mouseMoveDpms, base.mouseMoveDpms),
    kbLayoutOverride: layouts,
    kbVariantOverride: variants,
    kbGroupToggle: asBool(src.kbGroupToggle, base.kbGroupToggle),
    workspaceGesture: asBool(src.workspaceGesture, base.workspaceGesture),
  };
}

function luaNumber(n) {
  if (!isFinite(n)) return "0";
  if (Math.abs(n - Math.round(n)) < 1e-9) return String(Math.round(n));
  var s = (Math.round(n * 1000) / 1000).toFixed(3);
  return s.replace(/\.?0+$/, "");
}

function luaBool(v) {
  return v ? "true" : "false";
}

function luaString(v) {
  return (
    '"' +
    String(v)
      .replace(/\\/g, "\\\\")
      .replace(/\n/g, "\\n")
      .replace(/\r/g, "\\r")
      .replace(/\t/g, "\\t")
      .replace(/"/g, '\\"') +
    '"'
  );
}

function serializeLook(raw) {
  var s = clampLook(raw);
  var lines = [
    LOOK_BEGIN,
    "hl.config({",
    "  general = {",
    "    gaps_in = " + luaNumber(s.gapsIn) + ",",
    "    gaps_out = " + luaNumber(s.gapsOut) + ",",
    "    border_size = " + luaNumber(s.borderSize) + ",",
    "    layout = " + luaString(s.layout) + ",",
    "    allow_tearing = " + luaBool(s.allowTearing) + ",",
    "    resize_on_border = " + luaBool(s.resizeOnBorder) + ",",
    "  },",
    "  decoration = {",
    "    rounding = " + luaNumber(s.rounding) + ",",
    "    shadow = {",
    "      enabled = " + luaBool(s.shadow) + ",",
    "    },",
    "    blur = {",
    "      enabled = " + luaBool(s.blur) + ",",
    "    },",
    "    dim_inactive = " + luaBool(s.dimInactive) + ",",
    "    dim_strength = " + luaNumber(s.dimStrength) + ",",
    "    active_opacity = " + luaNumber(s.activeOpacity) + ",",
    "    inactive_opacity = " + luaNumber(s.inactiveOpacity) + ",",
    "  },",
    "  animations = {",
    "    enabled = " + luaBool(s.animations) + ",",
    "  },",
    "  scrolling = {",
    "    column_width = " + luaNumber(s.columnWidth) + ",",
    "  },",
    "  dwindle = {",
    "    preserve_split = " + luaBool(s.preserveSplit) + ",",
    "  },",
    "  misc = {",
    "    focus_on_activate = " + luaBool(s.focusOnActivate) + ",",
    "  },",
    "  cursor = {",
    "    hide_on_key_press = " + luaBool(s.cursorHideOnKey) + ",",
    "    warp_on_change_workspace = " + (s.cursorWarp ? "1" : "0") + ",",
    "  },",
    "})",
    "hl.env(" + luaString("HYPRCURSOR_SIZE") + ", " + luaString(String(s.cursorSize)) + ")",
    "hl.env(" + luaString("XCURSOR_SIZE") + ", " + luaString(String(s.cursorSize)) + ")",
    LOOK_END,
  ];
  return lines.join("\n");
}

function serializeInput(raw) {
  var s = clampInput(raw);
  var inputLines = [
    "    sensitivity = " + luaNumber(s.sensitivity) + ",",
    "    repeat_rate = " + luaNumber(s.repeatRate) + ",",
    "    repeat_delay = " + luaNumber(s.repeatDelay) + ",",
    "    numlock_by_default = " + luaBool(s.numlock) + ",",
    "    follow_mouse = " + luaNumber(s.followMouse) + ",",
  ];
  if (s.accelProfile)
    inputLines.splice(1, 0, "    accel_profile = " + luaString(s.accelProfile) + ",");
  inputLines.push("    emulate_discrete_scroll = " + luaNumber(s.emulateDiscreteScroll) + ",");
  if (s.kbLayoutOverride) {
    inputLines.push("    kb_layout = " + luaString(s.kbLayoutOverride) + ",");
    if (s.kbVariantOverride)
      inputLines.push("    kb_variant = " + luaString(s.kbVariantOverride) + ",");
    var options = "compose:caps,shift:both_capslock_cancel";
    if (s.kbGroupToggle) options += ",grp:alts_toggle";
    inputLines.push("    kb_options = " + luaString(options) + ",");
  }
  inputLines.push("    touchpad = {");
  inputLines.push("      natural_scroll = " + luaBool(s.naturalScroll) + ",");
  inputLines.push("      clickfinger_behavior = " + luaBool(s.clickfinger) + ",");
  inputLines.push("      scroll_factor = " + luaNumber(s.scrollFactor) + ",");
  inputLines.push("      disable_while_typing = " + luaBool(s.disableWhileTyping) + ",");
  inputLines.push("      drag_3fg = " + luaNumber(s.drag3fg) + ",");
  inputLines.push("    },");
  var lines = [
    INPUT_BEGIN,
    "hl.config({",
    "  input = {",
    inputLines.join("\n"),
    "  },",
    "  misc = {",
    "    key_press_enables_dpms = " + luaBool(s.keyPressDpms) + ",",
    "    mouse_move_enables_dpms = " + luaBool(s.mouseMoveDpms) + ",",
    "  },",
    "})",
  ];
  if (s.workspaceGesture)
    lines.push('hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })');
  lines.push(INPUT_END);
  return lines.join("\n");
}

function sentinelBounds(text, begin, end) {
  var src = String(text || "");
  var start = src.indexOf(begin);
  if (start === -1) return null;
  var stop = src.indexOf(end, start + begin.length);
  if (stop === -1) return null;
  return { start: start, stop: stop + end.length };
}

function hasSentinel(text, begin, end) {
  return sentinelBounds(text, begin, end) !== null;
}

function extractSentinel(text, begin, end) {
  var bounds = sentinelBounds(text, begin, end);
  if (!bounds) return "";
  return String(text).substring(bounds.start, bounds.stop);
}

function stripSentinel(text, begin, end) {
  var src = String(text || "");
  var bounds = sentinelBounds(src, begin, end);
  if (!bounds) return src;
  var before = src.substring(0, bounds.start).replace(/\s+$/, "");
  var after = src.substring(bounds.stop).replace(/^\s+/, "");
  if (before && after) return before + "\n\n" + after;
  return before || after;
}

function replaceSentinel(text, begin, end, block) {
  var src = String(text || "");
  var body = String(block || "").replace(/\s+$/, "");
  var bounds = sentinelBounds(src, begin, end);
  if (!bounds) {
    var trimmed = src.replace(/\s+$/, "");
    return trimmed ? trimmed + "\n\n" + body + "\n" : body + "\n";
  }
  return src.substring(0, bounds.start) + body + src.substring(bounds.stop);
}

function applyLookFile(text, raw) {
  return replaceSentinel(
    stripSentinel(text, LEGACY_LOOK_BEGIN, LEGACY_LOOK_END),
    LOOK_BEGIN,
    LOOK_END,
    serializeLook(raw),
  );
}

function applyInputFile(text, raw) {
  return replaceSentinel(
    stripSentinel(text, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END),
    INPUT_BEGIN,
    INPUT_END,
    serializeInput(raw),
  );
}

function resetLookFile(text) {
  return stripSentinel(
    stripSentinel(text, LEGACY_LOOK_BEGIN, LEGACY_LOOK_END),
    LOOK_BEGIN,
    LOOK_END,
  );
}

function resetInputFile(text) {
  return stripSentinel(
    stripSentinel(text, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END),
    INPUT_BEGIN,
    INPUT_END,
  );
}

function parseCssFirst(css) {
  var text = String(css || "").replace(/^\s+|\s+$/g, "");
  if (!text) return NaN;
  var first = text.split(/\s+/)[0];
  var n = Number(first);
  return isFinite(n) ? n : NaN;
}

function parseHyprOption(raw) {
  var data = raw;
  if (typeof raw === "string") {
    try {
      data = JSON.parse(raw || "{}");
    } catch (e) {
      return null;
    }
  }
  if (!data || typeof data !== "object") return null;
  if (typeof data.int === "number") return data.int;
  if (typeof data.bool === "boolean") return data.bool;
  if (typeof data.float === "number") return data.float;
  if (typeof data.css === "string") {
    var css = parseCssFirst(data.css);
    if (isFinite(css)) return css;
  }
  if (typeof data.str === "string") {
    if (data.str === "[[EMPTY]]") return "";
    return data.str;
  }
  return null;
}

function lookFromHyprOptions(opts) {
  var src = opts && typeof opts === "object" ? opts : {};
  var base = defaultLook();
  function pick(key, fallback) {
    return src[key] === undefined || src[key] === null ? fallback : src[key];
  }
  return clampLook({
    gapsIn: pick("gapsIn", base.gapsIn),
    gapsOut: pick("gapsOut", base.gapsOut),
    borderSize: pick("borderSize", base.borderSize),
    rounding: pick("rounding", base.rounding),
    blur: pick("blur", base.blur),
    shadow: pick("shadow", base.shadow),
    layout: pick("layout", base.layout),
    columnWidth: pick("columnWidth", base.columnWidth),
    dimInactive: pick("dimInactive", base.dimInactive),
    dimStrength: pick("dimStrength", base.dimStrength),
    animations: pick("animations", base.animations),
    cursorHideOnKey: pick("cursorHideOnKey", base.cursorHideOnKey),
    cursorWarp: pick("cursorWarp", base.cursorWarp),
    cursorSize: pick("cursorSize", base.cursorSize),
    allowTearing: pick("allowTearing", base.allowTearing),
    resizeOnBorder: pick("resizeOnBorder", base.resizeOnBorder),
    activeOpacity: pick("activeOpacity", base.activeOpacity),
    inactiveOpacity: pick("inactiveOpacity", base.inactiveOpacity),
    preserveSplit: pick("preserveSplit", base.preserveSplit),
    focusOnActivate: pick("focusOnActivate", base.focusOnActivate),
  });
}

function inLineComment(src, at) {
  var lineStart = src.lastIndexOf("\n", at > 0 ? at - 1 : 0) + 1;
  var i = lineStart;
  var inStr = false;
  while (i < at) {
    var c = src.charAt(i);
    if (inStr) {
      if (c === "\\") {
        i += 2;
        continue;
      }
      if (c === '"') inStr = false;
      i++;
      continue;
    }
    if (c === '"') {
      inStr = true;
      i++;
      continue;
    }
    if (c === "-" && src.charAt(i + 1) === "-") return true;
    i++;
  }
  return false;
}

function sentinelHasWorkspaceGesture(src) {
  var text = String(src || "");
  var i = 0;
  while (i < text.length) {
    var at = text.indexOf("hl.gesture(", i);
    if (at === -1) return false;
    if (inLineComment(text, at)) {
      i = at + 11;
      continue;
    }
    var end = text.indexOf(")", at);
    var body = text.substring(at, end === -1 ? text.length : end + 1);
    if (/action\s*=\s*["']workspace["']/.test(body)) return true;
    i = at + 11;
  }
  return false;
}

// Live hl.gesture in the managed input block. A commented example or an
// unmanaged line elsewhere in input.lua is not the Atmos toggle.
function inputHasWorkspaceGesture(text) {
  var src = extractSentinel(text, INPUT_BEGIN, INPUT_END);
  if (src) return sentinelHasWorkspaceGesture(src);
  src = extractSentinel(text, LEGACY_INPUT_BEGIN, LEGACY_INPUT_END);
  if (src) return sentinelHasWorkspaceGesture(src);
  return false;
}
