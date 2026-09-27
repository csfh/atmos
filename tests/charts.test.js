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
assert(heat[0].w < 50, "heatmap cell is narrower than its column slot");
assert(heat[1].x > heat[0].x + heat[0].w, "heatmap columns leave a gap");
assert(heat[2].y > heat[0].y + heat[0].h, "heatmap rows leave a gap");
assert(Math.abs(heat[1].x + heat[1].w - 100) < 1e-9, "heatmap last column reaches the right edge");
assert(Math.abs(heat[2].y + heat[2].h - 40) < 1e-9, "heatmap last row reaches the bottom");
const fills = heat.map(function (c) {
  return c.fill;
});
assert(fills[3] > fills[0], "heatmap maps a larger value to a darker fill");
assertEqual(heat[0].fill, 0, "heatmap min value fill is 0");
assertEqual(heat[3].fill, 1, "heatmap max value fill is 1");
const heatOne = charts.heatmapCells([{ id: "NMI", values: [4] }], 100, 40);
assertEqual(heatOne[0].w, 100, "a single heatmap cell fills the width");
assertEqual(heatOne[0].h, 40, "a single heatmap cell fills the height");
const heatDense = charts.heatmapCells(
  Array.from({ length: 40 }, function (_, i) {
    return { id: String(i), values: [i] };
  }),
  100,
  40,
);
assert(heatDense[0].h > 0, "a dense heatmap keeps a positive cell height");
assertEqual(heatDense[1].y, heatDense[0].y + heatDense[0].h, "a dense heatmap stays flush");
const heatIdle = charts.heatmapCells(
  [
    { id: "idle", values: [null, null] },
    { id: "hot", values: [1, 8] },
  ],
  100,
  40,
);
assertEqual(heatIdle.length, 2, "heatmap skips a row with no finite values");
assertEqual(heatIdle[0].row, "hot", "heatmap empty rows do not steal a slot");
assertEqual(heatIdle[0].y, 0, "heatmap first finite row is at y=0");
assertEqual(heatIdle[0].h, 40, "heatmap a single finite row fills the height");
const heatOverflow = charts.heatmapCells(
  Array.from({ length: 80 }, function (_, i) {
    return { id: String(i), values: [i] };
  }),
  100,
  40,
);
assertEqual(heatOverflow.length, 40, "heatmap keeps one row per pixel");
assertEqual(heatOverflow[0].row, "40", "heatmap keeps the hottest rows");
assertEqual(heatOverflow[39].row, "79", "heatmap preserves row order among keepers");
assert(heatOverflow[0].h >= 1, "heatmap capped rows still fill a pixel");
assert(
  Math.abs(heatOverflow[39].y + heatOverflow[39].h - 40) < 1e-9,
  "heatmap capped last row reaches the bottom",
);
const heatWide = charts.heatmapCells(
  [
    {
      id: "NMI",
      values: Array.from({ length: 80 }, function (_, i) {
        return i;
      }),
    },
  ],
  40,
  40,
);
assertEqual(heatWide.length, 40, "heatmap keeps one column per pixel");
assertEqual(heatWide[0].value, 40, "heatmap keeps the hottest columns");
assertEqual(heatWide[39].value, 79, "heatmap last kept column is the hottest");
assert(
  Math.abs(heatWide[39].x + heatWide[39].w - 40) < 1e-9,
  "heatmap capped last column reaches the right edge",
);
const heatSkipCpu = charts.heatmapCells([{ id: "NMI", values: [null, 4, 1] }], 100, 40);
assertEqual(heatSkipCpu[0].col, 1, "heatmap keeps the original CPU index");
assertEqual(charts.heatmapLayout([], 100, 40).labels.length, 0, "heatmap layout empty is empty");
const heatL = charts.heatmapLayout(
  [
    { id: "NMI", values: [1, 3] },
    { id: "RES", values: [2, 8] },
  ],
  100,
  40,
);
assertEqual(heatL.cells.length, 4, "heatmap layout keeps cells");
assert(heatL.cells[0].x > 0, "heatmap cells leave a row-label band");
assert(heatL.cells[0].y > 0, "heatmap cells leave a cpu-label band");
const heatRows = heatL.labels.filter(function (l) {
  return l.align === "right";
});
const heatCols = heatL.labels.filter(function (l) {
  return l.baseline === "top";
});
assertEqual(heatRows.length, 2, "heatmap labels each IRQ");
assertEqual(heatRows[0].label, "NMI", "heatmap first row is NMI");
assertEqual(heatRows[1].label, "RES", "heatmap keeps RES");
assertEqual(heatRows[0].align, "right", "heatmap IRQ names sit left of the grid");
assert(heatRows[0].lx <= heatL.cells[0].x, "heatmap IRQ names stay in the left band");
assertEqual(heatCols.length, 2, "heatmap labels each CPU");
assertEqual(heatCols[0].label, "0", "heatmap first cpu is 0");
assertEqual(heatCols[1].label, "1", "heatmap keeps cpu 1");
assertEqual(heatCols[0].baseline, "top", "heatmap cpu ids sit above the grid");
const heatSkipL = charts.heatmapLayout([{ id: "NMI", values: [null, 4, 1] }], 100, 40);
const skipCols = heatSkipL.labels.filter(function (l) {
  return l.baseline === "top";
});
assertEqual(skipCols[0].label, "1", "heatmap labels the kept CPU id");
assertEqual(skipCols[1].label, "2", "heatmap keeps the next CPU id");

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
assert(ridges[0].points[0][0] > 0, "ridgeline plot leaves a caption band");
const ridgeScene = charts.ridgelineLayout(
  [
    { id: "cpu", values: [0, 2, 4] },
    { id: "memory", values: [0, 1, 2] },
  ],
  90,
  40,
);
assertEqual(ridgeScene.labels.length, 2, "ridgeline emits a label per series");
assertEqual(ridgeScene.labels[0].label, "cpu", "ridgeline first label is cpu");
assertEqual(ridgeScene.labels[1].label, "memory", "ridgeline keeps memory");
assertEqual(ridgeScene.labels[0].align, "left", "ridgeline labels sit on the left");
assertEqual(ridgeScene.labels[0].lx, 0, "ridgeline labels are on the left edge");
assertEqual(ridgeScene.labels[0].baseline, "middle", "ridgeline labels are vertically centered");
assertEqual(
  ridgeScene.labels[0].ly,
  ridgeScene.ridges[0].baseline,
  "ridgeline labels follow the series baseline",
);
assert(
  ridgeScene.labels[0].maxW <= ridgeScene.ridges[0].points[0][0] + 0.01,
  "ridgeline labels stay in the caption band",
);
assert(
  ridgeScene.labels[0].ly >= 0 && ridgeScene.labels[0].ly <= 40,
  "ridgeline label y stays on canvas",
);
const ridgeBig = charts.ridgelineLayout(
  [
    { id: "cpu", values: [0, 2, 4] },
    { id: "memory", values: [0, 1, 2] },
  ],
  180,
  80,
);
const ridgeMid = charts.lerpScene(ridgeScene, ridgeBig, 0.5);
assertEqual(ridgeMid.labels.length, 2, "lerpScene keeps ridgeline labels");
assert(
  Math.abs(ridgeMid.labels[1].ly - (ridgeScene.labels[1].ly + ridgeBig.labels[1].ly) / 2) < 1e-9,
  "lerpScene tweens ridgeline label y",
);

