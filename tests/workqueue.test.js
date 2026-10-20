const { load, assert, assertEqual } = require("./harness");

const queue = load("services/WorkQueue.js");
const io = queue.createWorkQueue();
assert(queue.takeNext(io) === null, "a new queue has nothing to run");
assertEqual(Object.keys(io).indexOf("reads"), -1, "nothing reads through the queue");
queue.enqueueWrite(io, { kind: "mut", key: "a", argv: ["true"] });
queue.enqueueWrite(io, { kind: "mut", key: "b", argv: ["true"] });
assertEqual(queue.takeNext(io).key, "a", "writes run in the order they came");
assert(queue.takeNext(io) === null, "one write runs at a time");
queue.release(io);
assertEqual(queue.takeNext(io).key, "b", "the next write runs once the lock is released");
queue.release(io);
assert(queue.takeNext(io) === null, "nothing runs when the queue is empty");

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
queue.dropWriteKey(brightIo, "brightness:DP-1");
assertEqual(brightIo.writes.length, 3, "dropWriteKey removes one key and leaves the rest");
assert(
  !brightIo.writes.some(function (job) {
    return job.key === "brightness:DP-1";
  }),
  "dropWriteKey removes the named brightness write",
);

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
