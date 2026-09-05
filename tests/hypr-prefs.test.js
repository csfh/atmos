const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const hypr = load("services/HyprPrefs.js");
const look = hypr.clampLook({ gapsIn: 80, layout: "niri", dimStrength: 2 });
assertEqual(look.gapsIn, 64, "clampLook caps gaps");
assertEqual(look.layout, "dwindle", "clampLook rejects an unknown layout");
assertEqual(look.dimStrength, 1, "clampLook caps dim strength");

const lookLua = hypr.serializeLook({
  gapsIn: 8,
  gapsOut: 12,
  borderSize: 3,
  rounding: 6,
  blur: true,
  shadow: false,
  layout: "scrolling",
  columnWidth: 0.97,
  dimInactive: true,
  dimStrength: 0.15,
  animations: false,
  cursorHideOnKey: false,
  cursorWarp: true,
  allowTearing: true,
  resizeOnBorder: false,
});
assert(lookLua.indexOf("-- atmos:look begin") === 0, "serializeLook starts with the look sentinel");
assert(lookLua.indexOf("gaps_in = 8") !== -1, "serializeLook writes gaps_in");
assert(lookLua.indexOf('layout = "scrolling"') !== -1, "serializeLook writes scrolling");
assert(lookLua.indexOf("column_width = 0.97") !== -1, "serializeLook writes column width");
assert(lookLua.indexOf("warp_on_change_workspace = 1") !== -1, "serializeLook writes cursor warp");
assert(
  lookLua.indexOf('hl.env("HYPRCURSOR_SIZE", "24")') !== -1,
  "serializeLook writes default cursor size",
);
assert(lookLua.indexOf("active_opacity = 1") !== -1, "serializeLook writes default active opacity");
assert(
  lookLua.indexOf("inactive_opacity = 1") !== -1,
  "serializeLook writes default inactive opacity",
);
assert(lookLua.indexOf("preserve_split = false") !== -1, "serializeLook writes preserve_split");
assert(
  lookLua.indexOf("focus_on_activate = false") !== -1,
  "serializeLook writes focus_on_activate",
);

const lookExtras = hypr.serializeLook({
  cursorSize: 40,
  activeOpacity: 0.8,
  inactiveOpacity: 0.7,
  preserveSplit: true,
  focusOnActivate: true,
});
assert(
  lookExtras.indexOf('hl.env("HYPRCURSOR_SIZE", "40")') !== -1,
  "serializeLook writes a custom cursor size",
);
assert(
  lookExtras.indexOf('hl.env("XCURSOR_SIZE", "40")') !== -1,
  "serializeLook writes XCURSOR_SIZE",
);
assert(lookExtras.indexOf("active_opacity = 0.8") !== -1, "serializeLook writes active opacity");
assert(
  lookExtras.indexOf("inactive_opacity = 0.7") !== -1,
  "serializeLook writes inactive opacity",
);
assert(
  lookExtras.indexOf("preserve_split = true") !== -1,
  "serializeLook writes preserve_split on",
);
assertEqual(hypr.clampLook({ cursorSize: 90 }).cursorSize, 64, "clampLook caps cursor size");
assertEqual(
  hypr.clampLook({ activeOpacity: 0.05 }).activeOpacity,
  0.2,
  "clampLook floors active opacity",
);
assertEqual(
  hypr.clampLook({ inactiveOpacity: 0.05 }).inactiveOpacity,
  0.2,
  "clampLook floors inactive opacity",
);
assertEqual(
  hypr.lookFromHyprOptions({ inactiveOpacity: 0.85 }).inactiveOpacity,
  0.85,
  "lookFromHyprOptions keeps inactive opacity",
);
assertEqual(hypr.luaNumber(8), "8", "luaNumber writes an integer");
assertEqual(hypr.luaNumber(0.97), "0.97", "luaNumber trims trailing zeros");
assertEqual(hypr.luaNumber(NaN), "0", "luaNumber treats NaN as zero");

