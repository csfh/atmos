// Snapshot group policy. QML and Node both eval this file.

var GROUPS = ["look", "rest", "all", "network", "disks", "accounts", "system"];

var LOOK_KEYS = Object.freeze([
  "theme",
  "background",
  "font",
  "textSize",
  "themes",
  "extraThemes",
  "fonts",
  "stayAwake",
  "nightlight",
  "nightlightTemperature",
  "screensaverBranded",
  "aboutBranded",
  "plymouth",
  "plymouthThemes",
  "nightlightDay",
  "nightlightNight",
  "nightlightNightOn",
  "barPosition",
  "barTransparent",
  "barVisible",
  "idleScreensaver",
  "idleLock",
  "screensaverEnabled",
  "suspendEnabled",
  "clockPresent",
  "clockFormat",
  "clockFormatAlt",
  "clockWeekStart",
  "clockBirthYear",
  "clockLifeExpectancy",
  "indicatorsPresent",
  "indicatorsAlwaysShow",
  "indicatorsItems",
  "agentsPresent",
  "agentsRefreshIntervalSec",
  "agentsSync",
  "agentsSyncDir",
  "agentsSyncFileName",
  "agentsSyncDeviceId",
  "spacerPresent",
  "spacerSize",
  "trayPresent",
  "trayHidden",
  "trayPinned",
  "hyprLook",
  "hyprLookManaged",
  "hyprNoGaps",
  "hyprSquareAspect",
  "hyprWorkspaceLayout",
  "workspaces",
  "workspacesManaged",
  "workspaceWrapSwitch",
  "workspaceWheelSwitch",
  "monitorRules",
  "monitorRulesManaged",
  "monitors",
  "internalPresent",
  "internalEnabled",
  "mirroring",
  "externalPresent",
  "touchpadPresent",
  "touchpadEnabled",
  "touchscreenPresent",
  "touchscreenEnabled",
  "keyboardBacklightPresent",
  "keyboardBrightness",
  "isLaptop",
  "batteryPresent",
  "weatherPresent",
  "doNotDisturb",
  "presentationMode",
  "reminderCount",
  "reminderActive",
  "reminders",
]);

var NETWORK_KEYS = Object.freeze([
  "dns",
  "bluetooth",
  "wifiConnected",
  "wifiBand",
  "wifiBandSelected",
  "wifiBands",
  "wifiIface",
  "netKind",
  "netIface",
  "netSsid",
  "netSignal",
  "netIp",
  "netGateway",
  "netDnsServers",
  "netSpeed",
  "wifiHw",
  "wifiRadio",
  "wifiConnections",
  "bluetoothDevices",
  "tailscaleInstalled",
  "tailscaleRunning",
  "tailscalePeers",
]);

var DISKS_KEYS = Object.freeze([
  "disks",
  "luksDevices",
  "swapDevices",
  "snapshots",
  "snapperPresent",
  "snapperConfigs",
  "hibernationAvailable",
  "hibernationSupported",
  "hibernationConfigured",
  "snapperNumberLimit",
  "snapperTimeline",
  "fstrimEnabled",
]);

var ACCOUNTS_KEYS = Object.freeze(["fullName", "currentUser", "avatarPath", "users", "groups"]);

var SYSTEM_KEYS = Object.freeze([
  "hostname",
  "timezone",
  "timezones",
  "ntp",
  "ntpAvailable",
  "ntpSynchronized",
  "locale",
  "locales",
  "parallelDownloads",
  "keyboardLayout",
  "keyboardLayouts",
  "crashCapture",
  "diagnostics",
  "envVars",
  "envPathPrepend",
  "envDetected",
]);

