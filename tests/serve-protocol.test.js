// The JS side of the line protocol against the real `ratmos serve`: whatever
// BackendProtocol.js encodes, the server must answer in a way decodeLine reads.
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawn, spawnSync } = require("child_process");
const { load, assert, assertEqual } = require("./harness");

const proto = load("services/BackendProtocol.js");
const requests = load("services/Requests.js");

const repo = path.join(__dirname, "..");
const bin = path.join(repo, "backend", "target", "debug", "ratmos");
if (!fs.existsSync(bin)) {
  const built = spawnSync("cargo", [
    "build",
    "--manifest-path",
    path.join(repo, "backend", "Cargo.toml"),
  ]);
  assertEqual(built.status, 0, "ratmos builds for the protocol test");
}

const root = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-proto-"));
const child = spawn(bin, ["--backend", "plain", "--root", root, "serve"]);
const replies = new Map();
const events = [];
let buffer = "";
let waiting = null;

child.stdout.on("data", (chunk) => {
  buffer += chunk.toString();
  let nl;
  while ((nl = buffer.indexOf("\n")) !== -1) {
    const msg = proto.decodeLine(buffer.slice(0, nl));
    buffer = buffer.slice(nl + 1);
    if (msg.type === "reply") replies.set(msg.id, msg.envelope);
    else if (msg.type === "event") events.push(msg);
    if (waiting) waiting();
  }
});

function until(check) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("timed out")), 10000);
    const test = () => {
      const value = check();
      if (value) {
        clearTimeout(timer);
        waiting = null;
        resolve(value);
      }
    };
    waiting = test;
    test();
  });
}

function ask(id, body) {
  child.stdin.write(proto.encodeRequest(id, body));
  return until(() => replies.get(id));
}

(async () => {
  const set = await ask(1, requests.settingsSet("theme", "tokyo"));
  assertEqual(set.ok, true, "settings.set answers over the line protocol");
  assertEqual(set.result.value, "tokyo", "the written value comes back");

  const got = await ask(2, requests.settingsGet("theme"));
  assertEqual(got.result, "tokyo", "settings.get reads it back on the same process");

  const bad = await ask(3, { op: "nope" });
  assertEqual(bad.ok, false, "an unknown op is a failed envelope");
  assertEqual(bad.error.code, "bad_request", "the failure has a code");

  const watch = await ask(4, proto.watchBody({ paths: ["note.txt"], chrome: true }));
  assertEqual(watch.ok, true, "watch.set is accepted");
  const stamp = await until(() => events.find((e) => e.name === "stamp"));
  assertEqual(stamp.result.items[0].path, "note.txt", "a stamp event follows a watch");
  const chrome = await until(() => events.find((e) => e.name === "chrome"));
  assert(typeof chrome.result === "object", "a chrome event follows a chrome watch");

  child.stdin.end();
  await new Promise((resolve) => child.on("close", resolve));
  fs.rmSync(root, { recursive: true, force: true });
  console.log("ok - the JS protocol and ratmos serve agree");
})().catch((err) => {
  console.error(err);
  console.error("not ok - the JS protocol and ratmos serve agree");
  child.kill();
  process.exit(1);
});
