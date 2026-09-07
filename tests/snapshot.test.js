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

const patched = snapshot.patchMonitorBrightness(
  [
    { name: "DP-1", brightness: 40 },
    { name: "HDMI-A-1", brightness: 10 },
  ],
  "DP-1",
  80,
);
assertEqual(patched[0].brightness, 80, "patchMonitorBrightness updates the named monitor");
assertEqual(patched[1].brightness, 10, "patchMonitorBrightness leaves other monitors");

const scaled = snapshot.patchFocusedMonitorScale(
  [
    { name: "DP-1", focused: false, scale: 1 },
    { name: "eDP-1", focused: true, scale: 1 },
  ],
  1.25,
);
assertEqual(scaled[1].scale, 1.25, "patchFocusedMonitorScale updates the focused monitor");
assertEqual(scaled[0].scale, 1, "patchFocusedMonitorScale leaves other monitors");

const plugged = snapshot.patchPluginEnabled(
  [
    { id: "omarchy.clock", enabled: true },
    { id: "omarchy.weather", enabled: false },
  ],
  "omarchy.weather",
  true,
);
assertEqual(plugged[1].enabled, true, "patchPluginEnabled enables the named plugin");
assertEqual(plugged[0].enabled, true, "patchPluginEnabled leaves other plugins");

const extras = snapshot.patchRemoveMatching(["tokyo", "omarchy", "catppuccin"], "", "omarchy");
assertEqual(extras.join(","), "tokyo,catppuccin", "patchRemoveMatching drops a string extra theme");
const apps = snapshot.patchRemoveMatching(
  [
    { id: "foo", name: "Foo" },
    { id: "bar", name: "Bar" },
  ],
  "id",
  "foo",
);
assertEqual(apps.length, 1, "patchRemoveMatching drops a launcher by id");
assertEqual(apps[0].id, "bar", "patchRemoveMatching keeps the other launcher");

const btPatched = snapshot.patchRowField(
  [
    { address: "AA:BB", connected: false },
    { address: "CC:DD", connected: true },
  ],
  "address",
  "AA:BB",
  "connected",
  true,
);
assertEqual(btPatched[0].connected, true, "patchRowField updates the matched bluetooth row");
assertEqual(btPatched[1].connected, true, "patchRowField leaves other bluetooth rows");

const wifiPatched = snapshot.patchWifiActive(
  [
    { uuid: "11111111-1111-1111-1111-111111111111", name: "Home", active: true },
    { uuid: "22222222-2222-2222-2222-222222222222", name: "Cafe", active: false },
  ],
  "22222222-2222-2222-2222-222222222222",
  true,
);
assertEqual(wifiPatched[1].active, true, "patchWifiActive marks the named connection active");
assertEqual(wifiPatched[0].active, false, "patchWifiActive clears the previous active connection");

const hooked = snapshot.patchHookSample(
  [{ path: "/tmp/hooks/theme-set.d/notify.sample", name: "notify.sample", sample: true }],
  "/tmp/hooks/theme-set.d/notify.sample",
  true,
);
assertEqual(hooked[0].sample, false, "patchHookSample enables a sample hook");
assertEqual(hooked[0].name, "notify", "patchHookSample drops .sample from the name");

assertEqual(
  snapshot.patchKeyboardBrightness(40, "up"),
  50,
  "patchKeyboardBrightness steps up by 10",
);
assertEqual(
  snapshot.patchKeyboardBrightness(40, "down"),
  30,
  "patchKeyboardBrightness steps down by 10",
);
assertEqual(snapshot.patchKeyboardBrightness(40, "off"), 0, "patchKeyboardBrightness off is 0");
assertEqual(snapshot.patchKeyboardBrightness(96, "up"), 100, "patchKeyboardBrightness caps at 100");
assertEqual(snapshot.patchKeyboardBrightness(4, "down"), 0, "patchKeyboardBrightness floors at 0");
assertEqual(snapshot.patchKeyboardBrightness(0, "down"), 0, "patchKeyboardBrightness stays off");

const reminded = snapshot.patchAppendReminder([{ label: "Stand", minutes: 10 }], 5, "Tea");
assertEqual(reminded.length, 2, "patchAppendReminder appends a reminder");
assertEqual(reminded[1].message, "Tea", "patchAppendReminder keeps the message");
assertEqual(reminded[1].minutes, 5, "patchAppendReminder keeps the delay");
assertEqual(
  snapshot.patchAppendReminder([], 0, "nope").length,
  0,
  "patchAppendReminder rejects a zero delay",
);

const hookedAppend = snapshot.patchAppendHook(
  [{ path: "/tmp/hooks/theme-set.d/old.sh", name: "old.sh", type: "theme-set" }],
  {
    path: "/tmp/hooks/theme-set.d/notify.sh",
    name: "notify.sh",
    type: "theme-set",
    sample: false,
    flat: false,
  },
);
assertEqual(hookedAppend.length, 2, "patchAppendHook appends a new hook");
assertEqual(hookedAppend[1].name, "notify.sh", "patchAppendHook keeps the new hook name");
const upserted = snapshot.patchAppendHook(hookedAppend, {
  path: "/tmp/hooks/theme-set.d/old.sh",
  name: "old.sh",
  type: "theme-set",
  sample: true,
});
assertEqual(upserted.length, 2, "patchAppendHook upserts an existing hook path");
assertEqual(upserted[0].sample, true, "patchAppendHook updates the matched hook");

const replacedAuto = snapshot.patchReplaceManaged(
  [
    { command: "waybar", managed: false },
    { command: "mako", managed: true },
  ],
  ["hyprsunset"],
);
assertEqual(replacedAuto.length, 2, "patchReplaceManaged keeps unmanaged autostart rows");
assertEqual(replacedAuto[0].command, "waybar", "patchReplaceManaged keeps the unmanaged command");
assertEqual(replacedAuto[0].managed, false, "patchReplaceManaged leaves unmanaged rows unmanaged");
assertEqual(replacedAuto[1].command, "hyprsunset", "patchReplaceManaged appends managed commands");
assertEqual(replacedAuto[1].managed, true, "patchReplaceManaged marks replacement rows managed");
const replacedBinds = snapshot.patchReplaceManaged(
  [
    { keys: "SUPER + D", command: "desks", managed: false },
    { keys: "SUPER + F", command: "nautilus", managed: true },
  ],
  [{ keys: "SUPER + Q", command: "kill", unbind: true }],
);
assertEqual(replacedBinds.length, 2, "patchReplaceManaged keeps unmanaged bindings");
assertEqual(replacedBinds[0].keys, "SUPER + D", "patchReplaceManaged keeps the unmanaged chord");
assertEqual(replacedBinds[1].keys, "SUPER + Q", "patchReplaceManaged appends the managed bind");
assertEqual(replacedBinds[1].unbind, true, "patchReplaceManaged keeps bind fields");
assertEqual(replacedBinds[1].managed, true, "patchReplaceManaged marks replacement binds managed");

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
