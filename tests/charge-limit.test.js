const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { assert, assertEqual } = require("./harness");

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-charge-limit-"));
const battery = path.join(fixture, "power_supply", "BAT0");
const end = path.join(battery, "charge_control_end_threshold");
const start = path.join(battery, "charge_control_start_threshold");
const modes = path.join(battery, "charge_types");
const script = path.join(fixture, "set-charge-limit.sh");
const adaptive = "Trickle Fast Standard [Adaptive] Custom\n";
fs.mkdirSync(battery, { recursive: true });
fs.copyFileSync(path.join(__dirname, "..", "scripts", "set-charge-limit.sh"), script);
const elevations = path.join(fixture, "elevations");
// Never elevate in tests. The stub counts calls and grants write access to the
// battery files for the one command, except a mode write it is told to reject.
fs.writeFileSync(
  path.join(fixture, "as-root.sh"),
  [
    "#!/bin/bash",
    "set -euo pipefail",
    'echo call >>"$ATMOS_TEST_ELEVATIONS"',
    'files=("$ATMOS_POWER_SUPPLY_DIR"/BAT0/charge_control_end_threshold)',
    "if [[ ${ATMOS_TEST_REJECT_MODE:-0} != 1 ]]; then",
    '  files+=("$ATMOS_POWER_SUPPLY_DIR"/BAT0/charge_types)',
    "fi",
    'for f in "${files[@]}"; do [[ ! -e $f ]] || chmod u+w "$f"; done',
    "status=0",
    '"$@" || status=$?',
    'for f in "${files[@]}"; do [[ ! -e $f ]] || chmod u-w "$f"; done',
    'exit "$status"',
    "",
  ].join("\n"),
);

function elevationCount() {
  return fs.existsSync(elevations) ? fs.readFileSync(elevations, "utf8").split("\n").length - 1 : 0;
}

function run(limit, extraEnv) {
  return spawnSync("bash", [script, limit], {
    encoding: "utf8",
    env: Object.assign(
      {},
      process.env,
      {
        ATMOS_POWER_SUPPLY_DIR: path.dirname(battery),
        ATMOS_TEST_ELEVATIONS: elevations,
        ATMOS_TEST_REJECT_MODE: "0",
      },
      extraEnv,
    ),
  });
}

function reset(modeText) {
  fs.rmSync(elevations, { force: true });
  for (const file of [end, start, modes]) {
    if (fs.existsSync(file)) fs.chmodSync(file, 0o600);
  }
  fs.writeFileSync(end, "100\n");
  fs.writeFileSync(start, "50\n");
  if (modeText === null) {
    fs.rmSync(modes, { force: true });
  } else {
    fs.writeFileSync(modes, modeText);
  }
}

try {
  reset(adaptive);
  let result = run("80");
  assertEqual(result.status, 0, "Adaptive battery accepts an 80% limit");
  assertEqual(fs.readFileSync(modes, "utf8"), "Custom\n", "activates Custom mode");
  assertEqual(fs.readFileSync(end, "utf8"), "80\n", "writes the stop threshold");
  assertEqual(fs.readFileSync(start, "utf8"), "50\n", "preserves the start threshold");
  assertEqual(elevationCount(), 0, "writable battery files need no elevation");

  for (const modeText of ["Standard Adaptive [Custom]\n", "[Standard] Fast\n", null]) {
    reset(modeText);
    result = run("80");
    assertEqual(result.status, 0, "existing Custom and threshold-only batteries still work");
    assertEqual(fs.readFileSync(end, "utf8"), "80\n", "updates the limit on other batteries");
    assertEqual(
      fs.existsSync(modes) ? fs.readFileSync(modes, "utf8") : null,
      modeText,
      "leaves active Custom or unsupported modes alone",
    );
  }

  for (const limit of ["49", "101", "invalid", ""]) {
    reset(adaptive);
    result = run(limit);
    assertEqual(result.status, 2, "rejects invalid limits before changing battery settings");
    assertEqual(fs.readFileSync(modes, "utf8"), adaptive, "invalid limit preserves mode");
    assertEqual(fs.readFileSync(end, "utf8"), "100\n", "invalid limit preserves threshold");
  }

  // Permission bits cannot force the helper path when the suite runs as root.
  if (process.getuid() !== 0) {
    reset(adaptive);
    fs.chmodSync(modes, 0o400);
    fs.chmodSync(end, 0o400);
    result = run("80");
    assertEqual(result.status, 0, "privileged path accepts an 80% limit");
    assertEqual(fs.readFileSync(modes, "utf8"), "Custom\n", "privileged path activates Custom");
    assertEqual(fs.readFileSync(end, "utf8"), "80\n", "privileged path writes the limit");
    assertEqual(elevationCount(), 1, "mode and threshold share one elevation");

    reset(adaptive);
    fs.chmodSync(end, 0o400);
    result = run("80");
    assertEqual(result.status, 0, "read-only threshold with writable mode still succeeds");
    assertEqual(fs.readFileSync(modes, "utf8"), "Custom\n", "elevated write still selects Custom");
    assertEqual(elevationCount(), 1, "one elevation when only the threshold is read-only");

    reset("[Standard] Fast\n");
    fs.chmodSync(end, 0o400);
    result = run("80");
    assertEqual(result.status, 0, "threshold-only elevation succeeds without Custom");
    assertEqual(
      fs.readFileSync(modes, "utf8"),
      "[Standard] Fast\n",
      "elevation leaves other modes",
    );
    assertEqual(fs.readFileSync(end, "utf8"), "80\n", "elevation writes the limit alone");

    reset(adaptive);
    fs.chmodSync(modes, 0o400);
    result = run("80", { ATMOS_TEST_REJECT_MODE: "1" });
    assert(result.status !== 0, "reports a failed Custom mode write");
    assertEqual(fs.readFileSync(modes, "utf8"), adaptive, "rejected mode write keeps Adaptive");
    assertEqual(fs.readFileSync(end, "utf8"), "100\n", "mode failure aborts the threshold write");
  }

  fs.unlinkSync(end);
  result = run("80");
  assertEqual(result.status, 1, "reports a battery without threshold support");
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
