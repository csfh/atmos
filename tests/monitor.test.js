const { load, assert, assertEqual } = require("./harness");

const monitor = load("services/Monitor.js");
const live = load("services/LiveStats.js");
const proc = load("services/Processes.js");

assertEqual(monitor.intervalMs("500"), 500, "intervalMs maps 0.5s");
assertEqual(monitor.intervalMs("nope"), 2000, "intervalMs defaults to 2s");
assertEqual(monitor.pageCards().length, 6, "pageCards lists the Monitor children");
assertEqual(monitor.alertLevel(null), "unknown", "alertLevel unknown stays unknown");
assertEqual(monitor.alertLevel(50), "ok", "alertLevel 50 is ok");
assertEqual(monitor.alertLevel(80), "warm", "alertLevel 80 is warm");
assertEqual(monitor.alertLevel(95), "hot", "alertLevel 95 is hot");

const bars = monitor.barRects([0, 50, 100], 10, 10, 1);
assertEqual(bars.length, 3, "barRects keeps one rect per core");
assertEqual(bars[0].h, 0, "barRects 0% has no height");
assertEqual(bars[2].h, 10, "barRects 100% fills the height");
assertEqual(bars[2].alert, "hot", "barRects 100% is hot");
assertEqual(monitor.barRects([], 10, 10, 1).length, 0, "barRects empty is empty");

const stacked = monitor.stackedRects(
  [
    { id: "used", label: "Used", kb: 50 },
    { id: "free", label: "Free", kb: 50 },
  ],
  100,
  8,
);
assertEqual(stacked.length, 2, "stackedRects keeps segments");
assertEqual(stacked[0].w, 50, "stackedRects used is half");
assertEqual(stacked[1].id, "free", "stackedRects keeps ids");

const hist = monitor.cpuHistogram([
  { cpu: 0 },
  { cpu: 0.02 },
  { cpu: 3 },
  { cpu: 12 },
  { cpu: 90 },
]);
assertEqual(hist[0].count, 2, "cpuHistogram buckets idle");
assertEqual(hist[2].count, 1, "cpuHistogram buckets 1–5%");
assertEqual(hist[5].count, 1, "cpuHistogram buckets hot");
assertEqual(monitor.histogramRects(hist, 60, 20).length, 6, "histogramRects matches bins");

assertEqual(monitor.tcpParts({ established: 4, listen: 2 }).length, 4, "tcpParts has four buckets");
assertEqual(live.formatLoad(1.234), "1.23", "formatLoad is two decimals");
assertEqual(live.formatLoad(null), "", "formatLoad unknown is empty");
assertEqual(live.formatMhz(2400), "2.40 GHz", "formatMhz uses GHz above 1000");
assertEqual(live.formatMhz(800), "800 MHz", "formatMhz keeps MHz");
assertEqual(live.formatPsi(0), "0%", "formatPsi zero is 0%");
assertEqual(live.formatRpm(1200), "1200 RPM", "formatRpm labels revolutions");

const parts = live.memParts({
  memTotal: 1000,
  memFree: 200,
  memBuffers: 100,
  memCached: 300,
  memUsed: 400,
});
assertEqual(parts[0].kb, 400, "memParts used is total-free-buffers-cached");
assertEqual(parts[3].id, "free", "memParts ends on free");

const rich = live.parse(
  JSON.stringify({
    uid: 1000,
    cpuIdle: 10,
    cpuTotal: 20,
    cpus: [{ id: 0, idle: 5, total: 10, freqMhz: 2400, governor: "schedutil" }],
    load1: 0.5,
    load5: 0.4,
    load15: 0.3,
    memUsed: 400,
    memTotal: 1000,
    memFree: 200,
    memBuffers: 50,
    memCached: 200,
    swapUsed: 10,
    swapTotal: 100,
    netRx: 100,
    netTx: 20,
    ifaces: [{ name: "eth0", rx: 100, tx: 20 }],
    disks: [{ name: "sda", readSectors: 10, writeSectors: 4 }],
    psi: { cpu: 1.2, memory: 0, io: 0.4 },
    tcp: { established: 3, listen: 1, timeWait: 2, closeWait: 0, total: 8 },
    sensors: [
      { id: "k10temp:temp1_input", chip: "k10temp", label: "Tctl", kind: "temp", value: 64 },
    ],
    gpus: [
      {
        card: "card0",
        name: "AMD",
        driver: "amdgpu",
        busy: 22,
        vramUsed: 1024,
        vramTotal: 8192,
        temp: 55,
      },
    ],
    processes: [
      {
        pid: 200,
        ppid: 1,
        comm: "firefox",
        cmdline: "firefox",
        uid: 1000,
        rssKb: 80,
        ticks: 30,
        state: "S",
        threads: 12,
        nice: 0,
        kthread: false,
        mine: true,
        readBytes: 1000,
        writeBytes: 200,
      },
    ],
  }),
);
assertEqual(rich.cpus[0].freqMhz, 2400, "parse keeps a core frequency");
assertEqual(rich.ifaces[0].name, "eth0", "parse keeps an interface");
assertEqual(rich.disks[0].name, "sda", "parse keeps a disk");
assertEqual(rich.psi.cpu, 1.2, "parse keeps PSI");
assertEqual(rich.tcp.established, 3, "parse keeps TCP counts");
assertEqual(rich.sensors[0].kind, "temp", "parse keeps a sensor");
assertEqual(rich.gpus[0].busy, 22, "parse keeps GPU busy");
assertEqual(rich.processes[0].state, "S", "parse keeps process state");
assertEqual(rich.processes[0].mine, true, "parse keeps mine");

