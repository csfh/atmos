function optionValue(item) {
  return item && typeof item === "object" ? String(item.value) : String(item);
}

function optionLabel(item) {
  return item && typeof item === "object" ? String(item.label) : String(item);
}

function filterOptions(options, query) {
  var list = Array.isArray(options) ? options : [];
  var q = String(query || "").toLowerCase();
  if (!q) return list.slice();
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var label = optionLabel(list[i]).toLowerCase();
    var value = optionValue(list[i]).toLowerCase();
    if (label.indexOf(q) !== -1 || value.indexOf(q) !== -1) out.push(list[i]);
  }
  return out;
}

function parseQrOutput(raw) {
  var text = String(raw || "").replace(/\r/g, "");
  var lines = text.split("\n");
  var iface = "";
  var security = "";
  var ssid = "";
  var rows = [];
  var i = 0;
  if (lines.length && lines[0].indexOf("meta\t") === 0) {
    var parts = lines[0].split("\t");
    iface = parts[1] || "";
    security = parts[2] || "";
    ssid = parts.slice(3).join("\t");
    i = 1;
  }
  for (; i < lines.length; i++) {
    var line = lines[i];
    if (!line) continue;
    if (!/^[01]+$/.test(line)) continue;
    var cells = [];
    for (var c = 0; c < line.length; c++) cells.push(line.charAt(c) === "1" ? 1 : 0);
    rows.push(cells);
  }
  if (rows.length === 0)
    return {
      ok: false,
      error: "Could not read a Wi-Fi QR matrix",
      iface: iface,
      security: security,
      ssid: ssid,
      rows: [],
      size: 0,
    };
  var size = rows[0].length;
  for (var r = 0; r < rows.length; r++) {
    if (rows[r].length !== size)
      return {
        ok: false,
        error: "QR matrix rows are uneven",
        iface: iface,
        security: security,
        ssid: ssid,
        rows: [],
        size: 0,
      };
  }
  return {
    ok: true,
    error: "",
    iface: iface,
    security: security,
    ssid: ssid,
    rows: rows,
    size: size,
  };
}

function parseMbpsLine(line) {
  var value = parseFloat(String(line || "").replace(/^\s+|\s+$/g, ""));
  if (!isFinite(value) || value < 0) return NaN;
  return value;
}

function formatMbps(raw) {
  var n = parseFloat(String(raw || "").replace(/^\s+|\s+$/g, ""));
  if (!isFinite(n) || n < 0) return "";
  if (n === 0) return "0.0";
  if (n < 10) return (Math.round(n * 10) / 10).toFixed(1);
  return String(Math.round(n));
}

function mbpsLabel(raw) {
  var n = formatMbps(raw);
  return n ? n + " Mbps" : "";
}

function mbpsCopyText(direction, raw) {
  var label = mbpsLabel(raw);
  if (!label) return "";
  var prefix = String(direction || "").replace(/^\s+|\s+$/g, "");
  return prefix ? prefix + " " + label : label;
}

function parseMonitorMode(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  var m = /^(\d+)\s*[x×]\s*(\d+)\s*@\s*([0-9.]+)\s*Hz$/i.exec(text);
  if (!m) return null;
  var width = Number(m[1]);
  var height = Number(m[2]);
  var refresh = Number(m[3]);
  if (!isFinite(width) || !isFinite(height) || !isFinite(refresh)) return null;
  if (width <= 0 || height <= 0 || refresh < 0) return null;
  return { raw: text, width: width, height: height, refresh: refresh };
}

function formatMonitorHz(refresh) {
  var n = Number(refresh);
  if (!isFinite(n) || n < 0) return "";
  var rounded = Math.round(n);
  if (Math.abs(n - rounded) < 0.05) return String(rounded);
  return String(Math.round(n * 100) / 100);
}

function formatMonitorMode(mode) {
  var parsed =
    mode && typeof mode === "object" && isFinite(mode.width) ? mode : parseMonitorMode(mode);
  if (!parsed) return "";
  var hz = formatMonitorHz(parsed.refresh);
  return parsed.width + "×" + parsed.height + (hz ? " @ " + hz + " Hz" : "");
}

