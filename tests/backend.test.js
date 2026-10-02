const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { assert, assertEqual } = require("./harness");

const root = path.join(__dirname, "..");
const omarchy = fs.readFileSync(path.join(root, "services", "Omarchy.qml"), "utf8");
const live = fs.readFileSync(path.join(root, "services", "LiveStatsStore.qml"), "utf8");
const launcher = fs.readFileSync(path.join(root, "bin", "atmos"), "utf8");
const xdg = fs.readFileSync(path.join(root, "scripts", "atmos-xdg.sh"), "utf8");
const settings = require(path.join(root, "services", "Settings.js"));

function body(src, name, next) {
  const start = src.indexOf("function " + name + "(");
  const end = src.indexOf("function " + next + "(", start);
  assert(start !== -1 && end > start, name + " is present");
  return src.slice(start, end);
}

const io = body(omarchy, "onReadRequested", "snapshotRefreshGroup");
assert(
  io.indexOf("Requests.settingsSnapshot") !== -1 &&
    io.indexOf("Requests.settingsSet") !== -1 &&
    io.indexOf("Backend.request") !== -1 &&
    io.indexOf("gui-snapshot") === -1,
  "settings loads and writes go through backend request ops",
);
const displayApply = body(omarchy, "applyDisplays", "domainValue");
assert(
  displayApply.indexOf("hardware") !== -1 &&
    displayApply.indexOf("disks") !== -1 &&
    displayApply.indexOf("systemdUnits") !== -1 &&
    displayApply.indexOf("desktopApps") !== -1 &&
    displayApply.indexOf("diagnostics") !== -1 &&
    displayApply.indexOf("displayDoc") !== -1,
  "the display snapshot is applied to the page properties",
);
assert(
  omarchy.indexOf("function displayDoc") !== -1 && omarchy.indexOf("value.error") !== -1,
  "a failed display kind does not replace the last good page",
);
assert(
  omarchy.indexOf("Requests.displaySnapshot") !== -1 && omarchy.indexOf("loadDisplays") !== -1,
  "display inventories are requested from the Rust backend",
);
assert(
  live.indexOf('Requests.displayGet("live")') !== -1 &&
    live.indexOf("Backend.request") !== -1 &&
    live.indexOf("liveStatsScript") === -1,
  "live stats load through the Rust backend without a script path",
);
assert(
  launcher.indexOf("exec quickshell") !== -1 && launcher.indexOf("ATMOS_BACKEND") !== -1,
  "bin/atmos still launches Quickshell and points it at the backend",
);
assert(
  xdg.indexOf("atmos_build_backend") !== -1 && xdg.indexOf("ratmos") !== -1,
  "the per-user install builds ratmos",
);

function backendBin() {
  const candidates = [
    path.join(root, "bin", "ratmos"),
    path.join(root, "backend", "target", "release", "ratmos"),
    path.join(root, "backend", "target", "debug", "ratmos"),
  ];
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) return candidate;
  }
  return "";
}

const bin = backendBin();
assert(bin.length > 0, "ratmos is built");
const version = spawnSync(bin, ["version"], { encoding: "utf8" });
assertEqual(version.status, 0, "ratmos version exits 0");
assertEqual(String(version.stdout).trim(), "0.1.0", "ratmos reports 0.1.0");

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "ratmos-list-"));
const listed = spawnSync(bin, ["--backend", "omarchy", "--root", fixture, "request"], {
  encoding: "utf8",
  input: JSON.stringify({ op: "settings.list" }),
});
fs.rmSync(fixture, { recursive: true, force: true });
assertEqual(listed.status, 0, "settings.list exits 0");
const payload = JSON.parse(listed.stdout);
assertEqual(payload.version, "0.1.0", "settings.list reports 0.1.0");
assertEqual(payload.platform.id, "omarchy", "settings.list names the omarchy platform");
const domains = new Set(payload.result.map((row) => row.domain));
settings.settingsCatalog().forEach(function (entry) {
  assert(domains.has(entry.key), "backend lists " + entry.key);
});
