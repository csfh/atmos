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
  let parsed = { workspaceGesture: false, workspaceGestureManaged: false };
  try {
    fs.writeFileSync(file, text);
    const result = spawnSync(
      "python3",
      [path.join(__dirname, "..", "scripts", "hypr-sentinel.py"), "input", "list", file],
      { encoding: "utf8" },
    );
    assertEqual(result.status, 0, "hypr-sentinel.py input list exits 0");
    parsed = JSON.parse(String(result.stdout || "").replace(/^\s+|\s+$/g, "") || "{}");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  return parsed;
}

function pythonApplyInput(existing, payload) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-input-apply-"));
  const file = path.join(dir, "input.lua");
  let text = "";
  try {
    fs.writeFileSync(file, existing);
    const result = spawnSync(
      "python3",
      [path.join(__dirname, "..", "scripts", "hypr-sentinel.py"), "input", "apply", file],
      { input: JSON.stringify(payload), encoding: "utf8" },
    );
    assertEqual(result.status, 0, "hypr-sentinel.py input apply exits 0");
    text = fs.readFileSync(file, "utf8");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  return text;
}

function assertGestureState(state, expected, description) {
  assertEqual(state.workspaceGesture, expected.workspaceGesture, description + " · on");
  assertEqual(
    state.workspaceGestureManaged,
    expected.workspaceGestureManaged,
    description + " · managed",
  );
  assertEqual(
    state.workspaceGestureUnmanaged,
    expected.workspaceGestureUnmanaged,
    description + " · unmanaged",
  );
}

const onManaged = {
  workspaceGesture: true,
  workspaceGestureManaged: true,
  workspaceGestureUnmanaged: false,
};
const off = {
  workspaceGesture: false,
  workspaceGestureManaged: false,
  workspaceGestureUnmanaged: false,
};
const onUnmanaged = {
  workspaceGesture: true,
  workspaceGestureManaged: false,
  workspaceGestureUnmanaged: true,
};

const liveGesture = hypr.serializeInput({ workspaceGesture: true });
assertGestureState(
  hypr.inputWorkspaceGestureState(liveGesture),
  onManaged,
  "inputWorkspaceGestureState sees a live managed gesture",
);
assertGestureState(
  pythonInputGesture(liveGesture),
  onManaged,
  "hypr-sentinel.py input list sees a live managed gesture",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(hypr.serializeInput({ workspaceGesture: false })),
  off,
  "inputWorkspaceGestureState misses a managed block with no gesture",
);

const commentedGesture = `-- atmos:input begin
hl.config({
  input = { sensitivity = 0 },
})
-- hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- atmos:input end
`;
assertGestureState(
  hypr.inputWorkspaceGestureState(commentedGesture),
  off,
  "inputWorkspaceGestureState skips a commented gesture in the sentinel",
);
assertGestureState(
  pythonInputGesture(commentedGesture),
  off,
  "hypr-sentinel.py input list skips a commented gesture in the sentinel",
);

const trailingGesture = `-- atmos:input begin
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" }) -- keep
-- atmos:input end
`;
assertGestureState(
  hypr.inputWorkspaceGestureState(trailingGesture),
  onManaged,
  "inputWorkspaceGestureState keeps a live gesture with a trailing comment",
);
assertGestureState(
  pythonInputGesture(trailingGesture),
  onManaged,
  "hypr-sentinel.py input list keeps a live gesture with a trailing comment",
);

const stringDashGesture = `-- atmos:input begin
hint = "flags --help"; hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- atmos:input end
`;
assertGestureState(
  hypr.inputWorkspaceGestureState(stringDashGesture),
  onManaged,
  "inputWorkspaceGestureState keeps a gesture after -- inside a string",
);
assertGestureState(
  pythonInputGesture(stringDashGesture),
  onManaged,
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
  "inputHasWorkspaceGesture is the managed bit and stays false for a bare gesture",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(unmanagedGesture),
  onUnmanaged,
  "inputWorkspaceGestureState reports a bare gesture as on and not managed",
);
assertGestureState(
  pythonInputGesture(unmanagedGesture),
  onUnmanaged,
  "hypr-sentinel.py input list reports a bare gesture as on and not managed",
);