function currentMonitorModeValue(monitor) {
  if (!monitor) return "";
  var w = Number(monitor.width);
  var h = Number(monitor.height);
  var r = Number(monitor.refresh);
  var list = Array.isArray(monitor.availableModes) ? monitor.availableModes : [];
  var best = "";
  var bestDiff = 1e9;
  for (var i = 0; i < list.length; i++) {
    var parsed = parseMonitorMode(list[i]);
    if (!parsed || parsed.width !== w || parsed.height !== h) continue;
    var diff = Math.abs(parsed.refresh - r);
    if (diff < bestDiff) {
      bestDiff = diff;
      best = String(list[i]);
    }
  }
  if (best) return best;
  if (isFinite(w) && isFinite(h) && w > 0 && h > 0)
    return w + "x" + h + "@" + (isFinite(r) && r > 0 ? r : 0) + "Hz";
  return "";
}

function monitorModeOptions(monitor) {
  var list = monitor && Array.isArray(monitor.availableModes) ? monitor.availableModes : [];
  var seen = {};
  var out = [];
  function add(raw) {
    var text = String(raw || "");
    if (!text || seen[text]) return;
    var parsed = parseMonitorMode(text);
    seen[text] = true;
    out.push({ value: text, label: parsed ? formatMonitorMode(parsed) : text });
  }
  for (var i = 0; i < list.length; i++) add(list[i]);
  add(currentMonitorModeValue(monitor));
  return out;
}

function monitorModeCopyText(monitor) {
  var label = formatMonitorMode(currentMonitorModeValue(monitor));
  var name = monitor && monitor.name ? String(monitor.name) : "";
  if (label && name) return name + " " + label;
  return label || name;
}

function parseDiskSpeedLine(line) {
  var text = String(line || "").replace(/^\s+|\s+$/g, "");
  if (!text) return null;
  var parts = text.split(/\s+/);
  if (parts.length < 2) return null;
  if (parts[0] === "disk") return { kind: "disk", value: parts.slice(1).join(" ") };
  if (parts[0] === "read" || parts[0] === "write") {
    var n = parseFloat(parts[1]);
    if (!isFinite(n) || n < 0) return null;
    return { kind: parts[0], value: n };
  }
  return null;
}

function parseSnapperList(raw) {
  var data;
  try {
    data = typeof raw === "string" ? JSON.parse(raw || "{}") : raw;
  } catch (e) {
    return [];
  }
  var out = [];
  if (!data || typeof data !== "object") return out;

  function pushEntry(config, entry) {
    if (!entry || typeof entry !== "object") return;
    var id = entry.number !== undefined ? entry.number : entry.id;
    var n = Math.round(Number(id));
    if (!isFinite(n) || n < 1) return;
    out.push({
      config: String(config || ""),
      id: n,
      date: String(entry.date || entry.dateIso || ""),
      description: String(entry.description || entry.userdata || ""),
      type: String(entry.type || ""),
    });
  }

  if (Array.isArray(data)) {
    for (var i = 0; i < data.length; i++) pushEntry("", data[i]);
    return out;
  }

  var keys = Object.keys(data);
  for (var k = 0; k < keys.length; k++) {
    var name = keys[k];
    var list = data[name];
    if (!Array.isArray(list)) continue;
    for (var j = 0; j < list.length; j++) pushEntry(name, list[j]);
  }
  out.sort(function (a, b) {
    return b.id - a.id;
  });
  return out;
}

function isIpv4(token) {
  var parts = String(token || "").split(".");
  if (parts.length !== 4) return false;
  for (var i = 0; i < 4; i++) {
    if (!/^[0-9]{1,3}$/.test(parts[i])) return false;
    var n = parseInt(parts[i], 10);
    if (n > 255) return false;
  }
  return true;
}

function ipv6SplitGroups(s) {
  if (s === "") return [];
  var g = String(s).split(":");
  for (var i = 0; i < g.length; i++) {
    if (!/^[0-9a-fA-F]{1,4}$/.test(g[i])) return null;
  }
  return g;
}

function isIpv6(token) {
  var t = String(token || "");
  if (t.indexOf(":") === -1) return false;
  if (/[^0-9a-fA-F:]/.test(t)) return false;
  if (t === "::") return false;
  var first = t.indexOf("::");
  if (first !== -1 && t.indexOf("::", first + 2) !== -1) return false;
  if (first === -1) {
    var full = ipv6SplitGroups(t);
    return !!(full && full.length === 8);
  }
  var left = ipv6SplitGroups(t.slice(0, first));
  var right = ipv6SplitGroups(t.slice(first + 2));
  if (!left || !right) return false;
  return left.length + right.length < 8;
}

