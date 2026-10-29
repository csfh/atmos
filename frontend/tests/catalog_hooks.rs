//! The Hooks page (`pages/hooks.rs`): its events, scripts and texts, and Add's checks, are
//! `Hooks.js` and the QML page's own code, run under Node. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::hooks::{self, Draft};
use common::qml;
use serde_json::json;

#[test]
fn the_hooks_rows_are_the_qml_rows() {
    let listing = json!([
        {"type": "theme-set", "name": "a.sh", "path": "/home/u/.config/omarchy/hooks/theme-set.d/a.sh"},
        {"type": "theme-set", "name": "b.sh.sample", "path": "/p/b.sh.sample", "sample": true},
        {"type": "post-boot", "name": "post-boot", "path": "/p/post-boot", "flat": true},
        {"type": "custom-one", "name": "x", "path": "/p/x"},
        {"type": "Bad", "name": "x", "path": "/p/x"}, {"type": "post-boot", "name": "../x", "path": "/p"}, null,
    ]);
    let drafts = json!([
        ["theme-set", "file", "", "", ""],
        ["theme-set", "file", "/home/u/n.sh", "", ""],
        ["Bad!", "file", "/x", "", ""],
        ["font-set", "command", "", "", "echo"],
        ["font-set", "command", "", "notify", ""],
        ["font-set", "command", "", "notify", "echo $1"],
        ["battery-low", "command", "", "x.sample", "y"],
    ]);
    let names = json!([
        "notify", "a.py", "x.sample", "a/b", "-x", "ok_1.sh", "", "n.", "a..b"
    ]);
    let script = format!(
        r#"{root}
const H = load("services/Hooks.js");
let wrote = null;
const O = {{ hooks: {listing}, theme: "tokyo", font: "JetBrains",
  installHook: (t, p) => {{ wrote = ["install", t, p]; }}, createHook: (t, n, c) => {{ wrote = ["create", t, n, c]; }} }};
const r = pageRoot("HooksPage.qml", O, {{}});
globalThis.addDialog = {{ open() {{}}, close() {{}} }};
const field = (v) => ({{ currentText: () => v, setText() {{}} }});
process.stdout.write(JSON.stringify({{
  types: H.displayTypes(O.hooks).map((t) => [t.id, t.label, H.eventBlurb(t.id), r.runHint(t.id),
    r.itemsFor(t.id).map((i) => [i.name, i.path, r.describeItem(i)])]),
  names: {names}.map((n) => [H.sanitizeName(n), H.destHint("theme-set", n)]),
  drafts: {drafts}.map(([k, m, f, n, c]) => {{
    wrote = null;
    Object.assign(r, {{ installType: k, installMode: m, installFile: f, nameDraft: n, commandDraft: c, addError: "" }});
    Object.assign(globalThis, {{ nameField: field(n), commandField: field(c) }});
    const dest = r.destText();
    r.submitAdd();
    return [wrote, r.addError, dest];
  }}),
}}));
"#,
        root = qml::PAGE_ROOT,
        listing = listing,
        names = names,
        drafts = drafts,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let s = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let ours = json!({
        "types": hooks::display_types(Some(&listing)).into_iter().map(|(id, label)| json!([
            id, label, hooks::blurb(&id), hooks::run_hint(&id, "tokyo", "JetBrains"),
            hooks::items_for(Some(&listing), &id).iter().map(|i| json!([i["name"], i["path"], hooks::describe(i)])).collect::<Vec<_>>(),
        ])).collect::<Vec<_>>(),
        "names": names.as_array().unwrap().iter().map(|n| json!([hooks::sanitize_name(&s(n)), hooks::dest_hint("theme-set", &s(n))])).collect::<Vec<_>>(),
        "drafts": drafts.as_array().unwrap().iter().map(|d| {
            let draft = Draft { kind: s(&d[0]), mode: s(&d[1]), file: s(&d[2]), name: s(&d[3]), command: s(&d[4]) };
            let dest = hooks::dest_text(&draft);
            let called = match hooks::submit(&draft) {
                Err(why) => return json!([null, why, dest]),
                Ok(_) if draft.mode == "file" => json!(["install", draft.kind, draft.file]),
                Ok(_) => json!(["create", draft.kind, hooks::sanitize_name(&draft.name), hooks::sanitize_line(&draft.command)]),
            };
            json!([called, "", dest])
        }).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);
}
