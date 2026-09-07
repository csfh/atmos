// Health snapshot for the Diagnostics page and the copyable report.
// QML and Node both eval this file.

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function cleanLine(value, max) {
  var s = String(value == null ? "" : value)
    .replace(/[\r\n\t]+/g, " ")
    .replace(/^\s+|\s+$/g, "");
  if (!s) return "";
  var lower = s.toLowerCase();
  if (
    lower.indexOf("password") !== -1 ||
    lower.indexOf("passwd") !== -1 ||
    lower.indexOf("secret") !== -1 ||
    lower.indexOf("token") !== -1 ||
    lower.indexOf("bearer ") !== -1
  )
    return "";
  var limit = max || 240;
  if (s.length > limit) s = s.substring(0, limit);
  return s;
}

function cleanText(value, max) {
  return cleanLine(value, max);
}

function unitName(value) {
  var s = String(value == null ? "" : value).replace(/^\s+|\s+$/g, "");
  if (!s || s.length > 128) return "";
  if (!/^[A-Za-z0-9:_.@\\-]+$/.test(s)) return "";
  return s;
}

function serviceState(value) {
  var s = String(value == null ? "" : value).replace(/^\s+|\s+$/g, "");
  if (
    s === "active" ||
    s === "inactive" ||
    s === "failed" ||
    s === "activating" ||
    s === "deactivating" ||
    s === "unknown"
  )
    return s;
  return "";
}

function intOrZero(value) {
  var n = Math.round(Number(value));
  return isFinite(n) && n > 0 ? n : 0;
}

function intOrNone(value) {
  var n = Math.round(Number(value));
  return isFinite(n) && n >= 0 ? n : 0;
}

function asBool(value) {
  return value === true;
}

function emptyDiagnostics() {
  return {
    generatedAt: "",
    hostname: "",
    kernel: { sysname: "", release: "", machine: "" },
    omarchy: { version: "", channel: "", path: "" },
    atmos: {
      revision: "",
      installed: false,
      hyprAtmos: false,
      hyprAtmosLayout: false,
      sentinels: { look: false, input: false, autostart: false, bindings: false, windows: false },
    },
    hyprland: { version: "", configErrors: [] },
    failedUnits: [],
    disk: { path: "/", total: 0, used: 0, available: 0, percent: 0 },
    memory: { total: 0, used: 0, available: 0, swapTotal: 0, swapUsed: 0 },
    gpu: { driver: "", identity: "" },
    portals: {
      "xdg-desktop-portal": "",
      "xdg-desktop-portal-hyprland": "",
      "xdg-desktop-portal-gtk": "",
    },
    pipewire: { pipewire: "", "pipewire-pulse": "", wireplumber: "" },
    network: { online: false, kind: "" },
    pacman: { syncOk: false, lastSync: "", dbCount: 0 },
    recentErrors: [],
  };
}

function normalizeKernel(raw) {
  var src = asObject(raw);
  return {
    sysname: cleanText(src.sysname, 32),
    release: cleanText(src.release, 64),
    machine: cleanText(src.machine, 32),
  };
}

function normalizeOmarchy(raw) {
  var src = asObject(raw);
  return {
    version: cleanText(src.version, 64),
    channel: cleanText(src.channel, 32),
    path: cleanText(src.path, 240),
  };
}

function normalizeSentinels(raw) {
  var src = asObject(raw);
  return {
    look: asBool(src.look),
    input: asBool(src.input),
    autostart: asBool(src.autostart),
    bindings: asBool(src.bindings),
    windows: asBool(src.windows),
  };
}

function normalizeAtmos(raw) {
  var src = asObject(raw);
  return {
    revision: cleanText(src.revision, 80),
    installed: asBool(src.installed),
    hyprAtmos: asBool(src.hyprAtmos),
    hyprAtmosLayout: asBool(src.hyprAtmosLayout),
    sentinels: normalizeSentinels(src.sentinels),
  };
}

