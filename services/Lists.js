// A list read back from a QML property can be a sequence wrapper: indexed, with a
// numeric length, but neither Array.isArray nor instanceof Array accept it.
function isArray(value) {
  if (Array.isArray(value) || value instanceof Array) return true;
  if (!value || typeof value !== "object") return false;
  var n = value.length;
  return typeof n === "number" && n >= 0 && Math.floor(n) === n && (n === 0 || n - 1 in value);
}

// A real Array copy of a list, or [] when it is not one.
function toList(value) {
  return isArray(value) ? Array.prototype.slice.call(value) : [];
}

// Data that crossed a QML signal or property comes back with its arrays wrapped, nested
// ones too. A JSON round trip makes them real arrays again.
function plain(value) {
  return value === undefined ? value : JSON.parse(JSON.stringify(value));
}
