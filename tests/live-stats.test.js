const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { load, assert, assertEqual } = require("./harness");

const live = load("services/LiveStats.js");

assertEqual(live.parse(""), null, "parse empty is null");
assertEqual(live.parse("{"), null, "parse junk is null");
assertEqual(live.parse(null), null, "parse null is null");

const sample = live.parse(
  JSON.stringify({
    cpuIdle: 80,
    cpuTotal: 100,
    memUsed: 400,
    memTotal: 1000,
    memAvail: 600,
    netRx: 1000,
    netTx: 200,
    clkTck: 100,
    processes: [{ pid: 200, comm: "firefox", cmdline: "firefox", uid: 1000, rssKb: 80, ticks: 30 }],
  }),
);
assertEqual(sample.cpuIdle, 80, "parse reads cpuIdle");
assertEqual(sample.processes[0].comm, "firefox", "parse reads a process");
assertEqual(
  live.parse('{"cpuIdle":-1,"cpuTotal":10}').cpuIdle,
  null,
  "negative counters stay unknown",
);
assertEqual(live.cpuPercent(null, sample), null, "cpuPercent needs two samples");
assertEqual(live.memPercent({ memUsed: 0, memTotal: 0 }), null, "memPercent does not invent 0%");
assertEqual(live.memPercent(sample), 40, "memPercent is used/total");

const next = live.parse(
  JSON.stringify({
    cpuIdle: 90,
    cpuTotal: 200,
    memUsed: 500,
    memTotal: 1000,
    memAvail: 500,
    netRx: 3000,
    netTx: 700,
    clkTck: 100,
    processes: [{ pid: 200, comm: "firefox", cmdline: "firefox", uid: 1000, rssKb: 90, ticks: 50 }],
  }),
);
assertEqual(live.cpuPercent(sample, next), 90, "cpuPercent is 1 - idle/total");
assertEqual(live.netRate(sample, next, 1000).rxBps, 2000, "netRate is bytes per second");
assertEqual(live.processCpu(30, 50, 1000, 100), 20, "processCpu is percent of one core");
assertEqual(live.processCpu(30, 50, 0, 100), null, "processCpu without dt stays unknown");

let history = live.pushSample([], sample, 1000);
assertEqual(history.length, 1, "pushSample keeps the first sample");
assertEqual(history[0].cpu, null, "first cpu percent is unknown");
history = live.pushSample(history, next, 2000);
assertEqual(history[1].cpu, 90, "second sample gets a cpu percent");
assertEqual(history[1].processes[0].cpu, 20, "second sample decorates process cpu");
assertEqual(live.latest(history).mem, 50, "latest is the newest sample");
assertEqual(live.series(history, "cpu").join(","), "90", "series drops unknown cpu");

let filled = [];
for (let i = 0; i < 65; i++) {
  filled = live.pushSample(
    filled,
    {
      cpuIdle: i,
      cpuTotal: i + 10,
      memUsed: 1,
      memTotal: 10,
      netRx: i,
      netTx: 0,
      clkTck: 100,
      processes: [],
    },
    i * 1000,
  );
}
assertEqual(filled.length, 60, "pushSample caps at 60");
assertEqual(live.formatPercent(null), "", "formatPercent unknown is empty");
assertEqual(live.formatPercent(12.4), "12%", "formatPercent rounds");
assertEqual(live.formatBps(null), "", "formatBps unknown is empty");
assert(live.formatNet(history[1]).indexOf("↓") !== -1, "formatNet names rx and tx");
assertEqual(live.memBytes(2), 2048, "memBytes is KiB to bytes");
assertEqual(live.sparklinePoints([1], 100, 20, 2).length, 0, "sparklinePoints needs two values");
assertEqual(live.sparklinePoints([1, 3], 100, 20, 0)[0][0], 0, "sparklinePoints starts at x=0");
assertEqual(live.sparklinePoints([1, 3], 100, 20, 0)[1][0], 100, "sparklinePoints ends at width");
assert(
  live.sparklinePoints([1, 3], 100, 20, 0)[0][1] > live.sparklinePoints([1, 3], 100, 20, 0)[1][1],
  "sparklinePoints puts a larger value higher",
);

function write(file, text) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text);
}

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-live-"));
write(
  path.join(fixture, "proc/stat"),
  "cpu  10 0 10 70 10 0 0 0 0 0\ncpu0 10 0 10 70 10 0 0 0 0 0\n",
);
write(
  path.join(fixture, "proc/meminfo"),
  "MemTotal:        1000 kB\nMemFree:          100 kB\nMemAvailable:     400 kB\n",
);
write(
  path.join(fixture, "proc/net/dev"),
  "Inter-|   Receive                                                |  Transmit\n" +
    " face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed\n" +
    "    lo: 999 0 0 0 0 0 0 0 999 0 0 0 0 0 0 0\n" +
    "  eth0: 500 0 0 0 0 0 0 0 40 0 0 0 0 0 0 0\n",
);
write(
  path.join(fixture, "proc/200/stat"),
  "200 (firefox) S 1 200 200 0 -1 4194304 0 0 0 0 30 10 0 0 20 0 1 0\n",
);
write(
  path.join(fixture, "proc/200/status"),
  "Name:\tfirefox\nUid:\t1000\t1000\t1000\t1000\nVmRSS:\t      80 kB\n",
);
write(path.join(fixture, "proc/200/cmdline"), "firefox\0-P\0");
write(
  path.join(fixture, "proc/201/stat"),
  "201 (kworker) I 2 0 0 0 -1 2097152 0 0 0 0 1 0 0 0 20 0 1 0\n",
);
write(path.join(fixture, "proc/201/status"), "Name:\tkworker\nUid:\t0\t0\t0\t0\n");
write(path.join(fixture, "proc/201/cmdline"), "");
write(
  path.join(fixture, "proc/202/stat"),
  "202 (root) S 1 202 202 0 -1 4194304 0 0 0 0 1 0 0 0 20 0 1 0\n",
);
write(path.join(fixture, "proc/202/status"), "Name:\troot\nUid:\t0\t0\t0\t0\nVmRSS:\t10 kB\n");
write(path.join(fixture, "proc/202/cmdline"), "sshd\0");

const py = spawnSync("python3", [path.join(__dirname, "..", "scripts", "live-stats.py")], {
  encoding: "utf8",
  env: { ...process.env, ATMOS_SYS_ROOT: fixture, ATMOS_UID: "1000" },
});
assertEqual(py.status, 0, "live-stats.py exits 0 on a fixture");
const emitted = live.parse(py.stdout);
assertEqual(emitted.cpuIdle, 80, "live-stats.py idle is idle+iowait");
assertEqual(emitted.cpuTotal, 100, "live-stats.py total is the first eight fields");
assertEqual(emitted.memUsed, 600, "live-stats.py used is total-available");
assertEqual(emitted.netRx, 500, "live-stats.py skips loopback rx");
assertEqual(emitted.processes.length, 1, "live-stats.py keeps this user's processes");
assertEqual(emitted.processes[0].pid, 200, "live-stats.py emits firefox");
assertEqual(emitted.processes[0].ticks, 40, "live-stats.py ticks are utime+stime");

fs.rmSync(fixture, { recursive: true, force: true });
