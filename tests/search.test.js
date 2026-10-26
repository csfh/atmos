// Search runs in atmosd: the window sends the hub rows it holds in Hubs.js and
// atmosd reads the page sources under the app root for the labelled rows. These
// cases ask the real binary about the real pages directory.
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");
const { load, assert, assertEqual } = require("./harness");
const { atmosdBinary } = require("./sandbox");

const root = path.join(__dirname, "..");
const hubs = load("services/Hubs.js");
const requests = load("services/Requests.js");

function query(text, context) {
  const body = requests.searchQuery(text, context || hubs.searchContext());
  const run = spawnSync(atmosdBinary(), ["--backend", "plain", "request"], {
    encoding: "utf8",
    input: JSON.stringify(body),
    env: { ...process.env, ATMOS_ROOT: root },
  });
  assertEqual(run.status, 0, "atmosd answers a search request");
  const envelope = JSON.parse(run.stdout);
  assertEqual(envelope.ok, true, "a search request succeeds");
  return envelope.result;
}

const everything = query("").hits;
const has = (label, hub) => everything.some((row) => row.label === label && row.hub === hub);

assert(
  everything.some((row) => row.id === "appearance" && row.keywords.indexOf("wallpaper") !== -1),
  "hub rows carry the keywords Hubs.js gives them",
);
assertEqual(
  everything.filter((row) => row.id === row.hub && !row.id.includes("/")).length,
  hubs.searchHubs().length,
  "every hub is a row",
);
assert(
  has("Locale", "system") || everything.some((r) => r.label === "Locale"),
  "SettingRow blocks are indexed",
);
assert(has("Font", "appearance"), "the Font row is indexed");
assert(has("Text size", "appearance"), "shared Text size is indexed on Appearance");
assert(has("Text size", "accessibility"), "shared Text size is indexed on Accessibility");
assert(has("Animations", "windows"), "shared Animations is indexed on Windows");
assert(has("Inactive opacity", "windows"), "Inactive opacity is indexed on Windows");
assert(has("Touchscreen", "display"), "shared Touchscreen is indexed on Displays");
assert(has("Add a binding", "keybindings"), "binding rows go to the keybindings subpage");
assert(has("Add a rule", "windows/rules"), "window-rule rows go to the rules subpage");
assert(
  has("Next wallpaper", "appearance/background"),
  "wallpaper rows go to the background subpage",
);
assert(has("Unlock theme", "appearance/boot"), "Plymouth rows go to the boot subpage");
assert(
  has("Copy report", "system/diagnostics"),
  "diagnostic report rows go to the diagnostics subpage",
);
assert(has("Installed", "hardware"), "fitted memory goes to Hardware");
assert(has("Processor", "home"), "the Home processor row is indexed");
assert(has("Wi-Fi radio", "network/wifi"), "Wi-Fi rows go to the wifi subpage");
assert(has("Bluetooth radio", "bluetooth"), "Bluetooth rows go to the bluetooth subpage");
assert(has("Download", "network/speedtest"), "speed-test rows go to the speedtest subpage");
assert(
  everything.some(
    (row) =>
      row.label === "Add a binding" && row.hub === "keybindings" && row.hubTitle === "Keybindings",
  ),
  "binding search hits show the keybindings page title",
);
assert(
  !everything.some((row) => row.label === "Error dialog" || row.label === "Debug"),
  "the System Debug group is skipped",
);

const font = query("font").hits;
assert(
  font.some((row) => row.id === "appearance" || row.label === "Font"),
  "font finds the Font row",
);
assertEqual(query("FONT").hits.length, font.length, "FONT and font return the same hits");
assertEqual(query("  font ").hits.length, font.length, "a query is trimmed");
assert(
  query("zzzz-no-such-setting").hits.length === 0,
  "a query that matches nothing returns no hits",
);
const ordered = font.map((row) => row.hub + "\u0000" + row.label);
assertEqual(
  JSON.stringify(ordered),
  JSON.stringify([...ordered].sort()),
  "hits are ordered by hub, then label",
);
const echoed = query("Wi-Fi");
assertEqual(echoed.query, "Wi-Fi", "the reply echoes the query as asked");

const systemPageSrc = fs.readFileSync(path.join(root, "pages", "SystemPage.qml"), "utf8");
assert(
  systemPageSrc.indexOf("Omarchy.showDebugError()") !== -1,
  "System Debug shows the error dialog",
);
const omarchyDebugSrc = fs.readFileSync(path.join(root, "services", "Omarchy.qml"), "utf8");
assert(
  omarchyDebugSrc.indexOf("function showDebugError()") !== -1,
  "Omarchy.showDebugError is defined",
);
