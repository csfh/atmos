const fs = require("fs");
const path = require("path");
const { assert } = require("./harness");

function walk(dir, acc) {
  acc = acc || [];
  fs.readdirSync(dir, { withFileTypes: true }).forEach(function (ent) {
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(full, acc);
    else if (ent.name.endsWith(".qml")) acc.push(full);
  });
  return acc;
}

function namedBlocks(src, name) {
  const needle = name + " {";
  const out = [];
  let from = 0;
  while (true) {
    const start = src.indexOf(needle, from);
    if (start < 0) break;
    const brace = src.indexOf("{", start);
    if (brace < 0) break;
    let depth = 0;
    let end = brace;
    for (; end < src.length; end++) {
      const ch = src[end];
      if (ch === "{") depth++;
      else if (ch === "}") {
        depth--;
        if (depth === 0) {
          end++;
          break;
        }
      }
    }
    out.push(src.slice(brace, end));
    from = start + needle.length;
  }
  return out;
}

function stringProp(block, name) {
  const re = new RegExp(name + ':\\s*"((?:\\\\.|[^"\\\\])*)"');
  const m = String(block || "").match(re);
  return m ? m[1] : "";
}

const pages = walk(path.join(__dirname, "..", "pages"));
const resetGated = [];

pages.forEach(function (file) {
  const src = fs.readFileSync(file, "utf8");
  const rel = path.relative(path.join(__dirname, ".."), file);
  namedBlocks(src, "SettingRow").forEach(function (block) {
    const label = stringProp(block, "label");
    if (/^Reset\b/i.test(label) && /PrefsButton\b/.test(block) && /available:/.test(block)) {
      resetGated.push(rel + " " + label);
    }
  });
});

assert(
  resetGated.length === 0,
  "Reset rows keep their button in the layout",
  resetGated.join("\n"),
);

const appearanceSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AppearancePage.qml"),
  "utf8",
);
assert(
  /label: "Night light schedule"[\s\S]*PrefsField[\s\S]*label: "Use schedule"[\s\S]*PrefsToggle/.test(
    appearanceSrc,
  ),
  "Night light schedule is the times; Use schedule is the on switch",
);

const networkSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "NetworkPage.qml"), "utf8");
assert(
  /label: "Gateway"[\s\S]*text: "Copy"/.test(networkSrc) &&
    /label: "DNS"[\s\S]*text: "Copy"/.test(networkSrc),
  "Gateway and DNS copy like Address",
);

const powerSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "PowerPage.qml"), "utf8");
assert(
  /label: "CPU governor"[\s\S]*valueText:/.test(powerSrc) &&
    /label: "Energy preference"[\s\S]*valueText:/.test(powerSrc),
  "CPU governor and energy preference show the value on the right",
);

const envSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "system", "EnvironmentPage.qml"),
  "utf8",
);
assert(/label: "PATH prepend"[\s\S]*text: "Set"/.test(envSrc), "PATH prepend has a Set button");

const windowsSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "WindowsPage.qml"), "utf8");
assert(
  /label: "Swallow regex"[\s\S]*text: "Set"/.test(windowsSrc),
  "Swallow regex has a Set button",
);

const workspacesSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "WorkspacesPage.qml"),
  "utf8",
);
assert(
  /placeholder: "Bar name"[\s\S]*text: "Set"/.test(workspacesSrc),
  "Workspace bar name has a Set button",
);

const inputSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "InputPage.qml"), "utf8");
assert(
  inputSrc.indexOf("available: Omarchy.hyprInputManaged") === -1,
  "Reset input is not gated off the layout",
);

const bootSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "appearance", "BootPage.qml"),
  "utf8",
);
assert(
  bootSrc.indexOf('available: Omarchy.plymouth !== "default"') === -1,
  "Reset boot screen is not gated off the layout",
);

const lookSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "WindowsPage.qml"), "utf8");
assert(
  lookSrc.indexOf("available: Omarchy.hyprLookManaged") === -1,
  "Reset look is not gated off the layout",
);
