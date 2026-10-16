// Sentinel hub: rogue processes from the live sample. High CPU for a while,
// a large share of memory, or a listening port held by a long-running process.
// `latest` is a LiveStats sample. Nothing here reads the system.

var DEFAULTS = { cpuPercent: 80, memoryPercent: 20, portAgeDays: 7 };
var LIMITS = { cpuPercent: [50, 100], memoryPercent: [5, 90], portAgeDays: [1, 90] };
// CPU has to stay over its threshold for this many samples in a row.
var SUSTAIN = 3;
var ROW_CAP = 60;
var DAY = 86400;

function clampThreshold(key, raw) {
  var lim = LIMITS[key];
  var n = Math.round(Number(raw));
  if (!lim || !isFinite(n) || raw === null || raw === "") return DEFAULTS[key];
  return Math.min(lim[1], Math.max(lim[0], n));
}

// `src` holds the stored values by key (sentinelCpuPercent, ...). Unset or
// out-of-range values fall back to the defaults.
function thresholds(src) {
  var s = src && typeof src === "object" ? src : {};
  return {
    cpuPercent: clampThreshold("cpuPercent", s.sentinelCpuPercent),
    memoryPercent: clampThreshold("memoryPercent", s.sentinelMemoryPercent),
    portAgeDays: clampThreshold("portAgeDays", s.sentinelPortAgeDays),
  };
}

function humanKb(kb) {
  var n = Number(kb);
  if (!isFinite(n) || n < 0) return "";
  if (n < 1024) return Math.round(n) + " kB";
  n = n / 1024;
  if (n < 1024) return (n >= 100 ? Math.round(n) : n.toFixed(1)) + " MB";
  return (n / 1024).toFixed(1) + " GB";
}

function humanAge(sec) {
  var n = Number(sec);
  if (!isFinite(n) || n < 0) return "";
  if (n < 90) return Math.round(n) + " s";
  if (n < 5400) return Math.round(n / 60) + " min";
  if (n < DAY * 2) return Math.round(n / 3600) + " h";
  return Math.round(n / DAY) + " d";
}

function usable(row) {
  return !!row && !row.kthread && row.pid > 1;
}

// Keeps, per pid, when it first went over the CPU threshold and for how many
// samples in a row. A pid that drops under the threshold, or disappears, resets.
// The same sample passed twice leaves the state alone.
function track(prev, latest, th) {
  var old = prev && prev.over ? prev.over : {};
  if (!latest || !Array.isArray(latest.processes)) return { at: null, over: {} };
  if (prev && prev.at === latest.at) return prev;
  var limit = th ? th.cpuPercent : DEFAULTS.cpuPercent;
  var over = {};
  var i;
  var row;
  for (i = 0; i < latest.processes.length; i++) {
    row = latest.processes[i];
    if (!usable(row) || row.cpu === null || !(row.cpu >= limit)) continue;
    over[row.pid] = old[row.pid]
      ? { since: old[row.pid].since, n: old[row.pid].n + 1 }
      : { since: latest.at, n: 1 };
  }
  return { at: latest.at, over: over };
}

// "local only" matters more than the address: it is what decides whether the
// port can be reached from another machine.
function reach(addr) {
  var a = String(addr || "");
  if (a === "0.0.0.0" || a === "::") return "all interfaces";
  if (a.startsWith("127.") || a === "::1") return "local only";
  return a;
}

function procAge(proc, latest) {
  var clk = latest.clkTck > 0 ? latest.clkTck : 100;
  if (!proc || proc.startTicks === null || proc.startTicks === undefined) return null;
  if (latest.uptimeSec === null || latest.uptimeSec === undefined) return null;
  var age = latest.uptimeSec - proc.startTicks / clk;
  return age >= 0 ? age : null;
}

// Every listening port, once per process and port. A listener whose process is
// unreadable keeps its row with `pid: null` and a note instead of an age.
function listeners(latest) {
  var socks = latest && Array.isArray(latest.tcpSockets) ? latest.tcpSockets : [];
  var procs = latest && Array.isArray(latest.processes) ? latest.processes : [];
  var byPid = {};
  var i;
  for (i = 0; i < procs.length; i++) byPid[procs[i].pid] = procs[i];
  var seen = {};
  var out = [];
  var s;
  var key;
  var proc;
  for (i = 0; i < socks.length; i++) {
    s = socks[i];
    if (!s || s.state !== "LISTEN" || s.port === null || s.port === undefined) continue;
    key = (s.pid || "?" + s.uid) + "|" + s.port;
    if (seen[key]) continue;
    seen[key] = true;
    proc = s.pid ? byPid[s.pid] || null : null;
    out.push({
      port: s.port,
      addr: s.addr,
      reach: reach(s.addr),
      uid: s.uid,
      pid: s.pid || null,
      proc: proc,
      age: procAge(proc, latest),
    });
  }
  return out;
}

function portDetail(l) {
  var bits = [":" + l.port + " · " + l.reach];
  if (l.pid === null) bits.push("owner unknown (uid " + l.uid + ")");
  else if (l.age !== null) bits.push("process up " + humanAge(l.age));
  return bits.join(" · ");
}

function portRow(l, th, flagged) {
  var limit = th.portAgeDays * DAY;
  return {
    id: "port:" + l.port + ":" + (l.pid || "u" + l.uid),
    kind: "port",
    pid: l.pid,
    comm: l.proc ? l.proc.comm : ":" + l.port,
    cmdline: l.proc ? l.proc.cmdline : "",
    port: l.port,
    reach: l.reach,
    age: l.age,
    detail: portDetail(l),
    ratio: l.age === null ? 0 : l.age / limit,
    flagged: flagged,
    proc: l.proc,
  };
}

