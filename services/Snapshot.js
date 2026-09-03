// Shallow merge of snapshot JSON. Missing keys in a patch keep the current value.

function isPlainObject(value) {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function mergeSnapshot(current, patch) {
  var out = {};
  var src = isPlainObject(current) ? current : {};
  var add = isPlainObject(patch) ? patch : {};
  var key;
  for (key in src) {
    if (Object.prototype.hasOwnProperty.call(src, key)) out[key] = src[key];
  }
  for (key in add) {
    if (Object.prototype.hasOwnProperty.call(add, key)) out[key] = add[key];
  }
  return out;
}

function cloneRow(row) {
  var out = {};
  var key;
  if (!row || typeof row !== "object") return out;
  for (key in row) {
    if (Object.prototype.hasOwnProperty.call(row, key)) out[key] = row[key];
  }
  return out;
}

function patchMonitorBrightness(monitors, name, percent) {
  var list = Array.isArray(monitors) ? monitors.slice() : [];
  var i;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || list[i].name !== name) continue;
    var row = cloneRow(list[i]);
    row.brightness = percent;
    list[i] = row;
    break;
  }
  return list;
}

function patchFocusedMonitorScale(monitors, scale) {
  var n = Number(scale);
  var list = Array.isArray(monitors) ? monitors.slice() : [];
  var i;
  if (!isFinite(n) || n <= 0) return list;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || list[i].focused !== true) continue;
    var row = cloneRow(list[i]);
    row.scale = n;
    list[i] = row;
    break;
  }
  return list;
}

function patchPluginEnabled(plugins, id, on) {
  var list = Array.isArray(plugins) ? plugins.slice() : [];
  var want = String(id || "");
  var i;
  if (!want) return list;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || String(list[i].id || "") !== want) continue;
    var row = cloneRow(list[i]);
    row.enabled = on === true;
    list[i] = row;
    break;
  }
  return list;
}

function patchRemoveMatching(list, field, value) {
  var want = String(value || "");
  var src = Array.isArray(list) ? list : [];
  var out = [];
  var i;
  if (!want) return src.slice();
  for (i = 0; i < src.length; i++) {
    var row = src[i];
    if (typeof row === "string" || typeof row === "number") {
      if (String(row) !== want) out.push(row);
      continue;
    }
    if (!row || typeof row !== "object") {
      out.push(row);
      continue;
    }
    if (String(row[field] || "") !== want) out.push(row);
  }
  return out;
}

function patchRowField(list, matchField, matchValue, patchField, patchValue) {
  var want = String(matchValue || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var i;
  if (!want) return src;
  for (i = 0; i < src.length; i++) {
    if (!src[i] || String(src[i][matchField] || "") !== want) continue;
    var row = cloneRow(src[i]);
    row[patchField] = patchValue;
    src[i] = row;
    break;
  }
  return src;
}

function patchWifiActive(list, uuid, on) {
  var want = String(uuid || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var i;
  if (!want) return src;
  for (i = 0; i < src.length; i++) {
    if (!src[i]) continue;
    var row = cloneRow(src[i]);
    if (String(row.uuid || "") === want) row.active = on === true;
    else if (on === true) row.active = false;
    src[i] = row;
  }
  return src;
}

function patchHookSample(list, path, enabled) {
  var want = String(path || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var i;
  if (!want) return src;
  for (i = 0; i < src.length; i++) {
    if (!src[i] || String(src[i].path || "") !== want) continue;
    var row = cloneRow(src[i]);
    var p = String(row.path || "");
    if (enabled) {
      if (p.length >= 7 && p.substring(p.length - 7) === ".sample")
        p = p.substring(0, p.length - 7);
      row.sample = false;
    } else {
      if (!(p.length >= 7 && p.substring(p.length - 7) === ".sample")) p = p + ".sample";
      row.sample = true;
    }
    row.path = p;
    var slash = p.lastIndexOf("/");
    row.name = slash === -1 ? p : p.substring(slash + 1);
    src[i] = row;
    break;
  }
  return src;
}

function patchKeyboardBrightness(current, direction) {
  var n = Math.round(Number(current));
  if (!isFinite(n) || n < 0) n = 0;
  if (n > 100) n = 100;
  if (direction === "off") return 0;
  if (direction === "up") {
    n += 10;
    if (n > 100) n = 100;
    return n;
  }
  if (direction === "down") {
    n -= 10;
    if (n < 0) n = 0;
    return n;
  }
  return n;
}

function patchAppendReminder(list, minutes, message) {
  var src = Array.isArray(list) ? list.slice() : [];
  var mins = Math.round(Number(minutes));
  var msg = String(message || "");
  if (!isFinite(mins) || mins < 1) return src;
  src.push({
    unit: "",
    label: msg || mins + " min",
    message: msg,
    remaining: mins + " min",
    atTime: "",
    minutes: mins,
  });
  return src;
}

function patchAppendHook(list, row) {
  if (!row || typeof row !== "object") return Array.isArray(list) ? list.slice() : [];
  var path = String(row.path || "");
  var src = Array.isArray(list) ? list.slice() : [];
  var next = cloneRow(row);
  var i;
  if (!path) return src;
  for (i = 0; i < src.length; i++) {
    if (src[i] && String(src[i].path || "") === path) {
      src[i] = next;
      return src;
    }
  }
  src.push(next);
  return src;
}

function patchReplaceManaged(list, managed) {
  var src = Array.isArray(list) ? list : [];
  var next = Array.isArray(managed) ? managed : [];
  var out = [];
  var i;
  for (i = 0; i < src.length; i++) {
    var cur = src[i];
    if (cur && cur.managed === true) continue;
    if (cur && typeof cur === "object") out.push(cloneRow(cur));
    else if (typeof cur === "string" && cur) out.push({ command: cur, managed: false });
  }
  for (i = 0; i < next.length; i++) {
    var item = next[i];
    if (item == null) continue;
    if (typeof item === "string" || typeof item === "number") {
      var cmd = String(item);
      if (cmd) out.push({ command: cmd, managed: true });
      continue;
    }
    if (typeof item !== "object") continue;
    var row = cloneRow(item);
    row.managed = true;
    out.push(row);
  }
  return out;
}

function parseSnapshot(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return null;
  try {
    var value = JSON.parse(text);
    return isPlainObject(value) ? value : null;
  } catch (e) {
    return null;
  }
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    mergeSnapshot: mergeSnapshot,
    parseSnapshot: parseSnapshot,
    patchMonitorBrightness: patchMonitorBrightness,
    patchFocusedMonitorScale: patchFocusedMonitorScale,
    patchPluginEnabled: patchPluginEnabled,
    patchRemoveMatching: patchRemoveMatching,
    patchRowField: patchRowField,
    patchWifiActive: patchWifiActive,
    patchHookSample: patchHookSample,
    patchKeyboardBrightness: patchKeyboardBrightness,
    patchAppendReminder: patchAppendReminder,
    patchAppendHook: patchAppendHook,
    patchReplaceManaged: patchReplaceManaged,
    isPlainObject: isPlainObject,
  };
}
