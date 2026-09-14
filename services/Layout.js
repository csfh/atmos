// Whether to draw a split after each item. `items` is [{ visible: bool }, ...].
// A split sits between two visible items: never before the first visible
// item, never after the last.
function splitAfterVisible(items) {
  var list = Array.isArray(items) ? items : [];
  var flags = [];
  var lastVisible = -1;
  for (var i = 0; i < list.length; i++) {
    flags.push(false);
    var vis = !!(list[i] && list[i].visible);
    if (!vis) continue;
    if (lastVisible >= 0) flags[lastVisible] = true;
    lastVisible = i;
  }
  return flags;
}

// Line before this item: any earlier item was visible. First visible
// item is false; last visible item is true when another visible item
// precedes it (no trailing rule after the last).
function splitBeforeVisible(items) {
  var list = Array.isArray(items) ? items : [];
  var flags = [];
  var seen = false;
  for (var i = 0; i < list.length; i++) {
    var vis = !!(list[i] && list[i].visible);
    flags.push(vis && seen);
    if (vis) seen = true;
  }
  return flags;
}

function helpNormalize(s) {
  return String(s || "")
    .toLowerCase()
    .replace(/[^a-z0-9.]+/g, " ")
    .replace(/^\s+|\s+$/g, "")
    .replace(/\s+/g, " ");
}

var HELP_STOP = {
  about: true,
  also: true,
  back: true,
  been: true,
  does: true,
  each: true,
  from: true,
  have: true,
  here: true,
  into: true,
  just: true,
  only: true,
  onto: true,
  over: true,
  than: true,
  that: true,
  them: true,
  this: true,
  used: true,
  uses: true,
  what: true,
  when: true,
  with: true,
  your: true,
};

function helpContentWords(s) {
  var parts = helpNormalize(s).split(" ");
  var out = [];
  for (var i = 0; i < parts.length; i++) {
    var w = parts[i];
    if (w.length < 4 || HELP_STOP[w]) continue;
    out.push(w);
  }
  return out;
}

// True when `help` says something the already-visible copy does not.
function helpTextIsExtra(help, visibleParts) {
  var h = helpNormalize(help);
  if (!h) return false;
  var vis = helpNormalize((visibleParts || []).join(" "));
  if (!vis) return true;
  if (vis.indexOf(h) !== -1) return false;
  var words = helpContentWords(h);
  if (words.length === 0) return false;
  var hits = 0;
  for (var i = 0; i < words.length; i++) {
    if (vis.indexOf(words[i]) !== -1) hits++;
  }
  return hits / words.length < 0.6;
}

// Topics for the section help popover. A row is included only when its
// `detail` adds context beyond the visible description.
function sectionHelpTopics(rows) {
  var list = Array.isArray(rows) ? rows : [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var row = list[i];
    if (!row) continue;
    var title = String(row.label || "");
    var detail = String(row.detail || "");
    var description = String(row.description || "");
    if (!detail || helpNormalize(detail) === helpNormalize(description)) continue;
    if (!helpTextIsExtra(detail, [description])) continue;
    out.push({
      title: title,
      body: detail,
      command: String(row.hint || ""),
    });
  }
  return out;
}

function sectionHelpPayload(detail, hint, rows) {
  var list = Array.isArray(rows) ? rows : [];
  var visible = [];
  for (var i = 0; i < list.length; i++) {
    if (!list[i]) continue;
    if (list[i].label) visible.push(String(list[i].label));
    if (list[i].description) visible.push(String(list[i].description));
  }
  var topics = sectionHelpTopics(list);
  var body = helpTextIsExtra(detail, visible) ? String(detail || "") : "";
  var command = body || topics.length ? String(hint || "") : "";
  return { body: body, command: command, topics: topics };
}

function sectionHelpOpen(payload) {
  if (!payload) return false;
  if (payload.body && String(payload.body).length) return true;
  if (payload.command && String(payload.command).length) return true;
  return !!(payload.topics && payload.topics.length);
}

function helpAccessibleName(title) {
  var t = String(title || "").replace(/^\s+|\s+$/g, "");
  if (!t) return "About this section";
  if (/settings$/i.test(t)) return "About " + t;
  return "About " + t + " settings";
}

var NAV_GROUP_LABELS = {
  look: "Desktop",
  input: "Controls",
  device: "Machine",
  apps: "Apps",
  general: "General",
  admin: "Admin",
};

function navGroupLabel(id) {
  var key = String(id || "");
  return NAV_GROUP_LABELS[key] || "";
}

function emptyNavCluster(pages) {
  return { id: "", title: "", pages: Array.isArray(pages) ? pages : [] };
}

