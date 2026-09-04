const { load, assertEqual } = require("./harness");

const software = load("services/Software.js");
assertEqual(software.lookup("steam").wipe, true, "catalog marks Steam as a wipe remove");
assertEqual(
  software.presentIn(software.lookup("firefox"), { browsers: { firefox: true } }),
  true,
  "presentIn reads a browser bag",
);
assertEqual(
  software.presentIn(software.lookup("vscode"), { editors: { code: true } }),
  true,
  "presentIn maps vscode to editors.code",
);
assertEqual(software.isDevEnv("python"), true, "isDevEnv accepts python");
assertEqual(software.isDockerDb("PostgreSQL"), true, "isDockerDb accepts PostgreSQL");
assertEqual(software.groupItems("gaming").length > 3, true, "groupItems lists gaming installers");
assertEqual(software.lookup("nope"), null, "lookup misses an unknown id");
assertEqual(
  software.presentIn(software.lookup("zed"), { editors: { zeditor: true } }),
  true,
  "presentIn maps zed to editors.zeditor",
);
assertEqual(
  software.presentIn(software.lookup("firefox"), { browsers: {} }),
  false,
  "presentIn is false when the bag flag is missing",
);
assertEqual(software.isDevEnv("cobol"), false, "isDevEnv rejects an unknown env");
assertEqual(software.isDockerDb("sqlite"), false, "isDockerDb rejects an unknown db");
assertEqual(software.groupItems("browsers").length, 6, "groupItems lists the browser installers");
assertEqual(
  software.presentIn(software.lookup("1password"), { services: { onepassword: true } }),
  true,
  "presentIn maps 1password to services.onepassword",
);
assertEqual(software.groupItems("nope").length, 0, "groupItems empty for an unknown group");
assertEqual(
  software.presentIn(null, { browsers: { firefox: true } }),
  false,
  "presentIn is false without an item",
);
