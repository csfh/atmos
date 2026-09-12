const { load, assert, assertEqual } = require("./harness");

const hist = load("services/History.js");

const themeWrap = [
  "bash",
  "-c",
  '"$@" >/dev/null 2>&1 &',
  "theme-set",
  "omarchy",
  "theme",
  "set",
  "nord",
];
assertEqual(
  hist.describe(themeWrap),
  "omarchy theme set nord",
  "unwrap drops the theme detach wrapper",
);

const shiftWrap = [
  "bash",
  "-c",
  'PATH="$1:$PATH"; shift; exec "$@"',
  "prefs-job",
  "/tmp/atmos-stubs",
  "omarchy",
  "theme",
  "set",
  "nord",
];
assertEqual(
  hist.describe(shiftWrap),
  "omarchy theme set nord",
  "unwrap skips the stub directory a shift removes",
);
assert(
  hist.unwrap(shiftWrap).indexOf("/tmp/atmos-stubs") === -1,
  "shift wrapper does not display the stub directory",
);

const execOne = ["bash", "-c", 'exec "$1"', "prefs-one", "omarchy", "restart", "shell"];
assertEqual(hist.describe(execOne), "omarchy restart shell", 'unwrap matches exec "$1"');

assertEqual(
  hist.describe(["bash", "-c", "echo execute me", "name", "not-the-command"]),
  '"echo execute me" name not-the-command',
  "a script that only mentions exec is not treated as a wrapper",
);

assertEqual(hist.describe(null), "", "describe junk null");
assertEqual(hist.describe(undefined), "", "describe junk undefined");
assertEqual(hist.describe({}), "", "describe junk object");
assertEqual(
  hist.describe(["omarchy", null, "theme"]),
  "omarchy  theme",
  "describe keeps a hole for null argv",
);
assertEqual(hist.unwrap("omarchy").length, 0, "unwrap junk string is empty");
assertEqual(hist.targetFile(["echo", "hi"], "/home/pi"), "", "targetFile junk argv is empty");

assertEqual(
  hist.targetFile(["write", "/home/pieter/.config/hypr/looknfeel.lua"], "/home/pi"),
  "/home/pieter/.config/hypr/looknfeel.lua",
  "/home/pi is not a prefix of /home/pieter",
);
assertEqual(
  hist.targetFile(["write", "/home/pi/.config/hypr/looknfeel.lua"], "/home/pi"),
  "~/.config/hypr/looknfeel.lua",
  "/home/pi plus slash rewrites to ~",
);
assertEqual(
  hist.targetFile(["cat", "~/.config/hypr/hyprland.lua"], "/home/pieter"),
  "~/.config/hypr/hyprland.lua",
  "targetFile keeps a ~ path",
);
assert(
  hist.targetFile(["open", "~/.config/hypr"], "/home/pi") === "",
  "unanchored .conf does not steal ~/.conf from ~/.config",
);
assertEqual(
  hist.targetFile(["edit", "/tmp/theme.jsonc"], "/home/pi"),
  "",
  ".json does not eat the c off .jsonc",
);

const now = 1_700_000_000_000;
assertEqual(hist.relativeTime(null, now), "", "relativeTime null is empty");
assertEqual(hist.relativeTime(undefined, now), "", "relativeTime undefined is empty");
assertEqual(hist.relativeTime("", now), "", "relativeTime empty string is empty");
assertEqual(hist.relativeTime(0, now), "", "relativeTime 0 is empty, not 1970");
assertEqual(hist.relativeTime("0", now), "", "relativeTime string 0 is empty");
assertEqual(hist.relativeTime(false, now), "", "relativeTime false is empty");
assertEqual(hist.relativeTime(now - 4000, now), "just now", "relativeTime under 10s");
assertEqual(hist.relativeTime(now - 15000, now), "15s ago", "relativeTime seconds");
assertEqual(hist.relativeTime(now - 5 * 60 * 1000, now), "5m ago", "relativeTime minutes");
assertEqual(hist.relativeTime(now - 3 * 60 * 60 * 1000, now), "3h ago", "relativeTime hours");
assertEqual(hist.relativeTime(now - 48 * 60 * 60 * 1000, now), "2d ago", "relativeTime days");

