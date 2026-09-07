// nmcli argv for saved-connection extras. QML and Node both eval this file.

function sanitizeUuid(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!/^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/.test(text))
    return "";
  return text;
}

function sanitizeMetered(raw) {
  var text = String(raw || "");
  if (text === "yes" || text === "no" || text === "unknown") return text;
  return "";
}

function clampPriority(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n)) return 0;
  if (n < -999) n = -999;
  if (n > 999) n = 999;
  return n;
}

function sanitizeMac(raw) {
  var text = String(raw || "");
  if (
    text === "default" ||
    text === "random" ||
    text === "stable" ||
    text === "permanent" ||
    text === "preserve"
  )
    return text;
  return "";
}

function sanitizeIpv4(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  var m = /^([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})$/.exec(text);
  if (!m) return "";
  var i;
  for (i = 1; i <= 4; i++) {
    var n = Number(m[i]);
    if (n > 255) return "";
  }
  return text;
}

function sanitizePrefix(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 1 || n > 32) return 0;
  return n;
}

function sanitizeDns(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  var parts = text.split(/[ ,]+/);
  var out = [];
  var i;
  for (i = 0; i < parts.length; i++) {
    var ip = sanitizeIpv4(parts[i]);
    if (ip) out.push(ip);
  }
  return out.join(",");
}

function sanitizeHotspotSsid(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 32) return "";
  return text;
}

function sanitizeHotspotPassword(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  if (text.length < 8 || text.length > 63) return "";
  return text;
}

function sanitizeWgPath(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.charAt(0) !== "/") return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("..") !== -1) return "";
  if (!/\.(conf|nmconnection)$/.test(text)) return "";
  return text;
}

function argvFor(action, opts) {
  var act = String(action || "");
  var o = opts && typeof opts === "object" ? opts : {};
  if (act === "metered") {
    var uuid = sanitizeUuid(o.uuid);
    var metered = sanitizeMetered(o.value);
    if (!uuid || !metered) return null;
    return ["metered", uuid, metered];
  }
  if (act === "priority") {
    uuid = sanitizeUuid(o.uuid);
    if (!uuid) return null;
    return ["priority", uuid, String(clampPriority(o.value))];
  }
  if (act === "mac") {
    uuid = sanitizeUuid(o.uuid);
    var mac = sanitizeMac(o.value);
    if (!uuid || !mac) return null;
    return ["mac", uuid, mac];
  }
  if (act === "ipv4") {
    uuid = sanitizeUuid(o.uuid);
    if (!uuid) return null;
    if (o.method === "auto") return ["ipv4", uuid, "auto"];
    var address = sanitizeIpv4(o.address);
    var prefix = sanitizePrefix(o.prefix);
    var gateway = sanitizeIpv4(o.gateway);
    var dns = sanitizeDns(o.dns);
    if (!address || !prefix) return null;
    var argv = ["ipv4", uuid, "manual", address, String(prefix)];
    if (gateway) argv.push(gateway);
    else argv.push("");
    if (dns) argv.push(dns);
    return argv;
  }
  if (act === "wireguard-import") {
    var path = sanitizeWgPath(o.path);
    if (!path) return null;
    return ["wireguard-import", path];
  }
  if (act === "hotspot") {
    if (o.on === false) return ["hotspot", "off"];
    var ssid = sanitizeHotspotSsid(o.ssid);
    var password = sanitizeHotspotPassword(o.password);
    if (!ssid || !password) return null;
    return ["hotspot", "on", ssid, password];
  }
  return null;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    sanitizeUuid: sanitizeUuid,
    sanitizeMetered: sanitizeMetered,
    clampPriority: clampPriority,
    sanitizeMac: sanitizeMac,
    sanitizeIpv4: sanitizeIpv4,
    sanitizePrefix: sanitizePrefix,
    sanitizeHotspotSsid: sanitizeHotspotSsid,
    sanitizeWgPath: sanitizeWgPath,
    argvFor: argvFor,
  };
}
