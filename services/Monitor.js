// Presentation helpers for the Monitor hub. Geometry stays in JS so Node
// can test bar layouts without Quickshell.

function finiteNumber(v) {
  if (v === null || v === undefined || v === "") return null;
  var n = Number(v);
  return isFinite(n) ? n : null;
}

function intervalChips() {
  return [
    { id: "500", label: "0.5s", ms: 500 },
    { id: "1000", label: "1s", ms: 1000 },
    { id: "2000", label: "2s", ms: 2000 },
    { id: "5000", label: "5s", ms: 5000 },
  ];
}

function intervalMs(id) {
  var key = String(id || "2000");
  var list = intervalChips();
  var i;
  for (i = 0; i < list.length; i++) {
    if (list[i].id === key) return list[i].ms;
  }
  return 2000;
}

function pageCards() {
  return [
    {
      id: "processes",
      title: "Processes",
      description: "Search, sort, and signal this machine's tasks.",
    },
    {
      id: "cpu",
      title: "CPU",
      description: "Per-core load, frequency, and pressure.",
    },
    {
      id: "memory",
      title: "Memory",
      description: "Used, cache, buffers, swap, and stall.",
    },
    {
      id: "disk",
      title: "Disk I/O",
      description: "Read and write rates from diskstats.",
    },
    {
      id: "traffic",
      title: "Traffic",
      description: "Per-interface bandwidth and TCP counts.",
    },
    {
      id: "sensors",
      title: "Sensors",
      description: "Temperatures, fans, and GPU load.",
    },
  ];
}

function alertLevel(pct) {
  var n = finiteNumber(pct);
  if (n === null) return "unknown";
  if (n >= 90) return "hot";
  if (n >= 75) return "warm";
  return "ok";
}

function barRects(values, width, height, gap) {
  var src = Array.isArray(values) ? values : [];
  var n = src.length;
  var w = Number(width);
  var h = Number(height);
  if (!(n > 0) || !(w > 0) || !(h > 0)) return [];
  var g = Number(gap);
  if (!isFinite(g) || g < 0) g = 1;
  var totalGap = g * Math.max(0, n - 1);
  var bw = (w - totalGap) / n;
  if (!(bw > 0)) return [];
  var out = [];
  var i;
  var v;
  var bh;
  for (i = 0; i < n; i++) {
    v = finiteNumber(src[i]);
    if (v === null || v < 0) v = 0;
    if (v > 100) v = 100;
    bh = (v / 100) * h;
    out.push({
      x: i * (bw + g),
      y: h - bh,
      w: bw,
      h: bh,
      value: v,
      alert: alertLevel(v),
    });
  }
  return out;
}

function stackedRects(parts, width, height) {
  var list = Array.isArray(parts) ? parts : [];
  var w = Number(width);
  var h = Number(height);
  if (!(w > 0) || !(h > 0) || list.length === 0) return [];
  var total = 0;
  var i;
  var kb;
  for (i = 0; i < list.length; i++) {
    kb = finiteNumber(list[i] && list[i].kb);
    if (kb !== null && kb > 0) total += kb;
  }
  if (!(total > 0)) return [];
  var x = 0;
  var out = [];
  var rw;
  for (i = 0; i < list.length; i++) {
    kb = finiteNumber(list[i] && list[i].kb);
    if (kb === null || kb < 0) kb = 0;
    rw = (kb / total) * w;
    out.push({
      x: x,
      y: 0,
      w: rw,
      h: h,
      id: list[i] && list[i].id ? list[i].id : String(i),
      label: list[i] && list[i].label ? list[i].label : "",
      kb: kb,
    });
    x += rw;
  }
  return out;
}

function cpuHistogram(rows) {
  var bins = [
    { id: "idle", label: "idle", min: 0, max: 0.05, count: 0 },
    { id: "lt1", label: "<1%", min: 0.05, max: 1, count: 0 },
    { id: "lt5", label: "1–5%", min: 1, max: 5, count: 0 },
    { id: "lt15", label: "5–15%", min: 5, max: 15, count: 0 },
    { id: "lt40", label: "15–40%", min: 15, max: 40, count: 0 },
    { id: "hot", label: "40%+", min: 40, max: 100000, count: 0 },
  ];
  var src = Array.isArray(rows) ? rows : [];
  var i;
  var j;
  var cpu;
  for (i = 0; i < src.length; i++) {
    cpu = finiteNumber(src[i] && src[i].cpu);
    if (cpu === null) cpu = 0;
    for (j = 0; j < bins.length; j++) {
      if (cpu >= bins[j].min && cpu < bins[j].max) {
        bins[j].count += 1;
        break;
      }
    }
  }
  return bins;
}

function histogramRects(bins, width, height) {
  var list = Array.isArray(bins) ? bins : [];
  var counts = [];
  var i;
  var max = 0;
  for (i = 0; i < list.length; i++) {
    counts.push(list[i] && list[i].count ? list[i].count : 0);
    if (counts[i] > max) max = counts[i];
  }
  if (!(max > 0)) max = 1;
  var scaled = [];
  for (i = 0; i < counts.length; i++) scaled.push((counts[i] / max) * 100);
  var rects = barRects(scaled, width, height, 2);
  for (i = 0; i < rects.length; i++) {
    rects[i].id = list[i] && list[i].id ? list[i].id : String(i);
    rects[i].label = list[i] && list[i].label ? list[i].label : "";
    rects[i].count = counts[i];
  }
  return rects;
}

function tcpParts(tcp) {
  var src = tcp && typeof tcp === "object" ? tcp : {};
  return [
    { id: "established", label: "Established", kb: src.established || 0 },
    { id: "listen", label: "Listen", kb: src.listen || 0 },
    { id: "timeWait", label: "Time-wait", kb: src.timeWait || 0 },
    { id: "closeWait", label: "Close-wait", kb: src.closeWait || 0 },
  ];
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    intervalChips: intervalChips,
    intervalMs: intervalMs,
    pageCards: pageCards,
    alertLevel: alertLevel,
    barRects: barRects,
    stackedRects: stackedRects,
    cpuHistogram: cpuHistogram,
    histogramRects: histogramRects,
    tcpParts: tcpParts,
  };
}
