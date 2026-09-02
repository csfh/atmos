function optionValue(item) {
  return (item && typeof item === "object") ? String(item.value) : String(item)
}

function optionLabel(item) {
  return (item && typeof item === "object") ? String(item.label) : String(item)
}

function filterOptions(options, query) {
  var list = Array.isArray(options) ? options : []
  var q = String(query || "").toLowerCase()
  if (!q) return list.slice()
  var out = []
  for (var i = 0; i < list.length; i++) {
    var label = optionLabel(list[i]).toLowerCase()
    var value = optionValue(list[i]).toLowerCase()
    if (label.indexOf(q) !== -1 || value.indexOf(q) !== -1) out.push(list[i])
  }
  return out
}

function parseQrOutput(raw) {
  var text = String(raw || "").replace(/\r/g, "")
  var lines = text.split("\n")
  var iface = ""
  var security = ""
  var ssid = ""
  var rows = []
  var i = 0
  if (lines.length && lines[0].indexOf("meta\t") === 0) {
    var parts = lines[0].split("\t")
    iface = parts[1] || ""
    security = parts[2] || ""
    ssid = parts.slice(3).join("\t")
    i = 1
  }
  for (; i < lines.length; i++) {
    var line = lines[i]
    if (!line) continue
    if (!/^[01]+$/.test(line)) continue
    var cells = []
    for (var c = 0; c < line.length; c++) cells.push(line.charAt(c) === "1" ? 1 : 0)
    rows.push(cells)
  }
  if (rows.length === 0)
    return { ok: false, error: "Could not read a Wi-Fi QR matrix", iface: iface, security: security, ssid: ssid, rows: [], size: 0 }
  var size = rows[0].length
  for (var r = 0; r < rows.length; r++) {
    if (rows[r].length !== size)
      return { ok: false, error: "QR matrix rows are uneven", iface: iface, security: security, ssid: ssid, rows: [], size: 0 }
  }
  return { ok: true, error: "", iface: iface, security: security, ssid: ssid, rows: rows, size: size }
}

function parseMbpsLine(line) {
  var value = parseFloat(String(line || "").replace(/^\s+|\s+$/g, ""))
  if (!isFinite(value) || value < 0) return NaN
  return value
}

function parseDiskSpeedLine(line) {
  var text = String(line || "").replace(/^\s+|\s+$/g, "")
  if (!text) return null
  var parts = text.split(/\s+/)
  if (parts.length < 2) return null
  if (parts[0] === "disk")
    return { kind: "disk", value: parts.slice(1).join(" ") }
  if (parts[0] === "read" || parts[0] === "write") {
    var n = parseFloat(parts[1])
    if (!isFinite(n) || n < 0) return null
    return { kind: parts[0], value: n }
  }
  return null
}

function parseSnapperList(raw) {
  var data
  try {
    data = typeof raw === "string" ? JSON.parse(raw || "{}") : raw
  } catch (e) {
    return []
  }
  var out = []
  if (!data || typeof data !== "object") return out

  function pushEntry(config, entry) {
    if (!entry || typeof entry !== "object") return
    var id = entry.number !== undefined ? entry.number : entry.id
    var n = Math.round(Number(id))
    if (!isFinite(n) || n < 1) return
    out.push({
      config: String(config || ""),
      id: n,
      date: String(entry.date || entry.dateIso || ""),
      description: String(entry.description || entry.userdata || ""),
      type: String(entry.type || "")
    })
  }

  if (Array.isArray(data)) {
    for (var i = 0; i < data.length; i++) pushEntry("", data[i])
    return out
  }

  var keys = Object.keys(data)
  for (var k = 0; k < keys.length; k++) {
    var name = keys[k]
    var list = data[name]
    if (!Array.isArray(list)) continue
    for (var j = 0; j < list.length; j++) pushEntry(name, list[j])
  }
  out.sort(function(a, b) { return b.id - a.id })
  return out
}

function isIpv4(token) {
  var parts = String(token || "").split(".")
  if (parts.length !== 4) return false
  for (var i = 0; i < 4; i++) {
    if (!/^[0-9]{1,3}$/.test(parts[i])) return false
    var n = parseInt(parts[i], 10)
    if (n > 255) return false
  }
  return true
}

