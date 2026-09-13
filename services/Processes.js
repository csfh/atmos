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
  return {
    pid: pid,
    comm: comm,
    cmdline: String(row.cmdline || ""),
    rssKb: rssKb,
    cpu: cpu,
  };
}

function haystack(row) {
  if (!row) return "";
  return (row.comm + " " + row.cmdline + " " + row.pid).toLowerCase();
}

function matchesQuery(row, query) {
  var q = String(query || "")
    .replace(/^\s+|\s+$/g, "")
    .toLowerCase();
  if (!q) return true;
  return haystack(row).indexOf(q) !== -1;
}

function sortKey(row, sort) {
  if (sort === "memory") return row && row.rssKb != null ? row.rssKb : -1;
  return row && row.cpu != null ? row.cpu : -1;
}

function list(rows, query, sort) {
  var src = Array.isArray(rows) ? rows : [];
  var key = sort === "memory" ? "memory" : "cpu";
  var out = [];
  var i;
  var row;
  for (i = 0; i < src.length; i++) {
    row = normalize(src[i]);
    if (!row) continue;
    if (!matchesQuery(row, query)) continue;
    out.push(row);
  }
  out.sort(function (a, b) {
    return sortKey(b, key) - sortKey(a, key);
  });
  if (out.length > ROW_CAP) return out.slice(0, ROW_CAP);
  return out;
}

function sortChips() {
  return [
    { id: "cpu", label: "CPU" },
    { id: "memory", label: "Memory" },
  ];
}

function primaryAction() {
  return { id: "term", label: "End" };
}

function overflowActions() {
  return [
    { id: "kill", label: "Force quit" },
    { id: "copy", label: "Copy pid" },
  ];
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
    primaryAction: primaryAction,
    overflowActions: overflowActions,
  };
}
