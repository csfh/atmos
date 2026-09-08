const { load, assert, assertEqual } = require("./harness");

const fav = load("services/Favorites.js");

assertEqual(fav.slug("Text size"), "text-size", "slug hyphenates a label");
assertEqual(fav.slug("  GPU  "), "gpu", "slug trims");
assertEqual(fav.rowKey("appearance", "Theme"), "appearance/theme", "rowKey is hub plus slug");
assertEqual(
  fav.rowKey("appearance/background", "Set background"),
  "appearance/background/set-background",
  "rowKey keeps a child hub",
);
assertEqual(fav.rowKey("", "Theme"), "", "rowKey misses an empty hub");
assertEqual(fav.rowKey("Appearance", "Theme"), "", "rowKey refuses an uppercase hub");
assertEqual(fav.rowKey("look n feel", "Theme"), "", "rowKey refuses a spaced hub");

const item = fav.normalizeItem({
  hub: "sound",
  label: "Volume",
  description: "Speaker level.",
  hubTitle: "Sound",
});
assertEqual(item.id, "sound/volume", "normalizeItem ids from hub and label");
assertEqual(item.hubTitle, "Sound", "normalizeItem keeps hubTitle");
assertEqual(fav.normalizeItem({ hub: "sound" }), null, "normalizeItem drops a row without a label");
assertEqual(fav.normalizeItem("sound/volume"), null, "normalizeItem drops a bare string");

const parsed = fav.parseDocument(
  '{"items":[{"hub":"sound","label":"Volume"},{"hub":"sound","label":"Volume"},{"hub":"","label":"Nope"}]}',
);
assertEqual(parsed.length, 1, "parseDocument drops duplicates and invalid rows");
assertEqual(fav.parseDocument("not json").length, 0, "parseDocument survives bad JSON");
assertEqual(fav.parseDocument("").length, 0, "parseDocument treats empty as none");
assertEqual(
  fav.parseDocument('[{"hub":"bar","label":"Show bar"}]').length,
  1,
  "parseDocument accepts a bare list",
);

const added = fav.toggleItem([], { hub: "sound", label: "Volume", hubTitle: "Sound" });
assertEqual(added.length, 1, "toggleItem adds a missing row");
assert(fav.isFavorite(added, "sound", "Volume"), "isFavorite sees a starred row");
const removed = fav.toggleItem(added, { hub: "sound", label: "Volume" });
assertEqual(removed.length, 0, "toggleItem removes a starred row");
assert(!fav.isFavorite(removed, "sound", "Volume"), "isFavorite misses a removed row");
const forced = fav.toggleItem([], { hub: "sound", label: "Volume" }, false);
assertEqual(forced.length, 0, "toggleItem want false does not add");
const kept = fav.toggleItem(
  added,
  { hub: "sound", label: "Volume", description: "New copy." },
  true,
);
assertEqual(kept[0].description, "New copy.", "toggleItem want true refreshes copy");

const grouped = fav.groups([
  { hub: "sound", label: "Volume", hubTitle: "Sound" },
  { hub: "appearance", label: "Theme", hubTitle: "Appearance" },
  { hub: "sound", label: "Mute", hubTitle: "Sound" },
]);
assertEqual(grouped.length, 2, "groups keeps star order of hubs");
assertEqual(grouped[0].hub, "sound", "groups first hub is the first star");
assertEqual(grouped[0].items.length, 2, "groups collects rows for one hub");
assertEqual(grouped[1].title, "Appearance", "groups uses hubTitle");

const json = fav.serialize(added);
assert(json.indexOf('"items"') !== -1, "serialize wraps items");
assertEqual(fav.parseDocument(json)[0].id, "sound/volume", "serialize round-trips");
