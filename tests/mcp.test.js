// `ratmos mcp` end to end: JSON-RPC lines in, JSON-RPC lines out, on the plain
// backend under a fixture root so nothing touches the machine running the test.
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { assert, assertEqual } = require("./harness");
const { ratmosBinary } = require("./sandbox");

const root = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-mcp-"));
const lines = [
  { jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-06-18" } },
  { jsonrpc: "2.0", method: "notifications/initialized" },
  { jsonrpc: "2.0", id: 2, method: "tools/list" },
  { jsonrpc: "2.0", id: 3, method: "tools/call", params: { name: "list", arguments: {} } },
  {
    jsonrpc: "2.0",
    id: 4,
    method: "tools/call",
    params: { name: "set", arguments: { domain: "theme", value: "tokyo-night" } },
  },
  {
    jsonrpc: "2.0",
    id: 5,
    method: "tools/call",
    params: { name: "set", arguments: { domain: "passwordlessSudo", value: true } },
  },
  {
    jsonrpc: "2.0",
    id: 6,
    method: "tools/call",
    params: { name: "get", arguments: { domain: "theme" } },
  },
];

try {
  const run = spawnSync(ratmosBinary(), ["--backend", "plain", "--root", root, "mcp"], {
    encoding: "utf8",
    input: lines.map((l) => JSON.stringify(l)).join("\n") + "\n",
    timeout: 20000,
  });
  assertEqual(run.status, 0, "ratmos mcp exits when its input closes");
  const replies = run.stdout
    .split("\n")
    .filter(Boolean)
    .map((l) => JSON.parse(l));
  const byId = (id) => replies.find((r) => r.id === id);
  assertEqual(replies.length, 6, "a notification gets no reply");
  assertEqual(byId(1).result.protocolVersion, "2025-06-18", "initialize negotiates the version");
  assertEqual(byId(2).result.tools.length, 7, "seven tools are listed");
  const payload = (id) => JSON.parse(byId(id).result.content[0].text);
  const list = payload(3);
  assert(
    list.some((row) => row.domain === "theme" && row.writable === true),
    "list tags theme writable",
  );
  assert(
    list.some((row) => row.domain === "passwordlessSudo" && row.writable === false),
    "list tags a trust domain read-only",
  );
  const set = payload(4);
  assertEqual(set.value, "tokyo-night", "set writes through the real writer");
  assert("previous" in set, "set reports the previous value");
  assertEqual(byId(5).result.isError, true, "a trust domain is refused");
  assertEqual(payload(6), "tokyo-night", "get reads the value back");
} finally {
  fs.rmSync(root, { recursive: true, force: true });
}
