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
  if (rel === "services/I18n.qml") {
    assert(
      text.includes('path: Quickshell.shellDir + "/i18n.json"') &&
        !text.includes("Backend.request") &&
        !text.includes("Process {"),
      "I18n reads only the bundled translation resource, not host preferences",
    );
  } else {
    assert(!/\bFileView\b/.test(text), rel + " does not open a FileView");
  }
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

const snapshotStore = fs.readFileSync(
  path.join(__dirname, "..", "services", "SnapshotStore.qml"),
  "utf8",
);
const backend = fs.readFileSync(path.join(__dirname, "..", "services", "Backend.qml"), "utf8");
assert(
  theme.indexOf("Requests.hostChrome") !== -1 && theme.indexOf("Backend.chrome") !== -1,
  "theme chrome comes from ratmos",
);
assert(
  accounts.indexOf("Requests.hostAccounts") !== -1 && accounts.indexOf("Backend.accounts") !== -1,
  "accounts come from ratmos",
);
assert(
  snapshotStore.indexOf("Backend.watch(") !== -1 && snapshotStore.indexOf("Backend.stamp") !== -1,
  "file watches are pushed by ratmos, not polled",
);
assert(
  backend.indexOf('command(["serve"])') !== -1 && backend.indexOf("/bin/ratmos") !== -1,
  "the app talks to one ratmos serve process",
);
for (const [name, src] of [
  ["Theme", theme],
  ["AccountsStore", accounts],
]) {
  assert(!/\bProcess\b/.test(src), name + " does not own a Process; Backend does");
  assert(!/\bTimer\b/.test(src), name + " does not poll on a Timer");
}
assert(
  live.indexOf('Requests.displayGet("live")') !== -1 && live.indexOf("--sampler") === -1,
  "live stats do not pass a script path",
);
assert(
  disks.indexOf("Requests.speedtestDisk") !== -1 && disks.indexOf("Backend.request") !== -1,
  "disk speed test uses ratmos",
);
assert(
  speed.indexOf("Requests.speedtestNet") !== -1 && speed.indexOf("Backend.request") !== -1,
  "network speed test uses ratmos",
);
assert(
  exp.indexOf("Requests.hostRead") !== -1 &&
    exp.indexOf("Requests.hostWrite") !== -1 &&
    exp.indexOf("Requests.hostOpen") !== -1,
  "import and export file IO uses ratmos",
);
assert(
  services.indexOf("Requests.unitOutput") !== -1 && services.indexOf("Backend.request") !== -1,
  "unit status and logs use ratmos",
);
assert(
  omarchy.indexOf("jobProc.command = job.argv") === -1,
  "jobs are not started as a raw host argv",
);
assert(omarchy.indexOf("function backendApply") !== -1, "host actions go through ratmos apply");
const ioQueueSrc = fs.readFileSync(path.join(__dirname, "..", "services", "IoQueue.qml"), "utf8");
assert(
  ioQueueSrc.indexOf("function startInteractive(") !== -1 &&
    ioQueueSrc.indexOf("Backend.applyCommand(wrapped)") !== -1,
  "interactive tools start through ratmos",
);
assert(
  ioQueueSrc.indexOf("Backend.applyCommand(job.argv)") !== -1,
  "queued jobs and writes start through ratmos apply",
);
const systemd = omarchy.slice(
  omarchy.indexOf("function systemdAction("),
  omarchy.indexOf("function applyProfileValues("),
);
assert(systemd.indexOf("runCommand") !== -1, "systemctl start and stop go through runCommand");

console.log("ok - quickshell host reads go through ratmos");
