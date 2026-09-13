// Live CPU, memory, network, and process samples for Home.
// Counters come from live-stats.py. Percents and rates are deltas in this
// file so a missing sample stays unknown instead of painting as 0%.

var SAMPLE_CAP = 60;
var PROCESS_CAP = 80;

function finiteNumber(v) {
  if (v === null || v === undefined || v === "") return null;
  var n = Number(v);
  return isFinite(n) ? n : null;
}

function nonNeg(v) {
  var n = finiteNumber(v);
  if (n === null || n < 0) return null;
  return n;
}

function parseProcess(row) {
  if (!row || typeof row !== "object") return null;
  var pid = Number(row.pid);
  if (!isFinite(pid) || pid <= 0) return null;
  var comm = String(row.comm || "").replace(/^\s+|\s+$/g, "");
  if (!comm) return null;
  var rssKb = nonNeg(row.rssKb);
  var ticks = nonNeg(row.ticks);
  return {
    pid: pid,
    comm: comm,
    cmdline: String(row.cmdline || ""),
    uid: finiteNumber(row.uid),
    rssKb: rssKb,
    ticks: ticks,
    cpu: nonNeg(row.cpu),
  };
}

function parse(raw) {
  var data = raw;
  if (typeof raw === "string") {
    var text = raw.replace(/^\s+|\s+$/g, "");
    if (!text) return null;
    try {
      data = JSON.parse(text);
    } catch (e) {
      return null;
    }
  }
  if (!data || typeof data !== "object") return null;
  var procs = Array.isArray(data.processes) ? data.processes : [];
  var processes = [];
  var i;
  var row;
  for (i = 0; i < procs.length; i++) {
    row = parseProcess(procs[i]);
    if (row) processes.push(row);
  }
  return {
    cpuIdle: nonNeg(data.cpuIdle),
    cpuTotal: nonNeg(data.cpuTotal),
    memUsed: nonNeg(data.memUsed),
    memTotal: nonNeg(data.memTotal),
    memAvail: nonNeg(data.memAvail),
    netRx: nonNeg(data.netRx),
    netTx: nonNeg(data.netTx),
    clkTck: nonNeg(data.clkTck) || 100,
    cpuTemp: nonNeg(data.cpuTemp),
    gpus: parseGpus(data.gpus),
    processes: processes,
  };
}

function parseGpus(raw) {
  var src = Array.isArray(raw) ? raw : [];
  var out = [];
  var i;
  var g;
  var temp;
  for (i = 0; i < src.length; i++) {
    g = src[i];
    if (!g || typeof g !== "object") continue;
    temp = finiteNumber(g.temp);
    out.push({
      card: String(g.card || ""),
      name: String(g.name || ""),
      vendor: String(g.vendor || ""),
      driver: String(g.driver || ""),
      integrated: g.integrated === true,
      temp: temp,
    });
  }
  return out;
}

function cpuPercent(prev, next) {
  if (!prev || !next) return null;
  if (prev.cpuIdle === null || prev.cpuTotal === null) return null;
  if (next.cpuIdle === null || next.cpuTotal === null) return null;
  var dIdle = next.cpuIdle - prev.cpuIdle;
  var dTotal = next.cpuTotal - prev.cpuTotal;
  if (!(dTotal > 0) || dIdle < 0) return null;
  var used = dTotal - dIdle;
  if (used < 0) used = 0;
  return (used / dTotal) * 100;
}

function memPercent(sample) {
  if (!sample || sample.memUsed === null || sample.memTotal === null) return null;
  if (!(sample.memTotal > 0)) return null;
  var pct = (sample.memUsed / sample.memTotal) * 100;
  if (pct < 0) return 0;
  return pct;
}

function netRate(prev, next, dtMs) {
  var empty = { rxBps: null, txBps: null };
  if (!prev || !next || !(dtMs > 0)) return empty;
  if (prev.netRx === null || next.netRx === null) return empty;
  if (prev.netTx === null || next.netTx === null) return empty;
  var dRx = next.netRx - prev.netRx;
  var dTx = next.netTx - prev.netTx;
  if (dRx < 0 || dTx < 0) return empty;
  var sec = dtMs / 1000;
  return { rxBps: dRx / sec, txBps: dTx / sec };
}

function processCpu(prevTicks, nextTicks, dtMs, clkTck) {
  if (prevTicks === null || nextTicks === null) return null;
  if (!(dtMs > 0) || !(clkTck > 0)) return null;
  var d = nextTicks - prevTicks;
  if (d < 0) return null;
  return (100 * d) / (clkTck * (dtMs / 1000));
}

