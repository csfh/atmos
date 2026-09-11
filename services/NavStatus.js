// Derive a small live badge for each hub in the sidebar.
//
// The sidebar already carries a per-hub icon, but it is decoration -- it
// says the same thing whether a unit just failed or everything is fine.
// This turns that column into a status readout: the point is that you learn
// something about the machine without opening a page, and that anything
// actually wrong finds you instead of waiting to be discovered.
//
// Rules this follows on purpose:
//   * Only real state earns a badge. No badge is a valid, common answer,
//     and a hub with nothing to say stays silent rather than showing a 0.
//   * Only three inks, matching Theme: warn for something wrong, ok for
//     something actively good, info for a neutral count. Grayscale chrome
//     stays grayscale.
//   * Every read is defensive. This runs on every sidebar row on every
//     state change, so it must never throw on a half-populated snapshot.

function arr(v) {
  return Array.isArray(v) ? v : [];
}

function isFailedUnit(row) {
  if (!row) return false;
  var active = String(row.active || "").toLowerCase();
  var sub = String(row.sub || "").toLowerCase();
  return active === "failed" || sub === "failed";
}

// tone: "warn" | "ok" | "info". text is short -- one or two glyphs plus a
// small number, never a word, because the column is 220px wide and the
// label already owns most of it.
function badge(text, tone, title) {
  if (text === null || text === undefined || text === "") return null;
  return { text: String(text), tone: tone || "info", title: title || "" };
}

function servicesBadge(state) {
  var units = arr(state.systemdUnits);
  var failed = 0;
  for (var i = 0; i < units.length; i++) {
    if (isFailedUnit(units[i])) failed += 1;
  }
  if (failed > 0)
    return badge("▲" + failed, "warn", failed + " failed unit" + (failed === 1 ? "" : "s"));
  return null;
}

function bluetoothBadge(state) {
  // Positive evidence only, for the same reason as networkBadge: the
  // singleton declares bluetooth false before any snapshot has run, and the
  // ready gate can open while that default is still in place. Reporting a
  // radio "off" when it is on is the same false alarm in a different row.
  //
  // A connected device is something Atmos genuinely knows. A radio being off
  // is something the user turned off themselves.
  var devices = arr(state.bluetoothDevices);
  var connected = 0;
  for (var i = 0; i < devices.length; i++) {
    if (devices[i] && devices[i].connected === true) connected += 1;
  }
  if (connected > 0) return badge("●" + connected, "ok", connected + " connected");
  return null;
}

function displayBadge(state) {
  var monitors = arr(state.monitors);
  if (monitors.length === 0) return null;
  var off = 0;
  for (var i = 0; i < monitors.length; i++) {
    if (monitors[i] && monitors[i].disabled === true) off += 1;
  }
  if (off > 0)
    return badge(monitors.length - off + "/" + monitors.length, "warn", off + " output disabled");
  if (monitors.length > 1)
    return badge(String(monitors.length), "info", monitors.length + " outputs");
  return null;
}

function networkBadge(state) {
  var kind = String(state.netKind || "").toLowerCase();
  if (kind === "ethernet") return badge("eth", "ok", "Wired");
  if (kind === "wifi") {
    var ssid = String(state.netSsid || "");
    return badge("●", "ok", ssid ? "Connected to " + ssid : "Connected");
  }
  // Anything else is silent, including "disconnected".
  //
  // This badge has now cried wolf twice. The singleton declares netKind as
  // "disconnected" before any snapshot runs, and snapshotReady is set true
  // even when the snapshot was skipped or failed -- so the ready gate can
  // open while the defaults are still in place, and a working wifi
  // connection gets a warning triangle.
  //
  // Rather than chase a third guard, the badge now only ever speaks when it
  // has positive evidence: a named connection type. Being offline is the one
  // thing a user never needs an indicator to tell them, and a false alarm
  // here costs more than the missing signal is worth.
  return null;
}

function softwareBadge(state) {
  var n = 0;
  if (state.updateAvailable === true) n += 1;
  if (state.atmosUpdateAvailable === true) n += 1;
  if (n > 0) return badge("↑" + n, "info", n + " update" + (n === 1 ? "" : "s") + " available");
  return null;
}

function disksBadge(state) {
  var disks = arr(state.disks);
  var bad = 0;
  for (var i = 0; i < disks.length; i++) {
    var d = disks[i];
    if (!d) continue;
    var health = String(d.health || d.smart || "").toLowerCase();
    if (health && health !== "ok" && health !== "passed" && health !== "good") bad += 1;
  }
  if (bad > 0) return badge("▲" + bad, "warn", bad + " disk needs attention");
  return null;
}

function forHub(id, state) {
  var s = state || {};
  // Nothing is knowable until the snapshot has answered once.
  //
  // Guarding on undefined was not enough, and this is the second time this
  // bug has been fixed. The singleton's declared defaults are themselves
  // alarming values -- netKind starts at "disconnected" and bluetooth starts
  // at false -- so a freshly launched window reported the network down and
  // the radio off on a machine where neither was true, every single launch.
  // "Unknown" has to be asked about explicitly, because the resting state of
  // the data is indistinguishable from bad news.
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
    case "software":
      return softwareBadge(s);
    case "disks":
      return disksBadge(s);
    default:
      return null;
  }
}

// The hubs that can ever produce a badge, so a caller can cheaply skip the
// rest rather than running the switch for all 27 on every state change.
function badgedHubs() {
  return ["services", "bluetooth", "display", "network", "software", "disks"];
}

function hasBadge(id) {
  var list = badgedHubs();
  for (var i = 0; i < list.length; i++) {
    if (list[i] === id) return true;
  }
  return false;
}
