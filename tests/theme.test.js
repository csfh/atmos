const { load, assert, assertEqual } = require("./harness");

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
