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

const hot = live.parse(
  JSON.stringify({
    cpuIdle: 50,
    cpuTotal: 100,
    cpuTemp: 87,
    gpus: [
      {
        card: "card0",
        pciId: "1002:73ff",
        name: "AMD",
        vendor: "AMD",
        driver: "amdgpu",
        integrated: false,
        temp: 61,
      },
      {
        card: "card1",
        pciId: "8086:a7a0",
        name: "Intel",
        vendor: "Intel",
        driver: "i915",
        integrated: true,
        temp: null,
      },
    ],
  }),
);
assertEqual(hot.cpuTemp, 87, "parse reads cpuTemp");
assertEqual(hot.gpus.length, 2, "parse keeps GPU rows");
assertEqual(hot.gpus[0].temp, 61, "parse passes a GPU temperature");
assertEqual(hot.gpus[0].pciId, "1002:73ff", "parse keeps a GPU pciId");
assertEqual(hot.gpus[1].temp, null, "parse keeps an unknown GPU temperature as null");
assertEqual(hot.gpus[1].integrated, true, "parse keeps the integrated flag");
assertEqual(live.parseTemp(0), null, "parseTemp treats 0 as unknown");
assertEqual(live.parseTemp(-4), null, "parseTemp treats a non-positive as unknown");
assertEqual(live.parse('{"cpuTemp":0}').cpuTemp, null, "parse does not invent 0 °C");
assertEqual(live.gpuTempAt(hot, "card0"), 61, "gpuTempAt prefers the GPU sensor");
assertEqual(live.gpuTempAt(hot, { pciId: "1002:73ff" }), 61, "gpuTempAt matches hardware pciId");
assertEqual(live.gpuTempAt(hot, "card1"), 87, "gpuTempAt falls back to package on integrated");
assertEqual(
  live.gpuTempAt(
    live.parse(
      JSON.stringify({
        cpuTemp: 70,
        gpus: [{ card: "card9", name: "NVIDIA", driver: "nvidia", pciId: "10de:25a2" }],
      }),
    ),
    "card9",
  ),
  null,
  "gpuTempAt never borrows the CPU on discrete",
);
assertEqual(
  live.gpuTempAt({ cpuTemp: 70, gpus: [] }, { driver: "i915", pciId: "8086:a7a0" }),
  70,
  "gpuTempAt uses package when inventory i915 has no live row",
);
assertEqual(live.gpuTempAt(hot, "nope"), null, "gpuTempAt misses unknown cards");
assertEqual(live.gpuOwnTemp(hot, "card1"), null, "gpuOwnTemp is empty without a sensor");
assertEqual(live.formatTemp(87), "87 °C", "formatTemp formats celsius");
assertEqual(live.formatTemp(null), "", "formatTemp unknown is empty");
assertEqual(live.formatTemp(0), "", "formatTemp does not invent 0 °C");
assertEqual(
  live.gpuRows([{ name: "Radeon", pciId: "1002:73ff" }], hot).length,
  1,
  "gpuRows prefers hardware inventory",
);
assertEqual(live.gpuRows([], hot)[0].card, "card0", "gpuRows falls back to live DRM cards");

let th = live.pushSample(
  [],
  live.parse(JSON.stringify({ gpus: [{ card: "card0", name: "AMD", temp: 60 }] })),
  1,
);
th = live.pushSample(
  th,
  live.parse(JSON.stringify({ gpus: [{ card: "card0", name: "AMD", temp: 62 }] })),
  2,
);
assertEqual(live.gpuSeries(th, "card0").join(","), "60,62", "gpuSeries collects temps by card");
assertEqual(live.gpuSeries(th, "card9").length, 0, "gpuSeries misses unknown cards");

