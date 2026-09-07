// Managed o.launch_on_start() lines in ~/.config/hypr/autostart.lua.

var BEGIN = "-- atmos:autostart begin";
var END = "-- atmos:autostart end";

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

function sanitizeCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 256) return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  return text;
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

function splitDelay(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  var m = text.match(/^sleep\s+(\d+)\s+&&\s+(.+)$/);
  if (!m) return { delay: 0, command: sanitizeCommand(text) };
  var delay = Math.round(Number(m[1]));
  if (!isFinite(delay) || delay < 0) delay = 0;
  if (delay > 600) delay = 600;
  return { delay: delay, command: sanitizeCommand(m[2]) };
}

function joinDelay(command, delay) {
  var cmd = sanitizeCommand(command);
  var n = Math.round(Number(delay));
  if (!isFinite(n) || n < 0) n = 0;
  if (n > 600) n = 600;
  if (!cmd) return "";
  if (n > 0) return "sleep " + n + " && " + cmd;
  return cmd;
}

function parseCalls(text, disabled) {
  var src = String(text || "");
  var out = [];
  var re = /o\.launch_on_start\(\s*"((?:\\.|[^"\\])*)"\s*\)/g;
  var m;
  while ((m = re.exec(src))) {
    var commented = inLineComment(src, m.index);
    if (disabled === true) {
      if (!commented) continue;
    } else if (commented) continue;
    var split = splitDelay(unescapeLua(m[1]));
    if (split.command)
      out.push({ command: split.command, delay: split.delay, enabled: disabled !== true });
  }
  return out;
}

function sentinelBounds(text) {
  var src = String(text || "");
  var start = src.indexOf(BEGIN);
  if (start === -1) return null;
  var stop = src.indexOf(END, start + BEGIN.length);
  if (stop === -1) return null;
  return { start: start, stop: stop + END.length };
}

function extractSentinel(text) {
  var bounds = sentinelBounds(text);
  if (!bounds) return "";
  return String(text).substring(bounds.start, bounds.stop);
}

function parseFile(text) {
  var src = String(text || "");
  var bounds = sentinelBounds(src);
  var managed = [];
  var unmanaged = [];
  if (bounds) {
    managed = parseCalls(src.substring(bounds.start, bounds.stop));
    unmanaged = parseCalls(src.substring(0, bounds.start) + "\n" + src.substring(bounds.stop));
  } else {
    unmanaged = parseCalls(src);
  }
  var items = [];
  var i;
  var disabled = bounds ? parseCalls(src.substring(bounds.start, bounds.stop), true) : [];
  for (i = 0; i < unmanaged.length; i++) {
    items.push({
      command: unmanaged[i].command,
      delay: unmanaged[i].delay,
      enabled: true,
      managed: false,
    });
  }
  for (i = 0; i < managed.length; i++) {
    items.push({
      command: managed[i].command,
      delay: managed[i].delay,
      enabled: true,
      managed: true,
    });
  }
  for (i = 0; i < disabled.length; i++) {
    items.push({
      command: disabled[i].command,
      delay: disabled[i].delay,
      enabled: false,
      managed: true,
    });
  }
  return items;
}

function normalizeItem(row) {
  if (typeof row === "string") {
    var split = splitDelay(row);
    if (!split.command) return null;
    return { command: split.command, delay: split.delay, enabled: true };
  }
  if (!row || typeof row !== "object") return null;
  var command = sanitizeCommand(row.command);
  if (!command) return null;
  var delay = Math.round(Number(row.delay || 0));
  if (!isFinite(delay) || delay < 0) delay = 0;
  if (delay > 600) delay = 600;
  return { command: command, delay: delay, enabled: row.enabled !== false };
}

function serialize(commands) {
  var list = Array.isArray(commands) ? commands : [];
  var lines = [BEGIN];
  for (var i = 0; i < list.length; i++) {
    var row = normalizeItem(list[i]);
    if (!row) continue;
    var cmd = joinDelay(row.command, row.delay);
    if (!cmd) continue;
    var line = "o.launch_on_start(" + luaString(cmd) + ")";
    if (row.enabled === false) line = "-- " + line;
    lines.push(line);
  }
  lines.push(END);
  return lines.join("\n");
}

function replaceSentinel(text, block) {
  var src = String(text || "");
  var body = String(block || "").replace(/\s+$/, "");
  var bounds = sentinelBounds(src);
  if (!bounds) {
    var trimmed = src.replace(/\s+$/, "");
    return trimmed ? trimmed + "\n\n" + body + "\n" : body + "\n";
  }
  return src.substring(0, bounds.start) + body + src.substring(bounds.stop);
}

function applyFile(text, commands) {
  return replaceSentinel(text, serialize(commands));
}

function managedCommands(items) {
  var list = Array.isArray(items) ? items : [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var row = list[i];
    if (!row) continue;
    if (typeof row === "object" && row.managed === false) continue;
    var next = normalizeItem(typeof row === "string" ? row : row);
    if (next) out.push(next);
  }
  return out;
}
