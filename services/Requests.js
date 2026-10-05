// Every request the frontend can send to ratmos, one builder each. Callers
// build a body here and hand it to Backend.request, so an op name is spelled
// in one place. tests/requests.test.js checks this list against the ops the
// Rust backend accepts.

var OPS = [
  "version",
  "platform",
  "settings.list",
  "settings.get",
  "settings.set",
  "settings.snapshot",
  "display.get",
  "display.snapshot",
  "host.chrome",
  "host.themePack",
  "host.accounts",
  "host.stamp",
  "host.read",
  "host.write",
  "host.open",
  "speedtest.disk",
  "speedtest.net",
  "unit.output",
  "agents.mcp.list",
  "agents.mcp.set",
  "agents.mcp.check",
];

function settingsSnapshot(group, keys) {
  var body = { op: "settings.snapshot", group: String(group || "all") };
  if (Array.isArray(keys)) body.keys = keys.map(String);
  return body;
}

function settingsSet(domain, value) {
  return { op: "settings.set", domain: String(domain), value: value };
}

function settingsGet(domain) {
  return { op: "settings.get", domain: String(domain) };
}

function displayGet(kind) {
  return { op: "display.get", kind: String(kind) };
}

function displaySnapshot() {
  return { op: "display.snapshot" };
}

function hostChrome() {
  return { op: "host.chrome" };
}

function hostThemePack(name, home, part) {
  var body = { op: "host.themePack", name: String(name || ""), home: String(home || "") };
  if (part) body.part = String(part);
  return body;
}

function hostAccounts(user, home) {
  return { op: "host.accounts", user: String(user || ""), home: String(home || "") };
}

function hostStamp(paths) {
  return { op: "host.stamp", paths: (paths || []).map(String) };
}

function hostRead(paths) {
  return { op: "host.read", paths: (paths || []).map(String) };
}

function hostWrite(path, text) {
  return { op: "host.write", path: String(path), text: String(text || "") };
}

function hostOpen(path) {
  return { op: "host.open", path: String(path) };
}

function speedtestDisk(dir) {
  var body = { op: "speedtest.disk" };
  if (dir) body.dir = String(dir);
  return body;
}

function speedtestNet(phase) {
  return { op: "speedtest.net", phase: String(phase || "down") };
}

function unitOutput(kind, scope, unit) {
  return {
    op: "unit.output",
    kind: String(kind),
    scope: String(scope || "system"),
    unit: String(unit),
  };
}

function agentsMcpList() {
  return { op: "agents.mcp.list" };
}

function agentsMcpSet(agent, on, replace) {
  var body = { op: "agents.mcp.set", agent: String(agent || ""), on: on === true };
  if (replace === true) body.replace = true;
  return body;
}

function agentsMcpCheck() {
  return { op: "agents.mcp.check" };
}
