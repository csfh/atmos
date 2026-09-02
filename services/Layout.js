// Whether to draw a split after each item. `items` is [{ visible: bool }, ...].
// A split sits between two visible items: never before the first visible
// item, never after the last.
function splitAfterVisible(items) {
  var list = Array.isArray(items) ? items : []
  var flags = []
  var lastVisible = -1
  for (var i = 0; i < list.length; i++) {
    flags.push(false)
    var vis = !!(list[i] && list[i].visible)
    if (!vis) continue
    if (lastVisible >= 0) flags[lastVisible] = true
    lastVisible = i
  }
  return flags
}

// Line before this item: any earlier item was visible. First visible
// item is false; last visible item is true when another visible item
// precedes it (no trailing rule after the last).
function splitBeforeVisible(items) {
  var list = Array.isArray(items) ? items : []
  var flags = []
  var seen = false
  for (var i = 0; i < list.length; i++) {
    var vis = !!(list[i] && list[i].visible)
    flags.push(vis && seen)
    if (vis) seen = true
  }
  return flags
}

// Topics for the section "?" modal. Each row is
// { label, description, detail, hint }. Longer `detail` wins over `description`.
function sectionHelpTopics(rows) {
  var list = Array.isArray(rows) ? rows : []
  var out = []
  for (var i = 0; i < list.length; i++) {
    var row = list[i]
    if (!row) continue
    var title = String(row.label || "")
    var detail = String(row.detail || "")
    var description = String(row.description || "")
    var command = String(row.hint || "")
    var body = detail.length > 0 ? detail : description
    if (!title && !body && !command) continue
    out.push({ title: title, body: body, command: command })
  }
  return out
}
