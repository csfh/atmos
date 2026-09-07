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
  return {
    unit: unit,
    scope: scope,
    load: String(row.load || ""),
    active: String(row.active || ""),
    sub: String(row.sub || ""),
    description: String(row.description || "")
      .replace(/[\r\n]+/g, " ")
      .substring(0, 160),
    allowed: canMutate(unit, scope),
  };
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
    argvFor: argvFor,
    parseList: parseList,
  };
}