function rows(latest, state, th) {
  var t = th || thresholds(null);
  var out = [];
  if (!latest) return out;
  var over = state && state.over ? state.over : {};
  var procs = Array.isArray(latest.processes) ? latest.processes : [];
  var i;
  var p;
  var st;
  var pct;
  for (i = 0; i < procs.length; i++) {
    p = procs[i];
    if (!usable(p)) continue;
    st = over[p.pid];
    if (st && st.n >= SUSTAIN) {
      out.push({
        id: "cpu:" + p.pid,
        kind: "cpu",
        pid: p.pid,
        comm: p.comm,
        cmdline: p.cmdline,
        detail: Math.round(p.cpu) + "% CPU for " + humanAge((latest.at - st.since) / 1000),
        ratio: p.cpu / t.cpuPercent,
        flagged: true,
        proc: p,
      });
    }
    if (latest.memTotal > 0 && p.rssKb !== null) {
      pct = (100 * p.rssKb) / latest.memTotal;
      if (pct >= t.memoryPercent) {
        out.push({
          id: "memory:" + p.pid,
          kind: "memory",
          pid: p.pid,
          comm: p.comm,
          cmdline: p.cmdline,
          detail: humanKb(p.rssKb) + ", " + Math.round(pct) + "% of memory",
          ratio: pct / t.memoryPercent,
          flagged: true,
          proc: p,
        });
      }
    }
  }
  var ls = listeners(latest);
  for (i = 0; i < ls.length; i++) {
    out.push(portRow(ls[i], t, ls[i].age !== null && ls[i].age >= t.portAgeDays * DAY));
  }
  return out;
}

function byRatioDesc(a, b) {
  if (b.ratio !== a.ratio) return b.ratio - a.ratio;
  return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
}

// A process with several old listeners is one finding, not one per port.
function collapse(src) {
  var out = [];
  var groups = {};
  var i;
  var r;
  var g;
  for (i = 0; i < src.length; i++) {
    r = src[i];
    if (r.kind !== "port" || !r.flagged || r.pid === null) {
      out.push(r);
      continue;
    }
    g = groups[r.pid];
    if (!g) {
      g = groups[r.pid] = { first: r, ports: [], at: out.length };
      out.push(r);
    }
    g.ports.push(r);
  }
  Object.keys(groups).forEach(function (pid) {
    var grp = groups[pid];
    if (grp.ports.length < 2) return;
    var first = grp.first;
    var shown = grp.ports.slice(0, 3).map(function (p) {
      return ":" + p.port + " " + p.reach;
    });
    if (grp.ports.length > 3) shown.push("+" + (grp.ports.length - 3) + " more");
    out[grp.at] = {
      id: "ports:" + pid,
      kind: "port",
      pid: first.pid,
      comm: first.comm,
      cmdline: first.cmdline,
      detail: "process up " + humanAge(first.age) + " · " + shown.join(", "),
      search: grp.ports
        .map(function (p) {
          return ":" + p.port;
        })
        .join(" "),
      ratio: first.ratio,
      flagged: true,
      proc: first.proc,
    };
  });
  return out;
}

function flaggedRows(allRows) {
  var src = Array.isArray(allRows) ? allRows : [];
  return collapse(
    src.filter(function (r) {
      return r.flagged;
    }),
  );
}

// What the page shows. "flagged" is everything over a threshold; "port" lists
// every listener, long-running ones first; "cpu" and "memory" narrow the flags.
function list(allRows, kind, query) {
  var k = String(kind || "flagged");
  var q = String(query || "").toLowerCase();
  var src = Array.isArray(allRows) ? allRows : [];
  var out = src.filter(function (r) {
    return r.kind === k && (k === "port" || r.flagged);
  });
  if (k === "flagged") out = flaggedRows(src);
  if (q) {
    out = out.filter(function (r) {
      var text = r.comm + " " + r.cmdline + " " + r.detail + " " + r.pid + " " + (r.search || "");
      return text.toLowerCase().indexOf(q) >= 0;
    });
  }
  out.sort(byRatioDesc);
  return out.slice(0, ROW_CAP);
}

function counts(allRows) {
  var c = { flagged: 0, cpu: 0, memory: 0, port: 0 };
  var src = Array.isArray(allRows) ? allRows : [];
  var i;
  for (i = 0; i < src.length; i++) {
    if (src[i].kind === "port") c.port++;
    if (src[i].flagged && src[i].kind !== "port") c[src[i].kind]++;
  }
  c.flagged = flaggedRows(src).length;
  return c;
}

function kindChips(allRows) {
  var c = counts(allRows);
  return [
    { id: "flagged", label: "Flagged", count: c.flagged },
    { id: "cpu", label: "CPU", count: c.cpu },
    { id: "memory", label: "Memory", count: c.memory },
    { id: "port", label: "Ports", count: c.port },
  ];
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    DEFAULTS: DEFAULTS,
    LIMITS: LIMITS,
    SUSTAIN: SUSTAIN,
    clampThreshold: clampThreshold,
    thresholds: thresholds,
    humanKb: humanKb,
    humanAge: humanAge,
    track: track,
    listeners: listeners,
    rows: rows,
    list: list,
    counts: counts,
    kindChips: kindChips,
  };
}
