const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const assert = require("node:assert/strict");

const root = path.resolve(__dirname, "..");
const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-install-"));
const src = path.join(fixture, "src");
const dest = path.join(fixture, "installed");
const stubs = path.join(fixture, "stubs");

function write(file, text, mode = 0o644) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text, { mode });
}

function stage(fail) {
  return spawnSync(
    "bash",
    [
      "-c",
      'set -euo pipefail; source "$1"; atmos_stage "$2" "$3"',
      "test",
      path.join(root, "scripts/atmos-xdg.sh"),
      src,
      dest,
    ],
    {
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: stubs + ":" + process.env.PATH,
        FAIL_BUILD: fail ? "1" : "0",
        BACKEND_OUTPUT: path.join(src, "backend/target/release"),
      },
    },
  );
}

try {
  for (const item of ["components", "pages", "services", "scripts", "packaging", "icons"]) {
    write(path.join(src, item, "fixture"), "new");
  }
  write(path.join(src, "shell.qml"), "new shell");
  fs.mkdirSync(path.join(src, "bin"));
  fs.copyFileSync(path.join(root, "bin/atmos"), path.join(src, "bin/atmos"));
  fs.chmodSync(path.join(src, "bin/atmos"), 0o755);
  write(path.join(src, "backend/Cargo.toml"), "fixture");
  write(
    path.join(stubs, "cargo"),
    `#!/bin/bash
if [[ $FAIL_BUILD == 1 ]]; then
  echo "fixture: backend build failed" >&2
  exit 1
fi
mkdir -p "$BACKEND_OUTPUT"
printf '#!/bin/bash\\nexit 0\\n' >"$BACKEND_OUTPUT/ratmos"
chmod +x "$BACKEND_OUTPUT/ratmos"
`,
    0o755,
  );
  write(path.join(stubs, "quickshell"), '#!/bin/bash\nprintf "launched\\n"\n', 0o755);
  write(path.join(dest, "bin/atmos"), "old launcher", 0o755);
  write(path.join(dest, "bin/ratmos"), "old backend", 0o755);
  write(path.join(dest, "shell.qml"), "old shell");

  const failed = stage(true);
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

  const installed = stage(false);
  assert.equal(installed.status, 0, installed.stderr);
  const launch = spawnSync("bash", [path.join(dest, "bin/atmos")], {
    encoding: "utf8",
    env: { ...process.env, PATH: stubs + ":" + process.env.PATH, ATMOS_BACKEND: "" },
  });
  assert.equal(launch.status, 0, launch.stderr);
  assert.equal(launch.stdout.trim(), "launched");
  console.log("ok - staged app launches with its installed backend");

  const data = path.join(fixture, "data");
  const cached = path.join(fixture, "cache/atmos/src");
  const installedApp = path.join(data, "atmos");
  fs.mkdirSync(path.join(cached, ".git"), { recursive: true });
  fs.cpSync(dest, installedApp, { recursive: true });
  write(path.join(installedApp, "REVISION"), "abcdef1234\n");
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
    return spawnSync("bash", [path.join(root, "scripts/update-atmos.sh"), "check"], {
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: stubs + ":" + process.env.PATH,
        XDG_DATA_HOME: data,
        XDG_CACHE_HOME: path.join(fixture, "cache"),
        XDG_CONFIG_HOME: path.join(fixture, "config"),
      },
    });
  }
  const current = check();
  assert.equal(current.status, 0, current.stderr);
  assert.match(current.stdout, /^status current$/m);
  fs.unlinkSync(path.join(installedApp, "bin/ratmos"));
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
