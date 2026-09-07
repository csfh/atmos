const fs = require("fs");
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