var ALL_KEYS = Object.freeze([
  "theme",
  "background",
  "font",
  "textSize",
  "themes",
  "extraThemes",
  "desktopApps",
  "tuiApps",
  "webApps",
  "fonts",
  "barPosition",
  "barTransparent",
  "clockFormat",
  "clockFormatAlt",
  "clockWeekStart",
  "clockPresent",
  "clockBirthYear",
  "clockLifeExpectancy",
  "browser",
  "terminal",
  "editor",
  "agent",
  "dns",
  "idleScreensaver",
  "idleLock",
  "stayAwake",
  "nightlight",
  "nightlightTemperature",
  "screensaverEnabled",
  "screensaverBranded",
  "aboutBranded",
  "barVisible",
  "bluetooth",
  "wifiConnected",
  "wifiBand",
  "wifiBandSelected",
  "wifiBands",
  "wifiIface",
  "netKind",
  "netIface",
  "netSsid",
  "netSignal",
  "netIp",
  "netGateway",
  "netDnsServers",
  "netSpeed",
  "wifiHw",
  "wifiRadio",
  "wifiConnections",
  "bluetoothDevices",
  "audioSinks",
  "audioSources",
  "audioOutputVolume",
  "audioOutputMuted",
  "audioInputVolume",
  "audioInputMuted",
  "audioTuningMatch",
  "audioTuningOn",
  "disks",
  "hardware",
  "diagnostics",
  "luksDevices",
  "swapDevices",
  "snapperPresent",
  "snapperConfigs",
  "snapshots",
  "hibernationAvailable",
  "hibernationSupported",
  "hibernationConfigured",
  "suspendEnabled",
  "crashCapture",
  "doNotDisturb",
  "weatherLocation",
  "weatherCoords",
  "weatherAuto",
  "weatherPresent",
  "weatherUnit",
  "weatherRefreshMinutes",
  "reminderCount",
  "reminderActive",
  "reminders",
  "indicatorsPresent",
  "indicatorsAlwaysShow",
  "indicatorsItems",
  "agentsPresent",
  "agentsRefreshIntervalSec",
  "agentsSync",
  "agentsSyncDir",
  "agentsSyncFileName",
  "agentsSyncDeviceId",
  "spacerPresent",
  "spacerSize",
  "trayPresent",
  "trayHidden",
  "trayPinned",
  "powerPresent",
  "powerShowPercentage",
  "isLaptop",
  "batteryPresent",
  "monitors",
  "internalPresent",
  "internalEnabled",
  "externalPresent",
  "mirroring",
  "touchpadPresent",
  "touchpadEnabled",
  "touchscreenPresent",
  "touchscreenEnabled",
  "keyboardBacklightPresent",
  "keyboardBrightness",
  "powerProfile",
  "powerProfileAc",
  "powerProfileBattery",
  "powerProfiles",
  "powerGovernor",
  "amdPstate",
  "chargeLimit",
  "chargeLimitAvailable",
  "presentationMode",
  "plymouth",
  "plymouthThemes",
  "hasAether",
  "browsers",
  "terminals",
  "editors",
  "hostname",
  "fullName",
  "currentUser",
  "avatarPath",
  "users",
  "groups",
  "timezone",
  "timezones",
  "ntp",
  "ntpAvailable",
  "ntpSynchronized",
  "locale",
  "locales",
  "parallelDownloads",
  "keyboardLayout",
  "keyboardLayouts",
  "hyprLook",
  "hyprInput",
  "hyprLookManaged",
  "hyprInputManaged",
  "hyprWorkspaceGesture",
  "hyprWorkspaceGestureManaged",
  "hyprWorkspaceGestureUnmanaged",
  "hyprNoGaps",
  "hyprSquareAspect",
  "hyprWorkspaceLayout",
  "workspaces",
  "workspacesManaged",
  "workspaceWrapSwitch",
  "workspaceWheelSwitch",
  "monitorRules",
  "monitorRulesManaged",
  "fingerprintAvailable",
  "fingerprintConfigured",
  "fido2Configured",
  "sshdEnabled",
  "sshdActive",
  "passwordlessSudo",
  "sudolessDocker",
  "omarchyVersion",
  "omarchyChannel",
  "updateAvailable",
  "updateSummary",
  "atmosRevision",
  "atmosChannel",
  "atmosInstalled",
  "voxtypeInstalled",
  "hybridGpuAvailable",
  "hybridGpuMode",
  "hwNvidia",
  "hwNvidiaGsp",
  "hwNvidiaWithoutGsp",
  "hwVulkan",
  "hwIntel",
  "hwIntelPtl",
  "hwWebcam",
  "hwFramework16",
  "hwAsusRog",
  "hwSurface",
  "dmiVendor",
  "dmiProduct",
  "dmiFamily",
  "cpuStat",
  "memoryStat",
  "cpuIdentity",
  "gpuIdentity",
  "npuIdentity",
  "tailscaleInstalled",
  "tailscaleRunning",
  "plugins",
  "snapperNumberLimit",
  "snapperTimeline",
  "fstrimEnabled",
  "directBootAvailable",
  "directBoot",
  "mimePdf",
  "mimeImage",
  "mimeVideo",
  "mimePdfOptions",
  "mimeImageOptions",
  "mimeVideoOptions",
  "picturesDir",
  "videosDir",
  "recordingActive",
  "webcamOverlay",
  "services",
  "gaming",
  "extras",
  "hooks",
  "autostart",
  "autostartManaged",
  "bindings",
  "bindingsManaged",
  "windowRules",
  "windowRulesManaged",
  "tweaks",
  "envVars",
  "envPathPrepend",
  "envDetected",
  "systemdUnits",
  "keybindings",
  "focusedClass",
  "cupsActive",
  "printerSetup",
  "nightlightDay",
  "nightlightNight",
  "nightlightNightOn",
  "tailscalePeers",
]);

