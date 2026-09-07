const { load, assertEqual } = require("./harness");

const sys = load("services/Systemd.js");
assertEqual(sys.canMutate("pipewire.service", "user"), true, "allowlist includes user pipewire");
assertEqual(sys.canMutate("sshd.service", "system"), false, "sshd is not allowlisted");
assertEqual(sys.canMutate("evil.service;rm", "system"), false, "unsafe unit is refused");
assertEqual(
  sys.argvFor("restart", "bluetooth.service", "system").join(" "),
  "systemctl restart bluetooth.service",
  "argvFor system restart",
);
assertEqual(
  sys.argvFor("start", "pipewire.service", "user").join(" "),
  "systemctl --user start pipewire.service",
  "argvFor user start",
);
assertEqual(sys.argvFor("start", "sshd.service", "system"), null, "argvFor refuses sshd");
assertEqual(sys.argvFor("mask", "bluetooth.service", "system"), null, "argvFor refuses mask");
const rows = sys.parseList("bluetooth.service loaded failed failed Bluetooth service\n", "system");
assertEqual(rows[0].unit, "bluetooth.service", "parseList reads a unit");
assertEqual(rows[0].allowed, true, "parseList marks allowlisted units");
assertEqual(
  sys.parseList("sshd.service loaded failed failed OpenSSH\n", "system")[0].allowed,
  false,
  "parseList marks sshd read-only",
);
