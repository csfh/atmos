//! `meters.rs` is the Monitor pages' own code: each row's value is its QML `valueText`, and the
//! disks, interfaces, sensors, GPUs, processes and CPU histogram each page lists are its own
//! properties, run under Node on the same rows `stats.rs` keeps. Without `node` the test says so
//! and stops.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::catalog::{self, Control, custom::Custom};
use atmos_ui::meters::{self, Read};
use atmos_ui::{processes, stats};
use common::qml;
use serde_json::{Value, json};

fn sample(i: u64, odd: bool) -> Value {
    let n = i as f64;
    json!({
        "uid": 1000, "cpuIdle": 1000.0 + n * 70.0, "cpuTotal": 2000.0 + n * 100.0,
        "cpus": [{"id": 0, "idle": 500.0 + n * 30.0, "total": 1000.0 + n * 50.0, "freqMhz": 2400, "governor": "powersave"}, {"id": 1, "idle": 1, "total": 2}],
        "load1": 0.5, "load5": 0.25, "load15": 1.0,
        "memUsed": 4_000_000, "memTotal": 16_000_000, "memAvail": 12_000_000, "memFree": 2_000_000, "memBuffers": 300_000,
        "memCached": 5_000_000, "memAnon": 1_000_000, "memShared": 200_000, "memDirty": if odd { Value::Null } else { json!(512) },
        "swapUsed": 1000, "swapTotal": if odd { json!(0) } else { json!(2_000_000) },
        "netRx": 1_000_000.0 + n * 52_431.0, "netTx": 500_000.0 + n * 1_234.5,
        "ifaces": [{"name": "wlan0", "rx": 1000.0 + n * 5000.0, "tx": 10.0 + n}, {"name": "eth0", "rx": 1.0 + n * 90_000.0, "tx": 1.0}, {"name": "tun0", "rx": 5, "tx": 5}],
        "disks": [{"name": "nvme0n1", "readSectors": 1000 + i * 2048, "writeSectors": 50 + i * 3}, {"name": "sda", "readSectors": 1, "writeSectors": 1 + i * 100}],
        "psi": if odd { Value::Null } else { json!({"cpu": 0.04, "memory": 12.5, "io": null}) },
        "tcp": {"established": 12, "listen": 3, "timeWait": 2, "closeWait": 1, "total": 40},
        "cpuTemp": 55.4,
        "gpus": [{"card": "card1", "pciId": "10de:2204", "temp": 61, "busy": 12 + i, "vramUsed": 1_048_576, "vramTotal": 8_388_608}],
        "sensors": [{"chip": "k10temp", "label": "Tctl", "kind": "temp", "value": 48.25 + n}, {"id": "fan1", "chip": "nct", "label": "Fan", "kind": "fan", "value": 1200 + i}, {"chip": "acpi", "label": "Board", "kind": "temp", "value": 95}],
        "processes": [
            {"pid": 200, "comm": "firefox", "uid": 1000, "rssKb": 900_000, "ticks": 100 + i * 150, "mine": true, "readBytes": 4096 * i, "writeBytes": 10 * i},
            {"pid": 300, "comm": "bash", "uid": 1000, "rssKb": 4000, "ticks": 5, "mine": true},
            {"pid": 400, "comm": "Xorg", "uid": 0, "rssKb": 300_000, "ticks": 10 + i * 30, "mine": false, "readBytes": 1},
        ],
    })
}

/// No rows; one row; three rows; three rows with fields missing.
fn histories() -> Vec<Vec<Value>> {
    let mut out = vec![Vec::new()];
    for odd in [false, true] {
        let mut h = Vec::new();
        for i in 0..3 {
            h = stats::push(
                &h,
                &stats::parse(&sample(i, odd)).unwrap(),
                2000.0 * (i + 1) as f64,
            );
            if i == 0 && !odd {
                out.push(h.clone());
            }
        }
        out.push(h);
    }
    out
}

