//! The pages that list what Atmos manages of a setting and add to it, Window rules and
//! Workspaces: their lists, what each row says, the checks their forms make and the writes their
//! own functions make are the QML page's code, run under Node. What a setter then sends is
//! `catalog_setters.rs`'s. Without `node` the tests say so and stop.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::catalog;
use common::qml;
use rows::{page_src, property};
use serde_json::{Value, json};

/// Window rules: its list and what each rule says are the page's own code, run under Node, and
/// Add's checks are its `submitAdd`, run with the fields it reads.
#[test]
fn the_window_rules_rows_are_the_qml_rows() {
    use atmos_ui::rules;
    let page = catalog::page("windows/rules").unwrap();
    let src = page_src(page);
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "root.placementDraft = \"float\" root.centerDraft = true root.widthDraft = 0 root.heightDraft = 0",
        "placeholder: \"firefox\"",
        "text: \"Use focused\" enabled: Omarchy.focusedClass.length > 0",
        "from: 0 to: 4000 stepSize: 10 value: root.widthDraft",
        "from: 0 to: 4000 stepSize: 10 value: root.heightDraft",
        "placeholder: \"Workspace (optional)\"",
        "title: \"Add a window rule\"",
        "text: \"Match is the window class Hyprland sees. Regex is fine. Leave size at 0 to skip it.\"",
        "text: \"Add\" primary: true enabled: root.matchDraft.length > 0",
        "enabled: modelData && modelData.managed",
        "title: \"Remove window rule\"",
        "message: \"Remove the Atmos rule for \" + root.pendingMatch + \"?\"",
        "onConfirmed: Omarchy.removeWindowRule(root.pendingMatch)",
        "available: root.ruleRows.length === 0",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    assert_eq!(
        rules::remove_message("mpv"),
        "Remove the Atmos rule for mpv?"
    );
    let listed = src.split("model: root.ruleRows").nth(1).unwrap();
    let exprs = json!({
        "label": property(listed, "label"),
        "description": property(listed, "description"),
    });
    let list = json!([
        {"match": "steam", "placement": "float", "managed": false},
        {"match": "mpv", "placement": "float", "center": true, "width": 1280, "height": 720, "managed": true},
        {"match": "obs", "workspace": "4", "title": "Pre", "pin": true, "fullscreen": true, "opacity": 0.9, "managed": true},
        {"match": "dev.csfh.atmos", "placement": "float", "managed": false},
        {"match": "dev.csfh.atmos", "placement": "tile", "managed": true},
        {"match": "bare", "managed": true},
        {"match": "", "managed": true},
        {"placement": "tile"},
        null,
    ]);
    let cases = json!([
        ["", "", "float", true, 0, 0],
        ["a\nb", "", "float", true, 0, 0],
        ["a]]b", "", "float", true, 0, 0],
        ["firefox", "bad ws!", "float", true, 0, 0],
        ["firefox", "", "float", true, 50, 0],
        ["firefox", "", "float", true, 0, 4000],
        ["firefox", "", "", false, 0, 0],
        [" firefox ", " 3 ", "", false, 0, 0],
        ["mpv", "", "tile", false, 1280, 720],
        ["x", "  ", "", true, 0, 0],
        ["x", "", "", false, 100, 100],
        ["x", "special:term", "", false, 0, 0],
    ]);
    let script = format!(
        r#"{root}
const e = {exprs};
const RuleJs = load("services/WindowRules.js");
const at = (expr, m) => new Function("modelData", "RuleJs", "return (" + expr + ");")(m, RuleJs);
let added = null;
const O = {{ windowRules: {list}, focusedClass: "", addWindowRule: (...a) => {{ added = a; }} }};
const r = pageRoot("windows/RulesPage.qml", O, {{}});
globalThis.addDialog = {{ open() {{}}, close() {{}} }};
const submits = {cases}.map(([m, ws, p, c, w, h]) => {{
  added = null;
  globalThis.matchField = {{ currentText: () => m }};
  globalThis.workspaceField = {{ currentText: () => ws }};
  Object.assign(r, {{ placementDraft: p, centerDraft: c, widthDraft: w, heightDraft: h, addError: "" }});
  r.submitAdd();
  return [added, r.addError];
}});
process.stdout.write(JSON.stringify([
  r.ruleRows.map((m) => [at(e.label, m), at(e.description, m)]),
  r.placementOptions.map((o) => [o.value, o.label]),
  submits,
]));
"#,
        root = qml::PAGE_ROOT,
        exprs = exprs,
        list = list,
        cases = cases
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let submits: Vec<Value> = cases
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let d = rules::Draft {
                matches: c[0].as_str().unwrap().into(),
                workspace: c[1].as_str().unwrap().into(),
                placement: c[2].as_str().unwrap().into(),
                center: c[3].as_bool().unwrap(),
                width: c[4].as_f64().unwrap(),
                height: c[5].as_f64().unwrap(),
            };
            match rules::submit(&d) {
                Ok((m, p, center, w, h, ws)) => {
                    let (w, h) = (atmos_ui::js::json_number(w), atmos_ui::js::json_number(h));
                    json!([[m, p, center, w, h, ws], ""])
                }
                Err(why) => json!([null, why]),
            }
        })
        .collect();
    let ours = json!([
        rules::rows(Some(&list))
            .iter()
            .map(|r| json!([rules::label(r), rules::description(r)]))
            .collect::<Vec<_>>(),
        rules::PLACEMENTS,
        submits,
    ]);
    assert_eq!(ours, theirs);
}

