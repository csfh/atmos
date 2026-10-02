// What a failed ratmos call means. The backend answers every request with a
// JSON envelope; on failure it is { ok: false, error: { code, message,
// context } }. Older callers and scripts still print plain text to stderr, so
// this module turns either into one string for the error banner. Pure so
// Node can test it.

function trim(text) {
  return String(text || "").replace(/^\s+|\s+$/g, "");
}

// One line for a person. Accepts the structured error or a bare string.
function errorText(error) {
  if (error === null || typeof error === "undefined") return "";
  if (typeof error === "string") return trim(error);
  if (typeof error !== "object") return trim(String(error));
  var message = trim(error.message);
  var context = trim(error.context);
  if (!message) return context;
  if (context && message.indexOf(context) === -1) return context + ": " + message;
  return message;
}

// The message inside a failed envelope, or "" when the text is not one.
function envelopeFailure(raw) {
  var text = trim(raw);
  if (text.charAt(0) !== "{") return "";
  var value;
  try {
    value = JSON.parse(text);
  } catch (e) {
    return "";
  }
  if (!value || typeof value !== "object" || value.ok !== false) return "";
  return errorText(value.error) || "Command failed";
}

// stderr and stdout of a command that exited non-zero, as banner text.
// A failed envelope on stdout wins over the raw JSON.
function commandText(err, out) {
  var e = trim(err);
  var o = envelopeFailure(out) || trim(out);
  if (e && o && o !== e) return e + "\n" + o;
  return e || o;
}

// `ratmos apply` ends with one summary line on stdout:
//   @@ratmos-apply@@ {"ok":false,"code":3,"message":"...","warnings":[...]}
// `message` is the stderr that is not known noise, so there is no guessing
// here about which lines matter.
var APPLY_MARKER = "@@ratmos-apply@@ ";

// The summary a line carries, or null if it is any other line.
function applyLine(line) {
  var text = String(line === undefined || line === null ? "" : line);
  if (text.indexOf(APPLY_MARKER) !== 0) return null;
  try {
    var value = JSON.parse(text.slice(APPLY_MARKER.length));
    return value && typeof value === "object" ? value : null;
  } catch (e) {
    return null;
  }
}

// Whole stdout of an apply: the text without the summary line, and the summary.
function splitApply(stdout) {
  var lines = String(stdout || "").split("\n");
  var kept = [];
  var result = null;
  for (var i = 0; i < lines.length; i++) {
    var parsed = applyLine(lines[i]);
    if (parsed) result = parsed;
    else kept.push(lines[i]);
  }
  return { text: kept.join("\n"), result: result };
}

// What to put in the error banner for an apply that ended. "" means no banner:
// it succeeded, or it exited non-zero having printed only known noise.
// `fallback` is the plain stderr/stdout text, used when the backend sent no
// summary (an older ratmos, or one that was killed).
function applyBanner(result, exitCode, fallback) {
  if (result) {
    if (result.ok === true) return "";
    var message = trim(result.message);
    if (message) return message;
    var noise = Array.isArray(result.warnings) ? result.warnings.length : 0;
    return noise > 0 ? "" : "Command failed";
  }
  if (exitCode === 0) return "";
  return trim(fallback) || "Command failed";
}
