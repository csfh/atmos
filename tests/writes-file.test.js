const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const writes = load("services/WritesFile.js");

function claimsWholeFile(text) {
  const s = String(text || "").toLowerCase();
  return (
    s.indexOf("managed by atmos") !== -1 ||
    s.indexOf("atmos file") !== -1 ||
    s.indexOf("owns") !== -1 ||
    s.indexOf("edits in place") !== -1
  );
}

assertEqual(writes.normalizeMode(""), "sentinel", "empty mode is sentinel");
assertEqual(writes.normalizeMode("managed"), "sentinel", "unknown mode is sentinel");
assertEqual(writes.normalizeMode("true"), "sentinel", "bool-shaped mode is sentinel");
assertEqual(writes.normalizeMode("file"), "file", "file mode stays file");
assertEqual(writes.normalizeMode("command"), "command", "command mode stays command");
assertEqual(writes.ownsWholeFile(""), false, "default mode does not own the file");
assertEqual(writes.ownsWholeFile("sentinel"), false, "sentinel does not own the file");
assertEqual(writes.ownsWholeFile("command"), false, "command does not own the file");
assertEqual(writes.ownsWholeFile("file"), true, "file mode owns the file");

const inputCaption = writes.caption("~/.config/hypr/input.lua");
assertEqual(
  inputCaption,
  "~/.config/hypr/input.lua  ·  Atmos block",
  "default input caption is the Atmos block, not whole-file ownership",
);
assert(!claimsWholeFile(inputCaption), "sentinel caption does not claim whole-file ownership");
assert(
  !claimsWholeFile(writes.caption("~/.config/hypr/input.lua", "sentinel")),
  "explicit sentinel caption does not claim whole-file ownership",
);
assert(
  !claimsWholeFile(writes.caption("~/.config/hypr/input.lua", "")),
  "omitted mode cannot claim whole-file ownership",
);
assertEqual(
  writes.caption("~/.config/hypr/input.lua", "file"),
  "~/.config/hypr/input.lua  ·  Atmos file",
  "file mode can say Atmos owns the file",
);
assertEqual(
  writes.caption("omarchy font set", "command"),
  "omarchy font set  ·  via command",
  "command mode names a command write",
);
assertEqual(writes.caption(""), "", "empty path has no caption");
assertEqual(writes.caption("   "), "", "whitespace path has no caption");
assertEqual(
  writes.caption("~/.config/hypr/input.lua", "sentinel", {
    note: "your gesture stays",
  }),
  "~/.config/hypr/input.lua  ·  Atmos block  ·  your gesture stays",
  "optional note appends after the sentinel status",
);
assert(
  !claimsWholeFile(
    writes.caption("~/.config/hypr/input.lua", "sentinel", { note: "your gesture stays" }),
  ),
  "unmanaged-gesture note still does not claim the whole file",
);

const prefsGroupSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsGroup.qml"),
  "utf8",
);
assert(
  prefsGroupSrc.indexOf('property string writesMode: "sentinel"') !== -1,
  "PrefsGroup defaults writesMode to sentinel",
);
assert(
  prefsGroupSrc.indexOf("property bool writesManaged") === -1,
  "PrefsGroup has no writesManaged bool that can default to ownership",
);
assert(
  prefsGroupSrc.indexOf('import "../services/WritesFile.js" as WritesFile') !== -1 &&
    prefsGroupSrc.indexOf("WritesFile.caption(") !== -1,
  "PrefsGroup builds the caption from WritesFile.js",
);

const headingAt = prefsGroupSrc.indexOf("id: headingHost");
assert(headingAt !== -1, "PrefsGroup has headingHost");
const rowsAt = prefsGroupSrc.indexOf("id: rowsWrap");
const headingHost = prefsGroupSrc.slice(headingAt, rowsAt);
const beforeHeading = prefsGroupSrc.slice(0, headingAt);
assert(
  beforeHeading.indexOf("provenanceLabel") === -1 &&
    beforeHeading.indexOf("writesFile") !== -1 &&
    beforeHeading.indexOf("id: headingHost") === -1,
  "writesFile properties live on the group; the caption is not a Column sibling above the heading",
);
assert(
  headingHost.indexOf("id: titleLabel") !== -1 &&
    headingHost.indexOf("id: provenanceLabel") !== -1 &&
    headingHost.indexOf("id: titleLabel") < headingHost.indexOf("id: provenanceLabel"),
  "provenance caption is inside headingHost after the section title",
);
assert(
  headingHost.indexOf("PrefsText") !== -1 &&
    headingHost.indexOf("id: provenanceLabel") !== -1 &&
    headingHost.slice(0, headingHost.indexOf("id: provenanceLabel")).lastIndexOf("PrefsText") !==
      -1,
  "provenance caption is a PrefsText like SettingRow captions",
);
assert(
  headingHost.indexOf("Theme.metaSize") !== -1 &&
    headingHost.indexOf("Theme.metaOpacity") !== -1 &&
    headingHost.indexOf("Theme.muted") !== -1,
  "provenance caption uses SettingRow caption tokens",
);
assert(
  headingHost.indexOf("spacing: Theme.titleGap") !== -1,
  "title and caption sit on titleGap inside the heading block",
);
assert(
  /\n  spacing: Theme.headingGap\n/.test(prefsGroupSrc),
  "Theme.headingGap still sits between the heading block and the first row",
);
assert(
  headingHost.indexOf("implicitHeight: headingColumn.implicitHeight") !== -1 ||
    prefsGroupSrc.indexOf("implicitHeight: headingColumn.implicitHeight") !== -1,
  "headingHost grows with the title-plus-caption column",
);
assert(
  headingHost.indexOf("reveal: headingHover.hovered") !== -1,
  "section help hover stays on the heading block",
);

const inputSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "InputPage.qml"), "utf8");
["Pointer", "Touchpad", "Keyboard", "Advanced"].forEach(function (title) {
  const needle = 'title: "' + title + '"';
  const start = inputSrc.indexOf(needle);
  assert(start !== -1, "Input has a " + title + " section");
  const next = inputSrc.indexOf('title: "', start + needle.length);
  const block = inputSrc.slice(start, next === -1 ? inputSrc.length : next);
  assert(
    block.indexOf('writesFile: "~/.config/hypr/input.lua"') !== -1 ||
      block.indexOf("writesFile: root.inputWritesFile") !== -1,
    title + " labels ~/.config/hypr/input.lua",
  );
  assert(
    block.indexOf('writesMode: "file"') === -1 && block.indexOf("writesManaged: true") === -1,
    title + " does not claim whole-file ownership of input.lua",
  );
});
assert(
  /title: "Advanced"[\s\S]*writesFile:/.test(inputSrc),
  "Advanced carries the same write-target caption; unmanaged gesture deferral lives there",
);
assert(
  inputSrc.indexOf("hyprWorkspaceGestureUnmanaged") !== -1 &&
    /title: "Advanced"[\s\S]*writesNote:/.test(inputSrc),
  "Advanced wires the unmanaged-gesture note to real state",
);
