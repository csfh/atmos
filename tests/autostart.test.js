const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const auto = load("services/Autostart.js");
const autoSeed = '-- Extra autostart processes.\no.launch_on_start("waybar")\n';
const autoApplied = auto.applyFile(autoSeed, ["hyprsunset", "mako"]);
assert(
  autoApplied.indexOf("-- Extra autostart processes.") !== -1,
  "applyFile keeps user comments",
);
assert(
  autoApplied.indexOf('o.launch_on_start("waybar")') !== -1,
  "applyFile keeps unmanaged launch lines",
);
assert(
  autoApplied.indexOf('o.launch_on_start("hyprsunset")') !== -1,
  "applyFile writes a managed command",
);
const autoParsed = auto.parseFile(autoApplied);
assertEqual(
  autoParsed.filter(function (row) {
    return row.managed;
  }).length,
  2,
  "parseFile marks managed commands",
);
assertEqual(
  autoParsed.filter(function (row) {
    return !row.managed && row.command === "waybar";
  }).length,
  1,
  "parseFile keeps unmanaged commands",
);
assertEqual(auto.sanitizeCommand("bad\ncmd"), "", "sanitizeCommand rejects a newline");
assertEqual(
  auto.sanitizeCommand("x".repeat(257)),
  "",
  "sanitizeCommand rejects a command over 256 chars",
);
assertEqual(auto.unescapeLua('\\"quoted\\"'), '"quoted"', "unescapeLua restores escaped quotes");
assertEqual(auto.luaString('say "hi"'), '"say \\"hi\\""', "luaString escapes quotes");
const quotedLaunch = auto.parseCalls('o.launch_on_start("echo \\"hi\\"")\n');
assertEqual(quotedLaunch[0], 'echo "hi"', "parseCalls unescapes a quoted command");
const managed = auto.managedCommands([
  { command: "waybar", managed: false },
  { command: "mako", managed: true },
  "hyprsunset",
  { command: "bad\ncmd", managed: true },
  null,
]);
assertEqual(
  managed.join(","),
  "mako,hyprsunset",
  "managedCommands keeps only managed or string rows",
);
const autoReplaced = auto.applyFile(autoApplied, ["swaybg"]);
assertEqual(
  (autoReplaced.match(/-- atmos:autostart begin/g) || []).length,
  1,
  "applyFile replaces an existing autostart sentinel",
);
assert(
  autoReplaced.indexOf('o.launch_on_start("swaybg")') !== -1,
  "applyFile writes the new managed command",
);
assert(
  autoReplaced.indexOf('o.launch_on_start("mako")') === -1,
  "applyFile drops previous managed commands",
);
assert(
  auto.extractSentinel(autoApplied).indexOf('o.launch_on_start("hyprsunset")') !== -1,
  "extractSentinel returns the autostart block",
);
assertEqual(
  auto.extractSentinel("-- Extra autostart processes.\n"),
  "",
  "extractSentinel misses a file without a sentinel",
);
assertEqual(auto.sanitizeCommand(""), "", "sanitizeCommand rejects empty");
assert(
  auto.serialize([]).indexOf("o.launch_on_start") === -1,
  "serialize empty list writes only the sentinel",
);
assertEqual(
  auto.parseCalls('o.launch_on_start("")').length,
  0,
  "parseCalls skips an empty command",
);
const omarchyAutostart = `-- Extra autostart processes.
-- o.launch_on_start("my-service")
o.launch_on_start("waybar") -- keep
`;
const liveAuto = auto.parseFile(omarchyAutostart);
assertEqual(liveAuto.length, 1, "parseFile skips Omarchy's commented autostart example");
assertEqual(liveAuto[0].command, "waybar", "parseFile keeps a live launch with a trailing comment");
assertEqual(liveAuto[0].managed, false, "parseFile marks a live launch outside the sentinel");

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
  JSON.stringify(pythonList("autostart", omarchyAutostart)),
  JSON.stringify(liveAuto),
  "hypr-sentinel.py autostart list matches parseFile on commented examples",
);

const stringDashAuto = `hint = "echo --help"; o.launch_on_start("waybar")\n`;
const stringDashParsed = auto.parseFile(stringDashAuto);
assertEqual(stringDashParsed.length, 1, "parseFile keeps a launch after -- inside a string");
assertEqual(stringDashParsed[0].command, "waybar", "parseFile reads the launch after a string --");
assertEqual(
  JSON.stringify(pythonList("autostart", stringDashAuto)),
  JSON.stringify(stringDashParsed),
  "hypr-sentinel.py autostart list matches parseFile on -- inside a string",
);

assertEqual(auto.unescapeLua("\\n"), "\n", "unescapeLua turns Lua \\n into a newline");
assertEqual(auto.unescapeLua("\\\\n"), "\\n", "unescapeLua keeps a Lua \\\\n as backslash-n");
assertEqual(auto.unescapeLua("a\\tb\\rc"), "a\tb\rc", "unescapeLua turns Lua \\t and \\r");
assertEqual(auto.luaString("a\tb\nc"), '"a\\tb\\nc"', "luaString escapes a tab and a newline");
assertEqual(
  auto.parseCalls('o.launch_on_start("echo hello\\nworld")').length,
  0,
  "parseCalls drops a launch whose Lua \\n is a newline",
);
assertEqual(
  auto.parseCalls('o.launch_on_start("echo hello\\\\nworld")')[0],
  "echo hello\\nworld",
  "parseCalls keeps a Lua \\\\n as a literal backslash-n",
);
assertEqual(
  auto.parseCalls('o.launch_on_start("echo a\\tb")')[0],
  "echo a\tb",
  "parseCalls turns Lua \\t into a tab",
);
const luaEscapesAuto = `
o.launch_on_start("echo hello\\nworld")
o.launch_on_start("echo hello\\\\nworld")
o.launch_on_start("echo a\\tb")
`;
const luaEscapesAutoParsed = auto.parseFile(luaEscapesAuto);
assertEqual(luaEscapesAutoParsed.length, 2, "parseFile drops a launch with a Lua newline");
assertEqual(
  luaEscapesAutoParsed[0].command,
  "echo hello\\nworld",
  "parseFile keeps a launch with a literal backslash-n",
);
assertEqual(
  luaEscapesAutoParsed[1].command,
  "echo a\tb",
  "parseFile keeps a launch with a Lua tab",
);
assertEqual(
  JSON.stringify(pythonList("autostart", luaEscapesAuto)),
  JSON.stringify(luaEscapesAutoParsed),
  "hypr-sentinel.py autostart list matches parseFile on Lua string escapes",
);
