const fs = require("fs");
const path = require("path");
const { assert } = require("./harness");

const compilePython = fs.readFileSync(path.join(__dirname, "compile-python"), "utf8");
assert(compilePython.indexOf("ast.parse") !== -1, "compile-python parses scripts/*.py");
assert(compilePython.indexOf("tabnanny.check") !== -1, "compile-python runs tabnanny");
const preCommit = fs.readFileSync(path.join(__dirname, "..", ".githooks", "pre-commit"), "utf8");
assert(preCommit.indexOf("tests/compile-python") !== -1, "pre-commit runs compile-python");
const testsRun = fs.readFileSync(path.join(__dirname, "run"), "utf8");
assert(testsRun.indexOf("tests/compile-python") !== -1, "tests/run runs compile-python");
assert(testsRun.indexOf("tests/*.test.js") !== -1, "tests/run runs each tests/*.test.js file");
assert(testsRun.indexOf("parse.test.js") === -1, "tests/run no longer calls parse.test.js");
const searchPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "SearchPage.qml"),
  "utf8",
);
assert(searchPageSrc.indexOf('"serve"') !== -1, "SearchPage keeps a SearchIndex serve process");
assert(
  searchPageSrc.indexOf('SearchIndex.js", "query"') === -1,
  "SearchPage does not spawn a query process per keystroke",
);
const omarchyQml = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
assert(
  omarchyQml.indexOf('enqueueRead(ioQueue, "rest")') !== -1,
  "startSession follows look with rest",
);
const snapshotSh = fs.readFileSync(path.join(__dirname, "..", "scripts", "snapshot.sh"), "utf8");
assert(snapshotSh.indexOf("GROUP == rest") !== -1, "snapshot.sh strips look keys from rest");
const workflow = fs.readFileSync(
  path.join(__dirname, "..", ".github", "workflows", "tests.yml"),
  "utf8",
);
assert(workflow.indexOf("./tests/run") !== -1, "GitHub Actions runs ./tests/run");
assert(workflow.indexOf("npm ci") !== -1, "GitHub Actions installs with npm ci");
assert(workflow.indexOf("ubuntu-latest") !== -1, "GitHub Actions uses ubuntu-latest");

const shellSrc = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");
assert(shellSrc.indexOf("id: errorDialog") !== -1, "error dialog is a PrefsDialog");
assert(shellSrc.indexOf("Omarchy.copyLastError()") !== -1, "error dialog copies lastError");
assert(shellSrc.indexOf("Omarchy.clearLastError()") !== -1, "error dialog dismisses lastError");
assert(
  shellSrc.indexOf("Omarchy.askAgentAboutError()") !== -1,
  "error dialog asks the default agent",
);
const omarchySrc = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
assert(omarchySrc.indexOf("function copyLastError()") !== -1, "Omarchy.copyLastError is defined");
assert(omarchySrc.indexOf("function clearLastError()") !== -1, "Omarchy.clearLastError is defined");
assert(
  omarchySrc.indexOf("function askAgentAboutError()") !== -1,
  "Omarchy.askAgentAboutError is defined",
);
assert(
  omarchySrc.indexOf("RichUi.clipboardPayload(lastError") !== -1,
  "copyLastError uses clipboardPayload",
);
assert(
  omarchySrc.indexOf("omarchy agent prompt") !== -1,
  "askAgentAboutError launches omarchy agent prompt",
);
assert(
  omarchySrc.indexOf("function commandFailureText(") !== -1,
  "failed jobs combine stdout and stderr",
);
assert(omarchySrc.indexOf("id: mutOut") !== -1, "mutProc keeps stdout for failure text");

const idleSh = fs.readFileSync(path.join(__dirname, "..", "scripts", "set-idle.sh"), "utf8");
assert(
  idleSh.indexOf("OMARCHY_PATH:=/usr/share/omarchy") !== -1,
  "set-idle.sh defaults OMARCHY_PATH before sourcing omarchy-shell-config",
);
const barSh = fs.readFileSync(path.join(__dirname, "..", "scripts", "set-bar-widget.sh"), "utf8");
assert(
  barSh.indexOf("OMARCHY_PATH:=/usr/share/omarchy") !== -1,
  "set-bar-widget.sh defaults OMARCHY_PATH before sourcing omarchy-shell-config",
);
const envSh = fs.readFileSync(path.join(__dirname, "..", "scripts", "atmos-env.sh"), "utf8");
assert(
  envSh.indexOf("hyprctl reload >/dev/null || true") !== -1,
  "atmos_hypr_reload does not fail the write when reload fails",
);