const legacyGesture = `-- omarchy-prefs:input begin
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
-- omarchy-prefs:input end
`;
assertGestureState(
  hypr.inputWorkspaceGestureState(legacyGesture),
  onManaged,
  "inputWorkspaceGestureState reads a leftover omarchy-prefs input gesture as managed",
);
assertGestureState(
  pythonInputGesture(legacyGesture),
  onManaged,
  "hypr-sentinel.py input list reads a leftover omarchy-prefs input gesture as managed",
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
assertGestureState(
  hypr.inputWorkspaceGestureState(leftoverAtmosOff),
  off,
  "inputWorkspaceGestureState prefers the atmos sentinel over a leftover gesture",
);
assertGestureState(
  pythonInputGesture(leftoverAtmosOff),
  off,
  "hypr-sentinel.py input list prefers the atmos sentinel over a leftover gesture",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(""),
  off,
  "inputWorkspaceGestureState misses an empty file",
);

const bareStock = 'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })\n';
assertEqual(
  hypr.inputHasWorkspaceGesture(bareStock),
  false,
  "inputHasWorkspaceGesture is false for a gesture with no sentinel",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(bareStock),
  onUnmanaged,
  "a live stock line with no sentinel is on and not managed",
);
assertGestureState(
  pythonInputGesture(bareStock),
  onUnmanaged,
  "hypr-sentinel.py input list treats a live stock line as on and not managed",
);

const commentedStock = `-- Enable touchpad gestures for changing workspaces.
-- hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })
`;
assertGestureState(
  hypr.inputWorkspaceGestureState(commentedStock),
  off,
  "the commented Omarchy stock line is not an unmanaged gesture",
);
assertGestureState(
  pythonInputGesture(commentedStock),
  off,
  "hypr-sentinel.py input list ignores the commented Omarchy stock line",
);

const unmanagedWritten = hypr.applyInputFile(unmanagedGesture, {
  sensitivity: -0.5,
  workspaceGesture: true,
});
assertEqual(
  (unmanagedWritten.match(/hl\.gesture\(/g) || []).length,
  1,
  "applyInputFile does not write a managed gesture when an unmanaged one is live",
);
assert(
  unmanagedWritten.indexOf(
    'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })',
  ) ===
    unmanagedWritten.lastIndexOf(
      'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })',
    ),
  "applyInputFile keeps the single unmanaged gesture line",
);
assert(
  /-- atmos:input begin[\s\S]*hl\.gesture[\s\S]*-- atmos:input end/.test(unmanagedWritten) ===
    false,
  "applyInputFile omits hl.gesture from the atmos block when unmanaged",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(unmanagedWritten),
  onUnmanaged,
  "after a write, the unmanaged gesture is still on and not managed",
);
const unmanagedPython = pythonApplyInput(unmanagedGesture, {
  sensitivity: -0.5,
  workspaceGesture: true,
});
assertEqual(
  unmanagedPython.replace(/\s+$/, ""),
  unmanagedWritten.replace(/\s+$/, ""),
  "hypr-sentinel.py apply matches applyInputFile when deferring to an unmanaged gesture",
);

const managedOnlyWritten = hypr.applyInputFile("-- keep input comments\n", {
  workspaceGesture: true,
});
assert(
  /-- atmos:input begin[\s\S]*hl\.gesture\(\{ fingers = 3[\s\S]*-- atmos:input end/.test(
    managedOnlyWritten,
  ),
  "applyInputFile still writes a managed gesture when nothing unmanaged is live",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(managedOnlyWritten),
  onManaged,
  "a managed-only write reports on and managed",
);
const managedOnlyPython = pythonApplyInput("-- keep input comments\n", { workspaceGesture: true });
assertEqual(
  managedOnlyPython.replace(/\s+$/, ""),
  managedOnlyWritten.replace(/\s+$/, ""),
  "hypr-sentinel.py apply matches applyInputFile on the managed-only path",
);

const commentedStockWritten = hypr.applyInputFile(commentedStock, { workspaceGesture: true });
assertEqual(
  (commentedStockWritten.match(/^\s*hl\.gesture\(/gm) || []).length,
  1,
  "a commented stock line does not count as unmanaged, so Atmos still writes",
);
assert(
  commentedStockWritten.indexOf(
    '-- hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })',
  ) !== -1,
  "applyInputFile leaves the commented stock line where it is",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(commentedStockWritten),
  onManaged,
  "writing over a commented stock line is managed",
);
assertEqual(
  pythonApplyInput(commentedStock, { workspaceGesture: true }).replace(/\s+$/, ""),
  commentedStockWritten.replace(/\s+$/, ""),
  "hypr-sentinel.py apply matches applyInputFile over a commented stock line",
);

const commentedOutUnmanaged = unmanagedWritten.replace(
  'hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })\n',
  '-- hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })\n',
);
const recovered = hypr.applyInputFile(commentedOutUnmanaged, { workspaceGesture: true });
assert(
  /-- atmos:input begin[\s\S]*hl\.gesture\(\{ fingers = 3[\s\S]*-- atmos:input end/.test(recovered),
  "after the unmanaged line is commented out, applyInputFile writes a managed gesture",
);
assertGestureState(
  hypr.inputWorkspaceGestureState(recovered),
  onManaged,
  "commenting the stock line out lets Atmos own the gesture",
);

assert(
  hypr.serializeInput({ workspaceGesture: true }, unmanagedGesture).indexOf("hl.gesture(") === -1,
  "serializeInput omits the Atmos gesture when existing text has an unmanaged one",
);
assert(
  hypr.serializeInput({ workspaceGesture: true }).indexOf("hl.gesture(") !== -1,
  "serializeInput still writes the Atmos gesture with no existing unmanaged line",
);
