const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const hubs = load("services/Hubs.js");
const iconsDir = path.join(__dirname, "..", "icons");
const pagesDir = path.join(__dirname, "..", "pages");
const atmosSrc = fs.readFileSync(path.join(__dirname, "..", "bin", "atmos"), "utf8");

function atmosHubIds() {
  const line = atmosSrc.split("\n").find(function (row) {
    return /\$HUB != appearance/.test(row);
  });
  const ids = [];
  const re = /\$HUB != ([A-Za-z0-9_-]+)/g;
  let m;
  while (line && (m = re.exec(line))) ids.push(m[1]);
  return ids;
}

function atmosSuffixes() {
  const m = atmosSrc.match(/\[\/([a-z0-9_|]+)\]/);
  return m ? m[1].split("|") : [];
}

const catalog = hubs.hubs();
assert(catalog.length > 0, "hubs() returns the catalog");
assertEqual(hubs.hubTitle("idle"), "Idle", "idle hub title is Idle");
assertEqual(hubs.hubTitle("export"), "Omafile", "export hub title is Omafile");
assert(
  hubs.hubById("export").keywords.indexOf("omafile") !== -1 &&
    hubs.hubById("export").keywords.indexOf("import") !== -1 &&
    hubs.hubById("export").keywords.indexOf("export") !== -1,
  "Omafile still matches omafile, import, and export in search",
);
assertEqual(
  hubs.hubTitle("windows/rules"),
  "Window rules",
  "child hub titles come from the catalog",
);
assertEqual(hubs.hubTitle("keybindings"), "Keybindings", "keybindings is a first-class hub");
assertEqual(hubs.rootHub("appearance/boot"), "appearance", "rootHub strips the child tail");
assertEqual(hubs.snapshotGroupForHub("appearance/boot"), "look", "appearance/boot reads look");
assertEqual(hubs.snapshotGroupForHub("network"), "network", "network hub reads network");
assertEqual(hubs.snapshotGroupForHub("idle"), "look", "idle hub reads look");
assertEqual(hubs.snapshotGroupForHub("hardware"), "all", "hardware hub reads all");
assertEqual(hubs.snapshotGroupForHub(""), "look", "empty hub reads look");
assertEqual(hubs.hubById("idle").navGroup, "device", "idle nav cluster is device");
assertEqual(hubs.hubById("system").navGroup, "general", "system nav cluster is general");
assertEqual(hubs.hubById("tweaks").navGroup, "general", "tweaks nav cluster is general");
assertEqual(hubs.hubById("export").navGroup, "general", "export nav cluster is general");
assertEqual(hubs.hubById("accounts").navGroup, "admin", "accounts nav cluster is admin");
assertEqual(hubs.hubById("security").navGroup, "admin", "security nav cluster is admin");
assertEqual(hubs.hubById("services").navGroup, "admin", "services nav cluster is admin");
assertEqual(hubs.hubById("appearance").snapshotGroup, "look", "appearance snapshot group is look");

const seenIcons = {};
catalog.forEach(function (hub) {
  assert(fs.existsSync(path.join(iconsDir, hub.icon + ".svg")), "icon file exists for " + hub.id);
  assert(!seenIcons[hub.icon], "hub icon is unique for " + hub.id);
  seenIcons[hub.icon] = hub.id;
  assert(fs.existsSync(path.join(pagesDir, hub.file)), "page file exists for " + hub.id);
  const kids = hub.children || [];
  for (let i = 0; i < kids.length; i++) {
    assert(
      fs.existsSync(path.join(pagesDir, kids[i].file)),
      "child page file exists for " + kids[i].id,
    );
  }
});

const pageFiles = [
  "AppearancePage.qml",
  "DisplaysPage.qml",
  "HardwarePage.qml",
  "WindowsPage.qml",
  "InputPage.qml",
  "AccessibilityPage.qml",
  "SoundPage.qml",
  "CapturePage.qml",
  "DisksPage.qml",
  "BarPage.qml",
  "NotificationsPage.qml",
  "DefaultsPage.qml",
  "ApplicationsPage.qml",
  "SoftwarePage.qml",
  "NetworkPage.qml",
  "PowerPage.qml",
  "IdlePage.qml",
  "SecurityPage.qml",
  "AccountsPage.qml",
  "HooksPage.qml",
  "SystemPage.qml",
  "ExportPage.qml",
  "appearance/BackgroundPage.qml",
  "appearance/BootPage.qml",
  "network/WifiPage.qml",
  "network/BluetoothPage.qml",
  "network/SpeedtestPage.qml",
  "windows/BindingsPage.qml",
  "windows/RulesPage.qml",
  "system/DiagnosticsPage.qml",
  "system/EnvironmentPage.qml",
  "applications/StartupPage.qml",
  "WorkspacesPage.qml",
  "TweaksPage.qml",
  "ServicesPage.qml",
  "ProfilesPage.qml",
];
pageFiles.forEach(function (file) {
  const id = hubs.fileHub(file);
  assert(!!id, "fileHub maps " + file);
  const found = hubs.hubById(id);
  assert(found && found.file === file, "fileHub round-trips " + file);
});

assertEqual(
  hubs.hubIds().join(","),
  atmosHubIds().join(","),
  "hubIds match the bin/atmos allow-list",
);

const childTails = hubs.childIds().map(function (id) {
  return id.split("/").pop();
});
const suffix = atmosSuffixes();
childTails.forEach(function (tail) {
  assert(suffix.indexOf(tail) !== -1, "bin/atmos suffix regex includes child tail " + tail);
});

const fakeChildren = hubs.childIds();
assert(fakeChildren.indexOf("appearance/theme") === -1, "theme is not an appearance child");
assert(fakeChildren.indexOf("hardware/gpu") === -1, "gpu is not a hardware child");
const aliases = hubs.launcherSuffixes();
["theme", "gpu", "cpu", "npu", "machine"].forEach(function (alias) {
  assert(aliases.indexOf(alias) !== -1, "launcherSuffixes includes alias " + alias);
});

const nav = hubs.navPages();
assertEqual(nav[0].id, "appearance", "navPages starts at appearance");
assertEqual(nav[0].group, "look", "navPages uses navGroup as group");
assert(nav[0].keywords.indexOf("wallpaper") !== -1, "navPages keywords include the union");
function consecutiveNavGroup(group) {
  const ids = nav
    .filter(function (page) {
      return page.group === group;
    })
    .map(function (page) {
      return page.id;
    });
  const start = nav.findIndex(function (page) {
    return page.group === group;
  });
  return nav
    .slice(start, start + ids.length)
    .map(function (page) {
      return page.id;
    })
    .join(",");
}
assertEqual(
  consecutiveNavGroup("general"),
  "system,tweaks,export",
  "general hubs stay consecutive in nav order",
);
assertEqual(
  consecutiveNavGroup("admin"),
  "accounts,security,services",
  "admin hubs stay consecutive in nav order",
);

const search = hubs.searchHubs();
assertEqual(search[0].description.indexOf("Theme") !== -1, true, "searchHubs keeps descriptions");
assert(Array.isArray(search[0].keywords), "searchHubs keywords are an array");
assert(
  hubs.hubById("input").keywords.indexOf("keyboard") !== -1,
  "input keywords union includes SearchIndex keyboard",
);
assert(
  hubs.hubById("software").keywords.indexOf("package") !== -1,
  "software keywords union includes SearchIndex package",
);
