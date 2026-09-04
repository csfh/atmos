const { load, assert, assertEqual } = require("./harness");

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
