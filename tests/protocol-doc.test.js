// docs/protocol.md names every op the Rust backend accepts, `watch.set` which
// `serve` answers itself, and every error code, so the page cannot fall behind
// a protocol change. The ops are read from the same serde renames that
// requests.test.js checks against Requests.js.
const fs = require("fs");
const path = require("path");
const { assert } = require("./harness");

const root = path.join(__dirname, "..");
const doc = fs.readFileSync(path.join(root, "docs", "protocol.md"), "utf8");
const request = fs.readFileSync(path.join(root, "backend", "src", "request.rs"), "utf8");
const errors = fs.readFileSync(path.join(root, "backend", "src", "error.rs"), "utf8");

const ops = [...request.matchAll(/#\[serde\(rename = "([^"]+)"\)\]/g)].map((m) => m[1]);
assert(ops.length > 0, "request.rs lists ops");
for (const op of [...ops, "watch.set"]) {
  assert(doc.includes(`| \`${op}\` |`), `docs/protocol.md has a row for ${op}`);
}

const codes = [...errors.matchAll(/Kind::\w+ => "([a-z_]+)"/g)].map((m) => m[1]);
assert(codes.length > 0, "error.rs lists error codes");
for (const code of [...codes, "unavailable"]) {
  assert(doc.includes(`| \`${code}\` |`), `docs/protocol.md explains the ${code} error code`);
}
