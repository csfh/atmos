// A failed `hyprctl monitors all -j` is not "no monitors". The look snapshot
// leaves the monitors key out (and the three flags derived from it), so the
// window keeps the list it has. A real empty list still comes through.
const { assert, assertEqual } = require("./harness");
const { createSandbox, atmosdBinary } = require("./sandbox");

const MONITOR = JSON.stringify([
  {
    id: 0,
    name: "DP-1",
    width: 1920,
    height: 1080,
    refreshRate: 60.0,
    x: 0,
    y: 0,
    scale: 1.0,
    focused: true,
    disabled: false,
    availableModes: ["1920x1080@60.00Hz"],
  },
]);

function lookSnapshot(monitorsReply) {
  const box = createSandbox({
    fakes: {
      hyprctl: [{ argv: ["monitors", "all", "-j"], ...monitorsReply }, { status: 1 }],
      omarchy: [{ status: 1 }],
    },
  });
  try {
    const run = box.spawn(atmosdBinary(), ["request"], {
      input: JSON.stringify({ op: "settings.snapshot", group: "look" }),
    });
    assertEqual(run.status, 0, `snapshot exits 0: ${run.err}`);
    const reply = JSON.parse(run.out);
    assertEqual(reply.ok, true, "the snapshot still succeeds");
    return { result: reply.result, err: run.err };
  } finally {
    box.cleanup();
  }
}

const derived = ["internalPresent", "internalEnabled", "mirroring"];

const garbage = lookSnapshot({ stdout: "not json", stderr: "socket error", status: 1 });
assert(!("monitors" in garbage.result), "garbage output leaves monitors out");
for (const key of derived) assert(!(key in garbage.result), `garbage output leaves ${key} out`);
assert(
  garbage.err.includes("keeping the last one") && garbage.err.includes("socket error"),
  "the reason reaches stderr",
  garbage.err,
);
console.log("ok - a failed hyprctl leaves the monitor list alone and says why");

const empty = lookSnapshot({ stdout: "", status: 0 });
assert(!("monitors" in empty.result), "empty output is a failure too, not an empty list");

const none = lookSnapshot({ stdout: "[]", status: 0 });
assertEqual(JSON.stringify(none.result.monitors), "[]", "a real empty list is reported as empty");
for (const key of derived) assert(key in none.result, `an empty list still carries ${key}`);
console.log("ok - a genuinely empty monitor list still reaches the window");

const one = lookSnapshot({ stdout: MONITOR, status: 0 });
assertEqual(one.result.monitors.length, 1, "one monitor comes through");
assertEqual(one.result.monitors[0].name, "DP-1", "with its name");
console.log("ok - a monitor list comes through unchanged");