function ipv6SplitGroups(s) {
  if (s === "") return []
  var g = String(s).split(":")
  for (var i = 0; i < g.length; i++) {
    if (!/^[0-9a-fA-F]{1,4}$/.test(g[i])) return null
  }
  return g
}

function isIpv6(token) {
  var t = String(token || "")
  if (t.indexOf(":") === -1) return false
  if (/[^0-9a-fA-F:]/.test(t)) return false
  if (t === "::") return false
  var first = t.indexOf("::")
  if (first !== -1 && t.indexOf("::", first + 2) !== -1) return false
  if (first === -1) {
    var full = ipv6SplitGroups(t)
    return !!(full && full.length === 8)
  }
  var left = ipv6SplitGroups(t.slice(0, first))
  var right = ipv6SplitGroups(t.slice(first + 2))
  if (!left || !right) return false
  return left.length + right.length < 8
}

function isPartialIpv4(token) {
  var t = String(token || "")
  if (!t) return true
  if (/[^0-9.]/.test(t)) return false
  if (t.indexOf("..") !== -1) return false
  var parts = t.split(".")
  if (parts.length > 4) return false
  for (var i = 0; i < parts.length; i++) {
    if (parts[i] === "") {
      if (i === parts.length - 1) continue
      return false
    }
    if (!/^[0-9]{1,3}$/.test(parts[i])) return false
    if (parseInt(parts[i], 10) > 255) return false
  }
  return true
}

function isPartialIpv6(token) {
  var t = String(token || "")
  if (!t) return true
  if (/[^0-9a-fA-F:]/.test(t)) return false
  if (t.indexOf(":::") !== -1) return false
  var first = t.indexOf("::")
  if (first !== -1 && t.indexOf("::", first + 2) !== -1) return false
  var body = t
  if (body.charAt(body.length - 1) === ":" && body.slice(-2) !== "::")
    body = body.slice(0, -1)
  if (body === "" || body === ":") return false
  first = body.indexOf("::")
  function count(s) {
    if (s === "") return 0
    var g = s.split(":")
    for (var i = 0; i < g.length; i++) {
      if (g[i] === "") return -1
      if (!/^[0-9a-fA-F]{1,4}$/.test(g[i])) return -1
    }
    return g.length
  }
  if (first === -1) {
    var n = count(body)
    return n >= 1 && n <= 8
  }
  var left = count(body.slice(0, first))
  var right = count(body.slice(first + 2))
  if (left < 0 || right < 0) return false
  return left + right < 8
}

function formatIpv4Piece(token) {
  var s = String(token || "").replace(/[^\d.]/g, "")
  var octets = []
  var extra = ""
  var cur = ""
  function commit() {
    if (octets.length >= 4) {
      extra += cur
      cur = ""
      return
    }
    octets.push(cur)
    cur = ""
  }
  for (var i = 0; i < s.length; i++) {
    var ch = s.charAt(i)
    if (ch === ".") {
      if (octets.length >= 4) continue
      commit()
      continue
    }
    if (octets.length >= 4) {
      extra += ch
      continue
    }
    var next = cur + ch
    var n = parseInt(next, 10)
    if (next.length > 3 || n > 255) {
      if (cur === "") {
        extra += ch
        continue
      }
      commit()
      if (octets.length >= 4) {
        extra += ch
        continue
      }
      cur = ch
      continue
    }
    cur = next
  }
  if (octets.length < 4) octets.push(cur)
  else extra += cur
  var text = octets.join(".")
  if (s.charAt(s.length - 1) === "." && octets.length < 4 && octets[octets.length - 1] !== "")
    text += "."
  return { text: text, extra: extra }
}

