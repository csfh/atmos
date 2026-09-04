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

// Topics for the section "?" modal. Each row is
// { label, description, detail, hint }. Longer `detail` wins over `description`.
function sectionHelpTopics(rows) {
  var list = Array.isArray(rows) ? rows : [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var row = list[i];
    if (!row) continue;
    var title = String(row.label || "");
    var detail = String(row.detail || "");
    var description = String(row.description || "");
    var command = String(row.hint || "");
    var body = detail.length > 0 ? detail : description;
    if (!title && !body && !command) continue;
    out.push({ title: title, body: body, command: command });
  }
  return out;
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
