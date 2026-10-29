//! The Applications page's rows (`pages/launchers.rs`): what Add's dialog says and checks
//! (`submitAdd`), each launcher's row and confirm, and the launch-on-start rows are the QML
//! page's own code, run under Node. What a setter then sends is `catalog_setters.rs`'s.
//! Without `node` the test says so and stops.

mod common;

use atmos_ui::launchers::{self, Draft};
use common::qml;
use serde_json::{Value, json};

#[test]
fn the_applications_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/ApplicationsPage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    let om = std::fs::read_to_string(qml::repo().join("services/Omarchy.qml")).unwrap();
    let body = |name: &str| {
        om.split(&format!("  function {name}("))
            .nth(1)
            .unwrap()
            .split("\n  }\n")
            .next()
            .unwrap()
            .to_owned()
    };
    assert!(
        body("removeDesktopApp").contains("sudo: true"),
        "a desktop launcher is removed as root"
    );
    assert!(!body("removeTui").contains("sudo") && !body("removeWebApp").contains("sudo"));
    for want in [
        "readonly property var windowStyleOptions: [ { value: \"tile\", label: \"Tiled\" }, { value: \"float\", label: \"Floating\" } ]",
        "addIconField.setText(kind === \"tui\" ? \"utilities-terminal\" : \"\")",
        "root.addWindowStyle = \"tile\"",
        "placeholder: root.addKind === \"tui\" ? \"lazydocker\" : \"Command\"",
        "placeholder: \"https://example.com\"",
        "placeholder: root.addKind === \"web\" ? \"Icon URL or name (optional)\" : \"Icon URL or name\"",
        "onClicked: root.openAdd(\"desktop\")",
        "onClicked: root.openAdd(\"tui\")",
        "onClicked: root.openAdd(\"web\")",
        "if (root.pendingKind === \"web\") Omarchy.removeWebApp(root.pendingId) else if (root.pendingKind === \"tui\") Omarchy.removeTui(root.pendingId) else Omarchy.removeDesktopApp(root.pendingId, root.pendingName)",
        "onSubmitted: function(value) { root.autostartDraft = value Omarchy.addAutostart(value) }",
        "enabled: root.autostartDraft.length > 0 onClicked: Omarchy.addAutostart(root.autostartDraft)",
        "available: Omarchy.autostart.length === 0 sectionHelp: false label: \"Launch on start\"",
        "visible: !!(modelData && modelData.managed) text: \"Remove…\"",
        "title: \"Remove autostart\" message: \"Stop launching \" + root.pendingAutostart + \" at login?\"",
        "onConfirmed: Omarchy.removeAutostart(root.pendingAutostart)",
        "dangerEnabled: !!(modelData && modelData.id) onDangered: root.askRemove(\"desktop\", modelData.id, modelData.name)",
        "dangerEnabled: !!(modelData && modelData.name) onDangered: root.askRemove(\"tui\", modelData.id, modelData.name)",
        "dangerEnabled: !!(modelData && modelData.name) onDangered: root.askRemove(\"web\", modelData.id, modelData.name)",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    for kind in launchers::KINDS {
        let (label, says) = launchers::empty(kind);
        let want = format!("label: {} description: {}", json!(label), json!(says));
        assert!(flat.contains(&want), "{want}");
    }
    let drafts = json!([
        ["web", " Notes ", "", "notes.example.com", "tile", ""],
        ["web", "a/b", "", "x.com", "tile", ""],
        ["web", "Mail", "", "ftp://x", "tile", "m"],
        ["web", "Mail", "", "has space.com", "tile", ""],
        ["tui", "Top", " htop ", "", "float", " "],
        ["tui", "Top", "", "", "float", ""],
        ["tui", "-x", "htop", "", "tile", ""],
        ["tui", "Top", "htop", "", "side", "icon"],
        ["desktop", "Edit", " zed ", "", "tile", "  "],
        ["desktop", "", "zed", "", "tile", ""],
    ]);
    let rows = json!([
        {"id": "notes", "name": "Notes", "detail": "notes.example.com"}, {"id": "", "name": ""}, {}, null,
    ]);
    let script = format!(
        r#"{root}
let job = null;
const O = {{ jobBusy: false, jobKind: "", autostart: [],
  installWebApp: (n, u, i) => {{ job = ["web", n, u, i]; }},
  installTui: (n, c, s, i) => {{ job = ["tui", n, c, s, i]; }},
  installDesktopApp: (n, c, i) => {{ job = ["desktop", n, c, i]; }} }};
const r = pageRoot("ApplicationsPage.qml", O, {{}});
const field = (v) => ({{ currentText: () => v, clear() {{}}, setText() {{}} }});
globalThis.addDialog = {{ open() {{}}, close() {{}} }};
const src = require("fs").readFileSync(process.env.REPO_DIR + "/pages/ApplicationsPage.qml", "utf8");
const delegates = ["model: Omarchy.desktopApps", "model: Omarchy.tuiApps", "model: Omarchy.webApps"].map((m) => {{
  const b = src.split(m)[1];
  const p = (n) => b.split(n + ": ")[1].split("\n")[0];
  return [p("label"), p("description"), p("hint")];
}});
process.stdout.write(JSON.stringify({{
  submits: {drafts}.map(([kind, name, command, url, style, icon]) => {{
    job = null;
    Object.assign(globalThis, {{ addNameField: field(name), addCommandField: field(command), addUrlField: field(url), addIconField: field(icon) }});
    Object.assign(r, {{ addKind: kind, addWindowStyle: style, addError: "" }});
    r.submitAdd();
    return [job, r.addError];
  }}),
  texts: ["desktop", "tui", "web"].map((k) => {{
    r.addKind = k;
    O.jobBusy = false;
    const idle = r.addBlurb();
    Object.assign(O, {{ jobBusy: true, jobKind: "tui-install" }});
    const making = r.addBlurb();
    Object.assign(O, {{ jobBusy: false, jobKind: "" }});
    return [r.addTitle(), idle, making, "Remove " + r.kindLabel(k)];
  }}),
  rows: delegates.map(([l, d, h]) => {rows}.map((modelData) => [
    new Function("modelData", "return (" + l + ");")(modelData),
    new Function("modelData", "return (" + d + ");")(modelData),
    new Function("modelData", "return (" + h + ");")(modelData),
  ])),
}}));
"#,
        root = qml::PAGE_ROOT,
        drafts = drafts,
        rows = rows,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let s = |v: &Value| v.as_str().unwrap().to_owned();
    let ours = json!({
        "submits": drafts.as_array().unwrap().iter().map(|d| {
            let draft = Draft { kind: s(&d[0]), name: s(&d[1]), command: s(&d[2]), url: s(&d[3]), style: s(&d[4]), icon: s(&d[5]) };
            match launchers::submit(&draft) {
                Ok(job) => {
                    let a = &job.args;
                    let call = match draft.kind.as_str() {
                        "web" => json!(["web", a["name"], a["url"], draft.icon]),
                        "tui" => json!(["tui", a["name"], a["command"], a["style"], a["icon"]]),
                        _ => json!(["desktop", a["name"], a["command"], a["icon"]]),
                    };
                    json!([call, ""])
                }
                Err(why) => json!([null, why]),
            }
        }).collect::<Vec<_>>(),
        "texts": launchers::KINDS.iter().map(|k| json!([
            launchers::add_title(k), launchers::add_blurb(k, false), launchers::add_blurb(k, true),
            launchers::remove_confirm(k, "x").0,
        ])).collect::<Vec<_>>(),
        "rows": launchers::KINDS.iter().map(|k| rows.as_array().unwrap().iter().map(|r| {
            let (label, says, hint, _) = launchers::row(k, r);
            json!([label, says, hint])
        }).collect::<Vec<_>>()).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);
}
