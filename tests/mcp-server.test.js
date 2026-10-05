const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawn } = require("child_process");
const { assert, assertEqual } = require("./harness");

const root = path.join(__dirname, "..");
const stub = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-mcp-serve-"));
const fake = path.join(stub, "ratmos");
fs.writeFileSync(
  fake,
  `#!/usr/bin/env node
const readline = require("readline");
const rl = readline.createInterface({ input: process.stdin });
rl.on("line", (line) => {
  const msg = JSON.parse(line);
  process.stdout.write(JSON.stringify({ id: msg.id, ok: true, result: { theme: "x" } }) + "\\n");
});
`,
  { mode: 0o755 },
);

const proc = spawn(process.execPath, [path.join(root, "services/McpServer.js")], {
  env: Object.assign({}, process.env, { ATMOS_BACKEND: fake }),
});
let stdout = "";
let stderr = "";
proc.stdout.on("data", (buf) => {
  stdout += buf.toString();
});
proc.stderr.on("data", (buf) => {
  stderr += buf.toString();
});

const timer = setTimeout(() => {
  proc.kill();
  console.error("not ok - atmos mcp timed out\n" + stderr);
  process.exit(1);
}, 5000);

proc.stdin.write(
  JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2024-11-05",
      capabilities: {},
      clientInfo: { name: "t", version: "0" },
    },
  }) + "\n",
);
proc.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) + "\n");
proc.stdin.write(
  JSON.stringify({
    jsonrpc: "2.0",
    id: 2,
    method: "tools/call",
    params: { name: "snapshot", arguments: { group: "look" } },
  }) + "\n",
);

function finish() {
  clearTimeout(timer);
  const lines = stdout
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
  assertEqual(lines.length, 2, "stdout is the initialize reply and the tool result");
  const init = JSON.parse(lines[0]);
  const call = JSON.parse(lines[1]);
  assertEqual(init.id, 1, "the initialize reply keeps its id");
  assertEqual(init.result.serverInfo.name, "atmos", "the server calls itself atmos");
  assertEqual(call.id, 2, "the tool reply keeps its id");
  const payload = JSON.parse(call.result.content[0].text);
  assertEqual(payload.theme, "x", "snapshot returns the fake serve result");
  assert(stdout.indexOf("settings.snapshot") === -1, "the serve line stays off stdout");
  fs.rmSync(stub, { recursive: true, force: true });
}

proc.stdout.on("data", () => {
  const lines = stdout.split("\n").filter((line) => line.trim());
  if (lines.length >= 2) proc.stdin.end();
});
proc.on("exit", (code) => {
  assertEqual(code, 0, "the tunnel exits when the agent closes stdin");
  finish();
});
