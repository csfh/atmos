const { load, assert, assertEqual } = require("./harness");

const charts = load("services/Charts.js");

assertEqual(charts.heatmapCells([], 100, 40).length, 0, "heatmap empty input is empty");
assertEqual(
  charts.heatmapCells(
    [
      [1, 2],
      [3, 4],
    ],
    0,
    40,
  ).length,
  0,
  "heatmap needs width",
);
const heat = charts.heatmapCells(
  [
    { id: "NMI", values: [1, 3] },
    { id: "RES", values: [2, 8] },
  ],
  100,
  40,
);
assertEqual(heat.length, 4, "heatmap emits a cell per matrix entry");
assertEqual(heat[0].x, 0, "heatmap first column is x=0");
assertEqual(heat[1].x, 50, "heatmap second column is half width");
assertEqual(heat[2].y, 20, "heatmap second row is half height");
const fills = heat.map(function (c) {
  return c.fill;
});
assert(fills[3] > fills[0], "heatmap maps a larger value to a darker fill");
assertEqual(heat[0].fill, 0, "heatmap min value fill is 0");
assertEqual(heat[3].fill, 1, "heatmap max value fill is 1");

assertEqual(charts.ridgelinePaths([], 100, 40).length, 0, "ridgeline empty input is empty");
const ridges = charts.ridgelinePaths(
  [
    { id: "cpu", values: [0, 2, 4] },
    { id: "memory", values: [0, 1, 2] },
  ],
  90,
  40,
);
assertEqual(ridges.length, 2, "ridgeline emits one path per series");
assertEqual(ridges[0].points[0][0], ridges[1].points[0][0], "ridgeline series share an x-scale");
assertEqual(ridges[0].points[2][0], 90, "ridgeline last x is width");
assert(ridges[0].points[2][1] < ridges[1].points[2][1], "ridgeline larger series peaks higher");
assert(ridges[1].baseline > ridges[0].baseline, "ridgeline offsets series in y");

assertEqual(charts.horizonBands([], 100, 20, 3).length, 0, "horizon empty input is empty");
const horizon = charts.horizonBands([0, 3, 6, 9], 90, 30, 3);
assertEqual(horizon.length, 3, "horizon folds into the requested band count");
assertEqual(horizon[0].layer, 0, "horizon first band is layer 0");
assert(horizon[2].fill > horizon[0].fill, "horizon darker fill is a larger band");
assertEqual(horizon[0].points.length, 4, "horizon each band follows the time series");
const hi = horizon[2].points[3][1];
const lo = horizon[2].points[0][1];
assert(hi < lo, "horizon a large value occupies more of the top band");

assertEqual(charts.treemapRects(null, 100, 100).length, 0, "treemap empty input is empty");
const tree = {
  id: "rss",
  children: [
    {
      id: "uid:1000",
      children: [
        { id: "firefox", value: 60 },
        { id: "term", value: 20 },
      ],
    },
    { id: "uid:0", children: [{ id: "sshd", value: 20 }] },
  ],
};
const trects = charts.treemapRects(tree, 100, 50);
assert(trects.length >= 3, "treemap emits nested rectangles");
const area = function (r) {
  return r.w * r.h;
};
const parent = trects.filter(function (r) {
  return r.depth === 0;
});
const kids = trects.filter(function (r) {
  return r.depth === 1;
});
const parentArea = parent.reduce(function (s, r) {
  return s + area(r);
}, 0);
const kidArea = kids.reduce(function (s, r) {
  return s + area(r);
}, 0);
assert(Math.abs(parentArea - 5000) < 1, "treemap roots fill the parent");
assert(Math.abs(kidArea - parentArea) < 1, "treemap children fill the parent");
const ff = trects.find(function (r) {
  return r.id === "firefox";
});
const sshd = trects.find(function (r) {
  return r.id === "sshd";
});
assert(!!ff && !!sshd, "treemap keeps leaf ids");
assert(Math.abs(area(ff) / area(sshd) - 3) < 0.05, "treemap leaf area is proportional to RSS");

