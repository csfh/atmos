const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { assert, assertEqual } = require("./harness");
const { ratmosBinary } = require("./sandbox");

// The named workspace bar goes through the same write path as the window:
// `settings.set` on a fixture root installs the clone, points the bar at it,
// and keeps the other field and any extra keys on the entry.
const root = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-wsbar-"));
const omarchy = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-wsbar-omarchy-"));
const widgets = path.join(omarchy, "shell", "plugins", "bar", "widgets");
fs.mkdirSync(widgets, { recursive: true });
fs.writeFileSync(
  path.join(widgets, "Workspaces.manifest.json"),
  JSON.stringify({
    id: "omarchy.workspaces",
    name: "Workspaces",
    barWidget: { displayName: "Workspaces" },
    omarchy: { builtin: true },
  }),
);
const shellFile = path.join(root, ".config", "omarchy", "shell.json");
fs.mkdirSync(path.dirname(shellFile), { recursive: true });
fs.writeFileSync(
  shellFile,
  JSON.stringify({
    version: 1,
    bar: {
      layout: {
        left: [{ id: "omarchy.menu" }, { id: "omarchy.workspaces" }],
        center: [],
        right: [],
      },
    },
    plugins: [],
  }),
);

function set(domain, value) {
  const result = spawnSync(ratmosBinary(), ["--backend", "omarchy", "--root", root, "request"], {
    encoding: "utf8",
    input: JSON.stringify({ op: "settings.set", domain, value }),
    env: Object.assign({}, process.env, { USER: "tester", OMARCHY_PATH: omarchy }),
  });
  const reply = JSON.parse(result.stdout);
  assertEqual(reply.ok, true, "settings.set " + domain + " replies ok");
}
function entry() {
  return JSON.parse(fs.readFileSync(shellFile, "utf8")).bar.layout.left[1];
}
const plugin = path.join(root, ".config", "omarchy", "plugins", "tester.workspaces");

set("workspaceBarNames", true);
assertEqual(entry().id, "tester.workspaces", "turning names on points the bar at the clone");
assert(fs.existsSync(path.join(plugin, "Workspaces.qml")), "the clone widget is installed");
assert(
  fs.readFileSync(path.join(plugin, "Workspaces.qml"), "utf8").indexOf("atmos:workspace-labels") !==
    -1,
  "the clone paints workspace names",
);
const manifest = JSON.parse(fs.readFileSync(path.join(plugin, "manifest.json"), "utf8"));
assertEqual(manifest.id, "tester.workspaces", "the clone manifest takes the clone id");
assertEqual(manifest.omarchy.clonedFrom, "omarchy.workspaces", "the manifest marks its origin");
assertEqual(manifest.barWidget.displayName, "My Workspaces", "the manifest names the widget");

set("workspaceBarNames", false);
assertEqual(entry().id, "tester.workspaces", "names off keeps the clone in the bar");
assertEqual(entry().showNames, false, "names off writes showNames false");

fs.writeFileSync(path.join(plugin, "Workspaces.qml"), "// stale clone\n");
const doc = JSON.parse(fs.readFileSync(shellFile, "utf8"));
doc.bar.layout.left[1].note = "keep";
fs.writeFileSync(shellFile, JSON.stringify(doc));
set("workspaceBarCount", 7);
assertEqual(entry().count, 7, "the count write sets the count");
assertEqual(entry().showNames, false, "the count write keeps names off");
assertEqual(entry().note, "keep", "the count write keeps other keys on the entry");
assert(
  fs.readFileSync(path.join(plugin, "Workspaces.qml"), "utf8").indexOf("extras-rule:") !== -1,
  "a count write refreshes a stale clone",
);

set("workspaceBarNames", true);
assertEqual(entry().count, 7, "names on keeps the count");
assertEqual(entry().showNames, true, "names on writes showNames true");
console.log("ok - the named workspace bar installs the clone and keeps the entry");
