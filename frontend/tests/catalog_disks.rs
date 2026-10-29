//! The Disks page's rows (`pages/disks.rs`): each drive's title and test mount, Snapper's summary,
//! the snapshots' labels, swap, the speed test's answer and texts, and the hibernation row are the
//! QML page's own code, run under Node. What each setter sends is `catalog_setters.rs`'s.
//! Without `node` the test says so and stops.

mod common;

use atmos_ui::disks::{self, Run};
use common::qml;
use serde_json::{Value, json};

#[test]
fn the_disks_rows_are_the_qml_rows() {
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
    for root in ["setSnapperNumberLimit", "setSnapperTimeline", "setFstrim"] {
        assert!(body(root).contains("sudo: true"), "{root} runs as root");
    }
    let src = std::fs::read_to_string(qml::repo().join("pages/DisksPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "from: 1 to: 20 stepSize: 1 value: Omarchy.snapperNumberLimit",
        "if (next !== Omarchy.snapperNumberLimit) Omarchy.setSnapperNumberLimit(next)",
        "checked: Omarchy.snapperTimeline enabled: Omarchy.snapperPresent onToggled: Omarchy.setSnapperTimeline(!Omarchy.snapperTimeline)",
        "checked: Omarchy.fstrimEnabled onToggled: Omarchy.setFstrim(!Omarchy.fstrimEnabled)",
        "onConfirmed: Omarchy.restoreSnapshot(root.pendingRollbackConfig, String(root.pendingRollbackId))",
        "root.pendingRollbackConfig = String(modelData.config || \"root\")",
        "available: Omarchy.snapperPresent && Omarchy.snapshots.length === 0",
        "luksHint.text = \"Passphrases cannot be empty.\"",
        "luksHint.text = \"New passphrases do not match.\"",
        "Omarchy.changeDrivePassword(root.pendingLuksDevice, luksCurrent.currentText(), a)",
        "text: Omarchy.hibernationConfigured ? \"Remove…\" : \"Set up…\"",
        "enabled: !Omarchy.jobBusy && (Omarchy.hibernationConfigured || Omarchy.hibernationSupported)",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    let disk_list = json!([
        {"model": "Samsung 990", "name": "nvme0n1", "mounts": [{"target": "/boot"}, {"target": "/"}, {"target": "/home"}]},
        {"name": "sda", "mounts": [{"target": "/boot/efi"}]},
        {"mounts": "x"},
    ]);
    let configs =
        json!([[], [{"name": "root", "subvolume": "/"}, {"name": "home"}, {"subvolume": "/x"}]]);
    let snaps = json!([{"id": 12, "date": "2026-10-01", "description": "boot", "config": "root"}, {"id": 3}, null]);
    let swaps = json!([{"label": "zram0", "path": "/dev/zram0", "size": 8589934592u64}, {"name": "sw"}, {}]);
    let answers = json!([
        {"ok": true, "result": {"stdout": "disk Samsung\nread 3100.5\nwrite x\nwrite 2800\n", "exit": 0}},
        {"ok": true, "result": {"stdout": "read 10\n", "stderr": " no space ", "exit": 1}},
        {"ok": false, "error": {"message": ""}},
    ]);
    let says = src
        .split("label: \"Speed test\"")
        .nth(1)
        .unwrap()
        .split("description: {")
        .nth(1)
        .unwrap()
        .split("\n        hint:")
        .next()
        .unwrap()
        .trim()
        .trim_end_matches('}')
        .to_owned();
    let swap_label = src
        .split("model: Omarchy.swapDevices")
        .nth(1)
        .unwrap()
        .split("label: ")
        .nth(1)
        .unwrap()
        .split('\n')
        .next()
        .unwrap()
        .to_owned();
    let swap_says = src
        .split("model: Omarchy.swapDevices")
        .nth(1)
        .unwrap()
        .split("description: ")
        .nth(1)
        .unwrap()
        .split("hint:")
        .next()
        .unwrap()
        .to_owned();
    let script = format!(
        r#"{root}
const RichUi = load("services/RichUi.js");
const O = {{ snapperConfigs: [], jobBusy: false }};
const r = pageRoot("DisksPage.qml", O, {{}});
const says = new Function("root", "modelData", {says});
const swapLabel = new Function("modelData", "RichUi", "return (" + {swap_label} + ");");
const swapSays = new Function("modelData", "RichUi", "return (" + {swap_says} + ");");
const answer = (env) => {{
  Object.assign(r, {{ diskRunning: true, diskTarget: "/", diskName: "", diskRead: "", diskWrite: "", diskError: "", diskResults: {{}} }});
  r.finishDiskSpeedtest(env);
  const last = r.diskResult("/");
  return [r.diskRunning, r.diskName, r.diskRead, r.diskWrite, r.diskError, last && [last.name, last.read, last.write, last.error], says(r, {{ mounts: [{{ target: "/" }}] }})];
}};
process.stdout.write(JSON.stringify({{
  disks: {disks}.map((d) => [r.diskTitle(d), r.speedtestDir(d)]),
  snapper: {configs}.map((c) => {{ O.snapperConfigs = c; return r.snapperSummary(); }}),
  snapshots: {snaps}.map((s) => r.snapshotLabel(s)),
  swaps: {swaps}.map((s) => [String(swapLabel(s, RichUi)), swapSays(s, RichUi)]),
  answers: {answers}.map(answer),
  running: (() => {{
    Object.assign(r, {{ diskRunning: true, diskTarget: "/", diskName: "", diskResults: {{}} }});
    const a = says(r, {{ mounts: [{{ target: "/" }}] }});
    r.diskName = "Samsung";
    return [a, says(r, {{ mounts: [{{ target: "/" }}] }}), says(r, {{ mounts: [{{ target: "/home" }}] }})];
  }})(),
}}));
"#,
        root = qml::PAGE_ROOT,
        says = json!(says),
        swap_label = json!(swap_label),
        swap_says = json!(swap_says),
        disks = disk_list,
        configs = configs,
        snaps = snaps,
        swaps = swaps,
        answers = answers,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let answer = |env: &Value| {
        let mut r = Run {
            running: true,
            target: "/".into(),
            ..Run::default()
        };
        r.token = 1;
        if env["ok"] == true {
            r.finish(Ok(&env["result"]))
        } else {
            r.finish(Err(env["error"]["message"].as_str().unwrap()))
        }
        let last = r
            .results
            .get("/")
            .map(|l| json!([l.name, l.read, l.write, l.error]));
        json!([
            r.running,
            r.name,
            r.read,
            r.write,
            r.error,
            last,
            r.says("/")
        ])
    };
    let mut running = Run {
        running: true,
        target: "/".into(),
        ..Run::default()
    };
    let first = running.says("/");
    running.name = "Samsung".into();
    let ours = json!({
        "disks": disk_list.as_array().unwrap().iter().map(|d| json!([disks::title(d), disks::speedtest_dir(d)])).collect::<Vec<_>>(),
        "snapper": configs.as_array().unwrap().iter().map(|c| disks::snapper_summary(Some(c))).collect::<Vec<_>>(),
        "snapshots": snaps.as_array().unwrap().iter().map(disks::snapshot_label).collect::<Vec<_>>(),
        "swaps": swaps.as_array().unwrap().iter().map(|s| { let (l, d) = disks::swap(s); json!([l, d]) }).collect::<Vec<_>>(),
        "answers": answers.as_array().unwrap().iter().map(answer).collect::<Vec<_>>(),
        "running": [first, running.says("/"), running.says("/home")],
    });
    assert_eq!(ours, theirs);
}
