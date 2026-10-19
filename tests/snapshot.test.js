const { load, assert, assertEqual } = require("./harness");

const snapshot = load("services/Snapshot.js");
const snapMerged = snapshot.mergeSnapshot(
  { theme: "omarchy", hardware: { cpu: { model: "X" } } },
  { theme: "tokyo" },
);
assertEqual(snapMerged.theme, "tokyo", "mergeSnapshot updates theme from a partial patch");
assertEqual(
  snapMerged.hardware.cpu.model,
  "X",
  "mergeSnapshot keeps hardware when the patch omits it",
);
assertEqual(
  snapshot.mergeSnapshot({ theme: "omarchy" }, { stayAwake: false }).stayAwake,
  false,
  "mergeSnapshot keeps false",
);
assertEqual(snapshot.parseSnapshot("{"), null, "parseSnapshot rejects junk");

const hypr = load("services/HyprPrefs.js");
const accounts = load("services/Accounts.js");
const hardware = load("services/Hardware.js");
const diagnostics = load("services/Diagnostics.js");
const sunset = load("services/HyprSunset.js");
const atmosUpdate = load("services/AtmosUpdate.js");
const richUi = load("services/RichUi.js");
const groups = load("services/SnapshotGroups.js");
const adapters = {
  clampLook: hypr.clampLook,
  clampInput: hypr.clampInput,
  applyAccountPatch: accounts.applyAccountPatch,
  normalizeHardware: hardware.normalize,
  normalizeDiagnostics: diagnostics.normalize,
  parseTime: sunset.parseTime,
  parseChannel: atmosUpdate.parseChannel,
  parseWeatherCoords: richUi.parseWeatherCoords,
  allowedKey: groups.allowedKey,
};

const fresh = snapshot.adopt({}, { theme: "tokyo", extraThemes: ["a"] }, adapters);
assertEqual(fresh.theme, "tokyo", "adopt sets theme from an empty record");
assertEqual(fresh.extraThemes.join(","), "a", "adopt sets extraThemes from an empty record");
assert(!("hardware" in fresh), "adopt from {} leaves hardware absent");

const withDiag = snapshot.adopt(
  {},
  { diagnostics: { hyprland: { version: "0.56.2", configErrors: [] } } },
  adapters,
);
assertEqual(withDiag.diagnostics.hyprland.version, "0.56.2", "adopt normalizes diagnostics");

const full = {
  hardware: { cpu: { model: "X" } },
  theme: "omarchy",
  hostname: "old",
  users: [{ name: "a" }],
};
const lookParsed = { theme: "x" };
const next = snapshot.adopt(full, lookParsed, adapters);
assert(next.hardware === full.hardware, "look/theme patch keeps hardware reference");
assert(next.users === full.users, "look/theme patch keeps users reference");
assertEqual(next.theme, "x", "look/theme patch updates theme");
assertEqual(next.hostname, "old", "merged look record still carries hostname from current");
assertEqual(
  snapshot.accountStorePatch(lookParsed),
  null,
  "look patch does not build an AccountsStore patch",
);
assertEqual(
  snapshot.accountStorePatch(next).hostname,
  "old",
  "merged record would wrongly patch stale hostname if used as the store src",
);
const hostPatch = snapshot.accountStorePatch({ hostname: "new", theme: "x" });
assertEqual(hostPatch.hostname, "new", "accountStorePatch keeps hostname from parsed");
assert(!("theme" in hostPatch), "accountStorePatch drops non-account keys");
assert(!("users" in hostPatch), "accountStorePatch omits unpatched users");

const sameHw = { cpu: { model: "X" } };
assert(
  snapshot.adoptValue(sameHw, sameHw) === sameHw,
  "adoptValue keeps the same object reference",
);
const sameUsers = [{ name: "a" }];
assert(
  snapshot.adoptArray(sameUsers, sameUsers) === sameUsers,
  "adoptArray keeps the same array reference",
);
const twinHw = { cpu: { model: "X" } };
assert(
  snapshot.adoptValue(sameHw, twinHw) === sameHw,
  "adoptValue keeps current when JSON-equal objects differ by reference",
);

