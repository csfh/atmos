const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const themeQml = fs.readFileSync(path.join(__dirname, "..", "services", "Theme.qml"), "utf8");
assert(themeQml.indexOf("inotifywait") !== -1, "Theme.qml watches currentDir with inotifywait");
assert(
  themeQml.indexOf("interval: 1000") !== -1,
  "Theme.qml restarts inotifywait after 1s, not a poll",
);
assert(themeQml.indexOf("interval: 800") === -1, "Theme.qml has no 800ms theme poll");

const theme = load("services/Theme.js");
const shell = load("services/ShellConfig.js");

const colors = theme.parseColors(`
foreground = "#a9b1d6"
background = "#1a1b26"
accent = "#7aa2f7"
muted = "#414868"
red = "#f7768e"
`);
assertEqual(colors.foreground, "#a9b1d6", "parseColors reads foreground");
assertEqual(colors.background, "#1a1b26", "parseColors reads background");
assertEqual(colors.accent, "#7aa2f7", "parseColors prefers accent over color4");
assertEqual(colors.muted, "#414868", "parseColors reads muted");
assertEqual(colors.urgent, "#f7768e", "parseColors maps red to urgent");

const legacy = theme.parseColors(`
color0 = "#111111"
color4 = "#0000ff"
color7 = "#eeeeee"
color8 = "#888888"
color1 = "#ff0000"
`);
assertEqual(legacy.background, "#111111", "parseColors falls back to color0");
assertEqual(legacy.foreground, "#eeeeee", "parseColors falls back to color7");
assertEqual(legacy.accent, "#0000ff", "parseColors falls back to color4");
assertEqual(legacy.muted, "#888888", "parseColors falls back to color8");
assertEqual(legacy.urgent, "#ff0000", "parseColors maps color1 to urgent");

const shellValues = theme.parseShell(`
[font]
base-size = 14
[controls]
normal-fill-alpha = 0.04
hover-cursor-fill-alpha = 0.08
# comment
[bar]
position = top
`);
assertEqual(shellValues["font.base-size"], "14", "parseShell reads numeric font.base-size");
assertEqual(shellValues["controls.normal-fill-alpha"], "0.04", "parseShell reads control alphas");
assertEqual(shellValues["bar.position"], "top", "parseShell reads bare strings");
assertEqual(
  theme.numberToken(shellValues, "font.base-size", 12),
  14,
  "numberToken coerces font size",
);

const merged = theme.mergeShell({ "font.base-size": "12" }, { "font.base-size": "16" });
assertEqual(merged["font.base-size"], "16", "user shell.toml wins over theme");
assertEqual(
  theme.themeSlug("Catppuccin Latte"),
  "catppuccin-latte",
  "themeSlug kebab-cases a display name",
);
assertEqual(theme.themeSlug("Miasma"), "miasma", "themeSlug lowercases a single word");
assertEqual(
  theme.themeNameFromSlug("miasma", ["Catppuccin Latte", "Miasma", "Tokyo Night"]),
  "Miasma",
  "themeNameFromSlug maps a slug onto the display name",
);
assertEqual(
  theme.themeNameFromSlug("catppuccin-latte", ["Catppuccin Latte", "Miasma"]),
  "Catppuccin Latte",
  "themeNameFromSlug maps a kebab slug onto a spaced name",
);
assertEqual(
  theme.themeNameFromSlug("unknown-theme", ["Miasma"]),
  "unknown-theme",
  "themeNameFromSlug keeps an unknown slug",
);
assertEqual(
  theme.themeFileCandidates("Catppuccin Latte", "colors.toml", "/home/u", "/usr/share/omarchy")[0],
  "/home/u/.config/omarchy/themes/catppuccin-latte/colors.toml",
  "themeFileCandidates prefers the user overlay",
);
assertEqual(
  theme.themeFileCandidates("Catppuccin Latte", "colors.toml", "/home/u", "/usr/share/omarchy")[1],
  "/usr/share/omarchy/themes/catppuccin-latte/colors.toml",
  "themeFileCandidates falls back to the packaged theme",
);
assertEqual(
  theme.themeFileCandidates("../x", "colors.toml", "/home/u").length,
  0,
  "themeFileCandidates rejects a path slug",
);
assertEqual(
  theme.themeFileCandidates("x", "../colors.toml", "/home/u").length,
  0,
  "themeFileCandidates rejects a path file",
);

