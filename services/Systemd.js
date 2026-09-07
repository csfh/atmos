// Allowlisted systemd units Atmos may start, stop, or enable.

function allowlist() {
  return [
    { unit: "pipewire.service", scope: "user", label: "PipeWire" },
    { unit: "pipewire-pulse.service", scope: "user", label: "PipeWire Pulse" },
    { unit: "wireplumber.service", scope: "user", label: "WirePlumber" },
    { unit: "xdg-desktop-portal.service", scope: "user", label: "Desktop portal" },
    { unit: "xdg-desktop-portal-hyprland.service", scope: "user", label: "Hyprland portal" },
    { unit: "bluetooth.service", scope: "system", label: "Bluetooth" },
    { unit: "NetworkManager.service", scope: "system", label: "NetworkManager" },
    { unit: "cups.service", scope: "system", label: "CUPS" },
    { unit: "docker.service", scope: "system", label: "Docker" },
    { unit: "tailscaled.service", scope: "system", label: "Tailscale" },
    { unit: "fstrim.timer", scope: "system", label: "TRIM timer" },
  ];
}

function unitName(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!s || s.length > 128) return "";
  if (!/^[A-Za-z0-9:_.@\\-]+$/.test(s)) return "";
  return s;
}

function findAllowed(unit, scope) {
  var list = allowlist();
  var name = unitName(unit);
  var sc = scope === "user" ? "user" : "system";
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i].unit === name && list[i].scope === sc) return list[i];
  }
  return null;
}

function canMutate(unit, scope) {
  return !!findAllowed(unit, scope);
}

