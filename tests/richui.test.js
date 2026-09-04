const { load, assert, assertEqual } = require("./harness");

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
assertEqual(ui.clipboardPayload(""), "", "clipboardPayload empty is empty");
assertEqual(
  ui.clipboardPayload("ok\nline", { singleLine: true }),
  "",
  "clipboardPayload singleLine drops newlines",
);
assertEqual(
  ui.clipboardPayload("ok\nline"),
  "ok\nline",
  "clipboardPayload keeps newlines for lastError",
);
assertEqual(ui.clipboardPayload("ab\0c"), "", "clipboardPayload drops NULs");
assertEqual(ui.clipboardPayload("abcdef", { maxLength: 3 }), "abc", "clipboardPayload caps length");
assertEqual(ui.agentErrorPrompt(""), "", "agentErrorPrompt empty is empty");
assert(
  ui.agentErrorPrompt("boom\nline").indexOf("boom\nline") !== -1,
  "agentErrorPrompt keeps the error body",
);
assert(
  ui.agentErrorPrompt("boom").indexOf("Atmos hit an error") === 0,
  "agentErrorPrompt starts with the Atmos brief",
);

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
