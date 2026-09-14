const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const tweaks = load("services/Tweaks.js");
const cat = tweaks.catalog();
assert(cat.length > 0, "catalog is not empty");
assertEqual(tweaks.byId("missing"), null, "byId misses");
assertEqual(
  tweaks.byId("accelFlat").key,
  "hyprInput.accelProfile",
  "accelFlat uses the input writer",
);
assertEqual(tweaks.resetValue(tweaks.byId("accelFlat")), "", "resetValue for accel is empty");
assertEqual(tweaks.gtkIni({ middlePaste: false }), "", "gtkIni empty when off");
assert(
  tweaks.gtkIni({ middlePaste: true }).indexOf("gtk-enable-primary-paste=false") !== -1,
  "gtkIni writes the paste flag",
);
assertEqual(
  tweaks.environmentLines({ electronWayland: true }).length,
  0,
  "electron on writes nothing",
);
assertEqual(
  tweaks.environmentLines({ electronWayland: false })[0],
  "ELECTRON_OZONE_PLATFORM_HINT=auto",
  "electron off writes an overlay",
);
assertEqual(
  tweaks.environmentLines({ forceZeroScaling: false })[0],
  "ATMOS_XWAYLAND_ZERO_SCALING=0",
  "zero scaling off writes an overlay",
);
assertEqual(
  tweaks.environmentLines({ forceZeroScaling: true }).length,
  0,
  "zero scaling on writes nothing",
);
assert(
  tweaks.sysctlConf({ swappiness: true }).indexOf("vm.swappiness") !== -1,
  "sysctlConf writes swappiness",
);
assertEqual(tweaks.sysctlConf({ swappiness: false }), "", "sysctlConf empty when off");

const grouped = tweaks.groupedCatalog();
const groupNames = grouped.map(function (g) {
  return g.group;
});
const uniqueNames = groupNames.filter(function (name, i) {
  return groupNames.indexOf(name) === i;
});
assertEqual(
  groupNames.join("|"),
  uniqueNames.join("|"),
  "groupedCatalog has one section per group name",
);
const pointer = grouped.filter(function (g) {
  return g.group === "Pointer";
})[0];
assert(pointer && pointer.items.length >= 2, "Pointer tweaks share one section");
assertEqual(
  pointer.items.length,
  cat.filter(function (t) {
    return t.group === "Pointer";
  }).length,
  "Pointer section holds every Pointer catalog row",
);
assertEqual(
  grouped.reduce(function (n, g) {
    return n + g.items.length;
  }, 0),
  cat.length,
  "groupedCatalog keeps every catalog row",
);
assertEqual(tweaks.byId("swappiness"), null, "swappiness lives on Kernel, not Tweaks");

const tweaksPage = fs.readFileSync(path.join(__dirname, "..", "pages", "TweaksPage.qml"), "utf8");
assert(tweaksPage.indexOf("groupedCatalog") !== -1, "TweaksPage renders groupedCatalog");
assert(
  !/Repeater\s*\{\s*model:\s*root\.rows[\s\S]{0,200}PrefsGroup/.test(tweaksPage),
  "TweaksPage does not open a PrefsGroup per catalog row",
);

const kernelPage = fs.readFileSync(
  path.join(__dirname, "..", "pages", "system", "KernelPage.qml"),
  "utf8",
);
assert(kernelPage.indexOf('label: "Direct EFI boot"') !== -1, "Kernel page has Direct EFI boot");
assert(kernelPage.indexOf('label: "Lower swappiness"') !== -1, "Kernel page has swappiness");
const bootPage = fs.readFileSync(
  path.join(__dirname, "..", "pages", "appearance", "BootPage.qml"),
  "utf8",
);
assert(bootPage.indexOf("Direct EFI boot") === -1, "Boot screen no longer hosts Direct EFI boot");

const script = path.join(__dirname, "..", "scripts", "set-tweaks.sh");
assert(fs.existsSync(script), "set-tweaks.sh exists");

function runTweaks(action, mode, env) {
  return spawnSync("bash", [script, action, mode], {
    encoding: "utf8",
    env: Object.assign({}, process.env, env),
  });
}

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-tweaks-"));
const gtkFile = path.join(fixture, "settings.ini");
const envFile = path.join(fixture, "10-atmos.conf");
const sysctlFile = path.join(fixture, "99-atmos-swappiness.conf");

let result = runTweaks("gtk-middle-paste", "on", { ATMOS_GTK4_FILE: gtkFile });
assertEqual(result.status, 0, "gtk-middle-paste on exits 0");
assert(
  fs.readFileSync(gtkFile, "utf8").indexOf("gtk-enable-primary-paste=false") !== -1,
  "gtk-middle-paste on writes the paste flag",
);
result = runTweaks("gtk-middle-paste", "off", { ATMOS_GTK4_FILE: gtkFile });
assertEqual(result.status, 0, "gtk-middle-paste off exits 0");
assertEqual(fs.existsSync(gtkFile), false, "gtk-middle-paste off removes the overlay");

result = runTweaks("electron-wayland", "off", { ATMOS_ENV_FILE: envFile });
assertEqual(result.status, 0, "electron-wayland off seeds a missing env file");
assert(
  fs.readFileSync(envFile, "utf8").indexOf("ELECTRON_OZONE_PLATFORM_HINT=auto") !== -1,
  "electron-wayland off writes the opt-out",
);
result = runTweaks("electron-wayland", "on", { ATMOS_ENV_FILE: envFile });
assertEqual(result.status, 0, "electron-wayland on exits 0");
assertEqual(
  fs.readFileSync(envFile, "utf8").indexOf("ELECTRON_OZONE_PLATFORM_HINT=") === -1,
  true,
  "electron-wayland on strips the overlay",
);

fs.unlinkSync(envFile);
result = runTweaks("force-zero-scaling", "off", { ATMOS_ENV_FILE: envFile });
assertEqual(result.status, 0, "force-zero-scaling off seeds a missing env file");
assert(
  fs.readFileSync(envFile, "utf8").indexOf("ATMOS_XWAYLAND_ZERO_SCALING=0") !== -1,
  "force-zero-scaling off writes the opt-out",
);
result = runTweaks("force-zero-scaling", "on", { ATMOS_ENV_FILE: envFile });
assertEqual(result.status, 0, "force-zero-scaling on exits 0");
assertEqual(
  fs.readFileSync(envFile, "utf8").indexOf("ATMOS_XWAYLAND_ZERO_SCALING=") === -1,
  true,
  "force-zero-scaling on strips the overlay",
);

result = runTweaks("swappiness", "on", { ATMOS_SYSCTL_FILE: sysctlFile });
assertEqual(result.status, 0, "swappiness on exits 0 without as-root when dest is writable");
assert(
  fs.readFileSync(sysctlFile, "utf8").indexOf("vm.swappiness = 10") !== -1,
  "swappiness on writes the drop-in",
);
result = runTweaks("swappiness", "off", { ATMOS_SYSCTL_FILE: sysctlFile });
assertEqual(result.status, 0, "swappiness off exits 0");
assertEqual(fs.existsSync(sysctlFile), false, "swappiness off removes the drop-in");

const threeArg = spawnSync("bash", [script, "gtk-middle-paste", "unused", "off"], {
  encoding: "utf8",
  env: Object.assign({}, process.env, { ATMOS_GTK4_FILE: gtkFile }),
});
assertEqual(threeArg.status, 0, "a leftover $3 off does not abort");
assertEqual(fs.existsSync(gtkFile), true, "mode is $2, so a $3 off does not take the reset path");

fs.rmSync(fixture, { recursive: true, force: true });
