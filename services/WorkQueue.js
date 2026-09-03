// In-memory read/write FIFOs with one lock. QML and Node both eval this file.

function createWorkQueue() {
  return { reads: [], writes: [], running: false };
}

function snapshotGroupForHub(hub) {
  var id = String(hub || "");
  var slash = id.indexOf("/");
  if (slash !== -1) id = id.substring(0, slash);
  if (!id || id === "appearance") return "look";
  return "all";
}

function enqueueRead(queue, group) {
  var g = String(group || "all");
  if (g !== "look" && g !== "rest" && g !== "all") g = "all";
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
  var writes = queue.writes || [];
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
  if (job) queue.running = true;
  return job;
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
    enqueueRead: enqueueRead,
    enqueueWrite: enqueueWrite,
    takeNext: takeNext,
    release: release,
    isIdle: isIdle,
  };
}
