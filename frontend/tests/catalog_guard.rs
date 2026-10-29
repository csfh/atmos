//! The keep-or-revert guard (`guard.rs`) is `Guard.js`, and the revert each guarded write opens
//! it with is the one the QML setter hands `dispatchSetting`, run under Node. Without `node` the
//! test says so and stops.

mod common;

use atmos_ui::guard::Guard;
use atmos_ui::system;
use common::qml;
use serde_json::{Value, json};

#[test]
fn the_guard_is_guard_js_and_each_revert_is_the_qml_setter_s() {
    let script = r#"
const fs = require("fs");
const { load } = require(process.env.REPO_DIR + "/tests/harness.js");
const G = load("services/Guard.js");
const om = fs.readFileSync(process.env.REPO_DIR + "/services/Omarchy.qml", "utf8");
const fn = (name, scope) => {
  const m = om.match(new RegExp("\\n  function " + name + "\\(([^)]*)\\) \\{([\\s\\S]*?)\\n  \\}\\n"));
  return new Function(...Object.keys(scope), "return function(" + m[1] + ") {" + m[2] + "}")(...Object.values(scope));
};
let s = G.empty();
const steps = [];
const open = (id, kind, now) => { const r = G.open(s, { id, revert: { kind } }, now); s = r.state; steps.push([r.armed, s.deadline, s.reverts.map((x) => x.id)]); };
open("a", "keyboardLayout", 1000);
open("b", "touchpad", 5000);
open("a", "keyboardLayout", 6000);
open("", "x", 7000);
steps.push(G.secondsLeft(s, 6000), G.secondsLeft(s, 18000));
let t = G.tick(s, 17999); steps.push(t.fire.length);
t = G.tick(s, 18000); steps.push(t.fire.map((f) => f.kind), t.state.open);
steps.push(G.revert(G.empty()).fire.length);
let sent = null;
const set = fn("setKeyboardLayout", { keyboardLayout: "us", dispatchSetting: (k, v, g) => { sent = [k, v, g]; } });
const guards = ["de", "us", "fr,us", "Bad"].map((l) => { sent = null; set(l); return sent; });
process.stdout.write(JSON.stringify({ steps, guards }));
"#;
    let Some(theirs) = qml::node(script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let mut g = Guard::default();
    let mut steps: Vec<Value> = Vec::new();
    for (id, kind, now) in [
        ("a", "keyboardLayout", 1000),
        ("b", "touchpad", 5000),
        ("a", "keyboardLayout", 6000),
        ("", "x", 7000),
    ] {
        let armed = g.open(id, json!({ "kind": kind }), now);
        let ids: Vec<&str> = g.reverts.iter().map(|(i, _)| i.as_str()).collect();
        steps.push(json!([armed, g.deadline, ids]));
    }
    steps.push(json!(g.seconds_left(6000)));
    steps.push(json!(g.seconds_left(18000)));
    steps.push(json!(g.tick(17_999).len()));
    let fired: Vec<Value> = g.tick(18_000).iter().map(|f| f["kind"].clone()).collect();
    steps.push(json!(fired));
    steps.push(json!(g.open));
    steps.push(json!(Guard::default().revert().len()));
    let guards: Vec<Value> = ["de", "us", "fr,us", "Bad"]
        .iter()
        .map(|l| match system::layout_write(l, "us") {
            Some((body, revert)) => {
                json!(["keyboardLayout", body["value"], {"id": "keyboardLayout", "revert": revert}])
            }
            None => Value::Null,
        })
        .collect();
    assert_eq!(json!({ "steps": steps, "guards": guards }), theirs);
}
