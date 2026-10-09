// Cross-checks between the layers: names that one layer uses and another has
// to provide. Nothing here runs the app; it reads files and compares lists.
const fs = require("fs");
const path = require("path");
const { assert, assertEqual } = require("./harness");

const root = path.join(__dirname, "..");
const read = (rel) => fs.readFileSync(path.join(root, rel), "utf8");

function walk(rel, exts) {
  const out = [];
  for (const entry of fs.readdirSync(path.join(root, rel), { withFileTypes: true })) {
    const child = path.join(rel, entry.name);
    if (entry.isDirectory()) out.push(...walk(child, exts));
    else if (exts.some((ext) => entry.name.endsWith(ext))) out.push(child);
  }
  return out;
}

function matches(files, pattern) {
  const found = new Map();
  for (const file of files) {
    for (const m of read(file).matchAll(pattern)) {
      if (!found.has(m[1])) found.set(m[1], file);
    }
  }
  return found;
}

const frontend = [
  ...walk("pages", [".qml"]),
  ...walk("components", [".qml"]),
  ...walk("services", [".qml", ".js"]),
  "shell.qml",
];
const rustSources = walk(path.join("backend", "src"), [".rs"]);

// A literal that only reads like a script name in the UI (a placeholder).
const notScripts = new Set(["notify.sh"]);

// --- scripts -----------------------------------------------------------------

const scriptsDir = path.join(root, "scripts");
const scriptFiles = fs.readdirSync(scriptsDir).filter((name) => /\.(sh|py)$/.test(name));

const fromFrontend = matches(frontend, /"(?:[^"\s]*\/)?([A-Za-z0-9_-]+\.(?:sh|py))"/g);
const fromRust = matches(rustSources, /"([A-Za-z0-9_-]+\.(?:sh|py))"/g);

assert(fromFrontend.size > 20, "the front end names scripts to look for");
assert(fromRust.size > 10, "the backend names scripts to look for");

for (const [source, found] of [
  ["QML and JS", fromFrontend],
  ["Rust", fromRust],
]) {
  for (const [name, file] of found) {
    if (notScripts.has(name)) continue;
    assert(scriptFiles.includes(name), `${source} script ${name} exists in scripts/ (${file})`);
  }
}

// A script with a shebang is meant to be run, so it has to be executable.
// Files meant to be sourced have no shebang and stay plain.
for (const name of scriptFiles) {
  const full = path.join(scriptsDir, name);
  const first = fs.readFileSync(full, "utf8").split("\n", 1)[0];
  if (!first.startsWith("#!")) continue;
  assert((fs.statSync(full).mode & 0o111) !== 0, `scripts/${name} has a shebang and is executable`);
}

// --- icons -------------------------------------------------------------------

const iconNames = new Set(
  fs
    .readdirSync(path.join(root, "icons"))
    .filter((name) => name.endsWith(".svg"))
    .map((name) => name.replace(/\.svg$/, "")),
);
const usedIcons = matches(frontend, /"([a-z0-9]+(?:-[a-z0-9]+)*-(?:line|fill))"/g);
assert(usedIcons.size > 20, "the front end names icons");
for (const [name, file] of usedIcons) {
  assert(iconNames.has(name), `icon ${name} exists in icons/ (${file})`);
}

// --- Theme tokens ------------------------------------------------------------

const themeSource = read("services/Theme.qml");
const themeDefined = new Set(
  [
    ...themeSource.matchAll(/(?:\bproperty\s+(?:alias\s+)?\S+\s+|\bfunction\s+|\bsignal\s+)(\w+)/g),
  ].map((m) => m[1]),
);
const themeUsed = matches(
  frontend.filter((file) => file !== path.join("services", "Theme.qml")),
  /(?<![\w./])Theme\.(?!(?:qml|js)\b)([A-Za-z_]\w*)/g,
);
assert(themeUsed.size > 40, "the front end reads Theme tokens");

for (const [token, file] of themeUsed) {
  assert(themeDefined.has(token), `Theme.${token} is defined in Theme.qml (${file})`);
}

// --- qmldir ------------------------------------------------------------------

const registered = new Map(
  [...read("services/qmldir").matchAll(/^singleton\s+(\w+)\s+\S+\s+(\S+\.qml)\s*$/gm)].map((m) => [
    m[1],
    m[2],
  ]),
);
const qmlServices = fs
  .readdirSync(path.join(root, "services"))
  .filter((name) => name.endsWith(".qml"));
for (const [type, file] of registered) {
  assertEqual(file, `${type}.qml`, `qmldir maps ${type} to ${type}.qml`);
  assert(qmlServices.includes(file), `qmldir entry ${type} has its file`);
}
for (const file of qmlServices) {
  assert(registered.has(file.replace(/\.qml$/, "")), `services/${file} is registered in qmldir`);
}
