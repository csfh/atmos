// Managed hl.monitor lines in ~/.config/hypr/monitors.lua.

var BEGIN = "-- atmos:monitors begin";
var END = "-- atmos:monitors end";

function luaString(v) {
  return '"' + String(v).replace(/\\/g, "\\\\").replace(/"/g, '\\"') + '"';
}

function luaBool(v) {
  return v ? "true" : "false";
}

function luaNumber(n) {
  if (!isFinite(n)) return "0";
  if (Math.abs(n - Math.round(n)) < 1e-9) return String(Math.round(n));
  return String(Math.round(n * 1000) / 1000);
}

function sanitizeOutput(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 64) return "";
  if (!/^[A-Za-z0-9._-]+$/.test(text)) return "";
  return text;
}

function modeFromHyprctl(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  text = text.replace(/Hz$/i, "");
  return sanitizeMode(text);
}

function sanitizeMode(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "preferred";
  if (text === "preferred" || text === "highres" || text === "highrr") return text;
  if (!/^[0-9]{3,5}x[0-9]{3,5}(@[0-9]+(\.[0-9]+)?)?$/.test(text)) return "";
  return text;
}

function sanitizePosition(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "auto";
  if (text === "auto" || text === "0x0") return text;
  if (/^-?[0-9]+x-?[0-9]+$/.test(text)) return text;
  return "auto";
}

function clampScale(raw) {
  var n = Number(raw);
  if (!isFinite(n) || n <= 0) n = 1;
  if (n > 4) n = 4;
  return Math.round(n * 1000) / 1000;
}

function clampTransform(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 0) n = 0;
  if (n > 7) n = 7;
  return n;
}

function clampVrr(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 0) n = 0;
  if (n > 3) n = 3;
  return n;
}

function clampBitdepth(raw) {
  var n = Math.round(Number(raw));
  if (n === 10) return 10;
  return 8;
}

function sanitizeCm(raw) {
  var text = String(raw || "");
  var allowed = ["", "auto", "srgb", "dcip3", "wide", "hdr", "hdredid"];
  return allowed.indexOf(text) === -1 ? "" : text;
}

function normalizeItem(row) {
  if (!row || typeof row !== "object") return null;
  var output = sanitizeOutput(row.output);
  if (!output) return null;
  var mode = sanitizeMode(row.mode);
  if (!mode) mode = "preferred";
  return {
    output: output,
    mode: mode,
    position: sanitizePosition(row.position),
    scale: clampScale(row.scale),
    transform: clampTransform(row.transform),
    disabled: row.disabled === true,
    mirror: sanitizeOutput(row.mirror),
    vrr: clampVrr(row.vrr),
    bitdepth: clampBitdepth(row.bitdepth),
    cm: sanitizeCm(row.cm),
  };
}

function clampState(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var list = Array.isArray(src.items) ? src.items : Array.isArray(src) ? src : [];
  var items = [];
  var seen = {};
  var i;
  for (i = 0; i < list.length; i++) {
    var row = normalizeItem(list[i]);
    if (!row || seen[row.output]) continue;
    seen[row.output] = true;
    items.push(row);
  }
  var layouts = src.layouts && typeof src.layouts === "object" ? src.layouts : {};
  return { items: items, layouts: layouts };
}

function serializeMonitor(row) {
  var parts = [
    "output = " + luaString(row.output),
    "mode = " + luaString(row.mode),
    "position = " + luaString(row.position),
    "scale = " + luaNumber(row.scale),
  ];
  if (row.transform) parts.push("transform = " + luaNumber(row.transform));
  if (row.disabled) parts.push("disabled = " + luaBool(true));
  if (row.mirror) parts.push("mirror = " + luaString(row.mirror));
  if (row.vrr) parts.push("vrr = " + luaNumber(row.vrr));
  if (row.bitdepth === 10) parts.push("bitdepth = 10");
  if (row.cm) parts.push("cm = " + luaString(row.cm));
  return "hl.monitor({ " + parts.join(", ") + " })";
}

function serialize(raw) {
  var s = clampState(raw);
  var lines = [BEGIN];
  var i;
  for (i = 0; i < s.items.length; i++) lines.push(serializeMonitor(s.items[i]));
  lines.push(END);
  return lines.join("\n");
}

