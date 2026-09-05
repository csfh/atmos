// In-memory read/write FIFOs with one lock. QML and Node both eval this file.

function createWorkQueue() {
  return { reads: [], writes: [], running: false, writeSeq: 0 };
}

function snapshotGroupForHub(hub) {
  var id = String(hub || "");
  var slash = id.indexOf("/");
  if (slash !== -1) id = id.substring(0, slash);
  if (
    !id ||
    id === "appearance" ||
    id === "display" ||
    id === "windows" ||
    id === "bar" ||
    id === "notifications" ||
    id === "idle"
  )
    return "look";
  if (id === "network") return "network";
  if (id === "disks") return "disks";
  if (id === "accounts") return "accounts";
  if (id === "system") return "system";
  return "all";
}

function snapshotGroupForWatchPath(path) {
  var p = String(path || "");
  var i = p.lastIndexOf("/");
  var base = i === -1 ? p : p.substring(i + 1);
  if (
    base === "shell.json" ||
    base === "shell.toml" ||
    base === "looknfeel.lua" ||
    base === "hyprsunset.conf" ||
    base === "monitors.lua" ||
    base === "screensaver.txt" ||
    base === "about.txt" ||
    base === "logo.txt" ||
    base === "icon.txt" ||
    base === "logo.png" ||
    base === "fonts.conf" ||
    base === "weather.json" ||
    base === "notifications.json" ||
    p.indexOf("/omarchy/current/background") !== -1 ||
    p.indexOf("/omarchy/themes") !== -1 ||
    p.indexOf("/omarchy/themes/") !== -1 ||
    p.indexOf("/usr/share/omarchy/themes") !== -1 ||
    p.indexOf("/omarchy/indicators") !== -1 ||
    p.indexOf("/omarchy-reminders") !== -1 ||
    p.indexOf("/toggles/hypr") !== -1
  )
    return "look";
  if (p.indexOf("/omarchy/toggles") !== -1 && p.indexOf("/toggles/hypr") === -1) return "all";
  return "rest";
}

function addPendingRefresh(pending, group) {
  var q = { reads: [], writes: [], running: false, writeSeq: 0 };
  var src = Array.isArray(pending) ? pending : [];
  var i;
  for (i = 0; i < src.length; i++) enqueueRead(q, src[i]);
  enqueueRead(q, group || "all");
  var out = [];
  var reads = q.reads || [];
  for (i = 0; i < reads.length; i++) {
    if (reads[i] && reads[i].group) out.push(reads[i].group);
  }
  return out;
}

function enqueueRead(queue, group) {
  var g = String(group || "all");
  if (
    g !== "look" &&
    g !== "rest" &&
    g !== "all" &&
    g !== "network" &&
    g !== "disks" &&
    g !== "accounts" &&
    g !== "system"
  )
    g = "all";
  var reads = queue.reads || [];
  var i;
  for (i = 0; i < reads.length; i++) {
    if (reads[i] && reads[i].kind === "read" && reads[i].group === g) return;
  }
  if (g === "all") {
    var kept = [];
    for (i = 0; i < reads.length; i++) {
      if (reads[i] && reads[i].group === "look") kept.push(reads[i]);
    }
    kept.push({ kind: "read", group: "all" });
    queue.reads = kept;
    return;
  }
  for (i = 0; i < reads.length; i++) {
    if (reads[i] && reads[i].group === "all") return;
  }
  reads.push({ kind: "read", group: g });
  queue.reads = reads;
}

function enqueueWrite(queue, job) {
  if (!job || typeof job !== "object") return;
  queue.writeSeq = (queue.writeSeq || 0) + 1;
  job.writeSeq = queue.writeSeq;
  var writes = queue.writes || [];
  var key = job.key ? String(job.key) : "";
  if (key) {
    var i;
    for (i = 0; i < writes.length; i++) {
      if (writes[i] && writes[i].key === key) {
        writes[i] = job;
        queue.writes = writes;
        return;
      }
    }
  }
  writes.push(job);
  queue.writes = writes;
}

function takeNext(queue) {
  if (!queue || queue.running) return null;
  var job = null;
  var writes = queue.writes || [];
  var reads = queue.reads || [];
  if (writes.length > 0) {
    job = writes[0];
    queue.writes = writes.slice(1);
  } else if (reads.length > 0) {
    job = reads[0];
    queue.reads = reads.slice(1);
  }
  if (job) {
    queue.running = true;
    if (job.kind === "read") job.readSeq = queue.writeSeq || 0;
  }
  return job;
}

function shouldApplyRead(job, queue) {
  if (!job || job.kind !== "read") return false;
  var seq = 0;
  if (queue && typeof queue.writeSeq === "number") seq = queue.writeSeq;
  return job.readSeq === seq;
}

function release(queue) {
  if (queue) queue.running = false;
}

function isIdle(queue) {
  if (!queue) return true;
  var reads = queue.reads || [];
  var writes = queue.writes || [];
  return !queue.running && reads.length === 0 && writes.length === 0;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    createWorkQueue: createWorkQueue,
    snapshotGroupForHub: snapshotGroupForHub,
    snapshotGroupForWatchPath: snapshotGroupForWatchPath,
    addPendingRefresh: addPendingRefresh,
    enqueueRead: enqueueRead,
    enqueueWrite: enqueueWrite,
    takeNext: takeNext,
    shouldApplyRead: shouldApplyRead,
    release: release,
    isIdle: isIdle,
  };
}
