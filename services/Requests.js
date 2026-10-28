// Every request the frontend can send to atmosd, one builder each. Callers
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
  "snapshot.keys",
  "snapshot.topic",
  "snapshot.invalidate",
  "write",
  "display.brightness",
  "display.scale",
  "search.query",
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
  "host.image",
  "wifi.scan",
  "bluetooth.scan",
  "agents.mcp.list",
  "agents.mcp.set",
  "agents.mcp.check",
];

function settingsSnapshot(group, keys) {
  var body = { op: "settings.snapshot", group: String(group || "all") };
  if (Array.isArray(keys)) body.keys = keys.map(String);
  return body;
}

// readback: snapshot keys the setting changes besides its own, read after the write.
function settingsSet(domain, value, readback) {
  var body = { op: "settings.set", domain: String(domain), value: value };
  if (Array.isArray(readback) && readback.length > 0) body.readback = readback.map(String);
  return body;
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

// What the machine reports now for some snapshot keys, and nothing else.
// expect: { key: value } the keys must show before they are returned.
function snapshotKeys(keys, expect) {
  var body = { op: "snapshot.keys", keys: Array.isArray(keys) ? keys.map(String) : [] };
  if (expect && typeof expect === "object") body.expect = expect;
  return body;
}

// One settings topic read from the machine now (docs/adr/0011). The daemon
// asks for this itself to fill a topic; a frontend holds the topics instead.
function snapshotTopic(topic) {
  return { op: "snapshot.topic", topic: String(topic) };
}

// Ask the daemon to read settings topics again (what a Refresh button sends).
// None named means every topic.
function snapshotInvalidate(topics) {
  return {
    op: "snapshot.invalidate",
    topics: Array.isArray(topics) ? topics.map(String) : [],
  };
}

// A typed write by name (docs/protocol.md, Writes). The backend checks the
// arguments and builds the command; the reply's snapshot is what to show.
function write(name, args) {
  return { op: "write", name: String(name), args: args && typeof args === "object" ? args : {} };
}

// A job or an interactive tool by name (docs/protocol.md, Jobs). Backend.startJob
// sends it as job.start; the daemon runs it and streams what it prints. A
// password or a plan is an argument and reaches the job as its stdin.
function job(name, args) {
  return { op: "job", name: String(name), args: args && typeof args === "object" ? args : {} };
}

// A short readable name for a typed op, for the history and the Preview list:
// the name and its plain arguments. Secrets and long text are left out, and
// the window never builds a command line.
var HIDDEN = ["password", "current", "new", "header", "plan", "json", "payload", "text", "stdin"];

function label(op) {
  var body = op && typeof op === "object" ? op : {};
  var parts = [String(body.name || body.op || "")];
  var args = body.args && typeof body.args === "object" ? body.args : body;
  var key;
  for (key in args) {
    if (!Object.prototype.hasOwnProperty.call(args, key)) continue;
    if (HIDDEN.indexOf(key) !== -1) continue;
    if (args === body && (key === "op" || key === "name" || key === "args")) continue;
    var value = args[key];
    if (value === null || typeof value === "object") continue;
    parts.push(String(value).slice(0, 60));
  }
  return parts;
}

// The focused monitor's scale, run by the daemon like the brightness write
// below; the reply carries result.snapshot the same way.
function displayScale(scale) {
  return { op: "display.scale", scale: String(scale) };
}

// A write the daemon runs on the monitor's own lane. The reply carries the
// monitor list as the machine now reports it, in result.snapshot.
function displayBrightness(monitor, percent) {
  return {
    op: "display.brightness",
    monitor: String(monitor || ""),
    percent: Math.round(Number(percent)),
  };
}

// context is Hubs.searchContext(): the hubs, page files, folder rules and titles.
function searchQuery(query, context) {
  var ctx = context && typeof context === "object" ? context : {};
  return {
    op: "search.query",
    query: String(query || ""),
    hubs: Array.isArray(ctx.hubs) ? ctx.hubs : [],
    files: ctx.files && typeof ctx.files === "object" ? ctx.files : {},
    prefixes: Array.isArray(ctx.prefixes) ? ctx.prefixes : [],
    titles: ctx.titles && typeof ctx.titles === "object" ? ctx.titles : {},
  };
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

function hostImage(which) {
  return { op: "host.image", which: String(which) };
}

function wifiScan() {
  return { op: "wifi.scan" };
}

function bluetoothScan(seconds) {
  var body = { op: "bluetooth.scan" };
  if (seconds) body.seconds = Number(seconds);
  return body;
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
