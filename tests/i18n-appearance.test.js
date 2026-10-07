const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { assert, assertEqual, load } = require("./harness");

const i18n = load("services/I18n.js");
const catalog = JSON.parse(fs.readFileSync(path.join(__dirname, "../i18n.json"), "utf8"));
const page = fs.readFileSync(path.join(__dirname, "../pages/AppearancePage.qml"), "utf8");
const sources = [...page.matchAll(/I18n\.tr\(\s*("(?:\\.|[^"\\])*")/g)].map((match) =>
  JSON.parse(match[1]),
);
for (const source of sources) {
  for (const [locale, entries] of Object.entries(catalog)) {
    assert(
      typeof entries[source] === "string" && entries[source].trim(),
      `${locale}: Appearance source is cataloged: ${source}`,
    );
  }
}

// Execute the production QML helper body, not a duplicated count formatter.
const countBody = page.match(/function extraCountText\(\) \{([\s\S]*?)\n  \}/);
assert(countBody, "extra-theme count helper exists");
for (const count of [0, 1, 3]) {
  const actual = vm.runInNewContext(`(function () {${countBody[1]} })()`, {
    Omarchy: { extraThemes: Array(count).fill("my-theme") },
    I18n: { tr: (source, args) => i18n.translate(catalog, "es-CL", source, args) },
  });
  assertEqual(
    actual,
    `Temas adicionales instalados en este equipo: ${count}.`,
    "dynamic theme count is localized",
  );
}
assertEqual(
  i18n.translate(catalog, "es", "Current file: {file}.", { file: "my-Ö-wallpaper.png" }),
  "Archivo actual: my-Ö-wallpaper.png.",
  "runtime filenames are preserved while their surrounding prose is translated",
);
assert(page.includes('label: "Installed themes"'), "stable setting identity remains English");
assert(page.includes('hint: "omarchy theme install"'), "commands remain unchanged");
assert(
  !page.includes('"Current file: " +'),
  "filenames use a named placeholder rather than assembled source keys",
);
