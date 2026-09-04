const fs = require("fs");
const path = require("path");
const vm = require("vm");

function load(rel) {
  const src = fs.readFileSync(path.join(__dirname, "..", rel), "utf8");
  const ctx = {};
  vm.runInNewContext(src, ctx, { filename: rel });
  return ctx;
}

function assert(condition, description, detail) {
  if (!condition) {
    if (detail) console.error(detail);
    console.error(`not ok - ${description}`);
    process.exit(1);
  }
  console.log(`ok - ${description}`);
}

function assertEqual(actual, expected, description) {
  assert(actual === expected, description, `expected: ${expected}\nactual:   ${actual}`);
}

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

const search = require("../services/SearchIndex.js");
assert(typeof search.setSetting === "undefined", "search index has no setSetting write path");
assert(typeof search.writePrefs === "undefined", "search index has no writePrefs path");
assert(typeof search.savePrefs === "undefined", "search index has no savePrefs path");

const catalog = [
  {
    id: "appearance/font",
    hub: "appearance",
    hubTitle: "Appearance",
    label: "Font",
    description: "The monospace face",
    hint: "omarchy font set",
    keywords: ["theme", "typeface"],
  },
  {
    id: "network/wifi",
    hub: "network",
    hubTitle: "Network",
    label: "Wi-Fi",
    description: "Wireless radio",
    hint: "nmcli",
    keywords: ["ssid"],
  },
];
const index = search.openIndex();
search.ingestRows(index, catalog);
const emptyHits = search.queryRows(index, "");
assert(emptyHits.length === 2, "empty query matches every indexed row");
const fontHits = search.queryRows(index, "font");
assert(
  fontHits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "font matches the theme/font row",
);
assert(
  !fontHits.some(function (row) {
    return row.id === "network/wifi";
  }),
  "font does not match an unrelated network row",
);
const fontUpperHits = search.queryRows(index, "FONT");
assert(
  fontUpperHits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "FONT matches the same lowered haystack",
);
assertEqual(fontUpperHits.length, fontHits.length, "FONT and font return the same hit count");
const networkHits = search.queryRows(index, "network");
assert(
  !networkHits.some(function (row) {
    return row.id === "appearance/font";
  }),
  "network does not match a theme/font row",
);

search.ingestSnapshot(index, { theme: "omarchy", barPosition: "top", nested: { lock: 300 } });
assertEqual(
  search.getState(index, "theme"),
  "omarchy",
  "derived state loads theme from snapshot JSON",
);
assertEqual(
  search.getState(index, "barPosition"),
  "top",
  "derived state loads barPosition from snapshot JSON",
);
assertEqual(
  search.getState(index, "nested.lock"),
  "300",
  "derived state flattens nested snapshot fields",
);
assertEqual(search.getState(index, "missing"), null, "derived state misses unknown keys");

search.ingestSnapshot(index, { theme: "tokyo" });
search.ingestSnapshot(index, { hardware: { cpu: { model: "X" } } });
assertEqual(
  search.getState(index, "theme"),
  "tokyo",
  "second ingest keeps theme from the first patch",
);
assertEqual(
  search.getState(index, "hardware.cpu.model"),
  "X",
  "second ingest adds hardware without a write-prefs path",
);
assertEqual(
  search.indexPath({ ATMOS_SEARCH_INDEX: "/tmp/atmos-search.sqlite" }),
  "/tmp/atmos-search.sqlite",
  "indexPath honors ATMOS_SEARCH_INDEX",
);
assertEqual(
  search.indexPath({ XDG_CACHE_HOME: "/tmp/xdg-cache", HOME: "/tmp/home" }),
  "/tmp/xdg-cache/atmos/search.sqlite",
  "indexPath uses XDG_CACHE_HOME when ATMOS_SEARCH_INDEX is unset",
);
assertEqual(
  search.indexPath({ HOME: "/tmp/home" }),
  "/tmp/home/.cache/atmos/search.sqlite",
  "indexPath falls back to HOME/.cache/atmos/search.sqlite",
);

const cacheDir = fs.mkdtempSync(path.join(require("os").tmpdir(), "atmos-search-"));
const cacheFile = path.join(cacheDir, "nested", "search.sqlite");
const onDisk = search.openIndex(cacheFile);
search.ingestRows(onDisk, catalog);
onDisk.close();
assert(fs.existsSync(cacheFile), "openIndex creates the sqlite file and parent dirs");
const reopened = search.openIndex(cacheFile);
assert(
  search.queryRows(reopened, "font").some(function (row) {
    return row.id === "appearance/font";
  }),
  "reopened sqlite file still matches font",
);
reopened.close();
fs.rmSync(cacheDir, { recursive: true, force: true });

const snapshot = load("services/Snapshot.js");
const snapMerged = snapshot.mergeSnapshot(
  { theme: "omarchy", hardware: { cpu: { model: "X" } } },
  { theme: "tokyo" },
);
assertEqual(snapMerged.theme, "tokyo", "mergeSnapshot updates theme from a partial patch");
assertEqual(
  snapMerged.hardware.cpu.model,
  "X",
  "mergeSnapshot keeps hardware when the patch omits it",
);
assertEqual(
  snapshot.mergeSnapshot({ theme: "omarchy" }, { stayAwake: false }).stayAwake,
  false,
  "mergeSnapshot keeps false",
);
assertEqual(snapshot.parseSnapshot("{"), null, "parseSnapshot rejects junk");

const queue = load("services/WorkQueue.js");
assertEqual(queue.snapshotGroupForHub("appearance"), "look", "appearance hub reads look first");
assertEqual(queue.snapshotGroupForHub("hardware"), "all", "other hubs read the full snapshot");
const io = queue.createWorkQueue();
queue.enqueueRead(io, "look");
queue.enqueueWrite(io, { kind: "mut", argv: ["true"] });
queue.enqueueRead(io, "rest");
const drained = [];
while (true) {
  const job = queue.takeNext(io);
  if (!job) break;
  drained.push(job);
  queue.release(io);
}
assertEqual(drained[0].kind, "mut", "write jobs run before waiting reads");
assertEqual(drained[1].group, "look", "queued look runs before rest");
assertEqual(drained[2].group, "rest", "rest stays last when look was already queued");
assert(queue.isIdle(io), "worker is idle when both queues are empty");
queue.enqueueRead(io, "look");
queue.enqueueRead(io, "look");
assertEqual(io.reads.length, 1, "duplicate look reads coalesce");
queue.enqueueRead(io, "all");
assertEqual(
  io.reads[io.reads.length - 1].group,
  "all",
  "all keeps a pending look then replaces rest",
);
assert(
  io.reads.some(function (job) {
    return job.group === "look";
  }),
  "all does not drop a waiting look",
);

const brightIo = queue.createWorkQueue();
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "brightness:DP-1",
  argv: ["omarchy", "brightness", "40%"],
});
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "brightness:DP-1",
  argv: ["omarchy", "brightness", "80%"],
});
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "brightness:HDMI-A-1",
  argv: ["omarchy", "brightness", "10%"],
});
assertEqual(brightIo.writes.length, 2, "same-key brightness writes coalesce");
assertEqual(brightIo.writes[0].argv[2], "80%", "coalesced write keeps the last percent");
assertEqual(brightIo.writes[1].key, "brightness:HDMI-A-1", "different monitor keys stay separate");
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "theme",
  argv: ["omarchy", "theme", "set", "omarchy"],
  apply: { theme: "omarchy" },
});
queue.enqueueWrite(brightIo, {
  kind: "mut",
  key: "theme",
  argv: ["omarchy", "theme", "set", "tokyo"],
  apply: { theme: "tokyo" },
});
assertEqual(brightIo.writes.length, 3, "theme writes coalesce separately from brightness");
assertEqual(brightIo.writes[2].apply.theme, "tokyo", "coalesced theme write keeps the last name");
queue.enqueueWrite(brightIo, {
  kind: "job",
  key: "wifi-join",
  argv: ["join", "cafe"],
});
queue.enqueueWrite(brightIo, {
  kind: "job",
  key: "wifi-join",
  argv: ["join", "home"],
});
assertEqual(brightIo.writes.length, 4, "same-kind jobs coalesce");
assertEqual(brightIo.writes[3].argv[1], "home", "coalesced job keeps the last argv");

const staleIo = queue.createWorkQueue();
queue.enqueueRead(staleIo, "all");
const staleRead = queue.takeNext(staleIo);
assert(staleRead && staleRead.kind === "read", "stale-read test starts a snapshot job");
queue.enqueueWrite(staleIo, { kind: "mut", key: "brightness:DP-1", argv: ["x"] });
assert(
  !queue.shouldApplyRead(staleRead, staleIo),
  "snapshot started before a later write is not applied",
);
queue.release(staleIo);
const lateWrite = queue.takeNext(staleIo);
assertEqual(lateWrite.key, "brightness:DP-1", "write runs after the discarded snapshot");
queue.release(staleIo);
assert(queue.isIdle(staleIo), "queue is idle after discarding a stale read and running the write");

const patched = snapshot.patchMonitorBrightness(
  [
    { name: "DP-1", brightness: 40 },
    { name: "HDMI-A-1", brightness: 10 },
  ],
  "DP-1",
  80,
);
assertEqual(patched[0].brightness, 80, "patchMonitorBrightness updates the named monitor");
assertEqual(patched[1].brightness, 10, "patchMonitorBrightness leaves other monitors");

const scaled = snapshot.patchFocusedMonitorScale(
  [
    { name: "DP-1", focused: false, scale: 1 },
    { name: "eDP-1", focused: true, scale: 1 },
  ],
  1.25,
);
assertEqual(scaled[1].scale, 1.25, "patchFocusedMonitorScale updates the focused monitor");
assertEqual(scaled[0].scale, 1, "patchFocusedMonitorScale leaves other monitors");

const plugged = snapshot.patchPluginEnabled(
  [
    { id: "omarchy.clock", enabled: true },
    { id: "omarchy.weather", enabled: false },
  ],
  "omarchy.weather",
  true,
);
assertEqual(plugged[1].enabled, true, "patchPluginEnabled enables the named plugin");
assertEqual(plugged[0].enabled, true, "patchPluginEnabled leaves other plugins");

const extras = snapshot.patchRemoveMatching(["tokyo", "omarchy", "catppuccin"], "", "omarchy");
assertEqual(extras.join(","), "tokyo,catppuccin", "patchRemoveMatching drops a string extra theme");
const apps = snapshot.patchRemoveMatching(
  [
    { id: "foo", name: "Foo" },
    { id: "bar", name: "Bar" },
  ],
  "id",
  "foo",
);
assertEqual(apps.length, 1, "patchRemoveMatching drops a launcher by id");
assertEqual(apps[0].id, "bar", "patchRemoveMatching keeps the other launcher");

const btPatched = snapshot.patchRowField(
  [
    { address: "AA:BB", connected: false },
    { address: "CC:DD", connected: true },
  ],
  "address",
  "AA:BB",
  "connected",
  true,
);
assertEqual(btPatched[0].connected, true, "patchRowField updates the matched bluetooth row");
assertEqual(btPatched[1].connected, true, "patchRowField leaves other bluetooth rows");

const wifiPatched = snapshot.patchWifiActive(
  [
    { uuid: "11111111-1111-1111-1111-111111111111", name: "Home", active: true },
    { uuid: "22222222-2222-2222-2222-222222222222", name: "Cafe", active: false },
  ],
  "22222222-2222-2222-2222-222222222222",
  true,
);
assertEqual(wifiPatched[1].active, true, "patchWifiActive marks the named connection active");
assertEqual(wifiPatched[0].active, false, "patchWifiActive clears the previous active connection");

const hooked = snapshot.patchHookSample(
  [{ path: "/tmp/hooks/theme-set.d/notify.sample", name: "notify.sample", sample: true }],
  "/tmp/hooks/theme-set.d/notify.sample",
  true,
);
assertEqual(hooked[0].sample, false, "patchHookSample enables a sample hook");
assertEqual(hooked[0].name, "notify", "patchHookSample drops .sample from the name");

assertEqual(
  snapshot.patchKeyboardBrightness(40, "up"),
  50,
  "patchKeyboardBrightness steps up by 10",
);
assertEqual(
  snapshot.patchKeyboardBrightness(40, "down"),
  30,
  "patchKeyboardBrightness steps down by 10",
);
assertEqual(snapshot.patchKeyboardBrightness(40, "off"), 0, "patchKeyboardBrightness off is 0");
assertEqual(snapshot.patchKeyboardBrightness(96, "up"), 100, "patchKeyboardBrightness caps at 100");
assertEqual(snapshot.patchKeyboardBrightness(4, "down"), 0, "patchKeyboardBrightness floors at 0");
assertEqual(snapshot.patchKeyboardBrightness(0, "down"), 0, "patchKeyboardBrightness stays off");

const reminded = snapshot.patchAppendReminder([{ label: "Stand", minutes: 10 }], 5, "Tea");
assertEqual(reminded.length, 2, "patchAppendReminder appends a reminder");
assertEqual(reminded[1].message, "Tea", "patchAppendReminder keeps the message");
assertEqual(reminded[1].minutes, 5, "patchAppendReminder keeps the delay");
assertEqual(
  snapshot.patchAppendReminder([], 0, "nope").length,
  0,
  "patchAppendReminder rejects a zero delay",
);

const hookedAppend = snapshot.patchAppendHook(
  [{ path: "/tmp/hooks/theme-set.d/old.sh", name: "old.sh", type: "theme-set" }],
  {
    path: "/tmp/hooks/theme-set.d/notify.sh",
    name: "notify.sh",
    type: "theme-set",
    sample: false,
    flat: false,
  },
);
assertEqual(hookedAppend.length, 2, "patchAppendHook appends a new hook");
assertEqual(hookedAppend[1].name, "notify.sh", "patchAppendHook keeps the new hook name");
const upserted = snapshot.patchAppendHook(hookedAppend, {
  path: "/tmp/hooks/theme-set.d/old.sh",
  name: "old.sh",
  type: "theme-set",
  sample: true,
});
assertEqual(upserted.length, 2, "patchAppendHook upserts an existing hook path");
assertEqual(upserted[0].sample, true, "patchAppendHook updates the matched hook");

