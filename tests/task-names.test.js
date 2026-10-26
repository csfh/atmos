const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");
const os = require("os");
const { load, assert } = require("./harness");
const { atmosdBinary } = require("./sandbox");

// Every `atmosd task <name>` the window can run has to be a task atmosd knows.
const root = path.join(__dirname, "..");
const mod = fs.readFileSync(path.join(root, "backend", "src", "tasks", "mod.rs"), "utf8");
const block = mod.slice(
  mod.indexOf("pub const NAMES"),
  mod.indexOf("];", mod.indexOf("pub const NAMES")),
);
const names = new Set(block.match(/"[a-z][a-z0-9-]*"/g).map((n) => n.slice(1, -1)));
assert(names.size > 30, "read the task names from tasks/mod.rs");

const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "atmos-task-names-"));
const listed = spawnSync(atmosdBinary(), ["--backend", "omarchy", "--root", fixture, "request"], {
  encoding: "utf8",
  input: JSON.stringify({ op: "settings.list" }),
});
const domains = new Set(JSON.parse(listed.stdout).result.map((row) => row.domain));
assert(domains.size > 100, "atmosd lists its settings domains");

const settings = load("services/Settings.js");
const samples = [true, false, "x", 1, 50, [], {}, ""];
const unknown = new Set();
let seen = 0;
Object.keys(settings.catalogByKey()).forEach((key) => {
  samples.forEach((value) => {
    let cmd;
    try {
      cmd = settings.commandFor(key, value, {}, { atmosd: "atmosd" });
    } catch {
      return;
    }
    if (!cmd || !Array.isArray(cmd.argv) || cmd.argv[1] !== "task") return;
    seen++;
    if (!names.has(cmd.argv[2])) unknown.add(key + " -> " + cmd.argv[2]);
    if (cmd.argv[2] === "settings-set" && !domains.has(cmd.argv[3])) {
      unknown.add(key + " -> settings-set " + cmd.argv[3]);
    }
  });
});
assert(seen > 100, "commandFor produced task argv for the catalog");
assert(unknown.size === 0, "every catalog writer names a atmosd task: " + [...unknown].join(", "));

const files = ["services", "pages", "components"].flatMap((dir) => {
  const full = path.join(root, dir);
  return fs.existsSync(full)
    ? fs
        .readdirSync(full)
        .filter((f) => /\.(qml|js)$/.test(f))
        .map((f) => path.join(full, f))
    : [];
});
const missing = [];
files.forEach((file) => {
  const text = fs.readFileSync(file, "utf8");
  for (const m of text.matchAll(/(?:Backend\.)?task\(\s*"([a-z][a-z0-9-]*)"/g)) {
    if (!names.has(m[1])) missing.push(path.basename(file) + " -> " + m[1]);
  }
});
assert(missing.length === 0, "every QML task call names a atmosd task: " + missing.join(", "));
console.log("ok - " + seen + " catalog commands and the QML task calls name known tasks");
