// The sandbox is what keeps the heavier tests off the live machine, so it gets
// its own tests: the guard refuses, the recorder keeps argv exact, nothing real
// is on PATH, and the backend wrapper cannot leave its root.
const fs = require("fs");
const os = require("os");
const path = require("path");
const { assert, assertEqual } = require("./harness");
const {
  createSandbox,
  assertHermetic,
  backendWrapper,
  hostWhich,
  UNEXPECTED,
} = require("./sandbox");

function throws(fn) {
  try {
    fn();
  } catch (err) {
    return String(err.message);
  }
  return "";
}

// --- the guard ---------------------------------------------------------------

const outside = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-outside-"));
process.on("exit", () => fs.rmSync(outside, { recursive: true, force: true }));
const fine = (root) => ({
  HOME: path.join(root, "home"),
  XDG_CONFIG_HOME: path.join(root, "home", ".config"),
  XDG_RUNTIME_DIR: path.join(root, "run"),
  PATH: path.join(root, "bin"),
});

assertEqual(
  throws(() => assertHermetic(fine(outside), outside)),
  "",
  "a contained environment passes",
);
assert(
  throws(() => assertHermetic({ ...fine(outside), HOME: os.homedir() }, outside)).includes(
    "real home",
  ),
  "the guard refuses the real home",
);
assert(
  throws(() => assertHermetic({ ...fine(outside), HOME: "/tmp/elsewhere" }, outside)).includes(
    "outside",
  ),
  "the guard refuses a HOME outside the sandbox",
);
assert(
  throws(() => assertHermetic({ ...fine(outside), XDG_CONFIG_HOME: "/etc" }, outside)).includes(
    "XDG_CONFIG_HOME",
  ),
  "the guard refuses an XDG path outside the sandbox",
);
assert(
  throws(() => assertHermetic({ ...fine(outside), PATH: "/usr/bin" }, outside)).includes(
    "PATH entry",
  ),
  "the guard refuses a real PATH entry",
);
assert(
  throws(() =>
    assertHermetic({ ...fine(outside), PATH: `${outside}/bin:/usr/bin` }, outside),
  ).includes("/usr/bin"),
  "the guard refuses a PATH that only starts in the sandbox",
);
assert(
  throws(() =>
    assertHermetic({ ...fine(outside), OMARCHY_PATH: "/usr/share/omarchy" }, outside),
  ).includes("OMARCHY_PATH"),
  "the guard refuses a real OMARCHY_PATH",
);
assert(
  throws(() => assertHermetic({ PATH: path.join(outside, "bin") }, outside)).includes(
    "HOME is not set",
  ),
  "the guard refuses a missing HOME",
);
const link = path.join(outside, "home-link");
fs.symlinkSync(os.homedir(), link);
assert(
  throws(() => assertHermetic({ ...fine(outside), HOME: link }, outside)).includes("real home"),
  "the guard sees through a symlink to the real home",
);

const box = createSandbox({
  fakes: {
    omarchy: [
      { argv: ["theme", "current"], stdout: "tokyo-night\n" },
      { argv: ["fail"], status: 3, stderr: "nope\n" },
    ],
    sudo: [{ status: 0 }],
  },
});

assertEqual(
  throws(() => box.env()),
  "",
  "the sandbox's own environment passes the guard",
);
assert(
  throws(() => box.env({ HOME: os.homedir() })).includes("real home"),
  "a test cannot slip the real home into a sandbox run",
);
assert(
  throws(() => box.env({ PATH: `${box.bin}:${process.env.PATH}` })).includes("PATH entry"),
  "a test cannot append the real PATH to a sandbox run",
);

// --- nothing real is reachable ----------------------------------------------

const probe = box.spawn(box.bash, [
  "-c",
  'for c in omarchy hyprctl systemctl nmcli sudo hostnamectl loginctl pacman; do printf "%s=%s\\n" "$c" "$(command -v $c || echo -)"; done',
]);
const found = Object.fromEntries(
  probe.out
    .trim()
    .split("\n")
    .map((line) => line.split("=")),
);
for (const name of ["omarchy", "hyprctl", "systemctl", "nmcli", "sudo", "hostnamectl"]) {
  assertEqual(found[name], path.join(box.bin, name), `${name} on the sandbox PATH is the fake`);
}
assertEqual(found.pacman, "-", "a command with no fake is simply absent");
assertEqual(found.loginctl, "-", "a real system tool is absent from the sandbox PATH");

// --- the recorder ------------------------------------------------------------

box.resetCalls();
const answered = box.spawn(path.join(box.bin, "omarchy"), ["theme", "current"]);
assertEqual(answered.out, "tokyo-night\n", "a scripted fake answers on stdout");
assertEqual(answered.status, 0, "a scripted fake exits 0 by default");

const failed = box.spawn(path.join(box.bin, "omarchy"), ["fail", "now"]);
assertEqual(failed.status, 3, "a scripted fake exits with its status");
assertEqual(failed.err, "nope\n", "a scripted fake answers on stderr");

const unexpected = box.spawn(path.join(box.bin, "omarchy"), ["theme", "set", "x"]);
assertEqual(unexpected.status, UNEXPECTED, "an unscripted call exits with the unexpected status");
assert(
  unexpected.err.includes("unexpected call: theme set x"),
  "an unscripted call says what it was",
  unexpected.err,
);

const trapped = box.spawn(path.join(box.bin, "nmcli"), ["connection", "delete", "home"]);
assertEqual(trapped.status, UNEXPECTED, "a trap with no script fails loudly");