const replacedAuto = snapshot.patchReplaceManaged(
  [
    { command: "waybar", managed: false },
    { command: "mako", managed: true },
  ],
  ["hyprsunset"],
);
assertEqual(replacedAuto.length, 2, "patchReplaceManaged keeps unmanaged autostart rows");
assertEqual(replacedAuto[0].command, "waybar", "patchReplaceManaged keeps the unmanaged command");
assertEqual(replacedAuto[0].managed, false, "patchReplaceManaged leaves unmanaged rows unmanaged");
assertEqual(replacedAuto[1].command, "hyprsunset", "patchReplaceManaged appends managed commands");
assertEqual(replacedAuto[1].managed, true, "patchReplaceManaged marks replacement rows managed");
const replacedBinds = snapshot.patchReplaceManaged(
  [
    { keys: "SUPER + D", command: "desks", managed: false },
    { keys: "SUPER + F", command: "nautilus", managed: true },
  ],
  [{ keys: "SUPER + Q", command: "kill", unbind: true }],
);
assertEqual(replacedBinds.length, 2, "patchReplaceManaged keeps unmanaged bindings");
assertEqual(replacedBinds[0].keys, "SUPER + D", "patchReplaceManaged keeps the unmanaged chord");
assertEqual(replacedBinds[1].keys, "SUPER + Q", "patchReplaceManaged appends the managed bind");
assertEqual(replacedBinds[1].unbind, true, "patchReplaceManaged keeps bind fields");
assertEqual(replacedBinds[1].managed, true, "patchReplaceManaged marks replacement binds managed");

const qmlCatalog = search.catalogFromQmlDir(path.join(__dirname, "..", "pages"));
assert(
  qmlCatalog.some(function (row) {
    return row.label === "Font" && row.hub === "appearance";
  }),
  "qml catalog indexes the Font row",
);

const remix = load("services/RemixIcons.js");
assertEqual(remix.viewBoxSize(), 24, "remix viewBox is 24");
assertEqual(remix.pathFor(""), "", "pathFor misses an empty name");
assertEqual(remix.pathFor("nope"), "", "pathFor misses an unknown name");
const remixDir = path.join(__dirname, "..", "icons");
const remixFiles = fs.readdirSync(remixDir).filter(function (name) {
  return name.endsWith(".svg");
});
assert(remixFiles.length >= 24, "remix icons include hub glyphs");
remixFiles.forEach(function (file) {
  const name = file.replace(/\.svg$/, "");
  const svg = fs.readFileSync(path.join(remixDir, file), "utf8");
  assertEqual(remix.pathFor(name), remix.pathFromSvg(svg), "pathFor matches icons/" + file);
});

const ui = load("services/RichUi.js");
const qr = ui.parseQrOutput("meta\twlan0\tWPA\tCafe\n0110\n1001\n");
assert(qr.ok === true, "parseQrOutput accepts a meta header and matrix");
assertEqual(qr.ssid, "Cafe", "parseQrOutput reads ssid");
assertEqual(qr.size, 4, "parseQrOutput matrix width");
assertEqual(qr.rows[1][0], 1, "parseQrOutput cell values");

assertEqual(ui.parseMbpsLine("12.3"), 12.3, "parseMbpsLine reads a rate");
assert(isNaN(ui.parseMbpsLine("nope")), "parseMbpsLine rejects junk");
assertEqual(ui.formatMbps("12.3"), "12", "formatMbps rounds a fast rate");
assertEqual(ui.formatMbps("9.14"), "9.1", "formatMbps keeps one decimal under 10");
assertEqual(ui.formatMbps("0"), "0.0", "formatMbps formats zero");
assertEqual(ui.formatMbps(""), "", "formatMbps empty");
assertEqual(ui.formatMbps("-1"), "", "formatMbps rejects a negative rate");
assertEqual(ui.mbpsLabel("42"), "42 Mbps", "mbpsLabel appends the unit");
assertEqual(
  ui.mbpsCopyText("Download", "42.2"),
  "Download 42 Mbps",
  "mbpsCopyText prefixes the direction",
);
assertEqual(ui.mbpsCopyText("Upload", ""), "", "mbpsCopyText empty without a sample");

assertEqual(
  ui.formatMonitorMode("6016x3384@59.99Hz"),
  "6016×3384 @ 60 Hz",
  "formatMonitorMode rounds Hz",
);
assertEqual(ui.parseMonitorMode("2560x1440@144.00Hz").width, 2560, "parseMonitorMode width");
assertEqual(
  ui.parseMonitorMode("2560 × 1440 @ 144 Hz").refresh,
  144,
  "parseMonitorMode spaced form",
);
assertEqual(ui.parseMonitorMode("nope"), null, "parseMonitorMode rejects junk");
assertEqual(
  ui.currentMonitorModeValue({
    width: 2560,
    height: 1440,
    refresh: 144,
    availableModes: ["1920x1080@60.00Hz", "2560x1440@144.00Hz"],
  }),
  "2560x1440@144.00Hz",
  "currentMonitorModeValue matches availableModes",
);
assertEqual(
  ui.monitorModeOptions({ width: 100, height: 100, refresh: 60, availableModes: [] }).length,
  1,
  "monitorModeOptions synthesizes the current mode",
);
assertEqual(
  ui.monitorModeCopyText({
    name: "DP-1",
    width: 1920,
    height: 1080,
    refresh: 60,
    availableModes: ["1920x1080@60.00Hz"],
  }),
  "DP-1 1920×1080 @ 60 Hz",
  "monitorModeCopyText includes the output name",
);

const diskLine = ui.parseDiskSpeedLine("read 450");
assertEqual(diskLine.kind, "read", "parseDiskSpeedLine kind");
assertEqual(diskLine.value, 450, "parseDiskSpeedLine value");
assertEqual(
  ui.parseDiskSpeedLine("disk WD Black").value,
  "WD Black",
  "parseDiskSpeedLine disk name",
);

const snaps = ui.parseSnapperList(
  '{"root":[{"number":3,"date":"2026-01-01","description":"pre"}]}',
);
assertEqual(snaps[0].id, 3, "parseSnapperList id");
assertEqual(snaps[0].config, "root", "parseSnapperList config");
const snapArr = ui.parseSnapperList(
  '[{"id":2,"dateIso":"2026-02-01","userdata":"post"},{"number":9,"date":"2026-03-01","description":"pre"},{"number":0}]',
);
assertEqual(snapArr.length, 2, "parseSnapperList array form skips id 0");
assertEqual(snapArr[0].id, 2, "parseSnapperList array form keeps input order");
assertEqual(snapArr[0].date, "2026-02-01", "parseSnapperList reads dateIso");
assertEqual(snapArr[0].description, "post", "parseSnapperList falls back to userdata");
assertEqual(snapArr[1].id, 9, "parseSnapperList array form reads number");
const snapKeyed = ui.parseSnapperList('{"root":[{"number":2},{"number":9}]}');
assertEqual(snapKeyed[0].id, 9, "parseSnapperList keyed form sorts by id descending");
assertEqual(ui.parseSnapperList("not json").length, 0, "parseSnapperList rejects junk JSON");
assertEqual(ui.parseQrOutput("011\n01\n").ok, false, "parseQrOutput rejects uneven rows");
assertEqual(
  ui.parseQrOutput("meta\twlan0\tWPA\tCafe\n").ok,
  false,
  "parseQrOutput rejects a header with no matrix",
);
assert(isNaN(ui.parseMbpsLine("-1")), "parseMbpsLine rejects a negative rate");
assertEqual(ui.parseDiskSpeedLine("read -4"), null, "parseDiskSpeedLine rejects a negative rate");
assertEqual(ui.parseDiskSpeedLine("other 1"), null, "parseDiskSpeedLine rejects an unknown kind");

const reminders = ui.parseReminders(
  '{"reminders":[{"unit":"m","label":"Tea","message":"Tea is ready","remaining":"2m","atTime":"10:00","minutes":2},null]}',
);
assertEqual(reminders.length, 1, "parseReminders skips null items");
assertEqual(reminders[0].label, "Tea", "parseReminders reads label");
assertEqual(
  ui.reminderCopyText(reminders[0]),
  "Tea — in 2m · at 10:00 · Tea is ready",
  "reminderCopyText includes remaining and time",
);
assertEqual(ui.reminderCopyText(null), "", "reminderCopyText empty without a row");
assertEqual(
  ui.bindingCopyText({ keys: "SUPER + Q", action: "Close window" }),
  "SUPER + Q — Close window",
  "bindingCopyText joins chord and action",
);
assertEqual(ui.bindingCopyText({ keys: "SUPER + F" }), "SUPER + F", "bindingCopyText keys only");
assertEqual(reminders[0].minutes, 2, "parseReminders reads minutes");
assertEqual(
  ui.parseReminders('{"reminders":[{"message":"Ping"}]}')[0].label,
  "Ping",
  "parseReminders falls back to message for label",
);
assertEqual(ui.parseReminders("not json").length, 0, "parseReminders rejects junk JSON");
assertEqual(ui.parseReminders("{}").length, 0, "parseReminders empty object");
assertEqual(
  ui.filterOptions(
    [
      { label: "Wi-Fi", value: "wifi" },
      { label: "Theme", value: "theme" },
    ],
    "wi",
  ).length,
  1,
  "filterOptions matches a label",
);
assertEqual(
  ui.filterOptions(
    [
      { label: "Wi-Fi", value: "wlan" },
      { label: "Theme", value: "theme" },
    ],
    "wlan",
  ).length,
  1,
  "filterOptions matches a value",
);
assertEqual(
  ui.filterOptions(["alpha", "beta"], "BE")[0],
  "beta",
  "filterOptions matches a string option case-insensitively",
);
assertEqual(
  ui.filterOptions(["alpha", "beta"], "").length,
  2,
  "filterOptions empty query keeps all",
);

assertEqual(ui.sliderValueUnit("42%"), "%", "sliderValueUnit keeps a percent sign");
assertEqual(ui.sliderValueUnit("12 px"), " px", "sliderValueUnit keeps a px suffix");
assertEqual(ui.sliderValueUnit("0.50"), "", "sliderValueUnit is empty without a suffix");
assertEqual(ui.sliderValueUnit(""), "", "sliderValueUnit empty valueText");
assertEqual(ui.formatSliderNumber(40), "40", "formatSliderNumber rounds a whole number");
assertEqual(ui.formatSliderNumber(0.5), "0.5", "formatSliderNumber keeps two-decimal tenths");
assertEqual(
  ui.formatSliderCaption(40, "12%", ""),
  "40%",
  "formatSliderCaption appends the percent from valueText",
);
assertEqual(
  ui.formatSliderCaption(8, "12 px", ""),
  "8 px",
  "formatSliderCaption appends a px suffix from valueText",
);
assertEqual(
  ui.formatSliderCaption(0.5, "12%", "50%"),
  "50%",
  "formatSliderCaption prefers an explicit tick string",
);

const live = ui.sliderLiveState(100);
const firstLive = ui.sliderLivePush(live, 1000, 10);
assertEqual(firstLive.emit, 10, "first live write emits immediately");
assertEqual(firstLive.delayMs, 0, "first live write does not schedule a wait");
const withinLive = ui.sliderLivePush(live, 1040, 20);
assertEqual(withinLive.emit, undefined, "live write skips a move inside 100ms");
assertEqual(withinLive.delayMs, 60, "live write waits the remaining 100ms window");
const laterLive = ui.sliderLivePush(live, 1060, 40);
assertEqual(laterLive.emit, undefined, "live write keeps coalescing inside the window");
assertEqual(laterLive.delayMs, 40, "live write keeps the original window");
const trailingLive = ui.sliderLiveTake(live, 1100);
assertEqual(trailingLive.emit, 40, "live write trailing emit is the latest value");
const sameFlush = ui.sliderLiveFlush(live, 1110, 40);
assertEqual(sameFlush.emit, undefined, "live write flush skips the last sent value");
const releaseFlush = ui.sliderLiveFlush(live, 1110, 55);
assertEqual(releaseFlush.emit, 55, "live write flush emits the release value");

const dns = ui.parseDnsServers("1.1.1.1 8.8.8.8");
assert(dns.ok === true, "parseDnsServers accepts IPv4");
assertEqual(dns.servers.length, 2, "parseDnsServers token count");
assert(ui.parseDnsServers("--bad").ok === false, "parseDnsServers rejects flags");
assert(ui.parseDnsServers("nope").ok === false, "parseDnsServers rejects hostnames");
assert(ui.parseDnsServers("2001:db8::1").ok === true, "parseDnsServers accepts compressed IPv6");
assert(ui.parseDnsServers("::").ok === false, "parseDnsServers rejects unspecified IPv6");
assertEqual(
  ui.formatDnsInput("1.1.1.1,8.8.8.8"),
  "1.1.1.1 8.8.8.8",
  "formatDnsInput turns commas into spaces",
);
assertEqual(
  ui.formatDnsInput("1921680"),
  "192.168.0",
  "formatDnsInput splits IPv4 octets as they overflow",
);
assertEqual(
  ui.formatDnsInput("1111"),
  "111.1",
  "formatDnsInput starts a new octet after three digits",
);
assertEqual(ui.formatDnsInput("2001:DB8::1"), "2001:db8::1", "formatDnsInput lowercases IPv6");
assert(ui.dnsInputStatus("1.1").error === "", "dnsInputStatus allows a partial IPv4");
assert(
  ui.dnsInputStatus("1:2:3:4:5:6:7:8:9").error.indexOf("Not an IPv4") === 0,
  "dnsInputStatus rejects an overlong IPv6",
);
assert(
  ui.dnsInputStatus("1..2.3").error.indexOf("Not an IPv4") === 0,
  "dnsInputStatus rejects a double-dot IPv4",
);
assert(ui.dnsInputStatus("1.1.1.1 8.8.8.8").ok === true, "dnsInputStatus accepts two IPv4 servers");
assert(ui.isPartialIpv6("2001:db8:") === true, "isPartialIpv6 allows a trailing colon");
assert(ui.isIpv6("fe80::1") === true, "isIpv6 accepts link-local");

assertEqual(ui.parseLauncherName("Notes"), "Notes", "parseLauncherName accepts a name");
assertEqual(ui.parseLauncherName("  Notes  "), "Notes", "parseLauncherName trims");
assertEqual(ui.parseLauncherName("foo/bar"), "", "parseLauncherName rejects a slash");
assertEqual(ui.parseLauncherName("-secret"), "", "parseLauncherName rejects a leading hyphen");
assertEqual(ui.parseWebAppUrl("example.com"), "https://example.com", "parseWebAppUrl adds https");
assertEqual(ui.parseWebAppUrl("https://hey.com"), "https://hey.com", "parseWebAppUrl keeps https");
assertEqual(
  ui.parseWebAppUrl("javascript:alert(1)"),
  "",
  "parseWebAppUrl rejects a non-http scheme",
);
assertEqual(ui.parseWebAppUrl("http://hey.com"), "http://hey.com", "parseWebAppUrl keeps http");
assertEqual(ui.parseWebAppUrl("hey com"), "", "parseWebAppUrl rejects spaces");
assertEqual(ui.parseWebAppUrl(""), "", "parseWebAppUrl empty");
assert(ui.isTuiWindowStyle("float") === true, "isTuiWindowStyle accepts float");
assert(ui.isTuiWindowStyle("tile") === true, "isTuiWindowStyle accepts tile");
assert(ui.isTuiWindowStyle("stack") === false, "isTuiWindowStyle rejects other values");

