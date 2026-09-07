const fs = require("fs");
const path = require("path");
const { assert, assertEqual } = require("./harness");

const page = fs.readFileSync(path.join(__dirname, "..", "pages", "ServicesPage.qml"), "utf8");
const row = fs.readFileSync(path.join(__dirname, "..", "components", "ServiceRow.qml"), "utf8");
const menu = fs.readFileSync(path.join(__dirname, "..", "components", "PrefsMenu.qml"), "utf8");

assert(
  page.indexOf('placeholder: "Search services…"') !== -1,
  "Services search placeholder is Search services…",
);
assert(page.indexOf("SystemdJs.filterChips") !== -1, "Services uses filter chips from Systemd.js");
assert(page.indexOf("SystemdJs.summaryParts") !== -1, "Services uses the compact summary");
assert(page.indexOf('title: "Failed"') === -1, "Services does not keep a Failed section");
assert(
  page.indexOf("Nothing failed.") === -1,
  "Services does not dedicate a row to Nothing failed",
);
assert(
  page.indexOf('text: "Start"') === -1,
  "Services does not hard-code a Start button on every row",
);
assert(page.indexOf('text: "Enable"') === -1, "Services does not hard-code Enable on every row");
assert(page.indexOf("ServiceRow") !== -1, "Services lists units with ServiceRow");
assert(page.indexOf("No matching services") !== -1, "Services has an empty state");
assert(page.indexOf('selectFilter("failed")') !== -1, "a failure count filters the list");

assert(row.indexOf("SystemdJs.statusLine") !== -1, "ServiceRow uses the compact status line");
assert(row.indexOf("SystemdJs.primaryAction") !== -1, "ServiceRow uses a context primary action");
assert(row.indexOf("PrefsMenu") !== -1, "ServiceRow puts secondary actions in PrefsMenu");
assert(row.indexOf("elide: Text.ElideRight") !== -1, "ServiceRow elides long unit names");
assert(
  row.indexOf("ToolTip.text: root.unitName") !== -1,
  "ServiceRow keeps the full unit name on hover",
);

assert(menu.indexOf('text: "…"') !== -1, "PrefsMenu trigger is an ellipsis");
assert(menu.indexOf('accessibleName: "More actions"') !== -1, "PrefsMenu is named More actions");
assertEqual(menu.indexOf("PrefsFlickable") === -1, true, "PrefsMenu is a compact overlay list");

const snapshot = fs.readFileSync(path.join(__dirname, "..", "scripts", "snapshot.sh"), "utf8");
assert(
  snapshot.indexOf("systemd-inventory.py") !== -1,
  "snapshot collects units through systemd-inventory.py",
);
