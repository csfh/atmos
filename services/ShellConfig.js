function parseShellJson(raw, defaultsRaw) {
  var data = parseObject(raw)
  if (!data) data = parseObject(defaultsRaw)
  if (!data) data = {}
  var idle = isObject(data.idle) ? data.idle : {}
  var bar = isObject(data.bar) ? data.bar : {}
  return {
    screensaver: positiveNumber(idle.screensaver, 150),
    lock: positiveNumber(idle.lock, 300),
    barPosition: barPosition(bar.position),
    barTransparent: bar.transparent === true
  }
}

function parseObject(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "")
  if (!text) return null
  try {
    var value = JSON.parse(text)
    return isObject(value) ? value : null
  } catch (e) {
    return null
  }
}

function isObject(value) {
  return !!value && typeof value === "object" && !Array.isArray(value)
}

function positiveNumber(value, fallback) {
  var n = Number(value)
  if (!isFinite(n) || n < 0) return fallback
  return Math.round(n)
}

function barPosition(value) {
  var position = String(value || "")
  if (position === "top" || position === "bottom" || position === "left" || position === "right")
    return position
  return "top"
}

function rowMatches(query, parts) {
  var q = String(query || "").toLowerCase().replace(/^\s+|\s+$/g, "")
  if (!q) return true
  var haystack = ""
  for (var i = 0; i < parts.length; i++) haystack += " " + String(parts[i] || "")
  return haystack.toLowerCase().indexOf(q) !== -1
}
