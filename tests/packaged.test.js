const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const { assert, assertEqual, load } = require("./harness");

const root = path.join(__dirname, "..");
const atmosUpdate = load("services/AtmosUpdate.js");

const packaged = atmosUpdate.parseCheckOutput(
  "status packaged\nsummary Atmos is managed by pacman. Update it with omarchy update.\n",
);
assertEqual(packaged.status, "packaged", "parseCheckOutput accepts the packaged status");
assertEqual(
  atmosUpdate.parseCheckOutput("status packaged\n").summary,
  "Atmos is managed by pacman. Update it with omarchy update.",
  "parseCheckOutput gives packaged a default summary",
);

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-packaged-"));
try {
  const hypr = path.join(fixture, ".config", "hypr");
  fs.mkdirSync(hypr, { recursive: true });
  // setup runs hyprctl reload; a stub keeps this test off the live session.
  const stubs = path.join(fixture, "stubs");
  fs.mkdirSync(stubs);
  fs.writeFileSync(path.join(stubs, "hyprctl"), "#!/bin/bash\nexit 0\n", { mode: 0o755 });
  const withStubs = stubs + ":" + process.env.PATH;
  const run = (script, args, env) =>
    spawnSync("bash", [path.join(root, "scripts", script), ...args], {
      encoding: "utf8",
      env: Object.assign(
        {},
        process.env,
        { HOME: fixture, ATMOS_HYPR_DIR: hypr, PATH: withStubs },
        env,
      ),
    });

  // A PACKAGED marker in the app root hands updates to pacman and never touches git.
  const app = path.join(fixture, "app");
  fs.mkdirSync(app);
  fs.writeFileSync(path.join(app, "PACKAGED"), "0.1.0-1\n");
  const check = run("update-atmos.sh", ["check"], {
    ATMOS_ROOT: app,
    PATH: path.join(fixture, "nogit") + ":/usr/bin:/bin",
  });
  assertEqual(check.status, 0, "update-atmos.sh check exits 0 when packaged");
  assert(/^status packaged$/m.test(check.stdout), "update-atmos.sh check reports packaged");
  const apply = run("update-atmos.sh", ["apply"], { ATMOS_ROOT: app });
  assert(/^status packaged$/m.test(apply.stdout), "update-atmos.sh apply stays out of a package");

  // atmos --setup [on|off|status] edits only the user's Hyprland files.
  fs.writeFileSync(
    path.join(hypr, "hyprland.lua"),
    'require("default.hypr.omarchy")\nrequire("default.hypr.toggles")\n',
  );
  assertEqual(run("setup-atmos.sh", ["status"]).stdout.trim(), "off", "setup status starts off");
  const on = run("setup-atmos.sh", ["on"]);
  assertEqual(on.status, 0, "setup on exits 0");
  const lua = fs.readFileSync(path.join(hypr, "hyprland.lua"), "utf8");
  assert(lua.includes('require("hypr.atmos")'), "setup on requires hypr.atmos");
  assert(lua.includes('require("hypr.atmos_layout")'), "setup on requires hypr.atmos_layout");
  assert(fs.existsSync(path.join(hypr, "atmos.lua")), "setup on writes atmos.lua");
  assert(fs.existsSync(path.join(hypr, "atmos_layout.lua")), "setup on writes atmos_layout.lua");
  assertEqual(run("setup-atmos.sh", ["status"]).stdout.trim(), "on", "setup status reads on");
  const again = run("setup-atmos.sh", ["on"]);
  assertEqual(again.status, 0, "setup on is idempotent");
  assertEqual(
    fs.readFileSync(path.join(hypr, "hyprland.lua"), "utf8"),
    lua,
    "a second setup on leaves hyprland.lua alone",
  );

  const off = run("setup-atmos.sh", ["off"]);
  assertEqual(off.status, 0, "setup off exits 0");
  assertEqual(
    fs.readFileSync(path.join(hypr, "hyprland.lua"), "utf8"),
    'require("default.hypr.omarchy")\nrequire("default.hypr.toggles")\n',
    "setup off removes only Atmos's requires",
  );
  assert(!fs.existsSync(path.join(hypr, "atmos_layout.lua")), "setup off removes atmos_layout.lua");
  assert(!fs.existsSync(path.join(hypr, "atmos.lua")), "setup off removes an untouched atmos.lua");

  run("setup-atmos.sh", ["on"]);
  fs.appendFileSync(path.join(hypr, "atmos.lua"), '\no.window("mine", { float = true })\n');
  run("setup-atmos.sh", ["off"]);
  assert(
    fs.existsSync(path.join(hypr, "atmos.lua")),
    "setup off keeps an atmos.lua the user edited",
  );

  assertEqual(run("setup-atmos.sh", ["bogus"]).status, 2, "setup rejects an unknown action");

  // In a package install only --setup switches the integration on. Reset and
  // every look/windows/workspaces write call `require apply`, which must leave
  // a bare hyprland.lua bare, and must keep healing one the user opted into.
  const pkgApp = path.join(fixture, "pkg");
  fs.mkdirSync(pkgApp);
  for (const item of ["scripts", "packaging"])
    fs.cpSync(path.join(root, item), path.join(pkgApp, item), { recursive: true });
  fs.writeFileSync(path.join(pkgApp, "PACKAGED"), "0.1.0-1\n");
  const bare = 'require("default.hypr.omarchy")\nrequire("default.hypr.toggles")\n';
  const hyprland = path.join(hypr, "hyprland.lua");
  fs.rmSync(path.join(hypr, "atmos.lua"), { force: true });
  fs.writeFileSync(hyprland, bare);
  const pkgRun = (script, args) =>
    spawnSync("bash", [path.join(pkgApp, "scripts", script), ...args], {
      encoding: "utf8",
      env: Object.assign({}, process.env, {
        HOME: fixture,
        ATMOS_ROOT: pkgApp,
        ATMOS_SKIP_HYPR: "1",
        PATH: withStubs,
      }),
    });
  const sentinel = path.join(pkgApp, "scripts", "hypr-sentinel.py");
  spawnSync("python3", [sentinel, "require", "apply", hyprland]);
  assertEqual(fs.readFileSync(hyprland, "utf8"), bare, "packaged require apply does not opt in");
  const lookWrite = pkgRun("set-hypr-look.sh", ["--reset"]);
  assertEqual(lookWrite.status, 0, "packaged set-hypr-look.sh runs");
  assertEqual(fs.readFileSync(hyprland, "utf8"), bare, "a look write does not opt in");
  const reset = pkgRun("reset-atmos.sh", []);
  assertEqual(reset.status, 0, "packaged reset-atmos.sh runs");
  assertEqual(fs.readFileSync(hyprland, "utf8"), bare, "Reset does not opt in");
  assert(
    !fs.existsSync(path.join(hypr, "atmos_layout.lua")),
    "Reset does not write atmos_layout.lua",
  );

  assertEqual(pkgRun("setup-atmos.sh", ["on"]).status, 0, "packaged setup on runs");
  assert(
    fs.readFileSync(hyprland, "utf8").includes('require("hypr.atmos")'),
    "packaged setup on opts in",
  );
  pkgRun("reset-atmos.sh", []);
  assert(
    fs.readFileSync(hyprland, "utf8").includes('require("hypr.atmos")'),
    "Reset keeps an integration the user opted into",
  );
  pkgRun("setup-atmos.sh", ["off"]);
  assert(!fs.readFileSync(hyprland, "utf8").includes("hypr.atmos"), "packaged setup off opts out");

  // The launcher routes --setup without quickshell on PATH.
  const launched = spawnSync("bash", [path.join(root, "bin", "atmos"), "--setup", "status"], {
    encoding: "utf8",
    env: Object.assign({}, process.env, {
      HOME: fixture,
      ATMOS_HYPR_DIR: hypr,
      PATH: withStubs,
    }),
  });
  assertEqual(launched.status, 0, "bin/atmos --setup runs without quickshell");
  assertEqual(launched.stdout.trim(), "off", "bin/atmos --setup status reports the state");
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