function formatDnsInput(text) {
  var s = String(text || "").replace(/[,\t;|\n\r]+/g, " ")
  var leading = /^\s/.test(s)
  var trailing = /\s$/.test(s)
  s = s.replace(/[^\s0-9a-fA-F.:]/g, "")
  var tokens = s.replace(/^\s+|\s+$/g, "").split(/\s+/)
  if (tokens.length === 1 && tokens[0] === "")
    return trailing || leading ? " " : ""
  var out = []
  for (var i = 0; i < tokens.length; i++) {
    var tok = tokens[i]
    if (!tok) continue
    if (tok.indexOf(":") !== -1) {
      out.push(tok.replace(/[^0-9a-fA-F:]/g, "").toLowerCase())
      continue
    }
    var piece = formatIpv4Piece(tok)
    out.push(piece.text)
    var rest = piece.extra
    while (rest) {
      var more = formatIpv4Piece(rest)
      if (!more.text) break
      out.push(more.text)
      if (more.extra === rest) break
      rest = more.extra
    }
  }
  var result = out.join(" ")
  if (trailing) result += " "
  return result
}

function dnsInputStatus(text) {
  var formatted = formatDnsInput(text)
  var trimmed = formatted.replace(/^\s+|\s+$/g, "")
  if (!trimmed)
    return { ok: false, error: "", servers: [], formatted: formatted }
  var trailingSpace = /\s$/.test(formatted)
  var tokens = trimmed.split(/\s+/)
  var servers = []
  for (var i = 0; i < tokens.length; i++) {
    var token = tokens[i]
    if (token.charAt(0) === "-")
      return { ok: false, error: "DNS servers cannot start with -", servers: servers, formatted: formatted }
    if (isIpv4(token) || isIpv6(token)) {
      servers.push(token)
      continue
    }
    var last = i === tokens.length - 1 && !trailingSpace
    if (last && (isPartialIpv4(token) || isPartialIpv6(token)))
      return { ok: false, error: "", servers: servers, formatted: formatted }
    return {
      ok: false,
      error: "Not an IPv4 or IPv6 address: " + token,
      servers: servers,
      formatted: formatted
    }
  }
  return { ok: true, error: "", servers: servers, formatted: formatted }
}

function parseDnsServers(text) {
  var status = dnsInputStatus(text)
  if (status.ok)
    return { ok: true, error: "", servers: status.servers }
  var error = status.error || "Enter one or more DNS servers"
  return { ok: false, error: error, servers: status.servers || [] }
}

function wifiRow(ssid, signal, securityKind, connected, known) {
  return {
    ssid: String(ssid || ""),
    signal: Math.round(Number(signal) || 0),
    securityKind: String(securityKind || "unknown"),
    connected: !!connected,
    known: !!known
  }
}

function sortWifiRows(rows) {
  var nets = Array.isArray(rows) ? rows.slice() : []
  nets.sort(function(a, b) {
    if (a.connected !== b.connected) return a.connected ? -1 : 1
    if (a.known !== b.known) return a.known ? -1 : 1
    return (b.signal || 0) - (a.signal || 0)
  })
  return nets
}

function requiresCredentials(kind) {
  return kind !== "open" && kind !== "owe"
}

function isEnterprise(kind) {
  return kind === "enterprise"
}

function bluetoothRow(address, name, connected, paired) {
  return {
    address: String(address || ""),
    name: String(name || ""),
    connected: !!connected,
    paired: !!paired
  }
}

function parseReminders(raw) {
  var data
  try {
    data = typeof raw === "string" ? JSON.parse(raw || "{}") : raw
  } catch (e) {
    return []
  }
  var list = data && data.reminders
  if (!Array.isArray(list)) return []
  var out = []
  for (var i = 0; i < list.length; i++) {
    var item = list[i]
    if (!item) continue
    out.push({
      unit: String(item.unit || ""),
      label: String(item.label || item.message || ""),
      message: String(item.message || ""),
      remaining: String(item.remaining || ""),
      atTime: String(item.atTime || ""),
      minutes: Math.round(Number(item.minutes) || 0)
    })
  }
  return out
}

function formatBytes(bytes) {
  var n = Number(bytes)
  if (!isFinite(n) || n < 0) n = 0
  var units = ["B", "KB", "MB", "GB", "TB"]
  var i = 0
  while (n >= 1024 && i < units.length - 1) {
    n = n / 1024
    i++
  }
  if (i === 0) return Math.round(n) + " " + units[i]
  return n.toFixed(n >= 10 ? 1 : 2) + " " + units[i]
}

