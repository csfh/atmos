const { load, assert, assertEqual } = require("./harness");

const queue = load("services/WorkQueue.js");
const netIo = queue.createWorkQueue();
queue.enqueueRead(netIo, "network");
assertEqual(netIo.reads[0].group, "network", "enqueueRead keeps a network group");
assertEqual(
  queue.addPendingRefresh(["look"], "rest").join(","),
  "look,rest",
  "pending look plus rest stays both",
);
assertEqual(
  queue.addPendingRefresh(["look", "rest"], "all").join(","),
  "look,all",
  "pending all keeps look and drops rest",
);
const sessionIo = queue.createWorkQueue();
queue.enqueueRead(sessionIo, "look");
queue.enqueueRead(sessionIo, "rest");
assertEqual(sessionIo.reads[0].group, "look", "session queues look before rest");
assertEqual(sessionIo.reads[1].group, "rest", "session queues rest after look");
const io = queue.createWorkQueue();
queue.enqueueRead(io, "look");
queue.enqueueWrite(io, { kind: "mut", argv: ["true"] });
queue.enqueueRead(io, "rest");
const drained = [];
while (true) {
  const job = queue.takeNext(io);
  if (!job) break;
  drained.push(job);
  queue.release(io);
}
assertEqual(drained[0].kind, "mut", "write jobs run before waiting reads");
assertEqual(drained[1].group, "look", "queued look runs before rest");
assertEqual(drained[2].group, "rest", "rest stays last when look was already queued");
assert(queue.isIdle(io), "worker is idle when both queues are empty");
queue.enqueueRead(io, "look");
queue.enqueueRead(io, "look");
assertEqual(io.reads.length, 1, "duplicate look reads coalesce");
queue.enqueueRead(io, "all");
assertEqual(
  io.reads[io.reads.length - 1].group,
  "all",
  "all keeps a pending look then replaces rest",
);
assert(
  io.reads.some(function (job) {
    return job.group === "look";
  }),
  "all does not drop a waiting look",
);

const brightIo = queue.createWorkQueue();
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "brightness:DP-1",
  argv: ["omarchy", "brightness", "40%"],
});
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "brightness:DP-1",
  argv: ["omarchy", "brightness", "80%"],
});
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "brightness:HDMI-A-1",
  argv: ["omarchy", "brightness", "10%"],
});
assertEqual(brightIo.writes.length, 2, "same-key brightness writes coalesce");
assertEqual(brightIo.writes[0].argv[2], "80%", "coalesced write keeps the last percent");
assertEqual(brightIo.writes[1].key, "brightness:HDMI-A-1", "different monitor keys stay separate");
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "theme",
  argv: ["omarchy", "theme", "set", "omarchy"],
  apply: { theme: "omarchy" },
});
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "theme",
  argv: ["omarchy", "theme", "set", "tokyo"],
  apply: { theme: "tokyo" },
});
assertEqual(brightIo.writes.length, 3, "theme writes coalesce separately from brightness");
assertEqual(brightIo.writes[2].apply.theme, "tokyo", "coalesced theme write keeps the last name");
queue.enqueueWrite(brightIo, {
  kind: "job",
  key: "wifi-join",
  argv: ["join", "cafe"],
});
queue.enqueueWrite(brightIo, {
  kind: "job",
  key: "wifi-join",
  argv: ["join", "home"],
});
assertEqual(brightIo.writes.length, 4, "same-kind jobs coalesce");
assertEqual(brightIo.writes[3].argv[1], "home", "coalesced job keeps the last argv");

const staleIo = queue.createWorkQueue();
queue.enqueueRead(staleIo, "all");
const staleRead = queue.takeNext(staleIo);
assert(staleRead && staleRead.kind === "read", "stale-read test starts a snapshot job");
queue.enqueueWrite(staleIo, { kind: "mut", key: "brightness:DP-1", argv: ["x"] });
assert(
  !queue.shouldApplyRead(staleRead, staleIo),
  "snapshot started before a later write is not applied",
);
queue.release(staleIo);
const lateWrite = queue.takeNext(staleIo);
assertEqual(lateWrite.key, "brightness:DP-1", "write runs after the discarded snapshot");
queue.release(staleIo);
assert(queue.isIdle(staleIo), "queue is idle after discarding a stale read and running the write");

const keyedIo = queue.createWorkQueue();
queue.enqueueWrite(keyedIo, { kind: "mut", key: "presentationMode", argv: ["on"] });
queue.takeNext(keyedIo);
queue.enqueueWrite(keyedIo, { kind: "mut", key: "presentationMode", argv: ["off"] });
assert(
  queue.hasQueuedKey(keyedIo, "presentationMode"),
  "hasQueuedKey sees a later write for the same key",
);
assert(!queue.hasQueuedKey(keyedIo, "theme"), "hasQueuedKey misses a key that is not queued");
assert(!queue.hasQueuedKey(keyedIo, ""), "hasQueuedKey ignores an empty key");
