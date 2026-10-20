// The in-memory write FIFO with one lock. Nothing reads through it: the
// settings are held and pushed by the daemon (docs/adr/0011). QML and Node both
// eval this file.

function createWorkQueue() {
  return { writes: [], running: false, writeSeq: 0 };
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
  if (writes.length > 0) {
    job = writes[0];
    queue.writes = writes.slice(1);
  }
  if (job) queue.running = true;
  return job;
}

function release(queue) {
  if (queue) queue.running = false;
}

function hasQueuedKey(queue, key) {
  var want = String(key || "");
  if (!want || !queue) return false;
  var writes = queue.writes || [];
  var i;
  for (i = 0; i < writes.length; i++) {
    if (writes[i] && String(writes[i].key || "") === want) return true;
  }
  return false;
}

function dropWriteKey(queue, key) {
  if (!queue) return;
  var want = String(key || "");
  if (!want) return;
  var writes = queue.writes || [];
  var next = [];
  var i;
  for (i = 0; i < writes.length; i++) {
    if (writes[i] && String(writes[i].key || "") === want) continue;
    next.push(writes[i]);
  }
  queue.writes = next;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = {
    createWorkQueue: createWorkQueue,
    enqueueWrite: enqueueWrite,
    takeNext: takeNext,
    hasQueuedKey: hasQueuedKey,
    dropWriteKey: dropWriteKey,
    release: release,
  };
}
