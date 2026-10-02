const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const guard = load("services/Guard.js");

const t0 = 1_000_000;
const scale = guard.open(
  null,
  { id: "monitorScale", revert: { kind: "monitorScale", scale: "1" } },
  t0,
);
assert(scale.armed, "a guarded scale change arms the bar");
assertEqual(scale.state.deadline, t0 + guard.WINDOW_MS, "the window is 12 seconds");
assertEqual(scale.state.reverts.length, 1, "one revert is stored");
assertEqual(scale.state.reverts[0].revert.scale, "1", "the revert target is the previous scale");

const again = guard.open(
  scale.state,
  { id: "monitorScale", revert: { kind: "monitorScale", scale: "2" } },
  t0 + 3000,
);
assertEqual(
  again.state.reverts[0].revert.scale,
  "1",
  "a second scale edit keeps the original baseline",
);
assertEqual(
  again.state.deadline,
  t0 + 3000 + guard.WINDOW_MS,
  "a second edit restarts the 12 seconds",
);

const both = guard.open(
  again.state,
  { id: "monitorRules", revert: { kind: "monitorRules", rules: [{ output: "eDP-1" }] } },
  t0 + 4000,
);
assertEqual(both.state.reverts.length, 2, "a different write joins the same bar");
assertEqual(both.state.reverts[0].revert.scale, "1", "the scale baseline stays the first value");

const early = guard.tick(both.state, both.state.deadline - 1);
assertEqual(early.fire.length, 0, "the bar stays open until the deadline");
assert(early.state.open, "a tick before the deadline leaves the guard open");

const expired = guard.tick(both.state, both.state.deadline);
assertEqual(expired.state.open, false, "expiry closes the guard");
assertEqual(expired.fire.length, 2, "expiry returns every revert");
assertEqual(expired.fire[0].kind, "monitorRules", "the later write reverts first");
assertEqual(expired.fire[1].scale, "1", "the earlier write reverts second");

const kept = guard.keep(both.state);
assertEqual(kept.open, false, "Keep drops the saved baseline");
assertEqual(kept.reverts.length, 0, "Keep stores nothing to write back");

const manual = guard.revert(both.state);
assertEqual(manual.fire[1].kind, "monitorScale", "Revert returns the same order as expiry");
assertEqual(manual.state.open, false, "Revert closes the guard");

assertEqual(guard.secondsLeft(both.state, t0 + 4000), 12, "a fresh window reads as 12 seconds");
assertEqual(
  guard.secondsLeft(both.state, both.state.deadline - 1),
  1,
  "the last instant still reads as 1 second",
);
assertEqual(
  guard.secondsLeft(both.state, both.state.deadline),
  0,
  "the deadline reads as 0 seconds",
);

const ignored = guard.open(null, { id: "monitorScale" }, t0);
assertEqual(ignored.armed, false, "a change without a revert payload does not arm");

const shell = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");
const escapeAt = shell.indexOf('sequences: ["Escape"]');
const escapeBody = shell.slice(escapeAt, escapeAt + 1200);
assert(escapeAt !== -1, "the window has an Escape shortcut");
assert(
  escapeBody.indexOf("Omarchy.revertGuard()") !== -1 &&
    escapeBody.indexOf("Omarchy.guardOpen") !== -1 &&
    escapeBody.indexOf("revertGuard()") < escapeBody.indexOf("pageStack.pop()"),
  "Escape reverts a pending guard before it leaves the page",
);
assert(shell.indexOf("PrefsGuardBar") !== -1, "the prefs window hosts the keep-or-revert bar");

const omarchy = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
assert(omarchy.indexOf('import "Guard.js" as GuardJs') !== -1, "Omarchy owns the guard state");
assert(omarchy.indexOf("function rememberGuard(") !== -1, "a queued job is what arms the bar");
assert(
  omarchy.indexOf("opts.bypassPreview = true") !== -1,
  "a revert write is not held by Preview",
);
assert(omarchy.indexOf('id: "monitorRules"') !== -1, "monitor rule writes are guarded");
assert(omarchy.indexOf('id: "monitorScale"') !== -1, "focused scale writes are guarded");
assert(omarchy.indexOf('id: "keyboardLayout"') !== -1, "the system layout write is guarded");
const hyprPrefsSrc = fs.readFileSync(
  path.join(__dirname, "..", "services", "HyprPrefs.js"),
  "utf8",
);
assert(
  hyprPrefsSrc.indexOf('id: "kbOverride"') !== -1 &&
    omarchy.indexOf("HyprPrefs.kbRunOptions") !== -1,
  "the Hyprland layout override is guarded",
);
assert(omarchy.indexOf('id: "touchpad"') !== -1, "turning the touchpad off is guarded");
assert(
  omarchy.indexOf("dropWriteKey") !== -1,
  "a touchpad revert drops a toggle that has not run yet",
);
