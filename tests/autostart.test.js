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
