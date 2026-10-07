"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const Search = require("../services/SearchIndex");

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-search-i18n-"));
try {
  const catalog = {
    es: {
      Appearance: "Apariencia",
      "Change how the desktop looks.": "Cambiar el aspecto del escritorio.",
      "Font size": "Tamaño de fuente",
      "Choose a font size.": "Elige un tamaño de fuente.",
    },
  };
  const catalogFile = path.join(tmp, "i18n.json");
  fs.writeFileSync(catalogFile, JSON.stringify(catalog));
  const source =
    'SettingRow { label: I18n.tr("Appearance"); description: I18n.tr("Change how the desktop looks.") }\nPrefsRow { label: I18n.tr("Font size"); description: I18n.tr("Choose a font size.") }';
  const qml = path.join(tmp, "page.qml");
  fs.writeFileSync(qml, source);
  const rows = Search.rowsFromQml(source, "appearance");
  assert.equal(rows.length, 2, "extract wrapped QML translations");
  const db = Search.openIndex(":memory:");
  Search.ingestRows(db, Search.localizedRows(rows, "es", catalog));
  assert.equal(Search.queryRows(db, "Apariencia").length, 1, "translated query matches");
  assert.equal(Search.queryRows(db, "Appearance").length, 1, "English alias matches");
  assert.equal(Search.queryRows(db, "Tamaño de fuente").length, 1, "translated details searchable");
  assert.equal(
    Search.queryRows(db, "Apariencia")[0].label,
    "Appearance",
    "source label stays stable",
  );
  db.close();
  assert.notEqual(
    Search.indexPath({ XDG_CACHE_HOME: tmp, ATMOS_LANG: "es_MX.UTF-8" }),
    Search.indexPath({ XDG_CACHE_HOME: tmp, ATMOS_LANG: "fr" }),
  );
  assert.equal(
    Search.indexPath({ ATMOS_SEARCH_INDEX: path.join(tmp, "custom.sqlite"), ATMOS_LANG: "es" }),
    path.join(tmp, "custom.sqlite"),
  );
  assert.equal(Search.localeFrom({ LANGUAGE: "es:en" }), "es");
  assert.equal(
    Search.localizedRows(rows, "es", {}).length,
    rows.length,
    "malformed catalog falls back to source",
  );
  console.log(
    "ok - translated search indexing, aliases, stable labels, locale cache and QML extraction",
  );
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}