assertEqual(ui.usagePercent(50, 100), 50, "usagePercent");
assertEqual(ui.usagePercent(50, 0), 0, "usagePercent zero size");
assertEqual(ui.usagePercent(-10, 100), 0, "usagePercent clamps below 0");
assertEqual(ui.usagePercent(200, 100), 100, "usagePercent clamps above 100");
assertEqual(ui.formatBytes(0), "0 B", "formatBytes zero");
assertEqual(ui.formatBytes(-4), "0 B", "formatBytes negative");
assertEqual(ui.formatBytes(1024), "1.00 KB", "formatBytes one kilobyte");
assertEqual(ui.formatBytes(1536), "1.50 KB", "formatBytes fractional kilobyte");
assertEqual(ui.formatBytes(10485760), "10.0 MB", "formatBytes one-decimal megabytes");
assertEqual(ui.pathFromUrl("file:///home/a/b.png"), "/home/a/b.png", "pathFromUrl strips file://");
assertEqual(
  ui.pathFromUrl("file:///C:/Users/a/b.png"),
  "C:/Users/a/b.png",
  "pathFromUrl strips a Windows file:// drive prefix",
);
assertEqual(
  ui.pathFromUrl("file:///home/a/my%20wall.png"),
  "/home/a/my wall.png",
  "pathFromUrl decodes percent-escapes",
);
assertEqual(ui.fileBasename("/home/a/wall.png"), "wall.png", "fileBasename last path segment");
assertEqual(ui.fileBasename("plain"), "plain", "fileBasename no slash");
assertEqual(ui.fileBasename(""), "", "fileBasename empty");

const hw = load("services/Hardware.js");
assertEqual(hw.parseDmiField(" Framework "), "Framework", "parseDmiField trims");
assertEqual(hw.parseDmiField("To be filled by O.E.M."), "", "parseDmiField drops OEM filler");
assertEqual(hw.parseDmiField("none"), "", "parseDmiField drops none");
assertEqual(
  hw.parseDmiField("Desktop (AMD Ryzen AI Max 300 Series)"),
  "Desktop (AMD Ryzen AI Max 300 Series)",
  "parseDmiField keeps a product name",
);
assertEqual(hw.parseDmiField("evil\nname"), "", "parseDmiField rejects a newline");
assertEqual(hw.parseSystemStats("cpu1%\nmemory27.5GB / 125GB").cpu, "1%", "parseSystemStats cpu");
assertEqual(
  hw.parseSystemStats("cpu1%\nmemory27.5GB / 125GB").memory,
  "27.5GB / 125GB",
  "parseSystemStats memory",
);
assertEqual(
  hw.npuSummary({ npuIdentity: "AMD Strix Halo NPU" }),
  "AMD Strix Halo NPU",
  "npuSummary present",
);
assertEqual(hw.npuSummary({}), "", "npuSummary absent");
assertEqual(
  hw.machineSummary({ name: "Desktop", vendor: "Framework" }).indexOf("Framework") !== -1,
  true,
  "machineSummary names the vendor",
);
assert(
  hw.cpuSummary({ model: "Ryzen 7", cores: 8, threads: 16, maxMhz: 5000.4 }).indexOf("8 cores") !==
    -1,
  "cpuSummary names cores",
);
assert(
  hw.cpuSummary({ model: "Ryzen 7", cores: 8, threads: 16 }).indexOf("16 threads") !== -1,
  "cpuSummary names extra threads",
);
assert(
  hw.biosSummary({ vendor: "AMI", version: "1.2", uefi: true }).indexOf("UEFI") !== -1,
  "biosSummary names UEFI",
);
assertEqual(hw.thermalSummary({ temp: 42.26 }), "42.3 °C", "thermalSummary rounds tenths");
assertEqual(hw.thermalSummary({}), "", "thermalSummary empty without a temp");
assertEqual(
  hw.tpmSummary({ present: true, version: "2.0" }).indexOf("TPM 2.0") !== -1,
  true,
  "tpmSummary names the version",
);
assertEqual(hw.tpmSummary({ present: false }), "", "tpmSummary empty when absent");

const cpuPresent = "processor\t: 0\nmodel name\t: AMD RYZEN AI MAX+ 395 w/ Radeon 8060S\n";
assertEqual(
  hw.parseCpuIdentity(cpuPresent),
  "AMD RYZEN AI MAX+ 395 w/ Radeon 8060S",
  "parseCpuIdentity present",
);
assertEqual(
  hw.parseCpuIdentity("processor\t: 0\nvendor_id\t: AuthenticAMD\n"),
  "",
  "parseCpuIdentity absent",
);
assertEqual(hw.parseCpuIdentity("model name\t: --nproc\n"), "", "parseCpuIdentity rejects a flag");
assertEqual(
  hw.parseCpuIdentity("Hardware\t: Raspberry Pi 5\n"),
  "Raspberry Pi 5",
  "parseCpuIdentity ARM Hardware field",
);

const gpuLine =
  "c3:00.0 Display controller [0380]: Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics] [1002:1586] (rev c1)\n";
const npuLine =
  "c4:00.1 Signal processing controller [1180]: Advanced Micro Devices, Inc. [AMD] Strix/Krackan/Strix Halo Neural Processing Unit [1022:17f0] (rev 11)\n";
const ethLine =
  "c1:00.0 Ethernet controller [0200]: Intel Corporation Ethernet Controller [8086:125c]\n";
assertEqual(
  hw.parseGpuIdentity(gpuLine + npuLine),
  "Advanced Micro Devices, Inc. [AMD/ATI] Strix Halo [Radeon Graphics / Radeon 8050S Graphics / Radeon 8060S Graphics]",
  "parseGpuIdentity present",
);
assertEqual(hw.parseGpuIdentity(ethLine + npuLine), "", "parseGpuIdentity absent");
assertEqual(
  hw.parseGpuIdentity("00:00.0 VGA compatible controller: ../../etc/passwd [1002:0000]\n"),
  "",
  "parseGpuIdentity rejects a path",
);
assertEqual(
  hw.parseNpuIdentity(gpuLine + npuLine),
  "Advanced Micro Devices, Inc. [AMD] Strix/Krackan/Strix Halo Neural Processing Unit",
  "parseNpuIdentity present",
);
assertEqual(hw.parseNpuIdentity(gpuLine + ethLine), "", "parseNpuIdentity absent");
assertEqual(
  hw.parseNpuIdentity("00:00.0 Neural Processing Unit: -inject\n"),
  "",
  "parseNpuIdentity rejects a flag",
);
assertEqual(hw.parseHwIdentity("evil\nname"), "", "parseHwIdentity rejects a newline");

const mem = hw.parseMeminfo(
  "MemTotal:       16398384 kB\nMemAvailable:    8000000 kB\nMemFree:         1000000 kB\nSwapTotal:             0 kB\nSwapFree:              0 kB\n",
);
assertEqual(mem.total, 16398384 * 1024, "parseMeminfo total bytes");
const cpuinfo = hw.parseCpuinfo(
  "processor: 0\nvendor_id: GenuineIntel\nmodel name: Intel(R) Xeon(R) Processor\ncpu cores: 4\nphysical id: 0\nflags: vmx avx2\n\nprocessor: 1\nvendor_id: GenuineIntel\nmodel name: Intel(R) Xeon(R) Processor\ncpu cores: 4\nphysical id: 0\n",
);
assertEqual(cpuinfo.model, "Intel(R) Xeon(R) Processor", "parseCpuinfo reads the model");
assertEqual(cpuinfo.cores, 4, "parseCpuinfo uses cpu cores per socket");
const lscpu = hw.parseLscpu(
  "Architecture: x86_64\nCPU(s): 16\nOn-line CPU(s) list: 0-15\nVendor ID: AuthenticAMD\nModel name: AMD Ryzen 7\nThread(s) per core: 2\nCore(s) per socket: 8\nSocket(s): 1\nCPU MHz: 3800.000\nCPU max MHz: 5000.000\nL3 cache: 32 MiB\nFlags: fpu vme sse\nVirtualization: AMD-V\nHypervisor vendor: KVM\n",
);
assertEqual(lscpu.model, "AMD Ryzen 7", "parseLscpu reads the model");
assertEqual(lscpu.cores, 8, "parseLscpu multiplies cores per socket");
assertEqual(lscpu.threads, 16, "parseLscpu reads CPU(s) as threads");
assertEqual(lscpu.mhz, 3800, "parseLscpu reads CPU MHz");
assertEqual(lscpu.maxMhz, 5000, "parseLscpu reads CPU max MHz");
assert(lscpu.caches.indexOf("L3 32 MiB") !== -1, "parseLscpu formats L3 cache");
assertEqual(lscpu.flags.join(" "), "fpu vme sse", "parseLscpu splits flags");
assertEqual(lscpu.virtualization, "AMD-V", "parseLscpu reads virtualization");
assertEqual(lscpu.hypervisor, "KVM", "parseLscpu reads hypervisor vendor");
assertEqual(mem.available, 8000000 * 1024, "parseMeminfo available bytes");
assertEqual(mem.used, (16398384 - 8000000) * 1024, "parseMeminfo used bytes");
const memFallback = hw.parseMeminfo(
  "MemTotal: 4096 kB\nMemFree: 1000 kB\nCached: 500 kB\nBuffers: 200 kB\nSwapTotal: 1024 kB\nSwapFree: 24 kB\n",
);
assertEqual(
  memFallback.available,
  (1000 + 500 + 200) * 1024,
  "parseMeminfo falls back to free+cached+buffers",
);
assertEqual(memFallback.swapUsed, 1000 * 1024, "parseMeminfo swap used");
const pci = hw.parseLspci(
  '00:00.0 "Host bridge [0600]" "Intel Corporation [8086]" "Raptor Lake-P Host Bridge [a74f]"\n00:02.0 "VGA compatible controller [0300]" "Intel Corporation [8086]" "Raptor Lake-P Integrated Graphics [a7a0]"\n',
);
assertEqual(
  hw.pickChipset(pci).name.indexOf("Raptor") !== -1,
  true,
  "pickChipset finds the host bridge",
);
assertEqual(hw.pickGpus(pci).length, 1, "pickGpus finds the VGA device");
assertEqual(hw.memoryTypeName(26), "DDR4", "memoryTypeName maps DDR4");
assertEqual(hw.chassisTypeName(9), "Laptop", "chassisTypeName maps laptop");
assertEqual(
  hw.formatModuleSize(16 * 1024 * 1024 * 1024),
  "16 GB",
  "formatModuleSize uses whole gigabytes",
);
assertEqual(
  hw.formatModuleSize(1536 * 1024 * 1024),
  "1.5 GB",
  "formatModuleSize fractional gigabytes",
);
assertEqual(hw.formatModuleSize(512 * 1024 * 1024), "512 MB", "formatModuleSize megabytes");
assertEqual(hw.formatModuleSize(0), "", "formatModuleSize empty for zero");
assert(
  hw.virtSummary({ hypervisor: "KVM", kvm: true }).indexOf("Running on KVM") !== -1,
  "virtSummary names the hypervisor",
);
assert(
  hw.virtSummary({ guest: true }).indexOf("guest") !== -1,
  "virtSummary guest without hypervisor",
);
assertEqual(hw.virtSummary({}), "", "virtSummary empty");
assertEqual(
  hw.notableFlags(["vmx", "avx2", "nope"]).join(","),
  "VT-x,AVX2",
  "notableFlags maps known CPU flags",
);
assertEqual(hw.notableFlags(["svm", "SVM"]).join(","), "AMD-V", "notableFlags dedupes labels");
assert(
  hw.batterySummary({ status: "Charging", capacity: 80, technology: "Li-ion" }).indexOf("80%") !==
    -1,
  "batterySummary names capacity",
);
assertEqual(
  hw
    .boardSummary({ vendor: "Framework", name: "Mainboard", version: "A7" })
    .indexOf("Version A7") !== -1,
  true,
  "boardSummary names the version",
);
assert(
  hw
    .gpuSummary({ vendor: "AMD", name: "Strix Halo", driver: "amdgpu", pciId: "1002:1586" })
    .indexOf("Driver amdgpu") !== -1,
  "gpuSummary names the driver",
);
assert(
  hw
    .nicSummary({ iface: "wlan0", name: "Wi-Fi", wireless: true, speed: 1200 })
    .indexOf("Wireless") !== -1,
  "nicSummary marks wireless",
);
assert(
  hw.moduleSummary({ size: 16 * 1024 * 1024 * 1024, typeCode: 26, speed: 5600 }).indexOf("DDR4") !==
    -1,
  "moduleSummary maps typeCode",
);
assertEqual(
  hw.pickNpus([
    { className: "Signal processing controller", name: "Strix Halo Neural Processing Unit" },
  ]).length,
  1,
  "pickNpus matches neural processing",
);
assertEqual(
  hw.pickNpus([{ className: "Ethernet controller", name: "I225" }]).length,
  0,
  "pickNpus skips NICs",
);
assert(
  hw
    .chipsetSummary({ vendor: "Intel", name: "Raptor", role: "Host bridge", southbridge: "LPC" })
    .indexOf("Southbridge LPC") !== -1,
  "chipsetSummary names the southbridge",
);
assertEqual(hw.parseKeyValues("A: 1\nB = 2", "=").B, "2", "parseKeyValues uses a custom delimiter");
const hwNorm = hw.normalize({
  machine: { chassis: 9, vendor: "Framework", name: "Laptop" },
  gpus: [{ name: "" }],
  nics: [{ iface: "wlan0" }],
});
assertEqual(hwNorm.machine.chassis, "Laptop", "normalize maps a chassis type code");
assertEqual(hwNorm.gpus.length, 0, "normalize drops a nameless GPU");
assertEqual(hwNorm.nics[0].iface, "wlan0", "normalize keeps a NIC iface");
assertEqual(hw.cleanText("To Be Filled By O.E.M."), "", "cleanText drops OEM filler");
assertEqual(hw.memoryFormName(8), "DIMM", "memoryFormName maps DIMM");
assertEqual(hw.memoryFormName(13), "SODIMM", "memoryFormName maps SODIMM");
assertEqual(hw.memoryFormName(99), "", "memoryFormName unknown code is empty");
assertEqual(hw.decodeMemorySize(0, 0), 0, "decodeMemorySize empty slot is zero");
assertEqual(hw.decodeMemorySize(0xffff, 0), 0, "decodeMemorySize unknown size is zero");
assertEqual(
  hw.decodeMemorySize(0x7fff, 8192),
  8192 * 1024 * 1024,
  "decodeMemorySize uses extended MiB",
);
assertEqual(hw.decodeMemorySize(0x7fff, 0), 0, "decodeMemorySize extended with no MiB is zero");
assertEqual(
  hw.decodeMemorySize(16, 0),
  16 * 1024 * 1024,
  "decodeMemorySize treats low words as MiB",
);
assertEqual(hw.decodeMemorySize(0x8004, 0), 4 * 1024, "decodeMemorySize bit 15 means KiB");

