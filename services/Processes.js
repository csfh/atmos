// User-process list for Home. Signals go through signal-process.sh.

var ROW_CAP = 80;

function parsePid(raw) {
  var s = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!/^[1-9][0-9]*$/.test(s)) return 0;
  var n = Number(s);
  if (!isFinite(n) || n < 2) return 0;
  return n;
}

function signalName(raw) {
  var s = String(raw || "")
    .replace(/^\s+|\s+$/g, "")
    .toUpperCase();
  if (s === "TERM" || s === "SIGTERM" || s === "15") return "TERM";
  if (s === "KILL" || s === "SIGKILL" || s === "9") return "KILL";
  return "";
}

function signalArgv(pid, signal, script) {
  var p = parsePid(pid);
  var sig = signalName(signal);
  var path = String(script || "");
  if (!p || !sig || !path) return null;
  return ["bash", path, String(p), sig];
}

function normalize(row) {
  if (!row || typeof row !== "object") return null;
  var pid = Number(row.pid);
  if (!isFinite(pid) || pid <= 0) return null;
  var comm = String(row.comm || "").replace(/^\s+|\s+$/g, "");
  if (!comm) return null;
  var rssKb = Number(row.rssKb);
  if (!isFinite(rssKb) || rssKb < 0) rssKb = null;
  var cpu = Number(row.cpu);
  if (!isFinite(cpu) || cpu < 0) cpu = null;
  var ppid = Number(row.ppid);
  if (!isFinite(ppid) || ppid < 0) ppid = 0;
  var threads = Number(row.threads);
  if (!isFinite(threads) || threads < 0) threads = null;
  var nice = Number(row.nice);
  if (!isFinite(nice)) nice = null;
  var readBps = Number(row.readBps);
  if (!isFinite(readBps) || readBps < 0) readBps = null;
  var writeBps = Number(row.writeBps);
  if (!isFinite(writeBps) || writeBps < 0) writeBps = null;
  var depth = Number(row.depth);
  if (!isFinite(depth) || depth < 0) depth = 0;
  var state = String(row.state || "")
    .replace(/^\s+|\s+$/g, "")
    .charAt(0);
  return {
    pid: pid,
    ppid: ppid,
    comm: comm,
    cmdline: String(row.cmdline || ""),
    uid: Number(row.uid),
    rssKb: rssKb,
    cpu: cpu,
    state: state,
    threads: threads,
    nice: nice,
    kthread: row.kthread === true,
    mine: row.mine === true ? true : row.mine === false ? false : null,
    readBps: readBps,
    writeBps: writeBps,
    depth: depth,
  };
}

function haystack(row) {
  if (!row) return "";
  return (row.comm + " " + row.cmdline + " " + row.pid + " " + row.state).toLowerCase();
}

function matchesQuery(row, query) {
  var q = String(query || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!q) return true;
  return haystack(row).indexOf(q) !== -1;
}

function stateGroup(state) {
  var s = String(state || "");
  if (s === "R") return "running";
  if (s === "S") return "sleeping";
  if (s === "D") return "disk";
  if (s === "Z") return "zombie";
  if (s === "T" || s === "t") return "stopped";
  if (s === "I") return "idle";
  return "other";
}

function inScope(row, scope, uid) {
  if (scope === "kernel") return row.kthread === true;
  if (row.kthread) return false;
  if (scope === "all") return true;
  if (row.mine === true) return true;
  if (row.mine === false) return false;
  if (uid != null && isFinite(Number(uid)) && isFinite(row.uid)) return row.uid === Number(uid);
  return true;
}

function sortKey(row, sort) {
  if (sort === "memory") return row && row.rssKb != null ? row.rssKb : -1;
  if (sort === "pid") return row ? row.pid : -1;
  if (sort === "name") return row && row.comm ? row.comm.toLowerCase() : "";
  if (sort === "threads") return row && row.threads != null ? row.threads : -1;
  if (sort === "read") return row && row.readBps != null ? row.readBps : -1;
  if (sort === "write") return row && row.writeBps != null ? row.writeBps : -1;
  if (sort === "io") {
    var r = row && row.readBps != null ? row.readBps : 0;
    var w = row && row.writeBps != null ? row.writeBps : 0;
    return r + w;
  }
  return row && row.cpu != null ? row.cpu : -1;
}

function compareRows(a, b, sort, reverse) {
  var key = String(sort || "cpu");
  var av = sortKey(a, key);
  var bv = sortKey(b, key);
  var cmp;
  if (typeof av === "string" || typeof bv === "string") {
    cmp = String(av).localeCompare(String(bv));
  } else if (key === "pid") {
    cmp = av - bv;
  } else {
    cmp = bv - av;
  }
  if (reverse) cmp = -cmp;
  if (cmp !== 0) return cmp;
  return a.pid - b.pid;
}

