// The snapshot keys the window mirrors, and the topics it holds. QML and Node
// both eval this file.
//
// Which key is in which topic is the backend's (backend/src/snapshot/topics.rs,
// docs/adr/0011): the window holds every topic and takes what is pushed, so
// nothing here says where a key lives.

var TOPICS = Object.freeze([
  "settings.look",
  "settings.network",
  "settings.accounts",
  "settings.system",
  "settings.host",
]);

// What a caller that wants "this part read again" calls it. A group is only a
// request for some topics.
var GROUPS = ["look", "rest", "all", "network", "disks", "accounts", "system"];

// The five inventories are held too (docs/adr/0010); a Refresh button that
// said "everything" means them as well.
var INVENTORIES = ["hardware", "disks", "services", "software", "diagnostics"];

var TOPICS_OF = {
  look: ["settings.look"],
  network: ["settings.network"],
  accounts: ["settings.accounts"],
  system: ["settings.system"],
  disks: ["settings.host", "disks"],
  rest: ["settings.network", "settings.accounts", "settings.system", "settings.host"],
  all: TOPICS.concat(INVENTORIES),
};

function normalizeGroup(g) {
  var s = String(g || "");
  var i;
  for (i = 0; i < GROUPS.length; i++) {
    if (GROUPS[i] === s) return s;
  }
  return "all";
}

// The topics to ask the daemon to read again for some groups, each once.
function topicsForGroups(groups) {
  var out = [];
  var list = Array.isArray(groups) ? groups : [groups];
  var i, j, names;
  for (i = 0; i < list.length; i++) {
    names = TOPICS_OF[normalizeGroup(list[i])];
    for (j = 0; j < names.length; j++) if (out.indexOf(names[j]) === -1) out.push(names[j]);
  }
  return out;
}

// What the window has a property for.
var KEYS = Object.freeze([
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
  "workspaceBarNames",
  "workspaceBarCount",
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
  "fullName",
  "currentUser",
  "avatarPath",
  "users",
  "groups",
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
  "desktopApps",
  "tuiApps",
  "webApps",
  "browser",
  "terminal",
  "editor",
  "agent",
  "audioSinks",
  "audioSources",
  "audioOutputVolume",
  "audioOutputMuted",
  "audioInputVolume",
  "audioInputMuted",
  "audioTuningMatch",
  "audioTuningOn",
  "hardware",
  "weatherLocation",
  "weatherCoords",
  "weatherAuto",
  "weatherUnit",
  "weatherRefreshMinutes",
  "powerPresent",
  "powerShowPercentage",
  "powerProfile",
  "powerProfileAc",
  "powerProfileBattery",
  "powerProfiles",
  "powerGovernor",
  "amdPstate",
  "chargeLimit",
  "chargeLimitAvailable",
  "hasAether",
  "browsers",
  "terminals",
  "editors",
  "hyprInput",
  "hyprInputManaged",
  "hyprWorkspaceGesture",
  "hyprWorkspaceGestureManaged",
  "hyprWorkspaceGestureUnmanaged",
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
  "atmosPackaged",
  "atmosVersion",
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
  "plugins",
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
  "windowOpaque",
  "tweaks",
  "systemdUnits",
  "keybindings",
  "focusedClass",
  "cupsActive",
  "printerSetup",
]);

// Every key the machine reports is taken; the window only has a property for
// some. A topic's `_failed` is a note, not a setting.
function allowedKey(group, key) {
  return key !== "group" && key !== "_failed";
}

var ACCOUNT_BAG_KEYS = Object.freeze([
  "hostname",
  "fullName",
  "currentUser",
  "avatarPath",
  "users",
  "groups",
]);

// Adopted onto the Omarchy bag, not reported by the backend.
var DERIVED_BAG_KEYS = Object.freeze(["audioSink", "audioSource"]);

function copyableBagKeys() {
  var skip = {};
  var i;
  for (i = 0; i < ACCOUNT_BAG_KEYS.length; i++) skip[ACCOUNT_BAG_KEYS[i]] = true;
  var out = [];
  for (i = 0; i < KEYS.length; i++) {
    if (!skip[KEYS[i]]) out.push(KEYS[i]);
  }
  for (i = 0; i < DERIVED_BAG_KEYS.length; i++) out.push(DERIVED_BAG_KEYS[i]);
  return Object.freeze(out);
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    GROUPS: GROUPS,
    TOPICS: TOPICS,
    KEYS: KEYS,
    normalizeGroup: normalizeGroup,
    topicsForGroups: topicsForGroups,
    allowedKey: allowedKey,
    copyableBagKeys: copyableBagKeys,
    ACCOUNT_BAG_KEYS: ACCOUNT_BAG_KEYS,
    DERIVED_BAG_KEYS: DERIVED_BAG_KEYS,
  };
}