const atmosUp = load("services/AtmosUpdate.js");
assertEqual(
  atmosUp.parseCheckOutput(
    "status behind\nchannel alpha\nlocal abcdef0\nremote abcdef1\nshort abcdef0\n",
  ).status,
  "behind",
  "parseCheckOutput behind",
);
assertEqual(
  atmosUp.parseCheckOutput("status behind\nchannel alpha\n").channel,
  "alpha",
  "parseCheckOutput channel",
);
assertEqual(
  atmosUp.parseCheckOutput("status current\nshort abcdef0\n").status,
  "current",
  "parseCheckOutput current",
);
assertEqual(atmosUp.parseChannel("alpha"), "alpha", "parseChannel accepts alpha");
assertEqual(atmosUp.parseChannel("main"), "", "parseChannel rejects main");
assertEqual(atmosUp.parseSha("--help"), "", "parseSha rejects a flag");
assertEqual(
  atmosUp.parseCheckOutput("status behind\nsummary rm -rf /\n").summary,
  "A newer Atmos is on alpha.",
  "parseCheckOutput drops a junk summary",
);
assertEqual(atmosUp.parseSha("AbCdEf01"), "abcdef01", "parseSha lowercases a hex sha");
assertEqual(atmosUp.parseSha("xyz"), "", "parseSha rejects non-hex");
const fetchFailed = atmosUp.parseCheckOutput(
  "status fetch-failed\nlocal deadbeef\nremote cafebabe\nshort dead\n",
);
assertEqual(fetchFailed.status, "fetch-failed", "parseCheckOutput fetch-failed");
assertEqual(fetchFailed.local, "deadbeef", "parseCheckOutput reads local sha");
assertEqual(fetchFailed.remote, "cafebabe", "parseCheckOutput reads remote sha");
assertEqual(fetchFailed.short, "dead", "parseCheckOutput reads short sha");
assertEqual(
  fetchFailed.summary,
  "Could not fetch the alpha branch.",
  "parseCheckOutput default fetch-failed summary",
);
assertEqual(
  atmosUp.parseCheckOutput("status current\n").summary,
  "Atmos is up to date.",
  "parseCheckOutput default current summary",
);
assertEqual(
  atmosUp.parseCheckOutput("status weird\n").status,
  "unknown",
  "parseCheckOutput rejects an unknown status",
);

const wifi = ui.sortWifiRows([
  ui.wifiRow("b", 10, "psk", false, false),
  ui.wifiRow("a", 80, "psk", true, true),
]);
assertEqual(wifi[0].ssid, "a", "sortWifiRows puts connected first");
const wifiKnown = ui.sortWifiRows([
  ui.wifiRow("open-net", 90, "open", false, false),
  ui.wifiRow("home", 40, "psk", false, true),
]);
assertEqual(
  wifiKnown[0].ssid,
  "home",
  "sortWifiRows puts a known network before a stronger unknown",
);
assert(ui.requiresCredentials("psk") === true, "psk needs a password");
assert(ui.requiresCredentials("open") === false, "open has no password");
assert(ui.requiresCredentials("owe") === false, "owe has no password");
assert(ui.isEnterprise("enterprise") === true, "enterprise kind");
const bt = ui.bluetoothRow("AA:BB:CC:DD:EE:FF", "Buds", 1, 0);
assertEqual(bt.address, "AA:BB:CC:DD:EE:FF", "bluetoothRow keeps the address");
assertEqual(bt.name, "Buds", "bluetoothRow keeps the name");
assert(bt.connected === true, "bluetoothRow coerces connected");
assert(bt.paired === false, "bluetoothRow coerces unpaired");

assert(ui.isTimezoneId("America/New_York") === true, "isTimezoneId accepts Area/City");
assert(ui.isTimezoneId("UTC") === true, "isTimezoneId accepts UTC");
assert(ui.isTimezoneId("Etc/GMT+12") === true, "isTimezoneId accepts Etc/GMT+12");
assert(ui.parseTimezoneId("  Europe/Paris  ") === "Europe/Paris", "parseTimezoneId trims");
assert(ui.isTimezoneId("../etc/passwd") === false, "isTimezoneId rejects path traversal");
assert(ui.isTimezoneId("-America/New_York") === false, "isTimezoneId rejects flags");
assert(ui.isTimezoneId("America/New York") === false, "isTimezoneId rejects spaces");
assert(ui.parseTimezoneId("nope!") === "", "parseTimezoneId rejects junk");

assert(ui.isHostname("hallas") === true, "isHostname accepts a label");
assert(ui.isHostname("my-pc") === true, "isHostname accepts a hyphen");
assert(ui.isHostname("a") === true, "isHostname accepts a single character");
assert(ui.isHostname("foo.bar") === true, "isHostname accepts an FQDN");
assert(ui.parseHostname("  omarchy  ") === "omarchy", "parseHostname trims");
assert(ui.isHostname("-bad") === false, "isHostname rejects a leading hyphen");
assert(ui.isHostname("bad-") === false, "isHostname rejects a trailing hyphen");
assert(ui.isHostname("has space") === false, "isHostname rejects spaces");
assert(ui.parseHostname("--flag") === "", "parseHostname rejects flags");

assert(ui.isKeyboardLayoutId("us") === true, "isKeyboardLayoutId accepts us");
assert(ui.isKeyboardLayoutId("gb") === true, "isKeyboardLayoutId accepts gb");
assert(ui.parseKeyboardLayoutId("us,ru") === "us", "parseKeyboardLayoutId takes the first layout");
assert(ui.isKeyboardLayoutId("US") === false, "isKeyboardLayoutId rejects uppercase");
assert(ui.isKeyboardLayoutId("--us") === false, "isKeyboardLayoutId rejects flags");
const xkbLayouts = ui.parseXkbLayoutList(`
! model
  pc105           Generic 105-key PC
! layout
  us              English (US)
  gb              English (UK)
  de              German
! variant
  intl            English (US, intl.)
`);
assertEqual(xkbLayouts.length, 3, "parseXkbLayoutList reads the layout section");
assertEqual(xkbLayouts[0].value, "us", "parseXkbLayoutList value");
assertEqual(xkbLayouts[0].label, "English (US)", "parseXkbLayoutList label");
assertEqual(xkbLayouts[1].value, "gb", "parseXkbLayoutList gb");

assert(ui.parseTimedatectlYes("yes") === true, "parseTimedatectlYes yes");
assert(ui.parseTimedatectlYes("YES") === true, "parseTimedatectlYes is case-insensitive");
assert(ui.parseTimedatectlYes("true") === true, "parseTimedatectlYes true");
assert(ui.parseTimedatectlYes("1") === true, "parseTimedatectlYes 1");
assert(ui.parseTimedatectlYes("on") === true, "parseTimedatectlYes on");
assert(ui.parseTimedatectlYes("no") === false, "parseTimedatectlYes no");
assert(ui.parseTimedatectlYes("maybe") === false, "parseTimedatectlYes rejects junk");

assert(ui.isLocaleId("en_US.UTF-8") === true, "isLocaleId accepts en_US.UTF-8");
assert(ui.isLocaleId("C.UTF-8") === true, "isLocaleId accepts C.UTF-8");
assert(ui.isLocaleId("ca_ES.UTF-8@valencia") === true, "isLocaleId accepts a modifier");
assert(ui.parseLocaleId("  de_DE.UTF-8  ") === "de_DE.UTF-8", "parseLocaleId trims");
assert(ui.isLocaleId("EN_US.UTF-8") === false, "isLocaleId rejects uppercase language");
assert(ui.isLocaleId("en") === false, "isLocaleId rejects a language-only tag");
assert(ui.parseLocaleId("--bad") === "", "parseLocaleId rejects flags");

assert(ui.isFullName("Ada Lovelace") === true, "isFullName accepts a display name");
assert(ui.isFullName("") === true, "isFullName allows empty");
assert(ui.parseFullName("  Jean-Luc  ") === "Jean-Luc", "parseFullName trims");
assert(ui.isFullName("bad:name") === false, "isFullName rejects a colon");
assert(ui.isFullName("Last, First") === false, "isFullName rejects a comma");
assert(ui.parseFullName("--flag") === "", "parseFullName rejects flags");

assertEqual(
  ui.parseParallelDownloads("ParallelDownloads = 5\n"),
  5,
  "parseParallelDownloads reads a value",
);
assertEqual(
  ui.parseParallelDownloads("#ParallelDownloads = 9\nParallelDownloads = 3\n"),
  3,
  "parseParallelDownloads skips comments",
);
assertEqual(ui.parseParallelDownloads("Color\n"), 0, "parseParallelDownloads missing");
assertEqual(
  ui.parseParallelDownloads("ParallelDownloads = 99\n"),
  20,
  "parseParallelDownloads clamps high values",
);
assertEqual(
  ui.parseParallelDownloads("ParallelDownloads = 0\n"),
  0,
  "parseParallelDownloads rejects zero",
);

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
  { label: "", description: "", detail: "", hint: "" },
  null,
]);
assertEqual(helpTopics.length, 2, "sectionHelpTopics skips empty rows");
assertEqual(
  helpTopics[0].body,
  "A named palette plus templates.",
  "sectionHelpTopics prefers detail",
);
assertEqual(helpTopics[1].body, "Monospace family.", "sectionHelpTopics falls back to description");
assertEqual(helpTopics[1].command, "omarchy font set", "sectionHelpTopics keeps the command");
assertEqual(layout.sectionHelpTopics(null).length, 0, "sectionHelpTopics ignores a non-array");
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

const wrap = load("services/TextWrap.js");
function ch(s) {
  return s.length;
}

assertEqual(
  wrap.prettyWrap("The quick brown fox jumps", ch, 20, 1),
  "The quick brown\nfox jumps",
  "prettyWrap balances a leftover last word",
);
assertEqual(
  wrap.balanceWrap("aaa bbb ccc ddd", ch, 7, 1),
  "aaa bbb\nccc ddd",
  "balanceWrap splits even groups",
);
assertEqual(
  wrap.balanceWrap("Word another last", ch, 12, 1),
  "Word another\nlast",
  "balanceWrap keeps the greedy line count",
);
assertEqual(
  wrap.prettyWrap("Word another last", ch, 12, 1),
  "Word\nanother last",
  "prettyWrap pulls a one-word last line up",
);
assertEqual(
  wrap.prettyWrap("Hello world", ch, 20, 1),
  "Hello world",
  "prettyWrap leaves a single line alone",
);
assertEqual(
  wrap.prettyWrap("/home/foo/very-long-path", ch, 10, 1),
  "/home/foo/very-long-path",
  "prettyWrap leaves a path without spaces alone",
);
assertEqual(wrap.prettyWrap("", ch, 20, 1), "", "prettyWrap empty");
assertEqual(wrap.shouldSkip("oneword"), true, "shouldSkip a token without spaces");
assertEqual(wrap.shouldSkip("two words"), false, "shouldSkip false when there is a space");
assertEqual(
  wrap.wrapAll("aaa bbb\nccc ddd", ch, 7, 1, false),
  "aaa bbb\nccc ddd",
  "wrapAll wraps each line separately",
);
assertEqual(
  wrap.wrapAll("aaa bbb", null, 7, 1, false),
  "aaa bbb",
  "wrapAll leaves text when measure is missing",
);
assertEqual(
  wrap.wrapAll("aaa bbb", ch, 0, 1, false),
  "aaa bbb",
  "wrapAll leaves text when maxWidth is zero",
);
assertEqual(
  wrap.splitWords("  alpha   beta ").join(","),
  "alpha,beta",
  "splitWords collapses whitespace",
);
assertEqual(
  wrap.wrapAll("aaa bbb\n\nccc ddd", ch, 7, 1, false),
  "aaa bbb\n\nccc ddd",
  "wrapAll preserves a blank line between paragraphs",
);
assertEqual(
  wrap.wrapAll("aaa bbb", ch, 6, -1, false),
  "aaa bbb",
  "wrapAll treats a negative space as zero",
);

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
assert(lookLua.indexOf("preserve_split = false") !== -1, "serializeLook writes preserve_split");
assert(
  lookLua.indexOf("focus_on_activate = false") !== -1,
  "serializeLook writes focus_on_activate",
);

