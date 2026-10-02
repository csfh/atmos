const { load, assertEqual } = require("./harness");

const layout = load("services/Layout.js");

const hubOnly = layout.breadcrumb("Appearance", "");
assertEqual(hubOnly.length, 1, "a hub page has one crumb");
assertEqual(hubOnly[0].link, false, "the hub crumb is not a link when it is current");

const sub = layout.breadcrumb("Appearance", "Background");
assertEqual(sub.length, 2, "a subpage adds a crumb");
assertEqual(sub[0].link, true, "the hub crumb links back from a subpage");
assertEqual(sub[1].label, "Background", "the subpage is the last crumb");
assertEqual(sub[1].link, false, "the last crumb is not a link");

assertEqual(layout.breadcrumb("", "").length, 0, "an empty trail has no crumbs");

const withGroup = layout.breadcrumb("Appearance", "Background", "Desktop");
assertEqual(withGroup.length, 3, "a group adds a leading crumb");
assertEqual(withGroup[0].label, "Desktop", "the group comes first");
assertEqual(withGroup[0].link, false, "the group is never a link");
assertEqual(withGroup[0].context, true, "the group is marked as context");
assertEqual(withGroup[1].link, true, "the hub links back from a subpage");
assertEqual(layout.breadcrumb("Home", "", "").length, 1, "a hub with no group has one crumb");
assertEqual(layout.breadcrumb("", "", "Desktop").length, 0, "a group alone is not a trail");

assertEqual(layout.railMode(800), true, "800px collapses the sidebar to a rail");
assertEqual(layout.railMode(960), false, "960px keeps the full sidebar");
assertEqual(layout.railMode(0), false, "an unknown width keeps the full sidebar");

const group = { title: "Desktop", pages: [{ id: "appearance" }, { id: "displays" }] };
assertEqual(layout.groupHolds(group, "displays"), true, "a group knows its pages");
assertEqual(layout.groupHolds(group, "sound"), false, "a group rejects other pages");
assertEqual(
  layout.groupOpen({ Desktop: true }, "Desktop", false),
  false,
  "a collapsed group is closed",
);
assertEqual(
  layout.groupOpen({ Desktop: true }, "Desktop", true),
  true,
  "the current group never closes",
);
assertEqual(
  layout.groupOpen({ Desktop: true }, "", false),
  true,
  "an untitled group is always open",
);
const toggled = layout.toggleGroup({}, "Desktop");
assertEqual(toggled.Desktop, true, "toggling collapses an open group");
assertEqual(layout.toggleGroup(toggled, "Desktop").Desktop, undefined, "toggling again reopens it");

const sidebar = [
  { title: "", pages: [{ id: "home" }, { id: "monitor" }] },
  { title: "Desktop", pages: [{ id: "appearance" }, { id: "displays" }] },
  { title: "Controls", pages: [{ id: "input" }, { id: "sound" }] },
];
assertEqual(
  Object.keys(layout.defaultCollapsed(sidebar, "appearance", 1000, 30)).length,
  0,
  "a sidebar that fits starts fully open",
);
const folded = layout.defaultCollapsed(sidebar, "appearance", 100, 30);
assertEqual(folded.Controls, true, "a group that does not hold the page starts collapsed");
assertEqual(folded.Desktop, undefined, "the group holding the page stays open");
assertEqual(
  Object.keys(layout.defaultCollapsed(sidebar, "home", 100, 30)).length,
  2,
  "an untitled group holds home and every titled group folds",
);
assertEqual(
  Object.keys(layout.defaultCollapsed(null, "home", 100, 30)).length,
  0,
  "no groups means nothing to fold",
);
