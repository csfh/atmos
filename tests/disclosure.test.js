const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const d = load("services/Disclosure.js");

assertEqual(d.normalizeHub("input"), "input", "normalizeHub keeps a hub id");
assertEqual(d.normalizeHub("windows/bindings"), "windows", "normalizeHub strips a subpage");
assertEqual(d.normalizeHub(""), "", "normalizeHub empty stays empty");

const idle = d.emptyState();
assertEqual(idle.simple, false, "default mode is Everything");
assertEqual(idle.revealedLabel, "", "default has no search pin");
assertEqual(idle.pending, false, "default is not pending a search land");

const accel = { advanced: true, query: "", label: "Acceleration", hub: "input" };
assertEqual(d.rowFolded(accel, idle), false, "Everything does not fold an advanced row");
assertEqual(
  d.rowFolded(accel, { simple: true }),
  true,
  "Simple folds an advanced row when the query is empty",
);
assertEqual(
  d.rowFolded({ advanced: true, query: "accel", label: "Acceleration" }, { simple: true }),
  false,
  "a non-empty query unfolds an advanced row",
);
assertEqual(
  d.rowFolded({ advanced: false, query: "", label: "Sensitivity" }, { simple: true }),
  false,
  "Simple leaves an untagged row alone",
);
assertEqual(
  d.rowFolded({ query: "" }, { simple: true }),
  false,
  "a row without advanced never folds",
);

let state = d.emptyState();
state.simple = true;
assert(d.rowFolded(accel, state), "Simple + empty query folds Acceleration");

state = d.revealFromSearch(state, "input", "Follow mouse");
assertEqual(state.pending, true, "search navigation marks the pin pending");
assertEqual(state.revealedHub, "input", "search pin stores the hub");
assertEqual(state.revealedLabel, "Follow mouse", "search pin stores the row label");
assertEqual(
  d.rowFolded({ advanced: true, query: "", label: "Follow mouse", hub: "input" }, state),
  false,
  "the landing row is not folded after a search hit",
);
assertEqual(
  d.rowFolded({ advanced: true, query: "", label: "Wake on key", hub: "input" }, state),
  true,
  "a different advanced row stays folded",
);

const mid = d.leaveHub(state, "appearance");
assertEqual(
  mid.revealedLabel,
  "Follow mouse",
  "pending pin survives the hub reload that clears search",
);
assertEqual(mid.pending, true, "leaveHub does not consume a pending search pin");

state = d.finishReveal(mid, "input");
assertEqual(state.pending, false, "landing the search hit consumes pending");
assertEqual(state.revealedLabel, "Follow mouse", "pin stays after the page opens");
assertEqual(
  d.rowFolded({ advanced: true, query: "", label: "Follow mouse", hub: "input" }, state),
  false,
  "the landing row stays revealed until the user leaves",
);

state = d.leaveHub(state, "appearance");
assertEqual(state.revealedLabel, "", "leaving the hub drops the pin");
assertEqual(
  d.rowFolded({ advanced: true, query: "", label: "Follow mouse", hub: "input" }, state),
  true,
  "the row folds again after leave",
);

const noLabel = d.revealFromSearch(d.emptyState(), "input", "");
assertEqual(noLabel.pending, false, "go without a label does not pin");
assertEqual(noLabel.revealedHub, "", "go without a label does not keep a hub");

const miss = d.finishReveal(
  d.revealFromSearch({ simple: true }, "input", "Follow mouse"),
  "accounts",
);
assertEqual(miss.revealedLabel, "", "landing on another hub drops the pin");
assertEqual(miss.pending, false, "a missed land is not left pending");

const group = {
  advanced: true,
  query: "",
  hub: "input",
  labels: ["Follow mouse", "Wake on key", "Three-finger swipe"],
};
assertEqual(d.groupFolded(group, idle), false, "Everything leaves an Advanced section up");
assertEqual(
  d.groupFolded(group, { simple: true }),
  true,
  "Simple folds an opted-in Advanced section",
);
assertEqual(
  d.groupFolded(group, { simple: true, revealedHub: "input", revealedLabel: "Follow mouse" }),
  false,
  "a search hit inside the section unfolds the whole section",
);
assertEqual(
  d.groupFolded(group, { simple: true, revealedHub: "input", revealedLabel: "Sensitivity" }),
  true,
  "a search hit on another row does not unfold the section",
);
assertEqual(
  d.groupFolded({ advanced: true, query: "follow", labels: ["Follow mouse"] }, { simple: true }),
  false,
  "a non-empty query unfolds an Advanced section",
);
assertEqual(
  d.groupFolded({ advanced: false, query: "", labels: ["Follow mouse"] }, { simple: true }),
  false,
  "an untagged section never folds",
);