/// Workspaces: its lists, the state its writes start from, its selects' choices and the writes
/// its own functions make are the page's code, run under Node. Its switches are the exception
/// `workspaces.rs` gives the reason for: the QML calls `writeWorkspaces`, this window the switch's
/// own setting, which `catalog_setters.rs` holds to `dispatchSetting`.
#[test]
fn the_workspaces_rows_are_the_qml_rows() {
    use atmos_ui::workspaces as ws;
    let page = catalog::page("workspaces").unwrap();
    let src = page_src(page);
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "from: 1 to: 10 stepSize: 1 value: Omarchy.workspaceBarCount",
        "var next = WsJs.clampShown(value) if (next !== Omarchy.workspaceBarCount) root.setCount(next)",
        "checked: Omarchy.workspaceWrapSwitch onToggled: function(next) { Omarchy.writeWorkspaces(Omarchy.workspaces, next, Omarchy.workspaceWheelSwitch)",
        "checked: Omarchy.workspaceWheelSwitch onToggled: function(next) { Omarchy.writeWorkspaces(Omarchy.workspaces, Omarchy.workspaceWrapSwitch, next)",
        "value: WsJs.defaultId(root.numbered) options: WsJs.defaultOptions(root.numbered)",
        "if (value === WsJs.defaultId(root.numbered)) return var state = root.currentState() root.writeItems(WsJs.withDefault(state.items, value), state.wrapSwitch, state.wheelSwitch)",
        "placeholder: \"Bar name\" value: modelData && modelData.name ? modelData.name : \"\" onSubmitted: function(value) { root.patchItem(modelData.id, { name: value }) }",
        "text: \"Set\" onClicked: root.patchItem(modelData.id, { name: workspaceNameField.currentText() })",
        "options: WsJs.monitorOptions(Omarchy.monitors, modelData && modelData.monitor)",
        "if (value !== cur) root.patchItem(modelData.id, { monitor: value })",
        "description: root.specialError.length ? root.specialError : \"Name it special:notes or similar.\"",
        "placeholder: \"notes\"",
        "if (state.items[i].id !== modelData.id) items.push(state.items[i])",
        "description: \"A scratch workspace. Remove drops the Atmos rule.\"",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    let named = src.split("model: root.numbered").nth(1).unwrap();
    let scratch = src.split("model: root.specials").nth(1).unwrap();
    let exprs = json!({
        "label": property(named, "label"),
        "description": property(named, "description"),
        "scratch": property(scratch, "label"),
    });
    let list = json!([
        {"id": "3", "name": "web", "monitor": "HDMI-A-1", "managed": false, "isDefault": true},
        {"id": "1", "name": "", "monitor": "DP-1", "default": true, "managed": true},
        {"id": 2, "name": "bad/name", "managed": true},
        {"workspace": "4", "default_name": "mail", "persistent": false, "managed": true},
        {"id": "special:notes", "special": true, "managed": true},
        {"id": "special:b", "managed": true},
        {"id": "11"},
        {"id": "3"},
        null,
    ]);
    let monitors = json!([{"name": "DP-1"}, {"id": "eDP-1"}, {"name": "DP-1"}, {}]);
    let drafts = json!(["music", " notes ", "", "a b", "x".repeat(25), "ok-1_2"]);
    let counts = json!([0, 1, 3.4, 9.6, 12, -2]);
    let script = format!(
        r#"{root}
const e = {exprs};
const WsJs = load("services/Workspaces.js");
const at = (expr, m) => new Function("modelData", "return (" + expr + ");")(m);
let wrote = null, set = null;
const O = {{
  workspaces: {list}, monitors: {monitors}, workspaceBarCount: 3,
  workspaceWrapSwitch: true, workspaceWheelSwitch: false,
  writeWorkspaces: (...a) => {{ wrote = a; }},
  dispatchSetting: (...a) => {{ set = a; }},
}};
const r = pageRoot("WorkspacesPage.qml", O, {{}});
const write = (f) => {{ wrote = null; f(); return wrote && wrote[0]; }};
process.stdout.write(JSON.stringify([
  // A row's label is a string property.
  r.numbered.map((m) => [String(at(e.label, m)), at(e.description, m)]),
  r.specials.map((m) => String(at(e.scratch, m))),
  r.currentState().items,
  WsJs.defaultId(r.numbered),
  WsJs.defaultOptions(r.numbered).map((o) => [o.value, o.label]),
  r.numbered.map((m) => WsJs.monitorOptions(O.monitors, m && m.monitor).map((o) => [o.value, o.label])),
  WsJs.withDefault(r.currentState().items, "4"),
  {counts}.map((n) => {{ set = null; r.setCount(n); return set; }}),
  write(() => r.patchItem("1", {{ name: "code" }})),
  write(() => r.patchItem(2, {{ monitor: "DP-1" }})),
  {drafts}.map((d) => {{
    r.specialError = "";
    const items = write(() => {{ r.specialDraft = d; r.addSpecial(); }});
    return [items, r.specialError];
  }}),
]));
"#,
        root = qml::PAGE_ROOT,
        exprs = exprs,
        list = list,
        monitors = monitors,
        counts = counts,
        drafts = drafts
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let l = Some(&list);
    let numbered = ws::numbered(l);
    let opts = |o: Vec<(String, String)>| json!(o);
    let ours = json!([
        numbered
            .iter()
            .map(|m| json!([ws::label(m, "Workspace"), ws::description(m)]))
            .collect::<Vec<_>>(),
        ws::specials(l)
            .iter()
            .map(|m| ws::label(m, "special"))
            .collect::<Vec<_>>(),
        ws::state(l),
        ws::default_id(&numbered),
        opts(ws::default_options(&numbered)),
        numbered
            .iter()
            .map(|m| opts(ws::monitor_options(Some(&monitors), m.get("monitor"))))
            .collect::<Vec<_>>(),
        ws::with_default(&ws::state(l), "4"),
        counts
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                let n = ws::clamp_shown(n.as_f64().unwrap());
                json!(["workspaceBarCount", atmos_ui::js::json_number(n)])
            })
            .collect::<Vec<_>>(),
        ws::patch(l, &json!("1"), "name", json!("code")),
        ws::patch(l, &json!(2), "monitor", json!("DP-1")),
        drafts
            .as_array()
            .unwrap()
            .iter()
            .map(|d| match ws::add_special(l, d.as_str().unwrap()) {
                Ok(items) => json!([items, ""]),
                Err(why) => json!([null, why]),
            })
            .collect::<Vec<_>>(),
    ]);
    assert_eq!(ours, theirs);
}
