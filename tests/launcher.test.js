const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { assert, assertEqual } = require("./harness");
const { atmosdBinary } = require("./sandbox");

const root = path.join(__dirname, "..");
const launcher = path.join(root, "bin", "atmos");

// `atmos` ends in `exec quickshell`, so the only way to see what it exported
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
      ATMOS_BACKEND: atmosdBinary(),
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
assertEqual(gtk.status, 0, "atmos launches under a gtk3 session");
assertEqual(
  themeFrom(gtk),
  "xdgdesktopportal",
  "atmos swaps the gtk3 platform theme for the out-of-process portal",
);

// A user who picked something else keeps it: the swap is aimed at the one
// theme that crashes, not at every theme.
const kde = runLauncher({ QT_QPA_PLATFORMTHEME: "kde" });
assertEqual(themeFrom(kde), "kde", "atmos leaves a non-gtk3 platform theme alone");

const unset = runLauncher({ QT_QPA_PLATFORMTHEME: "" });
assert(
  themeFrom(unset) === "" || themeFrom(unset) === "unset",
  "atmos does not invent a platform theme when none is set",
);

const src = fs.readFileSync(path.join(root, "backend", "src", "launch.rs"), "utf8");
assert(
  src.indexOf("quickshell-mirror/quickshell#1010") !== -1,
  "the launcher cites the upstream crash the swap works around",
);

// `atmos mcp` speaks JSON-RPC on stdio and never opens a window.
function runMcp() {
  const stub = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-mcp-"));
  fs.writeFileSync(path.join(stub, "quickshell"), "#!/bin/bash\nprintf 'QUICKSHELL\\n'\n", {
    mode: 0o755,
  });
  const result = spawnSync("bash", [launcher, "mcp", "--backend", "plain"], {
    encoding: "utf8",
    input:
      '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}\n',
    env: Object.assign({}, process.env, {
      PATH: stub + ":" + process.env.PATH,
      ATMOS_BACKEND: atmosdBinary(),
    }),
  });
  fs.rmSync(stub, { recursive: true, force: true });
  return result;
}

const mcp = runMcp();
assertEqual(mcp.status, 0, "atmos mcp exits when its input closes");
const reply = JSON.parse(mcp.stdout.split("\n")[0]);
assertEqual(reply.result.protocolVersion, "2025-03-26", "atmos mcp answers initialize");
assertEqual(reply.result.serverInfo.name, "atmos", "atmos mcp names itself");
assert(mcp.stdout.indexOf("QUICKSHELL") === -1, "atmos mcp does not open a window");
