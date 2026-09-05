const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
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
  "uwsm-app -- foot",
  "commandFromArg wraps launch with uwsm-app",
);
assertEqual(binds.commandFromArg({ launch: "bad\ncmd" }), "", "commandFromArg sanitizes launch");
assertEqual(
  binds.commandFromArg({ omarchy: "browser" }),
  "omarchy-launch-browser",
  "commandFromArg expands an omarchy table",
);
assertEqual(
  binds.commandFromArg({ tui: "btop" }),
  "omarchy-launch-tui 'btop'",
  "commandFromArg expands a tui table",
);
assertEqual(
  binds.commandFromArg({ tui: "cliamp", focus: true }),
  "omarchy-launch-or-focus-tui 'cliamp'",
  "commandFromArg expands a focused tui table",
);
assertEqual(
  binds.commandFromArg({ webapp: "https://x.com/" }),
  "omarchy-launch-webapp 'https://x.com/'",
  "commandFromArg expands a webapp table",
);
assertEqual(
  binds.commandFromArg({ webapp: "https://web.whatsapp.com/", focus: true }, "WhatsApp"),
  "omarchy-launch-or-focus-webapp 'WhatsApp' 'https://web.whatsapp.com/'",
  "commandFromArg expands a focused webapp with the bind label",
);
assertEqual(
  binds.commandFromArg({ launch: "obsidian", focus: "^obsidian$" }),
  "omarchy-launch-or-focus '^obsidian$' 'uwsm-app -- obsidian'",
  "commandFromArg expands launch-or-focus",
);
assertEqual(
  binds.commandFromArg({ omarchy: "terminal", launch: "foot" }),
  "omarchy-launch-terminal",
  "commandFromArg prefers omarchy over launch",
);
assertEqual(binds.shellQuote("it's"), "'it'\\''s'", "shellQuote POSIX-quotes an apostrophe");
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
  "uwsm-app -- foot",
  "parseCalls reads a launch table bind",
);
const omarchyBindings = `
o.bind("SUPER + D", "Desks", "omarchy-shell shell toggle com.mdtrr.omadesk")

-- Add a new binding.
-- o.bind("SUPER + SHIFT + R", "SSH", "alacritty -e ssh your-server")

-- hl.unbind("SUPER + SPACE")
-- o.bind("SUPER + SPACE", "Omarchy menu", "omarchy-menu toggle root")

-- hl.unbind("SUPER + SHIFT + B")
o.bind("SUPER + T", "Term", "kitty") -- live
`;
const liveBinds = binds.parseFile(omarchyBindings);
assertEqual(liveBinds.length, 2, "parseFile skips Omarchy's commented binding examples");
assertEqual(liveBinds[0].keys, "SUPER + D", "parseFile keeps a live bind before comments");
assertEqual(liveBinds[1].keys, "SUPER + T", "parseFile keeps a bind with a trailing comment");
assertEqual(
  liveBinds.filter(function (row) {
    return row.keys === "SUPER + SPACE" || row.keys === "SUPER + SHIFT + R";
  }).length,
  0,
  "parseFile ignores commented o.bind and hl.unbind examples",
);

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
  JSON.stringify(pythonList("bindings", omarchyBindings)),
  JSON.stringify(liveBinds),
  "hypr-sentinel.py bindings list matches parseFile on commented examples",
);

