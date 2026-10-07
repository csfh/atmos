const fs = require("fs");

// Source-contract tests inspect English literals, but translated QML wraps the
// same source value in I18n.tr("..."). Unwrap only literal calls so tests keep
// asserting their original structure and runtime-specific expressions.
function read(file) {
  const source = fs.readFileSync(file, "utf8");
  if (!file.endsWith(".qml")) return source;
  return source.replace(/I18n\.tr\(("(?:\\.|[^"\\])*")\)/g, "$1");
}

module.exports = { read };
