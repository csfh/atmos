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

function patchMonitorBrightness(monitors, name, percent) {
  var list = Array.isArray(monitors) ? monitors.slice() : [];
  var i;
  for (i = 0; i < list.length; i++) {
    if (!list[i] || list[i].name !== name) continue;
    var row = {};
    var key;
    for (key in list[i]) {
      if (Object.prototype.hasOwnProperty.call(list[i], key)) row[key] = list[i][key];
    }
    row.brightness = percent;
    list[i] = row;
    break;
  }
  return list;
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
    isPlainObject: isPlainObject,
  };
}
