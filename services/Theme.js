// Pure parsers for Omarchy theme files. QML imports this; Node tests eval it.

function parseColors(raw) {
  var result = {
    foreground: "#cacccc",
    background: "#101315",
    accent: "#cacccc",
    muted: "#707880",
    urgent: "#a55555",
    palette: {},
  };
  var foundAccent = false;
  var foundMuted = false;
  var loadedForeground = false;
  var loadedBackground = false;
  var color0Value = "";
  var color4Value = "";
  var color7Value = "";
  var color8Value = "";
  var lines = String(raw || "").split("\n");
  for (var i = 0; i < lines.length; i++) {
    var match = lines[i].match(/^\s*([A-Za-z0-9_-]+)\s*=\s*["']?(#[0-9A-Fa-f]{6})/);
    if (!match) continue;
    var key = match[1];
    var value = match[2];
    result.palette[key] = value;
    if (key === "foreground" || key === "fg") {
      result.foreground = value;
      loadedForeground = true;
    } else if (key === "background" || key === "bg") {
      result.background = value;
      loadedBackground = true;
    } else if (key === "accent") {
      result.accent = value;
      foundAccent = true;
    } else if (key === "muted") {
      result.muted = value;
      foundMuted = true;
    } else if (key === "color0") color0Value = value;
    else if (key === "color4") color4Value = value;
    else if (key === "color7") color7Value = value;
    else if (key === "color8") color8Value = value;
    else if (key === "red" || key === "color1" || key === "urgent") result.urgent = value;
  }
  if (!loadedBackground && color0Value.length > 0) result.background = color0Value;
  if (!loadedForeground && color7Value.length > 0) result.foreground = color7Value;
  if (!foundAccent && color4Value.length > 0) result.accent = color4Value;
  if (!foundMuted) result.muted = color8Value.length > 0 ? color8Value : result.foreground;
  return result;
}

function hexRgb(hex) {
  var s = String(hex || "").replace(/^#/, "");
  if (s.length !== 6) return null;
  var n = parseInt(s, 16);
  if (!isFinite(n)) return null;
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}

function hexDist(a, b) {
  var A = hexRgb(a);
  var B = hexRgb(b);
  if (!A || !B) return 999;
  var dr = A.r - B.r;
  var dg = A.g - B.g;
  var db = A.b - B.b;
  return Math.sqrt(dr * dr + dg * dg + db * db);
}

function lerpHex(a, b, t) {
  var A = hexRgb(a);
  var B = hexRgb(b);
  if (!A) return b || a || "#000000";
  if (!B) return a;
  var k = Number(t);
  if (!isFinite(k)) k = 0;
  if (k < 0) k = 0;
  if (k > 1) k = 1;
  function ch(x, y) {
    var n = Math.round(x + (y - x) * k);
    var s = n.toString(16);
    return s.length === 1 ? "0" + s : s;
  }
  return "#" + ch(A.r, B.r) + ch(A.g, B.g) + ch(A.b, B.b);
}

function heatHex(stops, t) {
  var list = Array.isArray(stops) ? stops : [];
  if (!list.length) return "#cacccc";
  if (list.length === 1) return list[0];
  var k = Number(t);
  if (!isFinite(k)) k = 0;
  if (k < 0) k = 0;
  if (k > 1) k = 1;
  var scaled = k * (list.length - 1);
  var i = Math.floor(scaled);
  if (i >= list.length - 1) return list[list.length - 1];
  return lerpHex(list[i], list[i + 1], scaled - i);
}

var SWATCH_KEYS = [
  "accent",
  "blue",
  "cyan",
  "green",
  "yellow",
  "orange",
  "magenta",
  "red",
  "brown",
  "bright_blue",
  "bright_cyan",
  "bright_green",
  "bright_yellow",
  "bright_magenta",
  "bright_red",
  "color4",
  "color6",
  "color2",
  "color3",
  "color5",
  "color1",
  "color12",
  "color14",
  "color10",
  "color11",
  "color13",
  "color9",
];

var STOP_KEYS = ["cyan", "blue", "magenta", "red", "yellow", "orange"];

function lookupHex(parsed, key) {
  if (!parsed) return "";
  if (parsed.palette && parsed.palette[key]) return parsed.palette[key];
  if (parsed[key]) return parsed[key];
  return "";
}

function chartSwatches(parsed) {
  var bg = (parsed && parsed.background) || "#000000";
  var seen = {};
  var out = [];
  function add(hex) {
    if (!hex) return;
    var key = String(hex).toLowerCase();
    if (seen[key]) return;
    if (hexDist(hex, bg) < 48) return;
    seen[key] = true;
    out.push(hex);
  }
  var i;
  for (i = 0; i < SWATCH_KEYS.length; i++) add(lookupHex(parsed, SWATCH_KEYS[i]));
  add(parsed && parsed.accent);
  add(parsed && parsed.urgent);
  add(parsed && parsed.foreground);
  if (!out.length) add("#cacccc");
  return out;
}

function chartStops(parsed) {
  var out = [];
  var seen = {};
  var i;
  var hex;
  for (i = 0; i < STOP_KEYS.length; i++) {
    hex = lookupHex(parsed, STOP_KEYS[i]);
    if (!hex) continue;
    if (seen[hex.toLowerCase()]) continue;
    seen[hex.toLowerCase()] = true;
    out.push(hex);
  }
  if (out.length < 2) {
    if (parsed && parsed.muted) out.push(parsed.muted);
    if (parsed && parsed.accent) out.push(parsed.accent);
    if (parsed && parsed.urgent) out.push(parsed.urgent);
  }
  if (out.length < 2) return chartSwatches(parsed).slice(0, 4);
  return out;
}

function copyHexList(list) {
  var src = Array.isArray(list) ? list : [];
  var out = [];
  var i;
  for (i = 0; i < src.length; i++) out.push(String(src[i]));
  return out;
}

function parseShell(raw) {
  var parsed = {};
  var text = String(raw || "");
  if (!text) return parsed;
  var lines = text.split("\n");
  var section = "";
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i].replace(/^\s+|\s+$/g, "");
    if (!line || line.charAt(0) === "#") continue;
    var sectionMatch = line.match(/^\[([A-Za-z0-9_-]+)\]\s*(#.*)?$/);
    if (sectionMatch) {
      section = sectionMatch[1];
      continue;
    }
    var stringKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*["']([^"']+)["']\s*(#.*)?$/);
    var numKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(-?\d+(?:\.\d+)?)\s*(#.*)?$/);
    var widthKv = line.match(
      /^([A-Za-z0-9_-]+)\s*=\s*(-?\d+(?:\.\d+)?(?:\s+-?\d+(?:\.\d+)?){1,3})\s*(#.*)?$/,
    );
    var bareKv = line.match(/^([A-Za-z0-9_-]+)\s*=\s*([A-Za-z][A-Za-z0-9_-]*)\s*(#.*)?$/);
    var kv = stringKv || numKv || widthKv || bareKv;
    if (!kv || !section) continue;
    parsed[section + "." + kv[1]] = kv[2];
  }
  return parsed;
}

function themeSlug(name) {
  return String(name || "")
    .replace(/<[^>]+>/g, "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase()
    .replace(/ /g, "-");
}

function themeNameFromSlug(slug, names) {
  var want = themeSlug(slug);
  if (!want) return "";
  var list = names || [];
  var i;
  for (i = 0; i < list.length; i++) {
    var item = list[i];
    var label =
      item && typeof item === "object" ? String(item.label || item.value || "") : String(item);
    if (themeSlug(label) === want) return label;
  }
  return want;
}

function themeFileCandidates(name, rel, home, omarchyPath) {
  var slug = themeSlug(name);
  if (!slug || slug.indexOf("/") !== -1 || slug.charAt(0) === ".") return [];
  rel = String(rel || "");
  if (!rel || rel.indexOf("..") !== -1) return [];
  home = home || "";
  omarchyPath = omarchyPath || "/usr/share/omarchy";
  return [
    home + "/.config/omarchy/themes/" + slug + "/" + rel,
    omarchyPath + "/themes/" + slug + "/" + rel,
  ];
}

function mergeShell(themeValues, userValues) {
  var merged = {};
  var key;
  var theme = themeValues || {};
  var user = userValues || {};
  for (key in theme) merged[key] = theme[key];
  for (key in user) merged[key] = user[key];
  return merged;
}

function copyMap(values) {
  var out = {};
  var key;
  var src = values || {};
  for (key in src) out[key] = src[key];
  return out;
}

// Live chrome comes from current/theme/{colors,shell}.toml plus the user
// shell.toml, already loaded into Theme properties. Hover preview must
// snapshot those values -- not reread a named theme directory on the way
// back, or in-place edits vanish.
function snapshotLiveTheme(colors, themeShellValues) {
  var c = colors || {};
  return {
    source: "live",
    colors: {
      foreground: c.foreground,
      background: c.background,
      accent: c.accent,
      muted: c.muted,
      urgent: c.urgent,
      swatches: copyHexList(c.swatches),
      stops: copyHexList(c.stops),
    },
    themeShellValues: copyMap(themeShellValues),
  };
}

function restoreLiveTheme(snapshot) {
  if (!snapshot || snapshot.source !== "live" || !snapshot.colors) return null;
  var c = snapshot.colors;
  return {
    source: "live",
    colors: {
      foreground: c.foreground,
      background: c.background,
      accent: c.accent,
      muted: c.muted,
      urgent: c.urgent,
      swatches: copyHexList(c.swatches),
      stops: copyHexList(c.stops),
    },
    themeShellValues: copyMap(snapshot.themeShellValues),
  };
}

function firstThemeFile(name, rel, home, readFn, omarchyPath) {
  var paths = themeFileCandidates(name, rel, home, omarchyPath);
  var i;
  var raw;
  for (i = 0; i < paths.length; i++) {
    raw = readFn ? readFn(paths[i]) : "";
    if (raw) return raw;
  }
  return "";
}

function numberToken(values, key, fallback) {
  var n = Number(values && values[key]);
  return isFinite(n) ? n : fallback;
}

function formatSeconds(n) {
  n = Number(n);
  if (!isFinite(n) || n < 0) n = 0;
  n = Math.round(n);
  if (n < 60) return n + "s";
  var minutes = Math.floor(n / 60);
  var seconds = n % 60;
  if (seconds === 0) return minutes + "m";
  return minutes + "m " + seconds + "s";
}
