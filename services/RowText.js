// List-row text the Hypr writers and the Omafile check both accept.
// Included, not copied, so a chord, command, or window rule changes in one place.

function sanitizeKeys(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "").replace(/\s+/g, " ");
  if (!text || text.length > 64) return "";
  if (!/^[A-Za-z0-9_ +.:,-]+$/.test(text)) return "";
  return text;
}

function sanitizeLabel(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 64) return "";
  return text;
}

function sanitizeCommand(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 256) return "";
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  return text;
}

function sanitizeMatch(raw) {
  var text = String(raw || "");
  if (text.indexOf("\n") !== -1 || text.indexOf("\r") !== -1) return "";
  text = text.replace(/^\s+|\s+$/g, "");
  if (!text || text.length > 128) return "";
  if (text.indexOf("]]") !== -1) return "";
  return text;
}

function sanitizeWorkspace(raw) {
  var text = String(raw || "").replace(/^\s+|\s+$/g, "");
  if (!text) return "";
  if (!/^[A-Za-z0-9:_-]{1,32}$/.test(text)) return "";
  return text;
}

function clampSize(raw) {
  var n = Math.round(Number(raw));
  if (!isFinite(n) || n < 100 || n > 4000) return 0;
  return n;
}
