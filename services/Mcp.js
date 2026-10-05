// Stdio MCP framing for `atmos mcp`. This file does not talk to ratmos.
// The server in McpServer.js turns a tool call into one or two serve
// requests and writes only JSON-RPC lines to stdout. Node tests load this
// file the same way they load Requests.js.

var SERVER_NAME = "atmos";
var SERVER_VERSION = "0.1.0";
var DEFAULT_PROTOCOL = "2024-11-05";
var KNOWN_PROTOCOLS = ["2024-11-05", "2025-03-26", "2025-06-18"];

var TRUST = {
  sshdEnabled: true,
  passwordlessSudo: true,
  sudolessDocker: true,
  fingerprintConfigured: true,
  fido2Configured: true,
  directBoot: true,
};

function tools() {
  return [
    tool(
      "list",
      "List every settings domain, its type, and whether Atmos tools may change it. Trust domains are readable and tagged writable false.",
      { type: "object", properties: {} },
    ),
    tool(
      "snapshot",
      "Read the current settings. group is look, rest, all, network, disks, accounts, or system. Omit group for all. keys limits the top-level names.",
      {
        type: "object",
        properties: {
          group: { type: "string" },
          keys: { type: "array", items: { type: "string" } },
        },
      },
    ),
    tool("get", "Read one settings domain.", {
      type: "object",
      properties: { domain: { type: "string" } },
      required: ["domain"],
    }),
    tool(
      "set",
      "Change one settings domain through the same writers the Atmos window uses. Trust domains (sshdEnabled, passwordlessSudo, sudolessDocker, fingerprintConfigured, fido2Configured, directBoot) are refused. The result includes the previous value.",
      {
        type: "object",
        properties: {
          domain: { type: "string" },
          value: {},
        },
        required: ["domain", "value"],
      },
    ),
    tool("displays", "Read the connected monitors.", { type: "object", properties: {} }),
    tool("sample", "Read one live sample of CPU, memory, network, temperatures, and processes.", {
      type: "object",
      properties: {},
    }),
    tool("diagnostics", "Read the diagnostics inventory.", { type: "object", properties: {} }),
  ];
}

function tool(name, description, inputSchema) {
  return { name: name, description: description, inputSchema: inputSchema };
}

function decodeMessage(line) {
  var text = String(line || "").replace(/^\s+|\s+$/g, "");
  if (!text) return { type: "empty" };
  var value;
  try {
    value = JSON.parse(text);
  } catch (e) {
    return { type: "invalid" };
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) return { type: "invalid" };
  return { type: "message", value: value };
}

// call(body, done) performs one serve request. done(response) runs once.
// A notification passes null. set is the only tool that calls serve twice.
function handleMessage(msg, call, done) {
  if (!msg || msg.type === "empty") {
    done(null);
    return;
  }
  if (!msg || msg.type !== "message") {
    done(rpcError(null, -32700, "Parse error"));
    return;
  }
  var value = msg.value;
  var method = String(value.method || "");
  var id = value.id;
  if (!method) {
    done(rpcError(id === undefined ? null : id, -32600, "Invalid request"));
    return;
  }
  if (method.indexOf("notifications/") === 0 || id === undefined) {
    done(null);
    return;
  }
  if (method === "initialize") {
    var requested = value.params && value.params.protocolVersion;
    done(
      rpcResult(id, {
        protocolVersion: protocolVersion(requested),
        capabilities: { tools: {} },
        serverInfo: { name: SERVER_NAME, version: SERVER_VERSION },
      }),
    );
    return;
  }
  if (method === "ping") {
    done(rpcResult(id, {}));
    return;
  }
  if (method === "tools/list") {
    done(rpcResult(id, { tools: tools() }));
    return;
  }
  if (method === "tools/call") {
    var params = value.params || {};
    var name = String(params.name || "");
    var args = params.arguments && typeof params.arguments === "object" ? params.arguments : {};
    dispatchTool(id, name, args, call, done);
    return;
  }
  done(rpcError(id, -32601, "Method not found"));
}

function protocolVersion(requested) {
  var version = String(requested || "");
  if (KNOWN_PROTOCOLS.indexOf(version) !== -1) return version;
  return DEFAULT_PROTOCOL;
}

function dispatchTool(id, name, args, call, done) {
  var plan = toolCall(name, args);
  if (plan.unknown) {
    done(rpcResult(id, toResult(refused(name + " is not a tool"))));
    return;
  }
  if (plan.refused) {
    done(rpcResult(id, toResult(refused(plan.refused))));
    return;
  }
  if (plan.set) {
    call(plan.get, function (got) {
      var previous = got && got.ok === true ? got.result : null;
      call(plan.set, function (env) {
        done(rpcResult(id, toResult(env, previous)));
      });
    });
    return;
  }
  call(plan.body, function (env) {
    var presented = name === "list" ? tagWritable(env) : env;
    done(rpcResult(id, toResult(presented)));
  });
}

// A serve body, a refused set, or the get-then-set pair.
function toolCall(name, args) {
  args = args || {};
  if (name === "list") return { body: { op: "settings.list" } };
  if (name === "snapshot") {
    var body = { op: "settings.snapshot", group: String(args.group || "all") };
    if (Array.isArray(args.keys)) body.keys = args.keys.map(String);
    return { body: body };
  }
  if (name === "get") {
    if (!args.domain) return { refused: "domain is required" };
    return { body: { op: "settings.get", domain: String(args.domain) } };
  }
  if (name === "set") {
    if (!args.domain) return { refused: "domain is required" };
    if (!Object.prototype.hasOwnProperty.call(args, "value"))
      return { refused: "value is required" };
    var domain = String(args.domain);
    if (TRUST[domain]) return { refused: domain + " is refused" };
    return {
      get: { op: "settings.get", domain: domain },
      set: { op: "settings.set", domain: domain, value: args.value },
    };
  }
  if (name === "displays") return { body: { op: "display.snapshot" } };
  if (name === "sample") return { body: { op: "display.get", kind: "live" } };
  if (name === "diagnostics") return { body: { op: "display.get", kind: "diagnostics" } };
  return { unknown: true };
}

function tagWritable(envelope) {
  if (!envelope || envelope.ok !== true || !Array.isArray(envelope.result)) return envelope;
  var result = [];
  var i, row, copy;
  for (i = 0; i < envelope.result.length; i++) {
    row = envelope.result[i] || {};
    copy = {};
    for (var key in row) {
      if (Object.prototype.hasOwnProperty.call(row, key)) copy[key] = row[key];
    }
    copy.writable = TRUST[String(row.domain || "")] !== true;
    result.push(copy);
  }
  return { ok: true, result: result };
}

function toResult(envelope, previous) {
  var ok = !!(envelope && envelope.ok === true);
  var payload;
  if (ok) {
    payload = envelope.result;
    if (previous !== undefined) {
      if (payload && typeof payload === "object" && !Array.isArray(payload)) {
        payload = Object.assign({ previous: previous }, payload);
      } else {
        payload = { previous: previous, result: payload };
      }
    }
  } else {
    var err = envelope && envelope.error ? envelope.error : { message: "The backend failed" };
    payload = { error: err };
  }
  var out = { content: [{ type: "text", text: JSON.stringify(payload) }] };
  if (!ok) out.isError = true;
  return out;
}

function refused(message) {
  return { ok: false, error: { code: "denied", message: String(message || "") } };
}

function rpcResult(id, result) {
  return { jsonrpc: "2.0", id: id, result: result };
}

function rpcError(id, code, message) {
  return { jsonrpc: "2.0", id: id, error: { code: code, message: message } };
}
