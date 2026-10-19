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
const { load } = require("./harness");

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
// The arguments the window sends are the ones the entry checks: every
// `Requests.write("name", { ... })` literal is filled in from what a real call
// would hold and sent, and none may come back refused.
const home = process.env.HOME || "/home/x";
const hookPath = `${home}/.config/omarchy/hooks/theme-set.d/x`;
const sample = (name, field) => {
  const table = {
    uuid: "123e4567-e89b-12d3-a456-426614174000",
    address: "AA:BB:CC:DD:EE:FF",
    id: "x",
    name: "x",
    on: true,
    enabled: true,
    n: 5,
    minutes: 5,
    message: "m",
    coords: "59.9,10.7",
    command: "true",
    user: "me",
    type: "theme-set",
    payload: "{}",
  };
  if (field === "path") return /^hook\.(remove|sample)$/.test(name) ? hookPath : "/tmp/a";
  return table[field];
};
const variants = { direction: ["up", "down", "off"], band: ["auto", "2.4", "5", "6"] };
const literalArgs = [...text.matchAll(/Requests\.write\("([^"]+)",\s*\{([^}]*)\}\)/g)];
assert(literalArgs.length >= 30, `found the argument literals (${literalArgs.length})`);
for (const [, name, body] of literalArgs) {
  const fields = {};
  const open = [];
  for (const part of body.split(",")) {
    const m = part.match(/^\s*(\w+):\s*(.+?)\s*$/);
    if (!m) continue;
    const [, field, value] = m;
    if (/^"[^"]*"$/.test(value)) fields[field] = JSON.parse(value);
    else if (/^(true|false)$/.test(value)) fields[field] = value === "true";
    else if (/^\d+$/.test(value)) fields[field] = Number(value);
    else if (variants[field]) open.push(field);
    else fields[field] = sample(name, field);
  }
  const runs = open.length ? variants[open[0]].map((v) => ({ ...fields, [open[0]]: v })) : [fields];
  for (const args of runs) {
    const reply = ask({ op: "write", name, args });
    assert(
      reply.ok === true,
      `the window's arguments for ${name} are accepted: ${JSON.stringify(args)} ${reply.error ? reply.error.message : ""}`,
    );
  }
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

// Every setting the window can write names the keys to read back and carries the
// value it sets, since the write goes through settings.set. A record with no
// value would run as a bare command and show nothing afterwards.
const catalog = load("services/Settings.js");
const dynamic = new Set();
for (const row of catalog.settingsCatalog()) {
  const sample =
    row.type === "boolean"
      ? true
      : row.type === "integer" || row.type === "number"
        ? 5
        : row.type === "list"
          ? []
          : row.choices && row.choices[0]
            ? (row.choices[0].value ?? row.choices[0])
            : "x";
  const record = catalog.commandFor(row.key, sample, {}, {});
  if (!record || record.skip) continue;
  assert(record.value !== undefined, `${row.key} writes a value through settings.set`);
  assert(
    Array.isArray(record.readback) && record.readback.length > 0,
    `${row.key} names keys to read back`,
  );
  for (const key of record.readback) dynamic.add(key);
}
const unknownDynamic = ask({ op: "snapshot.keys", keys: [...dynamic] }).result.unknown;
assert(
  unknownDynamic.length === 0,
  `the backend owns every key a setting reads back; unknown: ${JSON.stringify(unknownDynamic)}`,
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