function normalizeHyprland(raw) {
  var src = asObject(raw);
  var errors = asArray(src.configErrors);
  var out = [];
  var i;
  for (i = 0; i < errors.length && out.length < 20; i++) {
    var line = cleanLine(errors[i], 240);
    if (line) out.push(line);
  }
  return { version: cleanText(src.version, 80), configErrors: out };
}

function normalizeFailedUnit(raw) {
  var src = asObject(raw);
  var unit = unitName(src.unit);
  if (!unit) return null;
  var scope = src.scope === "user" ? "user" : "system";
  return {
    unit: unit,
    scope: scope,
    sub: cleanText(src.sub, 32),
    description: cleanText(src.description, 160),
  };
}

function normalizeFailedUnits(raw) {
  var list = asArray(raw);
  var out = [];
  var i;
  for (i = 0; i < list.length && out.length < 24; i++) {
    var row = normalizeFailedUnit(list[i]);
    if (row) out.push(row);
  }
  return out;
}

function normalizeDisk(raw) {
  var src = asObject(raw);
  var path = String(src.path || "/") === "/" ? "/" : "/";
  var percent = intOrNone(src.percent);
  if (percent > 100) percent = 100;
  return {
    path: path,
    total: intOrZero(src.total),
    used: intOrNone(src.used),
    available: intOrNone(src.available),
    percent: percent,
  };
}

function normalizeMemory(raw) {
  var src = asObject(raw);
  return {
    total: intOrZero(src.total),
    used: intOrNone(src.used),
    available: intOrNone(src.available),
    swapTotal: intOrNone(src.swapTotal),
    swapUsed: intOrNone(src.swapUsed),
  };
}

function normalizeGpu(raw) {
  var src = asObject(raw);
  return { driver: cleanText(src.driver, 64), identity: cleanText(src.identity, 160) };
}

function normalizeServiceMap(raw, names) {
  var src = asObject(raw);
  var out = {};
  var i;
  for (i = 0; i < names.length; i++) out[names[i]] = serviceState(src[names[i]]);
  return out;
}

function normalizeNetwork(raw) {
  var src = asObject(raw);
  var kind = String(src.kind || "");
  if (kind !== "wifi" && kind !== "ethernet" && kind !== "offline") kind = "";
  return { online: asBool(src.online), kind: kind };
}

function normalizePacman(raw) {
  var src = asObject(raw);
  return {
    syncOk: asBool(src.syncOk),
    lastSync: cleanText(src.lastSync, 64),
    dbCount: intOrNone(src.dbCount),
  };
}

function normalizeErrors(raw) {
  var list = asArray(raw);
  var out = [];
  var i;
  for (i = 0; i < list.length && out.length < 15; i++) {
    var line = cleanLine(list[i], 240);
    if (line) out.push(line);
  }
  return out;
}

function hostnameText(value) {
  var s = String(value == null ? "" : value);
  if (/[\r\n]/.test(s)) return "";
  return cleanText(s, 253);
}

function normalize(raw) {
  var src = asObject(raw);
  return {
    generatedAt: cleanText(src.generatedAt, 64),
    hostname: hostnameText(src.hostname),
    kernel: normalizeKernel(src.kernel),
    omarchy: normalizeOmarchy(src.omarchy),
    atmos: normalizeAtmos(src.atmos),
    hyprland: normalizeHyprland(src.hyprland),
    failedUnits: normalizeFailedUnits(src.failedUnits),
    disk: normalizeDisk(src.disk),
    memory: normalizeMemory(src.memory),
    gpu: normalizeGpu(src.gpu),
    portals: normalizeServiceMap(src.portals, [
      "xdg-desktop-portal",
      "xdg-desktop-portal-hyprland",
      "xdg-desktop-portal-gtk",
    ]),
    pipewire: normalizeServiceMap(src.pipewire, ["pipewire", "pipewire-pulse", "wireplumber"]),
    network: normalizeNetwork(src.network),
    pacman: normalizePacman(src.pacman),
    recentErrors: normalizeErrors(src.recentErrors),
  };
}

function failedCount(diag) {
  return asArray(asObject(diag).failedUnits).length;
}