let hist2 = live.pushSample([], rich, 1000);
const next = live.parse(
  JSON.stringify({
    uid: 1000,
    cpuIdle: 12,
    cpuTotal: 40,
    cpus: [{ id: 0, idle: 6, total: 20, freqMhz: 2500, governor: "schedutil" }],
    load1: 0.8,
    memUsed: 500,
    memTotal: 1000,
    swapUsed: 20,
    swapTotal: 100,
    netRx: 1100,
    netTx: 220,
    ifaces: [{ name: "eth0", rx: 1100, tx: 220 }],
    disks: [{ name: "sda", readSectors: 20, writeSectors: 8 }],
    gpus: [{ card: "card0", name: "AMD", driver: "amdgpu", busy: 40, temp: 57 }],
    processes: [
      {
        pid: 200,
        ppid: 1,
        comm: "firefox",
        cmdline: "firefox",
        uid: 1000,
        rssKb: 90,
        ticks: 50,
        state: "R",
        threads: 12,
        kthread: false,
        mine: true,
        readBytes: 2000,
        writeBytes: 400,
      },
    ],
  }),
);
hist2 = live.pushSample(hist2, next, 2000);
assertEqual(hist2[1].cpus[0].cpu, 90, "second sample decorates per-core cpu");
assertEqual(hist2[1].ifaces[0].rxBps, 1000, "second sample decorates iface rx");
assertEqual(hist2[1].disks[0].readBps, 5120, "disk rate is sectors*512 per second");
assertEqual(hist2[1].processes[0].readBps, 1000, "process readBps is bytes per second");
assertEqual(hist2[1].swap, 20, "swap percent is used/total");
assertEqual(live.coreSeries(hist2, 0).join(","), "90", "coreSeries drops the unknown first sample");
assertEqual(live.ifaceSeries(hist2, "eth0", "rxBps").join(","), "1000", "ifaceSeries is rxBps");
assertEqual(live.gpuBusyAt(hist2[1], "card0"), 40, "gpuBusyAt reads busy");
assertEqual(live.formatLoadLine(hist2[0]), "0.50  0.40  0.30", "formatLoadLine joins 1/5/15");

const mixed = [
  {
    pid: 10,
    comm: "firefox",
    cmdline: "firefox",
    rssKb: 80,
    cpu: 12,
    state: "R",
    mine: true,
    uid: 1000,
    threads: 8,
  },
  {
    pid: 11,
    comm: "sshd",
    cmdline: "sshd",
    rssKb: 20,
    cpu: 1,
    state: "S",
    mine: false,
    uid: 0,
    threads: 1,
  },
  {
    pid: 12,
    comm: "kworker",
    cmdline: "",
    rssKb: 0,
    cpu: 0,
    state: "I",
    kthread: true,
    mine: false,
    uid: 0,
    ppid: 2,
  },
  {
    pid: 13,
    comm: "node",
    cmdline: "node app",
    rssKb: 200,
    cpu: 3,
    state: "S",
    mine: true,
    uid: 1000,
    ppid: 10,
    threads: 4,
  },
];
assertEqual(proc.list(mixed, "", "cpu").length, 2, "list defaults to mine and drops kernel");
assertEqual(proc.list(mixed, "", "cpu", { scope: "all" }).length, 3, "scope all drops kernel");
assertEqual(
  proc.list(mixed, "", "cpu", { scope: "kernel" })[0].comm,
  "kworker",
  "scope kernel keeps kthreads",
);
assertEqual(
  proc.list(mixed, "", "cpu", { state: "running" })[0].comm,
  "firefox",
  "state running keeps R",
);
assertEqual(proc.list(mixed, "", "memory")[0].comm, "node", "sort memory still works");
assertEqual(proc.list(mixed, "", "name")[0].comm, "firefox", "sort name is A-Z");
assertEqual(proc.list(mixed, "", "pid")[0].pid, 10, "sort pid is low first");
assertEqual(
  proc.list(mixed, "", "cpu", { minCpu: 5 })[0].comm,
  "firefox",
  "minCpu drops quiet tasks",
);
assertEqual(
  proc.list(mixed, "", "cpu", { minRssKb: 100 })[0].comm,
  "node",
  "minRssKb drops small tasks",
);
assertEqual(
  proc.list(mixed, "", "cpu", { tree: true, scope: "mine" })[1].depth,
  1,
  "tree indents children",
);
assertEqual(proc.list(mixed, "", "cpu", { cap: 1 }).length, 1, "cap trims the list");
assertEqual(proc.stateLabel("R"), "R", "stateLabel keeps R");
assert(
  proc.scopeChips().some(function (c) {
    return c.id === "kernel";
  }),
  "scopeChips includes kernel",
);
assert(proc.minCpuChips().length >= 3, "minCpuChips has thresholds");
