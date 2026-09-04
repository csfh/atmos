const fs = require("fs");
const path = require("path");
const { assert, assertEqual } = require("./harness");

const search = require("../services/SearchIndex.js");
assert(typeof search.setSetting === "undefined", "search index has no setSetting write path");
assert(typeof search.writePrefs === "undefined", "search index has no writePrefs path");
assert(typeof search.savePrefs === "undefined", "search index has no savePrefs path");

const catalog = [
  {
    id: "appearance/font",
    hub: "appearance",
    hubTitle: "Appearance",
    label: "Font",
    description: "The monospace face",
    hint: "omarchy font set",
    keywords: ["theme", "typeface"],
  },
  {
    id: "network/wifi",
    hub: "network",
    hubTitle: "Network",
    label: "Wi-Fi",
    description: "Wireless radio",
    hint: "nmcli",
    keywords: ["ssid"],
  },
];
const index = search.openIndex();
search.ingestRows(index, catalog);
const emptyHits = search.queryRows(index, "");
assert(emptyHits.length === 2, "empty query matches every indexed row");
const fontHits = search.queryRows(index, "font");
const servedFont = JSON.parse(
  search.handleServeLine(index, JSON.stringify({ cmd: "query", query: "font" })),
);
assertEqual(servedFont.query, "font", "handleServeLine echoes the query");
assert(
  servedFont.hits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "handleServeLine hits font without reopening sqlite",
);
const servedAgain = JSON.parse(search.handleServeLine(index, "ssid"));
assert(
  servedAgain.hits.some(function (row) {
    return row.id === "network/wifi";
  }),
  "handleServeLine reuses the same index for a second query",
);
assertEqual(search.parseArgs(["serve", "--root", "/tmp"]).cmd, "serve", "parseArgs accepts serve");
assert(
  fontHits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "font matches the theme/font row",
);
assert(
  !fontHits.some(function (row) {
    return row.id === "network/wifi";
  }),
  "font does not match an unrelated network row",
);
const fontUpperHits = search.queryRows(index, "FONT");
assert(
  fontUpperHits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "FONT matches the same lowered haystack",
);
assertEqual(fontUpperHits.length, fontHits.length, "FONT and font return the same hit count");
const networkHits = search.queryRows(index, "network");
assert(
  !networkHits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "network does not match a theme/font row",
);

search.ingestSnapshot(index, { theme: "omarchy", barPosition: "top", nested: { lock: 300 } });
assertEqual(
  search.getState(index, "theme"),
  "omarchy",
  "derived state loads theme from snapshot JSON",
);
assertEqual(
  search.getState(index, "barPosition"),
  "top",
  "derived state loads barPosition from snapshot JSON",
);
assertEqual(
  search.getState(index, "nested.lock"),
  "300",
  "derived state flattens nested snapshot fields",
);
assertEqual(search.getState(index, "missing"), null, "derived state misses unknown keys");

search.ingestSnapshot(index, { theme: "tokyo" });
search.ingestSnapshot(index, { hardware: { cpu: { model: "X" } } });
assertEqual(
  search.getState(index, "theme"),
  "tokyo",
  "second ingest keeps theme from the first patch",
);
assertEqual(
  search.getState(index, "hardware.cpu.model"),
  "X",
  "second ingest adds hardware without a write-prefs path",
);
assertEqual(
  search.indexPath({ ATMOS_SEARCH_INDEX: "/tmp/atmos-search.sqlite" }),
  "/tmp/atmos-search.sqlite",
  "indexPath honors ATMOS_SEARCH_INDEX",
);
assertEqual(
  search.indexPath({ XDG_CACHE_HOME: "/tmp/xdg-cache", HOME: "/tmp/home" }),
  "/tmp/xdg-cache/atmos/search.sqlite",
  "indexPath uses XDG_CACHE_HOME when ATMOS_SEARCH_INDEX is unset",
);
assertEqual(
  search.indexPath({ HOME: "/tmp/home" }),
  "/tmp/home/.cache/atmos/search.sqlite",
  "indexPath falls back to HOME/.cache/atmos/search.sqlite",
);

const cacheDir = fs.mkdtempSync(path.join(require("os").tmpdir(), "atmos-search-"));
const cacheFile = path.join(cacheDir, "nested", "search.sqlite");
const onDisk = search.openIndex(cacheFile);
search.ingestRows(onDisk, catalog);
onDisk.close();
assert(fs.existsSync(cacheFile), "openIndex creates the sqlite file and parent dirs");
const reopened = search.openIndex(cacheFile);
assert(
  search.queryRows(reopened, "font").some(function (row) {
    return row.id === "appearance/font";
  }),
  "reopened sqlite file still matches font",
);
reopened.close();
fs.rmSync(cacheDir, { recursive: true, force: true });

const qmlCatalog = search.catalogFromQmlDir(path.join(__dirname, "..", "pages"));
assert(
  qmlCatalog.some(function (row) {
    return row.label === "Font" && row.hub === "appearance";
  }),
  "qml catalog indexes the Font row",
);
assert(
  qmlCatalog.some(function (row) {
    return row.label === "Text size" && row.hub === "appearance";
  }),
  "qml catalog indexes shared Text size on Appearance",
);
assert(
  qmlCatalog.some(function (row) {
    return row.label === "Text size" && row.hub === "accessibility";
  }),
  "qml catalog indexes shared Text size on Accessibility",
);
assert(
  qmlCatalog.some(function (row) {
    return row.label === "Animations" && row.hub === "windows";
  }),
  "qml catalog indexes shared Animations on Windows",
);
assert(
  qmlCatalog.some(function (row) {
    return row.label === "Touchscreen" && row.hub === "display";
  }),
  "qml catalog indexes shared Touchscreen on Displays",
);
assert(
  !qmlCatalog.some(function (row) {
    return row.label === "Error dialog" || row.label === "Debug";
  }),
  "qml catalog skips the System Debug group",
);
const systemPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "SystemPage.qml"),
  "utf8",
);
assert(
  systemPageSrc.indexOf("Omarchy.showDebugError()") !== -1,
  "System Debug shows the error dialog",
);
const omarchyDebugSrc = fs.readFileSync(
  path.join(__dirname, "..", "services", "Omarchy.qml"),
  "utf8",
);
assert(
  omarchyDebugSrc.indexOf("function showDebugError()") !== -1,
  "Omarchy.showDebugError is defined",
);
