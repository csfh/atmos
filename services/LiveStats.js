// Live CPU, memory, network, thermal, and process samples for Home.
// Counters come from live-stats.py. Percents, rates, and temps stay unknown
// when a sample is missing instead of painting as 0 or 0°.

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

// hw-inventory skips non-positive millidegree readings. A 0 °C from sysfs
// is almost always an unbound sensor, not a real package at freezing.
function parseTemp(v) {
  var n = finiteNumber(v);
  if (n === null || n <= 0) return null;
  return n;
}

function parseGpus(raw) {
  var src = Array.isArray(raw) ? raw : [];
  var out = [];
  var i;
  var g;
  for (i = 0; i < src.length; i++) {
    g = src[i];
    if (!g || typeof g !== "object") continue;
    out.push({
      card: String(g.card || ""),
      pciId: String(g.pciId || "").toLowerCase(),
      name: String(g.name || ""),
      vendor: String(g.vendor || ""),
      driver: String(g.driver || ""),
      integrated: g.integrated === true,
      temp: parseTemp(g.temp),
    });
  }
  return out;
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
    cpuTemp: parseTemp(data.cpuTemp),
    gpus: parseGpus(data.gpus),
    processes: processes,
  };
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

function gpuMatches(live, key) {
  if (!live || key == null) return false;
  if (typeof key === "string") {
    var needle = String(key);
    return live.card === needle || live.pciId === needle.toLowerCase();
  }
  var pci = String(key.pciId || "").toLowerCase();
  if (pci && live.pciId && pci === live.pciId) return true;
  if (key.card && live.card && key.card === live.card) return true;
  if (key.name && live.card && key.name === live.card) return true;
  return false;
}

function findGpu(sample, key) {
  var gs = sample && Array.isArray(sample.gpus) ? sample.gpus : [];
  var i;
  for (i = 0; i < gs.length; i++) {
    if (gpuMatches(gs[i], key)) return gs[i];
  }
  return null;
}

function isIntegratedGpu(gpu) {
  if (!gpu) return false;
  if (gpu.integrated === true) return true;
  return String(gpu.driver || "") === "i915";
}

function gpuOwnTemp(sample, key) {
  var g = findGpu(sample, key);
  return g ? parseTemp(g.temp) : null;
}

// Own sensor wins. An integrated part without one shares the die with the
// CPU, so the package temperature is the honest proxy. Discrete parts
// without a sensor stay unknown instead of borrowing the CPU.
function gpuTempAt(sample, key) {
  var g = findGpu(sample, key);
  var own = g ? parseTemp(g.temp) : null;
  if (own !== null) return own;
  if (g && g.integrated === true) return parseTemp(sample && sample.cpuTemp);
  if (!g && isIntegratedGpu(key)) return parseTemp(sample && sample.cpuTemp);
  return null;
}

function gpuSeries(history, key) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var t;
  for (i = 0; i < list.length; i++) {
    t = gpuTempAt(list[i], key);
    if (t !== null) out.push(t);
  }
  return out;
}

// Hardware inventory already named the GPUs. Use that list when present so
// Home does not re-probe PCI. Fall back to the live DRM cards before the
// snapshot arrives.
function gpuRows(hardwareGpus, sample) {
  var hw = Array.isArray(hardwareGpus) ? hardwareGpus : [];
  if (hw.length) return hw;
  return sample && Array.isArray(sample.gpus) ? sample.gpus : [];
}

function formatPercent(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  if (v < 0) v = 0;
  if (v < 10) return (Math.round(v * 10) / 10).toFixed(1) + "%";
  return Math.round(v) + "%";
}

function formatTemp(n) {
  var v = parseTemp(n);
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
    parseTemp: parseTemp,
    gpuRows: gpuRows,
    gpuOwnTemp: gpuOwnTemp,
    gpuTempAt: gpuTempAt,
    gpuSeries: gpuSeries,
    formatPercent: formatPercent,
    formatTemp: formatTemp,
    formatBps: formatBps,
    formatNet: formatNet,
    memBytes: memBytes,
    sparklinePoints: sparklinePoints,
  };
}