function isPartialIpv4(token) {
  var t = String(token || "");
  if (!t) return true;
  if (/[^0-9.]/.test(t)) return false;
  if (t.indexOf("..") !== -1) return false;
  var parts = t.split(".");
  if (parts.length > 4) return false;
  for (var i = 0; i < parts.length; i++) {
    if (parts[i] === "") {
      if (i === parts.length - 1) continue;
      return false;
    }
    if (!/^[0-9]{1,3}$/.test(parts[i])) return false;
    if (parseInt(parts[i], 10) > 255) return false;
  }
  return true;
}

function isPartialIpv6(token) {
  var t = String(token || "");
  if (!t) return true;
  if (/[^0-9a-fA-F:]/.test(t)) return false;
  if (t.indexOf(":::") !== -1) return false;
  var first = t.indexOf("::");
  if (first !== -1 && t.indexOf("::", first + 2) !== -1) return false;
  var body = t;
  if (body.charAt(body.length - 1) === ":" && body.slice(-2) !== "::") body = body.slice(0, -1);
  if (body === "" || body === ":") return false;
  first = body.indexOf("::");
  function count(s) {
    if (s === "") return 0;
    var g = s.split(":");
    for (var i = 0; i < g.length; i++) {
      if (g[i] === "") return -1;
      if (!/^[0-9a-fA-F]{1,4}$/.test(g[i])) return -1;
    }
    return g.length;
  }
  if (first === -1) {
    var n = count(body);
    return n >= 1 && n <= 8;
  }
  var left = count(body.slice(0, first));
  var right = count(body.slice(first + 2));
  if (left < 0 || right < 0) return false;
  return left + right < 8;
}

function formatIpv4Piece(token) {
  var s = String(token || "").replace(/[^\d.]/g, "");
  var octets = [];
  var extra = "";
  var cur = "";
  function commit() {
    if (octets.length >= 4) {
      extra += cur;
      cur = "";
      return;
    }
    octets.push(cur);
    cur = "";
  }
  for (var i = 0; i < s.length; i++) {
    var ch = s.charAt(i);
    if (ch === ".") {
      if (octets.length >= 4) continue;
      commit();
      continue;
    }
    if (octets.length >= 4) {
      extra += ch;
      continue;
    }
    var next = cur + ch;
    var n = parseInt(next, 10);
    if (next.length > 3 || n > 255) {
      if (cur === "") {
        extra += ch;
        continue;
      }
      commit();
      if (octets.length >= 4) {
        extra += ch;
        continue;
      }
      cur = ch;
      continue;
    }
    cur = next;
  }
  if (octets.length < 4) octets.push(cur);
  else extra += cur;
  var text = octets.join(".");
  if (s.charAt(s.length - 1) === "." && octets.length < 4 && octets[octets.length - 1] !== "")
    text += ".";
  return { text: text, extra: extra };
}

function formatDnsInput(text) {
  var s = String(text || "").replace(/[,\t;|\n\r]+/g, " ");
  var leading = /^\s/.test(s);
  var trailing = /\s$/.test(s);
  s = s.replace(/[^\s0-9a-fA-F.:]/g, "");
  var tokens = s.replace(/^\s+|\s+$/g, "").split(/\s+/);
  if (tokens.length === 1 && tokens[0] === "") return trailing || leading ? " " : "";
  var out = [];
  for (var i = 0; i < tokens.length; i++) {
    var tok = tokens[i];
    if (!tok) continue;
    if (tok.indexOf(":") !== -1) {
      out.push(tok.replace(/[^0-9a-fA-F:]/g, "").toLowerCase());
      continue;
    }
    var piece = formatIpv4Piece(tok);
    out.push(piece.text);
    var rest = piece.extra;
    while (rest) {
      var more = formatIpv4Piece(rest);
      if (!more.text) break;
      out.push(more.text);
      if (more.extra === rest) break;
      rest = more.extra;
    }
  }
  var result = out.join(" ");
  if (trailing) result += " ";
  return result;
}

