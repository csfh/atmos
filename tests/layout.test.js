const { load, assert, assertEqual } = require("./harness");

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
  { label: "Same", description: "Visible copy.", detail: "Visible copy.", hint: "omarchy x" },
  { label: "", description: "", detail: "", hint: "" },
  null,
]);
assertEqual(helpTopics.length, 1, "sectionHelpTopics keeps only extra row detail");
assertEqual(
  helpTopics[0].body,
  "A named palette plus templates.",
  "sectionHelpTopics uses extra detail",
);
assertEqual(helpTopics[0].command, "omarchy theme set", "sectionHelpTopics keeps the command");
assertEqual(layout.sectionHelpTopics(null).length, 0, "sectionHelpTopics ignores a non-array");
assertEqual(
  layout.helpAccessibleName("DNS"),
  "About DNS settings",
  "helpAccessibleName names the section",
);
assertEqual(
  layout.helpAccessibleName("Power settings"),
  "About Power settings",
  "helpAccessibleName does not double settings",
);
assert(
  layout.helpTextIsExtra(
    "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1.",
    ["Who answers name lookups for this machine."],
  ),
  "DNS section copy is extra help",
);
assertEqual(
  layout.helpTextIsExtra(
    "Font and size apply together to the shell, GTK apps, and terminals. Reset puts size back to 12 pixels.",
    [
      "Font",
      "The monospace face used by the shell and terminals.",
      "Text size",
      "How large type is in the shell, GTK apps, and terminals. You can pick 9 to 20 pixels.",
      "Reset text size",
      "Put type back to 12 pixels everywhere Omarchy sets it.",
    ],
  ),
  false,
  "Text section copy is already on the rows",
);
const textPayload = layout.sectionHelpPayload(
  "Font and size apply together to the shell, GTK apps, and terminals. Reset puts size back to 12 pixels.",
  "omarchy font set",
  [
    {
      label: "Font",
      description: "The monospace face used by the shell and terminals.",
      hint: "omarchy font set",
    },
    {
      label: "Text size",
      description:
        "How large type is in the shell, GTK apps, and terminals. You can pick 9 to 20 pixels.",
    },
    {
      label: "Reset text size",
      description: "Put type back to 12 pixels everywhere Omarchy sets it.",
    },
  ],
);
assertEqual(layout.sectionHelpOpen(textPayload), false, "redundant section help stays closed");
const dnsPayload = layout.sectionHelpPayload(
  "This writes NetworkManager and systemd-resolved so lookups go through the same resolvers. Cloudflare is 1.1.1.1 and 1.0.0.1. A VPN can still win for that connection.",
  "omarchy dns",
  [{ label: "DNS provider", description: "Who answers name lookups for this machine." }],
);
assert(layout.sectionHelpOpen(dnsPayload), "extra section help opens the icon");
assertEqual(dnsPayload.command, "omarchy dns", "extra section help still keeps the command");
const rowExtra = layout.sectionHelpPayload("Scale is the usual slider.", "", [
  {
    label: "Scale",
    description: "How large the interface looks on the focused monitor.",
    detail:
      "Scale is Hyprland's factor of UI pixels over physical pixels. 200% on a 4K panel makes chrome and text about the size they would be at 1080p.",
    hint: "omarchy hyprland monitor scaling",
  },
]);
assert(layout.sectionHelpOpen(rowExtra), "extra row detail opens the icon");
assertEqual(rowExtra.topics.length, 1, "extra row detail is a help topic");
assertEqual(layout.clusterByGroup(null).length, 0, "clusterByGroup ignores a non-array");
assertEqual(layout.clusterByGroup([]).length, 0, "clusterByGroup empty list");
assertEqual(layout.navGroupLabel("look"), "Desktop", "navGroupLabel names look");
assertEqual(layout.navGroupLabel("input"), "Controls", "navGroupLabel names input");
assertEqual(layout.navGroupLabel("device"), "Machine", "navGroupLabel names device");
assertEqual(layout.navGroupLabel("apps"), "Apps", "navGroupLabel names apps");
assertEqual(layout.navGroupLabel("general"), "General", "navGroupLabel names general");
assertEqual(layout.navGroupLabel("admin"), "Admin", "navGroupLabel names admin");
assertEqual(layout.navGroupLabel("nope"), "", "navGroupLabel misses an unknown group");
assertEqual(layout.navGroupLabel("home"), "", "navGroupLabel leaves home unlabeled");
const homeCluster = layout.clusterByGroup([
  { id: "home", group: "home" },
  { id: "favorites", group: "look" },
]);
assertEqual(homeCluster[0].title, "", "clusterByGroup draws home without a heading");
assertEqual(homeCluster[0].pages[0].id, "home", "clusterByGroup keeps home first");
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
assertEqual(
  layout.flattenNavPages(clustered).join(","),
  "appearance,display,input,system",
  "flattenNavPages walks clustered pages in drawn order",
);
assertEqual(
  layout.flattenNavPages(searched).join(","),
  "appearance,system",
  "flattenNavPages follows the filtered cluster, not the catalogue",
);
assertEqual(layout.flattenNavPages(null).length, 0, "flattenNavPages ignores a non-array");
assertEqual(
  layout.flattenNavPages([{ title: "Desktop" }]).length,
  0,
  "flattenNavPages skips a cluster with no pages",
);
assertEqual(layout.stepNavIndex(["a", "b", "c"], "a", 1), 1, "stepNavIndex j moves down");
assertEqual(layout.stepNavIndex(["a", "b", "c"], "b", -1), 0, "stepNavIndex k moves up");
assertEqual(layout.stepNavIndex(["a", "b", "c"], "a", -1), 0, "stepNavIndex k clamps at the top");
assertEqual(layout.stepNavIndex(["a", "b", "c"], "c", 1), 2, "stepNavIndex j clamps at the bottom");
assertEqual(
  layout.stepNavIndex(["a", "c"], "b", 1),
  0,
  "stepNavIndex j lands on the first match when the current hub is filtered away",
);
assertEqual(
  layout.stepNavIndex(["a", "c"], "b", -1),
  1,
  "stepNavIndex k lands on the last match when the current hub is filtered away",
);
assertEqual(layout.stepNavIndex([], "a", 1), -1, "stepNavIndex no-ops on an empty list");
assertEqual(layout.jumpNavIndex(["a", "b", "c"], false), 0, "jumpNavIndex g is the first hub");
assertEqual(layout.jumpNavIndex(["a", "b", "c"], true), 2, "jumpNavIndex G is the last hub");
assertEqual(layout.jumpNavIndex([], true), -1, "jumpNavIndex no-ops on an empty list");

