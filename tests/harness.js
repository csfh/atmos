const fs = require("fs");
const path = require("path");
const vm = require("vm");

function load(rel) {
  const src = fs
    .readFileSync(path.join(__dirname, "..", rel), "utf8")
    .replace(/^\.pragma library\s*$/m, "");
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

// JSON text with object keys in sorted order, so two writers that differ only
// in key order compare equal.
function canon(value) {
  return JSON.stringify(value, function (key, val) {
    if (val && typeof val === "object" && !Array.isArray(val)) {
      return Object.keys(val)
        .sort()
        .reduce(function (out, k) {
          out[k] = val[k];
          return out;
        }, {});
    }
    return val;
  });
}

module.exports = { load, assert, assertEqual, canon };
