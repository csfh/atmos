const { load, assert, assertEqual } = require("./harness");

const profiles = load("services/Profiles.js");
const settings = load("services/Settings.js");
assertEqual(profiles.builtins().length, 3, "three built-in profiles");
assertEqual(profiles.byId("gaming").title, "Gaming", "byId finds Gaming");
assertEqual(profiles.sanitizeUserName("My Desk"), "", "sanitizeUserName rejects a space");
assertEqual(profiles.sanitizeUserName("desk-1"), "desk-1", "sanitizeUserName keeps desk-1");
const changes = profiles.changesFor(profiles.byId("gaming"));
assert(
  changes.some(function (row) {
    return row.key === "hyprLook.animations" && row.value === false;
  }),
  "gaming disables animations",
);
const snap = {
  powerProfile: "balanced",
  stayAwake: false,
  doNotDisturb: false,
  screensaverEnabled: true,
  hyprLook: { animations: true, allowTearing: false },
};
const cmds = settings.planCommands(changes, snap, { root: pathJoin() });
function pathJoin() {
  const path = require("path");
  return path.join(__dirname, "..");
}
assert(
  cmds.some(function (cmd) {
    return cmd && cmd.argv && cmd.argv.join(" ").indexOf("set-hypr-look.sh") !== -1;
  }),
  "profile apply uses the look writer",
);
assert(
  cmds.some(function (cmd) {
    return cmd && cmd.key === "stayAwake";
  }),
  "profile apply includes stayAwake",
);
assert(
  cmds.some(function (cmd) {
    return cmd && cmd.key === "doNotDisturb";
  }),
  "profile apply includes doNotDisturb",
);
