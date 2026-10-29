//! The Bar page's rows Atmos draws itself (`pages/bar.rs`): the tray's texts, the indicators'
//! switches and what a toggle hands the setter, the clock's choices, the usage refresh's minutes,
//! and the plugins' rows are the QML page's own code, run under Node. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::bar;
use common::qml;
use serde_json::json;

#[test]
fn the_bar_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/BarPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "available: Omarchy.trayPresent && Omarchy.trayHidden.length === 0 label: \"Hidden icons\" description: \"No hidden tray icons.\"",
        "available: Omarchy.trayPresent && Omarchy.trayHidden.length > 0 label: \"Hidden icons\"",
        "text: \"Show all\" enabled: Omarchy.trayPresent onClicked: Omarchy.clearTrayHidden()",
        "text: \"Unpin all\" enabled: Omarchy.trayPresent onClicked: Omarchy.clearTrayPinned()",
        "checked: root.indicatorShown(modelData.value) enabled: Omarchy.indicatorsPresent onToggled: root.toggleIndicator(modelData.value)",
        "from: 5 to: 60 stepSize: 5 value: Math.max(5, Math.round(Omarchy.agentsRefreshIntervalSec / 60))",
        "var next = Math.round(value) * 60 if (next !== Omarchy.agentsRefreshIntervalSec) Omarchy.setAgentsRefreshIntervalSec(next)",
        "onSubmitted: function(value) { Omarchy.setAgentsSyncFileName(value) }",
        "onClicked: Omarchy.setAgentsSyncFileName(\"\")",
        "onSubmitted: function(value) { Omarchy.setAgentsSyncDeviceId(value) }",
        "if (value !== Omarchy.clockFormat) Omarchy.set(\"clockFormat\", value)",
        "if (value !== Omarchy.clockFormatAlt) Omarchy.set(\"clockFormatAlt\", value)",
        "if (value !== Omarchy.clockWeekStart) Omarchy.setClockWeekStart(value)",
        "from: 0 to: 2100 value: Omarchy.clockBirthYear enabled: Omarchy.clockPresent onChanged: function(value) { Omarchy.setClockBirthYear(value) }",
        "from: 0 to: 150 value: Omarchy.clockLifeExpectancy enabled: Omarchy.clockPresent onChanged: function(value) { Omarchy.setClockLifeExpectancy(value) }",
        "onClicked: Omarchy.setClockBirthYear(\"\")",
        "available: Omarchy.plugins.length === 0 label: \"Plugins\" description: \"No plugins listed.\"",
        "Omarchy.setPluginEnabled(String(modelData.id), !modelData.enabled)",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    let items = json!([[], ["Dnd"], ["Dnd", "Reminder", "x"], ["x"]]);
    let positions = json!([
        ["top", ""],
        ["left", "HH\nmm"],
        ["bottom", "custom"],
        ["right", "weird"]
    ]);
    let plugins = json!([{"id": "a", "name": "Clock", "canDisable": false}, {"id": "b", "firstParty": true}, {}]);
    let block = src.split("model: Omarchy.plugins").nth(1).unwrap();
    let p = |n: &str| {
        block
            .split(&format!("{n}: "))
            .nth(1)
            .unwrap()
            .split("\n        hint")
            .next()
            .unwrap()
            .split("\n        description")
            .next()
            .unwrap()
            .to_owned()
    };
    let script = format!(
        r#"{root}
let sent = null;
const O = {{ trayHidden: ["org/a", "b"], trayPinned: ["c"], indicatorsItems: [], barPosition: "top", clockFormat: "", clockFormatAlt: "", clockWeekStart: "",
  setIndicatorsItems: (l) => {{ sent = l; }} }};
const r = pageRoot("BarPage.qml", O, {{}});
const label = new Function("modelData", "return (" + {label} + ");");
const said = new Function("modelData", "return (" + {said} + ");");
process.stdout.write(JSON.stringify({{
  tray: [r.trayHiddenLabels(), r.trayPinnedLabels()],
  shown: {items}.map((l) => {{ O.indicatorsItems = l; return ["Dictation", "Dnd", "Reminder"].map((i) => r.indicatorShown(i)); }}),
  toggles: {items}.map((l) => {{ O.indicatorsItems = l; return ["Dnd", "StayAwake"].map((i) => {{ sent = null; r.toggleIndicator(i); return sent; }}); }}),
  clocks: {positions}.map(([pos, cur]) => {{ O.barPosition = pos; O.clockFormat = cur; return r.clockFormatOptions.map((o) => [o.value, o.label]); }}),
  weeks: ["", "friday"].map((w) => {{ O.clockWeekStart = w; return r.weekStartOptions.map((o) => [o.value, o.label]); }}),
  plugins: {plugins}.map((m) => [String(label(m)), said(m)]),
}}));
"#,
        root = qml::PAGE_ROOT,
        label = json!(p("label")),
        said = json!(p("description")),
        items = items,
        positions = positions,
        plugins = plugins,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let pairs = |v: Vec<(String, String)>| {
        v.into_iter()
            .map(|(a, b)| json!([a, b]))
            .collect::<Vec<_>>()
    };
    let weeks: Vec<_> = ["", "friday"]
        .iter()
        .map(|w| pairs(bar::week_options(w)))
        .collect();
    let ours = json!({
        "tray": [["a", "b"], ["c"]],
        "shown": items.as_array().unwrap().iter().map(|l| ["Dictation", "Dnd", "Reminder"].map(|i| bar::indicator_shown(Some(l), i))).collect::<Vec<_>>(),
        "toggles": items.as_array().unwrap().iter().map(|l| ["Dnd", "StayAwake"].map(|i| json!(bar::toggle_next(Some(l), i)))).collect::<Vec<_>>(),
        "clocks": positions.as_array().unwrap().iter().map(|p| pairs(bar::clock_options(p[0].as_str().unwrap(), p[1].as_str().unwrap()))).collect::<Vec<_>>(),
        "weeks": weeks,
        "plugins": plugins.as_array().unwrap().iter().map(|p| { let (l, s, _) = bar::plugin_row(p); json!([l, s]) }).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);
    assert_eq!(
        bar::hidden_says(Some(&json!(["org/a", "b"]))),
        "Hidden right now: a, b. Show all brings them back."
    );
}