function dnsInputStatus(text) {
  var formatted = formatDnsInput(text);
  var trimmed = formatted.replace(/^\s+|\s+$/g, "");
  if (!trimmed) return { ok: false, error: "", servers: [], formatted: formatted };
  var trailingSpace = /\s$/.test(formatted);
  var tokens = trimmed.split(/\s+/);
  var servers = [];
  for (var i = 0; i < tokens.length; i++) {
    var token = tokens[i];
    if (token.charAt(0) === "-")
      return {
        ok: false,
        error: "DNS servers cannot start with -",
        servers: servers,
        formatted: formatted,
      };
    if (isIpv4(token) || isIpv6(token)) {
      servers.push(token);
      continue;
    }
    var last = i === tokens.length - 1 && !trailingSpace;
    if (last && (isPartialIpv4(token) || isPartialIpv6(token)))
      return { ok: false, error: "", servers: servers, formatted: formatted };
    return {
      ok: false,
      error: "Not an IPv4 or IPv6 address: " + token,
      servers: servers,
      formatted: formatted,
    };
  }
  return { ok: true, error: "", servers: servers, formatted: formatted };
}

function parseDnsServers(text) {
  var status = dnsInputStatus(text);
  if (status.ok) return { ok: true, error: "", servers: status.servers };
  var error = status.error || "Enter one or more DNS servers";
  return { ok: false, error: error, servers: status.servers || [] };
}

function wifiRow(ssid, signal, securityKind, connected, known) {
  return {
    ssid: String(ssid || ""),
    signal: Math.round(Number(signal) || 0),
    securityKind: String(securityKind || "unknown"),
    connected: !!connected,
    known: !!known,
  };
}

function sortWifiRows(rows) {
  var nets = Array.isArray(rows) ? rows.slice() : [];
  nets.sort(function (a, b) {
    if (a.connected !== b.connected) return a.connected ? -1 : 1;
    if (a.known !== b.known) return a.known ? -1 : 1;
    return (b.signal || 0) - (a.signal || 0);
  });
  return nets;
}

function requiresCredentials(kind) {
  return kind !== "open" && kind !== "owe";
}

function isEnterprise(kind) {
  return kind === "enterprise";
}

function bluetoothRow(address, name, connected, paired) {
  return {
    address: String(address || ""),
    name: String(name || ""),
    connected: !!connected,
    paired: !!paired,
  };
}

function objectList(values) {
  if (!values) return [];
  if (Array.isArray(values)) return values.slice();
  var length = Number(values.length || 0);
  if (!isFinite(length) || length <= 0) return [];
  var list = [];
  var i;
  for (i = 0; i < length; i++) list.push(values[i]);
  return list;
}

function bluetoothDeviceName(device) {
  if (!device) return "";
  return String(device.deviceName || device.name || "").replace(/^\s+|\s+$/g, "");
}

function bluetoothLists(devices) {
  var values = objectList(devices);
  var connected = [];
  var known = [];
  var discovered = [];
  var i;
  for (i = 0; i < values.length; i++) {
    var d = values[i];
    if (!d || !d.address) continue;
    var name = bluetoothDeviceName(d);
    if (!name) name = String(d.address || "");
    if (!name) continue;
    var paired = !!(d.paired || d.bonded || d.trusted);
    var row = bluetoothRow(d.address, name, d.connected, paired);
    if (d.connected) connected.push(row);
    else if (paired) known.push(row);
    else discovered.push(row);
  }
  return {
    connected: connected,
    known: known,
    discovered: discovered,
    paired: connected.concat(known),
  };
}

function reminderCopyText(row) {
  if (!row) return "";
  var label = String(row.label || row.message || "").replace(/[\r\n\0]/g, " ");
  var bits = [];
  if (row.remaining) bits.push("in " + String(row.remaining).replace(/[\r\n\0]/g, " "));
  if (row.atTime) bits.push("at " + String(row.atTime).replace(/[\r\n\0]/g, " "));
  var extra = String(row.message || "").replace(/[\r\n\0]/g, " ");
  if (extra && extra !== label) bits.push(extra);
  var text = bits.length ? label + " — " + bits.join(" · ") : label;
  if (text.length > 1024) text = text.substring(0, 1024);
  return text;
}

