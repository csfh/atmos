const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const assert = require("node:assert/strict");
const { ratmosBinary } = require("./sandbox");

const root = path.resolve(__dirname, "..");
const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-install-"));
const src = path.join(fixture, "src");
const data = path.join(fixture, "data");
const dest = path.join(data, "atmos");
const stubs = path.join(fixture, "stubs");
const ratmos = ratmosBinary();

function write(file, text, mode = 0o644) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text, { mode });
}

function env(extra = {}) {
  return {
    ...process.env,
    PATH: stubs + ":" + process.env.PATH,
    HOME: path.join(fixture, "home"),
    XDG_DATA_HOME: data,
    XDG_CACHE_HOME: path.join(fixture, "cache"),
    XDG_CONFIG_HOME: path.join(fixture, "config"),
    XDG_BIN_HOME: path.join(fixture, "bin"),
    ATMOS_HYPR_DIR: path.join(fixture, "hypr"),
    ATMOS_ROOT: dest,
    XDG_RUNTIME_DIR: path.join(fixture, "run"),
    SYSTEMCTL_LOG: path.join(fixture, "systemctl.log"),
    FAIL_BUILD: "0",
    REAL_RATMOS: ratmos,
    BACKEND_OUTPUT: path.join(src, "backend/target/release"),
    ...extra,
  };
}

function install(fail) {
  return spawnSync(ratmos, ["install", src], {
    encoding: "utf8",
    env: env({ FAIL_BUILD: fail ? "1" : "0" }),
  });
}

try {
  for (const item of ["components", "pages", "services", "packaging", "icons"]) {
    write(path.join(src, item, "fixture"), "new");
  }
  fs.cpSync(path.join(root, "packaging"), path.join(src, "packaging"), { recursive: true });
  write(path.join(src, "shell.qml"), "new shell");
  fs.mkdirSync(path.join(src, "bin"));
  fs.copyFileSync(path.join(root, "bin/atmos"), path.join(src, "bin/atmos"));
  fs.chmodSync(path.join(src, "bin/atmos"), 0o755);
  write(path.join(src, "backend/Cargo.toml"), "fixture");
  // The build stub puts the real ratmos where cargo would, so the staged app can launch.
  write(
    path.join(stubs, "cargo"),
    `#!/bin/bash
if [[ $FAIL_BUILD == 1 ]]; then
  echo "fixture: backend build failed" >&2
  exit 1
fi
mkdir -p "$BACKEND_OUTPUT"
cp "$REAL_RATMOS" "$BACKEND_OUTPUT/ratmos"
`,
    0o755,
  );
  // Never the real session manager: the stub only records what install asked of it.
  write(path.join(stubs, "systemctl"), '#!/bin/bash\necho "$*" >> "$SYSTEMCTL_LOG"\n', 0o755);
  write(path.join(stubs, "quickshell"), '#!/bin/bash\nprintf "launched\\n"\n', 0o755);
  write(path.join(dest, "bin/atmos"), "old launcher", 0o755);
  write(path.join(dest, "bin/ratmos"), "old backend", 0o755);
  write(path.join(dest, "shell.qml"), "old shell");

  const failed = install(true);
  assert.notEqual(failed.status, 0, failed.stderr);
  assert.equal(
    fs.readFileSync(path.join(dest, "bin/ratmos"), "utf8"),
    "old backend",
    "failed build must preserve the installed backend",
  );
  assert.equal(
    fs.readFileSync(path.join(dest, "bin/atmos"), "utf8"),
    "old launcher",
    "failed build must preserve the launcher",
  );
  assert.equal(
    fs.readFileSync(path.join(dest, "shell.qml"), "utf8"),
    "old shell",
    "failed build must preserve the QML",
  );
  console.log("ok - failed backend build preserves the installed app");

  const installed = install(false);
  assert.equal(installed.status, 0, installed.stderr);
  const launch = spawnSync("bash", [path.join(dest, "bin/atmos")], {
    encoding: "utf8",
    env: env({ ATMOS_BACKEND: "" }),
  });
  assert.equal(launch.status, 0, launch.stderr);
  assert.equal(launch.stdout.trim(), "launched");
  console.log("ok - staged app launches with its installed backend");

  const link = path.join(fixture, "bin", "atmos");
  assert.equal(
    fs.readlinkSync(link),
    path.join(dest, "bin/atmos"),
    "the launcher is linked into the bin dir",
  );
  assert.match(
    fs.readFileSync(path.join(data, "applications/atmos.desktop"), "utf8"),
    new RegExp(`^Exec=${path.join(dest, "bin/atmos")}$`, "m"),
    "the desktop entry points at the installed launcher",
  );
  console.log("ok - install links the launcher and writes the desktop entry");

  const unit = fs.readFileSync(path.join(fixture, "config/systemd/user/ratmos.service"), "utf8");
  assert.ok(
    unit.includes(`ExecStart="${path.join(dest, "bin/ratmos")}" daemon`),
    "the daemon unit runs the installed ratmos",
  );
  const sampler = fs.readFileSync(
    path.join(fixture, "config/systemd/user/ratmos-sampler.service"),
    "utf8",
  );
  assert.ok(
    sampler.includes(`ExecStart="${path.join(dest, "bin/ratmos")}" sampler`),
    "the sampler unit runs the installed ratmos",
  );
  assert.match(sampler, /^CPUSchedulingPolicy=idle$/m, "the sampler runs at idle priority");
  assert.deepEqual(
    fs.readFileSync(path.join(fixture, "systemctl.log"), "utf8").trim().split("\n"),
    [
      "--user daemon-reload",
      "--user enable ratmos.service",
      "--user restart ratmos.service",
      "--user enable ratmos-sampler.service",
      "--user restart ratmos-sampler.service",
    ],
    "install reloads, then enables and restarts the daemon and the sampler in that order",
  );
  fs.writeFileSync(path.join(fixture, "systemctl.log"), "");
  assert.equal(install(false).status, 0);
  assert.deepEqual(
    fs.readFileSync(path.join(fixture, "systemctl.log"), "utf8").trim().split("\n"),
    [
      "--user enable ratmos.service",
      "--user restart ratmos.service",
      "--user enable ratmos-sampler.service",
      "--user restart ratmos-sampler.service",
    ],
    "unchanged units are not reloaded, but both still restart onto the new binary",
  );
  console.log("ok - install writes, enables and restarts the daemon and sampler units");

  const cached = path.join(fixture, "cache/atmos/src");
  fs.mkdirSync(path.join(cached, ".git"), { recursive: true });
  write(path.join(dest, "REVISION"), "abcdef1234\n");
  write(
    path.join(stubs, "git"),
    `#!/bin/bash
if [[ $3 == rev-parse ]]; then
  printf 'abcdef1234\\n'
fi
`,
    0o755,
  );
  function check() {
    return spawnSync(ratmos, ["task", "update-atmos", "check"], { encoding: "utf8", env: env() });
  }
  const current = check();
  assert.equal(current.status, 0, current.stderr);
  assert.match(current.stdout, /^status current$/m);
  fs.unlinkSync(path.join(dest, "bin/ratmos"));
  const incomplete = check();
  assert.equal(incomplete.status, 0, incomplete.stderr);
  assert.match(
    incomplete.stdout,
    /^status behind$/m,
    "a matching revision must not hide a missing backend",
  );
  console.log("ok - update check detects a missing backend at the current revision");
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
