// The pre-commit hook runs its checks on the working tree, so a partially
// staged file would be checked in one state and committed in another.
// tests/partial-index-guard stops that, and the hook has to call it.
const fs = require("fs");
const path = require("path");
const { assert, assertEqual } = require("./harness");
const { createSandbox, hostWhich } = require("./sandbox");

const guard = path.join(__dirname, "partial-index-guard");
const hook = fs.readFileSync(path.join(__dirname, "..", ".githooks", "pre-commit"), "utf8");
assert(hook.includes("./tests/partial-index-guard"), "pre-commit runs the partial-index guard");
assert(hook.includes("./tests/file-budget"), "pre-commit runs the file budget");

const box = createSandbox();
fs.symlinkSync(fs.realpathSync(hostWhich("git")), path.join(box.bin, "git"));
const env = {
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_AUTHOR_NAME: "t",
  GIT_AUTHOR_EMAIL: "t@example.com",
  GIT_COMMITTER_NAME: "t",
  GIT_COMMITTER_EMAIL: "t@example.com",
};

const repo = path.join(box.root, "repo");
fs.mkdirSync(path.join(repo, "services"), { recursive: true });
function git(...args) {
  const r = box.spawn("git", ["-C", repo, ...args], { env });
  if (r.status !== 0) throw new Error(`git ${args.join(" ")}\n${r.err}`);
  return r.out;
}
function run() {
  return box.spawn(box.bash, ["-c", 'cd "$1" && exec "$2" "$3"', "_", repo, box.bash, guard], {
    env,
  });
}
function write(rel, text) {
  fs.writeFileSync(path.join(repo, rel), text);
}

git("init", "-q");
write("services/A.js", "a\n");
write("README.md", "r\n");
git("add", "-A");
git("commit", "-q", "-m", "base");

assertEqual(run().status, 0, "a clean tree passes");

write("services/A.js", "a2\n");
git("add", "services/A.js");
assertEqual(run().status, 0, "a fully staged file passes");

write("services/A.js", "a3\n");
const split = run();
assertEqual(split.status, 1, "a file staged and then edited again fails");
assert(split.err.includes("services/A.js"), "the failure names the file");
assert(split.err.includes("--no-verify"), "the failure names the way out");

git("add", "services/A.js");
assertEqual(run().status, 0, "staging the rest clears it");

write("README.md", "r2\n");
git("add", "README.md");
write("README.md", "r3\n");
assertEqual(run().status, 0, "a partial file outside the checked paths passes");

write("services/B.js", "b\n");
assertEqual(run().status, 0, "an untracked file passes");
