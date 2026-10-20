// The architecture decision records stay well formed: each has its sections in
// order, is listed in the index, and every file it says enforces it exists.
const fs = require("fs");
const path = require("path");
const { assert } = require("./harness");

const root = path.join(__dirname, "..");
const dir = path.join(root, "docs", "adr");
const index = fs.readFileSync(path.join(dir, "README.md"), "utf8");
const records = fs
  .readdirSync(dir)
  .filter((name) => /^\d{4}-.+\.md$/.test(name))
  .sort();
assert(records.length >= 8, `the decisions are recorded (found ${records.length})`);

// Every file in the repo by name, for the records that name a bare file.
const names = new Set();
const walk = (folder) => {
  for (const entry of fs.readdirSync(folder, { withFileTypes: true })) {
    if ([".git", "node_modules", "target"].includes(entry.name)) continue;
    const full = path.join(folder, entry.name);
    if (entry.isDirectory()) walk(full);
    else names.add(path.relative(root, full));
  }
};
walk(root);
const byBase = new Set([...names].map((file) => path.basename(file)));

const sections = ["Context", "Decision", "Consequences", "Alternatives considered", "Enforced by"];
const numbers = [];
for (const file of records) {
  const text = fs.readFileSync(path.join(dir, file), "utf8");
  const number = file.slice(0, 4);
  numbers.push(Number(number));
  assert(text.startsWith(`# ${number}. `), `${file} is titled with its number`);
  assert(
    /^Status: (Accepted|Deprecated|Superseded by \d{4})\. Date: \d{4}-\d{2}-\d{2}\.$/m.test(text),
    `${file} has a status and a date`,
  );
  let at = -1;
  for (const heading of sections) {
    const found = text.indexOf(`\n## ${heading}\n`);
    assert(found > at, `${file} has "${heading}" after the one before it`);
    at = found;
  }
  assert(index.includes(`(${file})`), `${file} is in the index`);

  const enforced = text.slice(text.indexOf("\n## Enforced by\n"));
  for (const [, token] of enforced.matchAll(/`([^`]+)`/g)) {
    const looksLikeFile =
      token.includes("/") || /\.(rs|js|qml|md|service|socket|toml)$/.test(token);
    if (!looksLikeFile || /\s|\(/.test(token)) continue;
    const exists = token.includes("/") ? names.has(token) : byBase.has(token);
    assert(exists, `${file} names ${token}, which exists`);
  }
}
numbers.forEach((n, i) =>
  assert(n === i + 1, `ADR numbers run 1, 2, 3 with no gaps (${n} at ${i + 1})`),
);
for (const [, link] of index.matchAll(/\]\(([^)]+\.md)\)/g)) {
  assert(records.includes(link), `the index links to a record that exists: ${link}`);
}
console.log("ok - the ADRs are well formed and name files that exist");
