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