function bindingCopyText(row) {
  if (!row) return "";
  var keys = String(row.keys || "").replace(/[\r\n\0]/g, " ");
  var action = String(row.action || row.command || row.label || "").replace(/[\r\n\0]/g, " ");
  if (keys && action && action !== keys) return keys + " — " + action;
  return keys || action;
}

function parseReminders(raw) {
  var data;
  try {
    data = typeof raw === "string" ? JSON.parse(raw || "{}") : raw;
  } catch (e) {
    return [];
  }
  var list = data && data.reminders;
  if (!Array.isArray(list)) return [];
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var item = list[i];
    if (!item) continue;
    out.push({
      unit: String(item.unit || ""),
      label: String(item.label || item.message || ""),
      message: String(item.message || ""),
      remaining: String(item.remaining || ""),
      atTime: String(item.atTime || ""),
      minutes: Math.round(Number(item.minutes) || 0),
    });
  }
  return out;
}

function formatBytes(bytes) {
  var n = Number(bytes);
  if (!isFinite(n) || n < 0) n = 0;
  var units = ["B", "KB", "MB", "GB", "TB"];
  var i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n = n / 1024;
    i++;
  }
  if (i === 0) return Math.round(n) + " " + units[i];
  return n.toFixed(n >= 10 ? 1 : 2) + " " + units[i];
}

function usagePercent(used, size) {
  var u = Number(used);
  var s = Number(size);
  if (!isFinite(u) || !isFinite(s) || s <= 0) return 0;
  var pct = Math.round((u / s) * 100);
  if (pct < 0) return 0;
  if (pct > 100) return 100;
  return pct;
}

function fileBasename(path) {
  var s = String(path || "");
  if (!s) return "";
  var i = s.lastIndexOf("/");
  return i === -1 ? s : s.substring(i + 1);
}

function pathFromUrl(url) {
  var s = String(url || "");
  if (s.indexOf("file://") === 0) {
    s = s.substring(7);
    if (/^\/[A-Za-z]:\//.test(s)) s = s.substring(1);
  }
  try {
    s = decodeURIComponent(s);
  } catch (e) {}
  return s;
}

function isTimezoneId(id) {
  var text = String(id || "");
  if (!text || text.length > 64) return false;
  if (text.indexOf("..") !== -1) return false;
  if (text.charAt(0) === "-" || text.charAt(0) === "/") return false;
  return /^[A-Za-z0-9/_+-]+$/.test(text);
}

function parseTimezoneId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isTimezoneId(text) ? text : "";
}

function parseTimedatectlYes(raw) {
  var text = String(raw || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  return text === "yes" || text === "true" || text === "1" || text === "on";
}

function isLocaleId(id) {
  var text = String(id || "");
  if (text === "C.UTF-8") return true;
  return /^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$/.test(text);
}

function parseLocaleId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isLocaleId(text) ? text : "";
}

function isFullName(name) {
  var text = String(name || "").replace(/^\s+|\s+$/g, "");
  if (text.length > 256) return false;
  if (text.charAt(0) === "-") return false;
  if (/[:\n\r,]/.test(text)) return false;
  return true;
}

function parseFullName(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isFullName(text) ? text : "";
}

function isLauncherName(name) {
  var text = String(name || "");
  if (!text || text.length > 80) return false;
  if (text.charAt(0) === "-") return false;
  if (/[/\n\r]/.test(text)) return false;
  return true;
}

function parseLauncherName(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isLauncherName(text) ? text : "";
}

function parseWebAppUrl(raw) {
  var url = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!url || /\s/.test(url)) return "";
  if (!/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(url)) url = "https://" + url;
  return /^https?:\/\//i.test(url) ? url : "";
}

// Transports omarchy-git-url-check clones. ext:: and fd:: stay out because
// git would run a helper instead of fetching a repository.
var GIT_URL_TRANSPORTS = {
  ssh: true,
  git: true,
  "git+ssh": true,
  "ssh+git": true,
  http: true,
  https: true,
  ftp: true,
  ftps: true,
  file: true,
};

