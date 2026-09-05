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