#[test]
fn each_row_s_value_is_its_value_text() {
    let mut cases = Vec::new();
    for page in catalog::pages().filter(|p| p.id.starts_with("monitor/")) {
        for line in page.lines() {
            let Control::Custom(Custom::Meter(id)) = line.control else {
                continue;
            };
            if id.starts_with("no") {
                continue;
            }
            let row = rows::qml_row(page, line.label, rows::nth(page, line));
            let text = row.split("valueText: ").nth(1).unwrap();
            // A block runs as a function body; one line returns its expression.
            let expr = if text.starts_with('{') {
                rows::block(text)
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .to_owned()
            } else {
                format!("return ({});", text.lines().next().unwrap())
            };
            cases.push(json!({"id": id, "file": page.file, "body": expr}));
        }
    }
    assert!(cases.len() >= 20, "the meters are found");
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("meters_parity.json");
    std::fs::write(
        &input,
        json!({"cases": cases, "histories": histories()}).to_string(),
    )
    .unwrap();
    let script = format!(
        r#"{root}
const {{ cases, histories }} = JSON.parse(require("fs").readFileSync({input}, "utf8"));
const L = load("services/LiveStats.js"), M = load("services/Monitor.js"), R = load("services/RichUi.js");
const out = histories.map((history) => cases.map((c) => {{
  globalThis.LiveStatsStore = {{ history, latest: history.length ? history[history.length - 1] : null, waiting: history.length === 0 }};
  const r = pageRoot(c.file, {{ hardware: {{}} }}, {{}});
  return new Function("root", "LiveStatsJs", "MonitorJs", "RichUi", c.body)(r, L, M, R);
}}));
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        input = json!(input.to_string_lossy()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    for (i, h) in histories().iter().enumerate() {
        let r = Read {
            history: h,
            waiting: h.is_empty(),
        };
        for (j, c) in cases.iter().enumerate() {
            let ours = r.meter(c["id"].as_str().unwrap()).0;
            assert_eq!(json!(ours), theirs[i][j], "{} on {} rows", c["id"], h.len());
        }
    }
}

/// The lists: disks and interfaces by each sort and a filter, sensors by kind and filter, each
/// page's processes, the GPUs' words, and the CPU histogram.
#[test]
fn each_list_is_its_page_s() {
    let gpus = json!([{"name": "RTX", "card": "card1", "pciId": "10de:2204", "driver": "nvidia"}, {"name": "Iris", "card": "card0"}]);
    let script = format!(
        r#"{root}
const L = load("services/LiveStats.js"), M = load("services/Monitor.js");
const out = {histories}.map((history) => {{
  globalThis.LiveStatsStore = {{ history, latest: history.length ? history[history.length - 1] : null, waiting: history.length === 0 }};
  const latest = globalThis.LiveStatsStore.latest;
  const disks = ["io", "read", "write", "name"].map((s) => pageRoot("monitor/DiskPage.qml", {{}}, {{ diskSort: s, diskFilter: "" }}).disks.map((d) => d.name));
  const d = pageRoot("monitor/DiskPage.qml", {{}}, {{ diskSort: "io", diskFilter: "NV" }});
  const ifaces = ["io", "rx", "tx", "name"].map((s) => pageRoot("monitor/TrafficPage.qml", {{}}, {{ ifaceSort: s, ifaceFilter: "" }}).ifaces.map((x) => x.name));
  const t = pageRoot("monitor/TrafficPage.qml", {{}}, {{ ifaceSort: "io", ifaceFilter: "" }});
  const sensors = [["all", ""], ["temp", ""], ["fan", ""], ["all", "board"]].map(([k, q]) => pageRoot("monitor/SensorsPage.qml", {{ hardware: {{}} }}, {{ sensorKind: k, sensorFilter: q }}).sensors.map((s) => s.label));
  const s = pageRoot("monitor/SensorsPage.qml", {{ hardware: {{}} }}, {{}});
  const c = pageRoot("monitor/CpuPage.qml", {{}}, {{}}), m = pageRoot("monitor/MemoryPage.qml", {{}}, {{}});
  const gpuSays = (g) => {{ const bits = []; const v = s.gpuVramText(g); if (g && g.driver) bits.push(g.driver); if (v) bits.push(v); return bits.join(" · "); }};
  const gpuLine = (g) => {{ const busy = L.formatPercent(L.gpuBusyAt(latest, g)); const temp = L.formatTemp(L.gpuTempAt(latest, g)); if (busy && temp) return busy + "  ·  " + temp; return busy || temp || s.waiting(); }};
  return {{
    disks, filtered: d.disks.map((x) => x.name), lines: (latest ? latest.disks : []).map((x) => d.diskLine(x)),
    ifaces, ifaceLines: (latest ? latest.ifaces : []).map((x) => t.ifaceLine(x)),
    sensors, sensorLines: (latest ? latest.sensors : []).map((x) => s.sensorText(x)),
    procs: [c.hotProcs, m.fatProcs, d.ioProcs].map((l) => l.map((p) => p.pid)),
    gpus: {gpus}.map((g) => [gpuSays(g), gpuLine(g)]),
    hist: M.cpuHistogram(latest ? latest.processes : []).map((b) => [b.label, b.count]),
  }};
}});
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        histories = json!(histories()),
        gpus = gpus,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let names =
        |rows: Vec<Value>| -> Vec<Value> { rows.iter().map(|r| r["name"].clone()).collect() };
    let labels =
        |rows: Vec<Value>| -> Vec<Value> { rows.iter().map(|r| r["label"].clone()).collect() };
    for (i, h) in histories().iter().enumerate() {
        let r = Read {
            history: h,
            waiting: h.is_empty(),
        };
        let latest = h.last();
        let list = |k: &str| {
            latest
                .and_then(|l| l.get(k))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        };
        let dk = [("read", "readBps"), ("write", "writeBps")];
        let ik = [("rx", "rxBps"), ("tx", "txBps")];
        let uid = latest.and_then(|l| stats::finite(l.get("uid")));
        let procs: Vec<Vec<Value>> = ["cpu", "memory", "io"]
            .iter()
            .map(|p| {
                processes::list(
                    &list("processes"),
                    &processes::Filter {
                        uid,
                        ..meters::procs(p)
                    },
                )
                .iter()
                .map(|r| r["pid"].clone())
                .collect()
            })
            .collect();
        let disks: Vec<_> = ["io", "read", "write", "name"]
            .iter()
            .map(|s| names(meters::rates(&list("disks"), "", s, dk)))
            .collect::<Vec<_>>();
        let ifaces: Vec<_> = ["io", "rx", "tx", "name"]
            .iter()
            .map(|s| names(meters::rates(&list("ifaces"), "", s, ik)))
            .collect::<Vec<_>>();
        let sensors: Vec<_> = [("all", ""), ("temp", ""), ("fan", ""), ("all", "board")]
            .iter()
            .map(|(k, q)| labels(meters::sensors(&list("sensors"), q, k)))
            .collect::<Vec<_>>();
        let ours = json!({
            "disks": disks,
            "filtered": names(meters::rates(&list("disks"), "NV", "io", dk)),
            "lines": list("disks").iter().map(|d| r.pair_line(d, "readBps", "writeBps")).collect::<Vec<_>>(),
            "ifaces": ifaces,
            "ifaceLines": list("ifaces").iter().map(|d| r.pair_line(d, "rxBps", "txBps")).collect::<Vec<_>>(),
            "sensors": sensors,
            "sensorLines": list("sensors").iter().map(|s| r.sensor_line(s)).collect::<Vec<_>>(),
            "procs": procs,
            "gpus": gpus.as_array().unwrap().iter().map(|g| json!([r.gpu_says(g), r.gpu_line(g)])).collect::<Vec<_>>(),
            "hist": meters::histogram(&list("processes")).iter().map(|(l, n)| json!([l, n])).collect::<Vec<_>>(),
        });
        assert_eq!(ours, theirs[i], "on {} rows", h.len());
    }
}
