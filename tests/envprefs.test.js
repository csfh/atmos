const { load, assert, assertEqual } = require("./harness");

const env = load("services/EnvPrefs.js");
assertEqual(env.sanitizeKey("PATH"), "", "sanitizeKey refuses PATH");
assertEqual(env.sanitizeKey("FOO_BAR"), "FOO_BAR", "sanitizeKey keeps FOO_BAR");
assertEqual(
  env.sanitizePathPrepend("/opt/bin:/tmp/../etc"),
  "/opt/bin",
  "sanitizePathPrepend drops ..",
);
assertEqual(env.sanitizeShell("/bin/zsh"), "/bin/zsh", "sanitizeShell keeps zsh");
assertEqual(env.sanitizeShell("zsh"), "", "sanitizeShell requires an absolute path");
const text = env.serialize({
  pathPrepend: "/opt/bin",
  vars: [
    { key: "EDITOR", value: "nvim" },
    { key: "PATH", value: "nope" },
  ],
});
assert(text.indexOf("# atmos:env begin") === 0, "serialize starts with sentinel");
assert(text.indexOf("PATH=/opt/bin:$PATH") !== -1, "serialize prepends PATH");
assert(text.indexOf("EDITOR=nvim") !== -1, "serialize writes EDITOR");
assert(text.indexOf("PATH=nope") === -1, "serialize drops a PATH var");
const parsed = env.parseFile(text);
assertEqual(parsed.pathPrepend, "/opt/bin", "parseFile reads path prepend");
assertEqual(parsed.vars[0].key, "EDITOR", "parseFile reads EDITOR");
assertEqual(
  env.detected({ sessionType: "wayland\nbad" }).sessionType,
  "wayland bad",
  "detected strips a newline",
);
