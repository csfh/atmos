const { load, assertEqual } = require("./harness");

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
