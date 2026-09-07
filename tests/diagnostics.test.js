const { load, assert, assertEqual } = require("./harness");

const diag = load("services/Diagnostics.js");

assertEqual(diag.failedCount({}), 0, "failedCount empty");
assertEqual(diag.hyprOk({}), true, "hyprOk with no errors");
assertEqual(diag.statusLabel(true), "OK", "statusLabel ok");
assertEqual(diag.statusLabel(false), "Needs attention", "statusLabel fail");

const empty = diag.normalize(null);
assertEqual(empty.hyprland.version, "", "normalize null hypr version");
assertEqual(empty.failedUnits.length, 0, "normalize null failed units");
assertEqual(empty.disk.path, "/", "normalize null disk path");

const dirty = diag.normalize({
  hostname: " box\nname ",
  kernel: { sysname: "Linux", release: "7.1.9-arch1-2", machine: "x86_64" },
  omarchy: { version: "4.0.2-1", channel: "stable", path: "/usr/share/omarchy" },
  atmos: {
    revision: "abc123",
    installed: true,
    hyprAtmos: true,
    hyprAtmosLayout: true,
    sentinels: { look: true, input: true, autostart: false, bindings: false, windows: true },
  },
  hyprland: { version: "0.56.2", configErrors: ["bad bind", "password=secret"] },
  failedUnits: [
    { unit: "cups.service", scope: "system", sub: "failed", description: "CUPS" },
    { unit: "evil;rm", scope: "system", sub: "failed", description: "nope" },
    { unit: "app-foo.service", scope: "user", sub: "failed", description: "App" },
  ],
  disk: { path: "/", total: 1000, used: 950, available: 50, percent: 95 },
  memory: { total: 100, used: 40, available: 60, swapTotal: 10, swapUsed: 0 },
  gpu: { driver: "amdgpu", identity: "Radeon" },
  portals: {
    "xdg-desktop-portal": "active",
    "xdg-desktop-portal-hyprland": "failed",
    "xdg-desktop-portal-gtk": "active",
  },
  pipewire: { pipewire: "active", "pipewire-pulse": "active", wireplumber: "active" },
  network: { online: true, kind: "wifi" },
  pacman: { syncOk: true, lastSync: "2026-08-25 07:05", dbCount: 4 },
  recentErrors: ["nl80211: kernel reports multicast", "user : a password is required", "ok line"],
});

assertEqual(dirty.hostname, "", "normalize drops a hostname with a newline");
assertEqual(dirty.kernel.release, "7.1.9-arch1-2", "normalize keeps kernel release");
assertEqual(dirty.hyprland.configErrors.length, 1, "normalize drops secret config errors");
assertEqual(dirty.hyprland.configErrors[0], "bad bind", "normalize keeps a plain config error");
assertEqual(dirty.failedUnits.length, 2, "normalize drops an unsafe unit name");
assertEqual(dirty.failedUnits[1].scope, "user", "normalize keeps a user unit");
assertEqual(dirty.recentErrors.length, 2, "normalize drops a password journal line");
assertEqual(diag.failedCount(dirty), 2, "failedCount after normalize");
assertEqual(diag.hyprOk(dirty), false, "hyprOk false when errors remain");
assertEqual(diag.diskPressure(dirty), true, "diskPressure at 95%");
assertEqual(diag.memoryPressure(dirty), false, "memoryPressure under 95%");
assertEqual(diag.atmosOk(dirty), true, "atmosOk when hypr.atmos is required");
assertEqual(diag.servicesOk(dirty.portals), false, "portals not ok when one failed");
assertEqual(diag.servicesOk(dirty.pipewire), true, "pipewire ok when all active");
assert(diag.omarchySummary(dirty).indexOf("4.0.2-1") !== -1, "omarchySummary names the version");
assert(diag.atmosSummary(dirty).indexOf("abc123") !== -1, "atmosSummary names the revision");
assertEqual(diag.networkSummary(dirty), "Online on Wi-Fi.", "networkSummary wifi");
assert(
  diag.pacmanSummary(dirty).indexOf("4 sync") !== -1,
  "pacmanSummary names the database count",
);
assertEqual(diag.gpuSummary(dirty), "Radeon (amdgpu)", "gpuSummary identity and driver");
assertEqual(
  diag.reportFileName("my-box", new Date(2026, 8, 6, 15, 4)),
  "atmos-diagnostics-my-box-20260906-1504.txt",
  "reportFileName stamps the host and time",
);
assertEqual(
  diag.reportFileName("bad/name", new Date(2026, 0, 1, 0, 0)),
  "atmos-diagnostics-badname-20260101-0000.txt",
  "reportFileName strips a slash from the host",
);

const report = diag.reportText(dirty);
assert(report.indexOf("Atmos diagnostic report") === 0, "reportText starts with the title");
assert(report.indexOf("== Omarchy ==") !== -1, "reportText has Omarchy");
assert(report.indexOf("== Atmos ==") !== -1, "reportText has Atmos");
assert(report.indexOf("== Hyprland ==") !== -1, "reportText has Hyprland");
assert(report.indexOf("== Failed systemd units ==") !== -1, "reportText has failed units");
assert(report.indexOf("cups.service") !== -1, "reportText lists cups");
assert(report.indexOf("evil;rm") === -1, "reportText omits an unsafe unit");
assert(report.indexOf("== Disk ==") !== -1, "reportText has Disk");
assert(report.indexOf("== Memory ==") !== -1, "reportText has Memory");
assert(report.indexOf("== Kernel ==") !== -1, "reportText has Kernel");
assert(report.indexOf("Linux 7.1.9-arch1-2") !== -1, "reportText names the kernel");
assert(report.indexOf("== GPU ==") !== -1, "reportText has GPU");
assert(report.indexOf("== Portals ==") !== -1, "reportText has Portals");
assert(
  report.indexOf("xdg-desktop-portal-hyprland failed") !== -1,
  "reportText names a failed portal",
);
assert(report.indexOf("== PipeWire ==") !== -1, "reportText has PipeWire");
assert(report.indexOf("== Network ==") !== -1, "reportText has Network");
assert(report.indexOf("== Package database ==") !== -1, "reportText has pacman");
assert(report.indexOf("== Recent errors ==") !== -1, "reportText has recent errors");
assert(report.indexOf("password") === -1, "reportText omits password lines");
assert(report.indexOf("\0") === -1, "reportText has no NUL");
assert(report.charAt(report.length - 1) === "\n", "reportText ends with a newline");

const blank = diag.reportText({});
assert(blank.indexOf("Atmos diagnostic report") === 0, "empty report still has a title");
assert(blank.indexOf("None.") !== -1, "empty report says none for empty lists");
assert(blank.indexOf("== Hyprland ==") !== -1, "empty report keeps Hyprland");
assertEqual(diag.atmosOk({}), false, "atmosOk false without hypr.atmos");
assertEqual(diag.networkSummary({}), "No default route.", "networkSummary offline default");
