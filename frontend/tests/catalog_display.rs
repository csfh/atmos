//! The Displays page's rows (`pages/display.rs`): each monitor's title, modes, rule and texts are
//! the QML page's own code and `RichUi.js`'s, its mode and Scale writes are the page's handlers,
//! and the layouts and rule patches are `Monitors.js`, run under Node. Each guarded setter hands
//! `dispatchSetting` (or `runCommand`) the revert `display.rs` opens the bar with. What each
//! setter sends is `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::{display, monitors};
use common::qml;
use serde_json::{Value, json};

fn monitors() -> Value {
    json!([
        {"name": "eDP-1", "description": "BOE 0x0BCA", "width": 2880, "height": 1920,
         "refreshRate": 120.0, "scale": 2, "focused": true, "enabled": true, "transform": 0,
         "vrr": false, "currentFormat": "XRGB8888", "internal": true, "mirrorOf": "none",
         "availableModes": ["2880x1920@120.00Hz", "2880x1920@60.00Hz", "1920x1200@60.00Hz", "junk"],
         "brightnessAvailable": true, "brightness": 40},
        {"name": "DP-1", "description": "Dell Inc. DELL U2723QE 7XKJ8T3", "width": 3840,
         "height": 2160, "refreshRate": 59.997, "scale": 1.5, "focused": false, "enabled": true,
         "transform": 0, "vrr": 0, "currentFormat": "XBGR2101010",
         "availableModes": ["3840x2160@59.997Hz", "3840x2160@30.00Hz", "2560x1440@143.98Hz", "2560x1440@59.95Hz"]},
        {"name": "HDMI-A-1", "width": 1920, "height": 1080, "refreshRate": 60, "scale": "1.25",
         "enabled": false, "transform": 1, "vrr": 1, "mirrorOf": "eDP-1"},
        {"name": "DP-2", "width": 1280, "height": 1024, "refreshRate": 75.025, "scale": 0,
         "enabled": true, "availableVrr": true, "hdr": true},
        {},
    ])
}

fn rules() -> Value {
    json!([
        {"output": "DP-1", "mode": "3840x2160@60", "position": "auto", "scale": 1.5, "transform": 2,
         "disabled": false, "vrr": 2, "bitdepth": 10, "cm": "wide"},
        {"output": "HDMI-A-1", "mode": "preferred", "disabled": true, "vrr": null, "cm": "hdr"},
    ])
}

fn options(list: &[(String, String)]) -> Value {
    list.iter()
        .map(|(v, l)| json!({"value": v, "label": l}))
        .collect()
}

