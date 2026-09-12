const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const systemd = load("services/Systemd.js");
const nav = load("services/NavStatus.js");

function ready(extra) {
  const state = { ready: true, isFailed: systemd.isFailed };
  if (extra && typeof extra === "object") Object.assign(state, extra);
  return state;
}

function expectNull(id, state, description) {
  assertEqual(nav.forHub(id, state), null, description);
}

function expectBadge(id, state, text, tone, title, description) {
  const badge = nav.forHub(id, state);
  assert(badge && typeof badge === "object", description);
  assertEqual(badge.text, text, description + " text");
  assertEqual(badge.tone, tone, description + " tone");
  if (title !== undefined) assertEqual(badge.title, title, description + " title");
}

expectNull("services", null, "null state is silent");
expectNull("services", undefined, "undefined state is silent");
expectNull("services", "nope", "string state is silent");
expectNull("services", 0, "number state is silent");
expectNull("services", [], "array state is silent");
expectNull("services", {}, "missing ready is silent");
expectNull(
  "services",
  { ready: false, systemdUnits: [{ active: "failed" }] },
  "ready false is silent",
);
expectNull(
  "services",
  { ready: "true", systemdUnits: [{ active: "failed" }] },
  "ready string true is silent",
);
expectNull(
  "network",
  { ready: true, netKind: "disconnected" },
  "ready disconnected network is silent",
);
expectNull("network", { netKind: "wifi" }, "wifi before ready is silent");
expectNull("display", { monitors: [{ enabled: false }] }, "disabled output before ready is silent");

expectNull("constructor", ready(), "constructor hub id is silent");
expectNull("__proto__", ready(), "__proto__ hub id is silent");
expectNull("toString", ready(), "toString hub id is silent");
expectNull("", ready(), "empty hub id is silent");
expectNull(null, ready({ systemdUnits: [{ active: "failed" }] }), "null hub id is silent");

expectNull("services", ready({ systemdUnits: null }), "null systemdUnits is silent");
expectNull("services", ready({ systemdUnits: "failed" }), "string systemdUnits is silent");
expectNull(
  "services",
  ready({ systemdUnits: { active: "failed" } }),
  "object systemdUnits is silent",
);
expectNull(
  "services",
  ready({ systemdUnits: [null, undefined, 0, "x"] }),
  "junk unit rows are silent",
);
expectNull(
  "services",
  ready({ systemdUnits: [{ active: "active", sub: "running" }] }),
  "healthy units are silent",
);

expectBadge(
  "services",
  ready({ systemdUnits: [{ active: "failed", sub: "failed" }] }),
  "▲1",
  "warn",
  "1 failed unit",
  "one failed unit",
);
expectBadge(
  "services",
  ready({
    systemdUnits: [
      null,
      { active: "active", sub: "running" },
      { active: "failed" },
      { active: "inactive", sub: "failed" },
    ],
  }),
  "▲2",
  "warn",
  "2 failed units",
  "failed units skip nulls and count active or sub",
);
assert(systemd.isFailed({ active: "failed" }) === true, "Systemd.isFailed sees active=failed");
assert(
  nav.isFailedUnit({ active: "failed" }, ready()) === true,
  "NavStatus delegates failed units to Systemd.isFailed",
);
assert(
  nav.isFailedUnit({ active: "failed" }, { ready: true }) === false,
  "NavStatus does not invent a failed-unit check without Systemd.isFailed",
);

expectNull("display", ready({ monitors: null }), "null monitors are silent");
expectNull("display", ready({ monitors: {} }), "object monitors are silent");
expectNull("display", ready({ monitors: [] }), "empty monitors are silent");
expectNull(
  "display",
  ready({ monitors: [{ name: "eDP-1", enabled: true }] }),
  "one enabled output is silent",
);
expectNull(
  "display",
  ready({
    monitors: [
      { name: "eDP-1", enabled: true },
      { name: "HDMI-A-1", enabled: true },
    ],
  }),
  "all-enabled multi-monitor is silent",
);
expectNull(
  "display",
  ready({ monitors: [{ name: "eDP-1", disabled: true }] }),
  "monitor-rule disabled field is not a snapshot enabled flag",
);

expectBadge(
  "display",
  ready({
    monitors: [
      { name: "eDP-1", enabled: true },
      { name: "HDMI-A-1", enabled: false },
    ],
  }),
  "1/2",
  "warn",
  "1 output disabled",
  "disabled snapshot output uses enabled === false",
);
expectBadge(
  "display",
  ready({
    monitors: [
      null,
      { name: "eDP-1", enabled: false },
      { name: "DP-1", enabled: false },
      { name: "HDMI-A-1", enabled: true },
    ],
  }),
  "1/3",
  "warn",
  "2 outputs disabled",
  "display count skips null rows and still uses enabled",
);