function decorateProcesses(prev, sample, dtMs) {
  var list = sample && Array.isArray(sample.processes) ? sample.processes : [];
  var prevList = prev && Array.isArray(prev.processes) ? prev.processes : [];
  var prevMap = {};
  var i;
  for (i = 0; i < prevList.length; i++) {
    if (prevList[i] && prevList[i].pid) prevMap[prevList[i].pid] = prevList[i].ticks;
  }
  var clk = sample && sample.clkTck > 0 ? sample.clkTck : 100;
  var out = [];
  var row;
  var cpu;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row) continue;
    cpu = processCpu(
      Object.prototype.hasOwnProperty.call(prevMap, row.pid) ? prevMap[row.pid] : null,
      row.ticks,
      dtMs,
      clk,
    );
    out.push({
      pid: row.pid,
      comm: row.comm,
      cmdline: row.cmdline,
      uid: row.uid,
      rssKb: row.rssKb,
      ticks: row.ticks,
      cpu: cpu,
    });
  }
  return out;
}

function pushSample(history, sample, now) {
  var parsed = sample && sample.cpuIdle !== undefined ? sample : parse(sample);
  if (!parsed) return Array.isArray(history) ? history.slice() : [];
  var list = Array.isArray(history) ? history.slice() : [];
  var prev = list.length ? list[list.length - 1] : null;
  var at = Number(now);
  if (!isFinite(at)) at = 0;
  var dt = prev ? at - prev.at : 0;
  var rates = netRate(prev, parsed, dt);
  var row = {
    at: at,
    cpuIdle: parsed.cpuIdle,
    cpuTotal: parsed.cpuTotal,
    memUsed: parsed.memUsed,
    memTotal: parsed.memTotal,
    memAvail: parsed.memAvail,
    netRx: parsed.netRx,
    netTx: parsed.netTx,
    clkTck: parsed.clkTck,
    cpuTemp: parsed.cpuTemp,
    gpus: parsed.gpus,
    cpu: cpuPercent(prev, parsed),
    mem: memPercent(parsed),
    rxBps: rates.rxBps,
    txBps: rates.txBps,
    processes: decorateProcesses(prev, parsed, dt),
  };
  list.push(row);
  while (list.length > SAMPLE_CAP) list.shift();
  return list;
}

function latest(history) {
  var list = Array.isArray(history) ? history : [];
  return list.length ? list[list.length - 1] : null;
}

function series(history, key) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var n;
  for (i = 0; i < list.length; i++) {
    n = list[i] ? finiteNumber(list[i][key]) : null;
    if (n !== null) out.push(n);
  }
  return out;
}

// (timestamp, value) pairs for samples where the key is known. Samples are
// ~2 s apart, so pairs stay parallel to series() for the same key.
function timedSeries(history, key) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var n;
  var at;
  for (i = 0; i < list.length; i++) {
    n = list[i] ? finiteNumber(list[i][key]) : null;
    at = list[i] ? Number(list[i].at) : NaN;
    if (n !== null && isFinite(at)) out.push({ at: at, value: n });
  }
  return out;
}

function zipSeries(values, times) {
  var vs = Array.isArray(values) ? values : [];
  var ts = Array.isArray(times) ? times : [];
  var out = [];
  var i;
  var n;
  var at;
  for (i = 0; i < vs.length && i < ts.length; i++) {
    n = finiteNumber(vs[i]);
    at = Number(ts[i]);
    if (n !== null && isFinite(at)) out.push({ at: at, value: n });
  }
  return out;
}

function windowSpanMs(entries) {
  var src = Array.isArray(entries) ? entries : [];
  if (src.length < 2) return 0;
  var span = Number(src[src.length - 1].at) - Number(src[0].at);
  return span > 0 ? span : 0;
}

function formatWindow(spanMs) {
  if (!(spanMs > 0)) return "";
  var s = Math.round(spanMs / 1000);
  if (s < 60) return "Last " + s + " s";
  var m = Math.floor(s / 60);
  if (m < 60) return "Last " + m + " min";
  return "Last " + Math.floor(m / 60) + " h";
}

function pad2(n) {
  var v = Math.floor(Math.abs(Number(n)));
  if (!isFinite(v)) return "00";
  return (v < 10 ? "0" : "") + v;
}

// Local timezone via the Date getters. Short clock for short windows so the
// axis stays readable; date appears once the window exceeds an hour.
function formatClock(at, spanMs) {
  var d = new Date(Number(at));
  if (!isFinite(d.getTime())) return "";
  var hh = pad2(d.getHours());
  var mm = pad2(d.getMinutes());
  if (spanMs <= 120000) return hh + ":" + mm + ":" + pad2(d.getSeconds());
  if (spanMs <= 3600000) return hh + ":" + mm;
  return d.getMonth() + 1 + "/" + d.getDate() + " " + hh + ":" + mm;
}

// Up to maxLabels real sample times, evenly spread, always including the
// oldest and newest. frac positions each label over its sample: the poll is
// periodic, so index fractions track the line points.
function axisTicks(entries, maxLabels) {
  var src = Array.isArray(entries) ? entries : [];
  var n = src.length;
  if (n < 2) return [];
  var k = Math.max(2, Math.min(Number(maxLabels) || 4, n));
  if (n >= 3) k = Math.max(3, k);
  if (n <= 8) k = Math.min(3, n);
  var span = windowSpanMs(src);
  var out = [];
  var j;
  var idx;
  for (j = 0; j < k; j++) {
    idx = Math.round((j * (n - 1)) / (k - 1));
    out.push({
      index: idx,
      at: src[idx].at,
      frac: idx / (n - 1),
      label: formatClock(src[idx].at, span),
    });
  }
  return out;
}

