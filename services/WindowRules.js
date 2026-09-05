// Managed o.window() lines in ~/.config/hypr/atmos.lua.

var BEGIN = "-- atmos:windows begin";
var END = "-- atmos:windows end";
var REQUIRE_LINE = 'require("hypr.atmos")';
var LAYOUT_REQUIRE = 'require("hypr.atmos_layout")';
var OMARCHY_LINE = 'require("default.hypr.omarchy")';
var TOGGLES = 'require("default.hypr.toggles")';
var WINDOW_CLASS = "dev.csfh.atmos";

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

function sanitizeMatch(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 128) return "";
  if (text.indexOf("]]") !== -1) return "";
  return text;
}

function sanitizeWorkspace(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (!/^[A-Za-z0-9:_-]{1,32}$/.test(text)) return "";
  return text;
}

function clampSize(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 100 || n > 4000) return 0;
  return n;
}

function skipWs(s, i) {
  // Lua -- comments. Only called outside strings.
  while (i < s.length) {
    while (i < s.length && /[ \t\r\n]/.test(s.charAt(i))) i++;
    if (s.charAt(i) !== "-" || s.charAt(i + 1) !== "-") break;
    var nl = s.indexOf("\n", i + 2);
    i = nl === -1 ? s.length : nl + 1;
  }
  return i;
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

function identCont(s, i) {
  return i < s.length && /[A-Za-z0-9_]/.test(s.charAt(i));
}

function parseLuaString(s, i) {
  if (s.charAt(i) !== '"') return null;
  i++;
  var out = "";
  while (i < s.length) {
    var c = s.charAt(i);
    if (c === "\\") {
      if (i + 1 >= s.length) return null;
      var e = s.charAt(i + 1);
      if (e === "n") out += "\n";
      else if (e === "t") out += "\t";
      else if (e === "r") out += "\r";
      else out += e;
      i += 2;
      continue;
    }
    if (c === '"') return { value: out, next: i + 1 };
    out += c;
    i++;
  }
  return null;
}

function parseLuaTable(s, i) {
  i = skipWs(s, i);
  if (s.charAt(i) !== "{") return null;
  i = skipWs(s, i + 1);
  var obj = {};
  var arr = [];
  while (i < s.length && s.charAt(i) !== "}") {
    i = skipWs(s, i);
    if (s.charAt(i) === "}") break;
    var ident = s.substring(i).match(/^[A-Za-z_][A-Za-z0-9_]*/);
    var j = ident ? skipWs(s, i + ident[0].length) : i;
    if (ident && s.charAt(j) === "=") {
      var named = parseLuaValue(s, j + 1);
      if (!named) return null;
      obj[ident[0]] = named.value;
      i = named.next;
    } else {
      var item = parseLuaValue(s, i);
      if (!item) return null;
      arr.push(item.value);
      i = item.next;
    }
    i = skipWs(s, i);
    if (s.charAt(i) === ",") i++;
  }
  if (s.charAt(i) !== "}") return null;
  if (arr.length && Object.keys(obj).length === 0) return { value: arr, next: i + 1 };
  return { value: obj, next: i + 1 };
}

function parseLuaValue(s, i) {
  i = skipWs(s, i);
  if (i >= s.length) return null;
  var c = s.charAt(i);
  if (c === '"') return parseLuaString(s, i);
  if (c === "{") return parseLuaTable(s, i);
  if (s.substring(i, i + 3) === "nil" && !identCont(s, i + 3)) return { value: null, next: i + 3 };
  if (s.substring(i, i + 4) === "true" && !identCont(s, i + 4)) return { value: true, next: i + 4 };
  if (s.substring(i, i + 5) === "false" && !identCont(s, i + 5))
    return { value: false, next: i + 5 };
  var num = s.substring(i).match(/^-?\d+(?:\.\d+)?/);
  if (num) return { value: Number(num[0]), next: i + num[0].length };
  return null;
}

function parseCallArgs(s, i) {
  var args = [];
  i = skipWs(s, i);
  while (i < s.length && s.charAt(i) !== ")") {
    var val = parseLuaValue(s, i);
    if (!val) return null;
    args.push(val.value);
    i = skipWs(s, val.next);
    if (s.charAt(i) === ",") i = skipWs(s, i + 1);
  }
  if (s.charAt(i) !== ")") return null;
  return { args: args, next: i + 1 };
}

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
    float: rules.float === true,
    tile: rules.tile === true,
    center: rules.center === true,
    width: width,
    height: height,
    workspace: rules.workspace,
  });
}

