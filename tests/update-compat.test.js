const fs = require("fs");
const path = require("path");
const { assertEqual } = require("./harness");

// Installs older than 0.2.1 update by running their own updater, which copies
// `scripts/` out of the new tree and fails when it is missing. Keep the
// directory, and keep it free of code: Atmos has no helper scripts.
const dir = path.join(__dirname, "..", "scripts");
assertEqual(fs.existsSync(dir), true, "scripts/ stays for the updater in older installs");
assertEqual(
  fs.readdirSync(dir).sort().join(","),
  "README.md,hypr-sentinel.py",
  "scripts/ holds its README and the one shim, no helper scripts",
);
const shim = fs.readFileSync(path.join(dir, "hypr-sentinel.py"), "utf8");
assertEqual(
  shim.indexOf('"sentinel"') !== -1 && shim.split("\n").length < 20,
  true,
  "the shim only hands the call to ratmos sentinel",
);
console.log("ok - scripts/ is a placeholder for older updaters");
