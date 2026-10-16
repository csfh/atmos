const { load, assert, assertEqual } = require("./harness");

const sentinel = load("services/Sentinel.js");

const DAY = 86400;

function proc(pid, extra) {
  return Object.assign(
    {
      pid: pid,
      comm: "p" + pid,
      cmdline: "p" + pid + " --run",
      uid: 1000,
      mine: true,
      kthread: false,
      rssKb: 1000,
      cpu: 1,
      startTicks: 100,
    },
    extra || {},
  );
}

function sample(at, processes, extra) {
  return Object.assign(
    {
      at: at,
      memTotal: 1000000,
      clkTck: 100,
      uptimeSec: 20 * DAY,
      processes: processes,
      tcpSockets: [],
    },
    extra || {},
  );
}

// Thresholds
assertEqual(sentinel.thresholds(null).cpuPercent, 80, "thresholds default the CPU limit");
assertEqual(sentinel.thresholds({}).memoryPercent, 20, "thresholds default the memory limit");
assertEqual(sentinel.thresholds({}).portAgeDays, 7, "thresholds default the port age");
assertEqual(
  sentinel.thresholds({ sentinelCpuPercent: 10 }).cpuPercent,
  50,
  "a CPU limit under the floor is raised to it",
);
assertEqual(
  sentinel.thresholds({ sentinelCpuPercent: 500 }).cpuPercent,
  100,
  "a CPU limit over the ceiling is lowered to it",
);
assertEqual(
  sentinel.thresholds({ sentinelPortAgeDays: "14" }).portAgeDays,
  14,
  "a stored number as text still counts",
);
assertEqual(
  sentinel.thresholds({ sentinelMemoryPercent: "lots" }).memoryPercent,
  20,
  "junk falls back to the default",
);
assertEqual(
  sentinel.thresholds({ sentinelMemoryPercent: null }).memoryPercent,
  20,
  "null falls back to the default",
);
assertEqual(sentinel.clampThreshold("portAgeDays", 3.6), 4, "thresholds are whole numbers");

// Formatting
assertEqual(sentinel.humanAge(45), "45 s", "ages under a minute and a half are seconds");
assertEqual(sentinel.humanAge(600), "10 min", "ages are minutes");
assertEqual(sentinel.humanAge(7200), "2 h", "ages are hours");
assertEqual(sentinel.humanAge(12 * DAY), "12 d", "ages are days");
assertEqual(sentinel.humanAge(-1), "", "a negative age is blank");
assertEqual(sentinel.humanKb(512), "512 kB", "small sizes stay in kB");
assertEqual(sentinel.humanKb(3 * 1024 * 1024), "3.0 GB", "large sizes are GB");

// CPU has to be sustained
const th = sentinel.thresholds(null);
let state = sentinel.track(null, sample(1000, [proc(10, { cpu: 95 })]), th);
assertEqual(state.over[10].n, 1, "the first sample over the limit counts once");
let flagged = sentinel.rows(sample(1000, [proc(10, { cpu: 95 })]), state, th);
assertEqual(flagged.length, 0, "one hot sample is a spike, not a finding");
const same = sentinel.track(state, sample(1000, [proc(10, { cpu: 95 })]), th);
assertEqual(same, state, "the same sample twice leaves the state alone");
state = sentinel.track(state, sample(2000, [proc(10, { cpu: 90 })]), th);
state = sentinel.track(state, sample(3000, [proc(10, { cpu: 92 })]), th);
assertEqual(state.over[10].n, sentinel.SUSTAIN, "consecutive hot samples accumulate");
assertEqual(state.over[10].since, 1000, "the streak remembers when it began");
const hot = sentinel.rows(sample(3000, [proc(10, { cpu: 92 })]), state, th);
assertEqual(hot.length, 1, "a sustained hot process is flagged");
assertEqual(hot[0].kind, "cpu", "the finding is a CPU one");
assertEqual(hot[0].detail, "92% CPU for 2 s", "the finding says how long it has been hot");
state = sentinel.track(state, sample(4000, [proc(10, { cpu: 5 })]), th);
assertEqual(state.over[10], undefined, "cooling down resets the streak");
state = sentinel.track(state, sample(5000, []), th);
assertEqual(Object.keys(state.over).length, 0, "a process that is gone is forgotten");
assertEqual(sentinel.track(null, null, th).at, null, "no sample gives an empty state");
state = sentinel.track(
  null,
  sample(1, [proc(11, { cpu: 99, kthread: true }), proc(1, { cpu: 99 })]),
  th,
);
assertEqual(Object.keys(state.over).length, 0, "kernel threads and pid 1 are never tracked");

// Memory is a share of the machine
const big = sentinel.rows(
  sample(1, [proc(20, { rssKb: 300000 }), proc(21, { rssKb: 100000 })]),
  null,
  th,
);
assertEqual(big.length, 1, "only the process over the memory share is flagged");
assertEqual(big[0].kind, "memory", "the finding is a memory one");
assertEqual(big[0].detail, "293 MB, 30% of memory", "the finding gives size and share");
assertEqual(
  sentinel.rows(sample(1, [proc(22, { rssKb: 900000 })], { memTotal: null }), null, th).length,
  0,
  "memory is not judged without a total",
);