// The window peak, only when it stands clearly above the median. Returns
// {index, at, value} or null. Needs enough samples and a peak at least
// double the median with a 5-unit margin, so flat lines and tiny blips
// never mark.
function spikeOf(entries) {
  var src = Array.isArray(entries) ? entries : [];
  if (src.length < 8) return null;
  var vals = [];
  var i;
  for (i = 0; i < src.length; i++) vals.push(src[i].value);
  var sorted = vals.slice().sort(function (a, b) {
    return a - b;
  });
  var med = sorted[Math.floor(sorted.length / 2)];
  var peak = sorted[sorted.length - 1];
  if (!(peak > med * 2)) return null;
  if (!(peak - med > 5)) return null;
  for (i = 0; i < src.length; i++) {
    if (src[i].value === peak) return { index: i, at: src[i].at, value: peak };
  }
  return null;
}

function gpuSeries(history, card) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var t;
  for (i = 0; i < list.length; i++) {
    t = gpuTempAt(list[i], card);
    if (t !== null) out.push(t);
  }
  return out;
}

// A GPU's own sensor wins. An integrated part without one shares the die
// with the CPU, so the package temperature (which already includes the
// graphics tile) is the honest proxy. Discrete parts without a sensor
// stay unknown instead of borrowing the CPU.
function gpuTempAt(sample, card) {
  var gs = sample && Array.isArray(sample.gpus) ? sample.gpus : [];
  var i;
  var g;
  var t;
  for (i = 0; i < gs.length; i++) {
    g = gs[i];
    if (!g || g.card !== card) continue;
    t = finiteNumber(g.temp);
    if (t !== null) return t;
    if (g.integrated === true) return finiteNumber(sample.cpuTemp);
    return null;
  }
  return null;
}

function formatPercent(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  if (v < 0) v = 0;
  if (v < 10) return (Math.round(v * 10) / 10).toFixed(1) + "%";
  return Math.round(v) + "%";
}

function formatTemp(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  return Math.round(v) + " °C";
}

function formatBps(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  if (v < 0) v = 0;
  var units = ["B/s", "KB/s", "MB/s", "GB/s"];
  var i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v = v / 1024;
    i++;
  }
  if (i === 0) return Math.round(v) + " " + units[i];
  return (Math.round(v * 10) / 10).toFixed(1) + " " + units[i];
}

function formatNet(sample) {
  if (!sample) return "";
  var rx = formatBps(sample.rxBps);
  var tx = formatBps(sample.txBps);
  if (!rx || !tx) return "";
  return "↓ " + rx + "  ↑ " + tx;
}

function memBytes(kb) {
  var n = nonNeg(kb);
  if (n === null) return null;
  return n * 1024;
}

function sparklinePoints(values, width, height, pad) {
  var src = Array.isArray(values) ? values : [];
  var nums = [];
  var i;
  var n;
  for (i = 0; i < src.length; i++) {
    n = finiteNumber(src[i]);
    if (n !== null) nums.push(n);
  }
  var w = Number(width);
  var h = Number(height);
  var p = Number(pad);
  if (!(w > 0) || !(h > 0)) return [];
  if (!isFinite(p) || p < 0) p = 0;
  if (nums.length < 2) return [];
  var min = nums[0];
  var max = nums[0];
  for (i = 1; i < nums.length; i++) {
    if (nums[i] < min) min = nums[i];
    if (nums[i] > max) max = nums[i];
  }
  var span = max - min;
  var innerW = w - p * 2;
  var innerH = h - p * 2;
  if (!(innerW > 0) || !(innerH > 0)) return [];
  var out = [];
  var x;
  var y;
  for (i = 0; i < nums.length; i++) {
    x = p + (innerW * i) / (nums.length - 1);
    y = span === 0 ? p + innerH / 2 : p + innerH - ((nums[i] - min) / span) * innerH;
    out.push([x, y]);
  }
  return out;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    SAMPLE_CAP: SAMPLE_CAP,
    PROCESS_CAP: PROCESS_CAP,
    parse: parse,
    cpuPercent: cpuPercent,
    memPercent: memPercent,
    netRate: netRate,
    processCpu: processCpu,
    decorateProcesses: decorateProcesses,
    pushSample: pushSample,
    latest: latest,
    series: series,
    timedSeries: timedSeries,
    zipSeries: zipSeries,
    windowSpanMs: windowSpanMs,
    formatWindow: formatWindow,
    formatClock: formatClock,
    axisTicks: axisTicks,
    spikeOf: spikeOf,
    gpuSeries: gpuSeries,
    gpuTempAt: gpuTempAt,
    formatPercent: formatPercent,
    formatTemp: formatTemp,
    formatBps: formatBps,
    formatNet: formatNet,
    memBytes: memBytes,
    sparklinePoints: sparklinePoints,
  };
}
