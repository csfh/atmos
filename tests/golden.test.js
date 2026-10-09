// The protocol contract, recorded. Every case in tests/golden/cases.js is
// replayed against the real ratmos on a fixture root, over both transports the
// app uses (one-shot `request` and long-lived `serve`), and the reply must equal
// the one stored in tests/golden/cases/<name>.json. The request is built with
// the same Requests.js builder the app calls, so a builder that drifts from the
// backend, or a backend change the front end has not caught up with, fails here.
//
// After an intended change:  ATMOS_GOLDEN_RECORD=1 node tests/golden.test.js
// then read the diff of tests/golden/cases: that diff is the protocol change.
const fs = require("fs");
const path = require("path");
const { spawn, spawnSync } = require("child_process");
const { load, assert, assertEqual } = require("./harness");
const { createSandbox, ratmosBinary } = require("./sandbox");
const { cases } = require("./golden/cases");

const record = process.env.ATMOS_GOLDEN_RECORD === "1";
const casesDir = path.join(__dirname, "golden", "cases");
const proto = load("services/BackendProtocol.js");
const requests = load("services/Requests.js");

const bin = ratmosBinary();
// The backend runs in the sandbox: a throwaway HOME and a PATH of fakes, so a
// fixture-root request that reached for a real command would be recorded and
// fail instead of touching this machine.
const box = createSandbox();
const ROOT = "<root>";
const FIXED_TIME = 1700000000;

function substitute(value, from, to) {
  return JSON.parse(JSON.stringify(value).split(from).join(to));
}

function prepare(c, index) {
  const root = path.join(box.root, "cases", String(index));
  fs.mkdirSync(root, { recursive: true });
  for (const [rel, text] of Object.entries(c.files || {})) {
    const file = path.join(root, rel);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, substitute(text, ROOT, root));
    // host.stamp signs a file by size and mtime, so the mtime has to be fixed.
    fs.utimesSync(file, FIXED_TIME, FIXED_TIME);
  }
  return root;
}

function oneShot(backend, root, body) {
  const r = box.spawn(bin, ["--backend", backend, "--root", root, "request"], {
    input: JSON.stringify(body),
  });
  return { ...r, envelope: JSON.parse(r.out) };
}

// `serve` answers on a long-lived pipe, so ask, wait for the reply line, then
// close stdin to let it exit.
function overServe(backend, root, body) {
  const id = 41;
  return new Promise((resolve) => {
    const child = spawn(bin, ["--backend", backend, "--root", root, "serve"], {
      env: box.env(),
    });
    let buffer = "";
    const finish = (envelope) => {
      clearTimeout(timer);
      child.kill();
      resolve(envelope);
    };
    const timer = setTimeout(() => finish(null), 10000);
    child.stdout.on("data", (chunk) => {
      buffer += chunk.toString();
      let nl;
      while ((nl = buffer.indexOf("\n")) !== -1) {
        const msg = proto.decodeLine(buffer.slice(0, nl));
        buffer = buffer.slice(nl + 1);
        if (msg.type === "reply" && msg.id === id) return finish(msg.envelope);
      }
    });
    child.stdin.write(proto.encodeRequest(id, body));
  });
}

function same(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

fs.mkdirSync(casesDir, { recursive: true });
const names = new Set();
const ops = new Set();

async function replay(c, index) {
  assert(!names.has(c.name), `golden case ${c.name} has a unique name`);
  names.add(c.name);
  ops.add(c.request.op);

  const root = prepare(c, index);
  box.resetCalls();
  const body = substitute(c.request, ROOT, root);
  for (const step of c.setup || []) oneShot(c.backend, root, substitute(step, ROOT, root));

  const shot = oneShot(c.backend, root, body);
  // The stored form names the root as <root> and drops nothing else.
  const response = substitute(shot.envelope, root, ROOT);
  const file = path.join(casesDir, `${c.name}.json`);
  const stored = {
    backend: c.backend,
    ...(c.files ? { files: c.files } : {}),
    ...(c.setup ? { setup: c.setup } : {}),
    request: c.request,
    response,
  };
  if (record) fs.writeFileSync(file, JSON.stringify(stored, null, 2) + "\n");

  assert(
    fs.existsSync(file),
    `golden case ${c.name} has a recorded file (run with ATMOS_GOLDEN_RECORD=1)`,
  );
  const golden = JSON.parse(fs.readFileSync(file, "utf8"));

  assert(
    same(golden.request, c.request) && golden.backend === c.backend,
    `${c.name}: the stored request is what the builder produces now`,
    `stored:  ${JSON.stringify(golden.request)}\nbuilder: ${JSON.stringify(c.request)}`,
  );
  assert(
    same(golden.response, response),
    `${c.name}: the one-shot reply matches the recording`,
    `recorded: ${JSON.stringify(golden.response)}\nactual:   ${JSON.stringify(response)}`,
  );

  const served = await overServe(c.backend, root, body);
  assert(served !== null, `${c.name}: serve answers the request`);
  const { id: _id, ...servedBody } = served;
  assert(
    same(substitute(servedBody, root, ROOT), response),
    `${c.name}: serve and one-shot give the same reply`,
    `serve:    ${JSON.stringify(servedBody)}\none-shot: ${JSON.stringify(response)}`,
  );
  assertEqual(served.id, 41, `${c.name}: serve echoes the request id`);

  assertEqual(
    JSON.stringify(box.callLines()),
    "[]",
    `${c.name}: a fixture-root request reaches no faked command`,
  );

  // What the front end does with the reply.
  assert(
    same(proto.parseEnvelope(JSON.stringify(golden.response), 0), golden.response),
    `${c.name}: parseEnvelope keeps the reply`,
  );
  const decoded = proto.decodeLine(JSON.stringify({ ...golden.response, id: 9 }));
  assert(decoded.type === "reply" && decoded.id === 9, `${c.name}: decodeLine reads it as a reply`);
}

async function main() {
  for (let index = 0; index < cases.length; index++) await replay(cases[index], index);
  finish();
}

function finish() {
  // The repo formatter owns .json under tests/, so leave the recordings in the
  // shape `oxfmt --check` expects.
  const oxfmt = path.join(__dirname, "..", "node_modules", ".bin", "oxfmt");
  if (record && fs.existsSync(oxfmt)) spawnSync(oxfmt, [casesDir]);
  // Every op the app can send has at least one recorded case.
  for (const op of requests.OPS) {
    assert(ops.has(op), `an op ${op} has a golden case`);
  }

  // Nothing stale is left behind in the recordings.
  for (const file of fs.readdirSync(casesDir)) {
    assert(names.has(file.replace(/\.json$/, "")), `recording ${file} belongs to a case`);
  }
}

main();
