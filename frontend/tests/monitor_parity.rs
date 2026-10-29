//! `monitor.rs` is the Monitor hub's and Home's own code: each caption, Home's hero line and GPU
//! rows, the page cards, and the core bars and memory bar `Monitor.js` lays out, run under Node
//! on the same rows `stats.rs` keeps. Without `node` the test says so and stops.

mod common;

use atmos_ui::monitor::{self, Captions, Home};
use atmos_ui::stats;
use common::qml;
use serde_json::{Value, json};

fn sample(i: u64, temp: bool) -> Value {
    let n = i as f64;
    json!({
        "cpuIdle": 1000.0 + n * 70.0, "cpuTotal": 2000.0 + n * 100.0,
        "cpus": [{"id": 0, "idle": 500.0 + n * 30.0, "total": 1000.0 + n * 50.0}, {"id": 1, "idle": 400.0 + n * 9.0, "total": 900.0 + n * 50.0}],
        "load1": 0.5, "load5": 0.25, "load15": 1.0,
        "memUsed": 4_000_000, "memTotal": 16_000_000, "memAvail": 12_000_000, "memFree": 2_000_000, "memBuffers": 300_000, "memCached": 5_000_000,
        "netRx": 1_000_000.0 + n * 52_431.0, "netTx": 500_000.0 + n * 1_234.5,
        "disks": [{"name": "nvme0n1", "readSectors": 1000 + i * 2048, "writeSectors": 50 + i * 3}, {"name": "sda", "readSectors": 1}],
        "psi": {"cpu": 0.04, "memory": 12.5, "io": null},
        "tcp": {"established": 12, "listen": 3, "total": 40},
        "cpuTemp": if temp { json!(55.4) } else { json!(0) },
        "gpus": [{"card": "card1", "pciId": "10de:2204", "temp": 61}, {"card": "card0", "pciId": "8086:46a6", "integrated": true}],
    })
}

/// No rows; one row; three rows, the last without a temperature.
fn histories() -> Vec<Vec<Value>> {
    let mut out = vec![Vec::new()];
    let mut h = Vec::new();
    for i in 0..3 {
        h = stats::push(
            &h,
            &stats::parse(&sample(i, i < 2)).unwrap(),
            2000.0 * (i + 1) as f64,
        );
        if i == 0 || i == 2 {
            out.push(h.clone());
        }
    }
    out
}

#[test]
fn the_captions_are_the_pages_own() {
    let gpus = json!([
        {"name": "RTX", "vendor": "NVIDIA", "driver": "nvidia", "pciId": "10de:2204"},
        {"name": "Iris", "vendor": "Intel", "pciId": "8086:46a6", "driver": "i915"},
        {"card": "card9"},
    ]);
    let script = format!(
        r#"{root}
const out = {histories}.map((history) => {{
  globalThis.LiveStatsStore = {{ history, latest: history.length ? history[history.length - 1] : null, waiting: history.length === 0, sampleCount: history.length }};
  const m = pageRoot("MonitorPage.qml", {{}}, {{}});
  const h = pageRoot("HomePage.qml", {{ hardware: {{}} }}, {{}});
  return {{
    monitor: [m.cpuCaption(), m.memCaption(), m.netCaption(), m.diskCaption(), m.tempCaption(), m.loadCaption(), m.coresCaption(), m.psiCaption(), m.tcpCaption()],
    home: [h.cpuCaption(), h.memCaption(), h.netCaption(), h.cpuTempCaption(), h.heroLine(), ...{gpus}.map((g) => [h.gpuName(g), h.gpuDescription(g), h.gpuCaption(g)])],
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
    for (i, history) in histories().iter().enumerate() {
        let latest = history.last();
        let c = Captions {
            latest,
            waiting: history.is_empty(),
        };
        let h = Home {
            latest,
            rows: history.len(),
        };
        let mut home = vec![
            json!(h.cpu()),
            json!(h.mem()),
            json!(h.net()),
            json!(h.temp()),
            json!(h.hero()),
        ];
        home.extend(
            gpus.as_array()
                .unwrap()
                .iter()
                .map(|g| json!([monitor::gpu_name(g), monitor::gpu_says(g), h.gpu(g)])),
        );
        let ours = json!({
            "monitor": [c.cpu(), c.mem(), c.net(), c.disk(), c.temp(), c.load(), c.cores(), c.psi(), c.tcp()],
            "home": home,
        });
        assert_eq!(ours, theirs[i], "history of {}", history.len());
    }
}

/// The page cards, and where the core bars and the memory bar are drawn.
#[test]
fn the_cards_and_bars_are_monitor_js() {
    let values = [12.0, 95.0, -4.0, 130.0, 75.0, f64::NAN];
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const M = load("services/Monitor.js");
const L = load("services/LiveStats.js");
const parts = L.memParts({{ memTotal: 100, memFree: 20, memBuffers: 5, memCached: 25, memUsed: 1 }});
process.stdout.write(JSON.stringify({{
  cards: M.pageCards(),
  bars: M.barRects({values}, 200, 40, 1).map((r) => [r.x, r.y, r.w, r.h, r.alert === "hot"]),
  none: M.barRects([], 200, 40, 1),
  stacked: M.stackedRects(parts, 300, 10).map((r) => [r.x, r.y, r.w, r.h]),
}}));
"#,
        values = "[12, 95, -4, 130, 75, NaN]"
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let cards: Vec<Value> = monitor::PAGES
        .iter()
        .map(|(id, title, d)| json!({"id": id, "title": title, "description": d}))
        .collect();
    assert_eq!(json!(cards), theirs["cards"]);
    let close = |ours: Vec<Vec<f64>>, theirs: &Value| {
        let theirs = theirs.as_array().unwrap();
        assert_eq!(ours.len(), theirs.len());
        for (o, t) in ours.iter().zip(theirs) {
            for (k, v) in o.iter().enumerate() {
                let t = &t[k];
                match t.as_bool() {
                    Some(b) => assert_eq!(*v == 1.0, b),
                    None => assert!((v - t.as_f64().unwrap()).abs() < 1e-3, "{o:?} {t}"),
                }
            }
        }
    };
    let bars: Vec<Vec<f64>> = monitor::bars(&values, 200.0, 40.0)
        .iter()
        .map(|((x, y, w, h), hot)| {
            vec![
                *x as f64,
                *y as f64,
                *w as f64,
                *h as f64,
                if *hot { 1.0 } else { 0.0 },
            ]
        })
        .collect();
    close(bars, &theirs["bars"]);
    assert_eq!(theirs["none"], json!([]));
    let parts = stats::mem_parts(Some(
        &json!({"memTotal": 100, "memFree": 20, "memBuffers": 5, "memCached": 25, "memUsed": 1}),
    ));
    let stacked: Vec<Vec<f64>> = monitor::stacked(&parts, 300.0, 10.0)
        .iter()
        .map(|(x, y, w, h)| vec![*x as f64, *y as f64, *w as f64, *h as f64])
        .collect();
    close(stacked, &theirs["stacked"]);
    assert_eq!(monitor::sampling(0, "", ""), "No samples yet");
    assert_eq!(
        monitor::sampling(3, "cpu 0%", "bad"),
        "3 samples  ·  stall cpu 0%  ·  bad"
    );
}
