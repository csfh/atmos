// Starred setting rows. QML and Node both eval this file.

function slug(text) {
  return (
    String(text || "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "") || ""
  );
}

function rowKey(hub, label) {
  var h = String(hub || "").replace(/^\/+|\/+$/g, "");
  var s = slug(label);
  if (!h || !s) return "";
  if (!/^[a-z0-9]+(?:\/[a-z0-9-]+)*$/.test(h)) return "";
  return h + "/" + s;
}

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function normalizeItem(row) {
  if (!row || typeof row !== "object" || Array.isArray(row)) return null;
  var hub = String(row.hub || "");
  var label = String(row.label || "");
  var id = rowKey(hub, label);
  if (!id) return null;
  return {
    id: id,
    hub: hub,
    hubTitle: String(row.hubTitle || ""),
    label: label,
    description: String(row.description || ""),
  };
}

function normalizeItems(value) {
  var src = Array.isArray(value) ? value : [];
  var out = [];
  var seen = {};
  var i, item;
  for (i = 0; i < src.length; i++) {
    item = normalizeItem(src[i]);
    if (!item || seen[item.id]) continue;
    seen[item.id] = true;
    out.push(item);
  }
  return out;
}

function parseDocument(raw) {
  var data = raw;
  if (typeof raw === "string") {
    var text = String(raw || "").replace(/^\s+|\s+$/g, "");
    if (!text) return [];
    try {
      data = JSON.parse(text);
    } catch (e) {
      return [];
    }
  }
  if (Array.isArray(data)) return normalizeItems(data);
  var obj = asObject(data);
  return normalizeItems(obj.items);
}

function serialize(items) {
  return JSON.stringify({ items: normalizeItems(items) });
}

function isFavorite(items, hub, label) {
  var id = rowKey(hub, label);
  if (!id) return false;
  var list = normalizeItems(items);
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i].id === id) return true;
  }
  return false;
}

function toggleItem(items, row, want) {
  var list = normalizeItems(items);
  var next = normalizeItem(row);
  if (!next) return list;
  var i;
  var found = -1;
  for (i = 0; i < list.length; i++) {
    if (list[i].id === next.id) {
      found = i;
      break;
    }
  }
  var on = want;
  if (on !== true && on !== false) on = found === -1;
  if (on) {
    if (found === -1) list.push(next);
    else list[found] = next;
  } else if (found !== -1) {
    list.splice(found, 1);
  }
  return list;
}

function groups(items) {
  var list = normalizeItems(items);
  var out = [];
  var index = {};
  var i, row, bucket;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    bucket = index[row.hub];
    if (!bucket) {
      bucket = { hub: row.hub, title: row.hubTitle || row.hub, items: [] };
      index[row.hub] = bucket;
      out.push(bucket);
    }
    bucket.items.push(row);
  }
  return out;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    slug: slug,
    rowKey: rowKey,
    normalizeItem: normalizeItem,
    normalizeItems: normalizeItems,
    parseDocument: parseDocument,
    serialize: serialize,
    isFavorite: isFavorite,
    toggleItem: toggleItem,
    groups: groups,
  };
}
