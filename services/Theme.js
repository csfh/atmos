// Pure parsers for Omarchy theme files. QML imports this; Node tests eval it.

function parseColors(raw) {
  var result = {
    foreground: "#cacccc",
    background: "#101315",
    accent: "#cacccc",
    muted: "#707880",
    urgent: "#a55555",
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

function mergeShell(themeValues, userValues) {
  var merged = {};
  var key;
  var theme = themeValues || {};
  var user = userValues || {};
  for (key in theme) merged[key] = theme[key];
  for (key in user) merged[key] = user[key];
  return merged;
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
