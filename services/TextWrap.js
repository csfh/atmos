// Break wrapped labels the way CSS text-wrap: pretty / balance would:
// keep the same line count, then even the lines and avoid a one-word last line.

function splitWords(text) {
  var raw = String(text || "")
  if (!raw) return []
  return raw.split(/\s+/).filter(function(w) { return w.length > 0 })
}

function shouldSkip(text) {
  var t = String(text || "")
  if (!t) return true
  return t.indexOf(" ") === -1 && t.indexOf("\n") === -1
}

function greedyLines(widths, space, maxWidth) {
  var lines = []
  var current = []
  var lineW = 0
  for (var i = 0; i < widths.length; i++) {
    var w = widths[i]
    var next = current.length ? lineW + space + w : w
    if (current.length && next > maxWidth + 0.01) {
      lines.push(current)
      current = [i]
      lineW = w
    } else {
      current.push(i)
      lineW = next
    }
  }
  if (current.length) lines.push(current)
  return lines
}

function countLines(widths, space, maxWidth) {
  return greedyLines(widths, space, maxWidth).length
}

function minWidthForLineCount(widths, space, target, maxWidth) {
  var lo = 0
  var hi = maxWidth
  var i
  for (i = 0; i < widths.length; i++) {
    if (widths[i] > lo) lo = widths[i]
  }
  if (lo >= hi) return maxWidth
  for (i = 0; i < 20; i++) {
    var mid = (lo + hi) / 2
    if (countLines(widths, space, mid) <= target) hi = mid
    else lo = mid
    if (hi - lo < 0.25) break
  }
  return hi
}

function linesToText(words, lines) {
  var out = []
  for (var i = 0; i < lines.length; i++) {
    var bits = []
    for (var j = 0; j < lines[i].length; j++) bits.push(words[lines[i][j]])
    out.push(bits.join(" "))
  }
  return out.join("\n")
}

function stealOrphan(lines, widths, space, maxWidth) {
  if (!lines || lines.length < 2) return lines
  var last = lines[lines.length - 1]
  var prev = lines[lines.length - 2]
  if (last.length !== 1 || prev.length < 2) return lines
  var moved = prev[prev.length - 1]
  var newLastW = widths[moved] + space + widths[last[0]]
  if (newLastW > maxWidth + 0.01) return lines
  var copy = lines.slice()
  copy[copy.length - 2] = prev.slice(0, prev.length - 1)
  copy[copy.length - 1] = [moved, last[0]]
  return copy
}

function wrapParagraph(text, measure, maxWidth, space, pretty) {
  var words = splitWords(text)
  if (words.length < 2) return text
  var widths = []
  var total = 0
  var i
  for (i = 0; i < words.length; i++) {
    widths[i] = Number(measure(words[i])) || 0
    total += widths[i] + (i ? space : 0)
  }
  if (total <= maxWidth + 0.01) return words.join(" ")
  var n = countLines(widths, space, maxWidth)
  var width = maxWidth
  if (n > 1 && n <= 8)
    width = minWidthForLineCount(widths, space, n, maxWidth)
  var lines = greedyLines(widths, space, width)
  if (pretty) lines = stealOrphan(lines, widths, space, maxWidth)
  return linesToText(words, lines)
}

function wrapAll(text, measure, maxWidth, spaceWidth, pretty) {
  var raw = String(text || "")
  var width = Number(maxWidth)
  var space = Number(spaceWidth)
  if (!raw || !isFinite(width) || width <= 0) return raw
  if (!isFinite(space) || space < 0) space = 0
  if (shouldSkip(raw)) return raw
  if (typeof measure !== "function") return raw
  var parts = raw.split("\n")
  var out = []
  for (var i = 0; i < parts.length; i++) {
    if (!parts[i]) {
      out.push(parts[i])
      continue
    }
    out.push(wrapParagraph(parts[i], measure, width, space, pretty))
  }
  return out.join("\n")
}

function balanceWrap(text, measure, maxWidth, spaceWidth) {
  return wrapAll(text, measure, maxWidth, spaceWidth, false)
}

function prettyWrap(text, measure, maxWidth, spaceWidth) {
  return wrapAll(text, measure, maxWidth, spaceWidth, true)
}
