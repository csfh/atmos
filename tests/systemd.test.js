const { load, assert, assertEqual } = require("./harness");

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

const running = sys.normalizeUnit({
  unit: "pipewire.service",
  scope: "user",
  active: "active",
  sub: "running",
  unitFileState: "enabled",
  description: "PipeWire",
});
assertEqual(sys.runtimeState(running).label, "Running", "active units read as Running");
assertEqual(sys.runtimeState(running).mark, "●", "running uses a filled mark");
assertEqual(sys.startupState(running).label, "Enabled", "enabled unit-file state is Enabled");
assertEqual(sys.statusLine(running), "● Running   Enabled", "status line is runtime then startup");
assertEqual(sys.primaryAction(running).id, "stop", "a running allowlisted unit offers Stop");
assertEqual(sys.primaryAction(running).label, "Stop", "Stop is the running primary label");
assert(
  sys.overflowActions(running).some(function (item) {
    return item.id === "restart";
  }) &&
    !sys.overflowActions(running).some(function (item) {
      return item.id === "start" || item.id === "stop";
    }),
  "running overflow has Restart and not Start or Stop",
);
assert(
  sys.overflowActions(running).some(function (item) {
    return item.id === "disable" && item.label === "Disable at startup";
  }),
  "an enabled unit offers Disable at startup",
);

const stopped = sys.normalizeUnit({
  unit: "docker.service",
  scope: "system",
  active: "inactive",
  sub: "dead",
  unitFileState: "disabled",
});
assertEqual(sys.runtimeState(stopped).label, "Stopped", "inactive units read as Stopped");
assertEqual(sys.runtimeState(stopped).mark, "○", "stopped uses an empty mark");
assertEqual(sys.primaryAction(stopped).id, "start", "a stopped allowlisted unit offers Start");
assert(
  !sys.overflowActions(stopped).some(function (item) {
    return item.id === "stop" || item.id === "restart";
  }),
  "stopped overflow has neither Stop nor Restart",
);
assert(
  sys.overflowActions(stopped).some(function (item) {
    return item.id === "enable" && item.label === "Enable at startup";
  }),
  "a disabled unit offers Enable at startup",
);

const failed = sys.normalizeUnit({
  unit: "bluetooth.service",
  scope: "system",
  active: "failed",
  sub: "failed",
  unitFileState: "enabled",
});
assertEqual(sys.runtimeState(failed).label, "Failed", "failed active is Failed");
assertEqual(sys.runtimeState(failed).mark, "×", "failed uses a cross mark");
assertEqual(sys.primaryAction(failed).id, "restart", "a failed allowlisted unit offers Restart");
assert(sys.isFailed(failed), "isFailed sees active=failed");

const staticUnit = sys.normalizeUnit({
  unit: "dbus-broker.service",
  scope: "user",
  active: "active",
  sub: "running",
  unitFileState: "static",
});
assertEqual(sys.startupState(staticUnit).label, "Static", "static unit-file state is Static");
assertEqual(
  sys.primaryAction(staticUnit),
  null,
  "dbus-broker is not allowlisted so it has no primary action",
);
assert(
  !sys.overflowActions(staticUnit).some(function (item) {
    return (
      item.id === "enable" || item.id === "disable" || item.id === "start" || item.id === "stop"
    );
  }),
  "status-only overflow has no lifecycle actions",
);
assert(
  sys
    .overflowActions(staticUnit)
    .map(function (item) {
      return item.id;
    })
    .join(",") === "status,logs,copy",
  "status-only overflow is status, logs, copy",
);

const mixed = [
  running,
  stopped,
  failed,
  staticUnit,
  sys.normalizeUnit({
    unit: "sshd.service",
    scope: "system",
    active: "inactive",
    sub: "dead",
    unitFileState: "disabled",
  }),
];
const sum = sys.summarize(mixed);
assertEqual(sum.running, 2, "summarize counts running including static-running");
assertEqual(sum.stopped, 2, "summarize counts stopped");
assertEqual(sum.failed, 1, "summarize counts failed");
assertEqual(sum.enabled, 2, "summarize counts enabled unit files");
const parts = sys.summaryParts(sum);
assertEqual(parts[3].text, "1 failed", "summary names a failure count");
assertEqual(parts[3].failed, true, "summary marks failures");
assertEqual(
  sys.summaryParts({ running: 1, stopped: 0, enabled: 1, failed: 0 })[3].text,
  "✓ No failures",
  "zero failures stay inline",
);

assert(sys.matchesFilter(running, "running"), "running filter keeps a running unit");
assert(!sys.matchesFilter(running, "stopped"), "stopped filter drops a running unit");
assert(sys.matchesFilter(failed, "failed"), "failed filter keeps a failed unit");
assert(sys.matchesFilter(running, "enabled"), "enabled filter keeps an enabled unit");
assert(sys.matchesFilter(stopped, "disabled"), "disabled filter keeps a disabled unit");
assert(sys.matchesQuery(running, "pipewire"), "query matches the unit name");
assert(sys.matchesQuery(running, "Running"), "query matches the runtime label");
assertEqual(sys.listUnits(mixed, "docker", "all").length, 1, "listUnits applies search");
assertEqual(
  sys.listUnits(mixed, "", "failed")[0].unit,
  "bluetooth.service",
  "listUnits applies the failed filter",
);
assertEqual(sys.filterChips()[0].id, "all", "filter chips start at All");
assertEqual(sys.filterChips().length, 6, "filter chips cover All plus five states");