assertEqual(d.showModeToggle(false, {}), false, "no toggle when the page has no advanced rows");
assertEqual(d.showModeToggle(true, {}), true, "toggle when the page opted in");
assertEqual(
  d.showModeToggle(true, { embed: true }),
  false,
  "embedded pages do not show the toggle",
);
assertEqual(d.showModeToggle(true, { query: "mouse" }), false, "in-page query hides the toggle");
assertEqual(
  d.showModeToggle(false, { query: "" }),
  false,
  "empty query is not enough without advanced rows",
);

assertEqual(
  d.helpCollectsRow({ available: true, matches: true, visible: true, folded: false }),
  true,
  "help keeps a visible matching row",
);
assertEqual(
  d.helpCollectsRow({ available: true, matches: true, visible: false, folded: true }),
  false,
  "help skips a folded row unless asked",
);
assertEqual(
  d.helpCollectsRow(
    { available: true, matches: true, visible: false, folded: true },
    { includeFolded: true },
  ),
  true,
  "section help includes a folded row",
);
assertEqual(
  d.helpCollectsRow(
    { available: true, matches: false, visible: false, folded: true },
    { includeFolded: true },
  ),
  false,
  "help still omits a row the query missed",
);
assertEqual(
  d.helpCollectsRow(
    { available: false, matches: true, visible: false, folded: true },
    { includeFolded: true },
  ),
  false,
  "help still omits an unavailable row",
);
assertEqual(
  d.helpCollectsRow(
    { available: true, matches: true, visible: false, folded: true, sectionHelp: false },
    { includeFolded: true },
  ),
  false,
  "list rows stay out of section help",
);

function walkQml(dir, acc) {
  acc = acc || [];
  fs.readdirSync(dir, { withFileTypes: true }).forEach(function (ent) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) walkQml(full, acc);
    else if (ent.name.endsWith(".qml")) acc.push(full);
  });
  return acc;
}

const pagesDir = path.join(__dirname, "..", "pages");
const advancedBlocks = [];
const untriagedHits = [];
walkQml(pagesDir).forEach(function (file) {
  const src = fs.readFileSync(file, "utf8");
  const rel = path.relative(path.join(__dirname, ".."), file);
  const re = /title:\s*"Advanced"/g;
  let m;
  while ((m = re.exec(src))) {
    const window = src.slice(Math.max(0, m.index - 40), m.index + 80);
    advancedBlocks.push({ rel: rel, window: window });
  }
  if (/DiagnosticsPage|AccountsPage|FavoritesPage/.test(rel) && /advanced:\s*true/.test(src)) {
    untriagedHits.push(rel);
  }
});

assert(advancedBlocks.length >= 12, "existing Advanced sections are still in the tree");
advancedBlocks.forEach(function (block) {
  assert(
    /title:\s*"Advanced"[\s\S]{0,80}advanced:\s*true/.test(block.window) ||
      /advanced:\s*true[\s\S]{0,80}title:\s*"Advanced"/.test(block.window),
    block.rel + " Advanced section opts into Disclosure",
  );
});
assertEqual(untriagedHits.length, 0, "Diagnostics, Accounts, and Favorites stay untriaged");

const inputSrc = fs.readFileSync(path.join(pagesDir, "InputPage.qml"), "utf8");
assert(
  /title:\s*"Advanced"[\s\S]{0,80}advanced:\s*true/.test(inputSrc),
  "Input folds its existing Advanced section",
);
assert(
  !/label:\s*"Acceleration"[\s\S]{0,80}advanced:\s*true/.test(inputSrc) &&
    !/advanced:\s*true[\s\S]{0,40}label:\s*"Acceleration"/.test(inputSrc),
  "Input does not mark Acceleration as the Simple demo",
);
assert(
  !/label:\s*"Scroll inertia"[\s\S]{0,80}advanced:\s*true/.test(inputSrc),
  "Input does not mark Scroll inertia as the Simple demo",
);
assert(
  !/label:\s*"Three-finger drag"[\s\S]{0,80}advanced:\s*true/.test(inputSrc),
  "Input does not mark Three-finger drag as the Simple demo",
);