function hyprOk(diag) {
  var hypr = asObject(asObject(diag).hyprland);
  return asArray(hypr.configErrors).length === 0;
}

function servicesOk(map) {
  var src = asObject(map);
  var key;
  var any = false;
  for (key in src) {
    if (!Object.prototype.hasOwnProperty.call(src, key)) continue;
    var state = serviceState(src[key]);
    if (!state) continue;
    any = true;
    if (state !== "active") return false;
  }
  return any;
}

function diskPressure(diag) {
  return intOrNone(asObject(asObject(diag).disk).percent) >= 90;
}

function memoryPressure(diag) {
  var mem = asObject(asObject(diag).memory);
  if (!mem.total) return false;
  return mem.used / mem.total >= 0.95;
}

function atmosOk(diag) {
  var atmos = asObject(asObject(diag).atmos);
  return atmos.hyprAtmos === true;
}

function statusLabel(ok) {
  return ok ? "OK" : "Needs attention";
}

function kernelSummary(diag) {
  var k = asObject(asObject(diag).kernel);
  var parts = [];
  if (k.sysname) parts.push(k.sysname);
  if (k.release) parts.push(k.release);
  if (k.machine) parts.push(k.machine);
  return parts.join(" ");
}

function omarchySummary(diag) {
  var o = asObject(asObject(diag).omarchy);
  if (o.version && o.channel) return "Omarchy " + o.version + " (" + o.channel + ")";
  if (o.version) return "Omarchy " + o.version;
  return "Omarchy version was not readable.";
}

function atmosSummary(diag) {
  var a = asObject(asObject(diag).atmos);
  if (!a.installed) return "Atmos is not installed in XDG data.";
  var bits = [];
  if (a.revision) bits.push(a.revision);
  if (a.hyprAtmos) bits.push("hypr.atmos is required");
  else bits.push("hypr.atmos is missing from hyprland.lua");
  return bits.join(". ") + ".";
}

function hyprSummary(diag) {
  var h = asObject(asObject(diag).hyprland);
  var ver = h.version ? "Hyprland " + h.version : "Hyprland version was not readable";
  var n = asArray(h.configErrors).length;
  if (n === 0) return ver + ". No config errors.";
  if (n === 1) return ver + ". One config error.";
  return ver + ". " + n + " config errors.";
}

function failedSummary(diag) {
  var n = failedCount(diag);
  if (n === 0) return "No failed units.";
  if (n === 1) return "One failed unit.";
  return n + " failed units.";
}

function gpuSummary(diag) {
  var g = asObject(asObject(diag).gpu);
  if (g.driver && g.identity) return g.identity + " (" + g.driver + ")";
  if (g.driver) return g.driver;
  if (g.identity) return g.identity;
  return "GPU driver was not readable.";
}

function networkSummary(diag) {
  var n = asObject(asObject(diag).network);
  if (!n.online) return "No default route.";
  if (n.kind === "wifi") return "Online on Wi-Fi.";
  if (n.kind === "ethernet") return "Online on Ethernet.";
  return "Online.";
}

function pacmanSummary(diag) {
  var p = asObject(asObject(diag).pacman);
  if (!p.syncOk) return "Package databases were not readable.";
  if (p.lastSync) return p.dbCount + " sync databases. Last sync " + p.lastSync + ".";
  return p.dbCount + " sync databases.";
}

function serviceMapSummary(map) {
  var src = asObject(map);
  var bits = [];
  var key;
  for (key in src) {
    if (!Object.prototype.hasOwnProperty.call(src, key)) continue;
    var state = serviceState(src[key]);
    if (state) bits.push(key + " " + state);
  }
  return bits.length ? bits.join(". ") + "." : "No units reported.";
}

function pad2(n) {
  return n < 10 ? "0" + n : String(n);
}

function reportFileName(hostname, when) {
  var host = cleanText(hostname, 32).replace(/[^A-Za-z0-9._-]+/g, "") || "host";
  var d = when && typeof when.getFullYear === "function" ? when : new Date();
  var stamp =
    d.getFullYear() +
    pad2(d.getMonth() + 1) +
    pad2(d.getDate()) +
    "-" +
    pad2(d.getHours()) +
    pad2(d.getMinutes());
  return "atmos-diagnostics-" + host + "-" + stamp + ".txt";
}

