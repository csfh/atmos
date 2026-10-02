const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const requests = load("services/Requests.js");
const proto = load("services/BackendProtocol.js");

// The ops the Rust enum accepts, read from its serde renames.
const rust = fs.readFileSync(path.join(__dirname, "..", "backend", "src", "request.rs"), "utf8");
const rustOps = [...rust.matchAll(/#\[serde\(rename = "([^"]+)"\)\]/g)].map((m) => m[1]).sort();
const jsOps = requests.OPS.slice().sort();
assertEqual(
  JSON.stringify(jsOps),
  JSON.stringify(rustOps),
  "Requests.js lists exactly the ops ratmos accepts",
);

assertEqual(requests.settingsSnapshot("look").group, "look", "a snapshot names its group");
assertEqual(requests.settingsSnapshot().group, "all", "a snapshot defaults to all");
assertEqual(
  requests.settingsSnapshot("look", ["a", "b"]).keys.length,
  2,
  "a snapshot can name its keys",
);
assert(requests.settingsSnapshot("look").keys === undefined, "no keys means every key");
assertEqual(
  requests.hostThemePack("x", "/h", "colors.toml").part,
  "colors.toml",
  "a pack can ask for one part",
);
assert(requests.hostThemePack("x", "/h").part === undefined, "no part means both");
assertEqual(requests.speedtestNet().phase, "down", "net speed defaults to download");
assertEqual(
  requests.unitOutput("status", "", "a.service").scope,
  "system",
  "a unit defaults to the system scope",
);
assertEqual(requests.hostStamp(["a", 1]).paths[1], "1", "stamp paths are strings");

for (const body of [
  requests.settingsGet("theme"),
  requests.settingsSet("theme", "x"),
  requests.displaySnapshot(),
  requests.hostChrome(),
  requests.hostAccounts("u", "/h"),
  requests.hostRead(["a"]),
  requests.hostWrite("a", "t"),
  requests.hostOpen("a"),
  requests.speedtestDisk(),
]) {
  assert(requests.OPS.indexOf(body.op) !== -1, body.op + " is a known op");
}

// Protocol
const line = proto.encodeRequest(7, { op: "version", id: 99 });
assert(line.endsWith("\n"), "a request is one line");
assertEqual(JSON.parse(line).id, 7, "the caller's id wins over a stray id in the body");
assertEqual(JSON.parse(line).op, "version", "the body is carried");

const reply = proto.decodeLine('{"id":7,"ok":true,"result":1}');
assertEqual(reply.type, "reply", "a line with an id is a reply");
assertEqual(reply.id, 7, "the reply keeps its id");
assertEqual(reply.envelope.result, 1, "the reply keeps its envelope");
const ev = proto.decodeLine('{"event":"stamp","result":{"items":[]}}');
assertEqual(ev.type, "event", "a line with an event is an event");
assertEqual(ev.name, "stamp", "the event keeps its name");
assertEqual(proto.decodeLine("nonsense").type, "invalid", "junk is invalid");
assertEqual(proto.decodeLine("[1]").type, "invalid", "an array is invalid");
assertEqual(
  proto.decodeLine('{"ok":true}').type,
  "invalid",
  "a line with neither id nor event is invalid",
);
assertEqual(proto.decodeLine("").type, "invalid", "an empty line is invalid");

assertEqual(proto.unavailable("down").ok, false, "unavailable is a failed envelope");
assertEqual(proto.unavailable().error.code, "unavailable", "unavailable has a code");
assertEqual(
  proto.parseEnvelope('{"ok":true,"result":2}', 0).result,
  2,
  "a one-shot envelope parses",
);
assertEqual(proto.parseEnvelope("", 1).ok, false, "no output from a failed run is unavailable");
assertEqual(proto.parseEnvelope("junk", 0).ok, false, "junk output is unavailable");

assertEqual(proto.restartDelay(1), 1000, "the first restart waits a second");
assertEqual(proto.restartDelay(3), 4000, "restarts back off");
assertEqual(proto.restartDelay(50), 30000, "restarts stop backing off at the ceiling");
assertEqual(proto.restartDelay(0), 1000, "a bad count still waits");

const watch = proto.watchBody({
  paths: ["a", 2],
  chrome: true,
  accounts: { user: "u", home: "/h" },
});
assertEqual(watch.op, "watch.set", "a watch body is watch.set");
assertEqual(watch.paths[1], "2", "watch paths are strings");
assertEqual(watch.chrome, true, "chrome can be watched");
assertEqual(watch.accounts.home, "/h", "accounts can be watched");
assertEqual(proto.watchBody({}).accounts, null, "no accounts means none");
assertEqual(proto.watchBody().paths.length, 0, "no spec means nothing watched");

const merged = proto.mergeWatch({
  omarchy: { paths: ["/a", "/b"] },
  theme: { chrome: true },
  accounts: { accounts: { user: "u", home: "/h" } },
  other: { paths: ["/b", "/c"], accounts: { user: "x", home: "/x" } },
});
assertEqual(merged.paths.join(","), "/a,/b,/c", "merged paths are the union, in order");
assertEqual(merged.chrome, true, "chrome is watched if anyone asks");
assertEqual(merged.accounts.user, "u", "the first accounts request wins");
assertEqual(proto.mergeWatch({}).paths.length, 0, "no watchers means no paths");
assertEqual(proto.mergeWatch(null).chrome, false, "null specs merge to nothing");