function usagePercent(used, size) {
  var u = Number(used)
  var s = Number(size)
  if (!isFinite(u) || !isFinite(s) || s <= 0) return 0
  var pct = Math.round((u / s) * 100)
  if (pct < 0) return 0
  if (pct > 100) return 100
  return pct
}

function fileBasename(path) {
  var s = String(path || "")
  if (!s) return ""
  var i = s.lastIndexOf("/")
  return i === -1 ? s : s.substring(i + 1)
}

function pathFromUrl(url) {
  var s = String(url || "")
  if (s.indexOf("file://") === 0) {
    s = s.substring(7)
    if (/^\/[A-Za-z]:\//.test(s)) s = s.substring(1)
  }
  try { s = decodeURIComponent(s) } catch (e) {}
  return s
}

function isTimezoneId(id) {
  var text = String(id || "")
  if (!text || text.length > 64) return false
  if (text.indexOf("..") !== -1) return false
  if (text.charAt(0) === "-" || text.charAt(0) === "/") return false
  return /^[A-Za-z0-9/_+-]+$/.test(text)
}

function parseTimezoneId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  return isTimezoneId(text) ? text : ""
}

function parseTimedatectlYes(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "").toLowerCase()
  return text === "yes" || text === "true" || text === "1" || text === "on"
}

function isLocaleId(id) {
  var text = String(id || "")
  if (text === "C.UTF-8") return true
  return /^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(text)
}

function parseLocaleId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  return isLocaleId(text) ? text : ""
}

function isFullName(name) {
  var text = String(name || "")
  if (text.length > 256) return false
  if (text.charAt(0) === "-") return false
  if (/[:\n\r,]/.test(text)) return false
  return true
}

function parseFullName(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  return isFullName(text) ? text : ""
}

function isLauncherName(name) {
  var text = String(name || "")
  if (!text || text.length > 80) return false
  if (text.charAt(0) === "-") return false
  if (/[\/\n\r]/.test(text)) return false
  return true
}

function parseLauncherName(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  return isLauncherName(text) ? text : ""
}

function parseWebAppUrl(raw) {
  var url = String(raw || "").replace(/^\s+|\s+$/g, "")
  if (!url || /\s/.test(url)) return ""
  if (!/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(url))
    url = "https://" + url
  return /^https?:\/\//i.test(url) ? url : ""
}

function isTuiWindowStyle(style) {
  return style === "float" || style === "tile"
}

function parseParallelDownloads(text) {
  var lines = String(text || "").split("\n")
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i]
    if (/^\s*#/.test(line)) continue
    var m = line.match(/^\s*ParallelDownloads\s*=\s*([0-9]+)\s*$/)
    if (!m) continue
    var n = parseInt(m[1], 10)
    if (!isFinite(n) || n < 1) return 0
    if (n > 20) return 20
    return n
  }
  return 0
}

function isHostname(name) {
  var text = String(name || "")
  if (!text || text.length > 253) return false
  if (text.charAt(0) === "-") return false
  var labels = text.split(".")
  var labelRe = /^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?$/
  for (var i = 0; i < labels.length; i++) {
    if (!labelRe.test(labels[i])) return false
  }
  return true
}

function parseHostname(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  return isHostname(text) ? text : ""
}

function isKeyboardLayoutId(id) {
  var text = String(id || "")
  return /^[a-z0-9]{1,8}$/.test(text)
}

function parseKeyboardLayoutId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  if (text.indexOf(",") !== -1) text = text.split(",")[0]
  return isKeyboardLayoutId(text) ? text : ""
}

function parseXkbLayoutList(text) {
  var lines = String(text || "").split("\n")
  var inLayout = false
  var out = []
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i]
    if (line.indexOf("! ") === 0) {
      inLayout = /^\s*! layout\s*$/.test(line)
      continue
    }
    if (!inLayout) continue
    if (!String(line).replace(/^\s+|\s+$/g, "")) {
      if (out.length > 0) break
      continue
    }
    var m = line.match(/^\s+([a-z0-9]{1,8})\s+(\S.*)$/)
    if (!m) continue
    out.push({ value: m[1], label: m[2] })
  }
  return out
}