assertEqual(theme.formatSeconds(45), "45s", "formatSeconds under a minute");
assertEqual(theme.formatSeconds(150), "2m 30s", "formatSeconds minutes and seconds");
assertEqual(theme.formatSeconds(300), "5m", "formatSeconds whole minutes");
assertEqual(theme.formatSeconds(-12), "0s", "formatSeconds clamps negative to zero");
assertEqual(theme.formatSeconds("nope"), "0s", "formatSeconds treats NaN as zero");
const quotedShell = theme.parseShell(`
orphan = 1
[bar]
position = "left"
padding = 8 12 8 12
name = 'Dock'
# skip
[font]
family = Inter
`);
assertEqual(quotedShell.orphan, undefined, "parseShell skips keys before a section");
assertEqual(quotedShell["bar.position"], "left", "parseShell reads a quoted string");
assertEqual(quotedShell["bar.padding"], "8 12 8 12", "parseShell reads a width list");
assertEqual(quotedShell["bar.name"], "Dock", "parseShell reads a single-quoted string");
assertEqual(quotedShell["font.family"], "Inter", "parseShell reads a bare identifier");
assertEqual(
  theme.numberToken({}, "font.base-size", 12),
  12,
  "numberToken uses fallback when missing",
);
assertEqual(theme.mergeShell(null, { a: "1" }).a, "1", "mergeShell accepts a null theme map");
const fg = theme.parseColors('fg = "#abcdef"\nbg = "#010203"\nurgent = "#ff00aa"\n');
assertEqual(fg.foreground, "#abcdef", "parseColors reads fg alias");
assertEqual(fg.background, "#010203", "parseColors reads bg alias");
assertEqual(fg.urgent, "#ff00aa", "parseColors reads urgent key");
assertEqual(fg.muted, "#abcdef", "parseColors muted falls back to foreground");

const parsed = shell.parseShellJson(
  '{"idle":{"screensaver":90,"lock":120},"bar":{"position":"left","transparent":true}}',
  "{}",
);
assertEqual(parsed.screensaver, 90, "parseShellJson reads screensaver");
assertEqual(parsed.lock, 120, "parseShellJson reads lock");
assertEqual(parsed.barPosition, "left", "parseShellJson reads bar position");
assert(parsed.barTransparent === true, "parseShellJson reads bar transparency");

const fromDefaults = shell.parseShellJson(
  "",
  '{"idle":{"screensaver":150,"lock":300},"bar":{"position":"top"}}',
);
assertEqual(fromDefaults.screensaver, 150, "parseShellJson uses defaults when user file is empty");
assertEqual(fromDefaults.barPosition, "top", "parseShellJson default bar position");
const badIdle = shell.parseShellJson(
  '{"idle":{"screensaver":-5,"lock":"nope"},"bar":{"position":"side","transparent":"yes"}}',
  "{}",
);
assertEqual(badIdle.screensaver, 150, "parseShellJson falls back on a negative screensaver");
assertEqual(badIdle.lock, 300, "parseShellJson falls back on a non-numeric lock");
assertEqual(badIdle.barPosition, "top", "parseShellJson rejects an unknown bar position");
assert(badIdle.barTransparent === false, "parseShellJson requires transparent === true");
const junkShell = shell.parseShellJson(
  "not json",
  '{"idle":{"screensaver":40,"lock":80},"bar":{"position":"bottom"}}',
);
assertEqual(junkShell.screensaver, 40, "parseShellJson uses defaults when user JSON is junk");
assertEqual(
  junkShell.barPosition,
  "bottom",
  "parseShellJson default bar position from defaultsRaw",
);
assertEqual(
  shell.parseShellJson("[]", "{}").barPosition,
  "top",
  "parseShellJson rejects a JSON array",
);
assertEqual(shell.positiveNumber(2.4, 0), 2, "positiveNumber rounds");
assertEqual(shell.positiveNumber(-1, 9), 9, "positiveNumber falls back below zero");

assert(shell.rowMatches("", ["Theme"]), "empty query matches");
assert(shell.rowMatches("font", ["Theme", "omarchy font set"]), "query matches hint");
assert(!shell.rowMatches("network", ["Theme", "font"]), "query rejects unrelated rows");
assert(
  shell.haystackMatches("FONT", shell.joinSearchHaystack(["Theme", "omarchy font set"])),
  "haystackMatches reuses a lowered haystack",
);
assert(
  !shell.haystackMatches("network", shell.joinSearchHaystack(["Theme", "font"])),
  "haystackMatches rejects unrelated rows",
);

const bgPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "appearance", "BackgroundPage.qml"),
  "utf8",
);
assert(
  bgPageSrc.indexOf("FileDialog") !== -1 &&
    bgPageSrc.indexOf("Omarchy.setBackgroundPath") !== -1 &&
    bgPageSrc.indexOf("setBackgroundFromFile") === -1 &&
    bgPageSrc.indexOf("openBackgroundSwitcher") === -1,
  "Background Choose… is a Qt FileDialog, not the native switcher helpers",
);
const bgOmarchySrc = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
const bgChooseStart = bgOmarchySrc.indexOf("function setBackgroundPath(");
const bgChooseEnd = bgOmarchySrc.indexOf("function cacheBackgrounds(", bgChooseStart);
const bgChooseBody = bgOmarchySrc.slice(bgChooseStart, bgChooseEnd);
assert(
  bgChooseBody.indexOf("dispatchSetting") !== -1,
  "setBackgroundPath still applies a picked path through dispatchSetting",
);
assert(
  bgOmarchySrc.indexOf('runInteractive(["omarchy", "theme", "bg-switcher"]') !== -1 &&
    bgOmarchySrc.indexOf('kind: "background-file"') !== -1 &&
    bgOmarchySrc.indexOf("function runInteractive(") !== -1,
  "native background pickers do not hold the mut queue",
);

function qmlFunctionBody(src, name) {
  const start = src.indexOf("function " + name + "(");
  if (start < 0) return "";
  const brace = src.indexOf("{", start);
  if (brace < 0) return "";
  let depth = 0;
  for (let i = brace; i < src.length; i++) {
    if (src[i] === "{") depth++;
    else if (src[i] === "}") {
      depth--;
      if (depth === 0) return src.slice(start, i + 1);
    }
  }
  return "";
}

const liveColors = {
  foreground: "#aaa111",
  background: "#111111",
  accent: "#222222",
  muted: "#333333",
  urgent: "#444444",
};
const liveShell = { "font.base-size": "16", "controls.normal-fill-alpha": "0.2" };
const snap = theme.snapshotLiveTheme(liveColors, liveShell);
assertEqual(snap.source, "live", "snapshotLiveTheme tags the restore source as live");
assertEqual(snap.colors.foreground, "#aaa111", "snapshotLiveTheme copies live foreground");
assertEqual(
  snap.themeShellValues["font.base-size"],
  "16",
  "snapshotLiveTheme copies live themeShellValues",
);
liveShell["font.base-size"] = "99";
assertEqual(
  snap.themeShellValues["font.base-size"],
  "16",
  "snapshotLiveTheme copies themeShellValues, it does not alias them",
);
const restored = theme.restoreLiveTheme(snap);
assertEqual(restored.source, "live", "restoreLiveTheme keeps the live source tag");
assertEqual(restored.colors.background, "#111111", "restoreLiveTheme returns snapshotted colors");
assertEqual(
  restored.themeShellValues["font.base-size"],
  "16",
  "restoreLiveTheme returns snapshotted shell tokens",
);
assertEqual(theme.restoreLiveTheme(null), null, "restoreLiveTheme rejects a missing snapshot");
assertEqual(
  theme.restoreLiveTheme({ source: "named", colors: liveColors, themeShellValues: {} }),
  null,
  "restoreLiveTheme rejects a named-directory snapshot",
);

const files = {
  "/home/u/.config/omarchy/themes/miasma/colors.toml": 'foreground = "#aabbcc"\n',
  "/usr/share/omarchy/themes/miasma/shell.toml": "[font]\nbase-size = 18\n",
};
const readFn = function (p) {
  return files[p] || "";
};
assertEqual(
  theme.firstThemeFile("Miasma", "colors.toml", "/home/u", readFn, "/usr/share/omarchy"),
  'foreground = "#aabbcc"\n',
  "firstThemeFile reads a named theme colors.toml for preview",
);
assertEqual(
  theme.firstThemeFile("Miasma", "shell.toml", "/home/u", readFn, "/usr/share/omarchy"),
  "[font]\nbase-size = 18\n",
  "firstThemeFile can read shell.toml when asked",
);
assert(
  theme.themeFileCandidates("Miasma", "colors.toml", "/home/u").indexOf("/home/u/.local/state") ===
    -1,
  "named theme candidates never include current/theme",
);

