const fs = require("node:fs");
const path = require("node:path");
const { assert, assertEqual } = require("./harness");

const catalog = JSON.parse(fs.readFileSync(path.join(__dirname, "../i18n.json"), "utf8"));
const sourceKeys = Object.keys(catalog.en).sort();
assert(sourceKeys.length > 0, "English source catalog is not empty");
assert(!sourceKeys.includes(""), "empty UI strings do not become catalog entries");
assert(catalog.es && catalog.pt, "Spanish and Portuguese catalogs are shipped");
for (const locale of [
  "es",
  "pt",
  "fr",
  "it",
  "de",
  "nl",
  "pl",
  "uk",
  "ru",
  "tr",
  "ja",
  "ko",
  "zh",
  "ar",
]) {
  assert(catalog[locale], `${locale} catalog is shipped`);
}

function placeholders(text) {
  const counts = {};
  for (const match of text.matchAll(/\{([A-Za-z_][A-Za-z0-9_]*)\}/g)) {
    counts[match[1]] = (counts[match[1]] || 0) + 1;
  }
  return JSON.stringify(Object.entries(counts).sort());
}

for (const [locale, entries] of Object.entries(catalog)) {
  assertEqual(
    JSON.stringify(Object.keys(entries).sort()),
    JSON.stringify(sourceKeys),
    `${locale} has complete source-key coverage`,
  );
  for (const source of sourceKeys) {
    const translated = entries[source];
    assert(typeof translated === "string" && translated.trim(), `${locale}: nonempty ${source}`);
    assertEqual(
      placeholders(translated),
      placeholders(source),
      `${locale}: placeholders ${source}`,
    );
    if (locale === "en") assertEqual(translated, source, "English fallback preserves source text");
  }
}
