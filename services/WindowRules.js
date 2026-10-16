Qt.include("LuaText.js");
Qt.include("RowText.js");
// Managed o.window() lines in ~/.config/hypr/atmos.lua.

var BEGIN = "-- atmos:windows begin";
var END = "-- atmos:windows end";
var REQUIRE_LINE = 'require("hypr.atmos")';
var LAYOUT_REQUIRE = 'require("hypr.atmos_layout")';
var OMARCHY_LINE = 'require("default.hypr.omarchy")';
var TOGGLES = 'require("default.hypr.toggles")';
var WINDOW_CLASS = "dev.csfh.atmos";

function rowFromArgs(args) {
  if (!args || typeof args[0] !== "string") return null;
  var rules = args[1] && typeof args[1] === "object" && !Array.isArray(args[1]) ? args[1] : {};
  var width = 0;
  var height = 0;
  if (Array.isArray(rules.size) && rules.size.length >= 2) {
    width = clampSize(rules.size[0]);
    height = clampSize(rules.size[1]);
  }
  return normalize({
    match: args[0],
    title: rules.title,
    float: rules.float === true,
    tile: rules.tile === true,
    center: rules.center === true,
    width: width,
    height: height,
    workspace: rules.workspace,
    pin: rules.pin === true,
    fullscreen: rules.fullscreen === true,
    opacity: rules.opacity,
  });
}

function parseCalls(text) {
  var out = [];
  liveCalls(text, "o.window(", function (args) {
    var row = rowFromArgs(args);
    if (row) out.push(row);
  });
  return out;
}

function normalize(row) {
  if (!row) return null;
  var match = sanitizeMatch(row.match);
  if (!match) return null;
  var placement = String(row.placement || "");
  if (placement !== "float" && placement !== "tile") placement = "";
  if (row.float === true) placement = "float";
  if (row.tile === true && placement !== "float") placement = "tile";
  var width = clampSize(row.width);
  var height = clampSize(row.height);
  if (Array.isArray(row.size) && row.size.length >= 2) {
    width = clampSize(row.size[0]);
    height = clampSize(row.size[1]);
  }
  if (!(width && height)) {
    width = 0;
    height = 0;
  }
  var workspace = sanitizeWorkspace(row.workspace);
  var center = row.center === true;
  var title = sanitizeMatch(row.title);
  var pin = row.pin === true;
  var fullscreen = row.fullscreen === true;
  var opacity = sanitizeOpacity(row.opacity);
  if (!placement && !center && !width && !workspace && !title && !pin && !fullscreen && !opacity)
    return null;
  return {
    match: match,
    title: title,
    placement: placement,
    center: center,
    width: width,
    height: height,
    workspace: workspace,
    pin: pin,
    fullscreen: fullscreen,
    opacity: opacity,
  };
}

function sanitizeOpacity(raw) {
  if (raw == null || raw === "") return "";
  if (typeof raw === "number") {
    var n = Number(raw);
    if (!isFinite(n) || n < 0.2 || n > 1) return "";
    return String(Math.round(n * 100) / 100);
  }
  var text = String(raw).replace(/^\s+|\s+$/g, "");
  if (!/^[0-9.]+( [0-9.]+)?$/.test(text) || text.length > 16) return "";
  return text;
}

function parseFile(text) {
  var src = String(text || "");
  var split = splitSentinel(src, BEGIN, END);
  var managed = split ? parseCalls(split.inside) : [];
  var unmanaged = parseCalls(split ? split.outside : src);
  var items = [];
  var i;
  for (i = 0; i < unmanaged.length; i++) {
    items.push({
      match: unmanaged[i].match,
      title: unmanaged[i].title,
      placement: unmanaged[i].placement,
      center: unmanaged[i].center,
      width: unmanaged[i].width,
      height: unmanaged[i].height,
      workspace: unmanaged[i].workspace,
      pin: unmanaged[i].pin,
      fullscreen: unmanaged[i].fullscreen,
      opacity: unmanaged[i].opacity,
      managed: false,
    });
  }
  for (i = 0; i < managed.length; i++) {
    items.push({
      match: managed[i].match,
      title: managed[i].title,
      placement: managed[i].placement,
      center: managed[i].center,
      width: managed[i].width,
      height: managed[i].height,
      workspace: managed[i].workspace,
      pin: managed[i].pin,
      fullscreen: managed[i].fullscreen,
      opacity: managed[i].opacity,
      managed: true,
    });
  }
  return items;
}

