const { load, assert, assertEqual } = require("./harness");

const binds = load("services/Bindings.js");
assertEqual(
  binds.sanitizeKeys("SUPER + SHIFT + R"),
  "SUPER + SHIFT + R",
  "sanitizeKeys keeps a chord",
);
assertEqual(binds.sanitizeKeys("SUPER + F\n"), "", "sanitizeKeys rejects a newline");
assertEqual(binds.sanitizeCommand("bad\ncmd"), "", "sanitizeCommand rejects a newline");
const bindSeed =
  '-- Keep only your personal keybinding overrides here.\no.bind("SUPER + D", "Desks", "omarchy-shell shell toggle com.mdtrr.omadesk")\n';
const bindApplied = binds.applyFile(bindSeed, [
  { keys: "SUPER + F", label: "Files", command: "nautilus", unbind: true },
  { keys: "SUPER + SHIFT + B", unbind: true },
]);
assert(
  bindApplied.indexOf("-- Keep only your personal keybinding overrides here.") !== -1,
  "applyFile keeps binding comments",
);
assert(
  bindApplied.indexOf('o.bind("SUPER + D", "Desks"') !== -1,
  "applyFile keeps unmanaged binds",
);
assert(
  bindApplied.indexOf('hl.unbind("SUPER + F")') !== -1,
  "applyFile writes unbind before a replacement",
);
assert(
  bindApplied.indexOf('o.bind("SUPER + F", "Files", "nautilus")') !== -1,
  "applyFile writes a managed bind",
);
assert(
  bindApplied.indexOf('hl.unbind("SUPER + SHIFT + B")') !== -1,
  "applyFile writes an unbind-only row",
);
const bindParsed = binds.parseFile(bindApplied);
assertEqual(
  bindParsed.filter(function (row) {
    return row.managed;
  }).length,
  2,
  "parseFile marks managed bindings",
);
assertEqual(
  bindParsed.filter(function (row) {
    return !row.managed && row.keys === "SUPER + D";
  }).length,
  1,
  "parseFile keeps unmanaged bindings",
);
const printed = binds.parsePrint(
  "SUPER + Q                         \u2192 Close window\nSUPER + D                         \u2192 Desks\n",
);
assertEqual(printed.length, 2, "parsePrint reads display lines");
assertEqual(printed[0].action, "Close window", "parsePrint reads the action");
assertEqual(
  binds.catalogConflict(printed, "SUPER + Q"),
  "Close window",
  "catalogConflict finds a taken chord",
);
assertEqual(binds.catalogConflict(printed, "SUPER + Z"), "", "catalogConflict misses a free chord");
assertEqual(
  binds.catalogConflict(printed, "SUPER + Q\n"),
  "",
  "catalogConflict rejects a newline chord",
);
const asciiPrint = binds.parsePrint("SUPER + Return -> Terminal\nno arrow here\n");
assertEqual(asciiPrint.length, 1, "parsePrint reads an ASCII arrow");
assertEqual(asciiPrint[0].keys, "SUPER + Return", "parsePrint ASCII keys");
assertEqual(asciiPrint[0].action, "Terminal", "parsePrint ASCII action");
assertEqual(binds.sanitizeLabel(" Files "), "Files", "sanitizeLabel trims");
assertEqual(binds.sanitizeLabel("bad\nlabel"), "", "sanitizeLabel rejects a newline");
assertEqual(binds.sanitizeKeys("SUPER  +   Q"), "SUPER + Q", "sanitizeKeys collapses spaces");
assertEqual(binds.sanitizeKeys("A".repeat(65)), "", "sanitizeKeys rejects a chord over 64 chars");
assertEqual(
  binds.sanitizeKeys("SUPER + @"),
  "",
  "sanitizeKeys rejects punctuation outside the charset",
);
assertEqual(binds.sanitizeLabel("x".repeat(65)), "", "sanitizeLabel rejects a label over 64 chars");
const nilBind = binds.serialize([{ keys: "SUPER + X", command: "foot" }]);
assert(
  nilBind.indexOf('o.bind("SUPER + X", nil, "foot")') !== -1,
  "serialize uses nil when the label is empty",
);
const managedBinds = binds.managedItems([
  { keys: "SUPER + D", command: "desks", managed: false },
  { keys: "SUPER + F", command: "nautilus", managed: true },
  { keys: "SUPER + Q" },
  null,
]);
assertEqual(managedBinds.length, 1, "managedItems drops unmanaged and commandless rows");
assertEqual(managedBinds[0].keys, "SUPER + F", "managedItems keeps a managed bind");
assertEqual(
  binds.commandFromArg({ launch: "foot" }),
  "foot",
  "commandFromArg reads a launch table",
);
assertEqual(binds.commandFromArg({ launch: "bad\ncmd" }), "", "commandFromArg sanitizes launch");
const folded = binds.foldEvents([
  { kind: "unbind", keys: "SUPER + Q" },
  { kind: "bind", keys: "SUPER + Q", label: "Quit", command: "kill" },
  { kind: "unbind", keys: "SUPER + X" },
]);
assertEqual(folded[0].unbind, true, "foldEvents pairs unbind with the next bind");
assertEqual(folded[0].command, "kill", "foldEvents keeps the replacement command");
assertEqual(folded[1].command, "", "foldEvents keeps a lone unbind");
assertEqual(
  binds.parseCalls('o.bind("SUPER + T", nil, { launch = "foot" })')[0].command,
  "foot",
  "parseCalls reads a launch table bind",
);
