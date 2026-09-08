const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const toggle = load("services/PopupToggle.js");

function fakePopup(opened) {
  return {
    opened: !!opened,
    open: function () {
      this.opened = true;
    },
    close: function () {
      this.opened = false;
    },
  };
}

const closed = fakePopup(false);
toggle.toggle(closed);
assertEqual(closed.opened, true, "toggle opens a closed popup");

const open = fakePopup(true);
toggle.toggle(open);
assertEqual(open.opened, false, "toggle closes an open popup");

const disabledClosed = fakePopup(false);
toggle.toggle(disabledClosed, false);
assertEqual(disabledClosed.opened, false, "toggle leaves a disabled closed popup closed");

const disabledOpen = fakePopup(true);
toggle.toggle(disabledOpen, false);
assertEqual(disabledOpen.opened, true, "toggle leaves a disabled open popup open");

const fromOpen = fakePopup(true);
toggle.clickTrigger(fromOpen, true);
assertEqual(fromOpen.opened, false, "clickTrigger closes when the press started open");

const fromClosed = fakePopup(false);
toggle.clickTrigger(fromClosed, false);
assertEqual(fromClosed.opened, true, "clickTrigger opens when the press started closed");

const alreadyClosed = fakePopup(false);
toggle.clickTrigger(alreadyClosed, true);
assertEqual(
  alreadyClosed.opened,
  false,
  "clickTrigger does not reopen after CloseOnReleaseOutside",
);

const componentsDir = path.join(__dirname, "..", "components");
const openOnly = [];
fs.readdirSync(componentsDir).forEach(function (name) {
  if (!name.endsWith(".qml")) return;
  const src = fs.readFileSync(path.join(componentsDir, name), "utf8");
  if (/onClicked:\s*(?:function\s*\([^)]*\)\s*\{\s*)?popup\.open\(\)/.test(src))
    openOnly.push(name + " onClicked opens only");
  if (/Keys\.onReturnPressed:.*popup\.open\(\)/.test(src))
    openOnly.push(name + " Return opens only");
  if (/Keys\.onSpacePressed:.*popup\.open\(\)/.test(src)) openOnly.push(name + " Space opens only");
  if (/Accessible\.onPressAction:.*popup\.open\(\)/.test(src))
    openOnly.push(name + " Accessible press opens only");
});
assert(openOnly.length === 0, "trigger popups toggle instead of only opening", openOnly.join("\n"));

["PrefsMenu.qml", "PrefsSelect.qml", "PrefsHelp.qml"].forEach(function (name) {
  const src = fs.readFileSync(path.join(componentsDir, name), "utf8");
  assert(src.indexOf("PopupToggle.js") !== -1, name + " uses PopupToggle");
  assert(
    src.indexOf("CloseOnReleaseOutside") !== -1 && src.indexOf("CloseOnPressOutside") === -1,
    name + " closes on release outside so a second click can toggle",
  );
  assert(src.indexOf("openedAtPress") !== -1, name + " remembers open state on press");
});