function sentinelBounds(text) {
  var src = String(text || "");
  var start = src.indexOf(BEGIN);
  if (start === -1) return null;
  var stop = src.indexOf(END, start + BEGIN.length);
  if (stop === -1) return null;
  return { start: start, stop: stop + END.length };
}

function parseMonitors(text) {
  var src = String(text || "");
  var out = [];
  var re = /hl\.monitor\(\s*\{([^}]*)\}\s*\)/g;
  var m;
  while ((m = re.exec(src))) {
    var body = m[1];
    var row = {};
    var om = /output\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (om) row.output = om[1];
    var mm = /mode\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (mm) row.mode = mm[1];
    var pm = /position\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (pm) row.position = pm[1];
    var sm = /scale\s*=\s*([0-9.]+)/.exec(body);
    if (sm) row.scale = Number(sm[1]);
    var tm = /transform\s*=\s*([0-9]+)/.exec(body);
    if (tm) row.transform = Number(tm[1]);
    row.disabled = /disabled\s*=\s*true/.test(body);
    var mir = /mirror\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (mir) row.mirror = mir[1];
    var vr = /vrr\s*=\s*([0-9]+)/.exec(body);
    if (vr) row.vrr = Number(vr[1]);
    var bd = /bitdepth\s*=\s*([0-9]+)/.exec(body);
    if (bd) row.bitdepth = Number(bd[1]);
    var cm = /cm\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (cm) row.cm = cm[1];
    var n = normalizeItem(row);
    if (n) out.push(n);
  }
  return out;
}

function parseFile(text) {
  var bounds = sentinelBounds(text);
  var items = bounds ? parseMonitors(String(text).substring(bounds.start, bounds.stop)) : [];
  return clampState({ items: items });
}

function applyFile(text, raw) {
  var src = String(text || "");
  var body = serialize(raw).replace(/\s+$/, "");
  var bounds = sentinelBounds(src);
  if (!bounds) {
    var trimmed = src.replace(/\s+$/, "");
    return trimmed ? trimmed + "\n\n" + body + "\n" : body + "\n";
  }
  return src.substring(0, bounds.start) + body + src.substring(bounds.stop);
}

function layoutNames() {
  return ["desk", "laptop", "docked"];
}

function pickLayout(layouts, name) {
  var key = String(name || "");
  var src = layouts && typeof layouts === "object" ? layouts : {};
  return Array.isArray(src[key]) ? src[key] : [];
}

function liveMode(row) {
  var src = row && typeof row === "object" ? row : {};
  if (src.mode) return sanitizeMode(src.mode) || "preferred";
  var w = Math.round(Number(src.width));
  var h = Math.round(Number(src.height));
  var hz = Number(src.refreshRate);
  if (isFinite(w) && isFinite(h) && w >= 640 && h >= 480) {
    if (isFinite(hz) && hz > 0)
      return sanitizeMode(w + "x" + h + "@" + Math.round(hz)) || "preferred";
    return sanitizeMode(w + "x" + h) || "preferred";
  }
  return "preferred";
}

function layoutFromLive(name, live) {
  var key = String(name || "");
  var list = Array.isArray(live) ? live : [];
  var items = [];
  var i;
  for (i = 0; i < list.length; i++) {
    var src = list[i] && typeof list[i] === "object" ? list[i] : {};
    var output = sanitizeOutput(src.name || src.output);
    if (!output) continue;
    var internal = src.internal === true || /^eDP/i.test(output);
    var disabled = false;
    if (key === "laptop") disabled = !internal;
    else if (key === "docked") disabled = internal;
    var row = normalizeItem({
      output: output,
      mode: liveMode(src),
      position: "auto",
      scale: src.scale,
      transform: src.transform,
      disabled: disabled,
      vrr: src.vrr,
      bitdepth: src.bitdepth,
      cm: src.cm,
    });
    if (row) items.push(row);
  }
  return items;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    BEGIN: BEGIN,
    END: END,
    normalizeItem: normalizeItem,
    clampState: clampState,
    serialize: serialize,
    parseFile: parseFile,
    applyFile: applyFile,
    layoutNames: layoutNames,
    pickLayout: pickLayout,
    layoutFromLive: layoutFromLive,
    modeFromHyprctl: modeFromHyprctl,
    sanitizeMode: sanitizeMode,
  };
}
