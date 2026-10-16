const fs = require("fs");
const path = require("path");
const vm = require("vm");

function run(rel, ctx) {
  const src = fs
    .readFileSync(path.join(__dirname, "..", rel), "utf8")
    .replace(/^\.pragma library\s*$/gm, "");
  vm.runInNewContext(src, ctx, { filename: rel });
}

// Qt.include evaluates the other file in this same context. A fresh load()
// does not share that context with the next one.
function includeRelative(fromRel, file, ctx, loading) {
  const child = path.normalize(path.join(path.dirname(fromRel), file));
  if (loading.has(child)) return;
  loading.add(child);
  const previous = ctx.Qt.include;
  ctx.Qt.include = function includeNext(next) {
    includeRelative(child, next, ctx, loading);
  };
  try {
    run(child, ctx);
  } finally {
    ctx.Qt.include = previous;
    loading.delete(child);
  }
}

function load(rel) {
  const ctx = {};
  const loading = new Set();
  ctx.Qt = {
    include: function include(file) {
      includeRelative(rel, file, ctx, loading);
    },
  };
  run(rel, ctx);
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
