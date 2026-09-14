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
      busy: nonNeg(g.busy),
      vramUsed: nonNeg(g.vramUsed),
      vramTotal: nonNeg(g.vramTotal),
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
  var state = String(row.state || "")
    .replace(/^\s+|\s+$/g, "")
    .charAt(0);
  var ppid = Number(row.ppid);
  if (!isFinite(ppid) || ppid < 0) ppid = 0;
  return {
    pid: pid,
    ppid: ppid,
    comm: comm,
    cmdline: String(row.cmdline || ""),
    uid: finiteNumber(row.uid),
    rssKb: rssKb,
    ticks: ticks,
    cpu: nonNeg(row.cpu),
    state: state,
    threads: nonNeg(row.threads),
    nice: finiteNumber(row.nice),
    kthread: row.kthread === true,
    mine: row.mine === true ? true : row.mine === false ? false : null,
    readBytes: nonNeg(row.readBytes),
    writeBytes: nonNeg(row.writeBytes),
    readBps: nonNeg(row.readBps),
    writeBps: nonNeg(row.writeBps),
    depth: nonNeg(row.depth) || 0,
  };
}

function parseCpuCore(row, index) {
  if (!row || typeof row !== "object") return null;
  var id = finiteNumber(row.id);
  if (id === null) id = index;
  return {
    id: id,
    idle: nonNeg(row.idle),
    total: nonNeg(row.total),
    freqMhz: nonNeg(row.freqMhz),
    governor: String(row.governor || ""),
    cpu: nonNeg(row.cpu),
  };
}

function parseIface(row) {
  if (!row || typeof row !== "object") return null;
  var name = String(row.name || "").replace(/^\s+|\s+$/g, "");
  if (!name) return null;
  return {
    name: name,
    rx: nonNeg(row.rx),
    tx: nonNeg(row.tx),
    rxBps: nonNeg(row.rxBps),
    txBps: nonNeg(row.txBps),
  };
}

function parseDisk(row) {
  if (!row || typeof row !== "object") return null;
  var name = String(row.name || "").replace(/^\s+|\s+$/g, "");
  if (!name) return null;
  return {
    name: name,
    readSectors: nonNeg(row.readSectors),
    writeSectors: nonNeg(row.writeSectors),
    readBps: nonNeg(row.readBps),
    writeBps: nonNeg(row.writeBps),
  };
}

function parseSensor(row) {
  if (!row || typeof row !== "object") return null;
  var id = String(row.id || "").replace(/^\s+|\s+$/g, "");
  var kind = String(row.kind || "");
  if (kind !== "temp" && kind !== "fan") return null;
  var value = kind === "temp" ? parseTemp(row.value) : nonNeg(row.value);
  if (value === null) return null;
  return {
    id: id || String(row.chip || "") + ":" + String(row.label || ""),
    chip: String(row.chip || ""),
    label: String(row.label || ""),
    kind: kind,
    value: value,
  };
}

function parsePsi(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  return {
    cpu: nonNeg(src.cpu),
    memory: nonNeg(src.memory),
    io: nonNeg(src.io),
  };
}

function parseTcp(raw) {
  var src = raw && typeof raw === "object" ? raw : {};
  return {
    established: nonNeg(src.established) || 0,
    listen: nonNeg(src.listen) || 0,
    timeWait: nonNeg(src.timeWait) || 0,
    closeWait: nonNeg(src.closeWait) || 0,
    total: nonNeg(src.total) || 0,
  };
}

