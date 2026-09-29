const fs = require("fs");
const path = require("path");
const { assert } = require("./harness");

const roots = ["services", "pages", "components"].map((dir) => path.join(__dirname, "..", dir));

function qmlFiles(dir, out) {
  for (const name of fs.readdirSync(dir)) {
    const full = path.join(dir, name);
    const stat = fs.statSync(full);
    if (stat.isDirectory()) qmlFiles(full, out);
    else if (name.endsWith(".qml")) out.push(full);
  }
  return out;
}

const files = roots.flatMap((dir) => qmlFiles(dir, []));
const forbiddenArgv =
  /command\s*[:=]\s*\[[\s\S]{0,80}?["'](omarchy|hyprctl|pactl|nmcli|cat|xdg-open|systemctl|journalctl|inotifywait)["']/;

for (const file of files) {
  const text = fs.readFileSync(file, "utf8");
  const rel = path.relative(path.join(__dirname, ".."), file);
  assert(text.indexOf("inotifywait") === -1, rel + " does not run inotifywait");
  assert(!/\bFileView\b/.test(text), rel + " does not open a FileView");
  assert(!forbiddenArgv.test(text), rel + " does not spawn a host command itself");
}

const theme = fs.readFileSync(path.join(__dirname, "..", "services", "Theme.qml"), "utf8");
const accounts = fs.readFileSync(
  path.join(__dirname, "..", "services", "AccountsStore.qml"),
  "utf8",
);
const omarchy = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
const live = fs.readFileSync(path.join(__dirname, "..", "services", "LiveStatsStore.qml"), "utf8");
const disks = fs.readFileSync(path.join(__dirname, "..", "pages", "DisksPage.qml"), "utf8");
const speed = fs.readFileSync(
  path.join(__dirname, "..", "pages", "network", "SpeedtestPage.qml"),
  "utf8",
);
const exp = fs.readFileSync(path.join(__dirname, "..", "pages", "ExportPage.qml"), "utf8");
const services = fs.readFileSync(path.join(__dirname, "..", "pages", "ServicesPage.qml"), "utf8");

assert(
  theme.indexOf("host.chrome") !== -1 && theme.indexOf("/bin/ratmos") !== -1,
  "theme chrome comes from ratmos",
);
assert(accounts.indexOf("host.accounts") !== -1, "accounts come from ratmos");
assert(
  omarchy.indexOf("host.stamp") !== -1 && omarchy.indexOf("backendCommand") !== -1,
  "watches poll ratmos",
);
assert(
  live.indexOf('["display", "live"]') !== -1 && live.indexOf("--sampler") === -1,
  "live stats do not pass a script path",
);
assert(
  disks.indexOf("speedtest.disk") !== -1 && disks.indexOf("backendCommand") !== -1,
  "disk speed test uses ratmos",
);
assert(
  speed.indexOf("speedtest.net") !== -1 && speed.indexOf("backendCommand") !== -1,
  "network speed test uses ratmos",
);
assert(
  exp.indexOf("host.read") !== -1 &&
    exp.indexOf("host.write") !== -1 &&
    exp.indexOf("host.open") !== -1,
  "import and export file IO uses ratmos",
);
assert(
  services.indexOf("unit.output") !== -1 && services.indexOf("backendCommand") !== -1,
  "unit status and logs use ratmos",
);
assert(
  omarchy.indexOf("jobProc.command = job.argv") === -1,
  "jobs are not started as a raw host argv",
);
assert(omarchy.indexOf("function backendApply") !== -1, "host actions go through ratmos apply");
const interactive = omarchy.slice(
  omarchy.indexOf("function runInteractive("),
  omarchy.indexOf("function commandFailureText("),
);
assert(interactive.indexOf("backendApply") !== -1, "interactive tools start through ratmos");
const systemd = omarchy.slice(
  omarchy.indexOf("function systemdAction("),
  omarchy.indexOf("function applyProfileValues("),
);
assert(systemd.indexOf("runCommand") !== -1, "systemctl start and stop go through runCommand");

console.log("ok - quickshell host reads go through ratmos");
