const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

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
assert(
  searchPageSrc.indexOf("root.navigator.go(modelData.hub, modelData.label)") !== -1,
  "SearchPage passes the hit label so Simple can pin the landing row",
);
const omarchyQml = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
assert(
  omarchyQml.indexOf('enqueueRead(ioQueue, "rest")') !== -1,
  "startSession follows look with rest",
);
assert(
  omarchyQml.indexOf("function copyDiagnosticReport()") !== -1 &&
    omarchyQml.indexOf("diag-report.sh") !== -1,
  "Omarchy copies a diagnostic report through diag-report.sh",
);
assert(
  omarchyQml.indexOf("Diagnostics.js") !== -1 && omarchyQml.indexOf("normalizeDiagnostics") !== -1,
  "Omarchy normalizes diagnostics through Diagnostics.js",
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
  selectSrc.indexOf("readonly property bool overlayOpen:") !== -1 &&
    selectSrc.indexOf("overlayOpen: popup.opened") !== -1,
  "PrefsSelect exposes overlayOpen so SettingRow can keep the row rail",
);
const menuSrc = fs.readFileSync(path.join(__dirname, "..", "components", "PrefsMenu.qml"), "utf8");
assert(
  menuSrc.indexOf("readonly property bool overlayOpen:") !== -1 &&
    menuSrc.indexOf("overlayOpen: popup.opened") !== -1,
  "PrefsMenu exposes overlayOpen so SettingRow can keep the row rail",
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
  helpSrc.indexOf("Keys.onReturnPressed: root.togglePopup()") !== -1 &&
    helpSrc.indexOf("Keys.onSpacePressed: root.togglePopup()") !== -1,
  "PrefsHelp toggles from Enter and Space",
);
assert(
  helpSrc.indexOf("ToolTip") !== -1 && helpSrc.indexOf("helpMouse.containsMouse") !== -1,
  "PrefsHelp exposes the extra copy on hover",
);
const prefsGroupSrcEarly = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsGroup.qml"),
  "utf8",
);
const helpTipSrc = helpSrc.slice(
  helpSrc.indexOf("ToolTip {"),
  helpSrc.indexOf("component CommandBox"),
);
assert(
  helpTipSrc.indexOf("background:") !== -1 &&
    helpTipSrc.indexOf("color: Theme.background") !== -1 &&
    helpTipSrc.indexOf("border.width: Theme.borderWidth") !== -1 &&
    helpTipSrc.indexOf("border.color: Theme.borderColor()") !== -1 &&
    helpTipSrc.indexOf("radius: Theme.radius") !== -1,
  "section extra-copy hover uses Atmos surface, hairline, and radius",
);
assert(
  helpTipSrc.indexOf("contentItem: Text") !== -1 &&
    helpTipSrc.indexOf("color: Theme.foreground") !== -1,
  "section extra-copy hover text uses Theme.foreground",
);
const helpTipHead = helpTipSrc.slice(0, helpTipSrc.indexOf("contentItem:"));
assert(
  helpTipHead.indexOf("padding: Theme.pad") !== -1 &&
    /(?:^|\n)\s+(?:width|implicitWidth):[^\n]*Theme\.(?:confirmWidth|dialogWidth)/.test(
      helpTipHead,
    ) &&
    helpTipHead.indexOf("width: 360") === -1,
  "section extra-copy hover is width-capped and padded with Theme tokens",
);
assert(
  helpTipSrc.indexOf("wrapMode: Text.Wrap") !== -1 && helpTipSrc.indexOf("availableWidth") !== -1,
  "section extra-copy hover wraps to the capped ToolTip width",
);
assert(
  prefsGroupSrcEarly.indexOf("PrefsHelp") !== -1 &&
    helpSrc.indexOf("ToolTip {") !== -1 &&
    helpTipSrc.indexOf("color: Theme.background") !== -1,
  "section extra-copy hover goes through PrefsHelp's themed ToolTip",
);
assert(
  helpSrc.indexOf("property bool reveal:") !== -1 &&
    helpSrc.indexOf("opacity: root.showIcon ? 1 : 0") !== -1,
  "PrefsHelp keeps the heading slot and only paints the glyph when revealed",
);
assert(
  prefsGroupSrcEarly.indexOf("sectionHelpOpen") !== -1 &&
    prefsGroupSrcEarly.indexOf("root.detail.length > 0 || root.hint.length > 0") === -1,
  "section info icon shows only when help adds context",
);
assert(
  prefsGroupSrcEarly.indexOf("id: headingHost") !== -1 &&
    prefsGroupSrcEarly.indexOf("reveal: headingHover.hovered") !== -1,
  "section info reveals when the heading row is hovered",
);
const snapshotSh = fs.readFileSync(path.join(__dirname, "..", "scripts", "snapshot.sh"), "utf8");
assert(snapshotSh.indexOf("GROUP == rest") !== -1, "snapshot.sh strips look keys from rest");
assert(
  snapshotSh.indexOf('line.split("#", 1)[0]') !== -1,
  "snapshot.sh hyprsunset parser ignores commented profiles",
);
assert(
  snapshotSh.indexOf('hypr-sentinel.py" input list') !== -1,
  "snapshot.sh reads workspaceGesture from hypr-sentinel.py input list",
);
assert(
  snapshotSh.indexOf("workspaceGestureUnmanaged") !== -1,
  "snapshot.sh exposes an unmanaged workspace gesture separately from a managed one",
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
    themeQml.indexOf("readonly property int chamfer:") !== -1 &&
    themeQml.indexOf("readonly property int chamferSm:") !== -1 &&
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
assert(
  prefsButtonSrc.indexOf("TextMetrics") !== -1 && prefsButtonSrc.indexOf("labelMetrics") !== -1,
  "PrefsButton sizes from TextMetrics so a hidden row still gets a width",
);
assert(
  prefsButtonSrc.indexOf("Chamfer") !== -1 &&
    prefsButtonSrc.indexOf("Theme.chamferSm") !== -1 &&
    prefsButtonSrc.indexOf('color: "transparent"') !== -1 &&
    prefsButtonSrc.indexOf("border.width: 0") !== -1,
  "PrefsButton keeps a Rectangle root and paints a chamfered silhouette",
);
assert(
  prefsButtonSrc.indexOf("Behavior on bodyColor") !== -1 &&
    prefsButtonSrc.indexOf("Behavior on edgeColor") !== -1 &&
    prefsButtonSrc.indexOf("mouse.containsMouse") !== -1 &&
    prefsButtonSrc.indexOf("root.activeFocus") !== -1 &&
    prefsButtonSrc.indexOf("Theme.fill(Theme.hoverFill)") !== -1 &&
    prefsButtonSrc.indexOf("Theme.accentFill(Theme.primaryFill)") !== -1,
  "PrefsButton still animates hover, focus, and primary fill on the chamfer",
);
assert(
  prefsButtonSrc.indexOf("anchors.fill: parent") !== -1 &&
    prefsButtonSrc.indexOf("hoverEnabled: true") !== -1,
  "PrefsButton keeps a full-rect MouseArea so a transparent root still clicks",
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
assert(
  prefsToggleSrc.indexOf("signal toggled(bool next)") !== -1 &&
    prefsToggleSrc.indexOf("root.toggled(next)") !== -1,
  "PrefsToggle tells the handler the next checked value",
);
const powerPageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "PowerPage.qml"), "utf8");
assert(
  powerPageSrc.indexOf("onToggled: function(next) { Omarchy.setPresentationMode(next) }") !== -1 &&
    powerPageSrc.indexOf("setPresentationMode(!Omarchy.presentationMode)") === -1,
  "Presentation Mode uses the toggle next value, not a stale invert",
);
const presentationSh = fs.readFileSync(
  path.join(__dirname, "..", "scripts", "set-presentation.sh"),
  "utf8",
);
assert(
  presentationSh.indexOf("omarchy toggle idle stay-awake") !== -1 &&
    presentationSh.indexOf("omarchy toggle idle allow-idle") !== -1 &&
    presentationSh.indexOf("omarchy toggle idle on") === -1,
  "presentation script uses stay-awake and allow-idle",
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
assert(
  prefsPageSrc.indexOf("readonly property bool showDisclosure:") !== -1 &&
    prefsPageSrc.indexOf("DisclosureJs.showModeToggle(root.hasAdvanced") !== -1,
  "PrefsPage shows Simple/Everything only on pages that opted in",
);
assert(
  prefsPageSrc.indexOf(
    "visible: root.query.length === 0 && !root.embed && root.title.length > 0",
  ) === -1,
  "PrefsPage does not paint the mode toggle on every titled hub",
);

const disclosureQml = fs.readFileSync(
  path.join(__dirname, "..", "services", "Disclosure.qml"),
  "utf8",
);
assert(
  disclosureQml.indexOf("pragma Singleton") !== -1 &&
    disclosureQml.indexOf("property bool simple: false") !== -1 &&
    disclosureQml.indexOf("Settings") === -1 &&
    disclosureQml.indexOf("store") === -1,
  "Disclosure is a session singleton and is not persisted",
);
const qmldir = fs.readFileSync(path.join(__dirname, "..", "services", "qmldir"), "utf8");
assert(
  qmldir.indexOf("singleton Disclosure 1.0 Disclosure.qml") !== -1,
  "qmldir registers the Disclosure singleton",
);
const chromeSrc = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");
assert(
  chromeSrc.indexOf("function go(path, label)") !== -1 &&
    chromeSrc.indexOf("Disclosure.revealFromSearch(path, label)") !== -1 &&
    chromeSrc.indexOf("Disclosure.finishReveal(hub)") !== -1 &&
    chromeSrc.indexOf("Disclosure.leaveHub(hubId(id))") !== -1,
  "chrome search pins the landing row and drops the pin when leaving the hub",
);

const settingRowSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "SettingRow.qml"),
  "utf8",
);
assert(
  settingRowSrc.indexOf("function toggleStar()") !== -1 &&
    settingRowSrc.indexOf("Theme.iconStar") !== -1 &&
    settingRowSrc.indexOf("Omarchy.toggleFavorite") !== -1,
  "SettingRow stars a row through Omarchy.toggleFavorite",
);
assert(
  settingRowSrc.indexOf("property bool advanced: false") !== -1 &&
    settingRowSrc.indexOf("DisclosureJs.rowFolded") !== -1 &&
    settingRowSrc.indexOf("shown: available && matches && !folded") !== -1,
  "SettingRow folds through Disclosure.js so search can pin the landing row",
);
assert(
  prefsGroupSrcEarly.indexOf("property bool advanced: false") !== -1 &&
    prefsGroupSrcEarly.indexOf("DisclosureJs.groupFolded") !== -1 &&
    prefsGroupSrcEarly.indexOf("collectPrefsRows({ includeFolded: true })") !== -1,
  "PrefsGroup folds Advanced sections with the same Disclosure model and keeps them in section help",
);
assert(
  settingRowSrc.indexOf("readonly property int favoriteGutter:") !== -1 &&
    settingRowSrc.indexOf("anchors.right: parent.right") !== -1 &&
    settingRowSrc.indexOf("opacity: root.showFavoriteIcon ? 1 : 0") !== -1 &&
    settingRowSrc.indexOf("id: favoriteHost") !== -1 &&
    settingRowSrc.indexOf("id: controlHost") < settingRowSrc.indexOf("id: favoriteHost"),
  "the favorite star sits in a reserved gutter to the right of the control and only paints on hover",
);
const favoriteHostSrc = settingRowSrc.slice(
  settingRowSrc.indexOf("id: favoriteHost"),
  settingRowSrc.indexOf("id: controlSlot"),
);
assert(
  favoriteHostSrc.indexOf("anchors.top: parent.top") !== -1 &&
    favoriteHostSrc.indexOf("anchors.verticalCenter") === -1,
  "the favorite star stays top-aligned in the control band",
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
  settingRowSrc.indexOf("readonly property bool shown:") !== -1 &&
    settingRowSrc.indexOf("readonly property bool hasChild: root.controlCount > 0") !== -1,
  "SettingRow counts control children even when the row started hidden",
);
assert(
  settingRowSrc.indexOf("property alias leading:") !== -1 &&
    settingRowSrc.indexOf("property bool interactive:") !== -1,
  "SettingRow can lead with a checkbox and toggle from the row",
);
assert(
  settingRowSrc.indexOf("import QtQuick.Window") !== -1 &&
    settingRowSrc.indexOf("root.Window.activeFocusItem") !== -1 &&
    settingRowSrc.indexOf("readonly property bool focusInside:") !== -1 &&
    settingRowSrc.indexOf("item === root") !== -1 &&
    settingRowSrc.indexOf("item = item.parent") !== -1 &&
    settingRowSrc.indexOf("guard < 40") !== -1,
  "SettingRow walks Window.activeFocusItem ancestry, not root.activeFocus",
);
assert(
  /readonly property bool (focusInside|keyboardHere|lit):[\s\S]{0,200}root\.activeFocus/.test(
    settingRowSrc,
  ) === false,
  "SettingRow does not treat root.activeFocus as the row-lit test",
);
assert(
  settingRowSrc.indexOf("visible: root.lit") !== -1 &&
    settingRowSrc.indexOf("Theme.fill(Theme.hoverFill)") !== -1 &&
    settingRowSrc.indexOf("z: -2") !== -1,
  "SettingRow hover fills the whole row behind the splitter",
);
assert(
  settingRowSrc.indexOf("visible: root.keyboardHere") !== -1 &&
    settingRowSrc.indexOf("Theme.railWidth") !== -1 &&
    settingRowSrc.indexOf("color: Theme.accent") !== -1 &&
    settingRowSrc.indexOf("z: -1") !== -1,
  "SettingRow paints the accent rail only when keyboard ancestry hits",
);
assert(
  settingRowSrc.indexOf("Behavior on opacity") === -1 &&
    settingRowSrc.indexOf("Theme.selectedFill") === -1,
  "SettingRow row chrome is instant hoverFill, not a selected-fill fade",
);
assert(
  settingRowSrc.indexOf("item.overlayOpen === true") !== -1 &&
    settingRowSrc.indexOf("readonly property bool childOverlayOpen:") !== -1,
  "SettingRow keeps the rail while a descendant overlay is open",
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
    networkPageSrc.indexOf('text: "Test…"') !== -1,
  "Wi-Fi uses Manage… and speed test uses Test…",
);
assert(
  networkPageSrc.indexOf('label: "Bluetooth"') !== -1 &&
    networkPageSrc.indexOf('text: "Open…"') !== -1 &&
    networkPageSrc.indexOf("setBluetooth") === -1 &&
    networkPageSrc.indexOf("bluetoothPage") === -1 &&
    networkPageSrc.indexOf('root.navigator.go("bluetooth")') !== -1,
  "Network links to the Bluetooth hub instead of nesting it",
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
  hardwarePageSrc.indexOf("hybridGpuConfirm") === -1 &&
    hardwarePageSrc.indexOf('label: "Active stack"') === -1 &&
    hardwarePageSrc.indexOf("toggleHybridGpu") === -1,
  "Hardware does not host GPU stack or hybrid switching",
);
assert(
  hardwarePageSrc.indexOf('text: "Open…"') !== -1 &&
    hardwarePageSrc.indexOf('navigator.go("drivers")') !== -1,
  "Hardware links to Drivers with Open…",
);
const driversPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "DriversPage.qml"),
  "utf8",
);
assert(
  driversPageSrc.indexOf("hybridGpuConfirm") !== -1 &&
    driversPageSrc.indexOf('label: "Active stack"') !== -1 &&
    driversPageSrc.indexOf("toggleHybridGpu") !== -1 &&
    driversPageSrc.indexOf("updateFirmware") !== -1 &&
    driversPageSrc.indexOf('label: "Firmware"') !== -1,
  "Drivers hosts GPU stack, hybrid switching, and fwupd",
);
assert(driversPageSrc.indexOf("No GPUs reported.") !== -1, "Drivers names an empty GPU list");
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
assert(
  displaysPageSrc.indexOf("root.canDisable(modelData)") !== -1 &&
    displaysPageSrc.indexOf("This is the only display on, so it has to stay on.") !== -1 &&
    displaysPageSrc.indexOf("Another display is mirroring this one.") !== -1,
  "disabling the last display on, or a mirror source, stays off",
);
assert(
  displaysPageSrc.indexOf("enabled: Omarchy.internalPresent") !== -1 &&
    displaysPageSrc.indexOf("enabled: Omarchy.externalPresent") !== -1,
  "layouts that would leave no display on stay off",
);
assert(
  displaysPageSrc.indexOf('label: "Refresh rate"') !== -1 &&
    displaysPageSrc.indexOf("RichUi.monitorResolutions(modelData)") !== -1 &&
    displaysPageSrc.indexOf(
      "RichUi.monitorRefreshRates(modelData, root.resolutionValue(modelData))",
    ) !== -1 &&
    displaysPageSrc.indexOf("RichUi.monitorModeOptions") === -1 &&
    displaysPageSrc.indexOf("MonJs.sanitizeMode(") !== -1 &&
    displaysPageSrc.indexOf("RichUi.monitorModeCopyText(modelData)") !== -1,
  "resolution lists sizes only with refresh rate in its own row behind it",
);
const richUiSrc = fs.readFileSync(path.join(__dirname, "..", "services", "RichUi.js"), "utf8");
assert(
  richUiSrc.indexOf("function monitorModeOptions") === -1 &&
    richUiSrc.indexOf("function monitorResolutions") !== -1 &&
    richUiSrc.indexOf("function monitorRefreshRates") !== -1 &&
    richUiSrc.indexOf("function currentMonitorResolutionValue") !== -1 &&
    richUiSrc.indexOf("function currentMonitorRefreshValue") !== -1 &&
    richUiSrc.indexOf("function monitorModeCopyText") !== -1,
  "RichUi splits size and refresh and drops the combined mode picker",
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
  prefsGroupSrc.indexOf("kid.split = !(root.framed && first)") !== -1 ||
    prefsGroupSrc.indexOf("rows[i].split = !(root.framed && first)") !== -1,
  "the first row in a framed card has no hairline under the card edge",
);
assert(
  prefsGroupSrc.indexOf("function collectPrefsRows(") !== -1 &&
    prefsGroupSrc.indexOf("kid.available === false") !== -1 &&
    prefsGroupSrc.indexOf("walk(kid)") !== -1,
  "framed split walks Repeater delegates and skips a hidden empty row",
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
assert(
  shellSrc.indexOf("readonly property var flatNavPages:") !== -1 &&
    shellSrc.indexOf("LayoutJs.flattenNavPages(root.groupedPages)") !== -1,
  "flatNavPages walks groupedPages, not the unfiltered catalogue",
);
assert(
  shellSrc.indexOf("typing: searchField.activeFocus") === -1,
  "typing is not searchField.activeFocus alone",
);
assert(
  shellSrc.indexOf("item instanceof TextInput") !== -1 &&
    shellSrc.indexOf("item instanceof TextEdit") !== -1,
  "typing watches the real focus item for text fields",
);
assert(
  shellSrc.indexOf("p instanceof Popup") !== -1 &&
    shellSrc.indexOf("keysDialog.visible") !== -1 &&
    shellSrc.indexOf("sudoModeDialog.visible") !== -1 &&
    shellSrc.indexOf("errorDialog.visible") !== -1,
  "nav shortcuts stay off while a shell dialog or page-level popup is open",
);
assert(
  shellSrc.indexOf("enabled: !root.navBusy") !== -1 &&
    shellSrc.indexOf("readonly property bool navBusy:") !== -1,
  "j/k/g/G/? are gated on navBusy, not search focus alone",
);
assert(
  shellSrc.indexOf("function revealNavItem(") !== -1 &&
    shellSrc.indexOf("navFlick.contentY") !== -1,
  "placeNavHighlight scrolls the selected hub into the nav viewport",
);
const searchFieldSrc = shellSrc.slice(
  shellSrc.indexOf("id: searchField"),
  shellSrc.indexOf("id: navFlick"),
);
assert(
  searchFieldSrc.indexOf("Keys.onEscapePressed") !== -1 &&
    searchFieldSrc.indexOf("focus = false") !== -1 &&
    searchFieldSrc.indexOf('text = ""') === -1,
  "Escape on search blurs first so j/k can walk the filter",
);
const keysDialogSrc = shellSrc.slice(
  shellSrc.indexOf("id: keysDialog"),
  shellSrc.indexOf("id: sudoModeDialog"),
);
assert(keysDialogSrc.indexOf("id: keysDialog") !== -1, "shortcut sheet is a PrefsDialog");
assert(
  keysDialogSrc.indexOf("Ctrl+F") !== -1 && keysDialogSrc.indexOf("/") !== -1,
  "shortcut sheet lists Ctrl+F alongside /",
);
assert(
  keysDialogSrc.indexOf("PrefsText") !== -1,
  "shortcut sheet uses PrefsText like the other dialogs",
);
assert(shellSrc.indexOf("id: errorDialog") !== -1, "error dialog is a PrefsDialog");
assert(shellSrc.indexOf("Omarchy.copyLastError()") !== -1, "error dialog copies lastError");
assert(shellSrc.indexOf("Omarchy.clearLastError()") !== -1, "error dialog dismisses lastError");
assert(
  shellSrc.indexOf("Omarchy.askAgentAboutError()") !== -1,
  "error dialog asks the default agent",
);
const omarchySrc = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
assert(
  omarchySrc.indexOf('if (key === "laptop" && !internalPresent) return') !== -1 &&
    omarchySrc.indexOf('if (key === "docked" && !externalPresent) return') !== -1,
  "applyMonitorLayout refuses Laptop or Docked when that would leave no display on",
);
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
assert(
  omarchySrc.indexOf("function runInteractive(") !== -1 &&
    omarchySrc.indexOf("property Process interactiveProc: Process") !== -1 &&
    omarchySrc.indexOf("prefs-interactive") !== -1 &&
    omarchySrc.indexOf("kill 0") !== -1,
  "interactive tools use their own Process and kill the process group on cancel",
);
const runInteractiveStart = omarchySrc.indexOf("function runInteractive(");
const runInteractiveEnd = omarchySrc.indexOf("function commandFailureText(", runInteractiveStart);
const runInteractiveBody = omarchySrc.slice(runInteractiveStart, runInteractiveEnd);
assert(
  runInteractiveBody.indexOf("enqueueIo") === -1 && runInteractiveBody.indexOf("mutProc") === -1,
  "runInteractive does not hold the mut queue",
);
const brandingStart = omarchySrc.indexOf("function setScreensaverBranding(");
const brandingEnd = omarchySrc.indexOf("function setTimezone(", brandingStart);
const brandingBody = omarchySrc.slice(brandingStart, brandingEnd);
assert(
  brandingBody.indexOf('runInteractive(["omarchy", "branding", "screensaver"') !== -1 &&
    brandingBody.indexOf('runInteractive(["omarchy", "branding", "about"') !== -1 &&
    brandingBody.indexOf('runCommand(["omarchy", "branding", "screensaver", "reset"]') !== -1 &&
    brandingBody.indexOf('runCommand(["omarchy", "branding", "about", "reset"]') !== -1,
  "branding Choose/Edit stay off the mut queue; reset stays on it",
);
const captureStart = omarchySrc.indexOf("function captureScreenshot(");
const captureEnd = omarchySrc.indexOf("function resizeWebcam(", captureStart);
const captureBody = omarchySrc.slice(captureStart, captureEnd);
assert(
  captureBody.indexOf("runInteractive") !== -1 &&
    captureBody.indexOf('kind: "screenshot"') !== -1 &&
    captureBody.indexOf('kind: "recording"') !== -1 &&
    captureBody.indexOf('kind: "capture-text"') !== -1 &&
    captureBody.indexOf('kind: "capture-qr"') !== -1 &&
    captureBody.indexOf("interactiveProc.running = false") !== -1,
  "capture pickers stay off the mut queue; stop cancels a pending start",
);
assert(
  omarchySrc.indexOf(
    'function openBackgroundFolder() { launchDetached(["omarchy", "theme", "bg", "install"]) }',
  ) !== -1 &&
    omarchySrc.indexOf('function openAether() { launchDetached(["aether"]) }') !== -1 &&
    omarchySrc.indexOf('launchDetached(["omarchy", "launch", "terminal", "herdr"])') !== -1,
  "folder and GUI launches detach instead of holding mutProc",
);
assert(
  omarchySrc.indexOf('runInteractive(["omarchy", "theme", "bg-switcher"]') !== -1 &&
    omarchySrc.indexOf("omarchy file select") !== -1 &&
    omarchySrc.indexOf('kind: "background-file"') !== -1 &&
    omarchySrc.indexOf('kind: "theme-switcher"') !== -1,
  "unused switcher helpers stay off the mut queue if a later UI wires them",
);
assert(
  omarchySrc.indexOf("presentationMode = on") !== -1 &&
    omarchySrc.indexOf("WorkQueue.hasQueuedKey(ioQueue, job.key)") !== -1,
  "Presentation Mode updates immediately and a later write skips a stale apply",
);
assert(
  omarchySrc.indexOf("function toggleFavorite(") !== -1 &&
    omarchySrc.indexOf("atmos-favorites.json") !== -1 &&
    omarchySrc.indexOf("set-favorites.sh") !== -1,
  "Omarchy writes favorites through set-favorites.sh",
);
const favoritesPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "FavoritesPage.qml"),
  "utf8",
);
assert(
  favoritesPageSrc.indexOf("No favorites yet.") !== -1 &&
    favoritesPageSrc.indexOf('text: "Open…"') !== -1,
  "Favorites names an empty list and opens the source hub",
);
const favoritesSh = fs.readFileSync(
  path.join(__dirname, "..", "scripts", "set-favorites.sh"),
  "utf8",
);
assert(favoritesSh.indexOf("os.replace") !== -1, "set-favorites.sh replaces the file atomically");
const resetAtmosSh = fs.readFileSync(
  path.join(__dirname, "..", "scripts", "reset-atmos.sh"),
  "utf8",
);
assert(resetAtmosSh.indexOf("favorites") === -1, "Reset Atmos leaves the favorites file alone");
assert(
  themeQml.indexOf('iconStar: "star-line"') !== -1 &&
    themeQml.indexOf('iconStarOn: "star-fill"') !== -1,
  "Theme names favorite star icons",
);

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
  appearanceSrc.indexOf("available: Omarchy.textSize !== 12") === -1 &&
    appearanceSrc.indexOf('label: "Reset text size"') !== -1 &&
    appearanceSrc.indexOf('text: "Reset"') !== -1,
  "Reset text size keeps its button even at 12 pixels",
);
assert(
  /label: "Night light schedule"[\s\S]*PrefsField[\s\S]*label: "Use schedule"[\s\S]*PrefsToggle/.test(
    appearanceSrc,
  ) && appearanceSrc.indexOf('label: "Automatic night profile"') === -1,
  "Night light schedule is the times; Use schedule is the on switch",
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
const diagPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "system", "DiagnosticsPage.qml"),
  "utf8",
);
assert(
  systemSrc.indexOf('label: "Health report"') !== -1 &&
    systemSrc.indexOf('openSubpage("diagnostics")') !== -1,
  "System opens the Diagnostics child page",
);
assert(
  systemSrc.indexOf('label: "Machine"') !== -1 &&
    systemSrc.indexOf('openSubpage("machine")') !== -1 &&
    systemSrc.indexOf('id === "machine"') !== -1 &&
    systemSrc.indexOf("stack.push(machinePage)") !== -1,
  "System opens the Machine child page",
);
assert(
  systemSrc.indexOf('label: "History"') !== -1 &&
    systemSrc.indexOf('openSubpage("history")') !== -1 &&
    systemSrc.indexOf('id === "history"') !== -1 &&
    systemSrc.indexOf("stack.push(historyPage)") !== -1,
  "System opens the History child page when stack is set",
);
const historyPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "system", "HistoryPage.qml"),
  "utf8",
);
assert(
  historyPageSrc.indexOf('hubId: "system/history"') !== -1 &&
    historyPageSrc.indexOf("Omarchy.applyHeld()") !== -1 &&
    historyPageSrc.indexOf("Omarchy.discardHeld()") !== -1,
  "History page applies and discards held writes",
);
const machinePageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "system", "MachinePage.qml"),
  "utf8",
);
assert(
  machinePageSrc.indexOf("HardwareJs.batterySummary") !== -1 &&
    machinePageSrc.indexOf('hubId: "system/machine"') !== -1,
  "Machine uses Hardware batterySummary for live charge",
);
assert(
  machinePageSrc.indexOf('valueText: "present"') === -1 &&
    machinePageSrc.indexOf('"charge limited to "') === -1,
  "Machine does not treat present/limit as the primary battery readout",
);
assert(
  machinePageSrc.indexOf("batteryValue") !== -1 && machinePageSrc.indexOf('|| "unknown"') !== -1,
  "Machine keeps unknown charge as unknown",
);
assert(
  systemSrc.indexOf('label: "Crash capture"') === -1,
  "Crash capture moved off System onto Diagnostics",
);
assert(
  systemSrc.indexOf("updateFirmware") === -1 &&
    systemSrc.indexOf("firmwareConfirm") === -1 &&
    systemSrc.indexOf('label: "Firmware"') === -1,
  "System does not host fwupd firmware updates",
);
assert(
  diagPageSrc.indexOf("Drivers has the PCI device list") !== -1,
  "Diagnostics GPU copy points at Drivers",
);
assert(
  displaysPageSrc.indexOf("GPU switching is on Drivers.") !== -1 &&
    displaysPageSrc.indexOf("GPU switching is on Hardware.") === -1,
  "Displays GPU switching copy points at Drivers",
);
assert(
  diagPageSrc.indexOf('label: "Crash capture"') !== -1 &&
    diagPageSrc.indexOf('label: "Copy report"') !== -1,
  "Diagnostics has Copy report and crash capture",
);
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
const settingsSrc = fs.readFileSync(path.join(__dirname, "..", "services", "Settings.js"), "utf8");
assert(
  settingsSrc.indexOf('weatherAuto: false, weatherCoords: ""') !== -1 ||
    settingsSrc.indexOf("weatherAuto: false") !== -1,
  "weather --set drops stale coordinates",
);
assert(
  settingsSrc.indexOf('weatherLocation: "", weatherAuto: true, weatherCoords: ""') !== -1,
  "weather --clear drops coordinates and returns to auto",
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
  inputPageSrc.indexOf("checked: Omarchy.hyprWorkspaceGesture") !== -1,
  "Input checks the workspace gesture from hyprWorkspaceGesture",
);
assert(
  inputPageSrc.indexOf("enabled: !Omarchy.hyprWorkspaceGestureUnmanaged") !== -1,
  "Input disables the workspace gesture switch when the line is unmanaged",
);
const gestureRowAt = inputPageSrc.indexOf('label: "Three-finger swipe"');
assert(gestureRowAt !== -1, "Input has a Three-finger swipe row");
const gestureRow = inputPageSrc.slice(
  gestureRowAt,
  inputPageSrc.indexOf("keywords: [", gestureRowAt),
);
assert(
  gestureRow.indexOf("stays on and Atmos will not change it") !== -1,
  "Input unmanaged workspace-gesture copy says the switch stays on and Atmos will not change it",
);
assert(
  gestureRow.indexOf("stays off") === -1,
  "Three-finger swipe description does not say the switch stays off",
);
assert(
  omarchySrc.indexOf("property bool hyprWorkspaceGestureUnmanaged") !== -1 &&
    omarchySrc.indexOf("if (hyprWorkspaceGestureUnmanaged) return") !== -1,
  "Omarchy keeps unmanaged workspace gesture state and refuses to toggle it",
);
assert(
  omarchySrc.indexOf("applyHyprWorkspaceGestureFromFile") !== -1 &&
    omarchySrc.indexOf("inputLuaView.waitForJob") !== -1 &&
    omarchySrc.indexOf('job.key === "hyprInput"') !== -1,
  "Omarchy refreshes workspace gesture ownership from the file after an input write",
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
  "Omarchy validates full name with AccountsJs.isFullName",
);

