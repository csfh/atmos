const { load, assert, assertEqual } = require("./harness");

const mcp = load("services/Mcp.js");

function round(line, call) {
  let out = "missing";
  mcp.handleMessage(mcp.decodeMessage(line), call || function () {}, function (response) {
    out = response;
  });
  return out;
}

function callMessage(id, name, args) {
  return JSON.stringify({
    jsonrpc: "2.0",
    id: id,
    method: "tools/call",
    params: { name: name, arguments: args || {} },
  });
}

function textOf(response) {
  return JSON.parse(response.result.content[0].text);
}

const init = round(
  JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: { protocolVersion: "2025-06-18", capabilities: {} },
  }),
);
assertEqual(init.result.protocolVersion, "2025-06-18", "initialize echoes a known version");
assertEqual(init.result.serverInfo.name, "atmos", "initialize names the atmos server");

const fallback = round(
  JSON.stringify({
    jsonrpc: "2.0",
    id: "abc",
    method: "initialize",
    params: { protocolVersion: "1999-01-01" },
  }),
);
assertEqual(fallback.id, "abc", "initialize keeps a string id");
assertEqual(fallback.result.protocolVersion, "2024-11-05", "an unknown version falls back");

const listed = round(JSON.stringify({ jsonrpc: "2.0", id: 2, method: "tools/list" }));
const names = listed.result.tools.map(function (tool) {
  return tool.name;
});
assertEqual(
  names.join(","),
  "list,snapshot,get,set,displays,sample,diagnostics",
  "tools/list names the seven tools",
);

let snapshotBody = null;
const snapshot = round(
  callMessage(3, "snapshot", { group: "look", keys: ["theme"] }),
  function (body, done) {
    snapshotBody = body;
    done({ ok: true, result: { theme: "x" } });
  },
);
assertEqual(snapshotBody.op, "settings.snapshot", "snapshot encodes settings.snapshot");
assertEqual(snapshotBody.group, "look", "snapshot keeps the group");
assertEqual(snapshotBody.keys[0], "theme", "snapshot keeps the keys");
assertEqual(textOf(snapshot).theme, "x", "snapshot returns the serve result");

let trustCalls = 0;
const trust = round(callMessage(4, "set", { domain: "sshdEnabled", value: true }), function () {
  trustCalls += 1;
});
assertEqual(trustCalls, 0, "a trust set does not call serve");
assert(trust.result.isError === true, "a trust set is a tool error");
assert(
  textOf(trust).error.message.indexOf("sshdEnabled is refused") !== -1,
  "a trust set says the domain is refused",
);

const bodies = [];
const set = round(callMessage(5, "set", { domain: "theme", value: "next" }), function (body, done) {
  bodies.push(body.op);
  if (body.op === "settings.get") done({ ok: true, result: "old" });
  else
    done({
      ok: true,
      result: { domain: "theme", value: "next", file: "theme", encoding: "map" },
    });
});
assertEqual(bodies.join(","), "settings.get,settings.set", "a set reads before it writes");
assertEqual(textOf(set).previous, "old", "a set returns the previous value");
assertEqual(textOf(set).value, "next", "a set returns the value serve read back");

const failed = round(callMessage(6, "get", { domain: "theme" }), function (body, done) {
  done({ ok: false, error: { code: "denied", message: "no" } });
});
assert(failed.result.isError === true, "a failed envelope sets isError");
assertEqual(textOf(failed).error.message, "no", "a failed envelope keeps the message");

const note = round(
  JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }),
  function () {},
);
assertEqual(note, null, "a notification returns null");

assertEqual(mcp.decodeMessage("nonsense").type, "invalid", "junk is invalid");

const tagged = round(callMessage(7, "list"), function (body, done) {
  done({
    ok: true,
    result: [
      { domain: "theme", type: "string" },
      { domain: "sshdEnabled", type: "bool" },
    ],
  });
});
assertEqual(textOf(tagged)[0].writable, true, "an ordinary domain is writable");
assertEqual(textOf(tagged)[1].writable, false, "a trust domain is tagged writable false");
