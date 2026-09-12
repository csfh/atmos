// Caption for which config file a PrefsGroup section writes.
// Sentinel is the default: Atmos replaces `-- atmos:…` and leaves the rest
// of the user's file. File mode is whole-file ownership. Command mode is
// an omarchy/script write that is not a config path we edit in place.

var MODE_SENTINEL = "sentinel";
var MODE_FILE = "file";
var MODE_COMMAND = "command";

function trim(s) {
  return String(s || "").replace(/^\s+|\s+$/g, "");
}

function normalizeMode(mode) {
  var m = trim(mode);
  if (m === MODE_FILE || m === MODE_COMMAND) return m;
  return MODE_SENTINEL;
}

function ownsWholeFile(mode) {
  return normalizeMode(mode) === MODE_FILE;
}

function statusLabel(mode) {
  var m = normalizeMode(mode);
  if (m === MODE_FILE) return "Atmos file";
  if (m === MODE_COMMAND) return "via command";
  return "Atmos block";
}

function caption(file, mode, extras) {
  var path = trim(file);
  if (!path) return "";
  extras = extras || {};
  var note = trim(extras.note);
  var out = path + "  ·  " + statusLabel(mode);
  if (note) out += "  ·  " + note;
  return out;
}