function serializeRule(row) {
  var parts = [];
  if (row.title) parts.push("title = " + luaString(row.title));
  if (row.placement === "float") parts.push("float = true");
  if (row.placement === "tile") parts.push("tile = true");
  if (row.center) parts.push("center = true");
  if (row.width && row.height) parts.push("size = { " + row.width + ", " + row.height + " }");
  if (row.workspace) parts.push("workspace = " + luaString(row.workspace));
  if (row.pin) parts.push("pin = true");
  if (row.fullscreen) parts.push("fullscreen = true");
  if (row.opacity) parts.push("opacity = " + luaString(row.opacity));
  if (!parts.length) return "";
  return "o.window(" + luaString(row.match) + ", { " + parts.join(", ") + " })";
}

function serialize(items) {
  var list = Array.isArray(items) ? items : [];
  var lines = [BEGIN];
  for (var i = 0; i < list.length; i++) {
    var row = normalize(typeof list[i] === "object" ? list[i] : null);
    if (!row) continue;
    var line = serializeRule(row);
    if (line) lines.push(line);
  }
  lines.push(END);
  return lines.join("\n");
}

function applyFile(text, items) {
  return replaceSentinel(text, BEGIN, END, serialize(items));
}

function managedItems(items) {
  var list = Array.isArray(items) ? items : [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var row = list[i];
    if (!row) continue;
    if (typeof row === "object" && row.managed === false) continue;
    var next = normalize(row);
    if (next) out.push(next);
  }
  return out;
}

function ensureRequire(text) {
  var src = String(text || "");
  if (src.indexOf("hypr.atmos") !== -1) return src;
  var trimmed = src.replace(/^\s+|\s+$/g, "");
  if (!trimmed) return src;
  var at = src.indexOf(TOGGLES);
  if (at !== -1) return src.substring(0, at) + REQUIRE_LINE + "\n" + src.substring(at);
  return src.replace(/\s+$/, "") + "\n\n" + REQUIRE_LINE + "\n";
}

function ensureLayoutRequire(text) {
  var src = String(text || "");
  if (src.indexOf("hypr.atmos_layout") !== -1) return src;
  var trimmed = src.replace(/^\s+|\s+$/g, "");
  if (!trimmed) return src;
  var at = src.indexOf(OMARCHY_LINE);
  if (at !== -1) return src.substring(0, at) + LAYOUT_REQUIRE + "\n" + src.substring(at);
  at = src.indexOf(REQUIRE_LINE);
  if (at !== -1) return src.substring(0, at) + LAYOUT_REQUIRE + "\n" + src.substring(at);
  return src.replace(/\s+$/, "") + "\n\n" + LAYOUT_REQUIRE + "\n";
}

function prefsSeed() {
  return [
    "-- Tile the Atmos window like other apps.",
    'o.window("' + WINDOW_CLASS + '", { tile = true })',
  ].join("\n");
}

function describe(row) {
  if (!row) return "";
  var bits = [];
  if (row.placement === "float") bits.push("float");
  if (row.placement === "tile") bits.push("tile");
  if (row.center) bits.push("center");
  if (row.width && row.height) bits.push(row.width + "\u00d7" + row.height);
  if (row.workspace) bits.push("workspace " + row.workspace);
  if (row.title) bits.push("title " + row.title);
  if (row.pin) bits.push("pin");
  if (row.fullscreen) bits.push("fullscreen");
  if (row.opacity) bits.push("opacity " + row.opacity);
  return bits.join(" \u00b7 ");
}
