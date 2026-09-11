// A record of what Atmos changed.
//
// Every mutation in Atmos goes through Omarchy.runCommand, which makes it
// the one honest place to record from. Recording per page or per control
// would mean trusting that every future control remembered to log, and the
// entries that got missed would be exactly the ones nobody thought about --
// which are the ones you most want when something breaks.
//
// Two features share this file because they are the same question asked at
// two different times. Preview asks "what would this do" before it happens;
// history asks "what did that do" after. Both need the command rendered as
// something a person can read.

var MAX_ENTRIES = 200;

// argv comes in wrapped for the shell more often than not, because Atmos
// detaches long-running writers. The wrapper is noise to a reader, so strip
// it and show the command that was actually meant.
function unwrap(argv) {
  var list = Array.isArray(argv) ? argv.slice() : [];
  if (list.length >= 3 && list[0] === "bash" && list[1] === "-c") {
    // bash -c SCRIPT NAME ARGS... The interesting part is ARGS, but only
    // when the script really is one of Atmos's own wrappers.
    var script = String(list[2] || "");
    // Match the wrapper shape, not the word. A plain substring test for
    // "exec" fires on any script that merely contains it -- "echo execute
    // me" was enough -- and then the command is rendered as whatever
    // happened to be in that argument slot.
    var isWrapper = /exec\s+"\$@"|exec\s+"\$1"|"\$@"\s*>/.test(script);
    if (!isWrapper) return list.slice(2);
    // A wrapper that shifts has consumed $1, so the real command starts one
    // later. Without this the stub directory that shift exists to remove is
    // exactly what gets displayed.
    var start = /(^|;|\s)shift(\s|;|$)/.test(script) ? 5 : 4;
    var rest = list.slice(start);
    return rest.length > 0 ? rest : list.slice(2);
  }
  return list;
}

function describe(argv) {
  var list = unwrap(argv);
  var out = [];
  for (var i = 0; i < list.length; i++) {
    var part = String(list[i] === undefined || list[i] === null ? "" : list[i]);
    // A value with a space in it reads as two arguments unless it is quoted.
    if (part.indexOf(" ") !== -1) part = '"' + part + '"';
    out.push(part);
  }
  return out.join(" ");
}

// Where a command lands, when Atmos knows. Used by preview so the answer to
// "what will this touch" is a path and not just a command line.
function targetFile(argv, home) {
  var text = describe(argv);
  var h = String(home || "");
  // The extension has to end the path. Unanchored, ".conf" matched inside
  // "~/.config/hypr" and reported the file as "~/.conf", and ".json" ate the
  // c off a .jsonc file.
  var m = text.match(/(~|\/home\/[^\s"]+)?\/[^\s"]*\.(lua|toml|json|conf|sh)(?![A-Za-z0-9])/);
  if (!m) return "";
  var path = m[0];
  // Boundary, not prefix. "/home/pi" is a prefix of "/home/pieter", so a
  // plain prefix test rewrote another user's path into "~eter/...".
  if (h && (path === h || path.indexOf(h + "/") === 0)) return "~" + path.slice(h.length);
  return path;
}

function entry(argv, opts, now) {
  var o = opts || {};
  return {
    at: typeof now === "number" ? now : Date.now(),
    key: String(o.key || ""),
    text: describe(argv),
    file: String(o.file || ""),
    source: String(o.source || "you"),
    sudo: o.sudo === true,
  };
}

// Newest first, capped. A settings app does not need an unbounded audit log
// in memory, and an unbounded one is a slow leak in a process that runs for
// weeks.
function push(list, item) {
  var out = Array.isArray(list) ? list.slice() : [];
  out.unshift(item);
  if (out.length > MAX_ENTRIES) out = out.slice(0, MAX_ENTRIES);
  return out;
}

function relativeTime(at, now) {
  // Number(null) and Number("") are both 0, and 0 is finite, so an isFinite
  // guard alone reported a missing timestamp as 1970 -- "20707d ago".
  if (at === null || at === undefined || at === "") return "";
  var then = Number(at);
  var ref = typeof now === "number" ? now : Date.now();
  if (!isFinite(then) || then <= 0) return "";
  var secs = Math.max(0, Math.round((ref - then) / 1000));
  if (secs < 10) return "just now";
  if (secs < 60) return secs + "s ago";
  var mins = Math.round(secs / 60);
  if (mins < 60) return mins + "m ago";
  var hours = Math.round(mins / 60);
  if (hours < 24) return hours + "h ago";
  return Math.round(hours / 24) + "d ago";
}

// Group by source so "what did the agent do" is answerable at a glance,
// which is the question that matters once anything other than you can
// propose a change.
function countsBySource(list) {
  var out = {};
  var items = Array.isArray(list) ? list : [];
  for (var i = 0; i < items.length; i++) {
    var src = String(items[i] && items[i].source ? items[i].source : "you");
    out["s_" + src] = (out["s_" + src] || 0) + 1;
  }
  return out;
}

function countFor(counts, source) {
  var c = counts || {};
  return c["s_" + String(source)] || 0;
}