// Listening ports
const listening = sample(
  1,
  [
    proc(30, { comm: "old-server", startTicks: 100 }),
    proc(31, { comm: "new-server", startTicks: 20 * DAY * 100 - 600 * 100 }),
  ],
  {
    tcpSockets: [
      { state: "LISTEN", port: 8080, addr: "0.0.0.0", uid: 1000, pid: 30 },
      { state: "LISTEN", port: 8080, addr: "::", uid: 1000, pid: 30 },
      { state: "LISTEN", port: 3000, addr: "127.0.0.1", uid: 1000, pid: 31 },
      { state: "LISTEN", port: 22, addr: "0.0.0.0", uid: 0, pid: null },
      { state: "LISTEN", port: 22, addr: "::", uid: 0, pid: null },
      { state: "ESTABLISHED", port: 5000, addr: "127.0.0.1", uid: 1000, pid: 30 },
    ],
  },
);
const ls = sentinel.listeners(listening);
assertEqual(ls.length, 3, "a port is listed once per process even on IPv4 and IPv6");
assertEqual(ls[0].reach, "all interfaces", "0.0.0.0 reaches every interface");
assertEqual(ls[1].reach, "local only", "127.0.0.1 is local only");
assertEqual(ls[2].pid, null, "an unreadable owner stays unknown");
assertEqual(ls[2].age, null, "an unknown owner has no age");
const all = sentinel.rows(listening, null, th);
const oldPort = all.find((r) => r.id === "port:8080:30");
assert(oldPort && oldPort.flagged, "a listener held by an old process is flagged");
assertEqual(oldPort.comm, "old-server", "a port row is named after its process");
assertEqual(
  oldPort.detail,
  ":8080 · all interfaces · process up 20 d",
  "a port row says where it listens and how long its process has been up",
);
const youngPort = all.find((r) => r.id === "port:3000:31");
assert(youngPort && !youngPort.flagged, "a young listener is listed but not flagged");
const unknown = all.find((r) => r.id === "port:22:u0");
assert(unknown && !unknown.flagged, "an unknown owner is listed but never flagged");
assertEqual(
  unknown.detail,
  ":22 · all interfaces · owner unknown (uid 0)",
  "an unknown owner is labelled as such",
);
assertEqual(unknown.proc, null, "an unknown owner has no process to signal");
assertEqual(
  sentinel
    .rows(listening, null, sentinel.thresholds({ sentinelPortAgeDays: 30 }))
    .filter((r) => r.flagged).length,
  0,
  "a longer port age spares the 20-day process",
);

// The list the page shows
assertEqual(sentinel.list(all, "flagged").length, 1, "flagged shows only what is over a limit");
assertEqual(sentinel.list(all, "port").length, 3, "ports shows every listener");
assertEqual(sentinel.list(all, "port")[0].id, "port:8080:30", "the longest-running listener leads");
assertEqual(sentinel.list(all, "cpu").length, 0, "cpu shows only CPU findings");
assertEqual(sentinel.list(all, "port", "ssh").length, 0, "a query filters rows");
assertEqual(sentinel.list(all, "port", "3000").length, 1, "a query matches the port text");
const chips = sentinel.kindChips(all);
assertEqual(chips.map((c) => c.id).join(","), "flagged,cpu,memory,port", "chips are in order");
assertEqual(chips[0].count, 1, "the flagged chip counts findings");
assertEqual(chips[3].count, 3, "the ports chip counts every listener");
assertEqual(sentinel.rows(null, null, th).length, 0, "no sample gives no rows");
assertEqual(sentinel.list(null, "flagged").length, 0, "no rows gives an empty list");

// One process with several old listeners is one flagged row, but every port stays in Ports
const many = sample(1, [proc(40, { comm: "steam", startTicks: 100 })], {
  tcpSockets: [
    { state: "LISTEN", port: 27036, addr: "0.0.0.0", uid: 1000, pid: 40 },
    { state: "LISTEN", port: 32853, addr: "127.0.0.1", uid: 1000, pid: 40 },
    { state: "LISTEN", port: 33037, addr: "127.0.0.1", uid: 1000, pid: 40 },
    { state: "LISTEN", port: 57343, addr: "127.0.0.1", uid: 1000, pid: 40 },
  ],
});
const manyRows = sentinel.rows(many, null, th);
const manyFlagged = sentinel.list(manyRows, "flagged");
assertEqual(manyFlagged.length, 1, "four old listeners of one process are one flagged row");
assertEqual(manyFlagged[0].id, "ports:40", "the merged row is named for the process");
assertEqual(
  manyFlagged[0].detail,
  "process up 20 d · :27036 all interfaces, :32853 local only, :33037 local only, +1 more",
  "the merged row leads with the age, then lists the first ports and counts the rest",
);
assertEqual(manyFlagged[0].proc.pid, 40, "the merged row keeps the process to signal");
assertEqual(sentinel.list(manyRows, "port").length, 4, "the Ports view still lists every port");
assertEqual(sentinel.counts(manyRows).flagged, 1, "the flagged count follows the merged rows");
assertEqual(sentinel.counts(manyRows).port, 4, "the port count stays per port");
assertEqual(
  sentinel.list(manyRows, "flagged", "57343").length,
  1,
  "a search finds a port that the merged row only counts as +1 more",
);
