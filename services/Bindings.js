// Managed o.bind / hl.unbind lines in ~/.config/hypr/bindings.lua.

var BEGIN = "-- atmos:bindings begin"
var END = "-- atmos:bindings end"

function luaString(v) {
  return '"' + String(v).replace(/\\/g, "\\\\").replace(/"/g, '\\"') + '"'
}

function sanitizeKeys(raw) {
  var text = String(raw || "")
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return ""
  text = text.replace(/^\s+|\s+$/g, "").replace(/\s+/g, " ")
  if (!text || text.length > 64) return ""
  if (!/^[A-Za-z0-9_ +.:-]+$/.test(text)) return ""
  return text
}

function sanitizeLabel(raw) {
  var text = String(raw || "")
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return ""
  text = text.replace(/^\s+|\s+$/g, "")
  if (!text || text.length > 64) return ""
  return text
}

function sanitizeCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  if (!text || text.length > 256) return ""
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return ""
  return text
}

function skipWs(s, i) {
  while (i < s.length && /[ \t\r\n]/.test(s.charAt(i))) i++
  return i
}

function identCont(s, i) {
  return i < s.length && /[A-Za-z0-9_]/.test(s.charAt(i))
}

function parseLuaString(s, i) {
  if (s.charAt(i) !== '"') return null
  i++
  var out = ""
  while (i < s.length) {
    var c = s.charAt(i)
    if (c === "\\") {
      if (i + 1 >= s.length) return null
      out += s.charAt(i + 1)
      i += 2
      continue
    }
    if (c === '"') return { value: out, next: i + 1 }
    out += c
    i++
  }
  return null
}

function parseLuaTable(s, i) {
  i = skipWs(s, i)
  if (s.charAt(i) !== "{") return null
  i = skipWs(s, i + 1)
  var obj = {}
  var arr = []
  while (i < s.length && s.charAt(i) !== "}") {
    i = skipWs(s, i)
    if (s.charAt(i) === "}") break
    var ident = s.substring(i).match(/^[A-Za-z_][A-Za-z0-9_]*/)
    var j = ident ? skipWs(s, i + ident[0].length) : i
    if (ident && s.charAt(j) === "=") {
      var named = parseLuaValue(s, j + 1)
      if (!named) return null
      obj[ident[0]] = named.value
      i = named.next
    } else {
      var item = parseLuaValue(s, i)
      if (!item) return null
      arr.push(item.value)
      i = item.next
    }
    i = skipWs(s, i)
    if (s.charAt(i) === ",") i++
  }
  if (s.charAt(i) !== "}") return null
  if (arr.length && Object.keys(obj).length === 0) return { value: arr, next: i + 1 }
  return { value: obj, next: i + 1 }
}

function parseLuaValue(s, i) {
  i = skipWs(s, i)
  if (i >= s.length) return null
  var c = s.charAt(i)
  if (c === '"') return parseLuaString(s, i)
  if (c === "{") return parseLuaTable(s, i)
  if (s.substring(i, i + 3) === "nil" && !identCont(s, i + 3)) return { value: null, next: i + 3 }
  if (s.substring(i, i + 4) === "true" && !identCont(s, i + 4)) return { value: true, next: i + 4 }
  if (s.substring(i, i + 5) === "false" && !identCont(s, i + 5)) return { value: false, next: i + 5 }
  var num = s.substring(i).match(/^-?\d+(?:\.\d+)?/)
  if (num) return { value: Number(num[0]), next: i + num[0].length }
  return null
}

function parseCallArgs(s, i) {
  var args = []
  i = skipWs(s, i)
  while (i < s.length && s.charAt(i) !== ")") {
    var val = parseLuaValue(s, i)
    if (!val) return null
    args.push(val.value)
    i = skipWs(s, val.next)
    if (s.charAt(i) === ",") i = skipWs(s, i + 1)
  }
  if (s.charAt(i) !== ")") return null
  return { args: args, next: i + 1 }
}

function commandFromArg(arg) {
  if (typeof arg === "string") return sanitizeCommand(arg)
  if (arg && typeof arg === "object" && arg.launch) return sanitizeCommand(arg.launch)
  return ""
}