function gitThemeName(url) {
  var path = String(url || "");
  var colon = path.indexOf(":");
  if (path.indexOf("://") === -1 && colon !== -1 && path.substring(0, colon).indexOf("/") === -1)
    path = path.substring(colon + 1);
  var i = path.lastIndexOf("/");
  var base = i === -1 ? path : path.substring(i + 1);
  if (base.length > 4 && base.substring(base.length - 4) === ".git")
    base = base.substring(0, base.length - 4);
  if (base.indexOf("omarchy-") === 0) base = base.substring(8);
  if (base.length >= 6 && base.substring(base.length - 6) === "-theme")
    base = base.substring(0, base.length - 6);
  base = base.toLowerCase();
  if (!base || !/^[a-z0-9_][a-z0-9._+-]*$/.test(base)) return "";
  return base;
}

// omarchy-theme-install runs omarchy-git-url-check, then names the
// directory from the URL. Flags, helpers, and unknown schemes stay here.
function parseGitUrl(raw) {
  var url = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!url || url.length > 2048) return "";
  if (/[\s\r\n]/.test(url)) return "";
  if (url.charAt(0) === "-") return "";
  if (/^[A-Za-z0-9][A-Za-z0-9+.-]*::/.test(url)) return "";
  var schemeM = url.match(/^([A-Za-z0-9][A-Za-z0-9+.-]*):\/\//);
  if (schemeM) {
    var scheme = schemeM[1].toLowerCase();
    if (!GIT_URL_TRANSPORTS[scheme]) return "";
    if (scheme !== schemeM[1]) url = scheme + url.substring(schemeM[1].length);
  } else {
    var colon = url.indexOf(":");
    var slash = url.indexOf("/");
    if (colon <= 0) return "";
    if (slash !== -1 && slash < colon) return "";
  }
  if (!gitThemeName(url)) return "";
  return url;
}

function isTuiWindowStyle(style) {
  return style === "float" || style === "tile";
}

function parseParallelDownloads(text) {
  var lines = String(text || "").split("\n");
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    if (/^\s*#/.test(line)) continue;
    var m = line.match(/^\s*ParallelDownloads\s*=\s*([0-9]+)\s*$/);
    if (!m) continue;
    var n = parseInt(m[1], 10);
    if (!isFinite(n) || n < 1) return 0;
    if (n > 20) return 20;
    return n;
  }
  return 0;
}

function isHostname(name) {
  var text = String(name || "");
  if (!text || text.length > 253) return false;
  if (text.charAt(0) === "-") return false;
  var labels = text.split(".");
  var labelRe = /^[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?$/;
  for (var i = 0; i < labels.length; i++) {
    if (!labelRe.test(labels[i])) return false;
  }
  return true;
}

function parseHostname(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  return isHostname(text) ? text : "";
}

// omarchy-weather-location --set <name> [lat,lon]. A leading hyphen would
// look like a flag if a caller ever concatenates; newlines cannot live in
// weather.json as a single-line city.
function parseWeatherLocation(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 128) return "";
  if (text.charAt(0) === "-") return "";
  if (/[\r\n]/.test(text)) return "";
  return text;
}

function coordToken(raw) {
  var text = String(raw || "");
  if (text.charAt(0) === "+") text = text.substring(1);
  return text;
}

// omarchy-weather-location --set name lat,lon. Spaces around the comma are
// usual; the CLI pattern does not allow them, so we strip to that form.
function parseWeatherCoords(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  var m = text.match(/^([+-]?[0-9]+(?:\.[0-9]+)?)\s*,\s*([+-]?[0-9]+(?:\.[0-9]+)?)$/);
  if (!m) return "";
  var lat = Number(m[1]);
  var lon = Number(m[2]);
  if (!isFinite(lat) || !isFinite(lon)) return "";
  if (lat < -90 || lat > 90 || lon < -180 || lon > 180) return "";
  return coordToken(m[1]) + "," + coordToken(m[2]);
}

function isSshKeyType(type) {
  var text = String(type || "");
  if (!text || text.length > 80 || text.charAt(0) === "-") return false;
  return /^(?:sk-)?(?:ssh-[a-z0-9]+|ecdsa-sha2-nistp(?:256|384|521))(?:-cert-v01@openssh\.com|@openssh\.com)?$/.test(
    text,
  );
}