function normalizeUnit(row) {
  if (!row || typeof row !== "object") return null;
  var unit = unitName(row.unit);
  if (!unit) return null;
  var scope = row.scope === "user" ? "user" : "system";
  var fileState = String(row.unitFileState || row.enabled || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  return {
    unit: unit,
    scope: scope,
    load: String(row.load || ""),
    active: String(row.active || ""),
    sub: String(row.sub || ""),
    unitFileState: fileState,
    description: String(row.description || "")
      .replace(/[\r\n]+/g, " ")
      .substring(0, 160),
    allowed: canMutate(unit, scope),
  };
}

function isFailed(row) {
  if (!row) return false;
  var active = String(row.active || "").toLowerCase();
  var sub = String(row.sub || "").toLowerCase();
  return active === "failed" || sub === "failed";
}

function isRunning(row) {
  if (!row || isFailed(row)) return false;
  return String(row.active || "").toLowerCase() === "active";
}

function isStopped(row) {
  return !!row && !isFailed(row) && !isRunning(row);
}

function runtimeState(row) {
  if (!row) return { key: "", mark: "", label: "" };
  var active = String(row.active || "").toLowerCase();
  if (isFailed(row)) return { key: "failed", mark: "×", label: "Failed" };
  if (active === "active") return { key: "running", mark: "●", label: "Running" };
  if (active === "activating") return { key: "starting", mark: "●", label: "Starting" };
  if (active === "deactivating") return { key: "stopping", mark: "○", label: "Stopping" };
  return { key: "stopped", mark: "○", label: "Stopped" };
}

function startupKey(row) {
  var raw = String((row && row.unitFileState) || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (raw === "enabled" || raw === "enabled-runtime") return "enabled";
  if (raw === "disabled") return "disabled";
  if (raw === "static") return "static";
  if (raw === "masked") return "masked";
  if (raw === "alias" || raw === "generated" || raw === "transient" || raw === "indirect")
    return raw;
  return "";
}

function startupState(row) {
  var key = startupKey(row);
  if (key === "enabled") return { key: key, label: "Enabled" };
  if (key === "disabled") return { key: key, label: "Disabled" };
  if (key === "static") return { key: key, label: "Static" };
  if (key === "masked") return { key: key, label: "Masked" };
  if (!key) return { key: "", label: "" };
  return { key: key, label: key.charAt(0).toUpperCase() + key.slice(1) };
}

function statusLine(row) {
  var run = runtimeState(row);
  var start = startupState(row);
  var parts = [];
  if (run.label) parts.push(run.mark + " " + run.label);
  if (start.label) parts.push(start.label);
  return parts.join("   ");
}

function searchHaystack(row) {
  if (!row) return "";
  var run = runtimeState(row);
  var start = startupState(row);
  return [
    row.unit,
    row.description,
    row.scope,
    run.label,
    start.label,
    row.allowed ? "allowlist" : "logs status",
  ].join(" ");
}

function matchesQuery(row, query) {
  var q = String(query || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!q) return true;
  return searchHaystack(row).toLowerCase().indexOf(q) !== -1;
}

function matchesFilter(row, filter) {
  var id = String(filter || "all");
  if (id === "all" || id === "") return true;
  if (id === "running") return isRunning(row);
  if (id === "stopped") return isStopped(row);
  if (id === "failed") return isFailed(row);
  if (id === "enabled") return startupKey(row) === "enabled";
  if (id === "disabled") return startupKey(row) === "disabled";
  return true;
}

function filterChips() {
  return [
    { id: "all", label: "All" },
    { id: "running", label: "Running" },
    { id: "stopped", label: "Stopped" },
    { id: "enabled", label: "Enabled" },
    { id: "disabled", label: "Disabled" },
    { id: "failed", label: "Failed" },
  ];
}

function summarize(rows) {
  var list = Array.isArray(rows) ? rows : [];
  var out = { total: 0, running: 0, stopped: 0, enabled: 0, failed: 0 };
  var i;
  for (i = 0; i < list.length; i++) {
    if (!list[i]) continue;
    out.total += 1;
    if (isFailed(list[i])) out.failed += 1;
    else if (isRunning(list[i])) out.running += 1;
    else out.stopped += 1;
    if (startupKey(list[i]) === "enabled") out.enabled += 1;
  }
  return out;
}

function summaryParts(sum) {
  var s = sum || summarize([]);
  return [
    { id: "running", text: s.running + " running" },
    { id: "stopped", text: s.stopped + " stopped" },
    { id: "enabled", text: s.enabled + " enabled" },
    {
      id: "failed",
      text: s.failed > 0 ? s.failed + " failed" : "✓ No failures",
      failed: s.failed > 0,
    },
  ];
}

function primaryAction(row) {
  if (!row || !row.allowed) return null;
  if (isFailed(row)) return { id: "restart", label: "Restart" };
  if (isRunning(row)) return { id: "stop", label: "Stop" };
  return { id: "start", label: "Start" };
}

function overflowActions(row) {
  var items = [];
  var primary = primaryAction(row);
  var primaryId = primary ? primary.id : "";
  var start = startupKey(row);
  if (row && row.allowed && isRunning(row) && primaryId !== "restart")
    items.push({ id: "restart", label: "Restart" });
  if (row && row.allowed && start !== "static" && start !== "masked" && start !== "") {
    if (start === "enabled") items.push({ id: "disable", label: "Disable at startup" });
    else items.push({ id: "enable", label: "Enable at startup" });
  }
  items.push({ id: "status", label: "View status" });
  items.push({ id: "logs", label: "View logs" });
  items.push({ id: "copy", label: "Copy unit name" });
  return items;
}

function listUnits(rows, query, filter) {
  var list = Array.isArray(rows) ? rows : [];
  var out = [];
  var i, row;
  for (i = 0; i < list.length; i++) {
    row = normalizeUnit(list[i]);
    if (!row) continue;
    if (!matchesQuery(row, query)) continue;
    if (!matchesFilter(row, filter)) continue;
    out.push(row);
  }
  return out;
}

function argvFor(action, unit, scope) {
  var allowed = findAllowed(unit, scope);
  if (!allowed) return null;
  var act = String(action || "");
  if (
    act !== "start" &&
    act !== "stop" &&
    act !== "restart" &&
    act !== "enable" &&
    act !== "disable"
  )
    return null;
  var cmd = ["systemctl"];
  if (allowed.scope === "user") cmd.push("--user");
  cmd.push(act, allowed.unit);
  return cmd;
}

function parseList(text, scope) {
  var lines = String(text || "").split("\n");
  var out = [];
  var i;
  for (i = 0; i < lines.length; i++) {
    var cols = String(lines[i] || "").split(/\s+/);
    if (!cols[0]) continue;
    var row = normalizeUnit({
      unit: cols[0],
      scope: scope,
      load: cols[1] || "",
      active: cols[2] || "",
      sub: cols[3] || "",
      description: cols.slice(4).join(" "),
    });
    if (row) out.push(row);
  }
  return out;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    allowlist: allowlist,
    unitName: unitName,
    findAllowed: findAllowed,
    canMutate: canMutate,
    normalizeUnit: normalizeUnit,
    isFailed: isFailed,
    isRunning: isRunning,
    isStopped: isStopped,
    runtimeState: runtimeState,
    startupKey: startupKey,
    startupState: startupState,
    statusLine: statusLine,
    searchHaystack: searchHaystack,
    matchesQuery: matchesQuery,
    matchesFilter: matchesFilter,
    filterChips: filterChips,
    summarize: summarize,
    summaryParts: summaryParts,
    primaryAction: primaryAction,
    overflowActions: overflowActions,
    listUnits: listUnits,
    argvFor: argvFor,
    parseList: parseList,
  };
}
