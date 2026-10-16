const { load, assert, assertEqual } = require("./harness");

const lua = load("services/LuaText.js");

assertEqual(lua.luaString('a\nb"'), '"a\\nb\\""', "luaString escapes a newline and a quote");
assertEqual(lua.luaString("a\tb\r"), '"a\\tb\\r"', "luaString escapes a tab and a carriage return");
assertEqual(lua.luaNumber(8), "8", "luaNumber writes an integer");
assertEqual(lua.luaNumber(0.97), "0.97", "luaNumber trims trailing zeros");
assertEqual(lua.luaNumber(0.4), "0.4", "luaNumber keeps a short fraction");
assertEqual(lua.luaNumber(-0.5), "-0.5", "luaNumber keeps a negative fraction");
assertEqual(lua.luaNumber(NaN), "0", "luaNumber treats NaN as zero");

const raw = 'a "quoted"\\ line\nnext\ttab';
assertEqual(lua.unescapeLua(lua.luaString(raw).slice(1, -1)), raw, "unescapeLua undoes luaString");

const quotedDash = 'o.bind("SUPER -- T", nil)';
assert(
  lua.inLineComment(quotedDash, quotedDash.indexOf("nil")) === false,
  "a -- inside quotes is not a comment",
);
const lineComment = 'o.bind("SUPER + T", nil)\n-- o.bind("NO", nil)\n';
assert(
  lua.inLineComment(lineComment, lineComment.lastIndexOf("o.bind")) === true,
  "a line comment comments out the rest of its line",
);

const bind = 'o.bind("SUPER + T", nil, { launch = "foot" })';
const bindArgs = lua.parseCallArgs(bind, bind.indexOf("(") + 1);
assertEqual(bindArgs.args[0], "SUPER + T", "parseCallArgs reads the key string");
assert(bindArgs.args[1] === null, "parseCallArgs reads nil");
assertEqual(bindArgs.args[2].launch, "foot", "parseCallArgs reads a named table field");

const win = 'o.window("foo\\"bar", { float = true })';
const winArgs = lua.parseCallArgs(win, win.indexOf("(") + 1);
assertEqual(winArgs.args[0], 'foo"bar', "parseCallArgs unescapes a quote in a string");
assert(winArgs.args[1].float === true, "parseCallArgs reads a bool table field");

const inserted = lua.replaceSentinel("keep\n", "-- b", "-- e", "-- b\nline\n-- e");
assert(
  inserted.indexOf("keep") !== -1 && inserted.indexOf("line") !== -1,
  "replaceSentinel appends when the markers are missing",
);
const replaced = lua.replaceSentinel(
  "before\n-- b\nold\n-- e\nafter\n",
  "-- b",
  "-- e",
  "-- b\nnew\n-- e",
);
assert(
  replaced.indexOf("old") === -1 &&
    replaced.indexOf("new") !== -1 &&
    replaced.indexOf("before") !== -1 &&
    replaced.indexOf("after") !== -1,
  "replaceSentinel replaces the block and keeps the text around it",
);
assert(
  lua.sentinelBounds("only -- b", "-- b", "-- e") === null,
  "a missing end marker returns null",
);
