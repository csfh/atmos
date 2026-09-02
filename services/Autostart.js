// Managed o.launch_on_start() lines in ~/.config/hypr/autostart.lua.

var BEGIN = "-- atmos:autostart begin"
var END = "-- atmos:autostart end"

function luaString(v) {
  return '"' + String(v).replace(/\\/g, "\\\\").replace(/"/g, '\\"') + '"'
}

function unescapeLua(v) {
  return String(v || "").replace(/\\"/g, '"').replace(/\\\\/g, "\\")
}

function sanitizeCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  if (!text || text.length > 256) return ""
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return ""
  return text
}

function parseCalls(text) {
  var src = String(text || "")
  var out = []
  var re = /o\.launch_on_start\(\s*"((?:\\.|[^"\\])*)"\s*\)/g
  var m
  while ((m = re.exec(src))) {
    var cmd = sanitizeCommand(unescapeLua(m[1]))
    if (cmd) out.push(cmd)
  }
  return out
}

function sentinelBounds(text) {
  var src = String(text || "")
  var start = src.indexOf(BEGIN)
  if (start === -1) return null
  var stop = src.indexOf(END, start + BEGIN.length)
  if (stop === -1) return null
  return { start: start, stop: stop + END.length }
}

function extractSentinel(text) {
  var bounds = sentinelBounds(text)
  if (!bounds) return ""
  return String(text).substring(bounds.start, bounds.stop)
}

function parseFile(text) {
  var src = String(text || "")
  var bounds = sentinelBounds(src)
  var managed = []
  var unmanaged = []
  if (bounds) {
    managed = parseCalls(src.substring(bounds.start, bounds.stop))
    unmanaged = parseCalls(src.substring(0, bounds.start) + "\n" + src.substring(bounds.stop))
  } else {
    unmanaged = parseCalls(src)
  }
  var items = []
  var i
  for (i = 0; i < unmanaged.length; i++) items.push({ command: unmanaged[i], managed: false })
  for (i = 0; i < managed.length; i++) items.push({ command: managed[i], managed: true })
  return items
}

function serialize(commands) {
  var list = Array.isArray(commands) ? commands : []
  var lines = [BEGIN]
  for (var i = 0; i < list.length; i++) {
    var cmd = sanitizeCommand(list[i])
    if (cmd) lines.push("o.launch_on_start(" + luaString(cmd) + ")")
  }
  lines.push(END)
  return lines.join("\n")
}

function replaceSentinel(text, block) {
  var src = String(text || "")
  var body = String(block || "").replace(/\s+$/, "")
  var bounds = sentinelBounds(src)
  if (!bounds) {
    var trimmed = src.replace(/\s+$/, "")
    return trimmed ? trimmed + "\n\n" + body + "\n" : body + "\n"
  }
  return src.substring(0, bounds.start) + body + src.substring(bounds.stop)
}

function applyFile(text, commands) {
  return replaceSentinel(text, serialize(commands))
}

function managedCommands(items) {
  var list = Array.isArray(items) ? items : []
  var out = []
  for (var i = 0; i < list.length; i++) {
    var row = list[i]
    if (!row) continue
    var cmd = sanitizeCommand(typeof row === "string" ? row : row.command)
    if (cmd && (typeof row === "string" || row.managed === true)) out.push(cmd)
  }
  return out
}