function parseCalls(text) {
  var src = String(text || "");
  var out = [];
  var i = 0;
  while (i < src.length) {
    var at = src.indexOf("o.window(", i);
    if (at === -1) break;
    if (inLineComment(src, at)) {
      i = at + 9;
      continue;
    }
    var parsed = parseCallArgs(src, at + 9);
    if (!parsed) {
      i = at + 9;
      continue;
    }
    var row = rowFromArgs(parsed.args);
    if (row) out.push(row);
    i = parsed.next;
  }
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
  if (!placement && !center && !width && !workspace) return null;
  return {
    match: match,
    placement: placement,
    center: center,
    width: width,
    height: height,
    workspace: workspace,
  };
}

function sentinelBounds(text) {
  var src = String(text || "");
  var start = src.indexOf(BEGIN);
  if (start === -1) return null;
  var stop = src.indexOf(END, start + BEGIN.length);
  if (stop === -1) return null;
  return { start: start, stop: stop + END.length };
}

function parseFile(text) {
  var src = String(text || "");
  var bounds = sentinelBounds(src);
  var managed = [];
  var unmanaged = [];
  if (bounds) {
    managed = parseCalls(src.substring(bounds.start, bounds.stop));
    unmanaged = parseCalls(src.substring(0, bounds.start) + "\n" + src.substring(bounds.stop));
  } else {
    unmanaged = parseCalls(src);
  }
  var items = [];
  var i;
  for (i = 0; i < unmanaged.length; i++) {
    items.push({
      match: unmanaged[i].match,
      placement: unmanaged[i].placement,
      center: unmanaged[i].center,
      width: unmanaged[i].width,
      height: unmanaged[i].height,
      workspace: unmanaged[i].workspace,
      managed: false,
    });
  }
  for (i = 0; i < managed.length; i++) {
    items.push({
      match: managed[i].match,
      placement: managed[i].placement,
      center: managed[i].center,
      width: managed[i].width,
      height: managed[i].height,
      workspace: managed[i].workspace,
      managed: true,
    });
  }
  return items;
}

function serializeRule(row) {
  var parts = [];
  if (row.placement === "float") parts.push("float = true");
  if (row.placement === "tile") parts.push("tile = true");
  if (row.center) parts.push("center = true");
  if (row.width && row.height) parts.push("size = { " + row.width + ", " + row.height + " }");
  if (row.workspace) parts.push("workspace = " + luaString(row.workspace));
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

function replaceSentinel(text, block) {
  var src = String(text || "");
  var body = String(block || "").replace(/\s+$/, "");
  var bounds = sentinelBounds(src);
  if (!bounds) {
    var trimmed = src.replace(/\s+$/, "");
    return trimmed ? trimmed + "\n\n" + body + "\n" : body + "\n";
  }
  return src.substring(0, bounds.start) + body + src.substring(bounds.stop);
}

function applyFile(text, items) {
  return replaceSentinel(text, serialize(items));
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
    "-- Float and center the Atmos window.",
    'o.window("' + WINDOW_CLASS + '", { float = true })',
    'o.window("' + WINDOW_CLASS + '", { center = true })',
    'o.window("' + WINDOW_CLASS + '", { size = { 960, 680 } })',
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
  return bits.join(" \u00b7 ");
}
