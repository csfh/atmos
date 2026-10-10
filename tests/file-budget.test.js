// tests/file-budget is a ratchet: new files stay under the cap, listed files
// only shrink, and the baseline never goes stale.
const fs = require("fs");
const path = require("path");
const { assert, assertEqual } = require("./harness");
const { createSandbox } = require("./sandbox");

const script = path.join(__dirname, "file-budget");
const box = createSandbox();
let n = 0;

function lines(count) {
  return Array.from({ length: count }, (_, i) => `// ${i}`).join("\n") + "\n";
}

// A fixture tree with one file under budget and one listed over it.
function tree(baseline = "900 services/Big.js\n") {
  const root = path.join(box.root, `tree-${n++}`);
  fs.mkdirSync(path.join(root, "services"), { recursive: true });
  fs.mkdirSync(path.join(root, "tests"), { recursive: true });
  fs.writeFileSync(path.join(root, "services", "Small.js"), lines(10));
  fs.writeFileSync(path.join(root, "services", "Big.js"), lines(900));
  if (baseline !== null)
    fs.writeFileSync(path.join(root, "tests", "file-budget.baseline"), baseline);
  return root;
}

function check(root, ...extra) {
  return box.spawn(process.execPath, [script, "--root", root, ...extra]);
}

const clean = check(tree());
assertEqual(clean.status, 0, "a listed file at its baseline passes");

const fresh = tree();
fs.writeFileSync(path.join(fresh, "services", "New.js"), lines(801));
const over = check(fresh);
assertEqual(over.status, 1, "a new file over the cap fails");
assert(
  over.out.includes("services/New.js is 801 lines"),
  "the failure names the file and its length",
);

const edge = tree();
fs.writeFileSync(path.join(edge, "services", "Edge.js"), lines(800));
assertEqual(check(edge).status, 0, "a file exactly at the cap passes");

const grown = tree();
fs.writeFileSync(path.join(grown, "services", "Big.js"), lines(901));
const grew = check(grown);
assertEqual(grew.status, 1, "a listed file that grows fails");
assert(grew.out.includes("grew to 901 lines from 900"), "the failure says how far it grew");
const refused = check(grown, "--update");
assertEqual(refused.status, 1, "--update refuses to raise the baseline");
assertEqual(
  fs.readFileSync(path.join(grown, "tests", "file-budget.baseline"), "utf8"),
  "900 services/Big.js\n",
  "a refused --update leaves the baseline alone",
);

const shrunk = tree();
fs.writeFileSync(path.join(shrunk, "services", "Big.js"), lines(850));
const shrank = check(shrunk);
assertEqual(shrank.status, 1, "a listed file that shrank fails until the baseline follows");
assert(shrank.out.includes("lower the baseline"), "the failure says to lower the baseline");
assertEqual(check(shrunk, "--update").status, 0, "--update lowers the baseline");
assert(
  fs
    .readFileSync(path.join(shrunk, "tests", "file-budget.baseline"), "utf8")
    .includes("850 services/Big.js"),
  "the baseline now holds the new length",
);
assertEqual(check(shrunk).status, 0, "the check passes after --update");

const split = tree();
fs.writeFileSync(path.join(split, "services", "Big.js"), lines(300));
const back = check(split);
assertEqual(back.status, 1, "a listed file back under budget fails until it leaves the list");
assert(back.out.includes("back under budget"), "the failure says it is back under budget");
assertEqual(check(split, "--update").status, 0, "--update drops it");
assert(
  !fs.readFileSync(path.join(split, "tests", "file-budget.baseline"), "utf8").includes("Big.js"),
  "the baseline no longer lists it",
);

const gone = tree();
fs.rmSync(path.join(gone, "services", "Big.js"));
const missing = check(gone);
assertEqual(missing.status, 1, "a listed file that is gone fails");
assert(missing.out.includes("gone or renamed"), "the failure says the entry is stale");

const noBaseline = check(tree(null));
assertEqual(noBaseline.status, 1, "a missing baseline fails");
assert(
  noBaseline.out.includes("file-budget.baseline is missing"),
  "the failure names the baseline",
);

const broken = check(tree("lots services/Big.js\n"));
assert(broken.status !== 0, "an unreadable baseline line fails");

const withBinary = tree();
fs.mkdirSync(path.join(withBinary, "bin"));
fs.writeFileSync(path.join(withBinary, "bin", "ratmos"), Buffer.from("\0\n".repeat(900)));
assertEqual(check(withBinary).status, 0, "a binary under bin/ is not counted as source");