const tableBinds = `
o.bind("SUPER + RETURN", "Terminal", { omarchy = "terminal" })
o.bind("SUPER + SHIFT + O", "Obsidian", { launch = "obsidian", focus = "^obsidian$" })
o.bind("SUPER + SHIFT + A", "ChatGPT", { webapp = "https://chatgpt.com" })
o.bind("SUPER + SHIFT + ALT + G", "WhatsApp", { webapp = "https://web.whatsapp.com/", focus = true })
o.bind("SUPER + CTRL + T", "Activity", { tui = "btop" })
o.bind("SUPER + SHIFT + ALT + M", "Music TUI", { tui = "cliamp", focus = true })
`;
const tableParsed = binds.parseFile(tableBinds);
assertEqual(tableParsed.length, 6, "parseFile reads Omarchy table-form binds");
assertEqual(tableParsed[0].command, "omarchy-launch-terminal", "parseFile expands { omarchy }");
assertEqual(
  tableParsed[1].command,
  "omarchy-launch-or-focus '^obsidian$' 'uwsm-app -- obsidian'",
  "parseFile expands { launch, focus }",
);
assertEqual(
  tableParsed[2].command,
  "omarchy-launch-webapp 'https://chatgpt.com'",
  "parseFile expands { webapp }",
);
assertEqual(
  tableParsed[3].command,
  "omarchy-launch-or-focus-webapp 'WhatsApp' 'https://web.whatsapp.com/'",
  "parseFile expands { webapp, focus } with the label",
);
assertEqual(tableParsed[4].command, "omarchy-launch-tui 'btop'", "parseFile expands { tui }");
assertEqual(
  tableParsed[5].command,
  "omarchy-launch-or-focus-tui 'cliamp'",
  "parseFile expands { tui, focus }",
);
assertEqual(
  JSON.stringify(pythonList("bindings", tableBinds)),
  JSON.stringify(tableParsed),
  "hypr-sentinel.py bindings list matches parseFile on table-form binds",
);

const innerCommentBinds = `
o.bind("SUPER + RETURN", "Terminal", {
  -- Omarchy helper
  omarchy = "terminal"
})
o.bind(
  "SUPER + T", -- chord
  "Term",
  "kitty"
)
hint = "flags --help"; o.bind("SUPER + H", "Help", "man")
`;
const innerParsed = binds.parseFile(innerCommentBinds);
assertEqual(innerParsed.length, 3, "parseFile keeps binds with comments inside the call");
assertEqual(
  innerParsed[0].command,
  "omarchy-launch-terminal",
  "parseFile skips a comment in a bind table",
);
assertEqual(innerParsed[1].keys, "SUPER + T", "parseFile skips a comment between bind args");
assertEqual(innerParsed[2].keys, "SUPER + H", "parseFile keeps a bind after -- inside a string");
assertEqual(
  JSON.stringify(pythonList("bindings", innerCommentBinds)),
  JSON.stringify(innerParsed),
  "hypr-sentinel.py bindings list matches parseFile on comments inside calls",
);

assertEqual(
  binds.parseCalls('o.bind("SUPER + T", "Term", "printf hello\\nworld")')[0].command,
  "",
  "parseCalls treats a Lua \\n in the command as a newline and drops it",
);
assertEqual(
  binds.parseCalls('o.bind("SUPER + T", "Term", "printf hello\\\\nworld")')[0].command,
  "printf hello\\nworld",
  "parseCalls keeps a Lua \\\\n as a literal backslash-n",
);
assertEqual(
  binds.parseCalls('o.bind("SUPER + T", "Term", "printf a\\tb")')[0].command,
  "printf a\tb",
  "parseCalls turns a Lua \\t in the command into a tab",
);
assertEqual(binds.luaString("a\tb\nc"), '"a\\tb\\nc"', "luaString escapes a tab and a newline");
const luaEscapesBinds = `
o.bind("SUPER + N", "Bad", "printf hello\\nworld")
o.bind("SUPER + T", "Term", "printf hello\\\\nworld")
o.bind("SUPER + B", "Tab", "printf a\\tb")
`;
const luaEscapesBindParsed = binds.parseFile(luaEscapesBinds);
assertEqual(
  luaEscapesBindParsed.filter(function (row) {
    return row.command;
  }).length,
  2,
  "parseFile drops a bind whose Lua \\n is a newline",
);
assertEqual(
  luaEscapesBindParsed.filter(function (row) {
    return row.keys === "SUPER + T";
  })[0].command,
  "printf hello\\nworld",
  "parseFile keeps a bind with a literal backslash-n",
);
assertEqual(
  luaEscapesBindParsed.filter(function (row) {
    return row.keys === "SUPER + B";
  })[0].command,
  "printf a\tb",
  "parseFile keeps a bind with a Lua tab",
);
assertEqual(
  JSON.stringify(pythonList("bindings", luaEscapesBinds)),
  JSON.stringify(luaEscapesBindParsed),
  "hypr-sentinel.py bindings list matches parseFile on Lua string escapes",
);