const lookExtras = hypr.serializeLook({
  cursorSize: 40,
  activeOpacity: 0.8,
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
  lookExtras.indexOf("preserve_split = true") !== -1,
  "serializeLook writes preserve_split on",
);
assertEqual(hypr.clampLook({ cursorSize: 90 }).cursorSize, 64, "clampLook caps cursor size");
assertEqual(
  hypr.clampLook({ activeOpacity: 0.05 }).activeOpacity,
  0.2,
  "clampLook floors active opacity",
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

const sunset = load("services/HyprSunset.js");
assertEqual(sunset.parseTime("7:00"), "07:00", "parseTime pads an hour");
assertEqual(sunset.parseTime("20:15"), "20:15", "parseTime keeps a valid time");
assertEqual(sunset.parseTime("25:00"), "", "parseTime rejects a bad hour");
const parsedSunset = sunset.parseConf(`
profile {
    time = 06:30
    identity = true
}

profile {
    time = 21:00
    temperature = 3800
}
`);
assertEqual(parsedSunset.day, "06:30", "parseConf reads the day profile");
assertEqual(parsedSunset.night, "21:00", "parseConf reads the night profile");
assertEqual(parsedSunset.nightOn, true, "parseConf sees a night profile");
assertEqual(parsedSunset.temperature, 3800, "parseConf reads temperature");
const writtenSunset = sunset.serializeConf({
  day: "07:00",
  night: "20:00",
  nightOn: true,
  temperature: 4000,
});
assert(writtenSunset.indexOf("time = 07:00") !== -1, "serializeConf writes day");
assert(writtenSunset.indexOf("temperature = 4000") !== -1, "serializeConf writes temperature");
assertEqual(
  sunset.serializeConf({ nightOn: false }).indexOf("temperature") === -1,
  true,
  "serializeConf omits night when off",
);
assertEqual(sunset.parseTime("9:05"), "09:05", "parseTime pads a single-digit hour");
assertEqual(sunset.parseTime("7:5"), "", "parseTime rejects a single-digit minute");
assertEqual(sunset.parseTime("24:00"), "", "parseTime rejects 24:00");
assertEqual(sunset.clampTemp(2500, 4000), 3000, "clampTemp floors below 3000");
assertEqual(sunset.clampTemp(9000, 4000), 6500, "clampTemp caps above 6500");
assertEqual(sunset.clampTemp("nope", 4000), 4000, "clampTemp uses fallback on NaN");
const clampedSunset = sunset.clampSchedule({
  day: "nope",
  night: "21:30",
  nightOn: "yes",
  temperature: 2000,
});
assertEqual(clampedSunset.day, "07:00", "clampSchedule falls back to default day");
assertEqual(clampedSunset.night, "21:30", "clampSchedule keeps a valid night");
assertEqual(clampedSunset.nightOn, false, "clampSchedule requires nightOn === true");
assertEqual(clampedSunset.temperature, 3000, "clampSchedule floors temperature");
const quotedSunset = sunset.parseConf(`
profile {
    time = "8:00"
    identity = true
}
profile {
    time = "19:45"
    temperature = 5000
}
`);
assertEqual(quotedSunset.day, "08:00", "parseConf reads a quoted day time");
assertEqual(quotedSunset.night, "19:45", "parseConf reads a quoted night time");
assertEqual(
  sunset.parseConf("").nightOn,
  false,
  "parseConf empty file uses defaults with night off",
);
assertEqual(
  sunset.parseConf("profile {\n    time = 99:99\n    identity = true\n}\n").day,
  "07:00",
  "parseConf skips an invalid day time",
);
assertEqual(
  sunset.parseConf("profile {\n    time = 21:00\n}\n").nightOn,
  false,
  "parseConf skips a profile with neither identity nor temperature",
);

const hooks = load("services/Hooks.js");
assertEqual(hooks.argFor("theme-set").indexOf("theme") !== -1, true, "argFor describes theme-set");
assertEqual(hooks.isType("battery-low"), true, "isType accepts battery-low");
assertEqual(hooks.isType("nope"), false, "isType rejects an unknown hook");
assertEqual(hooks.isHookId("waki-webapp-install"), true, "isHookId accepts a custom kebab id");
assertEqual(hooks.isHookId("Nope"), false, "isHookId rejects uppercase");
assertEqual(hooks.typeInfo("nope!"), null, "typeInfo rejects a bad extra id");
assertEqual(hooks.whenFor("post-boot"), "After the desktop starts.", "whenFor names post-boot");
assertEqual(hooks.runArgFor("theme-set"), "theme", "runArgFor names theme-set");
assert(
  hooks.eventBlurb("theme-set").indexOf("$1 is") !== -1,
  "eventBlurb includes $1 for theme-set",
);
assert(
  hooks.eventBlurb("post-boot").indexOf("no extra argument") !== -1,
  "eventBlurb notes a hook with no arg",
);
assertEqual(
  hooks.parseListing([
    { type: "theme-set", name: "mine.sh", path: "/tmp/../etc/passwd", sample: false },
  ]).length,
  0,
  "parseListing rejects a path with ..",
);
assertEqual(
  hooks.itemsFor(
    [
      {
        type: "theme-set",
        name: "a.sh",
        path: "/home/u/.config/omarchy/hooks/theme-set.d/a.sh",
        sample: false,
      },
      {
        type: "font-set",
        name: "b.sh",
        path: "/home/u/.config/omarchy/hooks/font-set.d/b.sh",
        sample: false,
      },
    ],
    "theme-set",
  ).length,
  1,
  "itemsFor filters by type",
);
assertEqual(hooks.labelFor("theme-set"), "Theme set", "labelFor names theme-set");
assertEqual(hooks.sanitizeName("notify"), "notify.sh", "sanitizeName adds .sh");
assertEqual(hooks.sanitizeName("bad/name"), "", "sanitizeName rejects a slash");
assertEqual(hooks.sanitizeName("keep.sample"), "", "sanitizeName rejects a sample suffix");
assertEqual(hooks.sanitizeLine('echo "$1"'), 'echo "$1"', "sanitizeLine keeps $1");
assertEqual(hooks.sanitizeLine("bad\nline"), "", "sanitizeLine rejects a newline");
assert(
  hooks.scriptBody("theme-set", 'echo "$1"').indexOf('echo "$1"') !== -1,
  "scriptBody writes the command",
);
assertEqual(
  hooks.destHint("theme-set", "notify"),
  "~/.config/omarchy/hooks/theme-set.d/notify.sh",
  "destHint names the install path",
);
assertEqual(
  hooks.displayTypes([
    {
      type: "waki-webapp-install",
      name: "waki-webapp-install",
      path: "/home/u/.config/omarchy/hooks/waki-webapp-install",
      sample: false,
      flat: true,
    },
  ]).length,
  7,
  "displayTypes appends an extra hook id",
);
assertEqual(
  hooks.typeIds().join(","),
  "theme-set,font-set,post-boot,post-update,pre-refresh-pacman,battery-low",
  "typeIds lists built-in hooks",
);
assertEqual(hooks.options().length, 6, "options lists built-in hook types");
assertEqual(
  hooks.typeInfo("waki-webapp-install").when.indexOf("hook folder") !== -1,
  true,
  "typeInfo describes an extra hook id",
);
assertEqual(hooks.destHint("Nope", "notify"), "", "destHint rejects a bad hook type");

const auto = load("services/Autostart.js");
const autoSeed = '-- Extra autostart processes.\no.launch_on_start("waybar")\n';
const autoApplied = auto.applyFile(autoSeed, ["hyprsunset", "mako"]);
assert(
  autoApplied.indexOf("-- Extra autostart processes.") !== -1,
  "applyFile keeps user comments",
);
assert(
  autoApplied.indexOf('o.launch_on_start("waybar")') !== -1,
  "applyFile keeps unmanaged launch lines",
);
assert(
  autoApplied.indexOf('o.launch_on_start("hyprsunset")') !== -1,
  "applyFile writes a managed command",
);
const autoParsed = auto.parseFile(autoApplied);
assertEqual(
  autoParsed.filter(function (row) {
    return row.managed;
  }).length,
  2,
  "parseFile marks managed commands",
);
assertEqual(
  autoParsed.filter(function (row) {
    return !row.managed && row.command === "waybar";
  }).length,
  1,
  "parseFile keeps unmanaged commands",
);
assertEqual(auto.sanitizeCommand("bad\ncmd"), "", "sanitizeCommand rejects a newline");
assertEqual(
  auto.sanitizeCommand("x".repeat(257)),
  "",
  "sanitizeCommand rejects a command over 256 chars",
);
assertEqual(auto.unescapeLua('\\"quoted\\"'), '"quoted"', "unescapeLua restores escaped quotes");
assertEqual(auto.luaString('say "hi"'), '"say \\"hi\\""', "luaString escapes quotes");
const quotedLaunch = auto.parseCalls('o.launch_on_start("echo \\"hi\\"")\n');
assertEqual(quotedLaunch[0], 'echo "hi"', "parseCalls unescapes a quoted command");
const managed = auto.managedCommands([
  { command: "waybar", managed: false },
  { command: "mako", managed: true },
  "hyprsunset",
  { command: "bad\ncmd", managed: true },
  null,
]);
assertEqual(
  managed.join(","),
  "mako,hyprsunset",
  "managedCommands keeps only managed or string rows",
);
const autoReplaced = auto.applyFile(autoApplied, ["swaybg"]);
assertEqual(
  (autoReplaced.match(/-- atmos:autostart begin/g) || []).length,
  1,
  "applyFile replaces an existing autostart sentinel",
);
assert(
  autoReplaced.indexOf('o.launch_on_start("swaybg")') !== -1,
  "applyFile writes the new managed command",
);
assert(
  autoReplaced.indexOf('o.launch_on_start("mako")') === -1,
  "applyFile drops previous managed commands",
);
assert(
  auto.extractSentinel(autoApplied).indexOf('o.launch_on_start("hyprsunset")') !== -1,
  "extractSentinel returns the autostart block",
);
assertEqual(
  auto.extractSentinel("-- Extra autostart processes.\n"),
  "",
  "extractSentinel misses a file without a sentinel",
);
assertEqual(auto.sanitizeCommand(""), "", "sanitizeCommand rejects empty");
assert(
  auto.serialize([]).indexOf("o.launch_on_start") === -1,
  "serialize empty list writes only the sentinel",
);
assertEqual(
  auto.parseCalls('o.launch_on_start("")').length,
  0,
  "parseCalls skips an empty command",
);

const software = load("services/Software.js");
assertEqual(software.lookup("steam").wipe, true, "catalog marks Steam as a wipe remove");
assertEqual(
  software.presentIn(software.lookup("firefox"), { browsers: { firefox: true } }),
  true,
  "presentIn reads a browser bag",
);
assertEqual(
  software.presentIn(software.lookup("vscode"), { editors: { code: true } }),
  true,
  "presentIn maps vscode to editors.code",
);
assertEqual(software.isDevEnv("python"), true, "isDevEnv accepts python");
assertEqual(software.isDockerDb("PostgreSQL"), true, "isDockerDb accepts PostgreSQL");
assertEqual(software.groupItems("gaming").length > 3, true, "groupItems lists gaming installers");
assertEqual(software.lookup("nope"), null, "lookup misses an unknown id");
assertEqual(
  software.presentIn(software.lookup("zed"), { editors: { zeditor: true } }),
  true,
  "presentIn maps zed to editors.zeditor",
);
assertEqual(
  software.presentIn(software.lookup("firefox"), { browsers: {} }),
  false,
  "presentIn is false when the bag flag is missing",
);
assertEqual(software.isDevEnv("cobol"), false, "isDevEnv rejects an unknown env");
assertEqual(software.isDockerDb("sqlite"), false, "isDockerDb rejects an unknown db");
assertEqual(software.groupItems("browsers").length, 6, "groupItems lists the browser installers");
assertEqual(
  software.presentIn(software.lookup("1password"), { services: { onepassword: true } }),
  true,
  "presentIn maps 1password to services.onepassword",
);
assertEqual(software.groupItems("nope").length, 0, "groupItems empty for an unknown group");
assertEqual(
  software.presentIn(null, { browsers: { firefox: true } }),
  false,
  "presentIn is false without an item",
);

const binds = load("services/Bindings.js");
assertEqual(
  binds.sanitizeKeys("SUPER + SHIFT + R"),
  "SUPER + SHIFT + R",
  "sanitizeKeys keeps a chord",
);
assertEqual(binds.sanitizeKeys("SUPER + F\n"), "", "sanitizeKeys rejects a newline");
assertEqual(binds.sanitizeCommand("bad\ncmd"), "", "sanitizeCommand rejects a newline");
const bindSeed =
  '-- Keep only your personal keybinding overrides here.\no.bind("SUPER + D", "Desks", "omarchy-shell shell toggle com.mdtrr.omadesk")\n';
const bindApplied = binds.applyFile(bindSeed, [
  { keys: "SUPER + F", label: "Files", command: "nautilus", unbind: true },
  { keys: "SUPER + SHIFT + B", unbind: true },
]);
assert(
  bindApplied.indexOf("-- Keep only your personal keybinding overrides here.") !== -1,
  "applyFile keeps binding comments",
);
assert(
  bindApplied.indexOf('o.bind("SUPER + D", "Desks"') !== -1,
  "applyFile keeps unmanaged binds",
);
assert(
  bindApplied.indexOf('hl.unbind("SUPER + F")') !== -1,
  "applyFile writes unbind before a replacement",
);
assert(
  bindApplied.indexOf('o.bind("SUPER + F", "Files", "nautilus")') !== -1,
  "applyFile writes a managed bind",
);
assert(
  bindApplied.indexOf('hl.unbind("SUPER + SHIFT + B")') !== -1,
  "applyFile writes an unbind-only row",
);
const bindParsed = binds.parseFile(bindApplied);
assertEqual(
  bindParsed.filter(function (row) {
    return row.managed;
  }).length,
  2,
  "parseFile marks managed bindings",
);
assertEqual(
  bindParsed.filter(function (row) {
    return !row.managed && row.keys === "SUPER + D";
  }).length,
  1,
  "parseFile keeps unmanaged bindings",
);
const printed = binds.parsePrint(
  "SUPER + Q                         \u2192 Close window\nSUPER + D                         \u2192 Desks\n",
);
assertEqual(printed.length, 2, "parsePrint reads display lines");
assertEqual(printed[0].action, "Close window", "parsePrint reads the action");
assertEqual(
  binds.catalogConflict(printed, "SUPER + Q"),
  "Close window",
  "catalogConflict finds a taken chord",
);
assertEqual(binds.catalogConflict(printed, "SUPER + Z"), "", "catalogConflict misses a free chord");
assertEqual(
  binds.catalogConflict(printed, "SUPER + Q\n"),
  "",
  "catalogConflict rejects a newline chord",
);
const asciiPrint = binds.parsePrint("SUPER + Return -> Terminal\nno arrow here\n");
assertEqual(asciiPrint.length, 1, "parsePrint reads an ASCII arrow");
assertEqual(asciiPrint[0].keys, "SUPER + Return", "parsePrint ASCII keys");
assertEqual(asciiPrint[0].action, "Terminal", "parsePrint ASCII action");
assertEqual(binds.sanitizeLabel(" Files "), "Files", "sanitizeLabel trims");
assertEqual(binds.sanitizeLabel("bad\nlabel"), "", "sanitizeLabel rejects a newline");
assertEqual(binds.sanitizeKeys("SUPER  +   Q"), "SUPER + Q", "sanitizeKeys collapses spaces");
assertEqual(binds.sanitizeKeys("A".repeat(65)), "", "sanitizeKeys rejects a chord over 64 chars");
assertEqual(
  binds.sanitizeKeys("SUPER + @"),
  "",
  "sanitizeKeys rejects punctuation outside the charset",
);
assertEqual(binds.sanitizeLabel("x".repeat(65)), "", "sanitizeLabel rejects a label over 64 chars");
const nilBind = binds.serialize([{ keys: "SUPER + X", command: "foot" }]);
assert(
  nilBind.indexOf('o.bind("SUPER + X", nil, "foot")') !== -1,
  "serialize uses nil when the label is empty",
);
const managedBinds = binds.managedItems([
  { keys: "SUPER + D", command: "desks", managed: false },
  { keys: "SUPER + F", command: "nautilus", managed: true },
  { keys: "SUPER + Q" },
  null,
]);
assertEqual(managedBinds.length, 1, "managedItems drops unmanaged and commandless rows");
assertEqual(managedBinds[0].keys, "SUPER + F", "managedItems keeps a managed bind");
assertEqual(
  binds.commandFromArg({ launch: "foot" }),
  "foot",
  "commandFromArg reads a launch table",
);
assertEqual(binds.commandFromArg({ launch: "bad\ncmd" }), "", "commandFromArg sanitizes launch");
const folded = binds.foldEvents([
  { kind: "unbind", keys: "SUPER + Q" },
  { kind: "bind", keys: "SUPER + Q", label: "Quit", command: "kill" },
  { kind: "unbind", keys: "SUPER + X" },
]);
assertEqual(folded[0].unbind, true, "foldEvents pairs unbind with the next bind");
assertEqual(folded[0].command, "kill", "foldEvents keeps the replacement command");
assertEqual(folded[1].command, "", "foldEvents keeps a lone unbind");
assertEqual(
  binds.parseCalls('o.bind("SUPER + T", nil, { launch = "foot" })')[0].command,
  "foot",
  "parseCalls reads a launch table bind",
);

const rules = load("services/WindowRules.js");
assertEqual(rules.sanitizeMatch("firefox"), "firefox", "sanitizeMatch keeps a class");
assertEqual(rules.sanitizeMatch("bad]]class"), "", "sanitizeMatch rejects ]]");
assertEqual(rules.sanitizeWorkspace("../etc"), "", "sanitizeWorkspace rejects a path");
assertEqual(
  rules.sanitizeWorkspace(" special:5 "),
  "special:5",
  "sanitizeWorkspace keeps a named workspace",
);
assertEqual(rules.sanitizeWorkspace(""), "", "sanitizeWorkspace empty");
assertEqual(rules.sanitizeMatch("bad\nclass"), "", "sanitizeMatch rejects a newline");
assertEqual(
  rules.sanitizeMatch("x".repeat(129)),
  "",
  "sanitizeMatch rejects a match over 128 chars",
);
assertEqual(rules.describe(null), "", "describe empty for a missing row");
assertEqual(rules.describe({ center: true }), "center", "describe names center");
const ruleSeed =
  'o.window("dev.csfh.atmos", { float = true })\no.window("dev.csfh.atmos", { center = true })\n';
const ruleApplied = rules.applyFile(ruleSeed, [
  { match: "^Emulator$", placement: "float", center: true, width: 1280, height: 800 },
  { match: "qemu", workspace: "5" },
]);
assert(
  ruleApplied.indexOf('o.window("dev.csfh.atmos", { float = true })') !== -1,
  "applyFile keeps prefs window rules",
);
assert(
  ruleApplied.indexOf(
    'o.window("^Emulator$", { float = true, center = true, size = { 1280, 800 } })',
  ) !== -1,
  "applyFile writes a managed window rule",
);
assert(
  ruleApplied.indexOf('o.window("qemu", { workspace = "5" })') !== -1,
  "applyFile writes a workspace rule",
);
const ruleParsed = rules.parseFile(ruleApplied);
assertEqual(
  ruleParsed.filter(function (row) {
    return row.managed;
  }).length,
  2,
  "parseFile marks managed window rules",
);
assertEqual(
  ruleParsed.filter(function (row) {
    return !row.managed && row.match === "dev.csfh.atmos";
  }).length,
  2,
  "parseFile keeps the prefs window rules",
);
const required = rules.ensureRequire(
  'require("hypr.autostart")\nrequire("default.hypr.toggles")\n',
);
assert(
  required.indexOf('require("hypr.atmos")\nrequire("default.hypr.toggles")') !== -1,
  "ensureRequire inserts before toggles",
);
assertEqual(rules.ensureRequire(""), "", "ensureRequire leaves an empty hyprland.lua alone");
assertEqual(
  rules.describe({ placement: "float", center: true }).indexOf("float") !== -1,
  true,
  "describe names float",
);
assertEqual(rules.clampSize(1280), 1280, "clampSize keeps a valid size");
assertEqual(rules.clampSize(50), 0, "clampSize rejects below 100");
assertEqual(rules.clampSize(5000), 0, "clampSize rejects above 4000");
assertEqual(rules.clampSize("nope"), 0, "clampSize rejects NaN");
assert(
  rules.prefsSeed().indexOf('o.window("dev.csfh.atmos"') !== -1,
  "prefsSeed floats the Atmos class",
);
assert(rules.prefsSeed().indexOf("size = { 960, 680 }") !== -1, "prefsSeed sets the Atmos size");
assertEqual(
  rules.describe({ placement: "tile", workspace: "5" }),
  "tile \u00b7 workspace 5",
  "describe names tile and workspace",
);
assertEqual(
  rules.describe({ width: 1280, height: 800 }).indexOf("\u00d7") !== -1,
  true,
  "describe names a size",
);
const already = 'require("hypr.atmos")\nrequire("default.hypr.toggles")\n';
assertEqual(
  rules.ensureRequire(already),
  already,
  "ensureRequire is a no-op when hypr.atmos is present",
);
const appended = rules.ensureRequire('require("hypr.autostart")\n');
assert(
  appended.indexOf('require("hypr.atmos")') !== -1,
  "ensureRequire appends when toggles are missing",
);
const managedRules = rules.managedItems([
  { match: "qemu", workspace: "5", managed: false },
  { match: "firefox", placement: "float", managed: true },
  { match: "bad]]", placement: "float" },
  null,
]);
assertEqual(managedRules.length, 1, "managedItems drops unmanaged and invalid rows");
assertEqual(managedRules[0].match, "firefox", "managedItems keeps a managed float rule");
assertEqual(
  rules.parseCalls('o.window("foo\\"bar", { float = true })\n')[0].match,
  'foo"bar',
  "parseCalls unescapes a quoted window match",
);
assert(
  rules.serialize([{ match: "qemu", placement: "tile" }]).indexOf("tile = true") !== -1,
  "serialize writes tile = true",
);
assertEqual(
  rules.normalize({ match: "firefox", float: true }).placement,
  "float",
  "normalize maps float true to placement",
);
assertEqual(
  rules.normalize({ match: "foot", size: [1280, 800] }).width,
  1280,
  "normalize reads a size array",
);
assertEqual(rules.normalize({ match: "ghost" }), null, "normalize drops a match with no effects");

const accounts = load("services/Accounts.js");
assertEqual(accounts.isUsername("hallas"), true, "isUsername accepts a login");
assertEqual(accounts.parseUsername("  hallas  "), "hallas", "parseUsername trims");
assertEqual(accounts.isUsername("root"), false, "isUsername rejects root");
assertEqual(accounts.isUsername("sddm"), false, "isUsername rejects sddm");
assertEqual(accounts.isUsername("Hallas"), false, "isUsername rejects uppercase");
assertEqual(accounts.isUsername("hallas-"), false, "isUsername rejects a trailing hyphen");
assertEqual(accounts.isUsername("a".repeat(33)), false, "isUsername rejects 33 characters");
assertEqual(accounts.isUsername("_svc"), true, "isUsername accepts a leading underscore");
assertEqual(accounts.isGroupName("family"), true, "isGroupName accepts a group");
assertEqual(accounts.isGroupName("root"), false, "isGroupName rejects root");
assertEqual(accounts.isGroupName("nogroup"), false, "isGroupName rejects nogroup");
assertEqual(accounts.isGroupName("family-"), false, "isGroupName rejects a trailing hyphen");
assertEqual(accounts.isFullName(""), true, "isFullName allows empty");
assertEqual(accounts.isFullName("Christoffer Hallas"), true, "isFullName accepts a real name");
assertEqual(accounts.isFullName("Last, First"), false, "isFullName rejects a comma");
assertEqual(accounts.isFullName("bad:name"), false, "isFullName rejects a colon");
assertEqual(accounts.isFullName("-flag"), false, "isFullName rejects a leading hyphen");
assertEqual(accounts.parseFullName("  Ada  "), "Ada", "parseFullName trims");
assertEqual(accounts.parseFullName("Last, First"), "", "parseFullName rejects a comma");
assertEqual(
  accounts.gecosFullName("Ada Lovelace,Office,1"),
  "Ada Lovelace",
  "gecosFullName cuts at comma",
);
assertEqual(accounts.gecosFullName("Ada"), "Ada", "gecosFullName keeps a bare name");
assertEqual(accounts.isHumanUid(1000), true, "isHumanUid accepts 1000");
assertEqual(accounts.isHumanUid(999), false, "isHumanUid rejects 999");
assertEqual(accounts.isHumanUid(65534), false, "isHumanUid rejects nobody");
assertEqual(accounts.isRemovableGid(1000, "family"), true, "isRemovableGid accepts a user group");
assertEqual(accounts.isRemovableGid(1000, "wheel"), false, "isRemovableGid rejects wheel");
assertEqual(accounts.isRemovableGid(982, "docker"), false, "isRemovableGid rejects docker");
assertEqual(accounts.isRemovableGid(0, "root"), false, "isRemovableGid rejects gid 0");

const passwdText = [
  "root:x:0:0:root:/root:/bin/bash",
  "sddm:x:979:979:Simple Desktop Display Manager:/var/lib/sddm:/usr/bin/nologin",
  "hallas:x:1000:1000:Christoffer Hallas,Office:/home/hallas:/bin/bash",
  "alice:x:1001:1001:Alice:/home/alice:/bin/bash",
].join("\n");
const groupText = [
  "root:x:0:root",
  "wheel:x:998:hallas",
  "docker:x:982:hallas",
  "hallas:x:1000:",
  "alice:x:1001:",
  "family:x:1002:hallas,alice",
  "nobody:x:65534:",
].join("\n");
const parsedUsers = accounts.parsePasswd(passwdText, "hallas");
assertEqual(parsedUsers.length, 2, "parsePasswd keeps human logins");
assertEqual(parsedUsers[0].name, "hallas", "parsePasswd keeps the session user");
assertEqual(parsedUsers[0].fullName, "Christoffer Hallas", "parsePasswd reads GECOS");
assertEqual(parsedUsers[0].current, true, "parsePasswd marks the session user");
assertEqual(parsedUsers[1].name, "alice", "parsePasswd keeps another human login");
assert(
  parsedUsers.every(function (row) {
    return row.name !== "root" && row.name !== "sddm";
  }),
  "parsePasswd drops system accounts",
);
const withWheel = accounts.attachWheel(parsedUsers, accounts.parseGroup(groupText));
assertEqual(withWheel[0].wheel, true, "attachWheel marks hallas");
assertEqual(withWheel[1].wheel, false, "attachWheel leaves alice off wheel");
const shown = accounts.visibleGroups(accounts.parseGroup(groupText), withWheel);
const shownNames = shown.map(function (g) {
  return g.name;
});
assert(
  shownNames.indexOf("wheel") !== -1 && shownNames.indexOf("docker") !== -1,
  "visibleGroups keeps wheel and docker",
);
assert(shownNames.indexOf("family") !== -1, "visibleGroups keeps a human group");
assert(shownNames.indexOf("hallas") === -1, "visibleGroups hides a private user group");
assert(shownNames.indexOf("alice") === -1, "visibleGroups hides alice's private group");
assert(shownNames.indexOf("nobody") === -1, "visibleGroups hides nobody");
assertEqual(
  accounts.groupHasMember({ members: ["hallas", "alice"] }, "alice"),
  true,
  "groupHasMember finds a member",
);
assertEqual(
  accounts.groupHasMember({ members: ["hallas"] }, "alice"),
  false,
  "groupHasMember misses a stranger",
);
const inventory = accounts.parseAccounts({
  currentUser: "hallas",
  avatarPath: "/home/hallas/.face.icon",
  passwd: passwdText,
  group: groupText,
});
assertEqual(inventory.currentUser, "hallas", "parseAccounts keeps currentUser");
assertEqual(inventory.users.length, 2, "parseAccounts returns two humans");
assertEqual(inventory.users[0].wheel, true, "parseAccounts attaches wheel");
assert(
  inventory.groups.every(function (g) {
    return g.name !== "hallas" && g.name !== "alice";
  }),
  "parseAccounts omits private groups",
);
assertEqual(
  accounts.pickAvatarPath("/home/hallas", "hallas", function (path) {
    return path === "/var/lib/AccountsService/icons/hallas";
  }),
  "/var/lib/AccountsService/icons/hallas",
  "pickAvatarPath prefers AccountsService",
);
assertEqual(
  accounts.pickAvatarPath("/home/hallas", "hallas", function (path) {
    return path === "/home/hallas/.face";
  }),
  "/home/hallas/.face",
  "pickAvatarPath falls through to ~/.face",
);
assertEqual(
  accounts.pickAvatarPath("/home/hallas", "hallas", function () {
    return false;
  }),
  "",
  "pickAvatarPath is empty when nothing exists",
);
assertEqual(
  accounts.profileTitle("Chris Hallas", "hallas"),
  "Chris Hallas",
  "profileTitle prefers full name",
);
assertEqual(accounts.profileTitle("", "hallas"), "hallas", "profileTitle falls back to login");
assertEqual(accounts.profileTitle("", ""), "Account", "profileTitle empty is Account");
assertEqual(
  accounts.profileHost("hallas", "framework"),
  "hallas@framework",
  "profileHost joins login and hostname",
);
assertEqual(accounts.profileHost("hallas", ""), "hallas", "profileHost falls back to login");
assertEqual(accounts.profileHost("", "framework"), "", "profileHost empty without a login");
assertEqual(
  accounts.profileHost("hallas", "bad host"),
  "hallas",
  "profileHost drops a hostname with a space",
);

const settings = load("services/Settings.js");

const s_catalog = settings.settingsCatalog();
assert(s_catalog.length > 0, "settingsCatalog returns entries");

const seen = {};
let duplicate = "";
for (const item of s_catalog) {
  if (seen[item.key]) duplicate = item.key;
  seen[item.key] = true;
}
assertEqual(duplicate, "", "settingsCatalog has no duplicate keys");

const sectionIds = {};
for (const section of settings.settingsSections()) sectionIds[section.id] = true;
let strayCatalogSection = "";
for (const item of s_catalog) {
  if (!sectionIds[item.section]) strayCatalogSection = item.section;
}
assertEqual(strayCatalogSection, "", "every s_catalog entry lands in a known section");

const byKey = settings.catalogByKey(s_catalog);
assertEqual(byKey.theme.tier, "look", "theme is a look setting");
assertEqual(byKey.hostname.tier, "identity", "hostname is an identity setting");
assertEqual(byKey.fullName.needsRoot, true, "fullName needs root because as-root.sh writes it");
assertEqual(byKey.plymouth.needsRoot, true, "plymouth needs root like the Boot page");
assertEqual(byKey.bindings.extraConfirm, true, "bindings need a second confirm for commands");
assertEqual(byKey.autostart.extraConfirm, true, "autostart needs a second confirm for commands");
assertEqual(byKey.sshdEnabled.importable, false, "sshdEnabled is never importable");
assertEqual(byKey.passwordlessSudo.importable, false, "passwordlessSudo is never importable");
assertEqual(byKey.sudolessDocker.importable, false, "sudolessDocker is never importable");

let systemImportable = "";
for (const item of s_catalog) {
  if (item.tier === "system" && item.importable) systemImportable = item.key;
}
assertEqual(systemImportable, "", "no system-tier setting is importable");

let missingConsequence = "";
for (const item of s_catalog) {
  if (item.tier === "behavior" && !item.consequence && !item.hostBound)
    missingConsequence = item.key;
}
assertEqual(missingConsequence, "", "every portable behavior setting explains what changes");

const lookPreset = settings.presetKeys("look", s_catalog);
assert(lookPreset.indexOf("theme") !== -1, "the look preset takes the theme");
assert(lookPreset.indexOf("browser") === -1, "the look preset leaves the browser alone");
assert(lookPreset.indexOf("hostname") === -1, "the look preset leaves the hostname alone");

const portablePreset = settings.presetKeys("portable", s_catalog);
assert(portablePreset.indexOf("browser") !== -1, "the portable preset takes the browser");
assert(portablePreset.indexOf("hostname") === -1, "the portable preset leaves the hostname alone");
assert(
  portablePreset.indexOf("hyprInput.sensitivity") === -1,
  "the portable preset drops device-tuned input",
);

const fullPreset = settings.presetKeys("full", s_catalog);
assert(fullPreset.indexOf("hostname") !== -1, "the full preset takes the hostname");
assert(fullPreset.indexOf("sshdEnabled") === -1, "even the full preset refuses security settings");

assertEqual(
  settings.readValue({ hyprLook: { gapsIn: 8 } }, "hyprLook.gapsIn"),
  8,
  "readValue walks a dotted key",
);
assertEqual(
  settings.readValue({}, "hyprLook.gapsIn"),
  undefined,
  "readValue survives a missing branch",
);

const s_snapshot = {
  hostname: "atlas",
  theme: "tokyo-night",
  themes: ["tokyo-night", "catppuccin"],
  font: "CaskaydiaMono Nerd Font",
  fonts: ["CaskaydiaMono Nerd Font"],
  browser: "firefox",
  barPosition: "top",
  idleLock: 300,
  stayAwake: false,
  hyprLook: { gapsIn: 4, activeOpacity: 1 },
  hyprInput: { naturalScroll: false, sensitivity: 0 },
  sshdEnabled: true,
  passwordlessSudo: true,
  timezone: "America/New_York",
  timezones: ["America/New_York", "Europe/Berlin"],
};

const exported = settings.exportMarkdown(s_snapshot, ["theme", "browser", "hyprLook.gapsIn"], {
  exported: "2026-09-03T00:00:00Z",
  hardware: "abc123",
});
assert(exported.indexOf("# Atmos settings") === 0, "exportMarkdown opens with a heading");
assert(exported.indexOf("```toml atmos:meta") !== -1, "exportMarkdown writes a meta block");
assert(exported.indexOf('theme = "tokyo-night"') !== -1, "exportMarkdown writes a selected string");
assert(exported.indexOf("hyprLook.gapsIn = 4") !== -1, "exportMarkdown writes a dotted key");
assert(exported.indexOf("barPosition") === -1, "exportMarkdown leaves out unselected keys");
assert(
  exported.indexOf("- SSH server: on") !== -1,
  "exportMarkdown reports security state as prose",
);
assert(
  exported.indexOf("sshdEnabled =") === -1,
  "exportMarkdown never puts a security setting in a block",
);

const round = settings.parseSettingsMarkdown(exported);
assertEqual(round.errors.length, 0, "a file Atmos wrote reads back without errors");
assertEqual(round.meta.schema, 1, "parseSettingsMarkdown reads the schema");
assertEqual(round.meta.hardware, "abc123", "parseSettingsMarkdown reads the hardware fingerprint");
assertEqual(round.sections.appearance.theme, "tokyo-night", "parseSettingsMarkdown reads a value");
assertEqual(
  round.sections.windows["hyprLook.gapsIn"],
  4,
  "parseSettingsMarkdown reads a dotted key",
);
assertEqual(round.sections.security, undefined, "the reported security prose is not a block");

const proseSafe = settings.parseSettingsMarkdown(
  '# Notes\n\nkey = "not in a block"\n\n```toml atmos:appearance\ntheme = "catppuccin"\n```\n',
);
assertEqual(
  proseSafe.sections.appearance.theme,
  "catppuccin",
  "parseSettingsMarkdown reads only fenced blocks",
);
assertEqual(Object.keys(proseSafe.sections).length, 1, "prose outside a block is ignored");

const unclosed = settings.parseSettingsMarkdown('```toml atmos:appearance\ntheme = "x"\n');
assert(unclosed.errors.length === 1, "an unclosed block is an error");

const badValues = settings.parseSettingsMarkdown(
  '```toml atmos:appearance\nnope\ntheme = \n1bad = "x"\n```\n',
);
assertEqual(badValues.errors.length, 3, "parseSettingsMarkdown reports each unreadable line");

assertEqual(settings.parseTomlValue("true"), true, "parseTomlValue reads a boolean");
assertEqual(settings.parseTomlValue("-12"), -12, "parseTomlValue reads a negative integer");
assertEqual(settings.parseTomlValue("0.5"), 0.5, "parseTomlValue reads a float");
assertEqual(
  settings.parseTomlValue('"a \\"b\\" c"'),
  'a "b" c',
  "parseTomlValue unescapes a quoted string",
);
assertEqual(settings.parseTomlValue("[]").length, 0, "parseTomlValue reads an empty list");
assertEqual(
  settings.parseTomlValue('["a", "b"]')[1],
  "b",
  "parseTomlValue reads a list of strings",
);
assertEqual(settings.parseTomlValue("bare"), undefined, "parseTomlValue refuses a bare word");

function planFor(body, keys, options) {
  return settings.planImport(settings.parseSettingsMarkdown(body), s_snapshot, keys, options);
}

const plan = planFor(
  "```toml atmos:meta\nschema = 1\n```\n" +
    '```toml atmos:appearance\ntheme = "catppuccin"\nfont = "CaskaydiaMono Nerd Font"\n```\n' +
    '```toml atmos:defaults\nbrowser = "brave"\n```\n',
);
assertEqual(plan.changes.length, 2, "planImport counts only real changes");
assertEqual(plan.unchanged.length, 1, "planImport reports a value that already matches");
assertEqual(plan.changes[0].key, "browser", "planImport sorts changes by key");
assertEqual(plan.changes[0].from, "firefox", "planImport records the current value");
assertEqual(plan.changes[0].to, "brave", "planImport records the incoming value");
assert(plan.changes[0].consequence.length > 0, "a behavior change carries its consequence");
assertEqual(plan.summary, "2 changes, 1 already match", "planImport summarises the plan");

const singular = planFor(
  "```toml atmos:meta\nschema = 1\n```\n" + '```toml atmos:appearance\ntheme = "catppuccin"\n```\n',
);
assertEqual(singular.summary, "1 change", 'planImport says "1 change" for one change');

const unknownTheme = planFor('```toml atmos:appearance\ntheme = "does-not-exist"\n```\n');
assertEqual(unknownTheme.changes.length, 0, "a theme this machine lacks is not a change");
assertEqual(unknownTheme.blocked.length, 1, "a theme this machine lacks is blocked");
assert(
  unknownTheme.blocked[0].reason.indexOf("not available on this machine") !== -1,
  "the block says the value is not available here",
);

const security = planFor(
  "```toml atmos:security\nsshdEnabled = true\npasswordlessSudo = true\n```\n",
);
assertEqual(security.changes.length, 0, "a security block produces no changes");
assertEqual(security.blocked.length, 2, "every security key is blocked");
assert(
  security.blocked[0].reason.indexOf("never imports security settings") !== -1,
  "the block says security settings are never imported",
);

const wrongType = planFor('```toml atmos:idle\nidleLock = "soon"\n```\n');
assertEqual(wrongType.blocked.length, 1, "a value of the wrong type is blocked");
assert(
  wrongType.blocked[0].reason.indexOf("expects integer") !== -1,
  "the block names the expected type",
);

const unknownKey = planFor('```toml atmos:appearance\nmadeUpSetting = "x"\n```\n');
assertEqual(unknownKey.warnings.length, 2, "an unknown key warns, alongside the missing schema");
assert(
  unknownKey.warnings[1].message.indexOf("does not know this setting") !== -1,
  "the warning says the setting is unknown",
);

const wrongSection = planFor('```toml atmos:appearance\nbrowser = "brave"\n```\n');
assertEqual(wrongSection.changes.length, 0, "a key in the wrong section is not applied");
assert(
  wrongSection.warnings[1].message.indexOf("belongs to defaults") !== -1,
  "the warning names the section the key belongs to",
);

const future = planFor(
  '```toml atmos:meta\nschema = 99\n```\n```toml atmos:appearance\ntheme = "catppuccin"\n```\n',
);
assertEqual(future.changes.length, 0, "a newer schema stops the plan");
assert(
  future.blocked[0].reason.indexOf("Update Atmos first") !== -1,
  "a newer schema asks you to update Atmos",
);

const otherMachine = planFor(
  '```toml atmos:meta\nschema = 1\nhardware = "aaa"\n```\n' +
    "```toml atmos:input\nhyprInput.naturalScroll = true\nhyprInput.sensitivity = 0.4\n```\n",
  null,
  { hardware: "bbb" },
);
assertEqual(
  otherMachine.changes.length,
  1,
  "a device-tuned setting is held back on other hardware",
);
assertEqual(
  otherMachine.changes[0].key,
  "hyprInput.naturalScroll",
  "a portable input setting still applies",
);
assertEqual(
  otherMachine.warnings.length,
  2,
  "different hardware warns once for the file and once for the setting",
);

const sameMachine = planFor(
  '```toml atmos:meta\nschema = 1\nhardware = "aaa"\n```\n' +
    "```toml atmos:input\nhyprInput.sensitivity = 0.4\n```\n",
  null,
  { hardware: "aaa" },
);
assertEqual(sameMachine.changes.length, 1, "the same hardware keeps device-tuned settings");

const selected = planFor(
  "```toml atmos:meta\nschema = 1\n```\n" +
    '```toml atmos:appearance\ntheme = "catppuccin"\n```\n' +
    '```toml atmos:defaults\nbrowser = "brave"\n```\n',
  ["theme"],
);
assertEqual(selected.changes.length, 1, "planImport honours a selection");
assertEqual(selected.changes[0].key, "theme", "the selection keeps only the chosen key");

const encoded = JSON.parse(settings.planToJson(plan));
assertEqual(encoded.schema, 1, "planToJson stamps the schema");
assertEqual(encoded.changes.length, 2, "planToJson carries every change");
assertEqual(encoded.changes[0].key, "browser", "planToJson keeps the plan order");

// The s_catalog lives in JS and the writers live in bash. Nothing stops those
// two from drifting apart except this check.
const applyScript = fs.readFileSync(
  path.join(__dirname, "..", "scripts/apply-settings.sh"),
  "utf8",
);
const dispatch = applyScript.slice(
  applyScript.indexOf("while IFS= read -r key; do"),
  applyScript.indexOf("done < <(jq -r "),
);
assert(dispatch.length > 0, "apply-settings.sh has a dispatch loop");

const labels = [];
for (const line of dispatch.split("\n")) {
  const match = line.match(/^ {4}(\S[^)(]*?)\)/);
  if (!match) continue;
  for (const part of match[1].split("|")) labels.push(part.trim());
}
assert(labels.indexOf("*") !== -1, "the dispatcher refuses an unknown key");

function dispatched(key) {
  if (labels.indexOf(key) !== -1) return true;
  for (const label of labels) {
    if (label.slice(-2) !== ".*") continue;
    if (key.indexOf(label.slice(0, -2) + ".") === 0) return true;
  }
  return false;
}

let undispatched = "";
for (const item of s_catalog) {
  if (item.importable && !dispatched(item.key)) undispatched = item.key;
}
assertEqual(undispatched, "", "every importable setting has a writer in apply-settings.sh");

let orphanLabel = "";
for (const label of labels) {
  if (label === "*") continue;
  const key = label.slice(-2) === ".*" ? label.slice(0, -2) + ".gapsIn" : label;
  if (!byKey[label] && !byKey[key] && label.indexOf(".*") === -1) orphanLabel = label;
}
assertEqual(orphanLabel, "", "apply-settings.sh has no writer for a setting the s_catalog dropped");

// Lists: whole-list settings carried in a json block named for the setting.
assertEqual(byKey.bindings.kind, "list", "bindings is a list setting");
assertEqual(byKey.bindings.section, "bindings", "a list setting is its own section");
assert(byKey.bindings.consequence.length > 0, "replacing your keybindings explains itself");
assertEqual(byKey.autostart.kind, "list", "autostart is a list setting");
assertEqual(byKey.windowRules.kind, "list", "windowRules is a list setting");

const listSnapshot = {
  bindings: [
    { keys: "SUPER + RETURN", label: "Terminal", command: "kitty", unbind: false, managed: true },
    { keys: "SUPER + J", label: "", command: "", unbind: true, managed: false },
  ],
  autostart: [{ command: "solaar --window=hide", managed: false }],
  windowRules: [],
};

const listMd = settings.exportMarkdown(listSnapshot, ["bindings", "autostart"], {
  hardware: "aaa",
});
assert(
  listMd.indexOf("```json atmos:bindings") !== -1,
  "exportMarkdown writes a json block for a list",
);
assert(listMd.indexOf("## Keybindings") !== -1, "a list setting gets its own heading");
assert(listMd.indexOf('"SUPER + RETURN"') !== -1, "the exported list keeps the binding");

const listBack = settings.parseSettingsMarkdown(listMd);
assertEqual(listBack.errors.length, 0, "a list file reads back without errors");
assertEqual(
  listBack.sections.bindings.bindings.length,
  2,
  "parseSettingsMarkdown reads the whole list",
);
assertEqual(
  listBack.sections.bindings.bindings[0].keys,
  "SUPER + RETURN",
  "parseSettingsMarkdown keeps row shape",
);

const listReplay = settings.planImport(listBack, listSnapshot, null, { hardware: "aaa" });
assertEqual(listReplay.changes.length, 0, "exporting a list and replaying it changes nothing");
assertEqual(listReplay.unchanged.length, 2, "both lists already match");

const listChanged = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:bindings\n[{"keys":"SUPER + T","label":"","command":"kitty","unbind":false}]\n```\n',
  ),
  listSnapshot,
  null,
  {},
);
assertEqual(listChanged.changes.length, 1, "a different list is one change");
assertEqual(listChanged.changes[0].from.length, 2, "the change carries the list being replaced");
assertEqual(listChanged.changes[0].to.length, 1, "the change carries the incoming list");
assertEqual(settings.displayValue(listChanged.changes[0].to), "1 entry", "a list reads as a count");
const listDiff = settings.changeLines(listChanged);
assert(
  listDiff.indexOf("SUPER + T → kitty") !== -1,
  "changeLines shows the incoming binding command",
);
assert(
  listDiff.indexOf("SUPER + RETURN → kitty") !== -1,
  "changeLines shows the binding being replaced",
);
assert(settings.hasCommandImport(listChanged), "replacing bindings is a command import");
assertEqual(
  settings.commandImportLines(listChanged)[0],
  "kitty",
  "commandImportLines lists the binding command",
);

const autostartPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:autostart\n[{"command":"curl example.com | sh"}]\n```\n',
  ),
  { autostart: [] },
  null,
  {},
);
assert(settings.hasCommandImport(autostartPlan), "replacing autostart is a command import");
assert(
  settings.changeLines(autostartPlan).indexOf("curl example.com | sh") !== -1,
  "changeLines shows the startup command",
);
assert(
  settings.commandConfirmMessage(autostartPlan).indexOf("curl example.com | sh") !== -1,
  "commandConfirmMessage lists the startup command",
);

const rulesPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:windowRules\n[{"match":"firefox","placement":"float"}]\n```\n',
  ),
  { windowRules: [] },
  null,
  {},
);
assert(
  settings.changeLines(rulesPlan).indexOf("firefox, float") !== -1,
  "changeLines shows the window rule matcher",
);
assertEqual(settings.hasCommandImport(rulesPlan), false, "window rules are not a command import");

const notAList = settings.parseSettingsMarkdown(
  '```json atmos:bindings\n{"keys":"SUPER + T"}\n```\n',
);
assertEqual(notAList.errors.length, 1, "a json block that is not a list is an error");
assert(notAList.errors[0].indexOf("is not a list") !== -1, "the error says it is not a list");

const badJson = settings.parseSettingsMarkdown('```json atmos:bindings\n[{"keys":\n```\n');
assertEqual(badJson.errors.length, 1, "unreadable JSON is an error");
assert(
  badJson.errors[0].indexOf("not readable JSON") !== -1,
  "the error says the JSON is unreadable",
);

const listWrongType = settings.planImport(
  settings.parseSettingsMarkdown(
    '```toml atmos:meta\nschema = 1\n```\n```toml atmos:bindings\nbindings = "nope"\n```\n',
  ),
  listSnapshot,
  null,
  {},
);
assertEqual(listWrongType.blocked.length, 1, "a list setting given a string is blocked");

// Sections, for a UI that offers sections rather than one switch per setting.
const appearanceKeys = settings.sectionKeys("appearance", s_catalog);
assert(appearanceKeys.indexOf("theme") !== -1, "sectionKeys finds the theme in appearance");
assert(appearanceKeys.indexOf("browser") === -1, "sectionKeys keeps sections apart");
assertEqual(
  settings.sectionKeys("security", s_catalog).length,
  0,
  "the security section offers no importable keys",
);

const selectable = settings.selectableSections(s_catalog);
let securityOffered = false;
let emptyOffered = "";
for (const section of selectable) {
  if (section.id === "security") securityOffered = true;
  if (section.count === 0) emptyOffered = section.id;
}
assertEqual(securityOffered, false, "the security section is never offered as a switch");
assertEqual(emptyOffered, "", "no empty section is offered");
assert(selectable.length >= 8, "every section holding settings is offered");

let identityDefault = true;
let appearanceDefault = false;
for (const section of selectable) {
  if (section.id === "system") identityDefault = section.byDefault;
  if (section.id === "appearance") appearanceDefault = section.byDefault;
}
assertEqual(identityDefault, false, "machine identity starts switched off");
assertEqual(appearanceDefault, true, "appearance starts switched on");

const twoSections = settings.keysForSections(["appearance", "defaults"], s_catalog);
assert(twoSections.indexOf("theme") !== -1, "keysForSections takes the first section");
assert(twoSections.indexOf("browser") !== -1, "keysForSections takes the second section");
assert(twoSections.indexOf("hostname") === -1, "keysForSections leaves out what was not asked for");
assertEqual(settings.keysForSections([], s_catalog).length, 0, "no sections means no keys");

let sectionTotal = 0;
for (const section of selectable) sectionTotal += section.count;
assertEqual(
  sectionTotal,
  settings.presetKeys("full", s_catalog).length,
  "the sections between them cover every importable setting",
);

// A sentinel writer leaves hand-written rows where they are, so importing a
// list Atmos does not already own leaves two copies in the file.
const handWritten = {
  bindings: [
    { keys: "SUPER + D", label: "Desks", command: "desk", unbind: false, managed: false },
    { keys: "SUPER + B", label: "Brave", command: "brave", unbind: false, managed: false },
  ],
};
const shadowPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:bindings\n[{"keys":"SUPER + T","label":"","command":"kitty","unbind":false}]\n```\n',
  ),
  handWritten,
  null,
  {},
);
assertEqual(shadowPlan.changes.length, 1, "replacing hand-written bindings is still one change");
assertEqual(shadowPlan.warnings.length, 1, "replacing hand-written bindings warns");
assert(
  shadowPlan.warnings[0].message.indexOf("2 rows") === 0,
  "the warning counts the hand-written rows",
);
assert(
  shadowPlan.warnings[0].message.indexOf("written by hand") !== -1,
  "the warning says the rows are hand-written",
);

