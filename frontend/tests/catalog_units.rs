//! The Services page (`pages/units.rs`) is the QML page: its units, filters, summary and each
//! unit's row and actions are `Systemd.js` and `ServiceRow.qml`, run under Node, and its writes
//! are `Omarchy.systemdAction`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::units;
use common::qml;
use serde_json::{Value, json};

#[test]
fn the_services_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/ServicesPage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "property string stateFilter: \"all\"",
        "placeholder: \"Search services…\" onEdited: function(value) { root.unitFilter = value }",
        "label: \"No matching services\"",
        "if (action === \"copy\") { Omarchy.copyText(row.unit) return }",
        "if (!row.allowed) return Omarchy.systemdAction(action, row.unit, row.scope)",
        "root.showOutput(\"Status · \" + row.unit, \"status\", row)",
        "root.showOutput(\"Logs · \" + row.unit, \"logs\", row)",
        "text = env && env.ok === true ? \"No output.\" : (FailureJs.errorText(env && env.error) || \"Could not read that unit.\")",
        "onClicked: root.selectFilter(\"failed\")",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    assert_eq!(units::output_title("logs", "a.service"), "Logs · a.service");
    let list = json!([
        {"unit": "bluetooth.service", "scope": "system", "active": "active", "sub": "running",
            "unitFileState": "Enabled", "description": "Bluetooth\nservice"},
        {"unit": "pipewire.service", "scope": "user", "active": "inactive", "unitFileState": "static"},
        {"unit": "cups.service", "active": "failed", "enabled": "disabled", "description": "\r\nCUPS"},
        {"unit": "tailscaled.service", "active": "activating", "unitFileState": "masked"},
        {"unit": "foo.service", "active": "active", "unitFileState": "enabled-runtime"},
        {"unit": "bar.timer", "active": "deactivating", "sub": "failed", "unitFileState": "generated"},
        {"unit": "bad unit", "active": "active"},
        {"unit": "docker.service", "scope": "user", "active": "inactive", "unitFileState": "disabled"},
        null,
    ]);
    let queries = json!([
        ["", "all"],
        ["", "running"],
        ["", "stopped"],
        ["", "failed"],
        ["", "enabled"],
        ["", "disabled"],
        ["BLUE", "all"],
        ["logs", "all"],
        ["allowlist", "stopped"],
        ["zzz", "all"]
    ]);
    let actions = json!([
        ["start", "bluetooth.service", "system"],
        ["restart", "pipewire.service", "user"],
        ["mask", "x.service", "system"],
        ["stop", "bad unit", "user"],
        ["enable", "a@b.service", "user"]
    ]);
    let script = format!(
        r#"{root}
const S = load("services/Systemd.js");
const Requests = load("services/Requests.js");
const row = fs.readFileSync(process.env.REPO_DIR + "/components/ServiceRow.qml", "utf8");
const hint = new Function("row", "return (" + row.split("readonly property string hint: ")[1].split("readonly property")[0] + ");");
const om = fs.readFileSync(process.env.REPO_DIR + "/services/Omarchy.qml", "utf8");
const m = om.match(/\n  function systemdAction\(([^)]*)\) \{{([\s\S]*?)\n  \}}\n/);
let sent = null;
const act = new Function("Requests", "runCommand", "return function(" + m[1] + ") {{" + m[2] + "}}")(
  Requests, (op, opts) => {{ sent = [op, !!(opts && opts.sudo)]; }});
const r = pageRoot("ServicesPage.qml", {{ systemdUnits: {list} }}, {{}});
process.stdout.write(JSON.stringify({{
  all: r.allRows,
  summary: r.summaryBits.map((b) => [b.id, b.text, !!b.failed]),
  chips: r.chips.map((c) => [c.id, c.label]),
  shown: {queries}.map(([q, f]) => S.listUnits(r.allRows, q, f).map((u) => u.unit)),
  rows: r.allRows.map((u) => [S.statusLine(u), hint(u), S.primaryAction(u), S.overflowActions(u).map((a) => [a.id, a.label])]),
  actions: {actions}.map(([a, u, s]) => {{ sent = null; act(a, u, s); return sent; }}),
}}));
"#,
        root = qml::PAGE_ROOT,
        list = list,
        queries = queries,
        actions = actions,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let all = units::all(Some(&list));
    let pairs = |v: &[(&str, &str)]| v.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>();
    let ours = json!({
        "all": all,
        "summary": units::summary(&all).into_iter().map(|(id, said, failed)| json!([id, said, failed])).collect::<Vec<_>>(),
        "chips": pairs(units::CHIPS),
        "shown": queries.as_array().unwrap().iter().map(|q| {
            units::shown(&all, q[0].as_str().unwrap(), q[1].as_str().unwrap())
                .iter().map(|u| u["unit"].clone()).collect::<Vec<_>>()
        }).collect::<Vec<_>>(),
        "rows": all.iter().map(|u| json!([
            units::status_line(u), units::hint(u),
            units::primary(u).map(|(id, label)| json!({"id": id, "label": label})),
            pairs(&units::overflow(u)),
        ])).collect::<Vec<_>>(),
        "actions": actions.as_array().unwrap().iter().map(|a| {
            let s = |i: usize| a[i].as_str().unwrap();
            units::action(s(0), s(1), s(2)).map(|(body, root)| json!([body, root]))
        }).collect::<Vec<Option<Value>>>(),
    });
    assert_eq!(ours, theirs);
}
