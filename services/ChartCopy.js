// Human copy for live charts. QML imports this; Node tests eval it.

var EXPLAINERS = {
  heatmap:
    "Each row is an interrupt the kernel handled. Each column is a CPU. Darker cells fired more often since the last sample.",
  ridgeline:
    "CPU, memory, and disk stall on one time axis. A taller ridge means tasks spent more of that window waiting.",
  violin: "One shape per core. The wide part is the clock that core used most in this window.",
  rose: "Each wedge is a kind of software interrupt. A larger wedge did more of that work.",
  horizon: "Free RAM stacked by page size. Darker fill is a larger free pool.",
  treemap:
    "Each box is a process. Size is RAM it holds, including children. Click a box to map that process and its children. Drag to pan. Roll the wheel to zoom.",
  sankey:
    "Total memory splits into anonymous pages, file cache, slab, and other kernel uses. Thicker links carry more kilobytes.",
  sunburst: "Rings are cgroup depth. Slice angle is how much memory that group is using.",
  icicle: "Kernel object caches as stacked bars. A longer bar has more live objects.",
  streamgraph: "Bandwidth per disk, stacked around the middle. A fatter band is a busier disk.",
  beeswarm:
    "One mark per socket, grouped by TCP state. A dense cluster is where most connections sit.",
  radar:
    "One outline per network interface. Axes are receive, transmit, packets, drops, and errors.",
  calendar: "Hottest package temperature in this window, laid out by weekday. Darker is hotter.",
  waterfall: "Energy used since the last sample, in steps through package, cores, and DRAM.",
  parallel:
    "One line per process across CPU, RAM, open files, threads, and nice. A line high on an axis is heavy there.",
  histogram: "How many processes fall in each CPU-use band in the current sample.",
  corebars: "One bar per logical CPU. Height is how busy that core was since the last sample.",
  stackedbar: "One bar split into parts of a total, such as used and cached memory.",
  sparkline: "A line of the last samples. Left is older. Right is now.",
};

var LEGENDS = {
  heatmap:
    "Names on the left are interrupt sources. Numbers along the top are CPU ids. Fill runs from cool (quiet) to warm (the hottest cell).",
  ridgeline:
    "Labels on the left are cpu, memory, and io stall. Each series has its own color from the theme.",
  violin: "Numbers under the shapes are core ids. Each core uses a different theme color.",
  rose: "Names around the ring are softirq kinds. Each wedge takes a theme color.",
  horizon: "Numbers on the left are fold ceilings. Darker bands sit above lighter ones.",
  treemap:
    "Text in a box is the process name. Color follows that process from the theme palette. The top label is the process you selected. A leftover box is that process's own RSS.",
  sankey: "Names beside the bars are memory buckets. A link uses the color of the bar it leaves.",
  sunburst:
    "Text on a slice is the cgroup name. Color follows the group. Outer rings are children.",
  icicle: "Text on a bar is the slab cache. Color follows the cache name.",
  streamgraph: "Text on a band is the disk id. Each disk has its own theme color.",
  beeswarm: "Labels on the floor are TCP states. Marks in a column share that state's color.",
  radar:
    "Words around the ring are counters. Each outline is a NIC in its own color, with the name in the fill.",
  calendar:
    "Weekdays sit on the left. Cell color is that day's hottest reading on the thermal ramp.",
  waterfall:
    "Names under the bars are RAPL domains. A rising bar uses a theme color. A drop uses the urgent color.",
  parallel: "Words on the top rails are the axes. Each line is a process in its own theme color.",
  histogram:
    "Each bar is a CPU-use band. The hot band uses the urgent color. The others use accent.",
  corebars:
    "Bars run from core 0 at the left. A bar in the urgent color is at or above 90 percent busy.",
  stackedbar:
    "The caption under the bar names each segment. Accent is used. The rest are lighter fills of the same ink.",
  sparkline:
    "The accent stroke is the main series. A second stroke, when present, uses the next theme color.",
};

function explainer(kind) {
  var k = String(kind || "");
  return EXPLAINERS[k] ? EXPLAINERS[k] : "";
}

function legend(kind) {
  var k = String(kind || "");
  return LEGENDS[k] ? LEGENDS[k] : "";
}

function blurb(kind) {
  var a = explainer(kind);
  var b = legend(kind);
  if (a && b) return a + " " + b;
  return a || b;
}

function kinds() {
  var out = [];
  var key;
  for (key in EXPLAINERS) {
    if (Object.prototype.hasOwnProperty.call(EXPLAINERS, key)) out.push(key);
  }
  out.sort();
  return out;
}