assertEqual(charts.violinPaths([], 100, 40).length, 0, "violin empty input is empty");
const violins = charts.violinPaths(
  [
    { id: "0", values: [3000, 3000, 3100, 2900, 3000] },
    { id: "1", values: [1000, 4000, 1500, 3500, 2000] },
  ],
  100,
  40,
);
assertEqual(violins.length, 2, "violin emits one shape per core");
assert(violins[0].points.length > 4, "violin is a mirrored density path");
const left = violins[0].points[0][0];
const right = violins[0].points[violins[0].points.length - 1][0];
assert(right > violins[0].x && violins[0].x > left, "violin mirrors around the core's x");

assertEqual(charts.beeswarmPoints([], 100, 40).length, 0, "beeswarm empty input is empty");
const bees = charts.beeswarmPoints(
  [
    { id: "1", group: "ESTABLISHED" },
    { id: "2", group: "ESTABLISHED" },
    { id: "3", group: "TIME_WAIT" },
    { id: "4", group: "LISTEN" },
  ],
  90,
  40,
);
assertEqual(bees.length, 4, "beeswarm emits one point per socket");
const est = bees.filter(function (p) {
  return p.group === "ESTABLISHED";
});
assertEqual(est.length, 2, "beeswarm groups ESTABLISHED together");
assert(Math.abs(est[0].x - est[1].x) < 30, "beeswarm jitter stays inside the group");
assert(
  bees[0].x !== bees[2].x || bees[0].group === bees[2].group,
  "beeswarm different groups sit apart",
);

assertEqual(charts.nightingaleWedges([], 100, 100).length, 0, "rose empty input is empty");
const rose = charts.nightingaleWedges(
  [
    { id: "NET_RX", value: 4 },
    { id: "TIMER", value: 1 },
    { id: "SCHED", value: 1 },
    { id: "RCU", value: 1 },
  ],
  100,
  100,
);
assertEqual(rose.length, 4, "rose emits equal-angle wedges");
assert(Math.abs(rose[0].theta - rose[1].theta) < 1e-9, "rose wedges share an angle");
assert(Math.abs(rose[0].area / rose[1].area - 4) < 1e-6, "rose area scales with value, not radius");
assert(Math.abs(rose[0].r / rose[1].r - 2) < 1e-6, "rose radius is sqrt of the value ratio");

assertEqual(charts.sankeyLayout([], [], 100, 40).links.length, 0, "sankey empty input is empty");
const sankey = charts.sankeyLayout(
  [
    { id: "total", col: 0 },
    { id: "anon", col: 1 },
    { id: "file", col: 1 },
  ],
  [
    { source: "total", target: "anon", value: 80 },
    { source: "total", target: "file", value: 20 },
  ],
  100,
  50,
);
assertEqual(sankey.links.length, 2, "sankey emits a link per flow");
assert(
  Math.abs(sankey.links[0].width / sankey.links[1].width - 4) < 0.05,
  "sankey link width is proportional to flow",
);
assert(sankey.nodes.length >= 2, "sankey lays out nodes");

assertEqual(
  charts.parallelPolylines([], ["cpu", "rss"], 100, 40).length,
  0,
  "parallel empty input is empty",
);
const lines = charts.parallelPolylines(
  [
    { id: "1", cpu: 10, rss: 100, fds: 8, threads: 2, nice: 0 },
    { id: "2", cpu: 50, rss: 400, fds: 20, threads: 8, nice: 5 },
  ],
  ["cpu", "rss", "fds", "threads", "nice"],
  100,
  40,
);
assertEqual(lines.length, 2, "parallel emits one polyline per process");
assertEqual(lines[0].points.length, 5, "parallel visits every axis");
assertEqual(lines[0].points[0][0], 0, "parallel first axis is x=0");
assertEqual(lines[0].points[4][0], 100, "parallel last axis is width");
assert(lines[1].points[0][1] < lines[0].points[0][1], "parallel higher CPU sits higher");

