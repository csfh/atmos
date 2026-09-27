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
// Never elevate in tests. This stub also models a driver that requires Custom
// before accepting the threshold, and can reject the mode write.
fs.writeFileSync(
  path.join(fixture, "as-root.sh"),
  [
    "#!/bin/bash",
    "set -euo pipefail",
    "file=$2",
    "if [[ ${ATMOS_TEST_REJECT_MODE:-0} == 1 && $file == */charge_types ]]; then",
    "  echo 'mode write rejected' >&2",
    "  exit 1",
    "fi",
    "if [[ $file == */charge_control_end_threshold ]]; then",
    '  [[ $(<"${file%/*}/charge_types") == Custom ]]',
    "fi",
    'chmod u+w "$file"',
    '"$@"',
    'chmod u-w "$file"',
    "",
  ].join("\n"),
);

function run(limit, extraEnv) {
  return spawnSync("bash", [script, limit], {
    encoding: "utf8",
    env: Object.assign(
      {},
      process.env,
      {
        ATMOS_POWER_SUPPLY_DIR: path.dirname(battery),
        ATMOS_TEST_REJECT_MODE: "0",
      },
      extraEnv,
    ),
  });
}

function reset(modeText) {
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
    assertEqual(result.status, 0, "privileged writes select Custom before the threshold");
    assertEqual(fs.readFileSync(end, "utf8"), "80\n", "privileged path writes the limit");

    reset(adaptive);
    fs.chmodSync(modes, 0o400);
    result = run("80", { ATMOS_TEST_REJECT_MODE: "1" });
    assert(result.status !== 0, "reports a failed Custom mode write");
    assertEqual(fs.readFileSync(end, "utf8"), "100\n", "mode failure aborts the threshold write");
  }

  fs.unlinkSync(end);
  result = run("80");
  assertEqual(result.status, 1, "reports a battery without threshold support");
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
