const fs = require("fs");
const path = require("path");
const { load, assert, assertEqual } = require("./harness");

const accounts = load("services/Accounts.js");
assertEqual(accounts.isUsername("hallas"), true, "isUsername accepts a login");
assertEqual(accounts.parseUsername("  hallas  "), "hallas", "parseUsername trims");
assertEqual(accounts.isUsername("root"), false, "isUsername rejects root");
assertEqual(accounts.isUsername("sddm"), false, "isUsername rejects sddm");
assertEqual(accounts.isUsername("Hallas"), false, "isUsername rejects uppercase");
assertEqual(accounts.isUsername("hallas-"), false, "isUsername rejects a trailing hyphen");
assertEqual(accounts.isUsername("a".repeat(33)), false, "isUsername rejects 33 characters");
assertEqual(accounts.isUsername("_svc"), true, "isUsername accepts a leading underscore");
assertEqual(accounts.isGroupName("family"), true, "isGroupName accepts a group");
assertEqual(accounts.isGroupName("root"), false, "isGroupName rejects root");
assertEqual(accounts.isGroupName("nogroup"), false, "isGroupName rejects nogroup");
assertEqual(accounts.isGroupName("family-"), false, "isGroupName rejects a trailing hyphen");
assertEqual(accounts.isFullName(""), true, "isFullName allows empty");
assertEqual(accounts.isFullName("Christoffer Hallas"), true, "isFullName accepts a real name");
assertEqual(accounts.isFullName("Last, First"), false, "isFullName rejects a comma");
assertEqual(accounts.isFullName("bad:name"), false, "isFullName rejects a colon");
assertEqual(accounts.isFullName("-flag"), false, "isFullName rejects a leading hyphen");
assertEqual(accounts.parseFullName("  Ada  "), "Ada", "parseFullName trims");
assertEqual(accounts.parseFullName("Last, First"), "", "parseFullName rejects a comma");
assertEqual(
  accounts.gecosFullName("Ada Lovelace,Office,1"),
  "Ada Lovelace",
  "gecosFullName cuts at comma",
);
assertEqual(accounts.gecosFullName("Ada"), "Ada", "gecosFullName keeps a bare name");
assertEqual(accounts.isHumanUid(1000), true, "isHumanUid accepts 1000");
assertEqual(accounts.isHumanUid(999), false, "isHumanUid rejects 999");
assertEqual(accounts.isHumanUid(65534), false, "isHumanUid rejects nobody");
assertEqual(accounts.isRemovableGid(1000, "family"), true, "isRemovableGid accepts a user group");
assertEqual(accounts.isRemovableGid(1000, "wheel"), false, "isRemovableGid rejects wheel");
assertEqual(accounts.isRemovableGid(982, "docker"), false, "isRemovableGid rejects docker");
assertEqual(accounts.isRemovableGid(0, "root"), false, "isRemovableGid rejects gid 0");