function parseEvents(text) {
  var src = String(text || "")
  var events = []
  var i = 0
  while (i < src.length) {
    var u = src.indexOf("hl.unbind(", i)
    var b = src.indexOf("o.bind(", i)
    if (u === -1 && b === -1) break
    if (u !== -1 && (b === -1 || u < b)) {
      var unbind = parseCallArgs(src, u + 10)
      if (!unbind) {
        i = u + 10
        continue
      }
      var unbindKeys = sanitizeKeys(unbind.args[0])
      if (unbindKeys) events.push({ kind: "unbind", keys: unbindKeys })
      i = unbind.next
    } else {
      var bind = parseCallArgs(src, b + 7)
      if (!bind) {
        i = b + 7
        continue
      }
      var keys = sanitizeKeys(bind.args[0])
      var label = bind.args[1] == null ? "" : sanitizeLabel(bind.args[1])
      var command = commandFromArg(bind.args[2])
      if (keys) events.push({ kind: "bind", keys: keys, label: label, command: command })
      i = bind.next
    }
  }
  return events
}

function foldEvents(events) {
  var list = Array.isArray(events) ? events : []
  var out = []
  var i = 0
  while (i < list.length) {
    var ev = list[i]
    if (ev.kind === "unbind") {
      var next = list[i + 1]
      if (next && next.kind === "bind" && next.keys === ev.keys) {
        out.push({ keys: ev.keys, label: next.label, command: next.command, unbind: true })
        i += 2
        continue
      }
      out.push({ keys: ev.keys, label: "", command: "", unbind: true })
      i++
      continue
    }
    out.push({ keys: ev.keys, label: ev.label, command: ev.command, unbind: false })
    i++
  }
  return out
}

function normalize(row) {
  if (!row) return null
  var keys = sanitizeKeys(row.keys)
  if (!keys) return null
  var label = sanitizeLabel(row.label)
  var command = sanitizeCommand(row.command)
  var unbind = row.unbind === true
  if (!command && !unbind) return null
  return { keys: keys, label: label, command: command, unbind: unbind }
}

function parseCalls(text) {
  return foldEvents(parseEvents(text))
}

function sentinelBounds(text) {
  var src = String(text || "")
  var start = src.indexOf(BEGIN)
  if (start === -1) return null
  var stop = src.indexOf(END, start + BEGIN.length)
  if (stop === -1) return null
  return { start: start, stop: stop + END.length }
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
  for (i = 0; i < unmanaged.length; i++) {
    items.push({
      keys: unmanaged[i].keys,
      label: unmanaged[i].label,
      command: unmanaged[i].command,
      unbind: unmanaged[i].unbind,
      managed: false
    })
  }
  for (i = 0; i < managed.length; i++) {
    items.push({
      keys: managed[i].keys,
      label: managed[i].label,
      command: managed[i].command,
      unbind: managed[i].unbind,
      managed: true
    })
  }
  return items
}

function serialize(items) {
  var list = Array.isArray(items) ? items : []
  var lines = [BEGIN]
  for (var i = 0; i < list.length; i++) {
    var row = normalize(typeof list[i] === "object" ? list[i] : null)
    if (!row) continue
    if (row.unbind) lines.push("hl.unbind(" + luaString(row.keys) + ")")
    if (row.command) {
      var labelArg = row.label ? luaString(row.label) : "nil"
      lines.push("o.bind(" + luaString(row.keys) + ", " + labelArg + ", " + luaString(row.command) + ")")
    }
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

function applyFile(text, items) {
  return replaceSentinel(text, serialize(items))
}

function managedItems(items) {
  var list = Array.isArray(items) ? items : []
  var out = []
  for (var i = 0; i < list.length; i++) {
    var row = list[i]
    if (!row) continue
    if (typeof row === "object" && row.managed === false) continue
    var next = normalize(row)
    if (next) out.push(next)
  }
  return out
}

function parsePrint(text) {
  var lines = String(text || "").split("\n")
  var out = []
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i]
    var sep = line.indexOf(" \u2192 ")
    if (sep === -1) sep = line.indexOf(" -> ")
    if (sep === -1) continue
    var arrow = line.indexOf(" \u2192 ") === sep ? 3 : 4
    var keys = String(line.substring(0, sep)).replace(/\s+$/g, "")
    var action = String(line.substring(sep + arrow)).replace(/^\s+|\s+$/g, "")
    if (keys && action) out.push({ keys: keys, action: action })
  }
  return out
}

function catalogConflict(catalog, keys) {
  var list = Array.isArray(catalog) ? catalog : []
  var chord = sanitizeKeys(keys)
  if (!chord) return ""
  for (var i = 0; i < list.length; i++) {
    if (list[i] && sanitizeKeys(list[i].keys) === chord) return String(list[i].action || "")
  }
  return ""
}
