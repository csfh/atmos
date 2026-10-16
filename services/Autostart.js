Qt.include("LuaText.js");
// Managed o.launch_on_start() lines in ~/.config/hypr/autostart.lua.

var BEGIN = "-- atmos:autostart begin";
var END = "-- atmos:autostart end";
// Callers pass the file only. The shared extractor also needs these markers.
var extractBlock = extractSentinel;
extractSentinel = function (text) {
  return extractBlock(text, BEGIN, END);
};

function sanitizeCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 256) return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  return text;
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
  var mark = "o.launch_on_start(";
  var i = 0;
  while (i < src.length) {
    var at = src.indexOf(mark, i);
    if (at === -1) break;
    var parsed = parseCallArgs(src, at + mark.length);
    if (!parsed) {
      i = at + mark.length;
      continue;
    }
    i = parsed.next;
    var commented = inLineComment(src, at);
    if (disabled === true) {
      if (!commented) continue;
    } else if (commented) continue;
    var arg = parsed.args[0];
    if (typeof arg !== "string") continue;
    var split = splitDelay(arg);
    if (split.command)
      out.push({ command: split.command, delay: split.delay, enabled: disabled !== true });
  }
  return out;
}

function parseFile(text) {
  var src = String(text || "");
  var bounds = sentinelBounds(src, BEGIN, END);
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

function applyFile(text, commands) {
  return replaceSentinel(text, BEGIN, END, serialize(commands));
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
