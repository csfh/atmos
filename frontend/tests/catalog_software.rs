//! The Software page's rows (`pages/software.rs`): its table of stacks is `Software.js`'s
//! `catalog()`, its choices are `devEnvs()` and `dockerDbs()`, and whether a stack is installed,
//! what its row says and what its confirm asks are the QML page's own code, run under Node.
//! What each job sends is `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::software::{self, ITEMS};
use common::qml;
use serde_json::json;

#[test]
fn the_software_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/SoftwarePage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        format!("property string devLang: {}", json!(software::DEV_LANG)),
        format!("property string dockerDb: {}", json!(software::DOCKER_DB)),
        "available: Omarchy.extras && Omarchy.extras.docker === true label: \"Docker database\"".to_owned(),
        "text: \"Install\" primary: true enabled: !Omarchy.jobBusy onClicked: Omarchy.installDockerDb(root.dockerDb)".to_owned(),
        "text: \"Install…\" primary: true enabled: !Omarchy.jobBusy onClicked: root.askDev(\"install\")".to_owned(),
        "text: \"Remove…\" danger: true enabled: !Omarchy.jobBusy onClicked: root.askDev(\"remove\")".to_owned(),
        "action: root.installed(modelData) ? \"\" : \"Install…\"".to_owned(),
        "dangerAction: root.installed(modelData) && modelData && modelData.remove ? \"Remove…\" : \"\"".to_owned(),
        "Omarchy.runSoftware(root.pendingArgv, root.pendingKind === \"remove\" ? \"software-remove\" : \"software-install\")".to_owned(),
        "if (root.pendingKind === \"remove\") Omarchy.removeDevEnv(root.pendingId) else Omarchy.installDevEnv(root.pendingId)".to_owned(),
        "confirmText: root.pendingKind === \"remove\" ? \"Remove\" : \"Install\"".to_owned(),
        "function installed(item) { return Soft.presentIn(item, root.bags) }".to_owned(),
        "readonly property var bags: ({ browsers: Omarchy.browsers, terminals: Omarchy.terminals, editors: Omarchy.editors, services: Omarchy.services, gaming: Omarchy.gaming, extras: Omarchy.extras })".to_owned(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }
    let bags = json!({
        "browsers": {"chrome": true, "brave": "yes"}, "terminals": {}, "editors": null,
        "services": {}, "gaming": {"steam": true}, "extras": {"docker": true},
    });
    let script = format!(
        r#"{root}
const Soft = load("services/Software.js");
const O = Object.assign({{ jobBusy: false }}, {bags});
const r = pageRoot("SoftwarePage.qml", O, {{}});
const src = require("fs").readFileSync(process.env.REPO_DIR + "/pages/SoftwarePage.qml", "utf8");
const block = src.split("id: softwareDelegate")[1];
const said = block.split("description: ")[1].split("hint:")[0];
const describe = new Function("root", "modelData", "return (" + said + ");");
const confirm = (kind, label, wipe) => {{
  Object.assign(r, {{ pendingKind: kind, pendingLabel: label, pendingWipe: wipe }});
  return [r.confirmTitle(), r.confirmMessage()];
}};
process.stdout.write(JSON.stringify({{
  catalog: Soft.catalog().map((i) => ({{ id: i.id, group: i.group, label: i.label, hint: i.hint || "",
    install: i.install || null, remove: i.remove || null, present: i.present, wipe: i.wipe === true }})),
  devEnvs: Soft.devEnvs().map((o) => [o.value, o.label]),
  dockerDbs: Soft.dockerDbs().map((o) => [o.value, o.label]),
  // `root.installed` is `Soft.presentIn(item, root.bags)`, run here with the items' own load: an
  // array from another load is not an `Array` there.
  rows: Soft.catalog().map((i) => {{
    const on = Soft.presentIn(i, r.bags);
    return [on, describe({{ installed: () => on }}, i)];
  }}),
  confirms: [confirm("install", "Steam", false), confirm("remove", "Steam", true), confirm("remove", "node", false)],
}}));
"#,
        root = qml::PAGE_ROOT,
        bags = bags,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours = json!({
        "catalog": ITEMS.iter().map(|i| json!({
            "id": i.id, "group": i.group, "label": i.label, "hint": i.hint,
            "install": i.install, "remove": i.remove, "present": [i.present.0, i.present.1], "wipe": i.wipe,
        })).collect::<Vec<_>>(),
        "devEnvs": software::DEV_ENVS,
        "dockerDbs": software::DOCKER_DBS,
        "rows": ITEMS.iter().map(|i| {
            let on = software::installed(i, bags.get(i.present.0));
            json!([on, software::description(i, on)])
        }).collect::<Vec<_>>(),
        "confirms": [
            software::confirm(false, "Steam", false),
            software::confirm(true, "Steam", true),
            software::confirm(true, "node", false),
        ],
    });
    assert_eq!(ours, theirs);
}
