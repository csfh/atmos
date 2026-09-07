const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const ws = load("services/Workspaces.js");
assertEqual(ws.clampCount(99), 10, "clampCount caps at 10");
assertEqual(ws.clampCount(0), 10, "clampCount floors empty to 10");
const state = ws.clampState({
  count: 2,
  items: [{ id: "1", name: "code", monitor: "DP-1", isDefault: true }],
});
assertEqual(state.count, 2, "clampState keeps count 2");
assertEqual(state.items.length >= 2, true, "clampState fills missing ids");
assertEqual(state.items[0].name, "code", "clampState keeps a name");
assertEqual(state.items[0].monitor, "DP-1", "clampState keeps a monitor");
const shrunk = ws.clampState({
  count: 3,
  items: [
    { id: "1" },
    { id: "2" },
    { id: "3" },
    { id: "4" },
    { id: "10" },
    { id: "special:notes" },
  ],
});
assertEqual(shrunk.count, 3, "clampState keeps count 3 when shrinking");
assertEqual(
  shrunk.items
    .filter(function (row) {
      return !row.special;
    })
    .map(function (row) {
      return row.id;
    })
    .join(","),
  "1,2,3",
  "clampState drops numbered workspaces above count",
);
assertEqual(
  shrunk.items.some(function (row) {
    return row.id === "special:notes";
  }),
  true,
  "clampState keeps specials when shrinking",
);
const shrinkLua = ws.serialize({ items: [{ id: "1" }, { id: "2" }, { id: "3" }] });
assert(
  shrinkLua.indexOf('workspace = "4"') === -1,
  "serialize without count does not refill workspace 4",
);
assert(shrinkLua.indexOf('workspace = "3"') !== -1, "serialize without count keeps workspace 3");
assertEqual(ws.normalizeItem({ id: "evil;rm" }), null, "normalizeItem drops an unsafe id");
assertEqual(ws.wrapBind(false), "r+1", "wrapBind false uses r+1");
assertEqual(ws.wrapBind(true), "e+1", "wrapBind true uses e+1");
assertEqual(ws.wrapBind(false, -1), "r-1", "wrapBind false previous uses r-1");

const lua = ws.serialize(state);
assert(lua.indexOf("-- atmos:workspaces begin") === 0, "serialize starts with sentinel");
assert(lua.indexOf('workspace = "1"') !== -1, "serialize writes workspace 1");
assert(lua.indexOf('default_name = "code"') !== -1, "serialize writes default_name");
assert(lua.indexOf('monitor = "DP-1"') !== -1, "serialize writes monitor");
assert(lua.indexOf("persistent = true") !== -1, "serialize writes persistent");
assert(lua.indexOf("default = true") !== -1, "serialize writes default");

const parsed = ws.parseFile(lua);
assertEqual(parsed.items[0].name, "code", "parseFile reads default_name");
assertEqual(parsed.items[0].isDefault, true, "parseFile reads default");
assertEqual(parsed.wrapSwitch, true, "parseFile defaults wrapSwitch on");
assertEqual(parsed.wheelSwitch, true, "parseFile defaults wheelSwitch on");

const noWrap = ws.serialize({
  count: 2,
  wrapSwitch: false,
  wheelSwitch: false,
  items: [{ id: "1" }, { id: "2" }],
});
assert(noWrap.indexOf("-- atmos:wrapSwitch = false") !== -1, "serialize writes wrapSwitch false");
assert(noWrap.indexOf("-- atmos:wheelSwitch = false") !== -1, "serialize writes wheelSwitch false");
assert(
  noWrap.indexOf("hyprctl dispatch workspace r+1") !== -1,
  "serialize uses r+1 when wrap is off",
);
assert(
  noWrap.indexOf('hl.unbind("SUPER + mouse_down")') !== -1,
  "serialize unbinds Super+wheel when wheel is off",
);
assert(
  noWrap.indexOf('o.bind("SUPER + mouse_down"') === -1,
  "serialize does not rebind Super+wheel when wheel is off",
);
const parsedFlags = ws.parseFile(noWrap);
assertEqual(parsedFlags.wrapSwitch, false, "parseFile reads wrapSwitch false");
assertEqual(parsedFlags.wheelSwitch, false, "parseFile reads wheelSwitch false");

const tmp = path.join(os.tmpdir(), "atmos-ws-" + process.pid + ".lua");
fs.writeFileSync(tmp, "");
const py = spawnSync(
  "python3",
  [
    path.join(__dirname, "..", "scripts", "hypr-sentinel.py"),
    "workspaces",
    "apply",
    tmp,
    JSON.stringify(state),
  ],
  { encoding: "utf8" },
);
assertEqual(py.status, 0, "hypr-sentinel.py workspaces apply exits 0");
const pyText = fs.readFileSync(tmp, "utf8");
assertEqual(
  pyText.replace(/\s+$/, ""),
  lua.replace(/\s+$/, ""),
  "Python workspaces serialize matches JS",
);
fs.unlinkSync(tmp);
