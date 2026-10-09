// A sandbox for tests that run real scripts or the real ratmos binary.
//
// The machine running the tests is somebody's live desktop, so nothing above
// unit level may reach it. A sandbox gives a test:
//   - a throwaway HOME and XDG tree,
//   - a private PATH holding plain utilities plus fakes for the commands that
//     change or inspect the system (sudo, omarchy, hyprctl, nmcli, ...),
//   - a record of every fake call with its exact argv, so a test asserts what
//     a script ran and not only what it printed,
//   - a guard that throws if anything points back at the real home or at a
//     real tool.
//
// Every fake records its call. A fake with no scripted answer exits 97, so an
// unexpected command is loud. A scripted answer is a list of rules; the first
// rule whose argv prefix matches wins. A rule is { argv?, status?, stdout?,
// stderr?, stdin? }; `stdin: true` makes the fake read and record its stdin,
// which it otherwise leaves alone.
//
//   const box = createSandbox({
//     fakes: { sudo: [{ argv: ["-n", "true"], status: 0 }, { status: 0 }] },
//   });
//   const r = box.run("rollback-snapshot.sh", ["root", "5"]);
//   box.calls("sudo");  // [{ cmd: "sudo", argv: ["snapper", "-c", "root", ...] }]
//   box.cleanup();
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const repo = path.join(__dirname, "..");
const scriptsDir = path.join(repo, "scripts");

// Plain tools a script needs before it reaches the part under test. Anything
// not listed here is not on the sandbox PATH.
const PLAIN_TOOLS = [
  "bash",
  "sh",
  "env",
  "dirname",
  "basename",
  "cat",
  "mkdir",
  "mktemp",
  "rm",
  "cp",
  "mv",
  "ln",
  "ls",
  "touch",
  "chmod",
  "head",
  "tail",
  "tr",
  "sed",
  "grep",
  "sort",
  "awk",
  "id",
  "flock",
  "date",
  "sleep",
  "readlink",
  "python3",
];

// Commands that must never run for real from a test. They are fakes by
// default, so a script that reaches one records the call and fails.
const DEFAULT_TRAPS = [
  "sudo",
  "pkexec",
  "snapper",
  "cryptsetup",
  "nmcli",
  "uuidgen",
  "omarchy",
  "omarchy-shell",
  "omarchy-shell-config",
  "omarchy-debug",
  "hyprctl",
  "systemctl",
  "timedatectl",
  "hostnamectl",
  "localectl",
  "wl-copy",
  "blkid",
  "pkill",
  "jq",
];

const UNEXPECTED = 97;

// Looks up a program on the PATH of the process running the tests.
function hostWhich(name) {
  for (const dir of (process.env.PATH || "").split(":")) {
    if (!dir) continue;
    const full = path.join(dir, name);
    try {
      fs.accessSync(full, fs.constants.X_OK);
      if (fs.statSync(full).isFile()) return full;
    } catch {}
  }
  return null;
}

function within(child, parent) {
  const rel = path.relative(parent, child);
  return rel === "" || (!rel.startsWith("..") && !path.isAbsolute(rel));
}

// Throws unless `env` is a sandbox environment: HOME and every XDG path inside
// the sandbox, HOME not the real one, and PATH made only of sandbox directories.
function assertHermetic(env, boxRoot) {
  const real = fs.realpathSync(os.homedir());
  const problems = [];
  const home = env.HOME ? path.resolve(env.HOME) : "";
  if (!home) problems.push("HOME is not set");
  else if (fs.existsSync(home) && fs.realpathSync(home) === real) {
    problems.push(`HOME is the real home ${real}`);
  } else if (!within(home, boxRoot)) problems.push(`HOME ${home} is outside the sandbox`);
  for (const key of Object.keys(env)) {
    if (!/^XDG_.*_(HOME|DIR)$/.test(key) && key !== "XDG_RUNTIME_DIR") continue;
    if (!within(path.resolve(env[key]), boxRoot)) {
      problems.push(`${key}=${env[key]} is outside the sandbox`);
    }
  }
  for (const dir of String(env.PATH || "").split(":")) {
    if (dir && !within(path.resolve(dir), boxRoot)) {
      problems.push(`PATH entry ${dir} is outside the sandbox`);
    }
  }
  if (env.OMARCHY_PATH && !within(path.resolve(env.OMARCHY_PATH), boxRoot)) {
    problems.push(`OMARCHY_PATH=${env.OMARCHY_PATH} is outside the sandbox`);
  }
  if (problems.length) {
    throw new Error(`refusing to run outside the sandbox:\n  ${problems.join("\n  ")}`);
  }
}

