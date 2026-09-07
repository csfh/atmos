// Managed hl.workspace_rule lines in ~/.config/hypr/atmos.lua.

var BEGIN = "-- atmos:workspaces begin";
var END = "-- atmos:workspaces end";

function luaString(v) {
  return '"' + String(v).replace(/\\/g, "\\\\").replace(/\n/g, "\\n").replace(/"/g, '\\"') + '"';
}

function luaBool(v) {
  return v ? "true" : "false";
}

function clampCount(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 1) n = 10;
  if (n > 10) n = 10;
  return n;
}

function countFromItems(list) {
  var rows = Array.isArray(list) ? list : [];
  var max = 0;
  var i, n;
  for (i = 0; i < rows.length; i++) {
    n = Number(rows[i] && (rows[i].id || rows[i].workspace));
    if (isFinite(n) && n >= 1 && n <= 10 && n > max) max = n;
  }
  return max < 1 ? 10 : max;
}

function sanitizeId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (/^special:[A-Za-z0-9_-]{1,24}$/.test(text)) return text;
  if (/^[1-9][0-9]?$/.test(text) && Number(text) <= 10) return text;
  return "";
}

function sanitizeName(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (text.length > 32) return "";
  if (text && !/^[A-Za-z0-9 _.-]+$/.test(text)) return "";
  return text;
}

function sanitizeMonitor(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (!/^[A-Za-z0-9._-]+$/.test(text)) return "";
  return text;
}

function sanitizeCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 256) return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  return text;
}

function clampTracking(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 0) n = 0;
  if (n > 2) n = 2;
  return n;
}

function defaultItem(id) {
  return {
    id: String(id),
    name: "",
    persistent: true,
    monitor: "",
    isDefault: false,
    special: false,
    onCreatedEmpty: "",
  };
}

function defaultState() {
  var items = [];
  var i;
  for (i = 1; i <= 10; i++) items.push(defaultItem(String(i)));
  return {
    count: 10,
    wrapSwitch: true,
    wheelSwitch: true,
    tracking: 0,
    items: items,
  };
}

function normalizeItem(row) {
  if (!row || typeof row !== "object") return null;
  var id = sanitizeId(row.id || row.workspace);
  if (!id) return null;
  var special = id.indexOf("special:") === 0 || row.special === true;
  return {
    id: id,
    name: sanitizeName(row.name || row.default_name),
    persistent: special ? true : row.persistent !== false,
    monitor: sanitizeMonitor(row.monitor),
    isDefault: row.isDefault === true || row.default === true,
    special: special,
    onCreatedEmpty: sanitizeCommand(row.onCreatedEmpty || row.on_created_empty),
  };
}

function clampState(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var list = Array.isArray(src.items) ? src.items : [];
  var count = src.count == null || src.count === "" ? countFromItems(list) : clampCount(src.count);
  var items = [];
  var seen = {};
  var i;
  for (i = 0; i < list.length; i++) {
    var row = normalizeItem(list[i]);
    if (!row || seen[row.id]) continue;
    if (!row.special && Number(row.id) > count) continue;
    seen[row.id] = true;
    items.push(row);
  }
  for (i = 1; i <= count; i++) {
    var id = String(i);
    if (!seen[id]) items.push(defaultItem(id));
  }
  items.sort(function (a, b) {
    if (a.special !== b.special) return a.special ? 1 : -1;
    var an = Number(a.id);
    var bn = Number(b.id);
    if (isFinite(an) && isFinite(bn)) return an - bn;
    return a.id < b.id ? -1 : 1;
  });
  var defaultSeen = false;
  for (i = 0; i < items.length; i++) {
    if (items[i].special) {
      items[i].isDefault = false;
      continue;
    }
    if (!items[i].isDefault) continue;
    if (defaultSeen) items[i].isDefault = false;
    else defaultSeen = true;
  }
  return {
    count: count,
    wrapSwitch: src.wrapSwitch !== false,
    wheelSwitch: src.wheelSwitch !== false,
    tracking: clampTracking(src.tracking),
    items: items,
  };
}

function defaultId(items) {
  var list = Array.isArray(items) ? items : [];
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i] && list[i].isDefault && !list[i].special) return String(list[i].id);
  }
  return "";
}

function withDefault(items, id) {
  var list = Array.isArray(items) ? items : [];
  var want = String(id || "");
  var out = [];
  var i, row, copy, k;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row) continue;
    copy = {};
    for (k in row) copy[k] = row[k];
    copy.isDefault = !copy.special && copy.id === want && want.length > 0;
    out.push(copy);
  }
  return out;
}

function defaultOptions(items) {
  var list = Array.isArray(items) ? items : [];
  var out = [{ value: "", label: "Hyprland default" }];
  var i, row, label;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row || row.special) continue;
    label = row.name ? String(row.id) + " · " + row.name : String(row.id);
    out.push({ value: String(row.id), label: label });
  }
  return out;
}

