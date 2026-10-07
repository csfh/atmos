const { load, assert, assertEqual } = require("./harness");
const i18n = load("services/I18n.js");

const previousCatalog = { en: { Save: "Save" }, es: { Save: "Guardar" } };
assertEqual(
  i18n.catalogFromText('{"en":{"Save":"Save"},"es":{"Save":"Salvar"}}', previousCatalog).es.Save,
  "Salvar",
  "valid flat catalog replaces the previous catalog",
);
for (const text of [
  "{",
  "null",
  "[]",
  "{}",
  '{"es":{"Save":"Guardar"}}',
  '{"en":[]}',
  '{"en":{"Save":42}}',
  '{"en":{"Save":"Save"},"es":"wrong"}',
]) {
  assert(
    i18n.catalogFromText(text, previousCatalog) === previousCatalog,
    "invalid catalog retains the last valid object: " + text,
  );
}
assertEqual(
  i18n.translate(i18n.catalogFromText("{}", previousCatalog), "es", "Save"),
  "Guardar",
  "malformed-but-valid JSON cannot discard active translations",
);

assertEqual(
  i18n.effectiveLocale({ LANG: "en_US.UTF-8" }, "es_CL.UTF-8"),
  "es-CL",
  "system locale changes override a stale session LANG",
);
assertEqual(
  i18n.effectiveLocale({ ATMOS_LANG: "en", LANG: "en_US.UTF-8" }, "es_CL.UTF-8"),
  "en",
  "explicit application override still wins over system locale",
);

assertEqual(
  i18n.normalizeLocale("es_CL.UTF-8"),
  "es-CL",
  "normalize locale encoding and separator",
);
assertEqual(
  i18n.resolveLocale({ ATMOS_LANG: "fr_CA", LANGUAGE: "es:en" }),
  "fr-CA",
  "Atmos override wins",
);
assertEqual(
  i18n.resolveLocale({ LANGUAGE: "C:es_MX", LANG: "de_DE.UTF-8" }),
  "es-MX",
  "skip C and resolve LANGUAGE preference",
);
assertEqual(
  i18n.resolveLocale({ LC_ALL: "pt_BR", LANG: "en_US" }),
  "pt-BR",
  "locale environment precedence",
);
assertEqual(i18n.resolveLocale({ LANG: "C" }), "en", "safe English default");
assertEqual(
  i18n.localeCandidates("es-CL").join(","),
  "es-CL,es,en",
  "regional locale falls back to language then English",
);
assertEqual(
  i18n.translate({ es: { Save: "Guardar" } }, "es-CL", "Save"),
  "Guardar",
  "regional catalog falls back to base language",
);
assertEqual(i18n.translate({}, "fr", "Save"), "Save", "missing translation uses source English");
assertEqual(
  i18n.translate({ es: { "Hi {name}": "Hola {name}" } }, "es", "Hi {name}", { name: "Ana" }),
  "Hola Ana",
  "named placeholders interpolate",
);
assertEqual(
  i18n.translate({ es: { "Hi {name}": "Hola {name}" } }, "es", "Hi {name}", {}),
  "Hola {name}",
  "unknown placeholders remain intact",
);
assert(i18n.isRtl("ar-EG"), "Arabic is RTL");
assert(!i18n.isRtl("es-CL"), "Spanish is not RTL");
assertEqual(i18n.pluralCategory("en", 1), "one", "English singular category");
assertEqual(i18n.pluralCategory("en", 2), "other", "English plural category");
assertEqual(i18n.pluralCategory("ar", 2), "two", "Arabic dual category");
assertEqual(
  i18n.translatePlural({}, "en", { one: "{count} item", other: "{count} items" }, 3),
  "3 items",
  "plural API interpolates count",
);
