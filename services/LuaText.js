// Sentinel Lua text shared by the Hypr writers.
// Quoting, comments, call arguments, and begin/end block edits stay here so
// the block modules do not each carry a copy. The Rust twin is hyprland/lua.rs.

function luaString(v) {
  return (
    '"' +
    String(v)
      .replace(/\\/g, "\\\\")
      .replace(/\n/g, "\\n")
      .replace(/\r/g, "\\r")
      .replace(/\t/g, "\\t")
      .replace(/"/g, '\\"') +
    '"'
  );
}

function luaBool(v) {
  return v ? "true" : "false";
}

function luaNumber(n) {
  if (!isFinite(n)) return "0";
  if (Math.abs(n - Math.round(n)) < 1e-9) return String(Math.round(n));
  var s = (Math.round(n * 1000) / 1000).toFixed(3);
  return s.replace(/\.?0+$/, "");
}

function unescapeLua(v) {
  var src = String(v || "");
  var out = "";
  var i = 0;
  while (i < src.length) {
    if (src.charAt(i) === "\\" && i + 1 < src.length) {
      var e = src.charAt(i + 1);
      if (e === "n") out += "\n";
      else if (e === "t") out += "\t";
      else if (e === "r") out += "\r";
      else out += e;
      i += 2;
      continue;
    }
    out += src.charAt(i);
    i++;
  }
  return out;
}

function skipWs(s, i) {
  // Lua -- comments. Only called outside strings.
  while (i < s.length) {
    while (i < s.length && /[ \t\r\n]/.test(s.charAt(i))) i++;
    if (s.charAt(i) !== "-" || s.charAt(i + 1) !== "-") break;
    var nl = s.indexOf("\n", i + 2);
    i = nl === -1 ? s.length : nl + 1;
  }
  return i;
}

function inLineComment(src, at) {
  var lineStart = src.lastIndexOf("\n", at > 0 ? at - 1 : 0) + 1;
  var i = lineStart;
  var inStr = false;
  while (i < at) {
    var c = src.charAt(i);
    if (inStr) {
      if (c === "\\") {
        i += 2;
        continue;
      }
      if (c === '"') inStr = false;
      i++;
      continue;
    }
    if (c === '"') {
      inStr = true;
      i++;
      continue;
    }
    if (c === "-" && src.charAt(i + 1) === "-") return true;
    i++;
  }
  return false;
}

function identCont(s, i) {
  return i < s.length && /[A-Za-z0-9_]/.test(s.charAt(i));
}

function parseLuaString(s, i) {
  if (s.charAt(i) !== '"') return null;
  i++;
  var out = "";
  while (i < s.length) {
    var c = s.charAt(i);
    if (c === "\\") {
      if (i + 1 >= s.length) return null;
      var e = s.charAt(i + 1);
      if (e === "n") out += "\n";
      else if (e === "t") out += "\t";
      else if (e === "r") out += "\r";
      else out += e;
      i += 2;
      continue;
    }
    if (c === '"') return { value: out, next: i + 1 };
    out += c;
    i++;
  }
  return null;
}

function parseLuaTable(s, i) {
  i = skipWs(s, i);
  if (s.charAt(i) !== "{") return null;
  i = skipWs(s, i + 1);
  var obj = {};
  var arr = [];
  while (i < s.length && s.charAt(i) !== "}") {
    i = skipWs(s, i);
    if (s.charAt(i) === "}") break;
    var ident = s.substring(i).match(/^[A-Za-z_][A-Za-z0-9_]*/);
    var j = ident ? skipWs(s, i + ident[0].length) : i;
    if (ident && s.charAt(j) === "=") {
      var named = parseLuaValue(s, j + 1);
      if (!named) return null;
      obj[ident[0]] = named.value;
      i = named.next;
    } else {
      var item = parseLuaValue(s, i);
      if (!item) return null;
      arr.push(item.value);
      i = item.next;
    }
    i = skipWs(s, i);
    if (s.charAt(i) === ",") i++;
  }
  if (s.charAt(i) !== "}") return null;
  if (arr.length && Object.keys(obj).length === 0) return { value: arr, next: i + 1 };
  return { value: obj, next: i + 1 };
}

function parseLuaValue(s, i) {
  i = skipWs(s, i);
  if (i >= s.length) return null;
  var c = s.charAt(i);
  if (c === '"') return parseLuaString(s, i);
  if (c === "{") return parseLuaTable(s, i);
  if (s.substring(i, i + 3) === "nil" && !identCont(s, i + 3)) return { value: null, next: i + 3 };
  if (s.substring(i, i + 4) === "true" && !identCont(s, i + 4)) return { value: true, next: i + 4 };
  if (s.substring(i, i + 5) === "false" && !identCont(s, i + 5))
    return { value: false, next: i + 5 };
  var num = s.substring(i).match(/^-?\d+(?:\.\d+)?/);
  if (num) return { value: Number(num[0]), next: i + num[0].length };
  return null;
}

function parseCallArgs(s, i) {
  var args = [];
  i = skipWs(s, i);
  while (i < s.length && s.charAt(i) !== ")") {
    var val = parseLuaValue(s, i);
    if (!val) return null;
    args.push(val.value);
    i = skipWs(s, val.next);
    if (s.charAt(i) === ",") i = skipWs(s, i + 1);
  }
  if (s.charAt(i) !== ")") return null;
  return { args: args, next: i + 1 };
}

// A commented call resumes just after the mark, so text inside it is scanned again.
function liveCalls(text, mark, visit) {
  var src = String(text || "");
  var i = 0;
  while (i < src.length) {
    var at = src.indexOf(mark, i);
    if (at === -1) break;
    if (inLineComment(src, at)) {
      i = at + mark.length;
      continue;
    }
    var parsed = parseCallArgs(src, at + mark.length);
    if (!parsed) {
      i = at + mark.length;
      continue;
    }
    visit(parsed.args);
    i = parsed.next;
  }
}

function sentinelBounds(text, begin, end) {
  var src = String(text || "");
  var start = src.indexOf(begin);
  if (start === -1) return null;
  var stop = src.indexOf(end, start + begin.length);
  if (stop === -1) return null;
  return { start: start, stop: stop + end.length };
}

function hasSentinel(text, begin, end) {
  return sentinelBounds(text, begin, end) !== null;
}

function extractSentinel(text, begin, end) {
  var bounds = sentinelBounds(text, begin, end);
  if (!bounds) return "";
  return String(text).substring(bounds.start, bounds.stop);
}

function stripSentinel(text, begin, end) {
  var src = String(text || "");
  var bounds = sentinelBounds(src, begin, end);
  if (!bounds) return src;
  var before = src.substring(0, bounds.start).replace(/\s+$/, "");
  var after = src.substring(bounds.stop).replace(/^\s+/, "");
  if (before && after) return before + "\n\n" + after;
  return before || after;
}

function replaceSentinel(text, begin, end, block) {
  var src = String(text || "");
  var body = String(block || "").replace(/\s+$/, "");
  var bounds = sentinelBounds(src, begin, end);
  if (!bounds) {
    var trimmed = src.replace(/\s+$/, "");
    return trimmed ? trimmed + "\n\n" + body + "\n" : body + "\n";
  }
  return src.substring(0, bounds.start) + body + src.substring(bounds.stop);
}
