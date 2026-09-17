const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { assert, assertEqual } = require("./harness");

const root = path.join(__dirname, "..");
const launcher = path.join(root, "bin", "atmos");

// bin/atmos ends in `exec quickshell`, so the only way to see what it exported
// is to put a quickshell on PATH that reports instead of launching.
function runLauncher(env) {
  const stub = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-launcher-"));
  fs.writeFileSync(
    path.join(stub, "quickshell"),
    '#!/bin/bash\nprintf "THEME=%s\\n" "${QT_QPA_PLATFORMTHEME-unset}"\n',
    { mode: 0o755 },
  );
  const result = spawnSync("bash", [launcher], {
    encoding: "utf8",
    env: Object.assign({}, process.env, env, {
      PATH: stub + ":" + process.env.PATH,
    }),
  });
  fs.rmSync(stub, { recursive: true, force: true });
  return result;
}

function themeFrom(result) {
  const match = String(result.stdout || "").match(/THEME=(.*)/);
  return match ? match[1].trim() : null;
}

// Omarchy sets QT_QPA_PLATFORMTHEME=gtk3 session-wide. Under that theme Qt
// loads libgtk-3 into a quickshell linked against jemalloc, and the first
// native picker aborts the whole process in g_malloc. Every page with a file
// dialog -- Omafile, Hooks, Background, Boot, Network, Bar, Accounts,
// Diagnostics -- dies with it, so the launcher has to steer off gtk3.
const gtk = runLauncher({ QT_QPA_PLATFORMTHEME: "gtk3" });
assertEqual(gtk.status, 0, "bin/atmos launches under a gtk3 session");
assertEqual(
  themeFrom(gtk),
  "xdgdesktopportal",
  "bin/atmos swaps the gtk3 platform theme for the out-of-process portal",
);

// A user who picked something else keeps it: the swap is aimed at the one
// theme that crashes, not at every theme.
const kde = runLauncher({ QT_QPA_PLATFORMTHEME: "kde" });
assertEqual(themeFrom(kde), "kde", "bin/atmos leaves a non-gtk3 platform theme alone");

const unset = runLauncher({ QT_QPA_PLATFORMTHEME: "" });
assert(
  themeFrom(unset) === "" || themeFrom(unset) === "unset",
  "bin/atmos does not invent a platform theme when none is set",
);

const src = fs.readFileSync(launcher, "utf8");
assert(
  src.indexOf("quickshell-mirror/quickshell#1010") !== -1,
  "bin/atmos cites the upstream crash the swap works around",
);
