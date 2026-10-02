const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const table = load("services/SettingTable.js");

assertEqual(table.plan("browser", "firefox", "chromium").dispatch, true, "a new string is sent");
assertEqual(
  table.plan("browser", "firefox", "firefox").dispatch,
  false,
  "an unchanged string is skipped",
);
assertEqual(table.plan("browser", "", "firefox").dispatch, false, "an empty string is skipped");
assertEqual(table.plan("browser", undefined, "firefox").dispatch, false, "no value is skipped");
assertEqual(
  table.plan("barVisible", false, true).dispatch,
  true,
  "a changed bool is sent, false included",
);
assertEqual(table.plan("barVisible", false, true).value, false, "the bool is carried");
assertEqual(table.plan("barVisible", true, true).dispatch, false, "an unchanged bool is skipped");
assertEqual(table.plan("nope", 1, 0).known, false, "a key outside the table is unknown");
assertEqual(table.plan("nope", 1, 0).dispatch, false, "an unknown key sends nothing");
assertEqual(table.plan("toString", "x", "").known, false, "inherited names are not settings");

// Every key in the table is a property of Omarchy.qml and a domain in the
// backend, so Omarchy.set can read the current value and the write lands.
const omarchy = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
const domains = fs.readFileSync(path.join(__dirname, "..", "backend", "src", "domain.rs"), "utf8");
for (const key of table.keys()) {
  assert(
    omarchy.includes("property string " + key + ":") ||
      omarchy.includes("property bool " + key + ":") ||
      omarchy.includes("property var " + key + ":"),
    key + " is an Omarchy property",
  );
  assert(new RegExp('\\(\\s*"' + key + '",').test(domains), key + " is a backend domain");
}
