// User overlay for ~/.config/environment.d/10-atmos.conf

var BEGIN = "# atmos:env begin";
var END = "# atmos:env end";

function sanitizeKey(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!/^[A-Za-z_][A-Za-z0-9_]{0,63}$/.test(text)) return "";
  if (text === "PATH") return "";
  return text;
}

function sanitizeValue(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1 || text.indexOf("\0") !== -1)
    return "";
  if (text.length > 512) return "";
  return text;
}

function sanitizePathPrepend(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (text.indexOf("\n") !== -1) return "";
  if (text.length > 512) return "";
  var parts = text.split(":");
  var out = [];
  var i;
  for (i = 0; i < parts.length; i++) {
    var p = parts[i].replace(/^\s+|\s+$/g, "");
    if (!p || p.indexOf("..") !== -1) continue;
    if (p.charAt(0) !== "/") continue;
    out.push(p);
  }
  return out.join(":");
}

function sanitizeShell(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (text.charAt(0) !== "/") return "";
  if (!/^\/[A-Za-z0-9/_+-]+$/.test(text)) return "";
  return text;
}

function defaultState() {
  return { pathPrepend: "", vars: [], shell: "" };
}

function normalizeVar(row) {
  if (!row || typeof row !== "object") return null;
  var key = sanitizeKey(row.key);
  var value = sanitizeValue(row.value);
  if (!key) return null;
  return { key: key, value: value };
}

function clampState(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  var vars = [];
  var seen = {};
  var list = Array.isArray(src.vars) ? src.vars : [];
  var i;
  for (i = 0; i < list.length; i++) {
    var row = normalizeVar(list[i]);
    if (!row || seen[row.key]) continue;
    seen[row.key] = true;
    vars.push(row);
  }
  return {
    pathPrepend: sanitizePathPrepend(src.pathPrepend),
    vars: vars,
    shell: sanitizeShell(src.shell),
  };
}

function serialize(raw) {
  var s = clampState(raw);
  var lines = [BEGIN];
  if (s.pathPrepend) lines.push("PATH=" + s.pathPrepend + ":$PATH");
  var i;
  for (i = 0; i < s.vars.length; i++) lines.push(s.vars[i].key + "=" + s.vars[i].value);
  lines.push(END);
  return lines.join("\n") + "\n";
}

function parseFile(text) {
  var src = String(text || "");
  var start = src.indexOf(BEGIN);
  var stop = src.indexOf(END);
  var body = "";
  if (start !== -1 && stop > start) body = src.substring(start + BEGIN.length, stop);
  else body = src;
  var vars = [];
  var pathPrepend = "";
  var lines = body.split("\n");
  var i;
  for (i = 0; i < lines.length; i++) {
    var line = lines[i].replace(/^\s+|\s+$/g, "");
    if (!line || line.charAt(0) === "#") continue;
    var cut = line.indexOf("=");
    if (cut < 1) continue;
    var key = line.substring(0, cut);
    var value = line.substring(cut + 1);
    if (key === "PATH") {
      pathPrepend = sanitizePathPrepend(value.replace(/:\$PATH$/, "").replace(/:\$\{PATH\}$/, ""));
      continue;
    }
    var row = normalizeVar({ key: key, value: value });
    if (row) vars.push(row);
  }
  return clampState({ pathPrepend: pathPrepend, vars: vars });
}

function detected(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  function clean(v, n) {
    return String(v || "")
      .replace(/[\r\n]+/g, " ")
      .substring(0, n || 240);
  }
  return {
    sessionType: clean(src.sessionType, 32),
    desktop: clean(src.desktop, 64),
    hyprlandVersion: clean(src.hyprlandVersion, 80),
    omarchyPath: clean(src.omarchyPath, 240),
    path: clean(src.path, 1024),
    xdgHome: clean(src.xdgHome, 240),
    xdgConfig: clean(src.xdgConfig, 240),
    xdgData: clean(src.xdgData, 240),
    editor: clean(src.editor, 64),
    browser: clean(src.browser, 64),
    terminal: clean(src.terminal, 64),
    shell: clean(src.shell, 64),
  };
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    BEGIN: BEGIN,
    END: END,
    sanitizeKey: sanitizeKey,
    sanitizeValue: sanitizeValue,
    sanitizePathPrepend: sanitizePathPrepend,
    sanitizeShell: sanitizeShell,
    clampState: clampState,
    serialize: serialize,
    parseFile: parseFile,
    detected: detected,
  };
}