var REST_DEL = {
  theme: true,
  background: true,
  font: true,
  textSize: true,
  themes: true,
  extraThemes: true,
  fonts: true,
  stayAwake: true,
  nightlight: true,
  nightlightTemperature: true,
  screensaverBranded: true,
  aboutBranded: true,
  plymouth: true,
  plymouthThemes: true,
  nightlightDay: true,
  nightlightNight: true,
  nightlightNightOn: true,
};

function keysWithout(src, drop) {
  var out = [];
  var i;
  for (i = 0; i < src.length; i++) {
    if (!drop[src[i]]) out.push(src[i]);
  }
  return Object.freeze(out);
}

var REST_KEYS = keysWithout(ALL_KEYS, REST_DEL);

var EMIT_KEYS = {
  look: LOOK_KEYS,
  network: NETWORK_KEYS,
  disks: DISKS_KEYS,
  accounts: ACCOUNTS_KEYS,
  system: SYSTEM_KEYS,
  rest: REST_KEYS,
  all: ALL_KEYS,
};

var WATCH_SPEC_KEYS = [
  ["userShellJson", "look"],
  ["defaultShellJson", "look"],
  ["userShellToml", "look"],
  ["weatherJson", "look"],
  ["notificationsJson", "look"],
  ["currentBackgroundFile", "look"],
  ["screensaverBrandFile", "look"],
  ["defaultScreensaverBrandFile", "look"],
  ["aboutBrandFile", "look"],
  ["defaultAboutBrandFile", "look"],
  ["plymouthLogoFile", "look"],
  ["defaultPlymouthLogoFile", "look"],
  ["packagedThemesDir", "look"],
  ["fontconfigFile", "look"],
  ["indicatorsDir", "look"],
  ["reminderDir", "look"],
  ["looknfeelLuaFile", "look"],
  ["hyprsunsetConfFile", "look"],
  ["monitorsLuaFile", "look"],
  ["hyprTogglesDir", "look"],
  ["touchpadDisabledFile", "look"],
  ["touchscreenDisabledFile", "look"],
  ["togglesDir", "all"],
  ["powerProfileAcFile", "rest"],
  ["powerProfileBatteryFile", "rest"],
  ["powerProfilesStateFile", "rest"],
  ["applicationsDir", "rest"],
  ["defaultEditorFile", "rest"],
  ["defaultAgentFile", "rest"],
  ["defaultTerminalFile", "rest"],
  ["defaultBrowserFile", "rest"],
  ["dnsConfFile", "rest"],
  ["bluetoothRfkillDir", "rest"],
  ["networkManagerDevicesDir", "rest"],
  ["inputLuaFile", "rest"],
  ["autostartLuaFile", "rest"],
  ["bindingsLuaFile", "rest"],
  ["windowsLuaFile", "rest"],
  ["envFile", "system"],
  ["presentationFile", "look"],
  ["localtimeFile", "rest"],
  ["vconsoleFile", "rest"],
  ["localeConfFile", "rest"],
  ["pacmanConfFile", "rest"],
];

