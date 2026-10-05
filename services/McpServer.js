"use strict";

// `atmos mcp`. Speaks newline-delimited JSON-RPC on stdin and stdout, and
// holds one `ratmos serve` whose stdout stays on a private pipe.

const fs = require("fs");
const path = require("path");
const readline = require("readline");
const vm = require("vm");
const { spawn } = require("child_process");

function load(name) {
  const src = fs
    .readFileSync(path.join(__dirname, name), "utf8")
    .replace(/^\.pragma library\s*$/m, "");
  const ctx = {};
  vm.runInNewContext(src, ctx, { filename: name });
  return ctx;
}

const mcp = load("Mcp.js");
const proto = load("BackendProtocol.js");

function serveArgs(argv) {
  const args = [];
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--backend" || arg === "--root" || arg === "--sampler") {
      const value = argv[i + 1];
      if (value) {
        args.push(arg, value);
        i += 1;
      }
    }
  }
  args.push("serve");
  return args;
}

function main() {
  const backend = process.env.ATMOS_BACKEND;
  if (!backend) {
    console.error("atmos mcp: ATMOS_BACKEND is not set");
    process.exit(1);
  }
  const child = spawn(backend, serveArgs(process.argv.slice(2)), {
    stdio: ["pipe", "pipe", "pipe"],
  });
  let clientOpen = true;
  let serveOpen = true;
  let nextId = 0;
  const pending = new Map();

  child.stderr.on("data", (buf) => {
    process.stderr.write(buf);
  });
  child.on("error", (err) => {
    console.error("atmos mcp: " + err.message);
    process.exit(1);
  });

  const serveLines = readline.createInterface({ input: child.stdout });
  serveLines.on("line", (line) => {
    const msg = proto.decodeLine(line);
    if (msg.type !== "reply") return;
    const waiter = pending.get(msg.id);
    if (!waiter) return;
    pending.delete(msg.id);
    waiter(msg.envelope);
  });

  function failPending(message) {
    for (const waiter of pending.values()) waiter(proto.unavailable(message));
    pending.clear();
  }

  function call(body, done) {
    if (!serveOpen) {
      done(proto.unavailable("The backend stopped"));
      return;
    }
    nextId += 1;
    const id = nextId;
    pending.set(id, done);
    child.stdin.write(proto.encodeRequest(id, body));
  }

  const input = readline.createInterface({ input: process.stdin });
  input.on("line", (line) => {
    mcp.handleMessage(mcp.decodeMessage(line), call, (response) => {
      if (response) process.stdout.write(JSON.stringify(response) + "\n");
    });
  });
  input.on("close", () => {
    clientOpen = false;
    if (serveOpen) child.stdin.end();
    else process.exit(0);
  });

  child.on("exit", () => {
    serveOpen = false;
    failPending("The backend stopped");
    if (clientOpen) process.exit(1);
    else process.exit(0);
  });
}

if (require.main === module) main();
