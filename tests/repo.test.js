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
assert(
  omarchyQml.indexOf("hyprsunsetConfFile") !== -1 && omarchyQml.indexOf("hyprsunset.conf") !== -1,
  "Omarchy watches hyprsunset.conf for the night-light schedule",
);
assert(
  omarchyQml.indexOf("HyprSunset.js") !== -1 && omarchyQml.indexOf("HyprSunset.parseTime") !== -1,
  "Omarchy validates night-light times with HyprSunset.parseTime",
);
assert(
  omarchyQml.indexOf("/^[0-2]?\\d:[0-5]\\d$/") === -1,
  "Omarchy does not keep 24:00 as a night-light time",
);
assert(
  omarchyQml.indexOf("autostartLuaFile") !== -1 && omarchyQml.indexOf("bindingsLuaFile") !== -1,
  "Omarchy watches autostart.lua and bindings.lua",
);
assert(
  omarchyQml.indexOf("windowsLuaFile") !== -1 && omarchyQml.indexOf("atmos.lua") !== -1,
  "Omarchy watches atmos.lua for window rules",
);
const spinSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSpinBox.qml"),
  "utf8",
);
assert(
  spinSrc.indexOf("if (root.value === value) root._holding = false") === -1,
  "PrefsSpinBox does not keep holding a value the model rejected",
);
assert(
  /changed\(value\)[\s\S]*root\._holding = false/.test(spinSrc),
  "PrefsSpinBox releases its hold after every write so a rejected number can snap back",
);
const flickSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsFlickable.qml"),
  "utf8",
);
assert(
  !/\n\s+anchors\.fill:\s*root\b/.test(flickSrc),
  "PrefsFlickable does not anchors.fill the viewport",
);
assert(
  flickSrc.indexOf("width: root.width") !== -1,
  "PrefsFlickable sizes the wheel area from the viewport",
);
const selectSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSelect.qml"),
  "utf8",
);
assert(
  selectSrc.indexOf("function placePopup()") !== -1,
  "PrefsSelect places the popup on Overlay",
);
assert(
  selectSrc.indexOf("popup.parent = overlay") !== -1,
  "PrefsSelect reparents the list onto Overlay so the pane clip cannot crop it",
);
assert(
  selectSrc.indexOf("parent: list") !== -1 &&
    selectSrc.indexOf("acceptedButtons: Qt.NoButton") !== -1,
  "PrefsSelect sizes a viewport wheel area on the options ListView",
);
assert(
  selectSrc.indexOf("wheel.accepted = true") !== -1,
  "PrefsSelect consumes wheel events so the page behind an open popup does not move",
);
const confirmSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsConfirm.qml"),
  "utf8",
);
assert(
  confirmSrc.indexOf("property bool destructive: true") === -1,
  "PrefsConfirm does not mark every confirm as destructive",
);
assert(
  confirmSrc.indexOf("RichUi.confirmIsDestructive(confirmText)") !== -1,
  "PrefsConfirm styles Remove as danger and Install as primary",
);
assert(
  confirmSrc.indexOf("function attachOverlay()") !== -1 &&
    confirmSrc.indexOf("parent = overlay") !== -1,
  "PrefsConfirm reparents onto Overlay so the pane clip cannot crop it",
);
const dialogSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsDialog.qml"),
  "utf8",
);
assert(
  dialogSrc.indexOf("function attachOverlay()") !== -1 &&
    dialogSrc.indexOf("parent = overlay") !== -1,
  "PrefsDialog reparents onto Overlay so the pane clip cannot crop it",
);
const helpSrc = fs.readFileSync(path.join(__dirname, "..", "components", "PrefsHelp.qml"), "utf8");
assert(
  helpSrc.indexOf("onAboutToShow:") !== -1 && helpSrc.indexOf("parent = overlay") !== -1,
  "PrefsHelp reparents the modal onto Overlay so the pane clip cannot crop it",
);
assert(
  helpSrc.indexOf("Accessible.name: root.accessibleName") !== -1 &&
    helpSrc.indexOf("helpAccessibleName") !== -1,
  "PrefsHelp names the section, such as About DNS settings",
);
assert(
  helpSrc.indexOf("Keys.onReturnPressed: popup.open()") !== -1 &&
    helpSrc.indexOf("Keys.onSpacePressed: popup.open()") !== -1,
  "PrefsHelp opens from Enter and Space",
);
assert(
  helpSrc.indexOf("ToolTip") !== -1 && helpSrc.indexOf("helpMouse.containsMouse") !== -1,
  "PrefsHelp exposes the extra copy on hover",
);
const prefsGroupSrcEarly = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsGroup.qml"),
  "utf8",
);
assert(
  prefsGroupSrcEarly.indexOf("sectionHelpOpen") !== -1 &&
    prefsGroupSrcEarly.indexOf("root.detail.length > 0 || root.hint.length > 0") === -1,
  "section info icon shows only when help adds context",
);
const snapshotSh = fs.readFileSync(path.join(__dirname, "..", "scripts", "snapshot.sh"), "utf8");
assert(snapshotSh.indexOf("GROUP == rest") !== -1, "snapshot.sh strips look keys from rest");
assert(
  snapshotSh.indexOf('line.split("#", 1)[0]') !== -1,
  "snapshot.sh hyprsunset parser ignores commented profiles",
);
assert(
  snapshotSh.indexOf('hypr-sentinel.py" input list') !== -1,
  "snapshot.sh reads workspaceGesture from the input sentinel",
);
assert(
  snapshotSh.indexOf("grep -q 'action = \"workspace\"'") === -1,
  "snapshot.sh does not grep the whole input.lua for a workspace gesture",
);
const workflow = fs.readFileSync(
  path.join(__dirname, "..", ".github", "workflows", "tests.yml"),
  "utf8",
);
assert(workflow.indexOf("./tests/run") !== -1, "GitHub Actions runs ./tests/run");
assert(workflow.indexOf("npm ci") !== -1, "GitHub Actions installs with npm ci");
assert(workflow.indexOf("ubuntu-latest") !== -1, "GitHub Actions uses ubuntu-latest");