// The fake that every fake command execs: record the call, then answer from
// the scripted rules or exit UNEXPECTED.
const RECORDER = `#!${process.execPath}
const fs = require("fs");
const cmd = process.argv[2];
const argv = process.argv.slice(3);
const dir = process.env.ATMOS_FAKE_DIR;
let rules = [];
try { rules = JSON.parse(fs.readFileSync(dir + "/rules.json", "utf8"))[cmd] || []; } catch {}
const rule = rules.find((r) => !r.argv || r.argv.every((word, i) => argv[i] === word));
// stdin is read only when the rule asks: a real sudo -n true reads nothing, and a
// fake that drained it would eat a password meant for the next call.
let stdin = null;
if (rule && rule.stdin === true) {
  try { stdin = fs.readFileSync(0, "utf8"); } catch { stdin = ""; }
}
fs.appendFileSync(dir + "/calls.jsonl", JSON.stringify({ cmd, argv, stdin }) + "\\n");
if (!rule) {
  process.stderr.write("fake " + cmd + ": unexpected call: " + argv.join(" ") + "\\n");
  process.exit(${UNEXPECTED});
}
if (rule.stdout) process.stdout.write(rule.stdout);
if (rule.stderr) process.stderr.write(rule.stderr);
process.exit(rule.status || 0);
`;

