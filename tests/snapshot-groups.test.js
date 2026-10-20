const { load, assert, assertEqual } = require("./harness");

const groups = load("services/SnapshotGroups.js");

// Which key is in which topic is the backend's (docs/adr/0011); the window only
// knows the topics it holds and what to call a request to read some again.
assertEqual(groups.TOPICS.length, 5, "the window holds five settings topics");
assert(
  groups.TOPICS.every(function (name) {
    return name.indexOf("settings.") === 0;
  }),
  "every topic is a settings topic",
);
assert(groups.snapshotGroupForHub === undefined, "no hub decides what is read first");
assert(groups.emitKeys === undefined && groups.groupForKey === undefined, "no group owns a key");

assertEqual(groups.normalizeGroup("look"), "look", "look is a group");
assertEqual(groups.normalizeGroup("nope"), "all", "an unknown group is all");
assertEqual(groups.topicsForGroups(["look"]).join(","), "settings.look", "look is one topic");
assertEqual(groups.topicsForGroups("network").join(","), "settings.network", "so is network");
assertEqual(
  groups.topicsForGroups(["disks"]).join(","),
  "settings.host,disks",
  "disks is the host topic and the disks inventory",
);
assertEqual(groups.topicsForGroups(["all"]).length, 10, "all is every topic and every inventory");
assertEqual(
  groups.topicsForGroups(["look", "rest"]).length,
  5,
  "a topic asked for twice is asked for once",
);
assert(
  groups.topicsForGroups(["rest"]).indexOf("settings.look") === -1,
  "rest leaves the look topic alone",
);
assert(
  groups.allowedKey("", "theme") &&
    !groups.allowedKey("", "group") &&
    !groups.allowedKey("", "_failed"),
  "a pushed topic is taken whole, apart from the notes the backend adds",
);

const bag = groups.copyableBagKeys();
assert(bag.indexOf("monitors") !== -1 && bag.indexOf("netSsid") !== -1, "the bag has the keys");
assert(bag.indexOf("hostname") === -1, "account keys are the accounts store's");
assert(bag.indexOf("audioSink") !== -1, "derived keys are adopted onto the bag too");
assertEqual(new Set(groups.KEYS).size, groups.KEYS.length, "no key is listed twice");
