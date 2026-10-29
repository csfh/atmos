//! The Speed test page (`pages/speedtest.rs`): what each row says in each state of a run, the
//! rates as `RichUi` writes them, and what an answer does to the run are the QML page's own code,
//! run under Node. Without `node` the test says so and stops.

mod common;

use atmos_ui::speedtest::{self, Run};
use common::qml;
use serde_json::{Value, json};

fn run(running: bool, phase: &str, down: &str, up: &str) -> Run {
    Run {
        running,
        phase: phase.into(),
        down: down.into(),
        up: up.into(),
        ..Run::default()
    }
}

#[test]
fn the_speed_test_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/network/SpeedtestPage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "Component.onCompleted: startSpeedtest()",
        "interval: 5000",
        "text: root.speedRunning ? \"Cancel\" : \"Run now\"",
        "enabled: root.speedRunning || Omarchy.netKind !== \"disconnected\"",
        "value: root.speedPhase === \"down\" ? 40 : (root.speedPhase === \"up\" ? 80 : (root.downloadMbps.length ? 100 : 0))",
        "available: root.speedError.length > 0 label: \"Could not run\" description: root.speedError",
        "text: \"Retry\" enabled: !root.speedRunning && Omarchy.netKind !== \"disconnected\"",
        "onClicked: Omarchy.copyText(RichUi.mbpsCopyText(\"Download\", root.downloadMbps))",
        "onClicked: Omarchy.copyText(RichUi.mbpsCopyText(\"Upload\", root.uploadMbps))",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    assert_eq!(speedtest::PHASE.as_millis(), 5000);
    let states = json!([
        [false, "", "", "", "wifi"],
        [true, "down", "", "", "wifi"],
        [true, "up", "93.4", "", "ethernet"],
        [true, "up", "93.4", "7.25", "wifi"],
        [false, "", "120.6", "0", "wifi"],
        [false, "", "", "", "disconnected"],
        [false, "", "5.04", "", "disconnected"],
        [true, "down", "", "", "disconnected"],
    ]);
    let rates = json!([
        "0", "0.04", "9.96", "10.5", "1234.5", " 42 Mbps", "-3", "x", "", "1e2", ".5", "5."
    ]);
    let answers = json!([
        ["up", {"ok": true, "result": {"stdout": "junk\n12.5\n", "exit": 0}}],
        ["down", {"ok": true, "result": {"stdout": "88\n", "stderr": " no route \n", "exit": 1}}],
        ["up", {"ok": true, "result": {"stdout": "", "exit": 2}}],
        ["down", {"ok": false, "error": {"message": "Backend went away"}}],
    ]);
    let script = format!(
        r#"{root}
const RichUi = load("services/RichUi.js");
globalThis.speedTimer = {{ stop() {{}}, restart() {{}} }};
const O = {{ netKind: "wifi" }};
const r = pageRoot("network/SpeedtestPage.qml", O, {{}});
const set = ([running, phase, down, up, net]) => {{
  O.netKind = net;
  Object.assign(r, {{ speedRunning: running, speedPhase: phase, downloadMbps: down, uploadMbps: up, speedError: "" }});
}};
process.stdout.write(JSON.stringify({{
  says: {states}.map((st) => {{
    set(st);
    return [r.phaseLabel(), r.progressDescription(), r.progressValueText(), r.downloadDescription(), r.uploadDescription()];
  }}),
  rates: {rates}.map((x) => [r.rateText(x), RichUi.mbpsCopyText(" Download ", x), RichUi.parseMbpsLine(x)]),
  answers: {answers}.map(([phase, env]) => {{
    set([true, phase, "", "", "wifi"]);
    r.speedAnswered(env);
    return [r.speedRunning, r.speedPhase, r.downloadMbps, r.uploadMbps, r.speedError];
  }}),
  offline: (() => {{ set([false, "", "", "", "disconnected"]); r.startSpeedtest(); return [r.speedRunning, r.speedError]; }})(),
}}));
"#,
        root = qml::PAGE_ROOT,
        states = states,
        rates = rates,
        answers = answers,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let s = |v: &Value| v.as_str().unwrap().to_owned();
    let mut off = Run::default();
    off.start(true);
    let offline = json!([off.running, off.error]);
    let ours = json!({
        "says": states.as_array().unwrap().iter().map(|st| {
            let r = run(st[0].as_bool().unwrap(), &s(&st[1]), &s(&st[2]), &s(&st[3]));
            let off = st[4] == "disconnected";
            json!([r.phase_label(off), r.progress_says(), r.progress_text(), r.down_says(off), r.up_says(off)])
        }).collect::<Vec<_>>(),
        "rates": rates.as_array().unwrap().iter().map(|x| {
            let n = speedtest::parse_mbps(&s(x));
            let n = if n.is_finite() { atmos_ui::js::json_number(n) } else { Value::Null };
            json!([speedtest::rate(&s(x)), speedtest::copy_text(" Download ", &s(x)), n])
        }).collect::<Vec<_>>(),
        "answers": answers.as_array().unwrap().iter().map(|a| {
            let mut r = run(true, &s(&a[0]), "", "");
            let env = &a[1];
            if env["ok"] == true {
                r.answered(Ok(&env["result"]));
            } else {
                r.answered(Err(env["error"]["message"].as_str().unwrap()));
            }
            json!([r.running, r.phase, r.down, r.up, r.error])
        }).collect::<Vec<_>>(),
        "offline": offline,
    });
    assert_eq!(ours, theirs);
}