assertEqual(
  snapshot.adopt({ barVisible: true }, { barVisible: false }, adapters).barVisible,
  false,
  "adopt keeps false",
);
assertEqual(
  snapshot.adopt({}, { clockWeekStart: "fun" }, adapters).clockWeekStart,
  "",
  "adopt drops an unknown week start",
);
assertEqual(
  snapshot.adopt({}, { agentsRefreshIntervalSec: 15 }, adapters).agentsRefreshIntervalSec,
  30,
  "adopt clamps agents refresh below 30s to 30, matching snapshot.sh",
);
assertEqual(
  snapshot.adopt({}, { agentsRefreshIntervalSec: 900 }, adapters).agentsRefreshIntervalSec,
  900,
  "adopt keeps a valid agents refresh",
);
assertEqual(
  snapshot.adopt({}, { nightlightTemperature: 0 }, adapters).nightlightTemperature,
  0,
  "adopt keeps nightlight 0 as unset",
);
assertEqual(
  snapshot.adopt({}, { nightlightTemperature: 1000 }, adapters).nightlightTemperature,
  3000,
  "adopt clamps nightlight warmth up to the writer floor",
);
assertEqual(
  snapshot.adopt({}, { nightlightTemperature: 9000 }, adapters).nightlightTemperature,
  6500,
  "adopt clamps nightlight warmth down to the writer ceiling",
);

const look = snapshot.adopt({}, { hyprLook: { gapsIn: 80, layout: "niri" } }, adapters);
assertEqual(look.hyprLook.gapsIn, 64, "adopt clamps hyprLook gapsIn via clampLook");
assertEqual(look.hyprLook.layout, "dwindle", "adopt clamps hyprLook layout via clampLook");

let missingLook = false;
try {
  snapshot.adopt(
    {},
    { hyprLook: { gapsIn: 80 } },
    { clampInput: hypr.clampInput, allowedKey: groups.allowedKey },
  );
} catch {
  missingLook = true;
}
assert(missingLook, "adopt throws when clampLook is missing");

let missingInput = false;
try {
  snapshot.adopt(
    {},
    { hyprInput: { sensitivity: 0 } },
    { clampLook: hypr.clampLook, allowedKey: groups.allowedKey },
  );
} catch {
  missingInput = true;
}
assert(missingInput, "adopt throws when clampInput is missing");

let missingAllowed = false;
try {
  snapshot.adopt({}, { theme: "x" }, { clampLook: hypr.clampLook, clampInput: hypr.clampInput });
} catch {
  missingAllowed = true;
}
assert(missingAllowed, "adopt throws when allowedKey is missing");

let missingAdapters = false;
try {
  snapshot.adopt({}, { theme: "x" });
} catch {
  missingAdapters = true;
}
assert(missingAdapters, "adopt throws when adapters are missing");

const taggedCurrent = { hardware: { cpu: { model: "X" } } };
const tagged = snapshot.adopt(
  taggedCurrent,
  { group: "look", theme: "x", hardware: { cpu: { model: "nope" } } },
  adapters,
);
assertEqual(tagged.theme, "x", "tagged look still merges theme");
assertEqual(tagged.hardware.cpu.model, "X", "tagged look does not take hardware (allowedKey)");
assert(tagged.hardware === taggedCurrent.hardware, "tagged look keeps hardware reference");
assert(!("group" in tagged), "adopt never copies group onto the record");

const untagged = snapshot.adopt(
  { hardware: { cpu: { model: "X" } } },
  { hardware: { cpu: { model: "yep" } } },
  adapters,
);
assertEqual(untagged.hardware.cpu.model, "yep", "untagged hardware still takes it");

assertEqual(snapshot.parseSnapshot("{"), null, "parseSnapshot rejects junk after adopt");
assertEqual(snapshot.patchGroup({ group: "look" }), "look", "patchGroup accepts look");
assertEqual(snapshot.patchGroup({ group: "nope" }), "", "patchGroup rejects an unknown group");
assertEqual(snapshot.sanitizeDmi("to be filled by o.e.m."), "", "sanitizeDmi drops DMI filler");