assertEqual(charts.calendarCells([], 100, 70).length, 0, "calendar empty input is empty");
const cal = charts.calendarCells(
  [
    { date: "2026-09-01", value: 40 },
    { date: "2026-09-02", value: 80 },
    { date: "2026-09-08", value: 50 },
  ],
  70,
  70,
);
assertEqual(cal.length, 3, "calendar emits a cell per day");
assertEqual(cal[0].weekday, 2, "calendar 2026-09-01 is a Tuesday");
assert(cal[2].week > cal[0].week, "calendar next week is a new column");
assert(cal[1].fill > cal[0].fill, "calendar hotter daily-max is darker");
assertEqual(
  cal.filter(function (c) {
    return c.weekday === 2;
  }).length,
  2,
  "calendar weekdays share a row",
);

assertEqual(charts.streamgraphLayers([], 100, 40).length, 0, "streamgraph empty input is empty");
const stream = charts.streamgraphLayers(
  [
    { id: "sda", values: [10, 10, 10] },
    { id: "nvme0n1", values: [10, 10, 10] },
  ],
  90,
  40,
);
assertEqual(stream.length, 2, "streamgraph emits a layer per disk");
assertEqual(stream[0].top.length, 3, "streamgraph follows the sample window");
const mid = 1;
assert(
  Math.abs((stream[0].top[mid][1] + stream[1].bottom[mid][1]) / 2 - 20) < 1,
  "streamgraph stacks about the centerline",
);

assertEqual(charts.sunburstArcs(null, 100, 100).length, 0, "sunburst empty input is empty");
const sun = charts.sunburstArcs(
  {
    id: "",
    children: [
      {
        id: "user.slice",
        value: 80,
        children: [{ id: "app.service", value: 80 }],
      },
      { id: "system.slice", value: 20 },
    ],
  },
  100,
  100,
);
assert(sun.length >= 3, "sunburst emits concentric arcs");
const rootArc = sun.find(function (a) {
  return a.depth === 0;
});
const user = sun.find(function (a) {
  return a.id === "user.slice";
});
const sys = sun.find(function (a) {
  return a.id === "system.slice";
});
const app = sun.find(function (a) {
  return a.id === "app.service";
});
assert(!!rootArc && !!user && !!sys && !!app, "sunburst keeps slice ids");
assert(user.innerR > rootArc.innerR, "sunburst radius follows cgroup depth");
assert(app.innerR > user.innerR, "sunburst child ring is farther out");
assert(
  Math.abs((user.end - user.start) / (sys.end - sys.start) - 4) < 0.05,
  "sunburst angle is proportional to memory.current",
);

assertEqual(
  charts.radarPolygons([], ["rx", "tx", "packets"], 100, 100).polygons.length,
  0,
  "radar empty input is empty",
);
const radar = charts.radarPolygons(
  [
    { id: "eth0", rx: 100, tx: 50, packets: 10, drops: 0, errs: 1 },
    { id: "wlan0", rx: 50, tx: 25, packets: 5, drops: 0, errs: 0 },
  ],
  ["rx", "tx", "packets", "drops", "errs"],
  100,
  100,
);
assertEqual(radar.axes.length, 5, "radar has a spoke per counter");
assertEqual(radar.polygons.length, 2, "radar emits one polygon per NIC");
assertEqual(radar.polygons[0].points.length, 5, "radar polygon visits every axis");

assertEqual(charts.waterfallBars([], 100, 40).length, 0, "waterfall empty input is empty");
const fall = charts.waterfallBars(
  [
    { id: "package", value: 10 },
    { id: "core", value: 4 },
    { id: "dram", value: -2 },
  ],
  90,
  40,
);
assertEqual(fall.length, 3, "waterfall emits a bar per RAPL domain");
assertEqual(fall[0].origin, 0, "waterfall first origin is 0");
assertEqual(fall[1].origin, 10, "waterfall bar i origin is the signed sum of 0..i-1");
assertEqual(fall[2].origin, 14, "waterfall tracks a running total through a negative step");