const passwdText = [
  "root:x:0:0:root:/root:/bin/bash",
  "sddm:x:979:979:Simple Desktop Display Manager:/var/lib/sddm:/usr/bin/nologin",
  "hallas:x:1000:1000:Christoffer Hallas,Office:/home/hallas:/bin/bash",
  "alice:x:1001:1001:Alice:/home/alice:/bin/bash",
].join("\n");
const groupText = [
  "root:x:0:root",
  "wheel:x:998:hallas",
  "docker:x:982:hallas",
  "hallas:x:1000:",
  "alice:x:1001:",
  "family:x:1002:hallas,alice",
  "nobody:x:65534:",
].join("\n");
const parsedUsers = accounts.parsePasswd(passwdText, "hallas");
assertEqual(parsedUsers.length, 2, "parsePasswd keeps human logins");
assertEqual(parsedUsers[0].name, "hallas", "parsePasswd keeps the session user");
assertEqual(parsedUsers[0].fullName, "Christoffer Hallas", "parsePasswd reads GECOS");
assertEqual(parsedUsers[0].current, true, "parsePasswd marks the session user");
assertEqual(parsedUsers[1].name, "alice", "parsePasswd keeps another human login");
assert(
  parsedUsers.every(function (row) {
    return row.name !== "root" && row.name !== "sddm";
  }),
  "parsePasswd drops system accounts",
);
const withWheel = accounts.attachWheel(parsedUsers, accounts.parseGroup(groupText));
assertEqual(withWheel[0].wheel, true, "attachWheel marks hallas");
assertEqual(withWheel[1].wheel, false, "attachWheel leaves alice off wheel");
const shown = accounts.visibleGroups(accounts.parseGroup(groupText), withWheel);
const shownNames = shown.map(function (g) {
  return g.name;
});
assert(
  shownNames.indexOf("wheel") !== -1 && shownNames.indexOf("docker") !== -1,
  "visibleGroups keeps wheel and docker",
);
assert(shownNames.indexOf("family") !== -1, "visibleGroups keeps a human group");
assert(shownNames.indexOf("hallas") === -1, "visibleGroups hides a private user group");
assert(shownNames.indexOf("alice") === -1, "visibleGroups hides alice's private group");
assert(shownNames.indexOf("nobody") === -1, "visibleGroups hides nobody");
assertEqual(
  accounts.groupHasMember({ members: ["hallas", "alice"] }, "alice"),
  true,
  "groupHasMember finds a member",
);
assertEqual(
  accounts.groupHasMember({ members: ["hallas"] }, "alice"),
  false,
  "groupHasMember misses a stranger",
);
const inventory = accounts.parseAccounts({
  currentUser: "hallas",
  avatarPath: "/home/hallas/.face.icon",
  passwd: passwdText,
  group: groupText,
});
assertEqual(inventory.currentUser, "hallas", "parseAccounts keeps currentUser");
assertEqual(inventory.users.length, 2, "parseAccounts returns two humans");
assertEqual(inventory.users[0].wheel, true, "parseAccounts attaches wheel");
assert(
  inventory.groups.every(function (g) {
    return g.name !== "hallas" && g.name !== "alice";
  }),
  "parseAccounts omits private groups",
);
assertEqual(
  accounts.pickAvatarPath("/home/hallas", "hallas", function (path) {
    return path === "/var/lib/AccountsService/icons/hallas";
  }),
  "/var/lib/AccountsService/icons/hallas",
  "pickAvatarPath prefers AccountsService",
);
assertEqual(
  accounts.pickAvatarPath("/home/hallas", "hallas", function (path) {
    return path === "/home/hallas/.face";
  }),
  "/home/hallas/.face",
  "pickAvatarPath falls through to ~/.face",
);
assertEqual(
  accounts.pickAvatarPath("/home/hallas", "hallas", function () {
    return false;
  }),
  "",
  "pickAvatarPath is empty when nothing exists",
);
assertEqual(
  accounts.profileTitle("Chris Hallas", "hallas"),
  "Chris Hallas",
  "profileTitle prefers full name",
);
assertEqual(accounts.profileTitle("", "hallas"), "hallas", "profileTitle falls back to login");
assertEqual(accounts.profileTitle("", ""), "Account", "profileTitle empty is Account");
assertEqual(
  accounts.profileHost("hallas", "framework"),
  "hallas@framework",
  "profileHost joins login and hostname",
);
assertEqual(accounts.profileHost("hallas", ""), "hallas", "profileHost falls back to login");
assertEqual(accounts.profileHost("", "framework"), "", "profileHost empty without a login");
assertEqual(
  accounts.profileHost("hallas", "bad host"),
  "hallas",
  "profileHost drops a hostname with a space",
);
assert(
  accounts.faceRowDescription("").indexOf("greeter") !== -1,
  "faceRowDescription mentions the greeter when empty",
);
assert(
  accounts.faceRowDescription("/home/hallas/.face.icon").indexOf("/home/hallas/.face.icon") === 0,
  "faceRowDescription starts with the path",
);
assert(
  accounts.faceRowDescription("/home/hallas/.face.icon").indexOf("greeter") !== -1,
  "faceRowDescription mentions the greeter when a face is set",
);
const accountsPageSrc = fs.readFileSync(
  path.join(__dirname, "..", "pages", "AccountsPage.qml"),
  "utf8",
);
assert(
  accountsPageSrc.indexOf("AccountsJs.faceRowDescription(Omarchy.avatarPath)") !== -1,
  "Accounts face row uses faceRowDescription",
);
const shellProfileSrc = fs.readFileSync(path.join(__dirname, "..", "shell.qml"), "utf8");
assert(shellProfileSrc.indexOf("id: profileHostSlot") !== -1, "sidebar reserves a host line slot");
const seeded = accounts.seedFromDisk({
  currentUser: "hallas",
  hostname: "hallas\n",
  passwd: passwdText,
  group: groupText,
  home: "/home/hallas",
  exists: function (path) {
    return path === "/home/hallas/.face.icon";
  },
});
assertEqual(seeded.currentUser, "hallas", "seedFromDisk keeps the session user");
assertEqual(seeded.hostname, "hallas", "seedFromDisk trims hostname");
assertEqual(seeded.fullName, "Christoffer Hallas", "seedFromDisk reads GECOS");
assertEqual(seeded.avatarPath, "/home/hallas/.face.icon", "seedFromDisk picks ~/.face.icon");
const patched = accounts.applyAccountPatch(
  { hostname: "old", fullName: "Ada", currentUser: "ada", avatarPath: "", users: [], groups: [] },
  { hostname: "hallas", avatarPath: "/home/hallas/.face.icon" },
);
assertEqual(patched.hostname, "hallas", "applyAccountPatch updates hostname");
assertEqual(patched.fullName, "Ada", "applyAccountPatch keeps fullName when omitted");
assertEqual(patched.avatarPath, "/home/hallas/.face.icon", "applyAccountPatch updates avatarPath");
assertEqual(
  accounts.applyAccountPatch({}, { hostname: "bad host" }).hostname,
  "",
  "applyAccountPatch drops a bad hostname",
);
const omarchySrc = fs.readFileSync(path.join(__dirname, "..", "services", "Omarchy.qml"), "utf8");
assert(
  omarchySrc.indexOf("AccountsStore.hostname") !== -1,
  "Omarchy forwards hostname from AccountsStore",
);
assert(
  omarchySrc.indexOf('passwdFile, group: "rest"') === -1,
  "Omarchy no longer watches passwd for a rest snapshot",
);
const storeSrc = fs.readFileSync(
  path.join(__dirname, "..", "services", "AccountsStore.qml"),
  "utf8",
);
assert(
  storeSrc.indexOf("function reloadFromDisk()") !== -1,
  "AccountsStore reloads from passwd and hostname",
);
