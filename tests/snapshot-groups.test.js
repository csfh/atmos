const { load, assert, assertEqual } = require("./harness");

const groups = load("services/SnapshotGroups.js");
const hubs = load("services/Hubs.js");
assertEqual(groups.snapshotGroupForHub("appearance"), "all", "unwired hubs read all");
groups.setSnapshotGroupForHub(hubs.snapshotGroupForHub);

assertEqual(groups.snapshotGroupForHub("appearance"), "look", "appearance hub reads look first");
assertEqual(groups.snapshotGroupForHub("display"), "look", "displays hub reads look first");
assertEqual(groups.snapshotGroupForHub("windows"), "look", "windows hub reads look first");
assertEqual(groups.snapshotGroupForHub("bar"), "look", "bar hub reads look first");
assertEqual(
  groups.snapshotGroupForHub("notifications"),
  "look",
  "notifications hub reads look first",
);
assertEqual(groups.snapshotGroupForHub("idle"), "look", "idle hub reads look first");
assertEqual(groups.snapshotGroupForHub("appearance/boot"), "look", "child hubs use the root id");
assertEqual(groups.snapshotGroupForHub("network"), "network", "network hub reads network first");
assertEqual(groups.snapshotGroupForHub("disks"), "disks", "disks hub reads disks first");
assertEqual(
  groups.snapshotGroupForHub("accounts"),
  "accounts",
  "accounts hub reads accounts first",
);
assertEqual(groups.snapshotGroupForHub("system"), "system", "system hub reads system first");
assertEqual(groups.snapshotGroupForHub("hardware"), "all", "other hubs read the full snapshot");
assertEqual(groups.snapshotGroupForHub(""), "look", "empty hub reads look first");
groups.setSnapshotGroupForHub(function () {
  return "network";
});
assertEqual(
  groups.snapshotGroupForHub("appearance"),
  "network",
  "setSnapshotGroupForHub overrides",
);
groups.setSnapshotGroupForHub(hubs.snapshotGroupForHub);
assertEqual(
  groups.snapshotGroupForHub("appearance"),
  "look",
  "setSnapshotGroupForHub restores Hubs",
);

assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.config/hypr/looknfeel.lua"),
  "look",
  "looknfeel.lua watch is look",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.config/hypr/hyprsunset.conf"),
  "look",
  "hyprsunset.conf watch is look",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.config/hypr/input.lua"),
  "rest",
  "input.lua watch is rest",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.config/hypr/autostart.lua"),
  "rest",
  "autostart.lua watch is rest",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.config/hypr/bindings.lua"),
  "rest",
  "bindings.lua watch is rest",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.config/hypr/atmos.lua"),
  "rest",
  "atmos.lua watch is rest",
);
assertEqual(groups.snapshotGroupForWatchPath("/etc/hostname"), "rest", "hostname watch is rest");
assertEqual(groups.snapshotGroupForWatchPath("/home/x/.face.icon"), "rest", "face watch is rest");
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.local/state/omarchy/toggles"),
  "all",
  "toggles dir watch is all",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/home/x/.local/state/omarchy/toggles/hypr"),
  "look",
  "hypr toggles watch is look",
);

const lookKeys = groups.emitKeys("look");
assert(lookKeys.indexOf("theme") !== -1, "look emit includes theme");
assert(lookKeys.indexOf("hyprLook") !== -1, "look emit includes hyprLook");
assert(lookKeys.indexOf("hardware") === -1, "look emit does not include hardware");
assert(lookKeys.indexOf("disks") === -1, "look emit does not include disks");
assert(lookKeys.indexOf("group") === -1, "look emit does not include group");
assert(groups.emitKeys("accounts").indexOf("hostname") === -1, "accounts emit omits hostname");
assert(groups.emitKeys("system").indexOf("hostname") !== -1, "system emit includes hostname");
assert(groups.emitKeys("system").indexOf("diagnostics") !== -1, "system emit includes diagnostics");
assert(groups.emitKeys("rest").indexOf("hardware") !== -1, "rest emit still carries hardware");
assert(groups.emitKeys("rest").indexOf("diagnostics") !== -1, "rest emit includes diagnostics");
assert(
  groups.emitKeys("rest").indexOf("barPosition") !== -1,
  "rest emit still carries barPosition",
);
assert(groups.emitKeys("rest").indexOf("theme") === -1, "rest emit drops theme");
assert(groups.emitKeys("rest").indexOf("plymouth") === -1, "rest emit drops plymouth");

