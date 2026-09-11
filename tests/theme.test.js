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

assertEqual(
  theme.previewExtensions().join(","),
  "png,jpg,jpeg,webp,gif,bmp",
  "previewExtensions lists the switcher image types",
);
assertEqual(
  theme.previewCandidates("Tokyo Night", "/home/u")[0],
  "/home/u/.cache/omarchy/theme-selector/previews/tokyo-night.png",
  "previewCandidates points at the native switcher cache",
);
assertEqual(
  theme.previewCandidates("Tokyo Night", "/home/u").length,
  6,
  "previewCandidates covers every image extension",
);
assertEqual(
  theme.previewCandidates("../x", "/home/u").length,
  0,
  "previewCandidates rejects a path slug",
);
assertEqual(theme.previewCandidates("x", "").length, 0, "previewCandidates needs a home");
assertEqual(
  theme
    .swatchList({
      background: "#1a1b26",
      foreground: "#a9b1d6",
      accent: "#7aa2f7",
      muted: "#414868",
      urgent: "#f7768e",
    })
    .join(","),
  "#1a1b26,#a9b1d6,#7aa2f7,#414868,#f7768e",
  "swatchList keeps background-first preview order",
);
assertEqual(
  theme.swatchList({}).join(","),
  "#101315,#cacccc,#cacccc,#707880,#a55555",
  "swatchList falls back to the default palette",
);

const hoverSelectSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSelect.qml"),
  "utf8",
);
assert(
  hoverSelectSrc.indexOf("signal previewed(string value)") !== -1,
  "PrefsSelect reports a hovered option without committing",
);
assert(
  hoverSelectSrc.indexOf("onEntered: root.previewValue(") !== -1,
  "PrefsSelect previews on hover",
);
assert(
  hoverSelectSrc.indexOf("onCurrentIndexChanged: root.previewCurrent()") !== -1,
  "PrefsSelect previews when arrow keys move the highlight",
);
assert(
  hoverSelectSrc.indexOf("Only pickValue() commits") !== -1,
  "PrefsSelect documents that only a pick commits",
);
assert(
  hoverSelectSrc.indexOf("root.previewValue(root.shownValue)") !== -1,
  "PrefsSelect resets the preview to the shown value on close",
);
const hoverAppearanceSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AppearancePage.qml"),
  "utf8",
);
assert(
  hoverAppearanceSrc.indexOf("onPreviewed: function(value) { root.showThemePreview(value) }") !==
    -1,
  "Current theme shows a hover preview instead of applying",
);
assert(
  hoverAppearanceSrc.indexOf('label: "Theme switcher"') !== -1 &&
    hoverAppearanceSrc.indexOf('text: "Open switcher…"') !== -1 &&
    hoverAppearanceSrc.indexOf("Omarchy.openThemeSwitcher()") !== -1,
  "Theme offers the native thumbnail switcher next to the dropdown",
);
assert(
  hoverAppearanceSrc.indexOf("liveApply") === -1 &&
    hoverAppearanceSrc.indexOf("onHighlighted") === -1,
  "Current theme hover never calls setTheme",
);
assert(
  hoverAppearanceSrc.indexOf("Nothing is applied until you click a theme.") === -1,
  "Theme preview keeps no note below the swatches",
);
assert(
  hoverAppearanceSrc.indexOf("Click a theme above to apply it.") !== -1,
  "Theme preview description still tells how to apply",
);
assert(
  hoverAppearanceSrc.indexOf(
    "if (Omarchy.theme.length > 0) root.showThemePreview(Omarchy.theme)",
  ) !== -1 &&
    hoverAppearanceSrc.indexOf(
      "if (root.themePreviewName.length === 0 && Omarchy.theme.length > 0)",
    ) !== -1,
  "Theme preview starts on the current theme",
);
assert(
  hoverAppearanceSrc.indexOf("Omarchy.ensureThemePreviews()") === -1,
  "Appearance never warms previews through the mut queue",
);
assert(
  hoverAppearanceSrc.indexOf("root.themePreviewRequest = name") !== -1 &&
    hoverAppearanceSrc.indexOf("Theme.previewSwatches(") === -1,
  "hover preview resolves the palette async instead of blocking the UI",
);
assert(
  hoverAppearanceSrc.indexOf("property FileView previewColorsFile: FileView") !== -1 &&
    hoverAppearanceSrc.indexOf("watchChanges: false") !== -1,
  "hover preview reads colors through a one-shot FileView",
);
assert(
  hoverAppearanceSrc.indexOf("id: themePreviewWarmer") !== -1 &&
    hoverAppearanceSrc.indexOf("omarchy theme switcher --preload >/dev/null 2>&1 &") !== -1,
  "Appearance warms the native preview cache with an in-page Process",
);
const hoverOmarchySrc = fs.readFileSync(
  path.join(__dirname, "..", "services", "Omarchy.qml"),
  "utf8",
);
assert(
  hoverOmarchySrc.indexOf("ensureThemePreviews") === -1,
  "no queued preview job can bump writeSeq and discard the look snapshot",
);
assert(
  hoverOmarchySrc.indexOf("root.applySnapshot(JSON.stringify({ theme: name }))") !== -1,
  "setTheme shows the new theme immediately instead of waiting for a snapshot",
);
assert(
  hoverOmarchySrc.indexOf("function fireThemeSet(argv)") !== -1 &&
    hoverOmarchySrc.indexOf("property Process themeSetProc: Process") !== -1 &&
    hoverOmarchySrc.indexOf("root.fireThemeSet(cmd.argv)") !== -1,
  "setTheme fires on a dedicated detached process instead of the mut queue",
);
assert(
  hoverOmarchySrc.indexOf("root.pendingPaint = name") !== -1 &&
    hoverOmarchySrc.indexOf("property Timer themePaintTimer: Timer") !== -1,
  "setTheme defers the blocking chrome paint so the label frame renders first",
);
assert(
  hoverOmarchySrc.indexOf("parsed.theme = root.pendingTheme") !== -1,
  "a stale snapshot keeps the pending theme instead of flapping the label back",
);
assert(
  hoverOmarchySrc.indexOf("var confirmed = name === root.pendingTheme") !== -1 &&
    hoverOmarchySrc.indexOf("} else if (confirmed) {") !== -1,
  "the theme.name watcher confirms the pending switch and refreshes the look",
);
assert(
  hoverOmarchySrc.indexOf("property string pendingTheme:") !== -1 &&
    hoverOmarchySrc.indexOf("root.pendingTheme = name") !== -1 &&
    hoverOmarchySrc.indexOf("themeConfirmTimer.restart()") !== -1,
  "setTheme tracks the requested theme for confirmation",
);
assert(
  hoverOmarchySrc.indexOf("function confirmThemeSwitch()") !== -1 &&
    hoverOmarchySrc.indexOf("Theme change to ") !== -1 &&
    hoverOmarchySrc.indexOf("property Timer themeConfirmTimer: Timer") !== -1,
  "an unlanded theme switch surfaces an error and resyncs",
);
assert(
  hoverSelectSrc.indexOf("optionMouse.containsMouse || index === list.currentIndex") !== -1 &&
    hoverSelectSrc.indexOf("root.optionValue(modelData) === root.shownValue") === -1,
  "popup rows highlight hover and position only, never the committed theme",
);
const hoverThemeQml = fs.readFileSync(path.join(__dirname, "..", "services", "Theme.qml"), "utf8");
assert(
  hoverThemeQml.indexOf("function readThemeColors(") !== -1 &&
    hoverThemeQml.indexOf("function previewSwatches(") !== -1 &&
    hoverThemeQml.indexOf("without touching the live") !== -1,
  "Theme reads a preview without touching the live theme",
);
assert(
  hoverThemeQml.indexOf("function colorCandidates(") !== -1 &&
    hoverThemeQml.indexOf("function swatchesFromText(") !== -1 &&
    hoverThemeQml.indexOf("function defaultSwatches(") !== -1,
  "Theme exposes async palette pieces for the hover preview",
);
