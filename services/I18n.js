/* .pragma library */

function normalizeLocale(value) {
  if (value === undefined || value === null) return "";
  var tag = String(value).trim().replace(/@.*$/, "").replace(/\..*$/, "").replace(/_/g, "-");
  if (!tag || /^(C|POSIX)$/i.test(tag) || !/^[A-Za-z]{2,3}(?:-[A-Za-z0-9]{2,8})*$/.test(tag))
    return "";
  var parts = tag.split("-");
  parts[0] = parts[0].toLowerCase();
  for (var i = 1; i < parts.length; i++) {
    parts[i] = parts[i].length === 2 ? parts[i].toUpperCase() : parts[i];
  }
  return parts.join("-");
}

function resolveLocale(environment) {
  var env = environment || {};
  var order = ["ATMOS_LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"];
  for (var i = 0; i < order.length; i++) {
    var raw = env[order[i]];
    if (!raw) continue;
    // LANGUAGE may contain a colon-separated gettext preference list.
    var candidates = String(raw).split(":");
    for (var j = 0; j < candidates.length; j++) {
      var locale = normalizeLocale(candidates[j]);
      if (locale) return locale;
    }
  }
  return "en";
}

function effectiveLocale(environment, systemLocale) {
  var env = environment || {};
  var explicit = {};
  var overrides = ["ATMOS_LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES"];
  for (var i = 0; i < overrides.length; i++) {
    if (env[overrides[i]]) explicit[overrides[i]] = env[overrides[i]];
  }
  if (Object.keys(explicit).length) return resolveLocale(explicit);
  return normalizeLocale(systemLocale) || resolveLocale(env);
}

function localeCandidates(locale) {
  var normalized = normalizeLocale(locale) || "en";
  var candidates = [normalized];
  var base = normalized.split("-")[0];
  if (base !== normalized) candidates.push(base);
  if (candidates.indexOf("en") < 0) candidates.push("en");
  return candidates;
}

function isValidCatalog(catalog) {
  if (!catalog || typeof catalog !== "object" || Array.isArray(catalog)) return false;
  var english = catalog.en;
  if (
    !english ||
    typeof english !== "object" ||
    Array.isArray(english) ||
    !Object.keys(english).length
  )
    return false;
  return Object.keys(catalog).every(function (locale) {
    var translations = catalog[locale];
    return (
      normalizeLocale(locale) === locale &&
      translations &&
      typeof translations === "object" &&
      !Array.isArray(translations) &&
      Object.keys(translations).every(function (source) {
        return typeof translations[source] === "string";
      })
    );
  });
}

function catalogFromText(text, previousCatalog) {
  try {
    var parsed = JSON.parse(text);
    if (isValidCatalog(parsed)) return parsed;
  } catch (error) {
    // A replacement observed mid-write must not discard the active catalog.
  }
  return previousCatalog || {};
}

function translate(catalog, locale, source, args) {
  var text = String(source);
  var map = catalog && typeof catalog === "object" ? catalog : {};
  var choices = localeCandidates(locale);
  for (var i = 0; i < choices.length; i++) {
    var translations = map[choices[i]];
    if (translations && typeof translations[text] === "string" && translations[text].length) {
      text = translations[text];
      break;
    }
  }
  var values = args && typeof args === "object" ? args : {};
  return text.replace(/\{([A-Za-z_][A-Za-z0-9_]*)\}/g, function (match, name) {
    return Object.prototype.hasOwnProperty.call(values, name) ? String(values[name]) : match;
  });
}

function isRtl(locale) {
  return (
    ["ar", "fa", "he", "ur", "ps", "dv", "ku", "yi"].indexOf(
      (normalizeLocale(locale) || "en").split("-")[0],
    ) >= 0
  );
}

function pluralCategory(locale, count) {
  var n = Math.abs(Number(count));
  if (!isFinite(n)) n = 0;
  var language = (normalizeLocale(locale) || "en").split("-")[0];
  if (["ar"].indexOf(language) >= 0) {
    if (n === 0) return "zero";
    if (n === 1) return "one";
    if (n === 2) return "two";
    if (n % 100 >= 3 && n % 100 <= 10) return "few";
    if (n % 100 >= 11 && n % 100 <= 99) return "many";
    return "other";
  }
  return n === 1 ? "one" : "other";
}

function translatePlural(catalog, locale, forms, count, args) {
  var category = pluralCategory(locale, count);
  var source = (forms && (forms[category] || forms.other)) || "";
  var values = {};
  var supplied = args && typeof args === "object" ? args : {};
  Object.keys(supplied).forEach(function (key) {
    values[key] = supplied[key];
  });
  values.count = count;
  return translate(catalog, locale, source, values);
}

var api = {
  isValidCatalog: isValidCatalog,
  catalogFromText: catalogFromText,
  effectiveLocale: effectiveLocale,
  normalizeLocale: normalizeLocale,
  resolveLocale: resolveLocale,
  localeCandidates: localeCandidates,
  translate: translate,
  isRtl: isRtl,
  pluralCategory: pluralCategory,
  translatePlural: translatePlural,
};
if (typeof module !== "undefined" && module.exports) module.exports = api;
