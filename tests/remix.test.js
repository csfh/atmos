const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const remix = load("services/RemixIcons.js");
assertEqual(remix.viewBoxSize(), 24, "remix viewBox is 24");
assertEqual(remix.pathFor(""), "", "pathFor misses an empty name");
assertEqual(remix.pathFor("nope"), "", "pathFor misses an unknown name");
const remixDir = path.join(__dirname, "..", "icons");
const remixFiles = fs.readdirSync(remixDir).filter(function (name) {
  return name.endsWith(".svg");
});
assert(remixFiles.length >= 24, "remix icons include hub glyphs");
remixFiles.forEach(function (file) {
  const name = file.replace(/\.svg$/, "");
  const svg = fs.readFileSync(path.join(remixDir, file), "utf8");
  assertEqual(remix.pathFor(name), remix.pathFromSvg(svg), "pathFor matches icons/" + file);
});