const seed = "-- keep this comment\n\nhl.config({ general = { gaps_in = 1 } })\n";
const applied = hypr.applyLookFile(seed, { gapsIn: 4, gapsOut: 8 });
assert(applied.indexOf("-- keep this comment") !== -1, "applyLookFile keeps user comments");
assert(
  applied.indexOf("hl.config({ general = { gaps_in = 1 } })") !== -1,
  "applyLookFile keeps earlier hl.config",
);
assert(
  hypr.hasSentinel(applied, hypr.LOOK_BEGIN, hypr.LOOK_END),
  "applyLookFile inserts the look sentinel",
);
const twice = hypr.applyLookFile(applied, { gapsIn: 9, gapsOut: 8 });
assertEqual(
  (twice.match(/-- atmos:look begin/g) || []).length,
  1,
  "applyLookFile replaces an existing look block",
);
assert(twice.indexOf("gaps_in = 9") !== -1, "applyLookFile updates gaps");
const resetLook = hypr.resetLookFile(twice);
assert(resetLook.indexOf("-- atmos:look begin") === -1, "resetLookFile strips the look block");
assert(resetLook.indexOf("-- keep this comment") !== -1, "resetLookFile keeps user comments");
assertEqual(
  hypr.extractSentinel(applied, hypr.LOOK_BEGIN, hypr.LOOK_END).indexOf("gaps_in = 4") !== -1,
  true,
  "extractSentinel returns the look block",
);
assertEqual(
  hypr.extractSentinel("-- none\n", hypr.LOOK_BEGIN, hypr.LOOK_END),
  "",
  "extractSentinel misses a file without a sentinel",
);
assertEqual(
  hypr.stripSentinel("-- keep\n", hypr.LOOK_BEGIN, hypr.LOOK_END),
  "-- keep\n",
  "stripSentinel is a no-op without a sentinel",
);
assert(
  hypr.hasSentinel("-- none\n", hypr.LOOK_BEGIN, hypr.LOOK_END) === false,
  "hasSentinel is false without a look block",
);