function normalizeGroup(g) {
  var s = String(g || "");
  var i;
  for (i = 0; i < GROUPS.length; i++) {
    if (GROUPS[i] === s) return s;
  }
  return "all";
}

var snapshotGroupForHubImpl = null;

function setSnapshotGroupForHub(fn) {
  snapshotGroupForHubImpl = typeof fn === "function" ? fn : null;
}

function snapshotGroupForHub(hub) {
  if (snapshotGroupForHubImpl) return snapshotGroupForHubImpl(hub);
  // Unwired callers keep today's look-first policy until Hubs is installed.
  var id = String(hub || "");
  var slash = id.indexOf("/");
  if (slash !== -1) id = id.substring(0, slash);
  if (
    !id ||
    id === "appearance" ||
    id === "display" ||
    id === "windows" ||
    id === "bar" ||
    id === "notifications" ||
    id === "idle"
  )
    return "look";
  if (id === "network") return "network";
  if (id === "disks") return "disks";
  if (id === "accounts") return "accounts";
  if (id === "system") return "system";
  return "all";
}

function emitKeys(group) {
  var keys = EMIT_KEYS[String(group || "")];
  return keys || Object.freeze([]);
}

function allowedKey(group, key) {
  if (!group || group === "all") return key !== "group";
  var keys = emitKeys(group);
  return keys.indexOf(key) !== -1;
}

function watchSpecs(paths) {
  var src = paths && typeof paths === "object" ? paths : {};
  var out = [];
  var i;
  for (i = 0; i < WATCH_SPEC_KEYS.length; i++) {
    var spec = WATCH_SPEC_KEYS[i];
    out.push({ path: String(src[spec[0]] || ""), group: spec[1] });
  }
  return out;
}

function snapshotGroupForWatchPath(path, specs) {
  var p = String(path || "");
  var list = Array.isArray(specs) ? specs : [];
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i] && p && String(list[i].path || "") === p) return list[i].group;
  }
  var slash = p.lastIndexOf("/");
  var base = slash === -1 ? p : p.substring(slash + 1);
  if (
    base === "shell.json" ||
    base === "shell.toml" ||
    base === "looknfeel.lua" ||
    base === "hyprsunset.conf" ||
    base === "monitors.lua" ||
    base === "screensaver.txt" ||
    base === "about.txt" ||
    base === "logo.txt" ||
    base === "icon.txt" ||
    base === "logo.png" ||
    base === "fonts.conf" ||
    base === "weather.json" ||
    base === "notifications.json" ||
    p.indexOf("/omarchy/current/background") !== -1 ||
    p.indexOf("/omarchy/themes") !== -1 ||
    p.indexOf("/omarchy/themes/") !== -1 ||
    p.indexOf("/usr/share/omarchy/themes") !== -1 ||
    p.indexOf("/omarchy/indicators") !== -1 ||
    p.indexOf("/omarchy-reminders") !== -1 ||
    p.indexOf("/toggles/hypr") !== -1
  )
    return "look";
  if (p.indexOf("/omarchy/toggles") !== -1 && p.indexOf("/toggles/hypr") === -1) return "all";
  return "rest";
}

function tag(obj, group) {
  var out = {};
  var src = obj && typeof obj === "object" ? obj : {};
  var key;
  for (key in src) {
    if (!Object.prototype.hasOwnProperty.call(src, key)) continue;
    out[key] = src[key];
  }
  out.group = normalizeGroup(group);
  return out;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    GROUPS: GROUPS,
    normalizeGroup: normalizeGroup,
    snapshotGroupForHub: snapshotGroupForHub,
    setSnapshotGroupForHub: setSnapshotGroupForHub,
    emitKeys: emitKeys,
    allowedKey: allowedKey,
    watchSpecs: watchSpecs,
    snapshotGroupForWatchPath: snapshotGroupForWatchPath,
    tag: tag,
  };
}
