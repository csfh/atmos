// A list read back from a QML `property var` after a patch is not always a JS
// Array: it can be a sequence wrapper with a length and indexes, where both
// Array.isArray and instanceof Array say no. The brightness slider emptied the
// monitor list this way: the first patch worked, the second read the wrapper as
// "no list" and wrote [].
const { load, assert, assertEqual } = require("./harness");

const snapshot = load("services/Snapshot.js");

// What QML hands back: object-like, numeric length, indexed, not an Array.
function wrapped(rows) {
  const seq = { length: rows.length };
  rows.forEach((row, i) => {
    seq[i] = row;
  });
  return seq;
}

const monitors = [
  { name: "DP-3", brightness: 33, scale: 1, focused: true },
  { name: "HDMI-A-1", brightness: 70, scale: 1, focused: false },
];

assert(!Array.isArray(wrapped(monitors)), "the fixture is not a real Array");
assert(snapshot.isArray(wrapped(monitors)), "isArray accepts a sequence wrapper");
assert(snapshot.isArray(wrapped([])), "an empty wrapper is a list too");
assert(snapshot.isArray([1]), "isArray still accepts an Array");
for (const notList of ["abc", 5, null, undefined, {}, { name: "x" }, { length: "3" }]) {
  assert(!snapshot.isArray(notList), `isArray rejects ${JSON.stringify(notList)}`);
}
assert(!snapshot.isPlainObject(wrapped(monitors)), "a wrapper is not a plain record");
assert(snapshot.isPlainObject({ a: 1 }), "a record still is");

// The slider: the patch result goes back through a property and comes out wrapped.
let held = wrapped(monitors);
for (const percent of [35, 38, 41, 0, 100]) {
  const patched = snapshot.patchMonitorBrightness(held, "DP-3", percent);
  assertEqual(patched.length, 2, `a brightness patch to ${percent} keeps both monitors`);
  assertEqual(patched[0].brightness, percent, `and sets brightness to ${percent}`);
  assertEqual(patched[1].brightness, 70, "and leaves the other monitor alone");
  held = wrapped(patched);
}
console.log("ok - repeated brightness patches never empty the monitor list");

const scaled = snapshot.patchFocusedMonitorScale(wrapped(monitors), 1.5);
assertEqual(scaled.length, 2, "a scale patch keeps both monitors");
assertEqual(scaled[0].scale, 1.5, "and scales the focused one");

assertEqual(
  snapshot.patchPluginEnabled(wrapped([{ id: "a", enabled: false }]), "a", true).length,
  1,
  "a plugin patch keeps the list",
);

// A reply without the key keeps what the window holds, wrapped or not.
const kept = wrapped(monitors);
assert(snapshot.adoptArray(kept, undefined) === kept, "a missing key keeps a wrapped list");
assertEqual(
  snapshot.adoptArray(wrapped(monitors), [{ name: "DP-3" }]).length,
  1,
  "a new list replaces a wrapped one",
);
console.log("ok - list patches accept a list that came back from a QML property");

// plain() is the boundary copy for a patch that came through a signal.
const original = { monitors: [{ name: "DP-3", availableModes: ["a", "b"] }], n: 1 };
const copy = snapshot.plain(original);
assertEqual(JSON.stringify(copy), JSON.stringify(original), "plain keeps the data");
assert(copy !== original && copy.monitors !== original.monitors, "plain returns a copy");
assert(
  Array.isArray(copy.monitors) && Array.isArray(copy.monitors[0].availableModes),
  "with real arrays",
);
assertEqual(snapshot.plain(undefined), undefined, "plain leaves undefined alone");
console.log("ok - plain copies a patch into real arrays");