function treeOrder(rows, sort, reverse) {
  var byPid = {};
  var children = {};
  var i;
  var row;
  for (i = 0; i < rows.length; i++) {
    row = rows[i];
    byPid[row.pid] = row;
    children[row.pid] = [];
  }
  var roots = [];
  for (i = 0; i < rows.length; i++) {
    row = rows[i];
    if (row.ppid && byPid[row.ppid]) children[row.ppid].push(row);
    else roots.push(row);
  }
  function sortKids(list) {
    list.sort(function (a, b) {
      return compareRows(a, b, sort, reverse);
    });
  }
  sortKids(roots);
  var out = [];
  function walk(node, depth) {
    node.depth = depth;
    out.push(node);
    var kids = children[node.pid] || [];
    sortKids(kids);
    var k;
    for (k = 0; k < kids.length; k++) walk(kids[k], depth + 1);
  }
  for (i = 0; i < roots.length; i++) walk(roots[i], 0);
  return out;
}

function list(rows, query, sort, opts) {
  var src = Array.isArray(rows) ? rows : [];
  var options = opts && typeof opts === "object" ? opts : {};
  var key = String(sort || "cpu");
  var scope = String(options.scope || "mine");
  var state = String(options.state || "all");
  var uid = options.uid;
  var minCpu = Number(options.minCpu);
  if (!isFinite(minCpu) || minCpu < 0) minCpu = 0;
  var minRssKb = Number(options.minRssKb);
  if (!isFinite(minRssKb) || minRssKb < 0) minRssKb = 0;
  var cap = Number(options.cap);
  if (!isFinite(cap) || cap <= 0) cap = ROW_CAP;
  var reverse = options.reverse === true;
  var tree = options.tree === true;
  var out = [];
  var i;
  var row;
  for (i = 0; i < src.length; i++) {
    row = normalize(src[i]);
    if (!row) continue;
    if (!inScope(row, scope, uid)) continue;
    if (state !== "all" && stateGroup(row.state) !== state) continue;
    if (minCpu > 0 && !(row.cpu != null && row.cpu >= minCpu)) continue;
    if (minRssKb > 0 && !(row.rssKb != null && row.rssKb >= minRssKb)) continue;
    if (!matchesQuery(row, query)) continue;
    out.push(row);
  }
  if (tree) out = treeOrder(out, key, reverse);
  else {
    out.sort(function (a, b) {
      return compareRows(a, b, key, reverse);
    });
  }
  if (out.length > cap) return out.slice(0, cap);
  return out;
}

function sortChips() {
  return [
    { id: "cpu", label: "CPU" },
    { id: "memory", label: "Memory" },
    { id: "pid", label: "Pid" },
    { id: "name", label: "Name" },
    { id: "threads", label: "Threads" },
    { id: "io", label: "I/O" },
  ];
}

function stateChips() {
  return [
    { id: "all", label: "All states" },
    { id: "running", label: "Running" },
    { id: "sleeping", label: "Sleeping" },
    { id: "idle", label: "Idle" },
    { id: "disk", label: "Disk sleep" },
    { id: "stopped", label: "Stopped" },
    { id: "zombie", label: "Zombie" },
  ];
}

function scopeChips() {
  return [
    { id: "mine", label: "Mine" },
    { id: "all", label: "All" },
    { id: "kernel", label: "Kernel" },
  ];
}

function capChips() {
  return [
    { id: "40", label: "40", value: 40 },
    { id: "80", label: "80", value: 80 },
    { id: "200", label: "200", value: 200 },
  ];
}

function minCpuChips() {
  return [
    { id: "0", label: "Any CPU", value: 0 },
    { id: "1", label: "≥1%", value: 1 },
    { id: "5", label: "≥5%", value: 5 },
    { id: "25", label: "≥25%", value: 25 },
  ];
}

function minRssChips() {
  return [
    { id: "0", label: "Any RSS", value: 0 },
    { id: "10240", label: "≥10 MB", value: 10240 },
    { id: "102400", label: "≥100 MB", value: 102400 },
    { id: "1048576", label: "≥1 GB", value: 1048576 },
  ];
}

function stateLabel(state) {
  var g = stateGroup(state);
  if (g === "running") return "R";
  if (g === "sleeping") return "S";
  if (g === "disk") return "D";
  if (g === "zombie") return "Z";
  if (g === "stopped") return "T";
  if (g === "idle") return "I";
  return state || "?";
}

function canSignal(row) {
  if (!row) return false;
  if (row.kthread) return false;
  if (row.mine === false) return false;
  return true;
}

function primaryAction() {
  return { id: "term", label: "End" };
}

function overflowActions(row) {
  var items = [
    { id: "kill", label: "Force quit" },
    { id: "copy", label: "Copy pid" },
    { id: "copyCmd", label: "Copy command" },
  ];
  if (row && !canSignal(row)) {
    return items.filter(function (item) {
      return item.id !== "kill";
    });
  }
  return items;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    ROW_CAP: ROW_CAP,
    parsePid: parsePid,
    signalName: signalName,
    signalArgv: signalArgv,
    normalize: normalize,
    list: list,
    sortChips: sortChips,
    stateChips: stateChips,
    scopeChips: scopeChips,
    capChips: capChips,
    minCpuChips: minCpuChips,
    minRssChips: minRssChips,
    stateGroup: stateGroup,
    stateLabel: stateLabel,
    canSignal: canSignal,
    primaryAction: primaryAction,
    overflowActions: overflowActions,
  };
}