const alreadyManaged = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:bindings\n[{"keys":"SUPER + T","label":"","command":"kitty","unbind":false}]\n```\n',
  ),
  { bindings: [{ keys: "SUPER + D", label: "", command: "desk", unbind: false, managed: true }] },
  null,
  {},
);
assertEqual(alreadyManaged.warnings.length, 0, "rows Atmos already owns need no warning");

// Plan rendering lives in the service so the wording stays under test.
const renderPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:defaults\nbrowser = "brave"\n```\n' +
      "```toml atmos:security\nsshdEnabled = true\n```\n",
  ),
  { browser: "firefox", sshdEnabled: false },
  null,
  {},
);
const changeText = settings.changeLines(renderPlan);
assert(changeText.indexOf("• Browser: firefox → brave") === 0, "changeLines reads as a sentence");
assert(
  changeText.indexOf("\n    Every link") !== -1,
  "changeLines indents the consequence under the change",
);
assert(settings.blockedLines(renderPlan).indexOf("• ") === 0, "blockedLines bullets each refusal");
assertEqual(settings.changeLines(null), "", "changeLines survives no plan");
assertEqual(settings.warningLines(null), "", "warningLines survives no plan");
assertEqual(settings.blockedLines(null), "", "blockedLines survives no plan");

assertEqual(settings.shownValue(""), "not set", "an empty value reads as not set");
assertEqual(settings.shownValue(null), "not set", "a missing value reads as not set");
assertEqual(settings.shownValue(false), "off", "false reads as off, not as missing");
assertEqual(settings.shownValue(0), "0", "zero reads as zero, not as missing");

