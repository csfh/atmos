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