assertEqual(charts.horizonBands([], 100, 20, 3).length, 0, "horizon empty input is empty");
const horizon = charts.horizonBands([0, 3, 6, 9], 90, 30, 3);
assertEqual(horizon.length, 3, "horizon folds into the requested band count");
assertEqual(horizon[0].layer, 0, "horizon first band is layer 0");
assert(horizon[2].fill > horizon[0].fill, "horizon darker fill is a larger band");
assertEqual(horizon[0].points.length, 4, "horizon each band follows the time series");
const hi = horizon[2].points[3][1];
const lo = horizon[2].points[0][1];
assert(hi < lo, "horizon a large value occupies more of the top band");
assertEqual(charts.horizonLayout([], 100, 20, 3).labels.length, 0, "horizon layout empty is empty");
const horizonScene = charts.horizonLayout([0, 3, 6, 9], 90, 30, 3);
assertEqual(horizonScene.bands.length, 3, "horizon layout keeps bands");
assert(horizonScene.bands[0].points[0][0] > 0, "horizon plot leaves a caption band");
assertEqual(horizonScene.labels.length, 3, "horizon emits a label per fold");
assertEqual(horizonScene.labels[0].label, "3", "horizon first label is the base fold");
assertEqual(horizonScene.labels[1].label, "6", "horizon keeps the middle fold");
assertEqual(horizonScene.labels[2].label, "9", "horizon keeps the top fold");
assertEqual(horizonScene.labels[0].align, "right", "horizon fold ceilings sit left of the plot");
assert(
  horizonScene.labels[0].lx <= horizonScene.bands[0].points[0][0],
  "horizon fold ceilings stay in the left band",
);
assertEqual(horizonScene.labels[0].baseline, "middle", "horizon labels are vertically centered");
assert(
  horizonScene.labels[0].maxW <= horizonScene.bands[0].points[0][0] + 0.01,
  "horizon labels stay in the caption band",
);
assert(
  horizonScene.labels[0].ly >= 0 && horizonScene.labels[0].ly <= 30,
  "horizon label y stays on canvas",
);
assert(
  horizonScene.labels[2].ly < horizonScene.labels[0].ly,
  "horizon darker folds sit above the base fold",
);
const horizonHuge = charts.horizonLayout([0, 1048576], 90, 30, 2);
assertEqual(horizonHuge.labels[1].label, "1048576", "horizon keeps the full fold ceiling");
const horizonShort = charts.horizonLayout([0, 3, 6, 9], 90, 14, 3);
assertEqual(horizonShort.labels.length, 3, "horizon labels every fold even on a short canvas");
const horizonBig = charts.horizonLayout([0, 3, 6, 9], 180, 60, 3);
const horizonMid = charts.lerpScene(horizonScene, horizonBig, 0.5);
assertEqual(horizonMid.labels.length, 3, "lerpScene keeps horizon labels");
assert(
  Math.abs(horizonMid.labels[2].ly - (horizonScene.labels[2].ly + horizonBig.labels[2].ly) / 2) <
    1e-9,
  "lerpScene tweens horizon label y",
);

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
assertEqual(ff.label, "firefox", "treemap leaf label is the comm");
assertEqual(sshd.label, "sshd", "treemap keeps sshd");
assertEqual(ff.align, "center", "treemap labels are centered in the leaf");
assertEqual(ff.baseline, "middle", "treemap labels are vertically centered");
assert(Math.abs(ff.lx - (ff.x + ff.w / 2)) < 1e-9, "treemap label x is the leaf center");
assert(Math.abs(ff.ly - (ff.y + ff.h / 2)) < 1e-9, "treemap label y is the leaf center");
assert(ff.maxW <= ff.w + 0.01, "treemap labels stay inside the leaf");
const uid = trects.find(function (r) {
  return r.id === "uid:1000";
});
assert(!!uid && uid.label == null, "treemap parents stay unlabeled");
const trectsBig = charts.treemapRects(tree, 200, 100);
const treemapMid = charts.lerpScene({ rects: trects }, { rects: trectsBig }, 0.5);
const ffMid = treemapMid.rects.find(function (r) {
  return r.id === "firefox";
});
const ffBig = trectsBig.find(function (r) {
  return r.id === "firefox";
});
assert(!!ffMid && !!ffBig, "lerpScene keeps treemap leaf labels");
assert(Math.abs(ffMid.lx - (ff.lx + ffBig.lx) / 2) < 1e-9, "lerpScene tweens treemap label x");
assertEqual(charts.findTreeNode(tree, "firefox").id, "firefox", "findTreeNode finds a leaf");
assertEqual(
  charts.treemapParentId(tree, "firefox"),
  "uid:1000",
  "treemapParentId of a leaf is the user",
);
assertEqual(
  charts.treemapParentId(tree, "uid:1000"),
  "",
  "treemapParentId of a top group is the root",
);
const focused = charts.treemapRects(tree, 100, 50, "uid:1000");
const focusedIds = focused.map(function (r) {
  return r.id;
});
assert(focusedIds.indexOf("firefox") !== -1, "focused treemap keeps the user's processes");
assert(focusedIds.indexOf("sshd") === -1, "focused treemap drops other users");
assert(focusedIds.indexOf("uid:1000") === -1, "focused treemap does not redraw the parent box");
const uidHit = charts.treemapHit(trects, uid.x + uid.w / 2, uid.y + uid.h / 2);
assert(
  uidHit && (uidHit.id === "firefox" || uidHit.id === "term"),
  "treemapHit picks the process under the cursor",
);
const zoomed = charts.treemapLayout(tree, 100, 50, "uid:1000");
assertEqual(zoomed.labels[0].label, "uid:1000", "zoomed treemap paints a crumb");
assert(zoomed.rects[0].y > 0, "zoomed treemap leaves a crumb band");

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
assertEqual(violins[0].label, "0", "violin first label is core 0");
assertEqual(violins[1].label, "1", "violin keeps core 1");
assertEqual(violins[0].baseline, "bottom", "violin labels sit on the bottom edge");
assertEqual(violins[0].ly, 40, "violin labels are on the canvas floor");
assertEqual(violins[0].align, "center", "violin labels are centered on the core");
assert(violins[0].maxW <= 50 + 0.01, "violin labels stay inside the core slot");
assert(violins[0].points[0][1] <= violins[0].ly - 8, "violin shapes sit above the label band");
const violinBig = charts.violinPaths(
  [
    { id: "0", values: [3000, 3000, 3100, 2900, 3000] },
    { id: "1", values: [1000, 4000, 1500, 3500, 2000] },
  ],
  200,
  80,
);
const violinMid = charts.lerpScene({ violins: violins }, { violins: violinBig }, 0.5);
assertEqual(violinMid.violins.length, 2, "lerpScene keeps violin labels");
assert(
  Math.abs(violinMid.violins[1].lx - (violins[1].lx + violinBig[1].lx) / 2) < 1e-9,
  "lerpScene tweens violin label x",
);
const violinCrowd = charts.violinPaths(
  Array.from({ length: 20 }, function (_, i) {
    return {
      id: String(i),
      values: i < 4 ? [1000, 4000, 2000, 3000] : [3000, 3000, 3001, 3000],
    };
  }),
  100,
  40,
);
assert(violinCrowd.length <= Math.floor(100 / 16), "violin drops cores that would collapse");
assertEqual(violinCrowd.length, 6, "violin keeps a readable slot per core");
assertEqual(
  violinCrowd
    .map(function (v) {
      return v.id;
    })
    .slice(0, 4)
    .join(","),
  "0,1,2,3",
  "violin keeps the cores with P-state spread",
);
assert(violinCrowd[0].maxW >= 16, "violin remaining cores have a readable slot");

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
assert(bees[0].r >= 2, "beeswarm marks have a readable radius");
assertEqual(bees[0].r, bees[3].r, "beeswarm marks share a radius");
assert(bees[0].y >= bees[0].r, "beeswarm marks stay inside the top edge");
assert(bees[0].y + bees[0].r <= 40, "beeswarm marks stay inside the bottom edge");
const beesBig = charts.beeswarmPoints(
  [
    { id: "1", group: "ESTABLISHED" },
    { id: "2", group: "ESTABLISHED" },
    { id: "3", group: "TIME_WAIT" },
    { id: "4", group: "LISTEN" },
  ],
  240,
  96,
);
assert(beesBig[0].r > bees[0].r, "beeswarm mark size grows with the canvas");
assert(beesBig[0].r <= 6, "beeswarm marks stay smaller than the group slot");
const beesMid = charts.lerpScene({ points: bees }, { points: beesBig }, 0.5);
assertEqual(beesMid.points.length, 4, "lerpScene keeps beeswarm points");
assert(
  Math.abs(beesMid.points[0].r - (bees[0].r + beesBig[0].r) / 2) < 1e-9,
  "lerpScene tweens beeswarm mark radius",
);
const swarm = charts.beeswarmLayout(
  [
    { id: "1", group: "ESTABLISHED" },
    { id: "2", group: "ESTABLISHED" },
    { id: "3", group: "TIME_WAIT" },
    { id: "4", group: "LISTEN" },
  ],
  90,
  40,
);
assertEqual(swarm.labels.length, 3, "beeswarm emits a label per TCP state");
assertEqual(swarm.labels[0].label, "ESTABLISHED", "beeswarm first label follows first group");
assertEqual(swarm.labels[1].label, "TIME_WAIT", "beeswarm keeps TIME_WAIT");
assertEqual(swarm.labels[2].label, "LISTEN", "beeswarm last label is LISTEN");
assertEqual(swarm.labels[0].baseline, "bottom", "beeswarm labels sit on the bottom edge");
assertEqual(swarm.labels[0].ly, 40, "beeswarm labels are on the canvas floor");
assert(swarm.labels[0].maxW <= 90 / 3, "beeswarm labels stay inside the group slot");
assert(
  swarm.points[0].y + swarm.points[0].r <= swarm.labels[0].ly - 8,
  "beeswarm marks sit above the label band",
);
const swarmBig = charts.beeswarmLayout(
  [
    { id: "1", group: "ESTABLISHED" },
    { id: "2", group: "ESTABLISHED" },
    { id: "3", group: "TIME_WAIT" },
    { id: "4", group: "LISTEN" },
  ],
  240,
  96,
);
const swarmMid = charts.lerpScene(swarm, swarmBig, 0.5);
assertEqual(swarmMid.labels.length, 3, "lerpScene keeps beeswarm labels");
assert(
  Math.abs(swarmMid.labels[2].lx - (swarm.labels[2].lx + swarmBig.labels[2].lx) / 2) < 1e-9,
  "lerpScene tweens beeswarm label x",
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
assertEqual(rose[0].label, "NET_RX", "rose first wedge keeps its irq name");
assertEqual(rose[1].label, "TIMER", "rose keeps TIMER");
assert(rose[0].r < 50 && rose[0].r > 24, "rose largest wedge leaves a caption band");
assertEqual(rose[0].align, "left", "rose NE label is left-aligned");
assertEqual(rose[0].baseline, "bottom", "rose NE label sits above the wedge");
assert(rose[0].lx >= 0 && rose[0].lx <= 100, "rose label x stays on canvas");
assert(rose[0].ly >= 0 && rose[0].ly <= 100, "rose label y stays on canvas");
assert(rose[0].lx > rose[0].cx, "rose first label sits outside the ring in x");
assert(rose[0].ly < rose[0].cy, "rose first label sits outside the ring in y");
const roseBig = charts.nightingaleWedges(
  [
    { id: "NET_RX", value: 4 },
    { id: "TIMER", value: 1 },
    { id: "SCHED", value: 1 },
    { id: "RCU", value: 1 },
  ],
  200,
  200,
);
const roseLabelMid = charts.lerpScene({ wedges: rose }, { wedges: roseBig }, 0.5);
assertEqual(roseLabelMid.wedges[0].label, "NET_RX", "lerpScene keeps rose labels");
assert(
  Math.abs(roseLabelMid.wedges[0].lx - (rose[0].lx + roseBig[0].lx) / 2) < 1e-9,
  "lerpScene tweens rose label x",
);

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
const sankeyTotal = sankey.nodes.find(function (n) {
  return n.id === "total";
});
const sankeyAnon = sankey.nodes.find(function (n) {
  return n.id === "anon";
});
const sankeyFile = sankey.nodes.find(function (n) {
  return n.id === "file";
});
assert(!!sankeyFile, "sankey keeps the file node");
assert(sankey.links[0].y0 < sankey.links[1].y0, "sankey outgoing links stack along the source");
assert(Math.abs(sankey.links[0].y0 - sankey.links[1].y0) > 1, "sankey flows do not share one y");
assert(sankey.links[0].width <= sankeyAnon.h + 0.01, "sankey large link fits the target node");
assert(sankey.links[1].width <= sankeyFile.h + 0.01, "sankey small link fits its node");
assert(
  sankey.links[0].y0 >= sankeyTotal.y && sankey.links[0].y0 <= sankeyTotal.y + sankeyTotal.h,
  "sankey first flow stays on the source",
);
assert(
  sankey.links[1].y1 >= sankeyFile.y && sankey.links[1].y1 <= sankeyFile.y + sankeyFile.h,
  "sankey small flow meets its target",
);
assert(!!sankeyTotal && !!sankeyAnon, "sankey keeps node ids");
assertEqual(sankeyTotal.label, "total", "sankey left node keeps its label");
assertEqual(sankeyAnon.label, "anon", "sankey right node keeps its label");
assertEqual(sankeyTotal.align, "right", "sankey left labels sit left of the bar");
assertEqual(sankeyAnon.align, "left", "sankey right labels sit right of the bar");
assertEqual(sankeyTotal.baseline, "middle", "sankey labels sit on the node midline");
assert(sankeyTotal.x > 0, "sankey left column leaves a label band");
assert(sankeyAnon.x + sankeyAnon.w < 100, "sankey right column leaves a label band");
assert(sankeyTotal.lx <= sankeyTotal.x, "sankey left label is outside the bar");
assert(sankeyAnon.lx >= sankeyAnon.x + sankeyAnon.w, "sankey right label is outside the bar");
assert(
  sankeyTotal.ly >= sankeyTotal.y && sankeyTotal.ly <= sankeyTotal.y + sankeyTotal.h,
  "sankey left label stays on the node",
);
assert(sankeyTotal.maxW <= sankeyTotal.x, "sankey left labels stay inside the band");
const sankeyBig = charts.sankeyLayout(
  [
    { id: "total", col: 0 },
    { id: "anon", col: 1 },
    { id: "file", col: 1 },
  ],
  [
    { source: "total", target: "anon", value: 80 },
    { source: "total", target: "file", value: 20 },
  ],
  200,
  80,
);
const sankeyMid = charts.lerpScene(sankey, sankeyBig, 0.5);
assertEqual(sankeyMid.nodes.length, sankey.nodes.length, "lerpScene keeps sankey nodes");
assert(
  Math.abs(sankeyMid.nodes[0].lx - (sankey.nodes[0].lx + sankeyBig.nodes[0].lx) / 2) < 1e-9,
  "lerpScene tweens sankey label x",
);
const memFlow = charts.meminfoSankey({ memTotal: 1000, memAnon: 400, memSlab: 100 });
const memSankey = charts.sankeyLayout(memFlow.nodes, memFlow.links, 100, 50);
assert(
  memSankey.nodes.some(function (n) {
    return n.label === "MemTotal";
  }),
  "meminfo Sankey labels MemTotal",
);
assert(
  memSankey.nodes.some(function (n) {
    return n.label === "Anon";
  }),
  "meminfo Sankey labels Anon",
);

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
assert(lines[1].points[0][1] > 0, "parallel high values sit below the label band");
const para = charts.parallelLayout(
  [
    { id: "1", cpu: 10, rss: 100, fds: 8, threads: 2, nice: 0 },
    { id: "2", cpu: 50, rss: 400, fds: 20, threads: 8, nice: 5 },
  ],
  ["cpu", "rss", "fds", "threads", "nice"],
  100,
  40,
);
assertEqual(para.rails.length, 5, "parallel emits a rail per field");
assertEqual(para.rails[0].label, "cpu", "parallel first rail is cpu");
assertEqual(para.rails[4].label, "nice", "parallel last rail is nice");
assertEqual(para.rails[0].x0, 0, "parallel first rail is x=0");
assertEqual(para.rails[4].x1, 100, "parallel last rail is width");
assert(para.rails[0].y0 > 0, "parallel rails leave a label band");
assertEqual(para.rails[0].align, "left", "parallel first label is left-aligned");
assertEqual(para.rails[2].align, "center", "parallel inner labels are centered");
assertEqual(para.rails[4].align, "right", "parallel last label is right-aligned");
assertEqual(para.rails[0].baseline, "top", "parallel labels sit on the top edge");
assert(para.polylines[1].points[0][1] >= para.rails[0].y0, "parallel marks stay in the plot");
const paraBig = charts.parallelLayout(
  [
    { id: "1", cpu: 10, rss: 100, fds: 8, threads: 2, nice: 0 },
    { id: "2", cpu: 50, rss: 400, fds: 20, threads: 8, nice: 5 },
  ],
  ["cpu", "rss", "fds", "threads", "nice"],
  200,
  80,
);
const paraMid = charts.lerpScene(para, paraBig, 0.5);
assertEqual(paraMid.rails.length, 5, "lerpScene keeps parallel rails");
assert(
  Math.abs(paraMid.rails[4].x1 - (para.rails[4].x1 + paraBig.rails[4].x1) / 2) < 1e-9,
  "lerpScene tweens parallel rail x",
);
assert(para.rails[2].maxW <= 100 / 4 + 0.01, "parallel labels stay inside the axis slot");
assertEqual(para.rails[0].maxW, para.rails[4].maxW, "parallel rails share a label budget");
const paraCrowd = charts.parallelPolylines(
  Array.from({ length: 40 }, function (_, i) {
    return { id: String(i), cpu: i, rss: 1, fds: 1, threads: 1, nice: 0 };
  }),
  ["cpu", "rss", "fds", "threads", "nice"],
  100,
  40,
);
assert(paraCrowd.length <= Math.floor(40 / 4), "parallel drops rows that would spaghetti");
assertEqual(paraCrowd.length, 10, "parallel keeps a readable polyline budget");
assertEqual(
  paraCrowd
    .map(function (row) {
      return row.id;
    })
    .join(","),
  "30,31,32,33,34,35,36,37,38,39",
  "parallel keeps the highest-scoring processes",
);

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
assert(cal[2].x > cal[0].x + cal[0].w, "calendar weeks leave a gap");
assert(cal[1].y > cal[0].y + cal[0].h, "calendar weekdays leave a gap");
assertEqual(
  cal.filter(function (c) {
    return c.weekday === 2;
  }).length,
  2,
  "calendar weekdays share a row",
);
const calLayout = charts.calendarLayout(
  [
    { date: "2026-09-01", value: 40 },
    { date: "2026-09-02", value: 80 },
    { date: "2026-09-08", value: 50 },
  ],
  70,
  70,
);
assertEqual(calLayout.labels.length, 7, "calendar emits a label per weekday");
assertEqual(calLayout.labels[0].label, "Sun", "calendar first label is Sunday");
assertEqual(calLayout.labels[2].label, "Tue", "calendar keeps Tuesday");
assertEqual(calLayout.labels[6].label, "Sat", "calendar last label is Saturday");
assertEqual(calLayout.labels[0].align, "left", "calendar labels sit on the left");
assertEqual(calLayout.labels[0].lx, 0, "calendar labels are on the left edge");
assertEqual(calLayout.labels[0].baseline, "middle", "calendar labels are vertically centered");
assert(
  calLayout.labels[0].maxW <= calLayout.cells[0].x + 0.01,
  "calendar labels stay in the caption band",
);
assert(calLayout.cells[0].x >= calLayout.labels[0].maxW, "calendar tiles leave a caption band");
assert(
  calLayout.labels[2].ly >= 0 && calLayout.labels[2].ly <= 70,
  "calendar label y stays on canvas",
);
assert(
  Math.abs(calLayout.labels[2].ly - (calLayout.cells[0].y + calLayout.cells[0].h / 2)) < 1e-9,
  "calendar Tuesday label follows the Tuesday row",
);
const calBig = charts.calendarLayout(
  [
    { date: "2026-09-01", value: 40 },
    { date: "2026-09-02", value: 80 },
    { date: "2026-09-08", value: 50 },
  ],
  140,
  140,
);
const calMid = charts.lerpScene(calLayout, calBig, 0.5);
assertEqual(calMid.labels.length, 7, "lerpScene keeps calendar labels");
assert(
  Math.abs(calMid.labels[2].ly - (calLayout.labels[2].ly + calBig.labels[2].ly) / 2) < 1e-9,
  "lerpScene tweens calendar label y",
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
assertEqual(stream[0].label, "sda", "streamgraph first label is the disk id");
assertEqual(stream[1].label, "nvme0n1", "streamgraph keeps nvme0n1");
assertEqual(stream[0].align, "right", "streamgraph labels sit at the latest peak");
assertEqual(stream[0].baseline, "middle", "streamgraph labels sit in the layer");
assert(Math.abs(stream[0].lx - 90) < 1e-9, "streamgraph equal samples label the last x");
assert(
  stream[0].ly > stream[0].top[2][1] && stream[0].ly < stream[0].bottom[2][1],
  "streamgraph first label stays inside its layer",
);
assert(
  stream[1].ly > stream[1].top[2][1] && stream[1].ly < stream[1].bottom[2][1],
  "streamgraph second label stays inside its layer",
);
assert(stream[0].lx >= 0 && stream[0].lx <= 90, "streamgraph label x stays on canvas");
assert(stream[0].ly >= 0 && stream[0].ly <= 40, "streamgraph label y stays on canvas");
assert(stream[0].maxW <= 90 + 0.01, "streamgraph labels stay inside the canvas");
const streamThin = charts.streamgraphLayers(
  [
    { id: "sda", values: [100, 100, 100] },
    { id: "loop0", values: [1, 1, 1] },
  ],
  90,
  40,
);
const loopLayer = streamThin.find(function (l) {
  return l.id === "loop0";
});
assertEqual(loopLayer && loopLayer.label, "loop0", "streamgraph thin layers keep the disk id");
const streamBig = charts.streamgraphLayers(
  [
    { id: "sda", values: [10, 10, 10] },
    { id: "nvme0n1", values: [10, 10, 10] },
  ],
  180,
  80,
);
const streamMid = charts.lerpScene({ layers: stream }, { layers: streamBig }, 0.5);
assertEqual(streamMid.layers.length, 2, "lerpScene keeps streamgraph labels");
assert(
  Math.abs(streamMid.layers[0].lx - (stream[0].lx + streamBig[0].lx) / 2) < 1e-9,
  "lerpScene tweens streamgraph label x",
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
assert(app.outerR < 50 && app.outerR > 24, "sunburst outer ring leaves a caption band");
assert(Math.abs(rootArc.outerR - user.innerR) < 1e-9, "sunburst rings are contiguous");
assert(
  Math.abs((user.end - user.start) / (sys.end - sys.start) - 4) < 0.05,
  "sunburst angle is proportional to memory.current",
);
assertEqual(rootArc.label, undefined, "sunburst empty root stays unlabeled");
assertEqual(user.label, "user.slice", "sunburst parent ring keeps its cgroup name");
assertEqual(sys.label, "system.slice", "sunburst keeps system.slice");
assertEqual(app.label, "app.service", "sunburst leaf keeps its unit name");
assertEqual(user.align, "center", "sunburst inner labels are centered in the ring");
assertEqual(user.baseline, "middle", "sunburst inner labels sit in the ring");
assertEqual(app.align, "left", "sunburst east leaf is left-aligned");
assertEqual(app.baseline, "top", "sunburst south-east leaf sits below the ring");
assert(user.lx >= 0 && user.lx <= 100, "sunburst label x stays on canvas");
assert(user.ly >= 0 && user.ly <= 100, "sunburst label y stays on canvas");
assert(app.lx >= 0 && app.lx <= 100, "sunburst leaf label x stays on canvas");
assert(app.ly >= 0 && app.ly <= 100, "sunburst leaf label y stays on canvas");
const userDx = user.lx - user.cx;
const userDy = user.ly - user.cy;
const userMidR = (user.innerR + user.outerR) / 2;
assert(
  Math.abs(Math.sqrt(userDx * userDx + userDy * userDy) - userMidR) < 1e-9,
  "sunburst inner label sits at mid-radius",
);
const appDx = app.lx - app.cx;
const appDy = app.ly - app.cy;
assert(
  Math.sqrt(appDx * appDx + appDy * appDy) > app.outerR,
  "sunburst outer label sits outside the ring",
);
const sunScene = charts.sunburstLayout(
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
assertEqual(sunScene.labels.length, 3, "sunburst layout emits a label per named slice");
assertEqual(sunScene.labels[0].label, "user.slice", "sunburst layout first label is user.slice");
assertEqual(sunScene.labels[2].label, "system.slice", "sunburst layout keeps system.slice");
const shallow = charts.sunburstArcs({ id: "root", children: [{ id: "leaf", value: 1 }] }, 100, 100);
const shallowRoot = shallow.find(function (a) {
  return a.id === "root";
});
const shallowLeaf = shallow.find(function (a) {
  return a.id === "leaf";
});
assert(!!shallowLeaf, "sunburst keeps a one-child tree");
assert(
  shallowLeaf.outerR < 50 && shallowLeaf.outerR > 24,
  "sunburst a shallow tree still leaves a caption band",
);
assertEqual(shallowRoot.label, "root", "sunburst named root is labeled");
assertEqual(shallowRoot.lx, 50, "sunburst root label is at center x");
assertEqual(shallowRoot.ly, 50, "sunburst root label is at center y");
assertEqual(shallowLeaf.label, "leaf", "sunburst shallow leaf stays labeled");
const sunSliver = charts.sunburstArcs(
  {
    id: "root",
    children: [
      { id: "big", value: 99 },
      { id: "tiny", value: 1 },
    ],
  },
  100,
  100,
);
const sunTiny = sunSliver.find(function (a) {
  return a.id === "tiny";
});
assert(!!sunTiny && sunTiny.label == null, "sunburst slivers stay unlabeled");
const sunBig = charts.sunburstArcs(
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
  200,
  200,
);
const sunMid = charts.lerpScene({ arcs: sun }, { arcs: sunBig }, 0.5);
const userMid = sunMid.arcs.find(function (a) {
  return a.id === "user.slice";
});
const userBig = sunBig.find(function (a) {
  return a.id === "user.slice";
});
assert(!!userMid && !!userBig, "lerpScene keeps sunburst labels");
assert(
  Math.abs(userMid.lx - (user.lx + userBig.lx) / 2) < 1e-9,
  "lerpScene tweens sunburst label x",
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
assertEqual(radar.axes[0].cx, 50, "radar spoke origin is canvas center x");
assertEqual(radar.axes[0].cy, 50, "radar spoke origin is canvas center y");
assertEqual(radar.axes[1].cx, radar.axes[0].cx, "radar spokes share origin x");
assertEqual(radar.axes[1].cy, radar.axes[0].cy, "radar spokes share origin y");
const spokeDx = radar.axes[0].x - radar.axes[0].cx;
const spokeDy = radar.axes[0].y - radar.axes[0].cy;
assert(
  Math.sqrt(spokeDx * spokeDx + spokeDy * spokeDy) < 50 &&
    Math.sqrt(spokeDx * spokeDx + spokeDy * spokeDy) > 24,
  "radar spokes leave a caption band",
);
assert(Math.abs(radar.axes[0].x - 50) < 1e-9, "radar first spoke is north");
assertEqual(radar.axes[0].label, "rx", "radar spoke keeps its counter name");
assertEqual(radar.axes[2].label, "packets", "radar keeps a longer counter label");
assertEqual(radar.axes[0].align, "center", "radar north label is centered");
assertEqual(radar.axes[0].baseline, "bottom", "radar north label sits above the spoke");
assert(radar.axes[0].lx >= 0 && radar.axes[0].lx <= 100, "radar label x stays on canvas");
assert(radar.axes[0].ly >= 0 && radar.axes[0].ly <= 100, "radar label y stays on canvas");
assert(radar.axes[0].ly < radar.axes[0].y, "radar north label sits outside the ring");
assertEqual(radar.labels.length, 2, "radar emits a label per NIC");
assertEqual(radar.labels[0].label, "eth0", "radar first label is eth0");
assertEqual(radar.labels[1].label, "wlan0", "radar keeps wlan0");
assertEqual(radar.labels[0].align, "center", "radar NIC labels are centered");
assertEqual(radar.labels[0].baseline, "middle", "radar NIC labels sit in the fill");
assert(radar.labels[0].lx >= 0 && radar.labels[0].lx <= 100, "radar NIC label x stays on canvas");
assert(radar.labels[0].ly >= 0 && radar.labels[0].ly <= 100, "radar NIC label y stays on canvas");
const nicDx = radar.labels[0].lx - radar.axes[0].cx;
const nicDy = radar.labels[0].ly - radar.axes[0].cy;
assert(Math.sqrt(nicDx * nicDx + nicDy * nicDy) < 36, "radar NIC labels sit inside the ring");
const radarQuiet = charts.radarPolygons(
  [{ id: "lo", rx: 0, tx: 0, packets: 0, drops: 0, errs: 0 }],
  ["rx", "tx", "packets", "drops", "errs"],
  100,
  100,
);
assertEqual(radarQuiet.labels.length, 0, "radar skips a collapsed NIC");
const radarBig = charts.radarPolygons(
  [
    { id: "eth0", rx: 100, tx: 50, packets: 10, drops: 0, errs: 1 },
    { id: "wlan0", rx: 50, tx: 25, packets: 5, drops: 0, errs: 0 },
  ],
  ["rx", "tx", "packets", "drops", "errs"],
  200,
  200,
);
const radarMid = charts.lerpScene(radar, radarBig, 0.5);
assertEqual(radarMid.axes.length, 5, "lerpScene keeps radar axes");
assert(
  Math.abs(radarMid.axes[0].lx - (radar.axes[0].lx + radarBig.axes[0].lx) / 2) < 1e-9,
  "lerpScene tweens radar label x",
);
assertEqual(radarMid.labels.length, 2, "lerpScene keeps radar NIC labels");
assert(
  Math.abs(radarMid.labels[0].lx - (radar.labels[0].lx + radarBig.labels[0].lx) / 2) < 1e-9,
  "lerpScene tweens radar NIC label x",
);

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
assertEqual(fall[0].label, "package", "waterfall first label is package");
assertEqual(fall[1].label, "core", "waterfall keeps core");
assertEqual(fall[2].label, "dram", "waterfall last label is dram");
assertEqual(fall[0].baseline, "bottom", "waterfall labels sit on the bottom edge");
assertEqual(fall[0].ly, 40, "waterfall labels are on the canvas floor");
assertEqual(fall[0].align, "center", "waterfall labels are centered on the bar");
assert(fall[0].maxW <= fall[0].w + 0.01, "waterfall labels stay inside the bar slot");
assert(fall[0].y + fall[0].h <= fall[0].ly - 8, "waterfall bars sit above the label band");
const fallBig = charts.waterfallBars(
  [
    { id: "package", value: 10 },
    { id: "core", value: 4 },
    { id: "dram", value: -2 },
  ],
  180,
  80,
);
const fallLabelMid = charts.lerpScene({ bars: fall }, { bars: fallBig }, 0.5);
assertEqual(fallLabelMid.bars.length, 3, "lerpScene keeps waterfall labels");
assert(
  Math.abs(fallLabelMid.bars[2].lx - (fall[2].lx + fallBig[2].lx) / 2) < 1e-9,
  "lerpScene tweens waterfall label x",
);
const fallNamed = charts.waterfallBars(
  [{ id: "intel-rapl:0", label: "package-0", value: 10 }],
  90,
  40,
);
assertEqual(fallNamed[0].id, "intel-rapl:0", "waterfall keeps the sysfs id");
assertEqual(fallNamed[0].label, "package-0", "waterfall prefers the domain name");
const fallCrowd = charts.waterfallBars(
  Array.from({ length: 20 }, function (_, i) {
    return { id: "d" + i, value: 1 };
  }),
  90,
  40,
);
assertEqual(fallCrowd.length, 20, "waterfall keeps every RAPL step");
assertEqual(fallCrowd[0].label, "d0", "waterfall crowded bars keep their domain names");
assertEqual(fallCrowd[19].label, "d19", "waterfall keeps the last crowded domain name");
assertEqual(fallCrowd[0].ly, 40, "waterfall crowded labels stay on the canvas floor");
assert(
  fallCrowd.every(function (b) {
    return b.label != null && b.lx >= 0 && b.lx <= 90;
  }),
  "waterfall crowded labels stay on canvas",
);

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
assertEqual(slabRoot.label, "slab", "icicle root label is the cache id");
assertEqual(km.label, "kmalloc", "icicle parent rows stay labeled");
assertEqual(k8.label, "kmalloc-8", "icicle leaf label is the cache id");
assertEqual(k8.align, "center", "icicle labels are centered in the row");
assertEqual(k8.baseline, "middle", "icicle labels are vertically centered");
assert(Math.abs(k8.lx - (k8.x + k8.w / 2)) < 1e-9, "icicle label x is the row center");
assert(Math.abs(k8.ly - (k8.y + k8.h / 2)) < 1e-9, "icicle label y is the row center");
assert(k8.maxW <= k8.w + 0.01, "icicle labels stay inside the row");
const iceSliver = charts.icicleRects(
  {
    id: "root",
    children: [
      { id: "big", value: 99 },
      { id: "tiny", value: 1 },
    ],
  },
  100,
  20,
);
const iceTiny = iceSliver.find(function (r) {
  return r.id === "tiny";
});
assert(!!iceTiny && iceTiny.label == null, "icicle slivers stay unlabeled");
const iceBig = charts.icicleRects(
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
  200,
  60,
);
const iceMid = charts.lerpScene({ rects: ice }, { rects: iceBig }, 0.5);
const k8Mid = iceMid.rects.find(function (r) {
  return r.id === "kmalloc-8";
});
const k8Big = iceBig.find(function (r) {
  return r.id === "kmalloc-8";
});
assert(!!k8Mid && !!k8Big, "lerpScene keeps icicle labels");
assert(Math.abs(k8Mid.lx - (k8.lx + k8Big.lx) / 2) < 1e-9, "lerpScene tweens icicle label x");

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
const ratesQuiet = charts.irqRateMatrix(
  {
    rows: [
      { id: "NMI", values: [10, 20] },
      { id: "LOC", values: [3, 3] },
    ],
  },
  {
    rows: [
      { id: "NMI", values: [12, 25] },
      { id: "LOC", values: [3, 3] },
    ],
  },
);
assertEqual(ratesQuiet.length, 1, "irqRateMatrix drops idle IRQs");
assertEqual(ratesQuiet[0].id, "NMI", "irqRateMatrix keeps a firing IRQ");
assertEqual(
  charts.irqRateMatrix({ rows: [] }, { rows: [{ id: "NMI", values: [12, 25] }] }).length,
  0,
  "irqRateMatrix drops IRQs without a previous row",
);
const irqSnap = { rows: [{ id: "NMI", values: [12, 25] }] };
assertEqual(
  charts.irqRateMatrix(irqSnap, irqSnap).length,
  0,
  "irqRateMatrix skips a repeated snapshot",
);
assertEqual(
  charts
    .irqRateMatrix(
      { rows: [{ id: "NMI", values: ["10", "20"] }] },
      { rows: [{ id: "NMI", values: ["12", "25"] }] },
    )[0]
    .values.join(","),
  "2,5",
  "irqRateMatrix still reads numeric strings",
);
const idleCrowd = Array.from({ length: 200 }, function (_, i) {
  return { id: "irq" + i, values: [i, i] };
});
const ratesCrowd = charts.irqRateMatrix(
  { rows: idleCrowd.concat([{ id: "NMI", values: [1, 1] }]) },
  { rows: idleCrowd.concat([{ id: "NMI", values: [4, 2] }]) },
);
assertEqual(ratesCrowd.length, 1, "irqRateMatrix skips a crowd of idle IRQs");
assertEqual(ratesCrowd[0].id, "NMI", "irqRateMatrix keeps the one firing IRQ");
assertEqual(ratesCrowd[0].values.join(","), "3,1", "irqRateMatrix still deltas the hot row");

const soft = charts.softirqWedges(
  {
    rows: [
      { id: "NET_RX", values: [10, 20] },
      { id: "TIMER", values: [2, 2] },
    ],
  },
  {
    rows: [
      { id: "NET_RX", values: [12, 25] },
      { id: "TIMER", values: [2, 2] },
    ],
  },
);
assertEqual(soft.length, 2, "softirqWedges keeps a wedge per vector");
assertEqual(soft[0].id, "NET_RX", "softirqWedges keeps the first vector id");
assertEqual(soft[0].value, 7, "softirqWedges sums per-CPU deltas");
assertEqual(soft[1].value, 0, "softirqWedges idle vectors stay at 0");
assertEqual(
  charts.softirqWedges(null, { rows: [] }).length,
  0,
  "softirqWedges empty next is empty",
);
assertEqual(
  charts.softirqWedges(null, { rows: [{ id: "NET_RX", values: [12, 25] }] })[0].value,
  0,
  "softirqWedges missing prev is a zero wedge",
);
assertEqual(
  charts.softirqWedges(
    {
      rows: [
        { id: "TIMER", values: [1, 1] },
        { id: "NET_RX", values: [10, 20] },
      ],
    },
    {
      rows: [
        { id: "NET_RX", values: [12, 25] },
        { id: "TIMER", values: [2, 2] },
      ],
    },
  )[0].value,
  7,
  "softirqWedges matches shuffled vectors by id",
);
assertEqual(
  charts.softirqWedges(
    { rows: [{ id: "NET_RX", values: ["10", "20"] }] },
    { rows: [{ id: "NET_RX", values: ["12", "25"] }] },
  )[0].value,
  7,
  "softirqWedges still reads numeric strings",
);
const emptyHeat = charts.heatmapLayout(null, 100, 40);
assertEqual(emptyHeat.cells.length, 0, "heatmapLayout empty model is empty");
assertEqual(emptyHeat.labels.length, 0, "heatmapLayout empty model has no labels");
const sameScene = { cells: heat };
assertEqual(
  charts.lerpScene(sameScene, sameScene, 0.5),
  sameScene,
  "lerpScene reuses an unchanged scene",
);
assertEqual(
  charts.lerpScene({ cells: heat }, { cells: heat }, 0.5).cells,
  heat,
  "lerpScene reuses an unchanged cell list",
);
const sameSeries = [1, 2, 3];
assertEqual(
  charts.lerpSeries(sameSeries, sameSeries, 0.5),
  sameSeries,
  "lerpSeries reuses an unchanged series",
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
  { pid: 1, ppid: 0, comm: "init", rssKb: 10 },
  { pid: 100, ppid: 1, comm: "firefox", rssKb: 40 },
  { pid: 101, ppid: 100, comm: "content", rssKb: 20 },
  { pid: 2, ppid: 1, comm: "sshd", rssKb: 20 },
  { pid: 3, ppid: 1, comm: "idle", rssKb: 0 },
]);
assertEqual(rss.children.length, 1, "rssTree roots at the parentless process");
assertEqual(rss.children[0].comm, "init", "rssTree keeps init");
assertEqual(charts.findTreeNode(rss, "100").comm, "firefox", "rssTree nests firefox under init");
assertEqual(charts.findTreeNode(rss, "101").comm, "content", "rssTree nests content under firefox");
assertEqual(charts.rssTree([]), null, "rssTree empty is null");
const ffMap = charts.treemapRects(rss, 100, 50, "100");
const ffMapIds = ffMap.map(function (r) {
  return r.id;
});
assert(ffMapIds.indexOf("101") !== -1, "selected process map keeps children");
assert(ffMapIds.indexOf("100:self") !== -1, "selected process map keeps that process's own RSS");
assert(ffMapIds.indexOf("2") === -1, "selected process map drops siblings");
assertEqual(charts.treemapFocusId("100:self"), "100", "treemapFocusId strips the self slice");
assertEqual(charts.clampViewScale(0.2), 1, "clampViewScale floors at 1");
assertEqual(charts.clampViewScale(80), 24, "clampViewScale caps at 24");
const zoomedView = charts.zoomView(1, 0, 0, 50, 25, 2, 100, 50);
assertEqual(zoomedView.scale, 2, "zoomView doubles scale");
const held = charts.worldPoint(50, 25, zoomedView.scale, zoomedView.x, zoomedView.y);
assert(Math.abs(held.x - 50) < 1e-9, "zoomView keeps the cursor world x");
assert(Math.abs(held.y - 25) < 1e-9, "zoomView keeps the cursor world y");
const mapped = charts.mapViewScene({ rects: [{ x: 10, y: 4, w: 8, h: 2 }] }, 2, -10, 0);
assertEqual(mapped.rects[0].x, 10, "mapViewScene scales and pans x");
assertEqual(mapped.rects[0].w, 16, "mapViewScene scales width");

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
const raplNamed = charts.raplSteps(
  { rapl: [{ id: "intel-rapl:0", name: "package-0", uj: 1e6 }] },
  { rapl: [{ id: "intel-rapl:0", name: "package-0", uj: 3e6 }] },
);
assertEqual(raplNamed[0].id, "intel-rapl:0", "raplSteps keeps the sysfs id");
assertEqual(raplNamed[0].label, "package-0", "raplSteps labels the domain name");
assertEqual(charts.slabTree([]), null, "slabTree empty is null");
assert(!!charts.slabTree([{ name: "kmalloc-8", active: 10 }]), "slabTree keeps a cache");

assertEqual(charts.lerpNum(0, 10, 0.5), 5, "lerpNum is linear");
assertEqual(charts.lerpNum(null, 8, 1), 8, "lerpNum missing from uses to");
const heatA = charts.heatmapCells([{ id: "NMI", values: [0, 10] }], 100, 20);
const heatB = charts.heatmapCells([{ id: "NMI", values: [10, 0] }], 100, 20);
const heatMid = charts.lerpScene({ cells: heatA }, { cells: heatB }, 0.5);
assertEqual(heatMid.cells.length, 2, "lerpScene keeps heatmap cells");
assertEqual(heatMid.cells[0].fill, 0.5, "lerpScene tweens heatmap fill");
assertEqual(heatMid.cells[0].x, heatA[0].x, "lerpScene keeps heatmap x");
const heatSwapA = [
  { x: 0, y: 0, w: 10, h: 10, fill: 0, value: 0, row: "NMI", col: 0 },
  { x: 10, y: 0, w: 10, h: 10, fill: 1, value: 10, row: "NMI", col: 1 },
];
const heatSwapB = [
  { x: 10, y: 0, w: 10, h: 10, fill: 0, value: 0, row: "NMI", col: 1 },
  { x: 0, y: 0, w: 10, h: 10, fill: 1, value: 10, row: "NMI", col: 0 },
];
const heatSwapMid = charts.lerpScene({ cells: heatSwapA }, { cells: heatSwapB }, 0.5);
assertEqual(heatSwapMid.cells[0].col, 1, "lerpScene matches heatmap cells by col when shuffled");
assertEqual(heatSwapMid.cells[0].fill, 0.5, "lerpScene tweens a shuffled cell's fill");
assertEqual(heatSwapMid.cells[1].col, 0, "lerpScene keeps the other shuffled cell");
assertEqual(heatSwapMid.cells[1].fill, 0.5, "lerpScene tweens fill after a shuffle");
const fallA = charts.waterfallBars([{ id: "package", value: 0 }], 90, 40);
const fallB = charts.waterfallBars([{ id: "package", value: 10 }], 90, 40);
const fallMid = charts.lerpScene({ bars: fallA }, { bars: fallB }, 0.5);
assert(
  Math.abs(fallMid.bars[0].h - (fallA[0].h + fallB[0].h) / 2) < 0.6,
  "lerpScene tweens waterfall bar height",
);
const roseA = charts.nightingaleWedges(
  [
    { id: "NET_RX", value: 1 },
    { id: "TIMER", value: 4 },
  ],
  100,
  100,
);
const roseB = charts.nightingaleWedges(
  [
    { id: "NET_RX", value: 4 },
    { id: "TIMER", value: 1 },
  ],
  100,
  100,
);
const roseMid = charts.lerpScene({ wedges: roseA }, { wedges: roseB }, 0.5);
assert(
  roseMid.wedges[0].r > roseA[0].r && roseMid.wedges[0].r < roseB[0].r,
  "lerpScene tweens rose radius",
);
assertEqual(
  charts.lerpScene({ cells: heatA }, { cells: heatB }, 0).cells[0].fill,
  heatA[0].fill,
  "lerpScene t=0 is from",
);
assertEqual(
  charts.lerpScene({ cells: heatA }, { cells: heatB }, 1).cells[0].fill,
  heatB[0].fill,
  "lerpScene t=1 is to",
);
assertEqual(charts.tweenProgress(1000, 1000, 200), 0, "tweenProgress at start is 0");
assertEqual(charts.tweenProgress(1000, 1100, 200), 0.5, "tweenProgress is elapsed over duration");
assertEqual(charts.tweenProgress(1000, 1300, 200), 1, "tweenProgress clamps past the end");
assertEqual(charts.tweenProgress(1000, 1100, 0), 1, "tweenProgress without duration is done");
const midSeries = charts.lerpSeries([0, 10], [10, 20], 0.5);
assertEqual(midSeries[0], 5, "lerpSeries tweens the first sample");
assertEqual(midSeries[1], 15, "lerpSeries tweens the last sample");
assertEqual(charts.lerpSeries([1], [1, 3], 1)[1], 3, "lerpSeries t=1 is the destination");
assertEqual(charts.lerpSeries([2, 4], [10, 20], 0)[0], 2, "lerpSeries t=0 is the source");
const midBins = charts.lerpKeyed(
  [{ id: "hot", count: 0 }],
  [{ id: "hot", count: 10, label: "hot" }],
  0.5,
  ["count"],
);
assertEqual(midBins[0].count, 5, "lerpKeyed tweens a numeric field");
assertEqual(midBins[0].label, "hot", "lerpKeyed keeps non-numeric fields");
assert(
  charts.inViewport(0, 0, 100, 40, 0, 0, 200, 80, 0) === true,
  "inViewport is true for a chart in the pane",
);
assert(
  charts.inViewport(0, 200, 100, 40, 0, 0, 200, 80, 0) === false,
  "inViewport is false for a chart below the pane",
);
assert(
  charts.inViewport(0, 200, 100, 40, 0, 0, 200, 80, 160) === true,
  "inViewport pad keeps a near-offscreen chart live",
);
assert(
  charts.inViewport(0, -50, 100, 40, 0, 0, 200, 80, 0) === false,
  "inViewport is false for a chart above the pane",
);
