//! What the QML window's search reads: the real `atmosd` reads the QML pages as `search.query`
//! does. Without a built `atmosd` or `node` a test says so and stops.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use atmos_ui::client::locate;
use serde_json::Value;

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

pub fn atmosd() -> Option<PathBuf> {
    let debug = repo().join("backend/target/debug/atmosd");
    let explicit = std::env::var_os("ATMOS_BACKEND").map_or(debug, PathBuf::from);
    let found = locate(Some(explicit), None);
    found.is_absolute().then_some(found)
}

/// Runs `script` under Node with `REPO_DIR` set, and reads what it prints as JSON. `None` when
/// `node` is not installed.
pub fn node(script: &str) -> Option<Value> {
    let out = Command::new("node")
        .args(["-e", script])
        .env("REPO_DIR", repo())
        .output()
        .ok()?;
    assert!(
        out.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

/// The `search.query` body the QML window sends for an empty query: every row.
pub fn body() -> Option<Value> {
    node(
        r#"
const root = process.env.REPO_DIR;
const R = require(root + "/tests/harness.js").load("services/Requests.js");
const H = require(root + "/services/Hubs.js");
process.stdout.write(JSON.stringify(R.searchQuery("", H.searchContext())));
"#,
    )
}

/// Every row of the QML catalog, as `search.query` finds it for `body`.
pub fn catalog(atmosd: &Path, body: &Value) -> Vec<Value> {
    let scratch = std::env::temp_dir().join(format!(
        "atmos-ui-search-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut child = Command::new(atmosd)
        .arg("request")
        .env("ATMOS_ROOT", repo())
        .env("ATMOS_NO_AUTOSTART", "1")
        .env("HOME", &scratch)
        .env("XDG_RUNTIME_DIR", &scratch)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&scratch);
    let reply: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(reply["ok"], true, "{reply}");
    reply["result"]["hits"].as_array().unwrap().clone()
}

/// The QML catalog, or `None` (said) when there is no built `atmosd` or no `node`.
pub fn hits() -> Option<Vec<Value>> {
    let Some(bin) = atmosd() else {
        eprintln!("skip: no built atmosd (cargo build --manifest-path backend/Cargo.toml)");
        return None;
    };
    let Some(body) = body() else {
        eprintln!("skip: node is not installed");
        return None;
    };
    Some(catalog(&bin, &body))
}

/// JavaScript that defines `pageRoot(file, Omarchy, values)`: the `root` of a QML page under
/// Node, with the page's top-level `property` and `readonly property` definitions (as getters
/// over the page's scope) and `function`s, the JavaScript libraries the page imports, and
/// `Omarchy` as the state given. `values` sets plain properties (`agents`, a draft). What the
/// page's code makes of a state is then the page's own code, run as the QML window runs it.
pub const PAGE_ROOT: &str = r#"
const fs = require("fs");
const path = require("path");
const { load } = require(process.env.REPO_DIR + "/tests/harness.js");
function pageRoot(file, Omarchy, values) {
  const full = path.join(process.env.REPO_DIR, "pages", file);
  const src = fs.readFileSync(full, "utf8");
  const scope = { Omarchy, Theme: {}, Backend: {}, Qt: { callLater: (f) => f() } };
  for (const m of src.matchAll(/^import "([^"]+\.js)" as (\w+)$/gm)) {
    try { scope[m[2]] = load(path.relative(process.env.REPO_DIR, path.join(path.dirname(full), m[1]))); } catch (e) {}
  }
  for (const lib of ["RichUi", "Theme"]) if (!scope[lib + "Js"]) {}
  const root = {};
  scope.root = root;
  // A bare name is the page's own property first, as in QML.
  const proxy = new Proxy(scope, {
    has: (t, k) => k !== Symbol.unscopables,
    get: (t, k) => (k in t ? t[k] : k in root ? root[k] : globalThis[k]),
    set: (t, k, v) => { if (!(k in t) && k in root) root[k] = v; else t[k] = v; return true; },
  });
  const compile = (body) => new Function("__s", `with (__s) { return function () { ${body} } }`)(proxy);
  // The page's own statements: at two spaces, inside `PrefsPage {`.
  const lines = src.split("\n");
  let i = lines.findIndex((l) => /^\w+ \{/.test(l)) + 1;
  while (i < lines.length) {
    const line = lines[i];
    let m = line.match(/^  (readonly )?property \w+ (\w+): (.*)$/);
    let f = line.match(/^  function (\w+)\(([^)]*)\) \{\s*$/);
    const take = () => {
      // The statement's text: to its closing brace, bracket or parenthesis, or its continuation
      // lines.
      const out = [line];
      const open = (l) => (l.match(/[{[(]/g) || []).length - (l.match(/[}\])]/g) || []).length;
      let depth = open(line);
      i += 1;
      while (i < lines.length && (depth > 0 || /^   /.test(lines[i]) && !/^  \S/.test(lines[i]))) {
        if (depth <= 0 && !/^\s{4,}\S/.test(lines[i])) break;
        depth += open(lines[i]);
        out.push(lines[i]);
        i += 1;
      }
      return out;
    };
    if (m) {
      const text = take();
      text[0] = m[3];
      const expr = text.join("\n").trim();
      const name = m[2];
      if (!m[1] && values && name in values) { root[name] = values[name]; continue; }
      const getter = expr.startsWith("{") && !expr.startsWith("({")
        ? compile(expr.slice(1, expr.lastIndexOf("}")))
        : compile("return (" + expr + ");");
      if (m[1]) Object.defineProperty(root, name, { get: getter, enumerable: true });
      else { try { root[name] = getter(); } catch (e) { root[name] = undefined; } }
      continue;
    }
    if (f) {
      const text = take();
      const body = text.slice(1, -1).join("\n");
      root[f[1]] = new Function("__s", `with (__s) { return function (${f[2]}) { ${body} } }`)(proxy);
      continue;
    }
    i += 1;
  }
  return root;
}
"#;