function mapList(raw, fn) {
  var src = Array.isArray(raw) ? raw : [];
  var out = [];
  var i;
  var row;
  for (i = 0; i < src.length; i++) {
    row = fn(src[i], i);
    if (row) out.push(row);
  }
  return out;
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
    uid: finiteNumber(data.uid),
    cpuIdle: nonNeg(data.cpuIdle),
    cpuTotal: nonNeg(data.cpuTotal),
    cpus: mapList(data.cpus, parseCpuCore),
    load1: nonNeg(data.load1),
    load5: nonNeg(data.load5),
    load15: nonNeg(data.load15),
    memUsed: nonNeg(data.memUsed),
    memTotal: nonNeg(data.memTotal),
    memAvail: nonNeg(data.memAvail),
    memFree: nonNeg(data.memFree),
    memBuffers: nonNeg(data.memBuffers),
    memCached: nonNeg(data.memCached),
    memShared: nonNeg(data.memShared),
    memSReclaimable: nonNeg(data.memSReclaimable),
    memAnon: nonNeg(data.memAnon),
    memDirty: nonNeg(data.memDirty),
    swapUsed: nonNeg(data.swapUsed),
    swapTotal: nonNeg(data.swapTotal),
    netRx: nonNeg(data.netRx),
    netTx: nonNeg(data.netTx),
    ifaces: mapList(data.ifaces, parseIface),
    disks: mapList(data.disks, parseDisk),
    psi: parsePsi(data.psi),
    tcp: parseTcp(data.tcp),
    clkTck: nonNeg(data.clkTck) || 100,
    cpuTemp: parseTemp(data.cpuTemp),
    gpus: parseGpus(data.gpus),
    sensors: mapList(data.sensors, parseSensor),
    processes: processes,
  };
}

function countersPercent(prevIdle, prevTotal, nextIdle, nextTotal) {
  if (prevIdle === null || prevTotal === null) return null;
  if (nextIdle === null || nextTotal === null) return null;
  var dIdle = nextIdle - prevIdle;
  var dTotal = nextTotal - prevTotal;
  if (!(dTotal > 0) || dIdle < 0) return null;
  var used = dTotal - dIdle;
  if (used < 0) used = 0;
  return (used / dTotal) * 100;
}

function cpuPercent(prev, next) {
  if (!prev || !next) return null;
  return countersPercent(prev.cpuIdle, prev.cpuTotal, next.cpuIdle, next.cpuTotal);
}

function corePercent(prev, next) {
  if (!prev || !next) return null;
  return countersPercent(prev.idle, prev.total, next.idle, next.total);
}

function memPercent(sample) {
  if (!sample || sample.memUsed === null || sample.memTotal === null) return null;
  if (!(sample.memTotal > 0)) return null;
  var pct = (sample.memUsed / sample.memTotal) * 100;
  if (pct < 0) return 0;
  return pct;
}

function swapPercent(sample) {
  if (!sample || sample.swapUsed === null || sample.swapTotal === null) return null;
  if (!(sample.swapTotal > 0)) return null;
  var pct = (sample.swapUsed / sample.swapTotal) * 100;
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

function byteRate(prev, next, dtMs) {
  if (prev === null || next === null || !(dtMs > 0)) return null;
  var d = next - prev;
  if (d < 0) return null;
  return d / (dtMs / 1000);
}

function decorateProcesses(prev, sample, dtMs) {
  var list = sample && Array.isArray(sample.processes) ? sample.processes : [];
  var prevList = prev && Array.isArray(prev.processes) ? prev.processes : [];
  var prevMap = {};
  var i;
  for (i = 0; i < prevList.length; i++) {
    if (prevList[i] && prevList[i].pid) prevMap[prevList[i].pid] = prevList[i];
  }
  var clk = sample && sample.clkTck > 0 ? sample.clkTck : 100;
  var out = [];
  var row;
  var last;
  var cpu;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row) continue;
    last = Object.prototype.hasOwnProperty.call(prevMap, row.pid) ? prevMap[row.pid] : null;
    cpu = processCpu(last ? last.ticks : null, row.ticks, dtMs, clk);
    out.push({
      pid: row.pid,
      ppid: row.ppid,
      comm: row.comm,
      cmdline: row.cmdline,
      uid: row.uid,
      rssKb: row.rssKb,
      ticks: row.ticks,
      cpu: cpu,
      state: row.state,
      threads: row.threads,
      nice: row.nice,
      kthread: row.kthread === true,
      mine: row.mine === true ? true : row.mine === false ? false : null,
      readBytes: row.readBytes,
      writeBytes: row.writeBytes,
      readBps: last ? byteRate(last.readBytes, row.readBytes, dtMs) : null,
      writeBps: last ? byteRate(last.writeBytes, row.writeBytes, dtMs) : null,
      depth: 0,
    });
  }
  return out;
}