expectNull("network", ready({ netKind: null }), "null netKind is silent");
expectNull("network", ready({ netKind: "" }), "empty netKind is silent");
expectNull("network", ready({ netKind: "disconnected" }), "disconnected is silent");
expectNull("network", ready({ netKind: "DISCONNECTED" }), "uppercase disconnected is silent");
expectNull("network", ready({ bluetooth: false }), "bluetooth default is not network evidence");
expectBadge(
  "network",
  ready({ netKind: "ethernet" }),
  "eth",
  "info",
  "Wired",
  "ethernet is positive evidence",
);
expectBadge(
  "network",
  ready({ netKind: "Ethernet" }),
  "eth",
  "info",
  "Wired",
  "ethernet kind is case-insensitive",
);
expectBadge(
  "network",
  ready({ netKind: "wifi", netSsid: "home" }),
  "●",
  "info",
  "Connected to home",
  "wifi is positive evidence",
);
expectBadge(
  "network",
  ready({ netKind: "wifi", netSsid: "" }),
  "●",
  "info",
  "Connected",
  "wifi without ssid still counts as connected",
);

expectNull("bluetooth", ready({ bluetoothDevices: null }), "null bluetooth devices are silent");
expectNull(
  "bluetooth",
  ready({ bluetoothDevices: [{ name: "buds" }] }),
  "unconnected device is silent",
);
expectNull(
  "bluetooth",
  ready({ bluetoothDevices: [{ connected: false }] }),
  "connected false is not evidence",
);
expectBadge(
  "bluetooth",
  ready({
    bluetoothDevices: [null, { connected: true }, { connected: true, name: "keyboard" }],
  }),
  "●2",
  "info",
  "2 connected",
  "connected true is positive evidence",
);

expectNull(
  "software",
  ready({ updateAvailable: true, atmosUpdateAvailable: true }),
  "updates do not hang on Software",
);
expectNull(
  "system",
  ready({ updateAvailable: false, atmosUpdateAvailable: false }),
  "no updates is silent",
);
expectBadge(
  "system",
  ready({ updateAvailable: true }),
  "↑1",
  "info",
  "Omarchy update available",
  "Omarchy update hangs on System",
);
expectBadge(
  "system",
  ready({ atmosUpdateAvailable: true }),
  "↑1",
  "info",
  "Atmos update available",
  "Atmos update hangs on System",
);
expectBadge(
  "system",
  ready({ updateAvailable: true, atmosUpdateAvailable: true }),
  "↑2",
  "info",
  "Omarchy and Atmos updates available",
  "both updates name the sources",
);

expectNull(
  "disks",
  ready({
    disks: [
      {
        path: "/dev/nvme0n1",
        name: "nvme0n1",
        model: "SSD",
        tran: "nvme",
        size: 512,
        info: "/dev/nvme0n1",
        mounts: [{ target: "/", source: "/dev/nvme0n1p2" }],
        smart: "FAILING",
        health: "FAILING",
      },
    ],
  }),
  "disks stay silent even when a fabricated smart field is present",
);

const themeQml = fs.readFileSync(path.join(__dirname, "..", "services", "Theme.qml"), "utf8");
assert(themeQml.indexOf("#6f8f6f") === -1, "Theme.qml has no hardcoded ok green");
assert(themeQml.indexOf("#b5904f") === -1, "Theme.qml has no hardcoded warn amber");
assert(themeQml.indexOf("property color ok:") === -1, "Theme.qml does not add a private ok ink");
assert(
  themeQml.indexOf("property color warn:") === -1,
  "Theme.qml does not add a private warn ink",
);
assert(themeQml.indexOf("readonly property int badgeSize:") !== -1, "Theme names badgeSize");
assert(themeQml.indexOf("urgent") !== -1, "Theme still carries urgent from colors.toml");

const shellSrc = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");
assert(shellSrc.indexOf("NavStatus.js") !== -1, "shell binds NavStatus.js");
assert(shellSrc.indexOf("id: navBadge") !== -1, "sidebar rows have a navBadge");
assert(shellSrc.indexOf("Theme.urgent") !== -1, "warn badges use Theme.urgent");
assert(shellSrc.indexOf("Theme.ok") === -1, "shell does not read a private Theme.ok");
assert(shellSrc.indexOf("Theme.warn") === -1, "shell does not read a private Theme.warn");
assert(shellSrc.indexOf("Theme.sidebarWidth") !== -1, "sidebar width token is unchanged");

const navStatusSrc = fs.readFileSync(
  path.join(__dirname, "..", "services", "NavStatus.js"),
  "utf8",
);
assert(
  navStatusSrc.indexOf("state.isFailed") !== -1,
  "NavStatus calls the Systemd.isFailed the caller passed",
);
assert(
  shellSrc.indexOf("SystemdJs.isFailed") !== -1,
  "shell passes Systemd.isFailed into navState",
);
assert(navStatusSrc.indexOf("enabled === false") !== -1, "Displays reads enabled, not disabled");
assert(navStatusSrc.indexOf('case "disks"') === -1, "Disks is not a badged hub");
assert(navStatusSrc.indexOf('case "software"') === -1, "Software is not a badged hub");
