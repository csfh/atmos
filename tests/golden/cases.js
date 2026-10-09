// The protocol cases. Each one builds its request with the same Requests.js
// builder the app uses, runs it against a fresh fixture root, and is compared
// with the reply recorded in tests/golden/cases/<name>.json.
//
// Under a fixture root the backend re-roots every path, absolute ones too, so a
// request names "/note.txt" and means <root>/note.txt. `<root>` is still
// understood in files and requests and stands for the fixture root; replies
// are stored with it. `files` are written under the root before the case runs,
// and `setup` requests run first to put the backend in the state it needs.
//
// Re-record after an intended protocol change:
//   ATMOS_GOLDEN_RECORD=1 node tests/golden.test.js
// and read the diff: that diff is the protocol change.
const { load } = require("../harness");

const R = load("services/Requests.js");

const cases = [
  // Identity.
  { name: "version", backend: "plain", request: { op: "version" } },
  { name: "platform-plain", backend: "plain", request: { op: "platform" } },
  { name: "platform-omarchy", backend: "omarchy", request: { op: "platform" } },

  // Settings.
  { name: "settings-list-omarchy", backend: "omarchy", request: { op: "settings.list" } },
  { name: "settings-list-plain", backend: "plain", request: { op: "settings.list" } },
  { name: "settings-get-unset", backend: "plain", request: R.settingsGet("theme") },
  {
    name: "settings-set-string",
    backend: "plain",
    request: R.settingsSet("theme", "tokyo-night"),
  },
  {
    name: "settings-set-int-omarchy",
    backend: "omarchy",
    request: R.settingsSet("hyprLook.gapsIn", 7),
  },
  {
    name: "settings-set-bool-omarchy",
    backend: "omarchy",
    request: R.settingsSet("hyprLook.blur", true),
  },
  {
    name: "settings-set-number-omarchy",
    backend: "omarchy",
    request: R.settingsSet("hyprLook.activeOpacity", 0.85),
  },
  {
    name: "settings-get-after-set",
    backend: "omarchy",
    setup: [R.settingsSet("hyprLook.gapsOut", 12)],
    request: R.settingsGet("hyprLook.gapsOut"),
  },
  {
    name: "settings-snapshot-all-plain",
    backend: "plain",
    setup: [R.settingsSet("theme", "tokyo-night"), R.settingsSet("hyprLook.gapsIn", 4)],
    request: R.settingsSnapshot(),
  },
  {
    name: "settings-snapshot-look-omarchy",
    backend: "omarchy",
    setup: [R.settingsSet("hyprLook.gapsIn", 4), R.settingsSet("hyprLook.blur", false)],
    request: R.settingsSnapshot("look"),
  },
  {
    name: "settings-snapshot-keys-plain",
    backend: "plain",
    setup: [R.settingsSet("theme", "tokyo-night"), R.settingsSet("font", "Iosevka")],
    request: R.settingsSnapshot("all", ["theme"]),
  },

  // Display documents.
  {
    name: "display-get-live-fixture",
    backend: "omarchy",
    files: { ".local/state/omarchy/display/live.json": '{"cpu":{"load":0.5}}\n' },
    request: R.displayGet("live"),
  },
  {
    name: "unit-output-user-scope",
    backend: "plain",
    request: R.unitOutput("status", "user", "pipewire.service"),
  },
  { name: "display-get-plain", backend: "plain", request: R.displayGet("hardware") },
  { name: "display-snapshot-plain", backend: "plain", request: R.displaySnapshot() },

  // Host reads and writes.
  { name: "host-chrome-plain", backend: "plain", request: R.hostChrome() },
  {
    name: "host-chrome-omarchy",
    backend: "omarchy",
    files: {
      ".local/state/omarchy/current/theme.name": "tokyo-night\n",
      ".local/state/omarchy/current/theme/colors.toml": 'accent = "#7aa2f7"\n',
      ".local/state/omarchy/current/theme/shell.toml": "[bar]\nradius = 4\n",
    },
    request: R.hostChrome(),
  },
  {
    name: "host-themepack-omarchy",
    backend: "omarchy",
    files: {
      "home/me/.config/omarchy/themes/tokyo-night/colors.toml": 'accent = "#7aa2f7"\n',
      "usr/share/omarchy/themes/tokyo-night/shell.toml": "[bar]\nradius = 4\n",
    },
    request: R.hostThemePack("tokyo-night", "/home/me"),
  },
  {
    // The gallery cards' color dots ask for colors.toml alone.
    name: "host-themepack-colors-only",
    backend: "omarchy",
    files: {
      "home/me/.config/omarchy/themes/tokyo-night/colors.toml": 'accent = "#7aa2f7"\n',
      "usr/share/omarchy/themes/tokyo-night/shell.toml": "[bar]\nradius = 4\n",
    },
    request: R.hostThemePack("tokyo-night", "/home/me", "colors.toml"),
  },
  {
    name: "host-accounts-plain",
    backend: "plain",
    request: R.hostAccounts("plain", "/home/plain"),
  },
  {
    name: "host-accounts-omarchy",
    backend: "omarchy",
    files: {
      "etc/hostname": "atmos-box\n",
      "etc/passwd": "me:x:1000:1000:Me:/home/me:/bin/bash\n",
      "etc/group": "me:x:1000:\n",
      "home/me/.face": "jpeg",
      "var/lib/AccountsService/icons/me": "png",
    },
    request: R.hostAccounts("me", "/home/me"),
  },
  {
    name: "host-stamp",
    backend: "plain",
    files: { "watched.txt": "hello\n" },
    request: R.hostStamp(["/watched.txt", "/missing.txt"]),
  },
  {
    name: "host-read",
    backend: "plain",
    files: { "note.txt": "one\ntwo\n" },
    request: R.hostRead(["/note.txt", "/missing.txt"]),
  },
  {
    name: "host-write",
    backend: "plain",
    request: R.hostWrite("/out/made.txt", "written\n"),
  },
  {
    name: "host-open-file",
    backend: "plain",
    files: { "doc.txt": "x\n" },
    request: R.hostOpen("/doc.txt"),
  },
  { name: "host-open-web", backend: "plain", request: R.hostOpen("https://example.com/") },

  // Fixture-backed command results.
  { name: "speedtest-disk-plain", backend: "plain", request: R.speedtestDisk() },
  {
    name: "speedtest-net-down-omarchy",
    backend: "omarchy",
    files: { ".local/state/omarchy/speed/net-down.txt": "94.2 Mbit/s\n" },
    request: R.speedtestNet(),
  },
  {
    name: "unit-output-plain",
    backend: "plain",
    request: R.unitOutput("status", "", "sshd.service"),
  },

  // Agents.
  { name: "agents-mcp-list", backend: "omarchy", request: R.agentsMcpList() },
  { name: "agents-mcp-check", backend: "omarchy", request: R.agentsMcpCheck() },
  { name: "agents-mcp-set-claude-on", backend: "omarchy", request: R.agentsMcpSet("claude", true) },
  {
    name: "agents-mcp-set-claude-replace",
    backend: "omarchy",
    request: R.agentsMcpSet("claude", true, true),
  },
  {
    name: "agents-mcp-set-claude-off",
    backend: "omarchy",
    request: R.agentsMcpSet("claude", false),
  },
  {
    name: "agents-mcp-set-unknown-agent",
    backend: "omarchy",
    request: R.agentsMcpSet("nope", true),
  },

  // Errors. Each fails the same way for every caller, so the code is the contract.
  { name: "error-unknown-op", backend: "plain", request: { op: "no.such.op" } },
  { name: "error-missing-field", backend: "plain", request: { op: "settings.get" } },
  { name: "error-wrong-type", backend: "plain", request: { op: "host.read", paths: "a" } },
  { name: "error-unknown-domain", backend: "plain", request: R.settingsGet("no.such.domain") },
  {
    name: "error-set-wrong-type",
    backend: "omarchy",
    request: R.settingsSet("hyprLook.gapsIn", "wide"),
  },
  {
    name: "host-read-absolute-path-is-rooted",
    backend: "plain",
    request: R.hostRead(["/etc/passwd"]),
  },
  {
    name: "host-write-absolute-path-is-rooted",
    backend: "plain",
    request: R.hostWrite("/etc/atmos-test", "x"),
  },
  {
    name: "error-read-dot-dot",
    backend: "plain",
    request: R.hostRead(["/../escape"]),
  },
  { name: "error-open-missing", backend: "plain", request: R.hostOpen("/missing.txt") },
  { name: "error-speedtest-bad-phase", backend: "omarchy", request: R.speedtestNet("sideways") },
  { name: "error-display-unknown-kind", backend: "plain", request: R.displayGet("nope") },
];

module.exports = { cases };
