// Derive a small live badge for each sidebar hub from snapshot state
// Atmos already holds. Icons stay; this is additive at the row edge.
//
// Silence is the common case. A hub with nothing to say renders exactly
// as it does today, and no badge ever shows a 0. Warn when something is
// actually wrong. A healthy multi-monitor desk stays silent rather than
// showing a count. Network and Bluetooth speak only on positive evidence,
// because snapshotReady can be true while declared defaults (netKind
// "disconnected", bluetooth false) are still in place.
//
// Disks stay silent: disk-inventory.py emits path/name/model/tran/size/
// info/mounts, not health or SMART. Do not badge a field the snapshot
// does not have.

function arr(v) {
  return Array.isArray(v) ? v : [];
}

function isFailedUnit(row, state) {
  // Caller passes Systemd.isFailed (shell.qml / Node tests). Do not copy
  // the failed-unit check; the Services page already owns that definition.
  if (!state || typeof state.isFailed !== "function") return false;
  return state.isFailed(row);
}

// tone: "warn" | "info". text is a glyph plus a small number, or a short
// kind mark. The column is 220px and the label already owns most of it.
function badge(text, tone, title) {
  if (text === null || text === undefined || text === "") return null;
  return { text: String(text), tone: tone || "info", title: title || "" };
}

function servicesBadge(state) {
  var units = arr(state.systemdUnits);
  var failed = 0;
  var i;
  for (i = 0; i < units.length; i++) {
    if (isFailedUnit(units[i], state)) failed += 1;
  }
  if (failed > 0)
    return badge("▲" + failed, "warn", failed + " failed unit" + (failed === 1 ? "" : "s"));
  return null;
}

function bluetoothBadge(state) {
  var devices = arr(state.bluetoothDevices);
  var connected = 0;
  var i;
  for (i = 0; i < devices.length; i++) {
    if (devices[i] && devices[i].connected === true) connected += 1;
  }
  if (connected > 0) return badge("●" + connected, "info", connected + " connected");
  return null;
}

function displayBadge(state) {
  var monitors = arr(state.monitors);
  var total = 0;
  var off = 0;
  var i;
  var row;
  for (i = 0; i < monitors.length; i++) {
    row = monitors[i];
    if (!row || typeof row !== "object") continue;
    total += 1;
    // Snapshot monitors use enabled (snapshot.sh: enabled: (.disabled != true)).
    // disabled lives on monitor rules, which this badge does not read.
    if (row.enabled === false) off += 1;
  }
  if (total === 0 || off === 0) return null;
  return badge(
    total - off + "/" + total,
    "warn",
    off + " output" + (off === 1 ? "" : "s") + " disabled",
  );
}

function networkBadge(state) {
  var kind = String(state.netKind || "").toLowerCase();
  if (kind === "ethernet") return badge("eth", "info", "Wired");
  if (kind === "wifi") {
    var ssid = String(state.netSsid || "");
    return badge("●", "info", ssid ? "Connected to " + ssid : "Connected");
  }
  // disconnected, empty, and anything else stay silent. Positive evidence
  // only: the singleton declares netKind "disconnected" before a snapshot
  // runs, and snapshotReady is set even when applySnapshot was skipped.
  return null;
}

function systemBadge(state) {
  var parts = [];
  if (state.updateAvailable === true) parts.push("Omarchy");
  if (state.atmosUpdateAvailable === true) parts.push("Atmos");
  if (parts.length === 0) return null;
  return badge(
    "↑" + parts.length,
    "info",
    parts.join(" and ") + " update" + (parts.length === 1 ? "" : "s") + " available",
  );
}

function forHub(id, state) {
  var s = state || {};
  if (s.ready !== true) return null;
  switch (String(id || "")) {
    case "services":
      return servicesBadge(s);
    case "bluetooth":
      return bluetoothBadge(s);
    case "display":
      return displayBadge(s);
    case "network":
      return networkBadge(s);
    case "system":
      return systemBadge(s);
    default:
      return null;
  }
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    arr: arr,
    isFailedUnit: isFailedUnit,
    badge: badge,
    servicesBadge: servicesBadge,
    bluetoothBadge: bluetoothBadge,
    displayBadge: displayBadge,
    networkBadge: networkBadge,
    systemBadge: systemBadge,
    forHub: forHub,
  };
}