function monitorOptions(monitors, current) {
  var out = [{ value: "", label: "Any" }];
  var seen = { "": true };
  var list = Array.isArray(monitors) ? monitors : [];
  var i, name;
  for (i = 0; i < list.length; i++) {
    name = String((list[i] && (list[i].name || list[i].id)) || "");
    if (!name || seen[name]) continue;
    seen[name] = true;
    out.push({ value: name, label: name });
  }
  var extra = String(current || "");
  if (extra && !seen[extra]) out.push({ value: extra, label: extra });
  return out;
}

function serializeRule(row) {
  var parts = ["workspace = " + luaString(row.id)];
  if (row.persistent) parts.push("persistent = " + luaBool(true));
  if (row.name) parts.push("default_name = " + luaString(row.name));
  if (row.monitor) parts.push("monitor = " + luaString(row.monitor));
  if (row.isDefault) parts.push("default = " + luaBool(true));
  if (row.onCreatedEmpty) parts.push("on_created_empty = " + luaString(row.onCreatedEmpty));
  return "hl.workspace_rule({ " + parts.join(", ") + " })";
}

function serialize(raw) {
  var s = clampState(raw);
  var lines = [
    BEGIN,
    "-- atmos:wrapSwitch = " + (s.wrapSwitch ? "true" : "false"),
    "-- atmos:wheelSwitch = " + (s.wheelSwitch ? "true" : "false"),
  ];
  var i;
  for (i = 0; i < s.items.length; i++) {
    if (!s.items[i].special && Number(s.items[i].id) > s.count) continue;
    lines.push(serializeRule(s.items[i]));
  }
  var binds = serializeSwitchBinds(s);
  for (i = 0; i < binds.length; i++) lines.push(binds[i]);
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

function parseRules(text) {
  var src = String(text || "");
  var out = [];
  var re = /hl\.workspace_rule\(\s*\{([^}]*)\}\s*\)/g;
  var m;
  while ((m = re.exec(src))) {
    var body = m[1];
    var row = {};
    var km = /workspace\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (km) row.id = km[1];
    var nm = /default_name\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (nm) row.name = nm[1];
    var mm = /monitor\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (mm) row.monitor = mm[1];
    row.persistent = /persistent\s*=\s*true/.test(body);
    row.isDefault = /default\s*=\s*true/.test(body);
    var om = /on_created_empty\s*=\s*"((?:\\.|[^"\\])*)"/.exec(body);
    if (om) row.onCreatedEmpty = om[1];
    var n = normalizeItem(row);
    if (n) out.push(n);
  }
  return out;
}

function parseFile(text) {
  var bounds = sentinelBounds(text);
  var items = bounds ? parseRules(String(text).substring(bounds.start, bounds.stop)) : [];
  var count = 0;
  var i;
  for (i = 0; i < items.length; i++) {
    if (!items[i].special) {
      var n = Number(items[i].id);
      if (n > count) count = n;
    }
  }
  if (count < 1) count = 10;
  var body = bounds ? String(text).substring(bounds.start, bounds.stop) : "";
  return clampState({
    count: count,
    items: items,
    wrapSwitch: body.indexOf("-- atmos:wrapSwitch = false") === -1,
    wheelSwitch: body.indexOf("-- atmos:wheelSwitch = false") === -1,
  });
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

function wrapBind(wrap, delta) {
  if (Number(delta) === -1) return wrap === false ? "r-1" : "e-1";
  return wrap === false ? "r+1" : "e+1";
}

function serializeSwitchBinds(s) {
  var wrapNext = wrapBind(s.wrapSwitch, 1);
  var wrapPrev = wrapBind(s.wrapSwitch, -1);
  var lines = [
    'hl.unbind("SUPER + TAB")',
    'hl.unbind("SUPER + SHIFT + TAB")',
    'o.bind("SUPER + TAB", "Next workspace", "hyprctl dispatch workspace ' + wrapNext + '")',
    'o.bind("SUPER + SHIFT + TAB", "Previous workspace", "hyprctl dispatch workspace ' +
      wrapPrev +
      '")',
    'hl.unbind("SUPER + mouse_down")',
    'hl.unbind("SUPER + mouse_up")',
  ];
  if (s.wheelSwitch !== false) {
    lines.push(
      'o.bind("SUPER + mouse_down", "Scroll active workspace forward", "hyprctl dispatch workspace ' +
        wrapNext +
        '")',
    );
    lines.push(
      'o.bind("SUPER + mouse_up", "Scroll active workspace backward", "hyprctl dispatch workspace ' +
        wrapPrev +
        '")',
    );
  }
  return lines;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    BEGIN: BEGIN,
    END: END,
    clampCount: clampCount,
    countFromItems: countFromItems,
    clampState: clampState,
    normalizeItem: normalizeItem,
    serialize: serialize,
    parseFile: parseFile,
    applyFile: applyFile,
    wrapBind: wrapBind,
    defaultState: defaultState,
    defaultId: defaultId,
    withDefault: withDefault,
    defaultOptions: defaultOptions,
    monitorOptions: monitorOptions,
  };
}