const themeQml = fs.readFileSync(path.join(__dirname, "..", "services", "Theme.qml"), "utf8");
assert(
  themeQml.indexOf("readonly property int pageTitleSize:") !== -1,
  "Theme names page title size",
);
assert(
  themeQml.indexOf("readonly property int sectionSize:") !== -1,
  "Theme names section heading size",
);
assert(
  themeQml.indexOf("readonly property int labelSize:") !== -1 &&
    themeQml.indexOf("readonly property int descriptionSize:") !== -1 &&
    themeQml.indexOf("readonly property int metaSize:") !== -1,
  "Theme names setting label, description, and meta sizes",
);
assert(
  themeQml.indexOf("readonly property int pageMargin:") !== -1 &&
    themeQml.indexOf("readonly property int sectionSpacing:") !== -1 &&
    themeQml.indexOf("readonly property int headingGap:") !== -1 &&
    themeQml.indexOf("readonly property int rowPad:") !== -1 &&
    themeQml.indexOf("readonly property int copyInset:") !== -1,
  "Theme names page, section, heading, row, and copy inset spacing",
);
assert(
  themeQml.indexOf("readonly property int controlHeight:") !== -1 &&
    themeQml.indexOf("readonly property int toggleWidth:") !== -1 &&
    themeQml.indexOf("readonly property int checkSize:") !== -1 &&
    themeQml.indexOf("readonly property int sliderTrack:") !== -1 &&
    themeQml.indexOf("readonly property int sliderBar:") !== -1 &&
    themeQml.indexOf("readonly property int sliderHandle:") !== -1 &&
    themeQml.indexOf("readonly property int sliderTickGap:") !== -1 &&
    themeQml.indexOf("readonly property real disabledOpacity:") !== -1,
  "Theme names control sizes and disabled opacity",
);
assert(
  themeQml.indexOf("readonly property int radius: 0") !== -1 &&
    themeQml.indexOf("readonly property int contentMaxWidth: 1000") !== -1 &&
    themeQml.indexOf("readonly property int railWidth:") !== -1,
  "Theme keeps square chrome, a capped content column, and a sidebar rail",
);
assert(
  themeQml.indexOf("function controlOpacity(on)") !== -1,
  "Theme owns enabled/disabled opacity",
);
const prefsButtonSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsButton.qml"),
  "utf8",
);
assert(
  prefsButtonSrc.indexOf("implicitHeight: Theme.controlHeight") !== -1 &&
    prefsButtonSrc.indexOf("Theme.controlOpacity(enabled)") !== -1 &&
    prefsButtonSrc.indexOf("Theme.motionFast") !== -1,
  "PrefsButton uses Theme control height, disabled opacity, and motion",
);
const prefsSliderSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSlider.qml"),
  "utf8",
);
assert(
  prefsSliderSrc.indexOf("RichUi.sliderTickValues") !== -1 &&
    prefsSliderSrc.indexOf("RichUi.sliderFitTicks") !== -1,
  "PrefsSlider uses major ticks and drops colliding labels",
);
assert(
  prefsSliderSrc.indexOf("property bool showValue: false") !== -1 &&
    prefsSliderSrc.indexOf("readonly property string displayValue:") !== -1,
  "PrefsSlider keeps the current value off the track for the setting label",
);
assert(
  prefsSliderSrc.indexOf("Theme.sliderBar") !== -1 &&
    prefsSliderSrc.indexOf("Theme.sliderHandle") !== -1 &&
    prefsSliderSrc.indexOf("Theme.sliderTickGap") !== -1 &&
    prefsSliderSrc.indexOf("Theme.radius") !== -1 &&
    prefsSliderSrc.indexOf("radius: 2") === -1,
  "PrefsSlider uses Theme track, handle, tick gap, and square chrome",
);
assert(
  prefsSliderSrc.indexOf("Accessible.role: Accessible.Slider") !== -1,
  "PrefsSlider stays keyboard-accessible",
);
const prefsStepperSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSliderStepper.qml"),
  "utf8",
);
assert(
  prefsStepperSrc.indexOf("PrefsSlider") !== -1 &&
    prefsStepperSrc.indexOf("PrefsSpinBox") !== -1 &&
    prefsStepperSrc.indexOf("showTicks: false") !== -1,
  "PrefsSliderStepper is a compact slider plus numeric stepper",
);
const textSizeRowSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "rows", "TextSizeRow.qml"),
  "utf8",
);
const cursorSizeRowSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "rows", "CursorSizeRow.qml"),
  "utf8",
);
assert(
  textSizeRowSrc.indexOf("PrefsSliderStepper") !== -1 &&
    cursorSizeRowSrc.indexOf("PrefsSliderStepper") !== -1 &&
    cursorSizeRowSrc.indexOf("stretchControl") === -1,
  "text size and cursor size use the compact slider plus stepper",
);
const windowsPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "WindowsPage.qml"),
  "utf8",
);
assert(
  windowsPageSrc.indexOf('label: "Outer gaps"') !== -1 &&
    windowsPageSrc.indexOf("stretchControl: true") !== -1 &&
    windowsPageSrc.indexOf("PrefsSlider") !== -1,
  "gaps keep a full-width slider",
);
assert(
  windowsPageSrc.indexOf('text: "Configure…"') !== -1 &&
    windowsPageSrc.indexOf('label: "Keybindings"') !== -1 &&
    windowsPageSrc.indexOf('label: "Window rules"') !== -1,
  "keybindings and window rules open with Configure…",
);
const soundSliderSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "SoundPage.qml"),
  "utf8",
);
assert(
  soundSliderSrc.indexOf('label: "Volume"') !== -1 &&
    soundSliderSrc.indexOf("stretchControl: true") !== -1,
  "volume keeps a full-width slider",
);
const prefsToggleSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsToggle.qml"),
  "utf8",
);
assert(
  prefsToggleSrc.indexOf("Theme.toggleWidth") !== -1 &&
    prefsToggleSrc.indexOf("Theme.toggleHeight") !== -1,
  "PrefsToggle uses Theme toggle dimensions",
);
const prefsPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsPage.qml"),
  "utf8",
);
assert(
  prefsPageSrc.indexOf("Theme.pageTitleSize") !== -1 &&
    prefsPageSrc.indexOf("Theme.pageDescriptionSize") !== -1 &&
    prefsPageSrc.indexOf("Theme.sectionSpacing") !== -1 &&
    prefsPageSrc.indexOf("Theme.pageMargin") !== -1 &&
    prefsPageSrc.indexOf("Theme.copyInset") !== -1,
  "PrefsPage uses Theme page title, description, margin, section spacing, and copy inset",
);

const settingRowSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "SettingRow.qml"),
  "utf8",
);
assert(settingRowSrc.indexOf("id: labelText") !== -1, "SettingRow names the label line");
assert(
  settingRowSrc.indexOf("Theme.labelSize") !== -1 &&
    settingRowSrc.indexOf("Theme.descriptionSize") !== -1 &&
    settingRowSrc.indexOf("Theme.copyInset") !== -1,
  "SettingRow uses Theme label type, description type, and copy inset",
);
assert(
  settingRowSrc.indexOf("inlineCopyHost.implicitHeight") === -1,
  "SettingRow does not stretch the control band to the wrapped description",
);
assert(
  settingRowSrc.indexOf("labelText.implicitHeight") !== -1,
  "SettingRow sizes the control band from the label line so controls stay aligned",
);
assert(
  settingRowSrc.indexOf("property string valueText") !== -1,
  "SettingRow can show trailing status text without a child control",
);
assert(
  settingRowSrc.indexOf("readonly property string shownValue:") !== -1 &&
    settingRowSrc.indexOf("id: valueHost") !== -1 &&
    settingRowSrc.indexOf("id: valueBit") !== -1 &&
    settingRowSrc.indexOf("root.stack && root.shownValue.length > 0") !== -1,
  "SettingRow shows a slider's current value on the label line when stacked",
);
assert(
  settingRowSrc.indexOf("id: valueMetrics") !== -1 &&
    settingRowSrc.indexOf("Math.min(valueMetrics.width") !== -1,
  "SettingRow value text sizes from TextMetrics, not its own implicitWidth",
);
assert(
  settingRowSrc.indexOf("informational booleans") !== -1,
  "SettingRow documents valueText for read-only booleans instead of a disabled switch",
);
const statusLabelSrc = settingRowSrc.slice(
  settingRowSrc.indexOf("id: statusLabel"),
  settingRowSrc.indexOf("id: statusLabel") + 500,
);
assert(
  statusLabelSrc.indexOf("id: statusLabel") !== -1 &&
    statusLabelSrc.indexOf("color: Theme.muted") !== -1 &&
    statusLabelSrc.indexOf("Theme.accent") === -1 &&
    statusLabelSrc.indexOf("Theme.urgent") === -1,
  "SettingRow status text uses muted value type, not a badge color",
);
assert(
  settingRowSrc.indexOf("readonly property bool stack:") !== -1 &&
    settingRowSrc.indexOf("root.stretchControl") !== -1 &&
    settingRowSrc.indexOf("Theme.controlColumnWidth") !== -1,
  "SettingRow stacks the control below the copy when the row is narrow or the control is wide",
);
assert(
  /stack:[^}]*root\.controlCol/.test(settingRowSrc) === false,
  "SettingRow stack does not read controlCol (that binding looped)",
);
assert(
  settingRowSrc.indexOf("parent: root.stack") === -1,
  "SettingRow does not reparent the control slot (that hid toggles)",
);
assert(
  settingRowSrc.indexOf("readonly property int controlCount:") !== -1 &&
    settingRowSrc.indexOf("controlSlot.children.length") !== -1,
  "SettingRow tracks control children so a toggle is not dropped after load",
);
assert(
  settingRowSrc.indexOf("property alias leading:") !== -1 &&
    settingRowSrc.indexOf("property bool interactive:") !== -1,
  "SettingRow can lead with a checkbox and toggle from the row",
);
const collectionRowSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "CollectionRow.qml"),
  "utf8",
);
assert(
  collectionRowSrc.indexOf("import QtQuick") !== -1 &&
    collectionRowSrc.indexOf("property string action:") !== -1 &&
    collectionRowSrc.indexOf("property string dangerAction:") !== -1,
  "CollectionRow is name, status, and a right-hand action",
);
assert(
  collectionRowSrc.indexOf("root.hovered || root.activeFocus") === -1 &&
    collectionRowSrc.indexOf("visible: root.dangerAction.length > 0") !== -1,
  "CollectionRow keeps Remove visible instead of revealing it on hover",
);
assert(
  fs
    .readFileSync(path.join(__dirname, "..", "pages", "SoftwarePage.qml"), "utf8")
    .indexOf("CollectionRow") !== -1 &&
    fs
      .readFileSync(path.join(__dirname, "..", "pages", "ApplicationsPage.qml"), "utf8")
      .indexOf("CollectionRow") !== -1 &&
    fs
      .readFileSync(path.join(__dirname, "..", "pages", "AccountsPage.qml"), "utf8")
      .indexOf("CollectionRow") !== -1 &&
    fs
      .readFileSync(path.join(__dirname, "..", "pages", "HooksPage.qml"), "utf8")
      .indexOf("CollectionRow") !== -1,
  "Software, Applications, Accounts, and Hooks use CollectionRow for object lists",
);
const softwareSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "SoftwarePage.qml"),
  "utf8",
);
assert(
  softwareSrc.indexOf('text: "Installed"') === -1,
  "installed software without a remove action does not keep a blank or status button",
);
assert(
  softwareSrc.indexOf("action:") !== -1 && softwareSrc.indexOf('"Install…"') !== -1,
  "software list Install opens a confirm, so the action uses an ellipsis",
);
const accountsSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AccountsPage.qml"),
  "utf8",
);
assert(
  accountsSrc.indexOf('"Manage…"') !== -1 &&
    accountsSrc.indexOf("id: manageGroupDialog") !== -1 &&
    accountsSrc.indexOf("Managing members") === -1 &&
    accountsSrc.indexOf('action: "Select"') === -1 &&
    accountsSrc.indexOf('text: "Select"') === -1,
  "Accounts groups open Manage… to add or remove members",
);
assert(
  accountsSrc.indexOf('text: "Admin"') !== -1 &&
    accountsSrc.indexOf("Puts this login in wheel.") !== -1 &&
    accountsSrc.indexOf("Admin (wheel)") === -1,
  "Add a user names the wheel toggle Admin",
);
const hooksSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "HooksPage.qml"), "utf8");
assert(hooksSrc.indexOf('text: "Open folder"') !== -1, "Hooks opens a directory with Open folder");
assert(hooksSrc.indexOf("Choose file") === -1, "hook file pickers use Choose…");
const disksSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "DisksPage.qml"), "utf8");
assert(disksSrc.indexOf('"Set up…"') !== -1, "hibernation setup uses Set up…");
assert(disksSrc.indexOf("Setup…") === -1, "Set up is two words");
assert(disksSrc.indexOf('"Run now"') !== -1, "disk speed tests use Run now");
const securitySrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "SecurityPage.qml"),
  "utf8",
);
assert(
  securitySrc.indexOf('text: "Turn on…"') !== -1 && securitySrc.indexOf("Enable…") === -1,
  "passwordless sudo pairs Turn on… with Turn off…",
);
const captureSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "CapturePage.qml"), "utf8");
assert(
  captureSrc.indexOf('text: "Open folder"') !== -1,
  "Capture opens Pictures and Videos with Open folder",
);
const networkPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "NetworkPage.qml"),
  "utf8",
);
assert(
  networkPageSrc.indexOf('text: "Manage…"') !== -1 &&
    networkPageSrc.indexOf('label: "Wi-Fi"') !== -1 &&
    networkPageSrc.indexOf('label: "Bluetooth"') !== -1 &&
    networkPageSrc.indexOf('text: "Test…"') !== -1,
  "Wi-Fi and Bluetooth use Manage… and speed test uses Test…",
);
const a11ySrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AccessibilityPage.qml"),
  "utf8",
);
assert(a11ySrc.indexOf('text: "Launch"') !== -1, "Herdr launches the screen reader");
function walkQml(dir, acc) {
  acc = acc || [];
  fs.readdirSync(dir, { withFileTypes: true }).forEach(function (ent) {
    const next = path.join(dir, ent.name);
    if (ent.isDirectory()) walkQml(next, acc);
    else if (ent.name.endsWith(".qml")) acc.push(next);
  });
  return acc;
}
const qmlFiles = walkQml(path.join(__dirname, "..", "pages")).concat(
  walkQml(path.join(__dirname, "..", "components")),
  [path.join(__dirname, "..", "shell.qml")],
);
const bannedAction = [
  ['"Setup…"', "use Set up…"],
  ['"Choose file…"', "use Choose…"],
  ['"Choose logo…"', "use Choose…"],
  ['"Browse…"', "use Choose…"],
  ['"Image…"', "use Choose…"],
  ['"Enable…"', "pair Turn on… with Turn off…"],
  ['action: "Select"', "do not use Select for membership"],
  ['text: "Select"', "do not use Select as an action label"],
  ['text: "Selected"', "do not use Selected as an action label"],
  ['text: "Folder"', "opening a directory is Open folder"],
  ['text: "Apply times"', "committing typed values is Set"],
  ['text: "Open"', "name the destination: Configure…, Manage…, Choose…, Test…, Open folder"],
];
qmlFiles.forEach(function (file) {
  const src = fs.readFileSync(file, "utf8");
  bannedAction.forEach(function (pair) {
    assert(
      src.indexOf(pair[0]) === -1,
      path.relative(path.join(__dirname, ".."), file) + ": " + pair[1],
    );
  });
  assert(
    !/PrefsToggle\s*\{[^}]*enabled:\s*false\b/.test(src),
    path.relative(path.join(__dirname, ".."), file) +
      ": a switch with enabled: false looks like a broken control; use SettingRow valueText for read-only state",
  );
});
const hardwarePageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "HardwarePage.qml"),
  "utf8",
);
assert(hardwarePageSrc.indexOf("PrefsToggle") === -1, "Hardware does not use switches");
assert(
  hardwarePageSrc.indexOf('label: "Secure Boot"') !== -1 &&
    hardwarePageSrc.indexOf('valueText: root.hw.secureBoot.enabled ? "On" : "Off"') !== -1 &&
    hardwarePageSrc.indexOf("Change this in UEFI setup, not here.") !== -1,
  "Secure Boot is On/Off status copy, not a switch",
);
const idlePageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "IdlePage.qml"), "utf8");
assert(
  idlePageSrc.indexOf('label: "Lid close"') !== -1 &&
    idlePageSrc.indexOf('valueText: "On"') !== -1 &&
    idlePageSrc.indexOf("omarchy-system-lid-close") !== -1,
  "Lid close is On status copy, not a switch",
);
const barPageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "BarPage.qml"), "utf8");
assert(
  barPageSrc.indexOf('valueText: modelData && modelData.canDisable === false ? "On" : ""') !== -1 &&
    barPageSrc.indexOf("visible: !(modelData && modelData.canDisable === false)") !== -1,
  "a bar plugin that cannot be disabled is On status, not a disabled switch",
);
assert(
  barPageSrc.indexOf('description: "No hidden tray icons."') !== -1 &&
    barPageSrc.indexOf('description: "No pinned tray icons."') !== -1 &&
    barPageSrc.indexOf("stay listed here") === -1,
  "empty tray lists say so directly instead of leaving a blank control",
);
assert(
  barPageSrc.indexOf('description: "No plugins listed."') !== -1,
  "an empty plugin list says so directly",
);
assert(
  barPageSrc.indexOf('label: "Show bar"') !== -1 &&
    barPageSrc.indexOf('description: "Keep the bar visible. Turn this off to hide it."') !== -1 &&
    barPageSrc.indexOf("Hide the bar.") === -1,
  "Show bar describes the on state, not hide",
);
const idleToggleSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "IdlePage.qml"), "utf8");
assert(
  idleToggleSrc.indexOf('label: "Screensaver"') !== -1 &&
    idleToggleSrc.indexOf('description: "The screensaver runs after the idle timeout."') !== -1 &&
    idleToggleSrc.indexOf("Off keeps the desktop") === -1,
  "Screensaver describes the on state",
);
assert(
  idleToggleSrc.indexOf('label: "Suspend menu"') !== -1 &&
    idleToggleSrc.indexOf('description: "Suspend stays in the system menu."') !== -1 &&
    idleToggleSrc.indexOf("Off hides that action") === -1,
  "Suspend menu describes the on state",
);
const soundPageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "SoundPage.qml"), "utf8");
assert(
  soundPageSrc.indexOf('description: "Cut microphone input while this is on."') !== -1 &&
    soundPageSrc.indexOf('description: "Silence speakers and headphones while this is on."') !== -1,
  "mute switches describe the on state",
);
function emptyStateHasNoCreate(src, description, label) {
  const re = new RegExp(
    'description: "' +
      description.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") +
      '"[\\s\\S]{0,240}PrefsButton',
  );
  assert(!re.test(src), label + " empty state does not repeat a create action");
}
const notificationsSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "NotificationsPage.qml"),
  "utf8",
);
assert(
  notificationsSrc.indexOf('description: "No reminders waiting."') !== -1 &&
    notificationsSrc.indexOf("None waiting") === -1,
  "an empty reminder list says No reminders waiting",
);
assert(
  notificationsSrc.indexOf(
    'description: "Hide ordinary notification toasts. Critical alerts still get through."',
  ) !== -1,
  "Do not disturb describes the on state",
);
emptyStateHasNoCreate(notificationsSrc, "No reminders waiting.", "Reminders");
const applicationsSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "ApplicationsPage.qml"),
  "utf8",
);
assert(
  applicationsSrc.indexOf('description: "No launch-on-start commands."') !== -1 &&
    applicationsSrc.indexOf('description: "No desktop launchers."') !== -1 &&
    applicationsSrc.indexOf('description: "No terminal launchers."') !== -1 &&
    applicationsSrc.indexOf('description: "No web apps."') !== -1 &&
    applicationsSrc.indexOf("You have not added") === -1,
  "empty application lists say so directly",
);
emptyStateHasNoCreate(applicationsSrc, "No launch-on-start commands.", "Autostart");
emptyStateHasNoCreate(applicationsSrc, "No desktop launchers.", "Desktop launchers");
emptyStateHasNoCreate(applicationsSrc, "No terminal launchers.", "Terminal launchers");
emptyStateHasNoCreate(applicationsSrc, "No web apps.", "Web apps");
const hooksPageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "HooksPage.qml"), "utf8");
assert(
  hooksPageSrc.indexOf('description: "No scripts."') !== -1,
  "empty hook lists say No scripts",
);
emptyStateHasNoCreate(hooksPageSrc, "No scripts.", "Hooks");
const bindingsPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "windows", "BindingsPage.qml"),
  "utf8",
);
assert(
  bindingsPageSrc.indexOf('description: "No personal bindings."') !== -1 &&
    bindingsPageSrc.indexOf('description: "No bindings reported."') !== -1 &&
    bindingsPageSrc.indexOf('description: "No matching bindings."') !== -1,
  "empty binding lists say so directly",
);
emptyStateHasNoCreate(bindingsPageSrc, "No personal bindings.", "Overrides");
const rulesPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "windows", "RulesPage.qml"),
  "utf8",
);
assert(
  rulesPageSrc.indexOf('description: "No personal window rules."') !== -1,
  "an empty window-rule list says so directly",
);
emptyStateHasNoCreate(rulesPageSrc, "No personal window rules.", "Rules");
const bluetoothPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "network", "BluetoothPage.qml"),
  "utf8",
);
assert(
  bluetoothPageSrc.indexOf('description: "No paired devices."') !== -1 &&
    bluetoothPageSrc.indexOf('description: "No unpaired devices nearby."') !== -1,
  "empty Bluetooth lists say so directly",
);
emptyStateHasNoCreate(bluetoothPageSrc, "No paired devices.", "Paired devices");
const wifiPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "network", "WifiPage.qml"),
  "utf8",
);
assert(
  wifiPageSrc.indexOf('description: "No Wi-Fi adapter."') !== -1 ||
    wifiPageSrc.indexOf('"No Wi-Fi adapter."') !== -1,
  "empty Wi-Fi list names a missing adapter",
);
assert(
  wifiPageSrc.indexOf('"No networks nearby."') !== -1 &&
    wifiPageSrc.indexOf("Refresh stays disabled") === -1,
  "empty Wi-Fi list says No networks nearby",
);
const accountsPageEmptySrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AccountsPage.qml"),
  "utf8",
);
assert(
  accountsPageEmptySrc.indexOf('description: "No human logins."') !== -1 &&
    accountsPageEmptySrc.indexOf('description: "No groups."') !== -1,
  "empty account lists say so directly",
);
const disksPageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "DisksPage.qml"), "utf8");
assert(disksPageSrc.indexOf('description: "No snapshots."') !== -1, "empty snapshot list says so");
emptyStateHasNoCreate(disksPageSrc, "No snapshots.", "Snapshots");
const displaysPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "DisplaysPage.qml"),
  "utf8",
);
assert(
  displaysPageSrc.indexOf('description: "No monitors reported."') !== -1,
  "a missing monitor list says No monitors reported",
);
const prefsCheckSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsCheck.qml"),
  "utf8",
);
assert(
  prefsCheckSrc.indexOf("Accessible.role: Accessible.CheckBox") !== -1 &&
    prefsCheckSrc.indexOf("Keys.onSpacePressed") !== -1,
  "PrefsCheck is a keyboard-accessible checkbox",
);
const exportPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "ExportPage.qml"),
  "utf8",
);
assert(exportPageSrc.indexOf("PrefsCheck") !== -1, "export sections use PrefsCheck");
assert(exportPageSrc.indexOf("PrefsToggle") === -1, "export sections do not use switches");
assert(
  exportPageSrc.indexOf('text: "All"') !== -1 && exportPageSrc.indexOf('text: "None"') !== -1,
  "export keeps All and None actions",
);
assert(
  exportPageSrc.indexOf('caption: modelData.count + " settings"') !== -1,
  "export keeps the setting count as secondary copy",
);
assert(
  exportPageSrc.indexOf('text: "Open file"') !== -1,
  "export opens the written file with Open file",
);
const prefsRowSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsRow.qml"),
  "utf8",
);
assert(
  /^\s*SettingRow\s*\{/m.test(prefsRowSrc),
  "PrefsRow is a SettingRow alias so existing rows keep working",
);
const prefsGroupSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsGroup.qml"),
  "utf8",
);
assert(
  prefsGroupSrc.indexOf("root.title.toUpperCase()") !== -1 &&
    prefsGroupSrc.indexOf("color: Theme.muted") !== -1 &&
    prefsGroupSrc.indexOf("Theme.sectionSize") !== -1,
  "section headings are uppercase muted labels, not setting-row titles",
);
assert(
  prefsGroupSrc.indexOf("property bool framed: false") !== -1,
  "ordinary PrefsGroups are a heading and rows, not a card",
);
assert(
  /\n  spacing: Theme.headingGap\n/.test(prefsGroupSrc),
  "section headings sit close to the first row in their group",
);
assert(
  prefsGroupSrc.indexOf("visible: root.framed") !== -1,
  "PrefsGroup draws a bordered card only when framed",
);
assert(
  settingRowSrc.indexOf("Theme.splitColor()") !== -1 &&
    settingRowSrc.indexOf("visible: root.split") !== -1,
  "SettingRow uses a hairline instead of a per-row box",
);
assert(
  prefsGroupSrc.indexOf("kid.split = !(root.framed && first)") !== -1,
  "the first row in a framed card has no hairline under the card edge",
);

