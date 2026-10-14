// Argument validation for the tasks that change the system. Each case feeds a
// script bad input and checks three things: it exits with the documented
// status, it says why, and it never reached a command that mutates anything.
//
// The tasks run with a throwaway HOME and a private PATH. That PATH holds
// only plain utilities plus trap commands (sudo, snapper, cryptsetup, nmcli,
// omarchy, ...) that record the call and fail, so a validation gap shows up as
// a recorded call instead of a change on the machine running the tests.
const fs = require("fs");
const path = require("path");
const { assert, assertEqual } = require("./harness");
const { createSandbox } = require("./sandbox");

// A throwaway HOME, plus fakes for sudo, snapper, cryptsetup, nmcli, omarchy,
// and the rest: see sandbox.js. An unscripted fake records the call and fails.
const box = createSandbox();
const sandbox = box.root;
const home = box.home;

// Run a script and report what it did, with the fake calls as one string.
function run(script, args, { input = "", env = {} } = {}) {
  const result = box.task(script.replace(/\.sh$/, ""), args, { input, env });
  return { ...result, trapped: box.callLines().join("\n") };
}

// `allow` lists command lines a script may run before it validates; anything
// else recorded in the trap log fails the case.
function rejects(label, script, args, opts, { status, says, allow = [] }) {
  const r = run(script, args, opts);
  assertEqual(r.status, status, `${script} ${label}: exits ${status}`);
  assert(r.err.includes(says), `${script} ${label}: says "${says}"`, r.err);
  const unexpected = r.trapped.split("\n").filter((line) => line && !allow.includes(line));
  assertEqual(unexpected.join("; "), "", `${script} ${label}: reaches no mutating command`);
  return r;
}

// --- rollback-snapshot ----------------------------------------------------

const usage = "Usage: rollback-snapshot <config> <id>";
const rollback = (label, args) =>
  rejects(label, "rollback-snapshot", args, {}, { status: 1, says: usage });
rollback("no arguments", []);
rollback("empty config", ["", "5"]);
rollback("config with a space", ["ro ot", "5"]);
rollback("config with a path", ["../root", "5"]);
rollback("config with a shell fragment", ["root;reboot", "5"]);
rollback("config with a newline", ["root\nroot", "5"]);
rollback("missing id", ["root"]);
rollback("empty id", ["root", ""]);
rollback("non-numeric id", ["root", "abc"]);
rollback("negative id", ["root", "-1"]);
rollback("id with a shell fragment", ["root", "5;reboot"]);
rollback("id with a newline", ["root", "5\n6"]);

// --- luks-change-key ------------------------------------------------------

const luks = (label, lines, says) =>
  rejects(label, "luks-change-key", [], { input: lines.join("\n") + "\n" }, { status: 1, says });
luks("empty device", ["", "old", "new"], "device must be a /dev path");
luks("device outside /dev", ["/etc/shadow", "old", "new"], "device must be a /dev path");
luks("relative device", ["dev/sda1", "old", "new"], "device must be a /dev path");
luks("device with dot-dot", ["/dev/../etc/passwd", "old", "new"], "device must be a /dev path");
luks("device with a space", ["/dev/sd a1", "old", "new"], "unexpected characters");
luks("device with a shell fragment", ["/dev/sda1;reboot", "old", "new"], "unexpected characters");
luks("device with a dollar", ["/dev/$HOME", "old", "new"], "unexpected characters");
luks("empty current passphrase", ["/dev/sda1", "", "new"], "passphrases cannot be empty");
luks("empty new passphrase", ["/dev/sda1", "old", ""], "passphrases cannot be empty");
{
  // blkid is a trap that prints nothing, so no device counts as LUKS. This
  // case is the one that reaches blkid itself, so it is the only one allowed to.
  const r = run("luks-change-key", [], { input: "/dev/sda1\nold\nnew\n" });
  assertEqual(r.status, 1, "luks-change-key: a non-LUKS device exits 1");
  assert(r.err.includes("is not a LUKS device"), "luks-change-key: says it is not LUKS", r.err);
  assert(
    r.trapped.split("\n").every((line) => line.startsWith("blkid ")),
    "luks-change-key: a non-LUKS device only asks blkid",
    r.trapped,
  );
}

// --- enterprise-wifi-connect ----------------------------------------------