assertEqual(groups.allowedKey("look", "theme"), true, "look allows theme");
assertEqual(groups.allowedKey("look", "hardware"), false, "look rejects hardware");
assertEqual(groups.allowedKey("look", "disks"), false, "look rejects disks");
assertEqual(groups.allowedKey("all", "hardware"), true, "all allows hardware");
assertEqual(groups.allowedKey("all", "group"), false, "all rejects group");
assertEqual(groups.allowedKey("", "hardware"), true, "empty group allows hardware");
assertEqual(groups.allowedKey("accounts", "hostname"), false, "accounts rejects hostname");
assertEqual(groups.allowedKey("system", "hostname"), true, "system allows hostname");
assertEqual(groups.allowedKey("rest", "hardware"), true, "rest allows hardware");
assertEqual(groups.allowedKey("rest", "theme"), false, "rest rejects theme");
assertEqual(groups.normalizeGroup("look"), "look", "normalizeGroup keeps look");
assertEqual(groups.normalizeGroup("nope"), "all", "normalizeGroup maps invalid to all");
assertEqual(groups.normalizeGroup(""), "all", "normalizeGroup maps empty to all");
assertEqual(groups.tag({ theme: "x" }, "look").group, "look", "tag sets group");
assertEqual(groups.tag({ theme: "x" }, "look").theme, "x", "tag keeps payload keys");

assertEqual(groups.groupForKey("theme"), "look", "groupForKey maps theme to look");
assertEqual(groups.groupForKey("hyprLook.gapsIn"), "look", "groupForKey maps dotted look keys");
assertEqual(groups.groupForKey("hyprInput"), "rest", "groupForKey maps hyprInput to rest");
assertEqual(groups.groupForKey("hostname"), "system", "groupForKey maps hostname to system");
assertEqual(groups.groupForKey("fullName"), "accounts", "groupForKey maps fullName to accounts");
assertEqual(groups.groupForKey("dns"), "network", "groupForKey maps dns to network");
assertEqual(groups.groupForKey("unknown"), "", "groupForKey returns empty for unknown keys");
assertEqual(
  groups.tagApply({ hyprLook: { gapsIn: 4 }, hyprLookManaged: true }).group,
  "look",
  "tagApply tags a look payload",
);
assertEqual(
  groups.tagApply({ hyprInput: { sensitivity: 0 }, hyprInputManaged: true }).group,
  "rest",
  "tagApply tags an input payload rest",
);
assertEqual(
  groups.tagApply({ theme: "x", browser: "y" }).group,
  undefined,
  "tagApply leaves mixed groups untagged",
);
const bagKeys = groups.copyableBagKeys();
assert(bagKeys.indexOf("hyprLook") !== -1, "copyableBagKeys includes hyprLook");
assert(bagKeys.indexOf("hyprGapsIn") === -1, "copyableBagKeys does not flatten gapsIn");
assert(bagKeys.indexOf("hostname") === -1, "copyableBagKeys skips account keys");
assert(bagKeys.indexOf("audioSink") !== -1, "copyableBagKeys includes derived audioSink");

