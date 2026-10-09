// Argument validation for scripts that change the system. Each case feeds a
// script bad input and checks three things: it exits with the documented
// status, it says why, and it never reached a command that mutates anything.
//
// The scripts run with a throwaway HOME and a private PATH. That PATH holds
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
const bin = box.bin;
assert(fs.existsSync(path.join(bin, "python3")), "the validation sandbox has python3");

// Run a script and report what it did, with the fake calls as one string.
function run(script, args, { input = "", env = {} } = {}) {
  const result = box.run(script, args, { input, env });
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

// --- rollback-snapshot.sh ----------------------------------------------------

const usage = "Usage: rollback-snapshot.sh <config> <id>";
const rollback = (label, args) =>
  rejects(label, "rollback-snapshot.sh", args, {}, { status: 1, says: usage });
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

// --- luks-change-key.sh ------------------------------------------------------

const luks = (label, lines, says) =>
  rejects(label, "luks-change-key.sh", [], { input: lines.join("\n") + "\n" }, { status: 1, says });
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
  const r = run("luks-change-key.sh", [], { input: "/dev/sda1\nold\nnew\n" });
  assertEqual(r.status, 1, "luks-change-key.sh: a non-LUKS device exits 1");
  assert(r.err.includes("is not a LUKS device"), "luks-change-key.sh: says it is not LUKS", r.err);
  assert(
    r.trapped.split("\n").every((line) => line.startsWith("blkid ")),
    "luks-change-key.sh: a non-LUKS device only asks blkid",
    r.trapped,
  );
}

// --- enterprise-wifi-connect.sh ----------------------------------------------

const wifi = (label, args, input) =>
  rejects(
    label,
    "enterprise-wifi-connect.sh",
    args,
    { input },
    { status: 1, says: "Usage: enterprise-wifi-connect.sh" },
  );
{
  const noArgs = run("enterprise-wifi-connect.sh", [], { input: "pw\n" });
  assertEqual(noArgs.status, 1, "enterprise-wifi-connect.sh: no arguments exits 1");
  assert(
    noArgs.err.includes("Usage: enterprise-wifi-connect.sh"),
    "enterprise-wifi-connect.sh: prints usage",
  );
  assertEqual(noArgs.trapped, "", "enterprise-wifi-connect.sh: no arguments reaches nmcli");
}
wifi("empty ssid", ["", "me@corp"], "pw\n");
wifi("empty identity", ["corp", ""], "pw\n");
wifi("missing identity", ["corp"], "pw\n");
{
  const empty = run("enterprise-wifi-connect.sh", ["corp", "me@corp"], { input: "\n" });
  assertEqual(empty.status, 1, "enterprise-wifi-connect.sh: an empty password exits 1");
  assert(empty.err.includes("password cannot be empty"), "enterprise-wifi-connect.sh: says why");
  assertEqual(empty.trapped, "", "enterprise-wifi-connect.sh: an empty password reaches nmcli");
  const none = run("enterprise-wifi-connect.sh", ["corp", "me@corp"], { input: "" });
  assertEqual(none.status, 1, "enterprise-wifi-connect.sh: no password line exits 1");
  assertEqual(none.trapped, "", "enterprise-wifi-connect.sh: no password line reaches nmcli");
}

// --- diag-report.sh ----------------------------------------------------------

