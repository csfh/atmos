const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const mon = load("services/Monitors.js");
assertEqual(
  mon.modeFromHyprctl("1920x1080@60.00Hz"),
  "1920x1080@60.00",
  "modeFromHyprctl strips Hz",
);
assertEqual(mon.modeFromHyprctl("preferred"), "preferred", "modeFromHyprctl keeps preferred");
assertEqual(mon.normalizeItem({ output: "DP-1;rm" }), null, "normalizeItem drops an unsafe output");
const row = mon.normalizeItem({
  output: "DP-1",
  mode: "2560x1440@144",
  position: "0x0",
  scale: 1.25,
  transform: 1,
  vrr: 2,
  bitdepth: 10,
  cm: "wide",
});
assertEqual(row.output, "DP-1", "normalizeItem keeps output");
assertEqual(row.scale, 1.25, "normalizeItem keeps scale");
assertEqual(row.bitdepth, 10, "normalizeItem keeps 10-bit");
const lua = mon.serialize({ items: [row] });
assert(lua.indexOf("-- atmos:monitors begin") === 0, "serialize starts with sentinel");
assert(lua.indexOf('output = "DP-1"') !== -1, "serialize writes output");
assert(lua.indexOf("vrr = 2") !== -1, "serialize writes vrr");
assert(lua.indexOf("bitdepth = 10") !== -1, "serialize writes bitdepth");
assert(lua.indexOf("disabled = true") === -1, "serialize omits disabled when false");
const disabled = mon.serialize({
  items: [{ output: "eDP-1", mode: "preferred", disabled: true, scale: 1 }],
});
assert(disabled.indexOf("disabled = true") !== -1, "serialize writes disabled");
const parsed = mon.parseFile(lua);
assertEqual(parsed.items[0].mode, "2560x1440@144", "parseFile reads mode");
assertEqual(
  mon.layoutNames().join(","),
  "desk,laptop,docked",
  "layoutNames lists Desk Laptop Docked",
);
const desk = mon.layoutFromLive("laptop", [
  { name: "eDP-1", internal: true, width: 1920, height: 1200, scale: 1 },
  { name: "DP-1", internal: false, width: 2560, height: 1440, scale: 1 },
]);
assertEqual(desk.length, 2, "layoutFromLive keeps both outputs");
assertEqual(desk[0].disabled, false, "laptop layout keeps the internal output");
assertEqual(desk[1].disabled, true, "laptop layout disables the external output");

const tmp = path.join(os.tmpdir(), "atmos-mon-" + process.pid + ".lua");
fs.writeFileSync(tmp, "");
const payload = { items: [row] };
const py = spawnSync(
  "python3",
  [
    path.join(__dirname, "..", "scripts", "hypr-sentinel.py"),
    "monitors",
    "apply",
    tmp,
    JSON.stringify(payload),
  ],
  { encoding: "utf8" },
);
assertEqual(py.status, 0, "hypr-sentinel.py monitors apply exits 0");
const pyText = fs.readFileSync(tmp, "utf8");
assertEqual(
  pyText.replace(/\s+$/, ""),
  lua.replace(/\s+$/, ""),
  "Python monitors serialize matches JS",
);
fs.unlinkSync(tmp);
