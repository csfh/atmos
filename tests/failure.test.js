const { load, assert, assertEqual } = require("./harness");

const f = load("services/Failure.js");

assertEqual(f.errorText("boom"), "boom", "a string error stays as it is");
assertEqual(
  f.errorText({ code: "denied", message: "permission denied", context: "/etc/hosts" }),
  "/etc/hosts: permission denied",
  "context leads the message",
);
assertEqual(
  f.errorText({ message: "/etc/hosts: permission denied", context: "/etc/hosts" }),
  "/etc/hosts: permission denied",
  "context is not repeated",
);
assertEqual(f.errorText(null), "", "no error is empty text");

const envelope = JSON.stringify({
  ok: false,
  error: { code: "bad_request", message: "unknown op x", context: "op" },
  result: null,
});
assertEqual(f.envelopeFailure(envelope), "unknown op x", "a failed envelope reads as its message");
assertEqual(f.envelopeFailure('{"ok":true,"result":1}'), "", "a good envelope is not a failure");
assertEqual(f.envelopeFailure("plain text"), "", "plain text is not an envelope");
assertEqual(f.envelopeFailure("{not json"), "", "broken JSON is not an envelope");
assertEqual(f.commandText("", envelope), "unknown op x", "stdout envelope becomes the banner");
assertEqual(f.commandText("bad", ""), "bad", "stderr alone passes through");
assertEqual(f.commandText("a", "b"), "a\nb", "stderr and stdout both show");
assertEqual(f.commandText("same", "same"), "same", "identical streams show once");

// apply summary line
const okLine = '@@ratmos-apply@@ {"ok":true,"code":0,"message":"","warnings":[]}';
const badLine =
  '@@ratmos-apply@@ {"ok":false,"code":3,"message":"disk full","warnings":["warn: wayland.x"]}';
const noiseLine =
  '@@ratmos-apply@@ {"ok":false,"code":1,"message":"","warnings":["warn: wayland.x"]}';
const emptyLine = '@@ratmos-apply@@ {"ok":false,"code":1,"message":"","warnings":[]}';
assertEqual(f.applyLine(badLine).code, 3, "a summary line parses");
assert(f.applyLine("plain output") === null, "any other line is not a summary");
assert(f.applyLine("@@ratmos-apply@@ {broken") === null, "a broken summary is not a summary");
assert(f.applyLine(undefined) === null, "no line is not a summary");

const split = f.splitApply("one\ntwo\n" + badLine + "\n");
assertEqual(split.result.message, "disk full", "the summary is lifted out of stdout");
assertEqual(split.text, "one\ntwo\n", "the rest of stdout is kept");
assert(f.splitApply("just output").result === null, "no summary means none");

assertEqual(f.applyBanner(f.applyLine(okLine), 0, ""), "", "a good run has no banner");
assertEqual(
  f.applyBanner(f.applyLine(badLine), 3, "x"),
  "disk full",
  "a failed run shows its message",
);
assertEqual(
  f.applyBanner(f.applyLine(noiseLine), 1, "warn: wayland.x"),
  "",
  "noise alone is not a banner",
);
assertEqual(
  f.applyBanner(f.applyLine(emptyLine), 1, ""),
  "Command failed",
  "a silent failure still says so",
);
assertEqual(f.applyBanner(null, 0, ""), "", "no summary and exit 0 is fine");
assertEqual(f.applyBanner(null, 2, "  oops "), "oops", "no summary falls back to the plain text");
assertEqual(
  f.applyBanner(null, 2, ""),
  "Command failed",
  "no summary and nothing printed is a failure",
);
