const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const rules = load("services/WindowRules.js");
assertEqual(rules.sanitizeMatch("firefox"), "firefox", "sanitizeMatch keeps a class");
assertEqual(rules.sanitizeMatch("bad]]class"), "", "sanitizeMatch rejects ]]");
assertEqual(rules.sanitizeWorkspace("../etc"), "", "sanitizeWorkspace rejects a path");
assertEqual(
  rules.sanitizeWorkspace(" special:5 "),
  "special:5",
  "sanitizeWorkspace keeps a named workspace",
);
assertEqual(rules.sanitizeWorkspace(""), "", "sanitizeWorkspace empty");
assertEqual(rules.sanitizeMatch("bad\nclass"), "", "sanitizeMatch rejects a newline");
assertEqual(
  rules.sanitizeMatch("x".repeat(129)),
  "",
  "sanitizeMatch rejects a match over 128 chars",
);
assertEqual(rules.describe(null), "", "describe empty for a missing row");
assertEqual(rules.describe({ center: true }), "center", "describe names center");
const ruleSeed =
  'o.window("dev.csfh.atmos", { float = true })\no.window("dev.csfh.atmos", { center = true })\n';
const ruleApplied = rules.applyFile(ruleSeed, [
  { match: "^Emulator$", placement: "float", center: true, width: 1280, height: 800 },
  { match: "qemu", workspace: "5" },
]);
assert(
  ruleApplied.indexOf('o.window("dev.csfh.atmos", { float = true })') !== -1,
  "applyFile keeps prefs window rules",
);
assert(
  ruleApplied.indexOf(
    'o.window("^Emulator$", { float = true, center = true, size = { 1280, 800 } })',
  ) !== -1,
  "applyFile writes a managed window rule",
);
assert(
  ruleApplied.indexOf('o.window("qemu", { workspace = "5" })') !== -1,
  "applyFile writes a workspace rule",
);
const ruleParsed = rules.parseFile(ruleApplied);
assertEqual(
  ruleParsed.filter(function (row) {
    return row.managed;
  }).length,
  2,
  "parseFile marks managed window rules",
);
assertEqual(
  ruleParsed.filter(function (row) {
    return !row.managed && row.match === "dev.csfh.atmos";
  }).length,
  2,
  "parseFile keeps the prefs window rules",
);
const required = rules.ensureRequire(
  'require("hypr.autostart")\nrequire("default.hypr.toggles")\n',
);
assert(
  required.indexOf('require("hypr.atmos")\nrequire("default.hypr.toggles")') !== -1,
  "ensureRequire inserts before toggles",
);
assertEqual(rules.ensureRequire(""), "", "ensureRequire leaves an empty hyprland.lua alone");
const withOmarchy = 'require("default.hypr.omarchy")\nrequire("hypr.bindings")\n';
assert(
  rules
    .ensureLayoutRequire(withOmarchy)
    .indexOf('require("hypr.atmos_layout")\nrequire("default.hypr.omarchy")') !== -1,
  "ensureLayoutRequire inserts before Omarchy defaults so SUPER+J wrap is in place",
);
assertEqual(
  rules.ensureLayoutRequire('require("hypr.atmos_layout")\nrequire("default.hypr.omarchy")\n'),
  'require("hypr.atmos_layout")\nrequire("default.hypr.omarchy")\n',
  "ensureLayoutRequire is a no-op when hypr.atmos_layout is present",
);
assertEqual(
  rules.ensureLayoutRequire(""),
  "",
  "ensureLayoutRequire leaves an empty hyprland.lua alone",
);
const layoutLua = fs.readFileSync(
  path.join(__dirname, "..", "packaging", "hypr-atmos-layout.lua"),
  "utf8",
);
assert(
  layoutLua.indexOf("tiled_layout") !== -1 && layoutLua.indexOf("togglesplit") !== -1,
  "atmos_layout skips dwindle togglesplit on scrolling workspaces",
);
{
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-layout-"));
  try {
    const hyprland = path.join(dir, "hyprland.lua");
    fs.writeFileSync(
      hyprland,
      'require("default.hypr.omarchy")\nrequire("hypr.atmos")\nrequire("default.hypr.toggles")\n',
    );
    const result = spawnSync(
      "python3",
      [path.join(__dirname, "..", "scripts", "hypr-sentinel.py"), "require", "apply", hyprland],
      { encoding: "utf8" },
    );
    assertEqual(result.status, 0, "hypr-sentinel.py require apply exits 0");
    const written = fs.readFileSync(hyprland, "utf8");
    assert(
      written.indexOf('require("hypr.atmos_layout")\nrequire("default.hypr.omarchy")') !== -1,
      "hypr-sentinel.py require apply inserts atmos_layout before Omarchy defaults",
    );
    const layoutFile = fs.readFileSync(path.join(dir, "atmos_layout.lua"), "utf8");
    assert(
      layoutFile.indexOf("togglesplit") !== -1,
      "hypr-sentinel.py require apply writes atmos_layout.lua",
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}
assertEqual(
  rules.describe({ placement: "float", center: true }).indexOf("float") !== -1,
  true,
  "describe names float",
);
assertEqual(rules.clampSize(1280), 1280, "clampSize keeps a valid size");
assertEqual(rules.clampSize(50), 0, "clampSize rejects below 100");
assertEqual(rules.clampSize(5000), 0, "clampSize rejects above 4000");
assertEqual(rules.clampSize("nope"), 0, "clampSize rejects NaN");
assert(
  rules.prefsSeed().indexOf('o.window("dev.csfh.atmos"') !== -1,
  "prefsSeed floats the Atmos class",
);
assert(rules.prefsSeed().indexOf("size = { 960, 680 }") !== -1, "prefsSeed sets the Atmos size");
assertEqual(
  rules.describe({ placement: "tile", workspace: "5" }),
  "tile \u00b7 workspace 5",
  "describe names tile and workspace",
);
assertEqual(
  rules.describe({ width: 1280, height: 800 }).indexOf("\u00d7") !== -1,
  true,
  "describe names a size",
);
const already = 'require("hypr.atmos")\nrequire("default.hypr.toggles")\n';
assertEqual(
  rules.ensureRequire(already),
  already,
  "ensureRequire is a no-op when hypr.atmos is present",
);
const appended = rules.ensureRequire('require("hypr.autostart")\n');
assert(
  appended.indexOf('require("hypr.atmos")') !== -1,
  "ensureRequire appends when toggles are missing",
);
const managedRules = rules.managedItems([
  { match: "qemu", workspace: "5", managed: false },
  { match: "firefox", placement: "float", managed: true },
  { match: "bad]]", placement: "float" },
  null,
]);
assertEqual(managedRules.length, 1, "managedItems drops unmanaged and invalid rows");
assertEqual(managedRules[0].match, "firefox", "managedItems keeps a managed float rule");
assertEqual(
  rules.parseCalls('o.window("foo\\"bar", { float = true })\n')[0].match,
  'foo"bar',
  "parseCalls unescapes a quoted window match",
);
assert(
  rules.serialize([{ match: "qemu", placement: "tile" }]).indexOf("tile = true") !== -1,
  "serialize writes tile = true",
);
assertEqual(
  rules.normalize({ match: "firefox", float: true }).placement,
  "float",
  "normalize maps float true to placement",
);
assertEqual(
  rules.normalize({ match: "foot", size: [1280, 800] }).width,
  1280,
  "normalize reads a size array",
);
assertEqual(rules.normalize({ match: "ghost" }), null, "normalize drops a match with no effects");
const commentedWindows = `-- o.window("firefox", { float = true })
o.window("dev.csfh.atmos", { float = true }) -- keep
`;
const liveWindows = rules.parseFile(commentedWindows);
assertEqual(liveWindows.length, 1, "parseFile skips a commented o.window example");
assertEqual(liveWindows[0].match, "dev.csfh.atmos", "parseFile keeps a live window rule");

function pythonList(kind, text) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-list-"));
  const file = path.join(dir, kind + ".lua");
  let parsed = [];
  try {
    fs.writeFileSync(file, text);
    const result = spawnSync(
      "python3",
      [path.join(__dirname, "..", "scripts", "hypr-sentinel.py"), kind, "list", file],
      { encoding: "utf8" },
    );
    assertEqual(result.status, 0, "hypr-sentinel.py " + kind + " list exits 0");
    parsed = JSON.parse(result.stdout);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  return parsed;
}

assertEqual(
  JSON.stringify(pythonList("windows", commentedWindows)),
  JSON.stringify(liveWindows),
  "hypr-sentinel.py windows list matches parseFile on commented examples",
);

const innerCommentWindows = `
o.window("firefox", {
  -- float the browser
  float = true,
  center = true, -- yes
})
hint = "class -- match"; o.window("foot", { tile = true })
`;
const innerWindows = rules.parseFile(innerCommentWindows);
assertEqual(innerWindows.length, 2, "parseFile keeps window rules with comments inside the call");
assertEqual(innerWindows[0].match, "firefox", "parseFile skips a comment in a window table");
assertEqual(innerWindows[0].center, true, "parseFile keeps fields after a table comment");
assertEqual(
  innerWindows[1].match,
  "foot",
  "parseFile keeps a window rule after -- inside a string",
);
assertEqual(
  JSON.stringify(pythonList("windows", innerCommentWindows)),
  JSON.stringify(innerWindows),
  "hypr-sentinel.py windows list matches parseFile on comments inside calls",
);

assertEqual(
  rules.parseCalls('o.window("foo\\nbar", { float = true })').length,
  0,
  "parseCalls drops a window match whose Lua \\n is a newline",
);
assertEqual(
  rules.parseCalls('o.window("foo\\\\nbar", { float = true })')[0].match,
  "foo\\nbar",
  "parseCalls keeps a Lua \\\\n in the match as a literal backslash-n",
);
assertEqual(rules.luaString("a\tb\nc"), '"a\\tb\\nc"', "luaString escapes a tab and a newline");
const luaEscapesWindows = `
o.window("foo\\nbar", { float = true })
o.window("foo\\\\nbar", { float = true })
`;
const luaEscapesWindowParsed = rules.parseFile(luaEscapesWindows);
assertEqual(luaEscapesWindowParsed.length, 1, "parseFile drops a window match with a Lua newline");
assertEqual(
  luaEscapesWindowParsed[0].match,
  "foo\\nbar",
  "parseFile keeps a window match with a literal backslash-n",
);
assertEqual(
  JSON.stringify(pythonList("windows", luaEscapesWindows)),
  JSON.stringify(luaEscapesWindowParsed),
  "hypr-sentinel.py windows list matches parseFile on Lua string escapes",
);