const diagCache = path.join(home, ".cache", "atmos", "diagnostic-report.txt");
rejects(
  "unknown mode",
  "diag-report.sh",
  ["upload"],
  { input: "head\n" },
  {
    status: 2,
    says: "usage: diag-report.sh",
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
    "diag-report.sh",
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
  const r = run("diag-report.sh", ["save", dest], { input: "ATMOS HEADER" });
  assertEqual(r.status, 0, "diag-report.sh: save to an absolute path exits 0");
  assertEqual(r.out.trim(), dest, "diag-report.sh: save prints the destination");
  assert(
    fs.readFileSync(dest, "utf8").startsWith("ATMOS HEADER\n"),
    "diag-report.sh: save writes the header with a final newline first",
  );
  assert(fs.existsSync(diagCache), "diag-report.sh: the report is cached under XDG_CACHE_HOME");
}

// --- set-presentation.sh -----------------------------------------------------

{
  const state = path.join(sandbox, "presentation.json");
  const env = { ATMOS_PRESENTATION_FILE: state };
  for (const args of [[], ["maybe"], ["ON"], ["on;off"]]) {
    const r = run("set-presentation.sh", args, { env });
    assertEqual(r.status, 2, `set-presentation.sh [${args}]: exits 2`);
    assert(
      r.err.includes("usage: set-presentation.sh on|off"),
      "set-presentation.sh: prints usage",
      r.err,
    );
    assertEqual(r.trapped, "", `set-presentation.sh [${args}]: reaches no omarchy/jq call`);
    assert(!fs.existsSync(state), `set-presentation.sh [${args}]: leaves no state file`);
  }
}

// --- set-favorites.sh --------------------------------------------------------

{
  const file = path.join(sandbox, "favorites.json");
  const env = { ATMOS_FAVORITES_FILE: file };
  const bad = [
    ["no arguments", [], 2, "usage: set-favorites.sh write"],
    ["wrong verb", ["read", "[]"], 2, "usage: set-favorites.sh write"],
    ["missing json", ["write"], 2, "usage: set-favorites.sh write"],
    ["too many arguments", ["write", "[]", "x"], 2, "usage: set-favorites.sh write"],
    ["invalid json", ["write", "{nope"], 2, ""],
    ["json that is not a list", ["write", '{"items": "x"}'], 2, ""],
    ["a bare string", ["write", '"x"'], 2, ""],
    ["a number", ["write", "3"], 2, ""],
  ];
  for (const [label, args, status, says] of bad) {
    const r = run("set-favorites.sh", args, { env });
    assertEqual(r.status, status, `set-favorites.sh ${label}: exits ${status}`);
    assert(r.err.includes(says), `set-favorites.sh ${label}: says "${says}"`, r.err);
    assert(!fs.existsSync(file), `set-favorites.sh ${label}: writes nothing`);
  }
  const bare = run("set-favorites.sh", ["write", '["a","b"]'], { env });
  assertEqual(bare.status, 0, "set-favorites.sh: a bare list is accepted");
  assertEqual(
    JSON.stringify(JSON.parse(fs.readFileSync(file, "utf8"))),
    '{"items":["a","b"]}',
    "set-favorites.sh: a bare list is stored under items",
  );
  const wrapped = run("set-favorites.sh", ["write", '{"items":["c"],"extra":1}'], { env });
  assertEqual(wrapped.status, 0, "set-favorites.sh: an object with items is accepted");
  assertEqual(
    JSON.stringify(JSON.parse(fs.readFileSync(file, "utf8"))),
    '{"items":["c"]}',
    "set-favorites.sh: only items survive a write",
  );
  const left = fs.readdirSync(sandbox).filter((name) => name.startsWith(".favorites."));
  assertEqual(left.length, 0, "set-favorites.sh: no temp file is left behind");
}

// --- set-idle.sh -------------------------------------------------------------

for (const [label, args] of [
  ["no arguments", []],
  ["empty screensaver", ["", "300"]],
  ["negative screensaver", ["-1", "300"]],
  ["fractional screensaver", ["1.5", "300"]],
  ["screensaver with a shell fragment", ["5;reboot", "300"]],
]) {
  rejects(label, "set-idle.sh", args, {}, { status: 1, says: "screensaver seconds" });
}
for (const [label, args] of [
  ["missing lock", ["300"]],
  ["empty lock", ["300", ""]],
  ["negative lock", ["300", "-5"]],
  ["lock with letters", ["300", "soon"]],
]) {
  rejects(label, "set-idle.sh", args, {}, { status: 1, says: "lock seconds" });
}
{
  // Valid numbers, but omarchy-shell-config is a trap here, so a trap on PATH
  // must not count as the helper being usable: the script only checks that it
  // resolves, so remove it for this case.
  const gone = path.join(bin, "omarchy-shell-config");
  fs.rmSync(gone);
  const r = run("set-idle.sh", ["300", "600"]);
  assertEqual(r.status, 1, "set-idle.sh: a missing omarchy-shell-config exits 1");
  assert(
    r.err.includes("omarchy-shell-config is not on PATH"),
    "set-idle.sh: says what is missing",
    r.err,
  );
  assertEqual(r.trapped, "", "set-idle.sh: a missing helper reaches no command");
}