const awkward = ["two words", "it's", 'say "hi"', "line\nbreak", "$HOME", "*", "", "--flag=a b"];
box.resetCalls();
box.spawn(path.join(box.bin, "sudo"), awkward, { input: "secret\n" });
const [call] = box.calls("sudo");
assertEqual(
  JSON.stringify(call.argv),
  JSON.stringify(awkward),
  "argv is recorded exactly, quoting and all",
);
assertEqual(call.stdin, null, "stdin is left alone unless a rule asks for it");
assertEqual(box.calls().length, 1, "one call is one record");

box.resetCalls();
box.spawn(path.join(box.bin, "omarchy"), ["theme", "current"]);
box.spawn(path.join(box.bin, "sudo"), ["true"]);
assertEqual(
  JSON.stringify(box.callLines()),
  JSON.stringify(["omarchy theme current", "sudo true"]),
  "calls come back in order",
);
assertEqual(box.calls("omarchy").length, 1, "calls can be narrowed to one command");

// --- asserting what real scripts run -----------------------------------------

{
  // as-root.sh prefers `sudo -n true` as its probe, then runs the command.
  const r = box.run("rollback-snapshot.sh", ["root", "5"]);
  assertEqual(r.status, 0, "rollback-snapshot.sh succeeds with a cooperative sudo");
  assertEqual(
    JSON.stringify(box.callLines()),
    JSON.stringify(["sudo -n true", "sudo snapper -c root rollback 5"]),
    "rollback-snapshot.sh runs exactly snapper -c root rollback 5 under sudo",
  );
}
{
  const r = box.run("rollback-snapshot.sh", ["../root", "5"]);
  assertEqual(r.status, 1, "rollback-snapshot.sh rejects a bad config");
  assertEqual(box.calls().length, 0, "and runs nothing at all");
}
{
  // Without sudo support the script falls back to pkexec.
  const nosudo = createSandbox({
    fakes: { sudo: [{ argv: ["-n", "true"], status: 1 }], pkexec: [{ status: 0 }] },
  });
  const r = nosudo.run("rollback-snapshot.sh", ["root", "7"]);
  assertEqual(r.status, 0, "rollback-snapshot.sh falls back to pkexec");
  assertEqual(
    JSON.stringify(nosudo.callLines("pkexec")),
    JSON.stringify(["pkexec snapper -c root rollback 7"]),
    "the pkexec fallback runs the same command",
  );
  nosudo.cleanup();
}

// --- stdin: a password has to reach the call that wants it -------------------

{
  // With ATMOS_SUDO_ASK, as-root.sh probes with `sudo -n true`, then runs
  // `sudo -S -p '' ...` and the in-app password arrives on stdin. A probe that
  // drained stdin would leave `sudo -S` with nothing.
  const ask = createSandbox({
    fakes: {
      sudo: [
        { argv: ["-n", "true"], status: 1 },
        { argv: ["-S"], stdin: true, status: 0 },
      ],
    },
  });
  const r = ask.run("rollback-snapshot.sh", ["root", "5"], {
    input: "hunter2\n",
    env: { ATMOS_SUDO_ASK: "1" },
  });
  assertEqual(r.status, 0, "rollback-snapshot.sh succeeds through sudo -S");
  const [probe, run] = ask.calls("sudo");
  assertEqual(probe.stdin, null, "the sudo -n true probe does not read stdin");
  assertEqual(
    JSON.stringify(run.argv),
    JSON.stringify(["-S", "-p", "", "snapper", "-c", "root", "rollback", "5"]),
    "the password path runs sudo -S -p '' snapper ...",
  );
  assertEqual(run.stdin, "hunter2\n", "the password reaches sudo -S");
  ask.cleanup();
}

// --- the backend wrapper -----------------------------------------------------

{
  const wrapped = backendWrapper(box.root);
  assert(fs.existsSync(wrapped.file), "the wrapper script exists");
  const version = box.spawn(wrapped.file, ["version"]);
  assertEqual(version.out.trim(), "0.1.0", "the wrapper runs the real ratmos");

  const set = box.spawn(wrapped.file, ["--backend", "plain", "request"], {
    input: JSON.stringify({ op: "settings.set", domain: "theme", value: "tokyo-night" }),
  });
  assertEqual(set.status, 0, "a write through the wrapper succeeds");
  assert(
    fs.existsSync(path.join(wrapped.root, ".config", "plain", "theme.json")),
    "the write lands under the wrapper's root",
  );
  assert(
    !fs.existsSync(path.join(box.home, ".config", "plain")),
    "and nothing lands in the sandbox HOME",
  );

  const moved = box.spawn(wrapped.file, ["--root", "/", "--backend", "plain", "version"]);
  assertEqual(moved.status, 64, "a caller cannot move the pinned root");
  assert(moved.err.includes("--root is pinned"), "and is told why", moved.err);

  // `apply` under a fixture root logs the command and does not run it.
  const marker = path.join(box.root, "ran-for-real");
  const logged = box.spawn(wrapped.file, ["--backend", "plain", "apply", "--", "touch", marker]);
  assertEqual(logged.status, 0, "apply under the wrapper exits 0");
  assert(!fs.existsSync(marker), "apply under the wrapper does not run the command");
  assert(
    fs.readFileSync(path.join(wrapped.root, "commands.log"), "utf8").includes(marker),
    "apply under the wrapper logs the argv instead",
  );

  // The terminator Backend.qml uses for `apply` must not turn the root into an argument.
  const applied = box.spawn(wrapped.file, ["--backend", "plain", "apply", "--", "true"]);
  assert(
    !applied.err.includes("unknown argument"),
    "the root stays a flag when `--` is used",
    applied.err,
  );
}

assert(hostWhich("bash") !== null, "the host has bash to build a sandbox from");