const counts = hist.countsBySource([
  { source: "you" },
  { source: "constructor" },
  { source: "toString" },
  { source: "constructor" },
  {},
]);
assertEqual(hist.countFor(counts, "you"), 2, "missing source counts as you");
assertEqual(
  hist.countFor(counts, "constructor"),
  2,
  "prototype-named source constructor is counted",
);
assertEqual(hist.countFor(counts, "toString"), 1, "prototype-named source toString is counted");
assertEqual(hist.countFor(counts, "missing"), 0, "countFor misses");
assertEqual(hist.countFor(null, "you"), 0, "countFor null counts");

let list = [];
for (let i = 0; i < hist.MAX_ENTRIES + 5; i++) {
  list = hist.push(list, { at: i, key: String(i) });
}
assertEqual(list.length, hist.MAX_ENTRIES, "push caps at MAX_ENTRIES");
assertEqual(list[0].key, String(hist.MAX_ENTRIES + 4), "push keeps the newest first");
assertEqual(list[list.length - 1].key, "5", "push drops the oldest past the cap");

const replayed = hist.replayOpts({
  key: "theme",
  apply: { theme: "nord" },
  refresh: "none",
  sudo: true,
  stdin: "secret\n",
  payload: { raw: 1 },
});
assertEqual(replayed.bypassPreview, true, "replayOpts sets bypassPreview");
assertEqual(replayed.key, "theme", "replayOpts keeps key");
assertEqual(replayed.apply.theme, "nord", "replayOpts keeps apply");
assertEqual(replayed.refresh, "none", "replayOpts keeps refresh");
assertEqual(replayed.sudo, true, "replayOpts keeps sudo");
assertEqual(replayed.stdin, "secret\n", "replayOpts keeps stdin");
assertEqual(replayed.payload.raw, 1, "replayOpts keeps payload");

const original = { key: "hyprLook", bypassPreview: false };
const copied = hist.replayOpts(original);
assertEqual(copied.bypassPreview, true, "replayOpts overrides a false bypass");
assertEqual(original.bypassPreview, false, "replayOpts does not mutate the held opts");

const calls = [];
hist.applyHeld(
  [
    { argv: ["omarchy", "theme", "set", "nord"], opts: { key: "theme", apply: { theme: "nord" } } },
    {
      argv: ["bash", "set-idle.sh", "300", "600"],
      opts: { key: "idle", refresh: "none", stdin: "" },
    },
    { text: "omarchy theme set rose", opts: { key: "missing-argv" } },
    { argv: [], opts: { key: "empty" } },
    null,
  ],
  function (argv, opts) {
    calls.push({ argv: argv, opts: opts });
  },
);
assertEqual(calls.length, 2, "applyHeld skips missing or empty argv");
assertEqual(calls[0].argv.join(" "), "bash set-idle.sh 300 600", "applyHeld replays oldest first");
assertEqual(
  calls[1].argv.join(" "),
  "omarchy theme set nord",
  "applyHeld then replays the newer hold",
);
assertEqual(calls[0].opts.bypassPreview, true, "applyHeld bypasses preview on the first replay");
assertEqual(calls[1].opts.bypassPreview, true, "applyHeld bypasses preview on every replay");
assertEqual(calls[0].opts.key, "idle", "applyHeld forwards key");
assertEqual(calls[0].opts.refresh, "none", "applyHeld forwards refresh");
assertEqual(calls[0].opts.stdin, "", "applyHeld forwards stdin");
assertEqual(calls[1].opts.apply.theme, "nord", "applyHeld forwards apply");

hist.applyHeld(null, function () {
  calls.push({ argv: ["should-not-run"] });
});
assertEqual(calls.length, 2, "applyHeld on junk held list is a no-op");