function decorateCpus(prev, sample) {
  var list = sample && Array.isArray(sample.cpus) ? sample.cpus : [];
  var prevList = prev && Array.isArray(prev.cpus) ? prev.cpus : [];
  var prevMap = {};
  var i;
  for (i = 0; i < prevList.length; i++) {
    if (prevList[i]) prevMap[prevList[i].id] = prevList[i];
  }
  var out = [];
  var row;
  var last;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row) continue;
    last = Object.prototype.hasOwnProperty.call(prevMap, row.id) ? prevMap[row.id] : null;
    out.push({
      id: row.id,
      idle: row.idle,
      total: row.total,
      freqMhz: row.freqMhz,
      governor: row.governor,
      cpu: last ? corePercent(last, row) : null,
    });
  }
  return out;
}

function decorateIfaces(prev, sample, dtMs) {
  var list = sample && Array.isArray(sample.ifaces) ? sample.ifaces : [];
  var prevList = prev && Array.isArray(prev.ifaces) ? prev.ifaces : [];
  var prevMap = {};
  var i;
  for (i = 0; i < prevList.length; i++) {
    if (prevList[i] && prevList[i].name) prevMap[prevList[i].name] = prevList[i];
  }
  var out = [];
  var row;
  var last;
  var rates;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row) continue;
    last = Object.prototype.hasOwnProperty.call(prevMap, row.name) ? prevMap[row.name] : null;
    rates = last
      ? netRate({ netRx: last.rx, netTx: last.tx }, { netRx: row.rx, netTx: row.tx }, dtMs)
      : { rxBps: null, txBps: null };
    out.push({
      name: row.name,
      rx: row.rx,
      tx: row.tx,
      rxBps: rates.rxBps,
      txBps: rates.txBps,
    });
  }
  return out;
}

function sectorRate(prev, next, dtMs) {
  var n = byteRate(prev, next, dtMs);
  if (n === null) return null;
  return n * 512;
}

