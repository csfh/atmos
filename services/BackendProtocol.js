// The line protocol spoken to `ratmos serve`: newline-delimited JSON. A
// request is { id, op, ...fields }. A reply is the usual envelope with the
// same id. An event is { event, result } pushed with no request. Pure so Node
// can test it without Quickshell.

function encodeRequest(id, body) {
  var out = {};
  var key;
  out.id = id;
  for (key in body) {
    if (Object.prototype.hasOwnProperty.call(body, key) && key !== "id") out[key] = body[key];
  }
  return JSON.stringify(out) + "\n";
}

// One line from the server, classified. Anything else is "invalid".
function decodeLine(line) {
  var text = String(line || "").replace(/^\s+|\s+$/g, "");
  if (!text) return { type: "invalid" };
  var value;
  try {
    value = JSON.parse(text);
  } catch (e) {
    return { type: "invalid" };
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) return { type: "invalid" };
  if (typeof value.event === "string") {
    return { type: "event", name: value.event, result: value.result };
  }
  if (value.id !== undefined && value.id !== null) {
    return { type: "reply", id: value.id, envelope: value };
  }
  return { type: "invalid" };
}

// The envelope a caller sees when the server cannot answer.
function unavailable(message) {
  return {
    ok: false,
    error: { code: "unavailable", message: String(message || "The backend is not running") },
    result: null,
  };
}

// Parse the whole stdout of a one-shot `ratmos request`.
function parseEnvelope(text, exitCode) {
  var raw = String(text || "").replace(/^\s+|\s+$/g, "");
  if (raw) {
    try {
      var value = JSON.parse(raw);
      if (value && typeof value === "object" && typeof value.ok === "boolean") return value;
    } catch (e) {
      // fall through
    }
  }
  return unavailable(exitCode === 0 ? "The backend sent no answer" : "The backend failed to run");
}

// How long to wait before restarting a server that stopped. Doubles from one
// second to a ceiling, so a backend that cannot start does not spin.
function restartDelay(failures) {
  var n = Math.max(1, Math.floor(Number(failures) || 1));
  return Math.min(30000, 1000 * Math.pow(2, Math.min(n - 1, 5)));
}

// A server that dies this soon after starting counts as a failed start.
var STABLE_MS = 5000;

// The watch.set body for what the frontend wants pushed.
function watchBody(spec) {
  var s = spec || {};
  return {
    op: "watch.set",
    paths: Array.isArray(s.paths) ? s.paths.map(String) : [],
    chrome: s.chrome === true,
    accounts: s.accounts
      ? { user: String(s.accounts.user || ""), home: String(s.accounts.home || "") }
      : null,
  };
}

// Several parts of the app each watch something. The server gets one
// subscription, so merge theirs: every path once, chrome if anyone wants it,
// and the first accounts request.
function mergeWatch(specs) {
  var merged = { paths: [], chrome: false, accounts: null };
  var seen = {};
  var owner;
  for (owner in specs || {}) {
    if (!Object.prototype.hasOwnProperty.call(specs, owner)) continue;
    var spec = specs[owner] || {};
    var paths = Array.isArray(spec.paths) ? spec.paths : [];
    for (var i = 0; i < paths.length; i++) {
      var path = String(paths[i]);
      if (!path || seen[path]) continue;
      seen[path] = true;
      merged.paths.push(path);
    }
    if (spec.chrome === true) merged.chrome = true;
    if (spec.accounts && !merged.accounts) merged.accounts = spec.accounts;
  }
  return merged;
}