// One authorized_keys line. omarchy-setup-security-sshd starts sshd before
// it checks the key, so junk must not leave this function.
function parseSshPublicKey(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 8192) return "";
  if (/[\r\n]/.test(text) || text.indexOf("-----") !== -1) return "";
  if (text.charAt(0) === "-") return "";
  var parts = text.split(/\s+/);
  if (parts.length < 2) return "";
  if (!isSshKeyType(parts[0])) return "";
  if (!/^[A-Za-z0-9+/]{32,}={0,2}$/.test(parts[1])) return "";
  var blobAt = text.indexOf(parts[1]);
  var comment = text.substring(blobAt + parts[1].length).replace(/^\s+/, "");
  return comment ? parts[0] + " " + parts[1] + " " + comment : parts[0] + " " + parts[1];
}

function isKeyboardLayoutId(id) {
  var text = String(id || "");
  return /^[a-z0-9]{1,8}$/.test(text);
}

function parseKeyboardLayoutId(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (text.indexOf(",") !== -1) text = text.split(",")[0];
  return isKeyboardLayoutId(text) ? text : "";
}

function sliderValueUnit(valueText) {
  var s = String(valueText || "");
  var m = s.match(/^-?\d+(?:\.\d+)?(.*)$/);
  return m ? m[1] : "";
}

function formatSliderNumber(v) {
  var n = Number(v);
  if (!isFinite(n)) n = 0;
  if (Math.abs(n - Math.round(n)) < 0.001) return String(Math.round(n));
  return String(Math.round(n * 100) / 100);
}

function formatSliderCaption(v, valueText, tickText) {
  if (tickText) return String(tickText);
  return formatSliderNumber(v) + sliderValueUnit(valueText);
}

function sliderTickNear(a, b) {
  return Math.abs(Number(a) - Number(b)) <= 1e-8;
}

function sliderTickClean(v) {
  return Math.round(Number(v) * 1e10) / 1e10;
}

function sliderTickUnique(list) {
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var v = sliderTickClean(list[i]);
    if (!out.length || !sliderTickNear(out[out.length - 1], v)) out.push(v);
  }
  return out;
}

function sliderEvenSplitTicks(from, to, step, parts) {
  var span = to - from;
  var steps = Math.round(span / step);
  if (parts < 1 || steps % parts !== 0) return null;
  var every = steps / parts;
  var ticks = [];
  for (var i = 0; i <= parts; i++) ticks.push(sliderTickClean(from + every * i * step));
  return ticks;
}

function sliderNiceInterval(raw, step) {
  var units = raw / step;
  if (!(units > 0) || !isFinite(units)) return step;
  var mag = Math.pow(10, Math.floor(Math.log(units) / Math.LN10));
  var r = units / mag;
  var niceU;
  if (r <= 1.5) niceU = 1;
  else if (r <= 2.25) niceU = 2;
  else if (r <= 3.5) niceU = 2.5;
  else if (r <= 7.5) niceU = 5;
  else niceU = 10;
  return niceU * mag * step;
}

function sliderNiceGridTicks(from, to, step, parts) {
  var nice = sliderNiceInterval((to - from) / parts, step);
  if (!(nice > 0)) nice = step;
  var ticks = [from];
  var start = Math.ceil((from + nice * 0.01) / nice) * nice;
  for (var v = start; v < to - nice * 0.01; v += nice) {
    var snapped = from + Math.round((v - from) / step) * step;
    if (snapped > from + step * 0.5 && snapped < to - step * 0.5) ticks.push(snapped);
  }
  ticks.push(to);
  return sliderTickUnique(ticks);
}

// Major tick values for a full-width slider. Always includes the endpoints.
// The slider itself keeps `step`; this only picks labels.
function sliderTickValues(from, to, step) {
  from = Number(from);
  to = Number(to);
  step = Number(step);
  if (!isFinite(from)) from = 0;
  if (!isFinite(to)) to = 0;
  if (!(step > 0) || !isFinite(step)) step = 1;
  if (!(to > from)) return [from];
  var n = Math.round((to - from) / step);
  if (n <= 4) {
    var all = [];
    for (var i = 0; i <= n; i++) all.push(sliderTickClean(from + i * step));
    return all;
  }
  var four = sliderEvenSplitTicks(from, to, step, 4);
  if (four) return four;
  var grid = sliderNiceGridTicks(from, to, step, 4);
  if (grid.length > 5) grid = sliderNiceGridTicks(from, to, step, 2);
  if (grid.length >= 3 && grid.length <= 5) return grid;
  var two = sliderEvenSplitTicks(from, to, step, 2);
  if (two) return two;
  var three = sliderEvenSplitTicks(from, to, step, 3);
  if (three) return three;
  if (grid.length >= 3) return grid;
  return [from, to];
}