function decorateDisks(prev, sample, dtMs) {
  var list = sample && Array.isArray(sample.disks) ? sample.disks : [];
  var prevList = prev && Array.isArray(prev.disks) ? prev.disks : [];
  var prevMap = {};
  var i;
  for (i = 0; i < prevList.length; i++) {
    if (prevList[i] && prevList[i].name) prevMap[prevList[i].name] = prevList[i];
  }
  var out = [];
  var row;
  var last;
  for (i = 0; i < list.length; i++) {
    row = list[i];
    if (!row) continue;
    last = Object.prototype.hasOwnProperty.call(prevMap, row.name) ? prevMap[row.name] : null;
    out.push({
      name: row.name,
      readSectors: row.readSectors,
      writeSectors: row.writeSectors,
      readBps: last ? sectorRate(last.readSectors, row.readSectors, dtMs) : null,
      writeBps: last ? sectorRate(last.writeSectors, row.writeSectors, dtMs) : null,
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
    uid: parsed.uid,
    cpuIdle: parsed.cpuIdle,
    cpuTotal: parsed.cpuTotal,
    cpus: decorateCpus(prev, parsed),
    load1: parsed.load1,
    load5: parsed.load5,
    load15: parsed.load15,
    memUsed: parsed.memUsed,
    memTotal: parsed.memTotal,
    memAvail: parsed.memAvail,
    memFree: parsed.memFree,
    memBuffers: parsed.memBuffers,
    memCached: parsed.memCached,
    memShared: parsed.memShared,
    memSReclaimable: parsed.memSReclaimable,
    memAnon: parsed.memAnon,
    memDirty: parsed.memDirty,
    swapUsed: parsed.swapUsed,
    swapTotal: parsed.swapTotal,
    netRx: parsed.netRx,
    netTx: parsed.netTx,
    ifaces: decorateIfaces(prev, parsed, dt),
    disks: decorateDisks(prev, parsed, dt),
    psi: parsed.psi,
    tcp: parsed.tcp,
    clkTck: parsed.clkTck,
    cpuTemp: parsed.cpuTemp,
    gpus: parsed.gpus,
    sensors: parsed.sensors,
    cpu: cpuPercent(prev, parsed),
    mem: memPercent(parsed),
    swap: swapPercent(parsed),
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

function formatLoad(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  return v.toFixed(2);
}

function formatLoadLine(sample) {
  if (!sample) return "";
  var a = formatLoad(sample.load1);
  var b = formatLoad(sample.load5);
  var c = formatLoad(sample.load15);
  if (!a || !b || !c) return "";
  return a + "  " + b + "  " + c;
}

function formatMhz(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  if (v >= 1000) return (Math.round((v / 1000) * 100) / 100).toFixed(2) + " GHz";
  return Math.round(v) + " MHz";
}

function formatRpm(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  return Math.round(v) + " RPM";
}

function formatPsi(n) {
  var v = finiteNumber(n);
  if (v === null) return "";
  if (v < 0.05) return "0%";
  if (v < 10) return (Math.round(v * 10) / 10).toFixed(1) + "%";
  return Math.round(v) + "%";
}

function memParts(sample) {
  if (!sample || sample.memTotal === null || !(sample.memTotal > 0)) return [];
  var total = sample.memTotal;
  var free = sample.memFree != null ? sample.memFree : 0;
  var buffers = sample.memBuffers != null ? sample.memBuffers : 0;
  var cached = sample.memCached != null ? sample.memCached : 0;
  var used = total - free - buffers - cached;
  if (used < 0) used = sample.memUsed != null ? sample.memUsed : 0;
  if (used < 0) used = 0;
  return [
    { id: "used", label: "Used", kb: used },
    { id: "buffers", label: "Buffers", kb: buffers },
    { id: "cached", label: "Cached", kb: cached },
    { id: "free", label: "Free", kb: free },
  ];
}

function namedSeries(history, listKey, matchKey, match, valueKey) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var j;
  var rows;
  var n;
  var needle = String(match);
  for (i = 0; i < list.length; i++) {
    rows = list[i] && Array.isArray(list[i][listKey]) ? list[i][listKey] : [];
    n = null;
    for (j = 0; j < rows.length; j++) {
      if (!rows[j]) continue;
      if (String(rows[j][matchKey]) !== needle) continue;
      n = finiteNumber(rows[j][valueKey]);
      break;
    }
    if (n !== null) out.push(n);
  }
  return out;
}

function coreSeries(history, id) {
  return namedSeries(history, "cpus", "id", id, "cpu");
}

function freqSeries(history, id) {
  return namedSeries(history, "cpus", "id", id, "freqMhz");
}

function ifaceSeries(history, name, key) {
  return namedSeries(history, "ifaces", "name", name, key);
}

function diskSeries(history, name, key) {
  return namedSeries(history, "disks", "name", name, key);
}

function sensorSeries(history, id) {
  return namedSeries(history, "sensors", "id", id, "value");
}

function gpuBusyAt(sample, key) {
  var g = findGpu(sample, key);
  return g ? nonNeg(g.busy) : null;
}

function gpuBusySeries(history, key) {
  var list = Array.isArray(history) ? history : [];
  var out = [];
  var i;
  var n;
  for (i = 0; i < list.length; i++) {
    n = gpuBusyAt(list[i], key);
    if (n !== null) out.push(n);
  }
  return out;
}

function corePercents(sample) {
  var list = sample && Array.isArray(sample.cpus) ? sample.cpus : [];
  var out = [];
  var i;
  for (i = 0; i < list.length; i++) out.push(list[i] && list[i].cpu != null ? list[i].cpu : 0);
  return out;
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
    swapPercent: swapPercent,
    netRate: netRate,
    processCpu: processCpu,
    decorateProcesses: decorateProcesses,
    decorateCpus: decorateCpus,
    decorateIfaces: decorateIfaces,
    decorateDisks: decorateDisks,
    pushSample: pushSample,
    latest: latest,
    series: series,
    parseTemp: parseTemp,
    gpuRows: gpuRows,
    findGpu: findGpu,
    gpuOwnTemp: gpuOwnTemp,
    gpuTempAt: gpuTempAt,
    gpuSeries: gpuSeries,
    gpuBusyAt: gpuBusyAt,
    gpuBusySeries: gpuBusySeries,
    formatPercent: formatPercent,
    formatTemp: formatTemp,
    formatBps: formatBps,
    formatNet: formatNet,
    formatLoad: formatLoad,
    formatLoadLine: formatLoadLine,
    formatMhz: formatMhz,
    formatRpm: formatRpm,
    formatPsi: formatPsi,
    memBytes: memBytes,
    memParts: memParts,
    coreSeries: coreSeries,
    freqSeries: freqSeries,
    ifaceSeries: ifaceSeries,
    diskSeries: diskSeries,
    sensorSeries: sensorSeries,
    corePercents: corePercents,
    sparklinePoints: sparklinePoints,
  };
}