// Cluster sidebar hubs. Consecutive pages with the same `group` stay together.
// When grouped is false (a search is active), everything is one unlabeled cluster.
function clusterByGroup(pages, grouped) {
  var list = Array.isArray(pages) ? pages : [];
  if (list.length === 0) return [];
  if (grouped === false) return [emptyNavCluster(list.slice())];
  var groups = [];
  var bucket = null;
  var last = null;
  var i;
  for (i = 0; i < list.length; i++) {
    var page = list[i];
    if (!page) continue;
    var g = String(page.group || "");
    if (!bucket || g !== last) {
      bucket = { id: g, title: navGroupLabel(g), pages: [] };
      groups.push(bucket);
      last = g;
    }
    bucket.pages.push(page);
  }
  return groups;
}

// Sidebar order as drawn: each cluster's pages, left to right, top to
// bottom. Search uses one unlabeled cluster, so this is the filtered list.
function flattenNavPages(groups) {
  var out = [];
  var list = Array.isArray(groups) ? groups : [];
  var i, j;
  for (i = 0; i < list.length; i++) {
    var pages = list[i] && list[i].pages ? list[i].pages : [];
    for (j = 0; j < pages.length; j++) {
      if (pages[j] && pages[j].id) out.push(pages[j].id);
    }
  }
  return out;
}

// Next index for j/k. Clamp at the ends; if the current hub is not in
// the drawn list (filtered away), j lands on the first match and k on
// the last.
function stepNavIndex(list, current, delta) {
  var ids = Array.isArray(list) ? list : [];
  if (ids.length === 0) return -1;
  var at = ids.indexOf(current);
  var next = at < 0 ? (delta > 0 ? 0 : ids.length - 1) : at + Number(delta);
  if (next < 0) next = 0;
  if (next > ids.length - 1) next = ids.length - 1;
  return next;
}

function jumpNavIndex(list, toEnd) {
  var ids = Array.isArray(list) ? list : [];
  if (ids.length === 0) return -1;
  return toEnd ? ids.length - 1 : 0;
}

// Visible PrefsGroups that may sit in a column. Hidden groups, wide
// groups, and non-group children (dialogs, repeaters) stay out.
function countGridSections(items) {
  var list = Array.isArray(items) ? items : [];
  var n = 0;
  var i;
  for (i = 0; i < list.length; i++) {
    var kid = list[i];
    if (!kid) continue;
    if (kid.prefsGroup !== true) continue;
    if (kid.visible === false) continue;
    if (kid.wide === true) continue;
    n++;
  }
  return n;
}

// How many equal columns fit. Never more than the number of grid
// sections, so a lone group stays full width.
function sectionColumnCount(avail, minColumn, gap, maxColumns, itemCount) {
  var items = Number(itemCount);
  if (!isFinite(items) || items < 1) items = 1;
  var max = Number(maxColumns);
  if (!isFinite(max) || max < 1) max = 1;
  max = Math.min(Math.floor(max), Math.floor(items));
  if (max < 1) max = 1;
  var w = Number(avail);
  var min = Number(minColumn);
  var g = Number(gap);
  if (!isFinite(w) || w <= 0) return 1;
  if (!isFinite(min) || min <= 0) min = 1;
  if (!isFinite(g) || g < 0) g = 0;
  var n = 1;
  while (n < max) {
    var next = n + 1;
    if (w < next * min + (next - 1) * g) break;
    n = next;
  }
  return n;
}

function sectionColumnWidth(avail, columns, gap) {
  var w = Number(avail);
  var n = Math.max(1, Math.floor(Number(columns) || 1));
  var g = Number(gap);
  if (!isFinite(w) || w < 0) w = 0;
  if (!isFinite(g) || g < 0) g = 0;
  if (n <= 1) return Math.max(0, Math.floor(w));
  return Math.max(0, Math.floor((w - g * (n - 1)) / n));
}

// Content column width. One column stays at `cap`. Two columns may
// grow up to `wideCap` when the window and the page both have room.
function pageContentWidth(avail, opts) {
  var o = opts || {};
  var margin = Number(o.margin);
  var cap = Number(o.cap);
  var wideCap = Number(o.wideCap);
  var minCol = Number(o.minColumn);
  var gap = Number(o.gap);
  var floor = Number(o.minWidth);
  var outer = Number(avail);
  if (!isFinite(margin) || margin < 0) margin = 0;
  if (!isFinite(cap) || cap <= 0) cap = 1;
  if (!isFinite(wideCap) || wideCap < cap) wideCap = cap;
  if (!isFinite(floor) || floor < 1) floor = 1;
  if (!isFinite(outer)) outer = 0;
  var inner = Math.max(0, outer - margin * 2);
  var cols = sectionColumnCount(Math.min(inner, wideCap), minCol, gap, o.maxColumns, o.itemCount);
  if (cols <= 1) return Math.max(floor, Math.min(cap, inner));
  return Math.max(floor, Math.min(wideCap, inner));
}