assertEqual(layout.countGridSections(null), 0, "countGridSections ignores a non-array");
assertEqual(layout.countGridSections([]), 0, "countGridSections empty list");
assertEqual(
  layout.countGridSections([
    { prefsGroup: true, visible: true },
    { prefsGroup: true, visible: true },
  ]),
  2,
  "countGridSections counts visible groups",
);
assertEqual(
  layout.countGridSections([
    { prefsGroup: true, visible: false },
    { prefsGroup: true, visible: true, wide: true },
    { prefsGroup: true, visible: true },
    { visible: true },
    null,
  ]),
  1,
  "countGridSections skips hidden, wide, and non-groups",
);

assertEqual(
  layout.sectionColumnCount(700, 520, 40, 2, 4),
  1,
  "sectionColumnCount stays 1 when two columns do not fit",
);
assertEqual(
  layout.sectionColumnCount(1080, 520, 40, 2, 4),
  2,
  "sectionColumnCount opens a second column at 2*min + gap",
);
assertEqual(
  layout.sectionColumnCount(1800, 520, 40, 2, 4),
  2,
  "sectionColumnCount honors maxColumns",
);
assertEqual(
  layout.sectionColumnCount(1800, 520, 40, 2, 1),
  1,
  "sectionColumnCount keeps a lone group full width",
);
assertEqual(
  layout.sectionColumnCount(1800, 520, 40, 2, 0),
  1,
  "sectionColumnCount is 1 when every group is wide",
);
assertEqual(
  layout.sectionColumnCount(-10, 520, 40, 2, 4),
  1,
  "sectionColumnCount ignores a bad width",
);
assertEqual(
  layout.sectionColumnWidth(1000, 1, 40),
  1000,
  "sectionColumnWidth is the full column when there is one",
);
assertEqual(
  layout.sectionColumnWidth(1080, 2, 40),
  520,
  "sectionColumnWidth splits the gap out of two columns",
);
assertEqual(layout.sectionColumnWidth(1400, 2, 40), 680, "sectionColumnWidth floors equal columns");

const pageOpts = {
  margin: 20,
  cap: 1000,
  wideCap: 1400,
  minColumn: 520,
  gap: 40,
  maxColumns: 2,
};
assertEqual(
  layout.pageContentWidth(740, Object.assign({ itemCount: 6 }, pageOpts)),
  700,
  "pageContentWidth uses the inner width when it is under the single-column cap",
);
assertEqual(
  layout.pageContentWidth(2000, Object.assign({ itemCount: 1 }, pageOpts)),
  1000,
  "pageContentWidth keeps a single group at the single-column cap",
);
assertEqual(
  layout.pageContentWidth(960, Object.assign({ itemCount: 6 }, pageOpts)),
  920,
  "pageContentWidth stays one column at the default window content width",
);
assertEqual(
  layout.pageContentWidth(1340, Object.assign({ itemCount: 6 }, pageOpts)),
  1300,
  "pageContentWidth grows past the single-column cap once two columns fit",
);
assertEqual(
  layout.pageContentWidth(2000, Object.assign({ itemCount: 6 }, pageOpts)),
  1400,
  "pageContentWidth caps the two-column grid",
);
assertEqual(
  layout.pageContentWidth(2000, Object.assign({ itemCount: 0 }, pageOpts)),
  1000,
  "pageContentWidth keeps a page of wide groups at the single-column cap",
);
assertEqual(
  layout.pageContentWidth(0, Object.assign({ itemCount: 1, minWidth: 240 }, pageOpts)),
  240,
  "pageContentWidth floors a tiny pane",
);
