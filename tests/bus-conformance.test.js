// The window's writes and jobs go through the daemon's buses (docs/adr/0004 and
// 0008). This counts what still does not and fails if any count rises. Each
// batch of conversions lowers the counts: run `node tests/bus-conformance.test.js
// --update` after a drop to lower tests/bus-conformance.baseline. The last
// batch takes every count to zero.
const fs = require("fs");
const path = require("path");
const { assert } = require("./harness");

const repo = path.join(__dirname, "..");
const baselineFile = path.join(__dirname, "bus-conformance.baseline");

const files = [];
const walk = (dir) => {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (["node_modules", "backend", "tests", "docs", ".git", "screenshots"].includes(entry.name))
        continue;
      walk(full);
    } else if (/\.(qml|js)$/.test(entry.name)) files.push(full);
  }
};
walk(repo);

// The text between the parentheses of a call that starts at `open`.
const callBody = (text, open) => {
  let depth = 0;
  let quote = "";
  for (let i = open; i < text.length; i++) {
    const c = text[i];
    if (quote) {
      if (c === "\\") i++;
      else if (c === quote) quote = "";
    } else if (c === '"' || c === "'" || c === "`") quote = c;
    else if (c === "(") depth++;
    else if (c === ")" && --depth === 0) return text.slice(open + 1, i);
  }
  return text.slice(open + 1);
};

const counts = {
  // A call that carries no Requests.* op runs an argv the window built.
  runCommandUntyped: 0,
  runJobUntyped: 0,
  runInteractiveUntyped: 0,
  applyCommandCalls: 0,
  // A Process outside the one seam that talks to the backend.
  processOutsideBackend: 0,
  // The window holds the display kinds (Backend.subscribe) and is pushed them.
  displayReads: 0,
  // The window holds the settings topics and is pushed them (docs/adr/0011):
  // it reads no snapshot group and queues no read.
  settingsSnapshotReads: 0,
  enqueueReads: 0,
};
const where = {};
const note = (key, file, text, at) => {
  counts[key]++;
  const line = text.slice(0, at).split("\n").length;
  (where[key] = where[key] || []).push(`${path.relative(repo, file)}:${line}`);
};

for (const file of files) {
  const text = fs.readFileSync(file, "utf8");
  const rel = path.relative(repo, file);
  for (const [name, key] of [
    ["runCommand", "runCommandUntyped"],
    ["runJob", "runJobUntyped"],
    ["runGumJob", "runJobUntyped"],
    ["runInteractive", "runInteractiveUntyped"],
  ]) {
    const re = new RegExp(`(?<![\\w.])${name}\\(`, "g");
    for (let m; (m = re.exec(text));) {
      const before = text.slice(Math.max(0, m.index - 12), m.index);
      if (/function\s+$/.test(before)) continue;
      const body = callBody(text, m.index + name.length);
      if (!/\bop:|\bRequests\.|\bNetPrefs\.opFor|\bnet\.opFor/.test(body))
        note(key, file, text, m.index);
    }
  }
  for (const m of text.matchAll(/Backend\.applyCommand\(/g))
    note("applyCommandCalls", file, text, m.index);
  for (const m of text.matchAll(/Requests\.display(Get|Snapshot)\(/g))
    note("displayReads", file, text, m.index);
  for (const m of text.matchAll(/Requests\.settingsSnapshot\(/g))
    note("settingsSnapshotReads", file, text, m.index);
  if (!rel.endsWith("services/WorkQueue.js")) {
    for (const m of text.matchAll(/(?<!function )\benqueueRead\(/g))
      note("enqueueReads", file, text, m.index);
  }
  if (!rel.endsWith("services/Backend.qml")) {
    for (const m of text.matchAll(/\bProcess\s*\{/g))
      note("processOutsideBackend", file, text, m.index);
  }
}

const baseline = fs.existsSync(baselineFile)
  ? JSON.parse(fs.readFileSync(baselineFile, "utf8"))
  : null;
if (process.argv.includes("--update") || !baseline) {
  const lowered = baseline
    ? Object.fromEntries(
        Object.keys(counts).map((k) => [k, Math.min(counts[k], baseline[k] ?? counts[k])]),
      )
    : counts;
  fs.writeFileSync(baselineFile, JSON.stringify(lowered, null, 2) + "\n");
  console.log(
    `ok - bus-conformance baseline ${baseline ? "lowered" : "written"}: ${JSON.stringify(lowered)}`,
  );
  process.exit(0);
}
for (const key of Object.keys(counts)) {
  const limit = baseline[key] ?? 0;
  assert(
    counts[key] <= limit,
    `${key} is ${counts[key]}, at or under its baseline of ${limit}`,
    `a new one must be a typed op or job; the latest are ${(where[key] || []).slice(-3).join(", ")}`,
  );
  if (counts[key] < limit) {
    console.log(
      `not ok - ${key} dropped to ${counts[key]} from ${limit}; lower the baseline with --update`,
    );
    process.exitCode = 1;
  }
}
console.log(`ok - nothing new bypasses the buses: ${JSON.stringify(counts)}`);