function createSandbox({ fakes = {}, traps = DEFAULT_TRAPS, env = {}, tools = PLAIN_TOOLS } = {}) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "atmos-sandbox-")));
  const home = path.join(root, "home");
  const bin = path.join(root, "bin");
  const fakeDir = path.join(root, "fake");
  for (const dir of [home, bin, fakeDir, path.join(root, "run")]) {
    fs.mkdirSync(dir, { recursive: true });
  }
  fs.chmodSync(path.join(root, "run"), 0o700);

  for (const name of tools) {
    const real = hostWhich(name);
    if (real) fs.symlinkSync(fs.realpathSync(real), path.join(bin, name));
  }

  const recorder = path.join(fakeDir, "recorder.js");
  fs.writeFileSync(recorder, RECORDER, { mode: 0o755 });
  const rules = {};
  function fake(name, scripted) {
    const file = path.join(bin, name);
    fs.rmSync(file, { force: true });
    fs.writeFileSync(file, `#!/bin/sh\nexec "${recorder}" "${name}" "$@"\n`, { mode: 0o755 });
    rules[name] = scripted || [];
    fs.writeFileSync(path.join(fakeDir, "rules.json"), JSON.stringify(rules));
  }
  for (const name of traps) fake(name, []);
  for (const [name, scripted] of Object.entries(fakes)) fake(name, scripted);
  fs.writeFileSync(path.join(fakeDir, "rules.json"), JSON.stringify(rules));

  const baseEnv = {
    PATH: bin,
    HOME: home,
    XDG_CONFIG_HOME: path.join(home, ".config"),
    XDG_CACHE_HOME: path.join(home, ".cache"),
    XDG_STATE_HOME: path.join(home, ".local", "state"),
    XDG_DATA_HOME: path.join(home, ".local", "share"),
    XDG_RUNTIME_DIR: path.join(root, "run"),
    ATMOS_FAKE_DIR: fakeDir,
    LANG: "C",
  };
  const callsFile = path.join(fakeDir, "calls.jsonl");

  const box = {
    root,
    home,
    bin,
    bash: hostWhich("bash"),
    fake,
    // The environment to give a child process, checked by the guard.
    env(extra = {}) {
      const merged = { ...baseEnv, ...env, ...extra };
      assertHermetic(merged, root);
      return merged;
    },
    // Every fake call so far, oldest first; `cmd` narrows it to one command.
    calls(cmd) {
      if (!fs.existsSync(callsFile)) return [];
      const all = fs
        .readFileSync(callsFile, "utf8")
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line));
      return cmd ? all.filter((call) => call.cmd === cmd) : all;
    },
    // Calls as "cmd arg arg" lines, for quick comparisons.
    callLines(cmd) {
      return box.calls(cmd).map((call) => [call.cmd, ...call.argv].join(" "));
    },
    resetCalls() {
      fs.rmSync(callsFile, { force: true });
    },
    // Run a script from scripts/ under bash, with the sandbox environment.
    run(script, args = [], { input = "", env: extra = {}, keepCalls = false } = {}) {
      if (!keepCalls) box.resetCalls();
      const result = spawnSync(box.bash, [path.join(scriptsDir, script), ...args], {
        input,
        encoding: "utf8",
        timeout: 20000,
        env: box.env(extra),
      });
      return { status: result.status, out: result.stdout, err: result.stderr };
    },
    // Run any program by path with the sandbox environment.
    spawn(file, args = [], { input = "", env: extra = {}, timeout = 20000 } = {}) {
      const result = spawnSync(file, args, {
        input,
        encoding: "utf8",
        timeout,
        env: box.env(extra),
      });
      return { status: result.status, out: result.stdout, err: result.stderr };
    },
    cleanup() {
      fs.rmSync(root, { recursive: true, force: true });
    },
  };
  process.on("exit", box.cleanup);
  return box;
}

// A stand-in for `ratmos` to put in ATMOS_BACKEND. It runs the real binary
// pinned to a fixture root: requests read and write under that root, and
// settings.set and `apply` log their commands to <root>/commands.log instead of
// running them. That covers ratmos only. The rest of the app (quickshell and
// any process it starts) is not restricted by it, so run that under box.env()
// as well. The root goes first so it survives the `apply --`
// terminator Backend.qml uses; a caller that passes its own --root is refused
// instead of being allowed to move the pin.
function backendWrapper(dir, { root, binary = ratmosBinary() } = {}) {
  const fixture = root || path.join(dir, "root");
  fs.mkdirSync(fixture, { recursive: true });
  const file = path.join(dir, "ratmos-fixture");
  const script = [
    "#!/bin/sh",
    'for arg in "$@"; do',
    '  if [ "$arg" = "--root" ]; then echo "ratmos-fixture: --root is pinned" >&2; exit 64; fi',
    "done",
    `exec "${binary}" --root "${fixture}" "$@"`,
    "",
  ].join("\n");
  fs.writeFileSync(file, script, { mode: 0o755 });
  return { file, root: fixture };
}

// The ratmos the repo's other tests use. Built every time, once per process:
// recording or replaying against a stale binary would pin old behaviour, and
// cargo returns at once when nothing changed.
let built = null;
function ratmosBinary() {
  if (built) return built;
  const manifest = path.join(repo, "backend", "Cargo.toml");
  const result = spawnSync("cargo", ["build", "--quiet", "--manifest-path", manifest], {
    encoding: "utf8",
  });
  if (result.status !== 0) throw new Error(`cargo build failed:\n${result.stderr}`);
  built = path.join(repo, "backend", "target", "debug", "ratmos");
  return built;
}

module.exports = {
  createSandbox,
  assertHermetic,
  backendWrapper,
  ratmosBinary,
  hostWhich,
  scriptsDir,
  repo,
  UNEXPECTED,
  DEFAULT_TRAPS,
};
