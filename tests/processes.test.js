const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { load, assert, assertEqual } = require("./harness");

const proc = load("services/Processes.js");

assertEqual(proc.parsePid("1"), 0, "parsePid refuses pid 1");
assertEqual(proc.parsePid("0"), 0, "parsePid refuses 0");
assertEqual(proc.parsePid("-3"), 0, "parsePid refuses a negative");
assertEqual(proc.parsePid("12;rm"), 0, "parsePid refuses a shell fragment");
assertEqual(proc.parsePid("2201"), 2201, "parsePid accepts a user pid");
assertEqual(proc.signalName("term"), "TERM", "signalName accepts TERM");
assertEqual(proc.signalName("SIGKILL"), "KILL", "signalName accepts SIGKILL");
assertEqual(proc.signalName("STOP"), "", "signalName refuses STOP");
assertEqual(proc.signalArgv(1, "TERM", "/x"), null, "signalArgv refuses pid 1");
assertEqual(proc.signalArgv(9, "STOP", "/x"), null, "signalArgv refuses STOP");
assertEqual(proc.signalArgv(9, "TERM", ""), null, "signalArgv needs a script");
assertEqual(
  proc.signalArgv(9, "TERM", "/tmp/signal-process.sh").join(" "),
  "bash /tmp/signal-process.sh 9 TERM",
  "signalArgv is bash script pid TERM",
);

const rows = [
  { pid: 10, comm: "firefox", cmdline: "firefox -P", rssKb: 80, cpu: 12 },
  { pid: 11, comm: "node", cmdline: "node app", rssKb: 200, cpu: 3 },
  { pid: 12, comm: "sleep", cmdline: "sleep 4", rssKb: 4, cpu: null },
];
assertEqual(proc.list(rows, "", "cpu")[0].comm, "firefox", "list sorts by cpu");
assertEqual(proc.list(rows, "", "memory")[0].comm, "node", "list sorts by memory");
assertEqual(proc.list(rows, "fire", "cpu").length, 1, "list filters comm");
assertEqual(proc.list(rows, "11", "cpu")[0].pid, 11, "list filters pid");
assertEqual(proc.list(rows, "zzz", "cpu").length, 0, "list misses a bad query");

const many = [];
for (let i = 2; i < 100; i++) many.push({ pid: i, comm: "p" + i, cmdline: "p", rssKb: i, cpu: i });
assertEqual(proc.list(many, "", "cpu").length, 80, "list caps at 80");
assertEqual(proc.primaryAction().id, "term", "primary action is End");
assert(
  proc.overflowActions().some(function (item) {
    return item.id === "kill";
  }),
  "overflow includes Force quit",
);

const script = path.join(__dirname, "..", "scripts", "signal-process.sh");
assert(fs.existsSync(script), "signal-process.sh exists");

function write(file, text) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text);
}

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-signal-"));
write(path.join(fixture, "proc/2201/status"), "Name:\tfirefox\nUid:\t1000\t1000\t1000\t1000\n");
write(path.join(fixture, "proc/9/status"), "Name:\troot\nUid:\t0\t0\t0\t0\n");

function run(args, extraEnv) {
  const env = {
    ...process.env,
    ATMOS_SYS_ROOT: fixture,
    ATMOS_UID: "1000",
    ATMOS_SIGNAL_DRY: "1",
    XDG_RUNTIME_DIR: fixture,
  };
  if (extraEnv) Object.assign(env, extraEnv);
  return spawnSync("bash", [script].concat(args), {
    encoding: "utf8",
    env: env,
  });
}

const ok = run(["2201", "TERM"]);
assertEqual(ok.status, 0, "signal-process.sh dry-runs a user pid");
assertEqual(ok.stdout.trim(), "kill -s TERM 2201", "signal-process.sh prints the kill line");

const init = run(["1", "TERM"]);
assert(init.status !== 0, "signal-process.sh refuses pid 1");

const other = run(["9", "KILL"]);
assert(other.status !== 0, "signal-process.sh refuses another uid");

const missing = run(["404", "TERM"]);
assert(missing.status !== 0, "signal-process.sh refuses a missing pid");

const badSig = run(["2201", "STOP"]);
assert(badSig.status !== 0, "signal-process.sh refuses STOP");

const self = run(["2201", "TERM"], { ATMOS_SELF_PID: "2201" });
assert(self.status !== 0, "signal-process.sh refuses ATMOS_SELF_PID");

fs.rmSync(fixture, { recursive: true, force: true });