function reportSection(title, body) {
  var text = String(body || "").replace(/^\s+|\s+$/g, "");
  if (!text) text = "(none)";
  return "== " + title + " ==\n" + text + "\n";
}

function sentinelsLine(sentinels) {
  var s = asObject(sentinels);
  var names = ["look", "input", "autostart", "bindings", "windows"];
  var bits = [];
  var i;
  for (i = 0; i < names.length; i++) bits.push(names[i] + "=" + (s[names[i]] ? "yes" : "no"));
  return bits.join(" ");
}

function failedLines(units) {
  var list = asArray(units);
  var lines = [];
  var i;
  for (i = 0; i < list.length; i++) {
    var row = list[i];
    if (!row || !row.unit) continue;
    var line = row.scope + " " + row.unit;
    if (row.sub) line += " " + row.sub;
    if (row.description) line += " - " + row.description;
    lines.push(line);
  }
  return lines.length ? lines.join("\n") : "None.";
}

function reportText(raw) {
  var d = normalize(raw);
  var lines = [
    "Atmos diagnostic report",
    d.generatedAt ? "Generated: " + d.generatedAt : "",
    d.hostname ? "Hostname: " + d.hostname : "",
    "",
    reportSection(
      "Omarchy",
      omarchySummary(d) + (d.omarchy.path ? "\nPath: " + d.omarchy.path : ""),
    ),
    reportSection("Atmos", atmosSummary(d) + "\nSentinels: " + sentinelsLine(d.atmos.sentinels)),
    reportSection(
      "Hyprland",
      hyprSummary(d) +
        (d.hyprland.configErrors.length ? "\n" + d.hyprland.configErrors.join("\n") : ""),
    ),
    reportSection("Failed systemd units", failedLines(d.failedUnits)),
    reportSection(
      "Disk",
      d.disk.total
        ? d.disk.path +
            " " +
            d.disk.percent +
            "% used (" +
            d.disk.used +
            " / " +
            d.disk.total +
            " bytes)"
        : "",
    ),
    reportSection(
      "Memory",
      d.memory.total
        ? "RAM used " +
            d.memory.used +
            " / " +
            d.memory.total +
            " bytes. Swap used " +
            d.memory.swapUsed +
            " / " +
            d.memory.swapTotal +
            " bytes."
        : "",
    ),
    reportSection("Kernel", kernelSummary(d)),
    reportSection("GPU", gpuSummary(d)),
    reportSection("Portals", serviceMapSummary(d.portals)),
    reportSection("PipeWire", serviceMapSummary(d.pipewire)),
    reportSection("Network", networkSummary(d)),
    reportSection("Package database", pacmanSummary(d)),
    reportSection("Recent errors", d.recentErrors.length ? d.recentErrors.join("\n") : "None."),
  ];
  var out = [];
  var i;
  for (i = 0; i < lines.length; i++) {
    if (lines[i] === "" && out.length && out[out.length - 1] === "") continue;
    out.push(lines[i]);
  }
  return out.join("\n").replace(/\s+$/, "") + "\n";
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    emptyDiagnostics: emptyDiagnostics,
    normalize: normalize,
    failedCount: failedCount,
    hyprOk: hyprOk,
    servicesOk: servicesOk,
    diskPressure: diskPressure,
    memoryPressure: memoryPressure,
    atmosOk: atmosOk,
    statusLabel: statusLabel,
    kernelSummary: kernelSummary,
    omarchySummary: omarchySummary,
    atmosSummary: atmosSummary,
    hyprSummary: hyprSummary,
    failedSummary: failedSummary,
    gpuSummary: gpuSummary,
    networkSummary: networkSummary,
    pacmanSummary: pacmanSummary,
    serviceMapSummary: serviceMapSummary,
    reportFileName: reportFileName,
    reportText: reportText,
  };
}