const shellSrc = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");
const navItemSrc = shellSrc.slice(
  shellSrc.indexOf("id: navItem"),
  shellSrc.indexOf("id: navMouse"),
);
const navHighlightSrc = shellSrc.slice(
  shellSrc.indexOf("id: navHighlight"),
  shellSrc.indexOf("id: navColumn"),
);
assert(navItemSrc.indexOf("id: navItem") !== -1, "sidebar hubs are a named navItem");
assert(
  navItemSrc.indexOf("readonly property bool selected:") !== -1 &&
    navItemSrc.indexOf("root.currentPage === modelData.id") !== -1,
  "sidebar selected state is the current hub, not hover",
);
assert(
  navItemSrc.indexOf("Theme.fill(Theme.hoverFill)") !== -1 &&
    navItemSrc.indexOf('"transparent"') !== -1 &&
    navHighlightSrc.indexOf("Theme.fill(Theme.selectedFill)") === -1,
  "sidebar hover uses a fill; the current hub is the sliding rail, not a selected box",
);
assert(
  navItemSrc.indexOf("navItem.hovered || navItem.activeFocus") !== -1,
  "sidebar hover fill also shows keyboard focus",
);
assert(
  shellSrc.indexOf("function placeNavHighlight(") !== -1,
  "sidebar moves one highlight to the current hub",
);
assert(
  navHighlightSrc.indexOf("Behavior on y") !== -1 &&
    navHighlightSrc.indexOf("NumberAnimation") !== -1,
  "sidebar selected highlight slides instead of fading",
);
assert(
  navHighlightSrc.indexOf("width: Theme.railWidth") !== -1 &&
    navHighlightSrc.indexOf("color: Theme.accent") !== -1,
  "sidebar current hub is the accent rail token",
);
assert(
  navItemSrc.indexOf("border.width") === -1 && navItemSrc.indexOf("border.color") === -1,
  "sidebar hubs have no focus outline",
);
assert(
  navItemSrc.indexOf("Accessible.role: Accessible.Button") !== -1 &&
    navItemSrc.indexOf("Keys.onReturnPressed") !== -1,
  "sidebar hubs are keyboard-activable buttons",
);
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
const appearanceSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AppearancePage.qml"),
  "utf8",
);
assert(
  appearanceSrc.indexOf("HyprSunset.parseTime") !== -1 &&
    appearanceSrc.indexOf("nightTimesValid") !== -1,
  "Appearance validates night-light times before apply or the night-profile toggle",
);
assert(
  appearanceSrc.indexOf("enabled: root.nightTimesValid") !== -1,
  "Appearance disables night-light apply when a time is invalid",
);
assert(
  appearanceSrc.indexOf("RichUi.parseGitUrl") !== -1 &&
    appearanceSrc.indexOf("themeUrlValid") !== -1,
  "Appearance validates a theme git URL before Install",
);
assert(
  appearanceSrc.indexOf("enabled: !Omarchy.jobBusy && root.themeUrlValid") !== -1,
  "Appearance disables theme Install until the git URL parses",
);
assert(
  appearanceSrc.indexOf('description: "No extra themes installed."') !== -1 &&
    appearanceSrc.indexOf("stay disabled until a clone exists") === -1,
  "empty extra themes say so without disabled Update/Remove",
);
assert(
  appearanceSrc.indexOf("Shift colors toward amber at night.") !== -1,
  "Night light describes the on state",
);
assert(
  appearanceSrc.indexOf('label: "Background"') !== -1 &&
    appearanceSrc.indexOf('text: "Choose…"') !== -1 &&
    appearanceSrc.indexOf('label: "Boot screen"') !== -1 &&
    appearanceSrc.indexOf('text: "Configure…"') !== -1,
  "background uses Choose… and boot screen uses Configure…",
);
assert(
  omarchyQml.indexOf("url = RichUi.parseGitUrl(url)") !== -1,
  "Omarchy validates theme URLs with RichUi.parseGitUrl",
);
const systemSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "SystemPage.qml"), "utf8");
assert(
  systemSrc.indexOf("RichUi.parseHostname") !== -1 && systemSrc.indexOf("hostnameValid") !== -1,
  "System validates hostname before Set",
);
assert(
  systemSrc.indexOf("enabled: root.hostnameValid && root.hostnameParsed !== Omarchy.hostname") !==
    -1,
  "System disables hostname Set when the name is invalid or unchanged",
);
assert(
  omarchySrc.indexOf("name = RichUi.parseHostname(name)") !== -1,
  "Omarchy validates hostname with RichUi.parseHostname",
);
assert(
  systemSrc.indexOf("RichUi.parseWeatherLocation") !== -1 &&
    systemSrc.indexOf("weatherLocationValid") !== -1,
  "System validates weather location before Set",
);
assert(
  systemSrc.indexOf(
    "enabled: root.weatherLocationValid && (Omarchy.weatherAuto || root.weatherLocationParsed !== Omarchy.weatherLocation)",
  ) !== -1,
  "System disables weather-location Set when the name is invalid, empty, or unchanged",
);
assert(
  omarchySrc.indexOf("name = RichUi.parseWeatherLocation(name)") !== -1,
  "Omarchy validates weather location with RichUi.parseWeatherLocation",
);
assert(
  omarchySrc.indexOf('apply: { weatherLocation: name, weatherAuto: false, weatherCoords: "" }') !==
    -1,
  "Omarchy drops stale weather coordinates when the city changes",
);
assert(
  omarchySrc.indexOf('apply: { weatherLocation: "", weatherAuto: true, weatherCoords: "" }') !== -1,
  "Omarchy drops weather coordinates when location returns to auto",
);
assert(
  systemSrc.indexOf("RichUi.parseWeatherCoords") !== -1 &&
    systemSrc.indexOf("weatherCoordsValid") !== -1,
  "System validates weather coordinates before Set",
);
assert(
  systemSrc.indexOf(
    "enabled: root.weatherCoordsValid && root.weatherCoordsParsed !== Omarchy.weatherCoords",
  ) !== -1,
  "System disables weather-coordinate Set when the pair is invalid or unchanged",
);
assert(
  omarchySrc.indexOf("coords = RichUi.parseWeatherCoords(coords)") !== -1,
  "Omarchy validates weather coordinates with RichUi.parseWeatherCoords",
);
assert(
  omarchySrc.indexOf("/^-?[0-9]+(\\.[0-9]+)?,-?[0-9]+(\\.[0-9]+)?$/") === -1,
  "Omarchy does not keep a space-intolerant weather-coordinate regex",
);
assert(
  securitySrc.indexOf("RichUi.parseSshPublicKey") !== -1 &&
    securitySrc.indexOf("sshKeyValid") !== -1,
  "Security validates an SSH public key before Turn on",
);
assert(
  securitySrc.indexOf("currentText().length > 20") === -1,
  "Security does not enable SSH from a length check that cannot track typing",
);
assert(
  omarchySrc.indexOf("key = RichUi.parseSshPublicKey(key)") !== -1,
  "Omarchy validates SSH keys with RichUi.parseSshPublicKey",
);
const inputPageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "InputPage.qml"), "utf8");
assert(
  inputPageSrc.indexOf("HyprPrefs.sanitizeLayoutList") !== -1 &&
    inputPageSrc.indexOf("kbLayoutValid") !== -1,
  "Input validates Hyprland layout lists before Set",
);
assert(
  inputPageSrc.indexOf("HyprPrefs.sanitizeVariantList") !== -1 &&
    inputPageSrc.indexOf("kbVariantValid") !== -1,
  "Input validates Hyprland variant lists before Set",
);
assert(
  inputPageSrc.indexOf("enabled: root.kbOverrideValid && root.kbOverrideDirty") !== -1,
  "Input disables layout Set when the list is invalid or unchanged",
);
assert(
  omarchySrc.indexOf("layouts = HyprPrefs.sanitizeLayoutList(layouts)") !== -1,
  "Omarchy validates Hyprland layouts with HyprPrefs.sanitizeLayoutList",
);
assert(
  omarchySrc.indexOf("/^[a-z0-9]{1,8}(,[a-z0-9]{1,8})*$/") === -1,
  "Omarchy does not keep a space-intolerant Hyprland layout regex",
);
const accountsPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AccountsPage.qml"),
  "utf8",
);
assert(
  accountsPageSrc.indexOf("AccountsJs.parseFullName") !== -1 &&
    accountsPageSrc.indexOf("fullNameValid") !== -1,
  "Accounts validates full name before Set",
);
assert(
  accountsPageSrc.indexOf(
    "enabled: root.fullNameValid && root.fullNameParsed !== Omarchy.fullName",
  ) !== -1,
  "Accounts disables full-name Set when the name is invalid or unchanged",
);
assert(
  omarchySrc.indexOf("if (!AccountsJs.isFullName(name)) return") !== -1,
  "Omarchy validates full name with AccountsJs.isFullName"
);

// runGumJob passes the stub dir as $1 and the command after it. Without the
// shift, "$@" still carries $1 and exec is handed the directory itself:
//   prefs-job: .../scripts/stubs: Is a directory
assert(
  omarchySrc.indexOf('PATH=\"$1:$PATH\"; shift; exec \"$@\"') !== -1,
  "runGumJob shifts the stub dir off before exec",
);