const wifi = (label, args, input) =>
  rejects(
    label,
    "enterprise-wifi-connect",
    args,
    { input },
    { status: 1, says: "Usage: enterprise-wifi-connect" },
  );
{
  const noArgs = run("enterprise-wifi-connect", [], { input: "pw\n" });
  assertEqual(noArgs.status, 1, "enterprise-wifi-connect: no arguments exits 1");
  assert(
    noArgs.err.includes("Usage: enterprise-wifi-connect"),
    "enterprise-wifi-connect: prints usage",
  );
  assertEqual(noArgs.trapped, "", "enterprise-wifi-connect: no arguments reaches nmcli");
}
wifi("empty ssid", ["", "me@corp"], "pw\n");
wifi("empty identity", ["corp", ""], "pw\n");
wifi("missing identity", ["corp"], "pw\n");
{
  const empty = run("enterprise-wifi-connect", ["corp", "me@corp"], { input: "\n" });
  assertEqual(empty.status, 1, "enterprise-wifi-connect: an empty password exits 1");
  assert(empty.err.includes("password cannot be empty"), "enterprise-wifi-connect: says why");
  assertEqual(empty.trapped, "", "enterprise-wifi-connect: an empty password reaches nmcli");
  const none = run("enterprise-wifi-connect", ["corp", "me@corp"], { input: "" });
  assertEqual(none.status, 1, "enterprise-wifi-connect: no password line exits 1");
  assertEqual(none.trapped, "", "enterprise-wifi-connect: no password line reaches nmcli");
}

// --- diag-report ----------------------------------------------------------

const diagCache = path.join(home, ".cache", "atmos", "diagnostic-report.txt");
rejects(
  "unknown mode",
  "diag-report",
  ["upload"],
  { input: "head\n" },
  {
    status: 2,
    says: "usage: diag-report",
  },
);
for (const [label, dest] of [
  ["save without a path", ""],
  ["save with a relative path", "report.txt"],
  ["save with a newline in the path", "/tmp/a\nb"],
  ["save with a carriage return in the path", "/tmp/a\rb"],
]) {
  rejects(
    label,
    "diag-report",
    ["save", dest],
    { input: "head\n" },
    {
      status: 1,
      says: "save needs an absolute path",
    },
  );
}
{
  const dest = path.join(sandbox, "saved.txt");
  const r = run("diag-report", ["save", dest], { input: "ATMOS HEADER" });
  assertEqual(r.status, 0, "diag-report: save to an absolute path exits 0");
  assertEqual(r.out.trim(), dest, "diag-report: save prints the destination");
  assert(
    fs.readFileSync(dest, "utf8").startsWith("ATMOS HEADER\n"),
    "diag-report: save writes the header with a final newline first",
  );
  assert(fs.existsSync(diagCache), "diag-report: the report is cached under XDG_CACHE_HOME");
}

// --- set-presentation -----------------------------------------------------

{
  const state = path.join(sandbox, "presentation.json");
  const env = { ATMOS_PRESENTATION_FILE: state };
  for (const args of [[], ["maybe"], ["ON"], ["on;off"]]) {
    const r = run("set-presentation", args, { env });
    assertEqual(r.status, 2, `set-presentation [${args}]: exits 2`);
    assert(
      r.err.includes("usage: set-presentation on|off"),
      "set-presentation: prints usage",
      r.err,
    );
    assertEqual(r.trapped, "", `set-presentation [${args}]: reaches no omarchy/jq call`);
    assert(!fs.existsSync(state), `set-presentation [${args}]: leaves no state file`);
  }
}

// --- set-favorites --------------------------------------------------------

{
  const file = path.join(sandbox, "favorites.json");
  const env = { ATMOS_FAVORITES_FILE: file };
  const bad = [
    ["no arguments", [], 2, "usage: set-favorites write"],
    ["wrong verb", ["read", "[]"], 2, "usage: set-favorites write"],
    ["missing json", ["write"], 2, "usage: set-favorites write"],
    ["too many arguments", ["write", "[]", "x"], 2, "usage: set-favorites write"],
    ["invalid json", ["write", "{nope"], 2, ""],
    ["json that is not a list", ["write", '{"items": "x"}'], 2, ""],
    ["a bare string", ["write", '"x"'], 2, ""],
    ["a number", ["write", "3"], 2, ""],
  ];
  for (const [label, args, status, says] of bad) {
    const r = run("set-favorites", args, { env });
    assertEqual(r.status, status, `set-favorites ${label}: exits ${status}`);
    assert(r.err.includes(says), `set-favorites ${label}: says "${says}"`, r.err);
    assert(!fs.existsSync(file), `set-favorites ${label}: writes nothing`);
  }
  const bare = run("set-favorites", ["write", '["a","b"]'], { env });
  assertEqual(bare.status, 0, "set-favorites: a bare list is accepted");
  assertEqual(
    JSON.stringify(JSON.parse(fs.readFileSync(file, "utf8"))),
    '{"items":["a","b"]}',
    "set-favorites: a bare list is stored under items",
  );
  const wrapped = run("set-favorites", ["write", '{"items":["c"],"extra":1}'], { env });
  assertEqual(wrapped.status, 0, "set-favorites: an object with items is accepted");
  assertEqual(
    JSON.stringify(JSON.parse(fs.readFileSync(file, "utf8"))),
    '{"items":["c"]}',
    "set-favorites: only items survive a write",
  );
  const left = fs.readdirSync(sandbox).filter((name) => name.startsWith(".favorites."));
  assertEqual(left.length, 0, "set-favorites: no temp file is left behind");
}
