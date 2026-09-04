const { load, assertEqual } = require("./harness");

const layout = load("services/Layout.js");

function flags(items) {
  return layout
    .splitAfterVisible(items)
    .map(function (v) {
      return v ? 1 : 0;
    })
    .join("");
}

assertEqual(
  flags([{ visible: true }, { visible: true }, { visible: true }]),
  "110",
  "splitAfterVisible splits between visible items, not after the last",
);
assertEqual(
  flags([{ visible: true }, { visible: false }, { visible: true }]),
  "100",
  "splitAfterVisible skips hidden items between two visible ones",
);
assertEqual(
  flags([{ visible: false }, { visible: true }, { visible: false }]),
  "000",
  "splitAfterVisible has no split for a single visible item",
);
assertEqual(
  flags([{ visible: false }, { visible: false }]),
  "00",
  "splitAfterVisible has no split when nothing is visible",
);
assertEqual(flags([]).length, 0, "splitAfterVisible empty list");
assertEqual(flags(null).length, 0, "splitAfterVisible ignores a non-array");

function beforeFlags(items) {
  return layout
    .splitBeforeVisible(items)
    .map(function (v) {
      return v ? 1 : 0;
    })
    .join("");
}

assertEqual(
  beforeFlags([{ visible: true }, { visible: true }, { visible: true }]),
  "011",
  "splitBeforeVisible draws on later items, not after the last",
);
assertEqual(
  beforeFlags([{ visible: true }, { visible: false }, { visible: true }]),
  "001",
  "splitBeforeVisible skips hidden items",
);
assertEqual(
  beforeFlags([{ visible: false }, { visible: true }, { visible: false }]),
  "000",
  "splitBeforeVisible has no split on the first visible item",
);

const helpTopics = layout.sectionHelpTopics([
  {
    label: "Theme",
    description: "Palette.",
    detail: "A named palette plus templates.",
    hint: "omarchy theme set",
  },
  { label: "Font", description: "Monospace family.", hint: "omarchy font set" },
  { label: "", description: "", detail: "", hint: "" },
  null,
]);
assertEqual(helpTopics.length, 2, "sectionHelpTopics skips empty rows");
assertEqual(
  helpTopics[0].body,
  "A named palette plus templates.",
  "sectionHelpTopics prefers detail",
);
assertEqual(helpTopics[1].body, "Monospace family.", "sectionHelpTopics falls back to description");
assertEqual(helpTopics[1].command, "omarchy font set", "sectionHelpTopics keeps the command");
assertEqual(layout.sectionHelpTopics(null).length, 0, "sectionHelpTopics ignores a non-array");
assertEqual(layout.clusterByGroup(null).length, 0, "clusterByGroup ignores a non-array");
assertEqual(layout.clusterByGroup([]).length, 0, "clusterByGroup empty list");
assertEqual(layout.navGroupLabel("look"), "Desktop", "navGroupLabel names look");
assertEqual(layout.navGroupLabel("input"), "Controls", "navGroupLabel names input");
assertEqual(layout.navGroupLabel("device"), "Machine", "navGroupLabel names device");
assertEqual(layout.navGroupLabel("apps"), "Apps", "navGroupLabel names apps");
assertEqual(layout.navGroupLabel("admin"), "Admin", "navGroupLabel names admin");
assertEqual(layout.navGroupLabel("nope"), "", "navGroupLabel misses an unknown group");
const clustered = layout.clusterByGroup([
  { id: "appearance", group: "look" },
  { id: "display", group: "look" },
  { id: "input", group: "input" },
  { id: "system", group: "admin" },
]);
assertEqual(clustered.length, 3, "clusterByGroup splits when the group changes");
assertEqual(clustered[0].title, "Desktop", "clusterByGroup labels look as Desktop");
assertEqual(clustered[0].pages.length, 2, "clusterByGroup keeps consecutive look hubs");
assertEqual(clustered[0].pages[1].id, "display", "clusterByGroup keeps order inside a group");
assertEqual(clustered[1].pages[0].id, "input", "clusterByGroup starts a new cluster");
assertEqual(clustered[1].title, "Controls", "clusterByGroup labels input as Controls");
assertEqual(clustered[2].title, "Admin", "clusterByGroup labels admin");
const searched = layout.clusterByGroup(
  [
    { id: "appearance", group: "look" },
    { id: "system", group: "admin" },
  ],
  false,
);
assertEqual(searched.length, 1, "clusterByGroup is one list while searching");
assertEqual(searched[0].title, "", "clusterByGroup drops labels while searching");
assertEqual(searched[0].pages.length, 2, "clusterByGroup keeps every search hit");
