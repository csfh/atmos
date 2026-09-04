const fs = require("fs");
const path = require("path");
const vm = require("vm");

function load(rel) {
  const src = fs.readFileSync(path.join(__dirname, "..", rel), "utf8");
  const ctx = {};
  vm.runInNewContext(src, ctx, { filename: rel });
  return ctx;
}

function assert(condition, description, detail) {
  if (!condition) {
    if (detail) console.error(detail);
    console.error(`not ok - ${description}`);
    process.exit(1);
  }
  console.log(`ok - ${description}`);
}

function assertEqual(actual, expected, description) {
  assert(actual === expected, description, `expected: ${expected}\nactual:   ${actual}`);
}

module.exports = { load, assert, assertEqual };