assertEqual(charts.icicleRects(null, 100, 40).length, 0, "icicle empty input is empty");
const ice = charts.icicleRects(
  {
    id: "slab",
    children: [
      {
        id: "kmalloc",
        children: [
          { id: "kmalloc-8", value: 30 },
          { id: "kmalloc-16", value: 10 },
        ],
      },
      { id: "dentry", value: 10 },
    ],
  },
  100,
  30,
);
assert(ice.length >= 4, "icicle emits cascading rectangles");
const slabRoot = ice.find(function (r) {
  return r.id === "slab";
});
const km = ice.find(function (r) {
  return r.id === "kmalloc";
});
const k8 = ice.find(function (r) {
  return r.id === "kmalloc-8";
});
assert(!!slabRoot && !!km && !!k8, "icicle keeps cache ids");
assertEqual(slabRoot.depth, 0, "icicle root depth is 0");
assertEqual(k8.depth, 2, "icicle depth follows the cache hierarchy");
assert(Math.abs(k8.w / km.w - 0.75) < 0.05, "icicle length is proportional to slab occupancy");

const rates = charts.irqRateMatrix(
  { rows: [{ id: "NMI", values: [10, 20] }] },
  { rows: [{ id: "NMI", values: [12, 25] }] },
);
assertEqual(rates[0].values.join(","), "2,5", "irqRateMatrix is a per-CPU delta");
assertEqual(
  charts.irqRateMatrix(null, { rows: [] }).length,
  0,
  "irqRateMatrix empty next is empty",
);

const psi = charts.psiRidges([
  { psi: { cpu: 1, memory: 2, io: 3 } },
  { psi: { cpu: 2, memory: 2, io: 4 } },
]);
assertEqual(psi[0].values.join(","), "1,2", "psiRidges keeps cpu avg10");
assertEqual(psi[2].id, "io", "psiRidges includes io");

assertEqual(
  charts.buddySeries([{ buddy: [1, 0, 1] }, { buddy: [1, 0, 2] }]).join(","),
  "5,9",
  "buddySeries weights orders",
);

const rss = charts.rssTree([
  { uid: 1000, comm: "firefox", rssKb: 60 },
  { uid: 1000, comm: "term", rssKb: 20 },
  { uid: 0, comm: "sshd", rssKb: 20 },
  { uid: 1000, comm: "idle", rssKb: 0 },
]);
assertEqual(rss.children.length, 2, "rssTree nests by user");
assertEqual(charts.rssTree([]), null, "rssTree empty is null");

assertEqual(
  charts.tcpBees({ tcpSockets: [{ inode: 9, state: "ESTABLISHED" }] }).length,
  1,
  "tcpBees one point per socket",
);
assertEqual(charts.meminfoSankey(null).links.length, 0, "meminfoSankey missing sample is empty");
const flow = charts.meminfoSankey({ memTotal: 1000, memAnon: 400, memSlab: 100 });
assertEqual(flow.links.length, 2, "meminfoSankey drops unknown buckets");
assertEqual(
  charts.processParallel([{ pid: 1, cpu: 1, rssKb: 1, threads: 1, nice: 0 }]).length,
  0,
  "processParallel needs fds",
);
assertEqual(
  charts.processParallel([{ pid: 1, cpu: 1, rssKb: 1, fds: 3, threads: 1, nice: 0 }]).length,
  1,
  "processParallel keeps a complete row",
);
assertEqual(charts.raplSteps(null, { rapl: [] }).length, 0, "raplSteps empty is empty");
const rapl = charts.raplSteps(
  { rapl: [{ id: "package-0", uj: 1e6 }] },
  { rapl: [{ id: "package-0", uj: 3e6 }] },
);
assertEqual(rapl[0].value, 2, "raplSteps converts microjoules to joules");
assertEqual(charts.slabTree([]), null, "slabTree empty is null");
assert(!!charts.slabTree([{ name: "kmalloc-8", active: 10 }]), "slabTree keeps a cache");