// PrefsRow's control slot takes its size from children[0], so a second
// direct child is drawn on top of the first. Wrap them in a Row instead.
function overlappingControlRows() {
  const found = [];
  const dirs = ["pages", "components"];
  for (const dir of dirs) {
    const full = path.join(__dirname, "..", dir);
    for (const name of fs.readdirSync(full)) {
      if (!name.endsWith(".qml")) continue;
      const lines = fs.readFileSync(path.join(full, name), "utf8").split("\n");
      for (let i = 0; i < lines.length; i++) {
        if (!/^\s*PrefsRow\s*\{/.test(lines[i])) continue;
        const base = lines[i].length - lines[i].trimStart().length;
        let depth = 1;
        const kids = [];
        for (let j = i + 1; j < lines.length && depth > 0; j++) {
          const line = lines[j];
          const indent = line.length - line.trimStart().length;
          const child = line.match(/^\s*([A-Z][A-Za-z0-9_]*)\s*\{/);
          if (child && depth === 1 && indent === base + 2) kids.push(child[1]);
          depth += line.split("{").length - 1 - (line.split("}").length - 1);
        }
        if (kids.length > 1) found.push(`${dir}/${name}:${i + 1} (${kids.join(", ")})`);
      }
    }
  }
  return found;
}

const overlapping = overlappingControlRows();
assertEqual(overlapping.join(" "), "", "no PrefsRow stacks two controls on top of each other");

// An exported file should say what it is, where it came from, and when.
const when = new Date(2026, 8, 3, 19, 30);
assertEqual(
  settings.exportFileName("vic", when),
  "atmos-export-vic-2026-09-03-1930.md",
  "exportFileName names the machine and the moment",
);
assertEqual(
  settings.exportFileName("Fred's Laptop!", when),
  "atmos-export-fred-s-laptop-2026-09-03-1930.md",
  "exportFileName makes an awkward hostname into a filename",
);
assertEqual(
  settings.exportFileName("", when),
  "atmos-export-2026-09-03-1930.md",
  "exportFileName drops the host when there is not one",
);
assertEqual(
  settings.exportFileName("---", when),
  "atmos-export-2026-09-03-1930.md",
  "a hostname of only punctuation leaves no stray dashes",
);
assertEqual(
  settings.exportFileName("vic", new Date(2026, 0, 7, 4, 5)),
  "atmos-export-vic-2026-01-07-0405.md",
  "exportFileName pads single digits so the names sort",
);
assert(
  /^atmos-export-vic-\d{4}-\d{2}-\d{2}-\d{4}\.md$/.test(settings.exportFileName("vic", null)),
  "exportFileName falls back to now when given no date",
);
assert(
  /^atmos-export-vic-\d{4}/.test(settings.exportFileName("vic", new Date("nonsense"))),
  "exportFileName survives an unusable date",
);
assertEqual(settings.fileSafe("a".repeat(80)).length, 40, "a very long hostname is trimmed");

// A machine that cannot list its own options should not have every value
// refused, but the check looking exact when it was never made is worse.
const cannotCheck = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:appearance\ntheme = "anything-at-all"\n```\n',
  ),
  { theme: "Unknown", themes: [] },
  null,
  {},
);
assertEqual(cannotCheck.blocked.length, 0, "an unlistable option is not blocked outright");
assertEqual(cannotCheck.changes.length, 1, "an unlistable option still plans the change");
assertEqual(cannotCheck.warnings.length, 1, "an unlistable option warns that it was not checked");
assert(
  cannotCheck.warnings[0].message.indexOf("could not be checked") !== -1,
  "the warning says the value was not checked",
);

// A plan says what would change. This says what you are about to trust.
const provenance = settings.parseSettingsMarkdown(
  '```toml atmos:meta\nschema = 1\nhostname = "vic"\nexported = "2026-09-03T20:06:11.000Z"\nhardware = "Victus by HP"\n```\n' +
    '```toml atmos:appearance\ntheme = "catppuccin"\n```\n' +
    '```toml atmos:defaults\nbrowser = "brave"\n```\n',
);
const provPlan = settings.planImport(
  provenance,
  { theme: "catppuccin", browser: "firefox" },
  null,
  {},
);
const summaryText = settings.fileSummary(provenance, provPlan);
assert(summaryText.indexOf("From vic") !== -1, "fileSummary names the machine it came from");
assert(
  summaryText.indexOf("2026-09-03 20:06:11") !== -1,
  "fileSummary reads the export time plainly",
);
assert(summaryText.indexOf("Victus by HP") !== -1, "fileSummary names the hardware");
assert(
  summaryText.indexOf("2 sections: appearance, defaults") !== -1,
  "fileSummary lists the sections",
);
assert(
  summaryText.indexOf("1 of these already match") !== -1,
  "fileSummary counts what already matches",
);

const anonymous = settings.parseSettingsMarkdown('```toml atmos:appearance\ntheme = "x"\n```\n');
const anonText = settings.fileSummary(anonymous, null);
assert(
  anonText.indexOf("1 section: appearance") !== -1,
  "fileSummary still describes a file with no meta",
);
assertEqual(settings.fileSummary(null, null), "", "fileSummary survives no document");
// A value the machine already holds needs no permission to stay. Plymouth
// reports "default" as its theme while the installable themes list does not
// contain it, so validating before comparing refused the machine its own
// setting and every export of that machine came back with one blocked row.
const holdsOwnValue = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:appearance\nplymouth = "default"\n```\n',
  ),
  { plymouth: "default", plymouthThemes: ["catppuccin", "everforest"] },
  null,
  {},
);
assertEqual(holdsOwnValue.blocked.length, 0, "a value the machine already holds is never blocked");
assertEqual(holdsOwnValue.unchanged.length, 1, "a value the machine already holds is unchanged");

// Changing to something genuinely absent is still refused.
const wantsMissing = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:appearance\nplymouth = "not-installed"\n```\n',
  ),
  { plymouth: "default", plymouthThemes: ["catppuccin", "everforest"] },
  null,
  {},
);
assertEqual(wantsMissing.blocked.length, 1, "a value the machine does not have is still blocked");

// Before pressing apply: how much, and whether it will interrupt you.
const rootPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:system\nhostname = "elsewhere"\ntimezone = "Europe/Berlin"\n```\n' +
      '```toml atmos:defaults\nbrowser = "brave"\n```\n',
  ),
  {
    hostname: "vic",
    timezone: "America/New_York",
    timezones: ["Europe/Berlin"],
    browser: "firefox",
  },
  null,
  {},
);
assertEqual(rootPlan.changes.length, 3, "three changes planned");
assertEqual(settings.passwordCount(rootPlan), 2, "two of them raise privileges");
const forecast = settings.applyForecast(rootPlan);
assert(forecast.indexOf("3 changes") === 0, "the forecast leads with the count");
assert(forecast.indexOf("2 changes need root") !== -1, "the forecast counts what needs root");
assert(
  forecast.indexOf("asks for sudo mode once") !== -1,
  "the forecast says Atmos asks once rather than per change",
);

const quietPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    '```toml atmos:meta\nschema = 1\n```\n```toml atmos:defaults\nbrowser = "brave"\n```\n',
  ),
  { browser: "firefox" },
  null,
  {},
);
assertEqual(settings.passwordCount(quietPlan), 0, "a cosmetic plan asks for nothing");
assert(
  settings.applyForecast(quietPlan).indexOf("password") === -1,
  "a plan that needs no password does not mention one",
);
assertEqual(settings.applyForecast({ changes: [] }), "", "no changes, no forecast");

const applyJson =
  "progress\t1\t2\ttheme\n" +
  '{"backup":"/tmp/imports/one","results":[{"key":"theme","status":"applied"},{"key":"font","status":"failed"}]}';
const applyResult = settings.parseApplyResult(applyJson);
assertEqual(applyResult.backup, "/tmp/imports/one", "parseApplyResult reads the backup path");
assertEqual(
  settings.appliedCountFromResult(applyResult),
  1,
  "appliedCountFromResult skips failures",
);
assertEqual(
  settings.backupDirFromResult(applyResult),
  "/tmp/imports/one",
  "backupDirFromResult reads backup",
);
assertEqual(
  settings.parseApplyResult("progress\t1\t1\ttheme").backup,
  "",
  "parseApplyResult survives no JSON",
);
