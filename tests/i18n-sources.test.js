const fs = require("node:fs");
const path = require("node:path");
const { assert } = require("./harness");

const root = path.join(__dirname, "..");
const catalog = JSON.parse(fs.readFileSync(path.join(root, "i18n.json"), "utf8"));
const auditedSources = JSON.parse(
  fs.readFileSync(path.join(__dirname, "i18n-ui-sources.json"), "utf8"),
);

function qmlFiles(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(directory, entry.name);
    return entry.isDirectory() ? qmlFiles(full) : entry.name.endsWith(".qml") ? [full] : [];
  });
}

const sources = new Map(auditedSources.map((source) => [source, "audited conditional UI copy"]));
for (const file of [
  path.join(root, "shell.qml"),
  ...qmlFiles(path.join(root, "pages")),
  ...qmlFiles(path.join(root, "components")),
]) {
  const text = fs.readFileSync(file, "utf8");
  for (const match of text.matchAll(/I18n\.tr\(\s*("(?:\\.|[^"\\])*")/g)) {
    const source = JSON.parse(match[1]);
    if (source) sources.set(source, path.relative(root, file));
  }
}

const missing = [];
for (const [source, origin] of sources) {
  for (const [locale, entries] of Object.entries(catalog)) {
    if (typeof entries[source] !== "string" || !entries[source].trim())
      missing.push(`${locale}: ${origin}: ${source}`);
  }
}
assert(
  missing.length === 0,
  "visible translation sources exist in every shipped locale",
  missing.join("\n"),
);
assert(!auditedSources.includes(""), "the audited inventory excludes empty strings");
assert(
  new Set(auditedSources).size === auditedSources.length,
  "the audited inventory is deduplicated",
);
