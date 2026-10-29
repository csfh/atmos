//! The History page (`pages/history.rs`) and the window's log (`history.rs`): what a change
//! says and where it lands, how long ago it was, and what the page's rows say are `History.js`,
//! `Requests.label` and the QML page's, run under Node. Without `node` the test says so and stops.

mod common;

use atmos_ui::history;
use common::qml;
use serde_json::json;

#[test]
fn the_history_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/system/HistoryPage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    // Every row filters on the page's query; that is not what these pins are about.
    let flat = flat.replace(" query: root.query", "");
    for want in [
        format!("description: Preview.active ? {} : {}", json!(history::preview_says(true)), json!(history::preview_says(false))),
        "checked: Preview.active onToggled: function (value) { Preview.active = value if (!value) Omarchy.discardHeld() }".to_owned(),
        format!("description: Omarchy.heldChanges.length === 1 ? {} : Omarchy.heldChanges.length + {}", json!(history::held_says(1)), json!(history::held_says(0)[1..])),
        "available: Omarchy.heldChanges.length > 0".to_owned(),
        "text: \"Apply\" primary: true onClicked: Omarchy.applyHeld()".to_owned(),
        "text: \"Discard\" onClicked: Omarchy.discardHeld()".to_owned(),
        "label: modelData && modelData.file ? modelData.file : (modelData ? modelData.key : \"\") description: modelData ? modelData.text : \"\" valueText: \"held\"".to_owned(),
        "label: \"Nothing yet\" description: \"Change something and it will show up here with the command it ran.\" available: Omarchy.changeHistory.length === 0".to_owned(),
        "available: Omarchy.changeHistory.length > 0 valueText: String(HistoryJs.countFor(root.counts, \"you\"))".to_owned(),
        "label: modelData && modelData.file ? modelData.file : (modelData && modelData.key ? modelData.key : \"change\")".to_owned(),
        "valueText: modelData ? HistoryJs.relativeTime(modelData.at) : \"\"".to_owned(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }
    let bodies = json!([
        {"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 4, "readback": ["hyprLook"]},
        {"op": "write", "name": "theme.remove", "args": {"name": "tokyo night"}},
        {"op": "write", "name": "clipboard.copy", "args": {"text": "secret"}},
        {"op": "job", "name": "sudo.enable", "args": {"minutes": 15, "password": "x"}},
        {"op": "settings.set", "domain": "background", "value": "/home/u/Pictures/a b.png"},
        {"op": "write", "name": "open.path", "args": {"path": "/home/u/.config/hypr/atmos.lua"}},
        {"op": "write", "name": "x", "args": {"path": "~/.config/omarchy/shell.json"}},
        {"op": "write", "name": "x", "args": {"path": "/etc/a.jsonc", "n": null, "list": [1]}},
        {"op": "write", "name": "x", "args": {"long": "y".repeat(70)}},
    ]);
    let times = json!([0, 1000, 5000, 59000, 61000, 3_600_000, 90_000_000]);
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const H = load("services/History.js");
const R = load("services/Requests.js");
const now = 100_000_000;
process.stdout.write(JSON.stringify({{
  texts: {bodies}.map((b) => {{ const t = H.describe(R.label(b)); return [t, H.targetFile(R.label(b), "/home/u")]; }}),
  times: {times}.map((ago) => H.relativeTime(ago === 0 ? 0 : now - ago, now)),
  max: H.MAX_ENTRIES,
}}));
"#,
        bodies = bodies,
        times = times,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let now = 100_000_000u64;
    let ours = json!({
        "texts": bodies.as_array().unwrap().iter().map(|b| {
            let t = history::describe(&history::label(b));
            json!([t, history::target_file(&t, "/home/u")])
        }).collect::<Vec<_>>(),
        "times": times.as_array().unwrap().iter().map(|a| {
            let ago = a.as_u64().unwrap();
            history::relative_time(if ago == 0 { 0 } else { now - ago }, now)
        }).collect::<Vec<_>>(),
        "max": history::MAX,
    });
    assert_eq!(ours, theirs);
}