function sliderTickTextWidth(text, charWidth) {
  return String(text || "").length * charWidth;
}

// Drop interior labels that would overlap at the current track width.
// Endpoints stay.
function sliderFitTicks(ticks, from, to, trackWidth, labels, charWidth, minGap) {
  var list = Array.isArray(ticks) ? ticks : [];
  if (list.length <= 2) return list.slice();
  var span = Number(to) - Number(from);
  var width = Number(trackWidth);
  if (!(span > 0) || !(width > 0)) return [list[0], list[list.length - 1]];
  var cw = Number(charWidth);
  if (!(cw > 0)) cw = 8;
  var gap = Number(minGap);
  if (!(gap > 0)) gap = 8;
  var names = Array.isArray(labels) ? labels : [];

  function half(i) {
    var text = i < names.length ? names[i] : formatSliderNumber(list[i]);
    return sliderTickTextWidth(text, cw) / 2;
  }
  function xOf(v) {
    return ((v - from) / span) * width;
  }

  var last = list.length - 1;
  var lastLeft = xOf(list[last]) - half(last);
  var kept = [list[0]];
  var prevRight = xOf(list[0]) + half(0);
  for (var i = 1; i < last; i++) {
    var x = xOf(list[i]);
    var h = half(i);
    if (x - h < prevRight + gap) continue;
    if (x + h + gap > lastLeft) continue;
    kept.push(list[i]);
    prevRight = x + h;
  }
  kept.push(list[last]);
  return kept;
}

function sliderLiveState(intervalMs) {
  var n = Number(intervalMs);
  return {
    intervalMs: isFinite(n) && n > 0 ? n : 100,
    lastEmitMs: 0,
    pending: undefined,
    lastSent: undefined,
  };
}

function sliderLiveTake(state, now) {
  var value = state.pending;
  state.pending = undefined;
  if (value === undefined || value === state.lastSent) return { emit: undefined, delayMs: 0 };
  state.lastSent = value;
  state.lastEmitMs = now;
  return { emit: value, delayMs: 0 };
}

function sliderLivePush(state, now, value) {
  state.pending = value;
  var wait = state.intervalMs - (now - state.lastEmitMs);
  if (wait <= 0) return sliderLiveTake(state, now);
  return { emit: undefined, delayMs: wait };
}

function sliderLiveFlush(state, now, value) {
  state.pending = value;
  return sliderLiveTake(state, now);
}

function agentErrorPrompt(err) {
  var text = clipboardPayload(err, { maxLength: 8192 });
  if (!text) return "";
  return "Atmos hit an error and I want help fixing it.\n\nThe error:\n" + text + "\n";
}

function confirmIsDestructive(text) {
  var t = String(text || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!t) return false;
  if (
    t === "remove" ||
    t === "forget" ||
    t === "clear" ||
    t === "reset" ||
    t === "undo" ||
    t === "prune" ||
    t === "restore" ||
    t === "turn off"
  )
    return true;
  return t.indexOf("roll back") !== -1;
}

function clipboardPayload(text, opts) {
  opts = opts && typeof opts === "object" ? opts : {};
  var s = String(text || "");
  if (!s) return "";
  if (s.indexOf("\0") !== -1) return "";
  if (opts.singleLine && /[\r\n]/.test(s)) return "";
  var max = Number(opts.maxLength);
  if (!isFinite(max) || max <= 0) max = 1024;
  if (s.length > max) s = s.substring(0, max);
  return s;
}

function parseXkbLayoutList(text) {
  var lines = String(text || "").split("\n");
  var inLayout = false;
  var out = [];
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    if (line.indexOf("! ") === 0) {
      inLayout = /^\s*! layout\s*$/.test(line);
      continue;
    }
    if (!inLayout) continue;
    if (!String(line).replace(/^\s+|\s+$/g, "")) {
      if (out.length > 0) break;
      continue;
    }
    var m = line.match(/^\s+([a-z0-9]{1,8})\s+(\S.*)$/);
    if (!m) continue;
    out.push({ value: m[1], label: m[2] });
  }
  return out;
}
