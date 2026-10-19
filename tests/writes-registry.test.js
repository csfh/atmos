// The window and the backend agree on the typed writes: every write name the
// QML sends is one the backend registry knows, and every snapshot key the
// window asks the backend to read back is one a producer owns. A name or key
// typed wrong would otherwise fail only when somebody used that control.
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { assert, assertEqual } = require("./harness");
const { ratmosBinary } = require("./sandbox");

const repo = path.join(__dirname, "..");
const read = (file) => fs.readFileSync(path.join(repo, file), "utf8");
const sources = [
  "services/Omarchy.qml",
  "services/Settings.js",
  "services/HyprPrefs.js",
  "services/Ops.qml",
];
const text = sources.map(read).join("\n");

const root = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-writes-"));
const ask = (body) => {
  const out = spawnSync(ratmosBinary(), ["--backend", "omarchy", "--root", root, "request"], {
    input: JSON.stringify(body),
    encoding: "utf8",
  });
  return JSON.parse(out.stdout);
};

// Every `Requests.write("name"` in the window is a name the backend builds.
const names = [...new Set([...text.matchAll(/Requests\.write\("([^"]+)"/g)].map((m) => m[1]))];
assert(names.length >= 30, `the window sends many typed writes (found ${names.length})`);
for (const name of names) {
  const reply = ask({ op: "write", name, args: {} });
  const unknown = reply.ok === false && /unknown write/.test(reply.error.message);
  assert(!unknown, `the backend knows the write ${name}`);
}
assertEqual(
  ask({ op: "write", name: "no.such.write", args: {} }).error.code,
  "bad_request",
  "an unknown write is a bad request",
);

// Every key named in a read-back list is one the backend can read.
const lists = [...text.matchAll(/(?:readback|keys): \[([^\]]*)\]/g)];
const keys = new Set();
for (const m of lists) for (const k of m[1].matchAll(/"([A-Za-z0-9]+)"/g)) keys.add(k[1]);
assert(keys.size >= 15, `the window reads back many keys (found ${keys.size})`);
const reply = ask({ op: "snapshot.keys", keys: [...keys] });
assertEqual(
  reply.result.unknown.length,
  0,
  `the backend owns every read-back key; unknown: ${JSON.stringify(reply.result.unknown)}`,
);

// Every setting the window can write is read back under a key the backend
// owns, or is one of the few that live in a store of their own.
const own = new Set([
  "customDns",
  "favorites",
  "sentinelCpuPercent",
  "sentinelMemoryPercent",
  "sentinelPortAgeDays",
  "internalDisplay",
  "internalMirror",
  "monitorScale",
  "displayBrightness",
]);
const domains = ask({ op: "settings.list" }).result.map((row) => row.domain.split(".")[0]);
const heads = [...new Set(domains)].filter((d) => !own.has(d));
const unowned = ask({ op: "snapshot.keys", keys: heads }).result.unknown;
assertEqual(
  unowned.length,
  0,
  `every setting is read back from a key the backend owns; unknown: ${JSON.stringify(unowned)}`,
);

fs.rmSync(root, { recursive: true, force: true });
console.log("ok - the window's typed writes and read-back keys exist in the backend");