const paths = {
  userShellJson: "/u/shell.json",
  defaultShellJson: "/d/shell.json",
  userShellToml: "/u/shell.toml",
  weatherJson: "/u/weather.json",
  notificationsJson: "/u/notifications.json",
  currentBackgroundFile: "/u/omarchy/current/background",
  screensaverBrandFile: "/u/screensaver.txt",
  defaultScreensaverBrandFile: "/d/logo.txt",
  aboutBrandFile: "/u/about.txt",
  defaultAboutBrandFile: "/d/icon.txt",
  plymouthLogoFile: "/d/logo.png",
  defaultPlymouthLogoFile: "/d/default/logo.png",
  extraThemesDir: "/u/omarchy/themes",
  packagedThemesDir: "/usr/share/omarchy/themes",
  fontconfigFile: "/u/fonts.conf",
  indicatorsDir: "/u/omarchy/indicators",
  reminderDir: "/tmp/omarchy-reminders",
  looknfeelLuaFile: "/home/x/.config/hypr/looknfeel.lua",
  hyprsunsetConfFile: "/home/x/.config/hypr/hyprsunset.conf",
  monitorsLuaFile: "/u/monitors.lua",
  hyprTogglesDir: "/home/x/.local/state/omarchy/toggles/hypr",
  touchpadDisabledFile: "/u/touchpad-disabled-name",
  touchscreenDisabledFile: "/u/touchscreen-disabled-name",
  togglesDir: "/home/x/.local/state/omarchy/toggles",
  powerProfileAcFile: "/u/powerprofiles/ac",
  powerProfileBatteryFile: "/u/powerprofiles/battery",
  powerProfilesStateFile: "/var/lib/power-profiles-daemon/state.ini",
  applicationsDir: "/u/applications",
  defaultEditorFile: "/u/defaults/editor",
  defaultAgentFile: "/u/defaults/agent",
  defaultTerminalFile: "/u/xdg-terminals.list",
  defaultBrowserFile: "/u/mimeapps.list",
  dnsConfFile: "/etc/NetworkManager/conf.d/20-omarchy-dns.conf",
  bluetoothRfkillDir: "/var/lib/systemd/rfkill",
  networkManagerDevicesDir: "/run/NetworkManager/devices",
  inputLuaFile: "/home/x/.config/hypr/input.lua",
  autostartLuaFile: "/home/x/.config/hypr/autostart.lua",
  bindingsLuaFile: "/home/x/.config/hypr/bindings.lua",
  windowsLuaFile: "/home/x/.config/hypr/atmos.lua",
  envFile: "/home/x/.config/environment.d/10-atmos.conf",
  presentationFile: "/home/x/.local/state/omarchy/atmos-presentation.json",
  localtimeFile: "/etc/localtime",
  vconsoleFile: "/etc/vconsole.conf",
  localeConfFile: "/etc/locale.conf",
  pacmanConfFile: "/etc/pacman.conf",
  gtkSettingsFile: "/home/x/.config/gtk-4.0/settings.ini",
  swappinessFile: "/etc/sysctl.d/99-atmos-swappiness.conf",
};
const specs = groups.watchSpecs(paths);
assertEqual(specs.length, 47, "watchSpecs covers 47 path/group pairs for every window");
assertEqual(
  specs
    .map(function (row) {
      return row.group;
    })
    .join(","),
  "look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,look,all,rest,rest,rest,rest,rest,rest,rest,rest,rest,rest,rest,rest,rest,rest,rest,system,look,rest,rest,rest,rest,rest,rest,rest",
  "watchSpecs group order matches today's Omarchy array",
);
assertEqual(specs[0].path, "/u/shell.json", "watchSpecs uses the passed userShellJson path");
assertEqual(specs[12].path, "/usr/share/omarchy/themes", "watchSpecs keeps packagedThemesDir");
assert(
  specs.every(function (row) {
    return row.path !== "/u/omarchy/themes";
  }),
  "watchSpecs does not FileView extraThemesDir",
);
assertEqual(specs[22].group, "all", "togglesDir stays all");
assertEqual(specs[22].path, paths.togglesDir, "togglesDir path is the live map value");
assertEqual(specs[44].path, paths.envFile, "envFile is watched twice for system and rest");
assertEqual(specs[44].group, "rest", "the second envFile watch refreshes rest tweaks");
assertEqual(specs[45].path, paths.gtkSettingsFile, "gtk settings are watched for tweaks");
assertEqual(specs[45].group, "rest", "gtk settings refresh rest");
assertEqual(specs[46].path, paths.swappinessFile, "swappiness is watched for tweaks");
assertEqual(specs[46].group, "rest", "swappiness refreshes rest");
assertEqual(
  groups.snapshotGroupForWatchPath(paths.looknfeelLuaFile, specs),
  "look",
  "exact watchSpecs match is look for looknfeel.lua",
);
assertEqual(
  groups.snapshotGroupForWatchPath(paths.inputLuaFile, specs),
  "rest",
  "exact watchSpecs match is rest for input.lua",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/etc/hostname", specs),
  "rest",
  "hostname stays rest via the fallback",
);
assertEqual(
  groups.snapshotGroupForWatchPath("/custom", [{ path: "/custom", group: "network" }]),
  "network",
  "exact specs match wins over the basename heuristic",
);
