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