let ig = live.pushSample(
  [],
  live.parse(
    JSON.stringify({
      cpuTemp: 70,
      gpus: [{ card: "card2", name: "Intel", driver: "i915", integrated: true, temp: null }],
    }),
  ),
  1,
);
assertEqual(
  live.gpuSeries(ig, "card2").join(","),
  "70",
  "gpuSeries graphs the package on integrated",
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
assertEqual(emitted.cpuTemp, null, "live-stats.py has no thermal on a bare fixture");
assertEqual(Array.isArray(emitted.gpus), true, "live-stats.py emits a GPU list");
assertEqual(emitted.gpus.length, 0, "live-stats.py has no DRM cards on a bare fixture");

write(path.join(fixture, "sys/class/hwmon/hwmon0/name"), "coretemp\n");
write(path.join(fixture, "sys/class/hwmon/hwmon0/temp1_input"), "0\n");
write(path.join(fixture, "sys/class/thermal/thermal_zone0/type"), "acpitz\n");
write(path.join(fixture, "sys/class/thermal/thermal_zone0/temp"), "41000\n");
const zeroPy = spawnSync("python3", [path.join(__dirname, "..", "scripts", "live-stats.py")], {
  encoding: "utf8",
  env: { ...process.env, ATMOS_SYS_ROOT: fixture, ATMOS_UID: "1000" },
});
assertEqual(zeroPy.status, 0, "live-stats.py exits 0 when hwmon is zero");
assertEqual(live.parse(zeroPy.stdout).cpuTemp, null, "live-stats.py does not invent 0 °C");

write(path.join(fixture, "sys/class/hwmon/hwmon0/temp1_input"), "83400\n");
write(path.join(fixture, "sys/class/drm/card0/device/vendor"), "0x1002\n");
write(path.join(fixture, "sys/class/drm/card0/device/device"), "0x73ff\n");
write(path.join(fixture, "sys/class/drm/card0/device/uevent"), "PCI_SLOT_NAME=0000:03:00.0\n");
write(path.join(fixture, "sys/class/drm/card0/device/hwmon/hwmon1/temp1_input"), "61200\n");
fs.symlinkSync("amdgpu", path.join(fixture, "sys/class/drm/card0/device/driver"));
write(path.join(fixture, "sys/class/drm/card1/device/vendor"), "0x8086\n");
write(path.join(fixture, "sys/class/drm/card1/device/device"), "0xa7a0\n");
write(path.join(fixture, "sys/class/drm/card1/device/uevent"), "PCI_SLOT_NAME=0000:00:02.0\n");
fs.symlinkSync("i915", path.join(fixture, "sys/class/drm/card1/device/driver"));
write(path.join(fixture, "sys/class/drm/card0-DP-1/device/vendor"), "0x1002\n");

const hotPy = spawnSync("python3", [path.join(__dirname, "..", "scripts", "live-stats.py")], {
  encoding: "utf8",
  env: { ...process.env, ATMOS_SYS_ROOT: fixture, ATMOS_UID: "1000" },
});
assertEqual(hotPy.status, 0, "live-stats.py exits 0 with thermal fixtures");
const hotEmit = live.parse(hotPy.stdout);
assertEqual(hotEmit.cpuTemp, 83, "live-stats.py reads coretemp");
assertEqual(hotEmit.gpus.length, 2, "live-stats.py skips DRM connectors");
assertEqual(hotEmit.gpus[0].pciId, "1002:73ff", "live-stats.py emits a GPU pciId");
assertEqual(hotEmit.gpus[0].driver, "amdgpu", "live-stats.py reads the DRM driver");
assertEqual(hotEmit.gpus[0].temp, 61, "live-stats.py reads amdgpu hwmon");
assertEqual(hotEmit.gpus[0].integrated, false, "live-stats.py marks amdgpu discrete");
assertEqual(hotEmit.gpus[1].driver, "i915", "live-stats.py reads i915");
assertEqual(hotEmit.gpus[1].temp, null, "live-stats.py leaves i915 without a sensor unknown");
assertEqual(hotEmit.gpus[1].integrated, true, "live-stats.py marks i915 integrated");

fs.rmSync(fixture, { recursive: true, force: true });