const inputLua = hypr.serializeInput({
  sensitivity: -0.64,
  accelProfile: "flat",
  naturalScroll: true,
  kbLayoutOverride: "us,dk",
  kbGroupToggle: true,
  workspaceGesture: true,
});
assert(inputLua.indexOf('accel_profile = "flat"') !== -1, "serializeInput writes a flat profile");
assert(inputLua.indexOf('kb_layout = "us,dk"') !== -1, "serializeInput writes a layout override");
assert(inputLua.indexOf("grp:alts_toggle") !== -1, "serializeInput adds the group toggle");
assert(
  inputLua.indexOf("hl.gesture({ fingers = 3") !== -1,
  "serializeInput writes the workspace gesture",
);
assert(
  inputLua.indexOf("emulate_discrete_scroll = 1") !== -1,
  "serializeInput writes default discrete scroll",
);
assert(
  hypr.serializeInput({ emulateDiscreteScroll: 0 }).indexOf("emulate_discrete_scroll = 0") !== -1,
  "serializeInput writes a smooth wheel",
);
assertEqual(
  hypr.clampInput({ emulateDiscreteScroll: 5 }).emulateDiscreteScroll,
  2,
  "clampInput caps discrete scroll",
);
assertEqual(
  hypr.clampInput({ emulateDiscreteScroll: -1 }).emulateDiscreteScroll,
  0,
  "clampInput floors discrete scroll",
);
assertEqual(
  hypr.clampInput({ kbLayoutOverride: "US,dk" }).kbLayoutOverride,
  "us,dk",
  "clampInput lowercases layouts",
);
assertEqual(
  hypr.clampInput({ kbLayoutOverride: "us/dk" }).kbLayoutOverride,
  "",
  "clampInput rejects a slash in layouts",
);
assertEqual(hypr.parseCssFirst("5 5 5 5"), 5, "parseCssFirst reads the first css number");
assert(isNaN(hypr.parseCssFirst("")), "parseCssFirst empty is NaN");
assert(isNaN(hypr.parseCssFirst("nope 10")), "parseCssFirst rejects a non-numeric first token");
assertEqual(
  hypr.parseHyprOption({ css: "nope", str: "dwindle" }),
  "dwindle",
  "parseHyprOption falls through invalid css to str",
);
assertEqual(hypr.parseHyprOption({ int: 40 }), 40, "parseHyprOption reads int");
assertEqual(hypr.parseHyprOption({ bool: false }), false, "parseHyprOption reads bool");
assertEqual(hypr.parseHyprOption({ css: "10 10 10 10" }), 10, "parseHyprOption reads css");
assertEqual(
  hypr.parseHyprOption({ str: "[[EMPTY]]" }),
  "",
  "parseHyprOption treats empty str as blank",
);
assertEqual(hypr.parseHyprOption({ str: "dwindle" }), "dwindle", "parseHyprOption reads str");
assertEqual(hypr.parseHyprOption({ float: 0.15 }), 0.15, "parseHyprOption reads float");
assertEqual(hypr.parseHyprOption("not json"), null, "parseHyprOption rejects junk JSON");
assertEqual(hypr.parseHyprOption('{"int":5}'), 5, "parseHyprOption parses JSON text");
assertEqual(hypr.sanitizeLayoutList(" us, DK "), "us,dk", "sanitizeLayoutList lowercases a list");
assertEqual(
  hypr.sanitizeLayoutList("us,too-long-id"),
  "",
  "sanitizeLayoutList rejects an overlong id",
);
assertEqual(
  hypr.sanitizeVariantList("intl,nodeadkeys", 2),
  "intl,nodeadkeys",
  "sanitizeVariantList keeps matching variants",
);
assertEqual(
  hypr.sanitizeVariantList("intl, nodeadkeys", 2),
  "intl,nodeadkeys",
  "sanitizeVariantList trims spaces around tokens",
);
assertEqual(
  hypr.sanitizeVariantList(" ,nodeadkeys", 2),
  ",nodeadkeys",
  "sanitizeVariantList keeps an empty first variant",
);
assertEqual(
  hypr.sanitizeVariantList("intl", 2),
  "",
  "sanitizeVariantList rejects a count mismatch",
);
assertEqual(
  hypr.clampInput({ kbLayoutOverride: "us,dk", kbVariantOverride: "intl" }).kbVariantOverride,
  "",
  "clampInput drops a mismatched variant list",
);
assert(
  hypr
    .serializeInput({ kbLayoutOverride: "us,dk", kbVariantOverride: "intl,nodeadkeys" })
    .indexOf('kb_variant = "intl,nodeadkeys"') !== -1,
  "serializeInput writes a matching variant list",
);
assertEqual(hypr.lookFromHyprOptions({ gapsIn: 8 }).gapsIn, 8, "lookFromHyprOptions picks gapsIn");
assertEqual(
  hypr.lookFromHyprOptions({ gapsIn: 8 }).gapsOut,
  10,
  "lookFromHyprOptions fills look defaults",
);
assertEqual(hypr.asBool("on", false), true, "asBool accepts on");
assertEqual(hypr.asBool("off", true), false, "asBool accepts off");
assertEqual(hypr.asBool("true", false), true, "asBool accepts true string");
assertEqual(hypr.asBool(1, false), true, "asBool accepts 1");
assertEqual(hypr.asBool("0", true), false, "asBool accepts 0 string");
assertEqual(hypr.asBool("maybe", true), true, "asBool uses fallback on junk");
assertEqual(hypr.clampFloat(0.1234, 0, 1, 0.5), 0.123, "clampFloat rounds to thousandths");
assertEqual(hypr.clampFloat(9, 0, 1, 0.5), 1, "clampFloat caps at max");
const inputSeed = "-- keep input comments\nhl.config({ input = { sensitivity = 0 } })\n";
const inputApplied = hypr.applyInputFile(inputSeed, {
  sensitivity: -0.2,
  accelProfile: "adaptive",
});
assert(inputApplied.indexOf("-- keep input comments") !== -1, "applyInputFile keeps user comments");
assert(
  hypr.hasSentinel(inputApplied, hypr.INPUT_BEGIN, hypr.INPUT_END),
  "applyInputFile inserts the input sentinel",
);
assert(
  inputApplied.indexOf('accel_profile = "adaptive"') !== -1,
  "applyInputFile writes adaptive accel",
);
const inputReset = hypr.resetInputFile(inputApplied);
assert(inputReset.indexOf("-- atmos:input begin") === -1, "resetInputFile strips the input block");
assert(inputReset.indexOf("-- keep input comments") !== -1, "resetInputFile keeps user comments");
const leftoverInput = [
  "-- keep input comments",
  "-- omarchy-prefs:input begin",
  'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })',
  "-- omarchy-prefs:input end",
  "-- atmos:input begin",
  'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })',
  "-- atmos:input end",
  "",
].join("\n");
const inputMigrated = hypr.applyInputFile(leftoverInput, { workspaceGesture: true });
assert(
  inputMigrated.indexOf("-- omarchy-prefs:input begin") === -1,
  "applyInputFile strips a leftover omarchy-prefs input block",
);
assertEqual(
  (inputMigrated.match(/hl\.gesture\(/g) || []).length,
  1,
  "applyInputFile leaves one workspace gesture",
);
assert(
  hypr
    .resetLookFile("-- omarchy-prefs:look begin\nhl.config({})\n-- omarchy-prefs:look end\n")
    .indexOf("omarchy-prefs") === -1,
  "resetLookFile strips a leftover omarchy-prefs look block",
);

// The live writer is hypr-sentinel.py. JS serializeLook/serializeInput is
// what Node tests. They drifted once (scroll inertia). Fail if they do again.
function pythonSentinel(kind, payload) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-sentinel-"));
  const file = path.join(dir, kind + ".lua");
  let status = 1;
  let text = "";
  try {
    fs.writeFileSync(file, "");
    const result = spawnSync(
      "python3",
      [path.join(__dirname, "..", "scripts", "hypr-sentinel.py"), kind, "apply", file],
      { input: JSON.stringify(payload), encoding: "utf8" },
    );
    status = result.status;
    if (status === 0) text = fs.readFileSync(file, "utf8").replace(/\s+$/, "");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  assertEqual(status, 0, "hypr-sentinel.py " + kind + " apply exits 0");
  return text;
}

const lookLock = {
  gapsIn: 8,
  gapsOut: 12,
  borderSize: 3,
  rounding: 6,
  blur: true,
  shadow: false,
  layout: "scrolling",
  columnWidth: 0.97,
  dimInactive: true,
  dimStrength: 0.15,
  animations: false,
  cursorHideOnKey: false,
  cursorWarp: true,
  cursorSize: 40,
  allowTearing: true,
  resizeOnBorder: false,
  activeOpacity: 0.8,
  inactiveOpacity: 0.7,
  preserveSplit: true,
  focusOnActivate: true,
};
assertEqual(
  pythonSentinel("look", lookLock),
  hypr.serializeLook(lookLock),
  "hypr-sentinel.py look matches serializeLook",
);

const inputLock = {
  sensitivity: -0.64,
  accelProfile: "flat",
  emulateDiscreteScroll: 0,
  naturalScroll: true,
  scrollFactor: 0.8,
  clickfinger: false,
  disableWhileTyping: false,
  drag3fg: 1,
  repeatRate: 50,
  repeatDelay: 300,
  numlock: false,
  followMouse: 2,
  keyPressDpms: false,
  mouseMoveDpms: false,
  kbLayoutOverride: "us,dk",
  kbVariantOverride: "intl,nodeadkeys",
  kbGroupToggle: true,
  workspaceGesture: true,
};
assertEqual(
  pythonSentinel("input", inputLock),
  hypr.serializeInput(inputLock),
  "hypr-sentinel.py input matches serializeInput",
);

const spacedInput = Object.assign({}, inputLock, {
  kbLayoutOverride: "us, dk",
  kbVariantOverride: "intl, nodeadkeys",
});
assertEqual(
  pythonSentinel("input", spacedInput),
  hypr.serializeInput(spacedInput),
  "hypr-sentinel.py input matches serializeInput with spaced layout lists",
);
assert(
  hypr.serializeInput(spacedInput).indexOf('kb_layout = "us,dk"') !== -1,
  "serializeInput strips spaces in layout lists",
);
assert(
  hypr.serializeInput(spacedInput).indexOf('kb_variant = "intl,nodeadkeys"') !== -1,
  "serializeInput strips spaces in variant lists",
);

function pythonInputGesture(text) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-gesture-"));
  const file = path.join(dir, "input.lua");
  let parsed = false;
  try {
    fs.writeFileSync(file, text);
    const result = spawnSync(
      "python3",
      [path.join(__dirname, "..", "scripts", "hypr-sentinel.py"), "input", "list", file],
      { encoding: "utf8" },
    );
    assertEqual(result.status, 0, "hypr-sentinel.py input list exits 0");
    parsed = String(result.stdout || "").replace(/^\s+|\s+$/g, "") === "true";
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  return parsed;
}

const liveGesture = hypr.serializeInput({ workspaceGesture: true });
assertEqual(
  hypr.inputHasWorkspaceGesture(liveGesture),
  true,
  "inputHasWorkspaceGesture sees a live managed gesture",
);
assertEqual(
  pythonInputGesture(liveGesture),
  true,
  "hypr-sentinel.py input list sees a live managed gesture",
);
assertEqual(
  hypr.inputHasWorkspaceGesture(hypr.serializeInput({ workspaceGesture: false })),
  false,
  "inputHasWorkspaceGesture misses a managed block with no gesture",
);

const commentedGesture = `-- atmos:input begin
hl.config({
  input = { sensitivity = 0 },
})
-- hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- atmos:input end
`;
assertEqual(
  hypr.inputHasWorkspaceGesture(commentedGesture),
  false,
  "inputHasWorkspaceGesture skips a commented gesture in the sentinel",
);
assertEqual(
  pythonInputGesture(commentedGesture),
  false,
  "hypr-sentinel.py input list skips a commented gesture in the sentinel",
);

const trailingGesture = `-- atmos:input begin
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" }) -- keep
-- atmos:input end
`;
assertEqual(
  hypr.inputHasWorkspaceGesture(trailingGesture),
  true,
  "inputHasWorkspaceGesture keeps a live gesture with a trailing comment",
);
assertEqual(
  pythonInputGesture(trailingGesture),
  true,
  "hypr-sentinel.py input list keeps a live gesture with a trailing comment",
);

const stringDashGesture = `-- atmos:input begin
hint = "flags --help"; hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- atmos:input end
`;
assertEqual(
  hypr.inputHasWorkspaceGesture(stringDashGesture),
  true,
  "inputHasWorkspaceGesture keeps a gesture after -- inside a string",
);
assertEqual(
  pythonInputGesture(stringDashGesture),
  true,
  "hypr-sentinel.py input list keeps a gesture after -- inside a string",
);

const unmanagedGesture = `hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- atmos:input begin
hl.config({
  input = { sensitivity = 0 },
})
-- atmos:input end
`;
assertEqual(
  hypr.inputHasWorkspaceGesture(unmanagedGesture),
  false,
  "inputHasWorkspaceGesture ignores an unmanaged gesture outside the sentinel",
);
assertEqual(
  pythonInputGesture(unmanagedGesture),
  false,
  "hypr-sentinel.py input list ignores an unmanaged gesture outside the sentinel",
);

const legacyGesture = `-- omarchy-prefs:input begin
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- omarchy-prefs:input end
`;
assertEqual(
  hypr.inputHasWorkspaceGesture(legacyGesture),
  true,
  "inputHasWorkspaceGesture reads a leftover omarchy-prefs input gesture",
);
assertEqual(
  pythonInputGesture(legacyGesture),
  true,
  "hypr-sentinel.py input list reads a leftover omarchy-prefs input gesture",
);

const leftoverAtmosOff = [
  "-- omarchy-prefs:input begin",
  'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })',
  "-- omarchy-prefs:input end",
  "-- atmos:input begin",
  "hl.config({",
  "  input = { sensitivity = 0 },",
  "})",
  "-- atmos:input end",
  "",
].join("\n");
assertEqual(
  hypr.inputHasWorkspaceGesture(leftoverAtmosOff),
  false,
  "inputHasWorkspaceGesture prefers the atmos sentinel over a leftover gesture",
);
assertEqual(
  pythonInputGesture(leftoverAtmosOff),
  false,
  "hypr-sentinel.py input list prefers the atmos sentinel over a leftover gesture",
);
assertEqual(
  hypr.inputHasWorkspaceGesture(""),
  false,
  "inputHasWorkspaceGesture misses an empty file",
);
assertEqual(
  hypr.inputHasWorkspaceGesture(
    'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })\n',
  ),
  false,
  "inputHasWorkspaceGesture misses a gesture with no sentinel",
);