#[test]
fn each_monitor_s_rows_are_the_qml_page_s() {
    let (ms, rs) = (monitors(), rules());
    let script = format!(
        r#"{root}
const O = {{ monitors: {ms}, monitorRules: {rs}, patched: [], patchMonitorRule: (n, p) => O.patched.push([n, p]) }};
const r = pageRoot("DisplaysPage.qml", O, {{}});
const R = load("services/RichUi.js");
const out = O.monitors.map((m) => {{
  const res = r.resolutionValue(m);
  return [
    r.monitorTitle(m), r.monitorSummary(m), r.resolutionDescription(m), res, r.refreshValue(m),
    R.monitorResolutions(m), R.monitorRefreshRates(m, res), R.monitorRefreshRates(m, "1920x1200"),
    R.monitorModeCopyText(m), R.currentMonitorModeValue(m), R.formatMonitorMode(R.currentMonitorModeValue(m)),
    r.ruleTransform(m), r.ruleDisabled(m), r.canDisable(m), r.disableDescription(m), r.ruleVrr(m),
    r.ruleBitdepth(m), r.ruleCm(m), r.vrrAvailable(m), r.hdrAvailable(m), r.scaleValue(m), r.scaleOptions(m),
  ];
}});
for (const [res, hz] of [["2560x1440", "143.98"], ["1920x1200", ""], ["", "60"], ["bad", "x"]])
  r.writeMonitorMode(O.monitors[0], res, hz);
r.writeMonitorMode({{}}, "1920x1080", "60");
process.stdout.write(JSON.stringify({{ out, patched: O.patched }}));
"#,
        root = qml::PAGE_ROOT
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let (live, rules) = (ms.as_array().unwrap(), rs.as_array().unwrap());
    for (i, m) in live.iter().enumerate() {
        let res = display::current_resolution(m);
        let ours = json!([
            display::title(m),
            display::summary(m),
            display::resolution_says(m),
            res,
            display::current_refresh(m, &res),
            options(&display::resolutions(m)),
            options(&display::refresh_rates(m, &res)),
            options(&display::refresh_rates(m, "1920x1200")),
            display::copy_text(m),
            display::current_mode(m),
            display::format_mode(&display::current_mode(m)),
            display::rule_transform(m, rules),
            display::rule_disabled(m, rules),
            display::can_disable(m, live, rules),
            display::disable_says(m, live, rules),
            display::rule_vrr(m, rules),
            display::rule_bitdepth(m, rules),
            display::rule_cm(m, rules),
            display::vrr_available(m),
            display::hdr_available(m, rules),
            display::scale_value(m),
            options(&display::scale_options(m)),
        ]);
        assert_eq!(ours, theirs["out"][i], "monitor {i}");
    }
    let mut patched = Vec::new();
    for (res, hz) in [
        ("2560x1440", "143.98"),
        ("1920x1200", ""),
        ("", "60"),
        ("bad", "x"),
    ] {
        if let Some(p) = display::mode_patch(res, hz) {
            patched.push(json!(["eDP-1", p]));
        }
    }
    assert_eq!(json!(patched), theirs["patched"]);
}

/// The Scale row's handler: the focused output's scale, or a rule for another.
#[test]
fn the_scale_row_writes_what_the_qml_handler_writes() {
    let src = std::fs::read_to_string(qml::repo().join("pages/DisplaysPage.qml")).unwrap();
    let row = src.split("label: \"Scale\"").nth(1).unwrap();
    let start =
        row.find("onChanged: function(value) {").unwrap() + "onChanged: function(value) {".len();
    let mut depth = 1;
    let mut end = start;
    for (i, c) in row[start..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            end = start + i;
            break;
        }
    }
    let handler = serde_json::to_string(&row[start..end]).unwrap();
    let (ms, rs) = (monitors(), rules());
    let script = format!(
        r#"{root}
const O = {{ monitors: {ms}, monitorRules: {rs}, sent: [],
  setMonitorScale: (v) => O.sent.push(["scale", v]), writeMonitorRules: (items) => O.sent.push(["rules", items]) }};
const r = pageRoot("DisplaysPage.qml", O, {{}});
const run = new Function("root", "Omarchy", "modelData", "value", {handler});
const out = [];
for (const m of O.monitors.slice(0, 4))
  for (const v of ["1", "2", "1.25", "3"]) {{ O.sent = []; run(r, O, m, v); out.push(O.sent); }}
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let (live, rules) = (ms.as_array().unwrap(), rs.as_array().unwrap());
    let mut ours = Vec::new();
    for m in &live[..4] {
        for v in ["1", "2", "1.25", "3"] {
            ours.push(if v == display::scale_value(m) {
                json!([])
            } else if m.get("focused") == Some(&Value::Bool(true)) {
                json!([["scale", v]])
            } else {
                json!([["rules", display::scale_rules(m, rules, v)]])
            });
        }
    }
    assert_eq!(json!(ours), theirs);
}

/// `Monitors.js`: the modes it keeps, and each layout's rules.
#[test]
fn modes_and_layouts_are_monitors_js() {
    let modes = [
        "1920x1080@60",
        " 2560x1440@143.98 ",
        "1920x1080",
        "preferred",
        "highrr",
        "1920x1080@60Hz",
        "bad",
        "1920x1080@",
        "99999x1@60",
        "",
    ];
    let hyprctl = [
        "2880x1920@120.00Hz",
        "1920x1080@59.94hz",
        "640x480",
        "x",
        "",
    ];
    let ms = monitors();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const M = load("services/Monitors.js");
const live = {ms};
process.stdout.write(JSON.stringify({{
  modes: {modes}.map((m) => M.sanitizeMode(m)),
  hyprctl: {hyprctl}.map((m) => M.modeFromHyprctl(m)),
  layouts: ["desk", "laptop", "docked", "other"].map((n) => M.layoutFromLive(n, live)),
}}));
"#,
        modes = json!(modes),
        hyprctl = json!(hyprctl),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let live = ms.as_array().unwrap();
    let layouts: Vec<Vec<Value>> = ["desk", "laptop", "docked", "other"]
        .iter()
        .map(|n| monitors::layout_from_live(n, live))
        .collect();
    let ours = json!({
        "modes": modes.iter().map(|m| monitors::sanitize_mode(m)).collect::<Vec<_>>(),
        "hyprctl": hyprctl.iter().map(|m| monitors::mode_from_hyprctl(m)).collect::<Vec<_>>(),
        "layouts": layouts,
    });
    assert_eq!(ours, theirs);
}

/// The reverts the guarded setters open the bar with, and what Edit opens.
#[test]
fn each_guard_is_the_one_the_setter_hands_on() {
    let (ms, rs) = (monitors(), rules());
    let script = format!(
        r#"
const fs = require("fs");
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const R = load("services/Requests.js");
const om = fs.readFileSync(process.env.REPO_DIR + "/services/Omarchy.qml", "utf8");
const fn = (name, scope) => {{
  const m = om.match(new RegExp("\\n  function " + name + "\\(([^)]*)\\) \\{{([\\s\\S]*?)\\n  \\}}\\n"));
  return new Function(...Object.keys(scope), "return function(" + m[1] + ") {{" + m[2] + "}}")(...Object.values(scope));
}};
let guard;
const dispatchSetting = (k, v, g) => {{ guard = g === undefined ? null : g; }};
const root = {{ reverting: false, copyJson: (x) => JSON.parse(JSON.stringify(x)) }};
const out = {{}};
out.rules = [{rs}, null, "x"].map((current) => {{
  guard = undefined;
  fn("writeMonitorRules", {{ monitorRules: current, root, dispatchSetting }})([]);
  return guard;
}});
out.touchpad = [[true, false], [false, true]].map(([now, on]) => {{
  guard = undefined;
  fn("setTouchpad", {{ touchpadEnabled: now, dispatchSetting }})(on);
  return guard;
}});
const SnapshotJs = {{ toList: (x) => (Array.isArray(x) ? x : []) }};
const runCommand = (op, opts) => {{ guard = (opts && opts.guard) || null; }};
out.scale = [[{ms}, "1.25"], [{ms}, "2"], [[{{ focused: true }}], "2"], [[], "2"]].map(([monitors, v]) => {{
  guard = undefined;
  fn("setMonitorScale", {{ monitors, root, SnapshotJs, Requests: R, runCommand }})(v);
  return guard === undefined ? "none" : guard;
}});
let opened = null;
fn("editMonitorsLua", {{
  Paths: {{ monitorsLuaFile: "/home/u/.config/hypr/monitors.lua" }},
  Quickshell: {{ env: () => "/home/u" }}, Requests: R, runCommand: (op) => {{ opened = op; }},
}})();
out.edit = opened;
process.stdout.write(JSON.stringify(out));
"#
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let guard = |id: &str, revert: Value| json!({"id": id, "revert": revert});
    let rules: Vec<Value> = [rs.clone(), Value::Null, json!("x")]
        .iter()
        .map(|current| {
            guard(
                "monitorRules",
                display::rules_write(Some(current), Vec::new()).1,
            )
        })
        .collect();
    let touchpad: Vec<Value> = [(true, false), (false, true)]
        .iter()
        .map(|(now, on)| {
            display::touchpad_write(*on, Some(&json!(now)))
                .and_then(|(_, r)| r)
                .map_or(Value::Null, |r| guard("touchpad", r))
        })
        .collect();
    let live = ms.as_array().unwrap().clone();
    let scale: Vec<Value> = [
        (live.clone(), "1.25"),
        (live, "2"),
        (vec![json!({"focused": true})], "2"),
        (Vec::new(), "2"),
    ]
    .iter()
    .map(|(monitors, v)| match display::scale_write(v, monitors) {
        None => json!("none"),
        Some((_, revert)) => revert.map_or(Value::Null, |r| guard("monitorScale", r)),
    })
    .collect();
    let ours = json!({
        "rules": rules, "touchpad": touchpad, "scale": scale,
        "edit": display::edit_write("/home/u"),
    });
    assert_eq!(ours, theirs);
}

/// What the page module draws without a function of the QML page's own to run.
#[test]
fn the_page_s_own_lines_are_those_display_rs_follows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/DisplaysPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "detail: \"Mode, scale, rotation, and disable write a monitor rule in ~/.config/hypr/monitors.lua. Brightness works on the built-in panel and on some external monitors.\"",
        "if (value !== root.resolutionValue(modelData)) root.writeMonitorMode(modelData, value, RichUi.currentMonitorRefreshValue(modelData, value))",
        "if (value !== root.refreshValue(modelData)) root.writeMonitorMode(modelData, root.resolutionValue(modelData), value)",
        "Omarchy.patchMonitorRule(modelData.name, { transform: Math.round(Number(value)) || 0 })",
        "if (!root.ruleDisabled(modelData) && !root.canDisable(modelData)) return Omarchy.patchMonitorRule(modelData.name, { disabled: !root.ruleDisabled(modelData) })",
        "available: !!(modelData && modelData.name && root.vrrAvailable(modelData))",
        "Omarchy.patchMonitorRule(modelData.name, { vrr: Math.round(Number(value)) || 0 })",
        "Omarchy.patchMonitorRule(modelData.name, { bitdepth: value === \"10\" ? 10 : 8 })",
        "Omarchy.patchMonitorRule(modelData.name, { cm: value })",
        "available: modelData && modelData.brightnessAvailable === true",
        "from: 1 to: 100 stepSize: 1",
        "value: modelData && modelData.brightness ? modelData.brightness : 1",
        "var next = Math.round(value) if (!modelData || next === modelData.brightness) return Omarchy.setDisplayBrightness(modelData.name, next)",
        "text: \"Desk\" onClicked: Omarchy.applyMonitorLayout(\"desk\")",
        "text: \"Laptop\" enabled: Omarchy.internalPresent onClicked: Omarchy.applyMonitorLayout(\"laptop\")",
        "text: \"Docked\" enabled: Omarchy.externalPresent onClicked: Omarchy.applyMonitorLayout(\"docked\")",
        "checked: Omarchy.touchpadEnabled enabled: Omarchy.touchpadPresent onToggled: Omarchy.setTouchpad(!Omarchy.touchpadEnabled)",
        "available: Omarchy.touchpadPresent",
        "available: Omarchy.keyboardBacklightPresent",
        "? (\"The keys are at \" + Omarchy.keyboardBrightness + \"%.\") : \"Step the keyboard LEDs up or down, or turn them off.\"",
        "text: \"Dim\" enabled: Omarchy.keyboardBacklightPresent onClicked: Omarchy.adjustKeyboardBacklight(\"down\")",
        "text: \"Brighten\" enabled: Omarchy.keyboardBacklightPresent onClicked: Omarchy.adjustKeyboardBacklight(\"up\")",
        "text: \"Turn off\" enabled: Omarchy.keyboardBacklightPresent && Omarchy.keyboardBrightness > 0 onClicked: Omarchy.adjustKeyboardBacklight(\"off\")",
        "available: Omarchy.monitors.length === 0 label: \"Monitors\" description: \"No monitors reported.\" hint: \"hyprctl monitors all\"",
        "text: \"Refresh\" onClicked: Omarchy.refresh()",
        "text: \"Edit\" onClicked: Omarchy.editMonitorsLua()",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    assert_eq!(display::backlight_says(40.0), "The keys are at 40%.");
    assert_eq!(
        display::backlight_says(0.0),
        "Step the keyboard LEDs up or down, or turn them off."
    );
}