const previewFn = qmlFunctionBody(themeQml, "previewNamedTheme");
assert(
  previewFn.indexOf("captureLivePreview") !== -1,
  "previewNamedTheme snapshots live chrome first",
);
assert(
  previewFn.indexOf("colors.toml") !== -1 && previewFn.indexOf("shell.toml") === -1,
  "previewNamedTheme reads colors.toml only, not shell.toml",
);
assert(
  previewFn.indexOf("applyNamedTheme") === -1 &&
    previewFn.indexOf("setTheme") === -1 &&
    previewFn.indexOf("omarchy") === -1,
  "previewNamedTheme does not commit a theme",
);

const restoreFn = qmlFunctionBody(themeQml, "restorePreview");
assert(
  restoreFn.indexOf("restoreLiveTheme") !== -1 && restoreFn.indexOf("applyNamedTheme") === -1,
  "restorePreview reapplies the live snapshot, not applyNamedTheme",
);
assert(
  restoreFn.indexOf("themeFileCandidates") === -1 && restoreFn.indexOf("currentThemePath") === -1,
  "restorePreview does not reread named dirs or current/ files",
);

const applyFn = qmlFunctionBody(themeQml, "applyNamedTheme");
assert(
  applyFn.indexOf("themeShellValues = raw ? ThemeJs.parseShell(raw) : ({})") !== -1,
  "applyNamedTheme resets themeShellValues when a theme has no shell.toml",
);

const setThemeFn = qmlFunctionBody(bgOmarchySrc, "setTheme");
assert(
  setThemeFn.indexOf("Theme.discardPreview()") !== -1 &&
    setThemeFn.indexOf("Theme.restorePreview()") === -1,
  "setTheme drops a hover snapshot instead of restoring it",
);

const selectSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSelect.qml"),
  "utf8",
);
assert(selectSrc.indexOf("signal previewed(string value)") !== -1, "PrefsSelect emits previewed");
assert(
  selectSrc.indexOf("onEntered: root.hoverOption(root.optionValue(modelData))") !== -1 &&
    selectSrc.indexOf("onExited:") === -1,
  "PrefsSelect previews on enter only, never per-row onExited",
);
assert(
  selectSrc.indexOf("HoverHandler") !== -1 && selectSrc.indexOf('root.hoverOption("")') !== -1,
  "PrefsSelect reverts from the list HoverHandler and popup close",
);
const pickFn = qmlFunctionBody(selectSrc, "pickValue");
assert(
  pickFn.indexOf("root.clearHover()") !== -1 && pickFn.indexOf("hoverOption") === -1,
  'pickValue clears hover state without emitting previewed("")',
);
const clearFn = qmlFunctionBody(selectSrc, "clearHover");
assert(
  clearFn.indexOf("hoveredOption") !== -1 && clearFn.indexOf("previewed") === -1,
  "clearHover does not emit previewed",
);
assert(
  selectSrc.indexOf("onCurrentIndexChanged:") !== -1,
  "PrefsSelect previews keyboard highlight moves",
);

const appearanceSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AppearancePage.qml"),
  "utf8",
);
const themeSelectStart = appearanceSrc.indexOf('label: "Current theme"');
const themeSelectEnd = appearanceSrc.indexOf('label: "Theme files"', themeSelectStart);
const themeSelect = appearanceSrc.slice(themeSelectStart, themeSelectEnd);
assert(
  themeSelect.indexOf("onPreviewed:") !== -1 &&
    themeSelect.indexOf("Theme.previewNamedTheme") !== -1,
  "Current theme hover calls previewNamedTheme",
);
assert(
  themeSelect.indexOf("Theme.restorePreview()") !== -1 &&
    themeSelect.indexOf("applyNamedTheme") === -1,
  "Current theme restore uses restorePreview, not applyNamedTheme",
);
assert(
  themeSelect.indexOf("Omarchy.setTheme") !== -1 &&
    /onPreviewed:[\s\S]*setTheme/.test(themeSelect) === false,
  "Current theme hover does not call setTheme",
);
assert(
  themeSelect.indexOf("onChanged:") !== -1 && themeSelect.indexOf("Omarchy.setTheme(value)") !== -1,
  "Current theme click still commits through Omarchy.setTheme",
);

const extraSelectStart = appearanceSrc.indexOf('label: "Installed themes"');
const extraBlock = appearanceSrc.slice(extraSelectStart, extraSelectStart + 1800);
assert(
  extraBlock.indexOf("onPreviewed") === -1,
  "Additional themes picker does not subscribe to previewed",
);