// runGumJob passes the stub dir as $1 and the command after it. Without the
// shift, "$@" still carries $1 and exec is handed the directory itself:
//   prefs-job: .../scripts/stubs: Is a directory
assert(
  omarchySrc.indexOf('PATH=\\"$1:$PATH\\"; shift; exec') !== -1,
  "runGumJob shifts the stub dir off before exec",
);

// A job whose input is read to EOF hangs unless stdin is closed after the
// write. enqueueIo re-arms stdinEnabled per job, so closing is safe.
assert(
  /write\(root\.jobStdin\)[\s\S]{0,400}stdinEnabled = false/.test(omarchySrc),
  "jobProc closes stdin after writing",
);

const applySh = fs.readFileSync(path.join(__dirname, "..", "scripts", "apply-settings.sh"), "utf8");
assert(
  applySh.indexOf("while IFS= read -r key; do") === -1,
  "apply-settings.sh no longer dispatches by key",
);
assert(
  applySh.indexOf('node "$ROOT/../services/Settings.js" commands') !== -1,
  "apply-settings.sh runs Settings.js for argv",
);
assert(applySh.indexOf("for tool in jq python3 node;") !== -1, "apply-settings.sh requires node");
assert(
  settingsSrc.indexOf('["omarchy", "toggle", "bar"]') !== -1 &&
    settingsSrc.indexOf("invert: true") !== -1,
  "barVisible passes the bar-off state, not the visibility",
);
assert(
  settingsSrc.indexOf("mute-deferred") !== -1 && settingsSrc.indexOf("mute-toggle") !== -1,
  "mute is applied after volume, against the state volume leaves behind",
);
assert(
  settingsSrc.indexOf('return "vertical" + base.charAt(0).toUpperCase()') !== -1,
  "a side bar writes verticalFormat",
);
assert(
  settingsSrc.indexOf('typeof require !== "undefined"') !== -1 &&
    settingsSrc.indexOf("require.main === module") !== -1,
  "Settings CLI is gated after module.exports",
);
assert(
  !/^(const|var|let)\s+\w+\s*=\s*require\(/.test(settingsSrc),
  "Settings.js has no top-level require",
);
assert(
  settingsSrc.indexOf("workspaceGestureUnmanaged") !== -1 &&
    applySh.indexOf('status:"skipped"') !== -1,
  "apply-settings.sh reports a no-op workspace gesture as skipped after a live re-scan",
);

const exportPage = fs.readFileSync(path.join(__dirname, "..", "pages", "ExportPage.qml"), "utf8");
assert(
  exportPage.indexOf("writeProc.stdinEnabled = true") !== -1,
  "export re-arms stdin, so a second export is not an empty file",
);
assert(
  exportPage.indexOf("workspaceGestureUnmanaged: Omarchy.liveWorkspaceGestureUnmanaged()") !== -1,
  "import re-scans input.lua for an unmanaged workspace gesture",
);

assert(omarchySrc.indexOf("function applyLookPatch") === -1, "applyLookPatch is gone");
assert(omarchySrc.indexOf("function applyHyprLook") === -1, "applyHyprLook is gone");
assert(omarchySrc.indexOf("function applyHyprInput") === -1, "applyHyprInput is gone");
assert(
  omarchySrc.indexOf('!("hardware" in parsed)') === -1 &&
    omarchySrc.indexOf('!("disks" in parsed)') === -1,
  "snapshot group is not inferred from missing hardware/disks",
);
const copyStart = omarchySrc.indexOf("function copyRecord(");
assert(copyStart !== -1, "copyRecord exists");
const copyEnd = omarchySrc.indexOf("\n  function refresh(", copyStart);
assert(copyEnd !== -1, "copyRecord is followed by refresh");
const copyBody = omarchySrc.slice(copyStart, copyEnd);
assert(
  copyBody.indexOf("HardwareJs.normalize") === -1,
  "copyRecord does not re-normalize hardware",
);
assert(copyBody.indexOf("applyHyprLook") === -1, "copyRecord does not re-clamp look");
assert(copyBody.indexOf("applyHyprInput") === -1, "copyRecord does not re-clamp input");
assert(copyBody.indexOf("sanitizeDmi") === -1, "copyRecord does not re-sanitize DMI");
assert(
  copyBody.indexOf("AccountsStore.applyPatch") === -1,
  "copyRecord does not apply merged account keys",
);
assert(
  omarchySrc.indexOf("AccountsStore.applyPatch(accounts)") !== -1 &&
    omarchySrc.indexOf("SnapshotJs.accountStorePatch(parsed)") !== -1,
  "applySnapshot applies account keys from the parsed patch, not the merge",
);
const groupsJs = load("services/SnapshotGroups.js");
assert(
  copyBody.indexOf("SnapshotGroups.copyableBagKeys") !== -1,
  "copyRecord walks SnapshotGroups.copyableBagKeys",
);
assert(copyBody.indexOf("hyprGapsIn") === -1, "copyRecord does not flatten hyprLook.gapsIn");
assert(
  copyBody.indexOf("hyprSensitivity") === -1,
  "copyRecord does not flatten hyprInput.sensitivity",
);
assert(copyBody.indexOf("hyprKbLayout") === -1, "copyRecord does not flatten kbLayout");
assert(
  groupsJs.copyableBagKeys().indexOf("hyprLook") !== -1,
  "copyableBagKeys includes nested hyprLook",
);
assert(
  groupsJs.copyableBagKeys().indexOf("hyprInput") !== -1,
  "copyableBagKeys includes nested hyprInput",
);
assert(
  groupsJs.copyableBagKeys().indexOf("hostname") === -1,
  "copyableBagKeys skips AccountsStore keys",
);
assert(
  groupsJs.copyableBagKeys().indexOf("audioSink") !== -1,
  "copyableBagKeys includes derived audioSink",
);
assert(omarchySrc.indexOf("property var hyprLook") !== -1, "Omarchy bag keeps nested hyprLook");
assert(omarchySrc.indexOf("property var hyprInput") !== -1, "Omarchy bag keeps nested hyprInput");
assert(omarchySrc.indexOf("function lookState(") === -1, "lookState is gone");
assert(omarchySrc.indexOf("function setHyprGapsIn(") === -1, "setHyprGapsIn setter is gone");
assert(
  omarchySrc.indexOf("applySnapshot(JSON.stringify(job.apply))") === -1,
  "applyWritePatch does not stringify job.apply through the Emit parser",
);
assert(
  omarchySrc.indexOf("applySnapshot(job.apply)") !== -1,
  "applyWritePatch adopts job.apply as an object",
);
assert(
  omarchySrc.indexOf("tagApply: SnapshotGroups.tagApply") !== -1,
  "scriptOpts passes SnapshotGroups.tagApply",
);
assert(settingsSrc.indexOf("var APPLY_GROUP = {") === -1, "Settings.js does not keep APPLY_GROUP");

const hubsJs = load("services/Hubs.js");
const atmosSrc = fs.readFileSync(path.join(__dirname, "..", "bin", "atmos"), "utf8");
const atmosAllow = atmosSrc.split("\n").find(function (row) {
  return /\$HUB != appearance/.test(row);
});
const atmosHubIds = [];
const atmosHubRe = /\$HUB != ([A-Za-z0-9_-]+)/g;
let atmosHubMatch;
while (atmosAllow && (atmosHubMatch = atmosHubRe.exec(atmosAllow)))
  atmosHubIds.push(atmosHubMatch[1]);
assertEqual(
  atmosHubIds.join(","),
  hubsJs.hubIds().join(","),
  "bin/atmos hub ids match Hubs.hubIds()",
);
const atmosSuffixMatch = atmosSrc.match(/\[\/([a-z0-9_|]+)\]/);
const atmosSuffixes = atmosSuffixMatch ? atmosSuffixMatch[1].split("|") : [];
hubsJs.childIds().forEach(function (id) {
  const tail = id.split("/").pop();
  assert(atmosSuffixes.indexOf(tail) !== -1, "bin/atmos suffix regex includes " + tail);
});
assert(shellSrc.indexOf("HubsJs.navPages()") !== -1, "shell pages come from Hubs.navPages");
assert(
  shellSrc.indexOf('currentPage: "home"') !== -1 &&
    shellSrc.indexOf('ATMOS_PAGE") || "home"') !== -1 &&
    shellSrc.indexOf("home: homePage") !== -1,
  "shell lands on Home",
);
const homePageSrc = fs.readFileSync(path.join(__dirname, "..", "pages", "HomePage.qml"), "utf8");
assert(
  homePageSrc.indexOf('hubId: "home"') !== -1 &&
    homePageSrc.indexOf("Omarchy.liveStatsScript") !== -1 &&
    homePageSrc.indexOf("snapshot.sh") === -1 &&
    homePageSrc.indexOf("Omarchy.signalProcess") !== -1,
  "Home polls live-stats.py and signals through Omarchy",
);
assert(
  homePageSrc.indexOf("interval: 2000") !== -1 && homePageSrc.indexOf("enqueueRead") === -1,
  "Home polls locally every 2s and does not enqueue a snapshot",
);
const sparkSrc = fs.readFileSync(
  path.join(__dirname, "..", "components", "PrefsSparkline.qml"),
  "utf8",
);
assert(sparkSrc.indexOf("Canvas") !== -1, "PrefsSparkline paints on a Canvas");
assert(
  omarchySrc.indexOf("function signalProcess(") !== -1 &&
    omarchySrc.indexOf("ProcessesJs.signalArgv") !== -1 &&
    omarchySrc.indexOf('refresh: "none"') !== -1,
  "Omarchy.signalProcess runs through runCommand",
);
assert(omarchySrc.indexOf('ATMOS_PAGE") || "home"') !== -1, "Omarchy starts the session from Home");
assert(
  shellSrc.indexOf('id: "appearance", title: "Appearance"') === -1,
  "shell does not inline hub titles",
);
const searchSrc = fs.readFileSync(path.join(__dirname, "..", "services", "SearchIndex.js"), "utf8");
assert(searchSrc.indexOf("const HUBS") === -1, "SearchIndex does not own HUBS");
assert(searchSrc.indexOf("FILE_HUB") === -1, "SearchIndex does not own FILE_HUB");
assert(searchSrc.indexOf("PAGE_TITLE") === -1, "SearchIndex does not own PAGE_TITLE");
assert(searchSrc.indexOf("hubsApi()") !== -1, "SearchIndex loads Hubs via vm");
assert(
  omarchyQml.indexOf("SnapshotGroups.setSnapshotGroupForHub(HubsJs.snapshotGroupForHub)") !== -1,
  "Omarchy installs Hubs.snapshotGroupForHub on SnapshotGroups at load",
);
assert(
  omarchyQml.indexOf("var first = SnapshotGroups.snapshotGroupForHub(hub)") !== -1,
  "startSession uses SnapshotGroups.snapshotGroupForHub",
);
const completedStart = omarchyQml.indexOf("Component.onCompleted:");
const completedEnd = omarchyQml.indexOf("readonly property var watchSpecs", completedStart);
const completedBody = omarchyQml.slice(completedStart, completedEnd);
assert(
  completedBody.indexOf("setSnapshotGroupForHub") !== -1 &&
    completedBody.indexOf("setSnapshotGroupForHub") < completedBody.indexOf("startSession"),
  "Omarchy wires snapshotGroupForHub before startSession",
);
assert(
  settingsSrc.indexOf('title: "Idle and light"') !== -1,
  "export Markdown keeps Idle and light",
);

const installStart = omarchySrc.indexOf("function installTheme(");
const installEnd = omarchySrc.indexOf("function updateThemes(", installStart);
const installBody = omarchySrc.slice(installStart, installEnd);
assert(installBody.indexOf('refresh: "look"') !== -1, "installTheme refreshes look after a clone");
assert(
  installBody.indexOf("RichUi.gitThemeName") !== -1 &&
    installBody.indexOf("extraThemes") !== -1 &&
    installBody.indexOf("themes") !== -1,
  "installTheme optimistic-patches extraThemes, themes, and theme",
);
const updateStart = omarchySrc.indexOf("function updateThemes(");
const updateEnd = omarchySrc.indexOf("function removeTheme(", updateStart);
const updateBody = omarchySrc.slice(updateStart, updateEnd);
assert(updateBody.indexOf('refresh: "look"') !== -1, "updateThemes refreshes look");
const runCommandStart = omarchySrc.indexOf("function runCommand(");
const runCommandEnd = omarchySrc.indexOf("function scriptOpts(", runCommandStart);
const runCommandBody = omarchySrc.slice(runCommandStart, runCommandEnd);
const runJobStart = omarchySrc.indexOf("function runJob(");
const runJobEnd = omarchySrc.indexOf("function cancelJob(", runJobStart);
const runJobBody = omarchySrc.slice(runJobStart, runJobEnd);
assert(
  runJobBody.indexOf('opts.refresh === "none" ? "none" : "all"') === -1,
  'runJob no longer forces refresh to "none"|"all"',
);
assert(
  runJobBody.indexOf("SnapshotGroups.normalizeGroup") !== -1,
  "runJob honors snapshot groups through normalizeGroup",
);
assert(runJobBody.indexOf("opts.apply") !== -1, "runJob copies opts.apply onto the job");
assert(
  runCommandBody.indexOf("Preview.active") !== -1 &&
    runCommandBody.indexOf("bypassPreview") !== -1 &&
    runCommandBody.indexOf("holdChange") !== -1 &&
    runCommandBody.indexOf("recordChange") !== -1,
  "runCommand holds in Preview and records a real write",
);
assert(
  omarchySrc.indexOf("HistoryJs.applyHeld(held, runCommand)") !== -1,
  "applyHeld replays through History.js so every option is forwarded",
);
assert(
  runCommandBody.indexOf("opts.stdin") !== -1 &&
    runCommandBody.indexOf("opts.key") !== -1 &&
    runCommandBody.indexOf("opts.apply") !== -1 &&
    runCommandBody.indexOf("opts.refresh") !== -1 &&
    runCommandBody.indexOf("opts.sudo") !== -1,
  "runCommand still forwards key, apply, refresh, sudo, and stdin",
);
const previewQml = fs.readFileSync(path.join(__dirname, "..", "services", "Preview.qml"), "utf8");
assert(
  previewQml.indexOf("property bool active: false") !== -1 &&
    previewQml.indexOf("FileView") === -1 &&
    previewQml.indexOf("StandardPaths") === -1,
  "Preview starts off and is not persisted",
);
assert(shellSrc.indexOf("inotifywait") === -1, "inotifywait is not in shell.qml");
assert(
  omarchySrc.indexOf("syncThemeFromDiskIfStale") === -1 &&
    omarchySrc.indexOf("function syncThemeFromDisk(") === -1,
  "syncThemeFromDiskIfStale is gone",
);
const themeQmlSrc = fs.readFileSync(path.join(__dirname, "..", "services", "Theme.qml"), "utf8");
assert(themeQmlSrc.indexOf("inotifywait") !== -1, "inotifywait is in Theme.qml");
const watchStart = omarchySrc.indexOf("readonly property var watchSpecs:");
const watchEnd = omarchySrc.indexOf("function applyThemeNameFromFile", watchStart);
const watchBody = omarchySrc.slice(watchStart, watchEnd);
assert(watchBody.indexOf("extraThemesDir") === -1, "extraThemesDir is not in watchSpecs");
assert(watchBody.indexOf("packagedThemesDir") !== -1, "packaged themes dir may stay FileView");
const extraWatchStart = omarchySrc.indexOf("property Process extraThemesWatcher:");
assert(extraWatchStart !== -1, "Omarchy watches extraThemesDir with a Process");
const extraWatchBody = omarchySrc.slice(extraWatchStart, extraWatchStart + 900);
assert(
  extraWatchBody.indexOf("inotifywait") !== -1 && extraWatchBody.indexOf("extraThemesDir") !== -1,
  "extraThemesDir has inotifywait",
);
const jobProcStart = omarchySrc.indexOf("property Process jobProc:");
assert(jobProcStart !== -1, "jobProc exists");
const jobProcSrc = omarchySrc.slice(jobProcStart);
const applyAt = jobProcSrc.indexOf("root.applyWritePatch(job)");
const readAt = jobProcSrc.indexOf("enqueueRead", applyAt);
assert(
  applyAt !== -1 && readAt !== -1 && applyAt < readAt,
  "jobProc calls applyWritePatch before enqueueRead",
);

assert(
  !fs.existsSync(path.join(__dirname, "..", "scripts", "instance-lock.sh")),
  "instance-lock.sh is gone; concurrent windows serialize on disk instead",
);
assert(
  omarchySrc.indexOf("instance-lock.sh") === -1 &&
    omarchySrc.indexOf("lostInstanceLock") === -1 &&
    omarchySrc.indexOf("property Process instanceLock") === -1,
  "Omarchy does not refuse a second window",
);
assert(
  runCommandBody.indexOf("lostInstanceLock") === -1 &&
    runJobBody.indexOf("lostInstanceLock") === -1,
  "runCommand and runJob stay live in every window",
);
const enqueueIoStart = omarchySrc.indexOf("function enqueueIo(");
const enqueueIoEnd = omarchySrc.indexOf("function requestSudoMode(", enqueueIoStart);
const enqueueIoBody = omarchySrc.slice(enqueueIoStart, enqueueIoEnd);
assert(enqueueIoBody.indexOf("lostInstanceLock") === -1, "enqueueIo stays live in every window");
assert(
  shellSrc.indexOf("secondInstanceDialog") === -1 &&
    shellSrc.indexOf("lostInstanceLock") === -1 &&
    shellSrc.indexOf("Atmos is already open") === -1,
  "shell does not show an already-open refusal",
);
assert(
  testsRun.indexOf("instance-lock.sh must stay gone") !== -1 &&
    testsRun.indexOf("ATMOS_INSTANCE_LOCK") === -1,
  "tests/run pins that instance-lock.sh stays gone",
);
assert(
  testsRun.indexOf("bin/atmos opens every launch as its own instance") !== -1,
  "tests/run asserts the launcher opens a window per launch",
);
