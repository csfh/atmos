//! `stats.rs` is `services/LiveStats.js`: a run of samples, some of them odd, pushed through both
//! leaves the same rows, and every series and text read from them is the same. Without `node`
//! the test says so and stops.

#![recursion_limit = "512"]

mod common;

use atmos_ui::stats;
use common::qml;
use serde_json::{Value, json};

/// The `i`th sample: counters that climb (one that goes back), and fields that are missing,
/// negative, text, or zero along the way.
fn sample(i: u64) -> Value {
    let n = i as f64;
    let back = i == 7;
    json!({
        "uid": if i.is_multiple_of(5) { json!("1000") } else { json!(1000) },
        "cpuIdle": if back { json!(10) } else { json!(1000.0 + n * 70.0) },
        "cpuTotal": 2000.0 + n * 100.0,
        "cpus": [
            {"id": 0, "idle": 500.0 + n * 30.0, "total": 1000.0 + n * 50.0, "freqMhz": 1800 + i * 10, "governor": "powersave"},
            {"id": "1", "idle": 400.0 + n * 45.0, "total": 900.0 + n * 50.0, "freqMhz": 999.5},
            {"idle": -1, "total": null},
            null,
        ],
        "load1": 0.5 + n / 8.0, "load5": "0.75", "load15": if i.is_multiple_of(3) { Value::Null } else { json!(1.125) },
        "memUsed": 4_000_000 + i * 1000, "memTotal": 16_000_000, "memAvail": 12_000_000, "memFree": 2_000_000,
        "memBuffers": 300_000, "memCached": 5_000_000, "swapUsed": if i.is_multiple_of(4) { json!(-5) } else { json!(1000) }, "swapTotal": 2_000_000,
        "netRx": 1_000_000.0 + n * 52_431.0, "netTx": 500_000.0 + n * 1_234.5,
        "ifaces": [
            {"name": "wlan0", "rx": 1_000_000.0 + n * 52_431.0, "tx": 500_000.0 + n * 1_234.5, "rxPackets": 10},
            {"name": "  ", "rx": 1},
            {"name": "lo", "rx": if back { 0.0 } else { 100.0 + n }, "tx": 100.0 + n},
        ],
        "disks": [{"name": "nvme0n1", "readSectors": 1000 + i * 2048, "writeSectors": 50 + i * 3}, {"name": "sda"}],
        "psi": {"cpu": 0.04 + n / 10.0, "memory": "12.5", "io": -1},
        "tcp": {"established": 12, "listen": "3", "timeWait": null},
        "tcpSockets": [{"inode": 77, "state": "ESTABLISHED", "addr": "10.0.0.1", "port": 443, "uid": 1000, "pid": 4}, {"state": ""}, {"state": "LISTEN"}],
        "interrupts": {"cpus": 2, "rows": [{"id": "0", "values": [1 + i, "x"]}, {"values": [1]}, {"id": 9, "values": []}]},
        "softirqs": if i.is_multiple_of(2) { json!(null) } else { json!({"rows": [{"id": "NET_RX", "values": [i, i * 2]}]}) },
        "buddy": if i.is_multiple_of(6) { json!([1, -1]) } else { json!([4, 3, 2, 1]) },
        "cgroups": {"id": "/", "children": [{"path": "user.slice", "value": 100 + i}, {"id": "empty"}, {"id": "a", "children": [{"id": "b", "value": 1}]}]},
        "rapl": [{"id": "package-0", "uj": 1_000_000 + i * 15_000_000}, {"id": "core", "name": "Cores", "uj": "x"}, {"name": "no id", "uj": 1}],
        "slabs": [{"name": "dentry", "active": 1000 + i, "num": 2000}, {"name": "", "active": 1}, {"name": "x"}],
        "clkTck": if i.is_multiple_of(9) { json!(0) } else { json!(100) },
        "uptimeSec": 1000 + i * 2,
        "cpuTemp": if i.is_multiple_of(4) { json!(0) } else { json!(41.6 + n / 3.0) },
        "gpus": [
            {"card": "card1", "pciId": "10DE:2204", "name": "RTX", "driver": "nvidia", "temp": 55 + i, "busy": 12.5},
            {"card": "card0", "pciId": "8086:46a6", "name": "Iris", "driver": "i915", "integrated": true, "temp": 0},
            "junk",
        ],
        "sensors": [
            {"chip": "k10temp", "label": "Tctl", "kind": "temp", "value": 48.25 + n},
            {"id": "fan1", "kind": "fan", "value": 1200 + i},
            {"kind": "temp", "value": -3},
            {"kind": "volt", "value": 1},
        ],
        "processes": [
            {"pid": 1, "ppid": 0, "comm": "systemd", "cmdline": "/sbin/init", "uid": 0, "rssKb": 12000, "ticks": 500 + i * 3, "startTicks": 1, "state": " Sleeping", "threads": 1, "nice": 0, "readBytes": 4096 * i, "writeBytes": 10},
            {"pid": 42, "ppid": 1, "comm": "  firefox ", "uid": 1000, "rssKb": "800000", "ticks": if back { 10 } else { 100 + i * 150 }, "mine": true, "kthread": "yes", "depth": 3},
            {"pid": 0, "comm": "swapper"},
            {"pid": 9, "comm": ""},
            {"pid": "77", "comm": "kworker", "kthread": true, "ppid": -4, "mine": false},
        ],
    })
}

/// When each sample is taken: two seconds apart, one late and one not after the one before.
fn at(i: u64) -> f64 {
    match i {
        11 => 23_500.0,
        12 => 23_500.0,
        _ => 2000.0 * i as f64,
    }
}

const COUNT: u64 = 64;

/// A value with each whole float written as JSON writes it, an integer.
fn norm(v: Value) -> Value {
    match v {
        Value::Number(n) => n
            .as_f64()
            .map_or(Value::Number(n), atmos_ui::js::json_number),
        Value::Array(a) => Value::Array(a.into_iter().map(norm).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, norm(v))).collect()),
        v => v,
    }
}

#[test]
fn pushed_samples_and_what_is_read_from_them_are_live_stats_js() {
    let samples: Vec<Value> = (0..COUNT).map(sample).collect();
    let times: Vec<f64> = (0..COUNT).map(at).collect();
    let keys = [
        "cpu", "mem", "swap", "rxBps", "txBps", "load1", "cpuTemp", "nope",
    ];
    let gpus = json!(["card1", "8086:46A6", {"pciId": "10de:2204"}, {"name": "card0"}, {"driver": "i915"}, {"card": "x"}]);
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("stats_parity.json");
    std::fs::write(
        &input,
        json!({"samples": samples, "times": times}).to_string(),
    )
    .unwrap();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const L = load("services/LiveStats.js");
const {{ samples, times }} = JSON.parse(require("fs").readFileSync({input}, "utf8"));
let history = [];
const early = [];
samples.forEach((s, i) => {{
  history = L.pushSample(history, L.parse(s), times[i]);
  if (i === 3) early.push(JSON.parse(JSON.stringify(history)));
}});
const last = L.latest(history);
const keys = {keys};
const gpus = {gpus};
const vals = [null, -3, 0, 0.04, 0.05, 3.25159, 9.96, 10, 99.5, "12", 1023, 1024, 1536, 1048575, 5e9];
process.stdout.write(JSON.stringify({{
  early, history,
  series: keys.map((k) => L.series(history, k)),
  cores: [0, 1, 2, "1", 7].map((id) => L.coreSeries(history, id)),
  disks: [["nvme0n1", "readBps"], ["nvme0n1", "writeBps"], ["sda", "readBps"]].map(([n, k]) => L.diskSeries(history, n, k)),
  gpuSeries: gpus.map((g) => L.gpuSeries(history, g)),
  gpuTemp: gpus.map((g) => [L.gpuTempAt(last, g), L.gpuOwnTemp(last, g)]),
  gpuRows: [L.gpuRows([{{ name: "hw" }}], last), L.gpuRows([], last), L.gpuRows(null, null)],
  percents: vals.map((v) => L.formatPercent(v)),
  temps: vals.map((v) => L.formatTemp(v)),
  bps: vals.map((v) => L.formatBps(v)),
  mhz: vals.map((v) => L.formatMhz(v)),
  psi: vals.map((v) => L.formatPsi(v)),
  bytes: vals.map((v) => L.memBytes(v)),
  texts: [L.formatNet(last), L.formatNet(history[0]), L.formatLoadLine(last), L.formatLoadLine(history[0]), L.formatNet(null)],
  memParts: [L.memParts(last), L.memParts(null), L.memParts({{ memTotal: 10, memFree: 8, memBuffers: 4, memCached: 0, memUsed: 3 }})],
  corePercents: [L.corePercents(last), L.corePercents(null)],
  spark: [L.sparklinePoints(L.series(history, "cpu"), 120, 32, 2), L.sparklinePoints([5, 5, 5], 100, 20, 0), L.sparklinePoints([1], 10, 10, 0), L.sparklinePoints([1, 2], 3, 3, 2)],
}}));
"#,
        input = json!(input.to_string_lossy()),
        keys = json!(keys),
        gpus = gpus,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let mut history = Vec::new();
    let mut early = Vec::new();
    for (i, s) in samples.iter().enumerate() {
        history = stats::push(&history, &stats::parse(s).unwrap(), times[i]);
        if i == 3 {
            early.push(json!(history));
        }
    }
    assert_eq!(json!(early), theirs["early"], "the first rows");
    assert_eq!(history.len(), stats::SAMPLE_CAP);
    for (i, (ours, theirs)) in history
        .iter()
        .zip(theirs["history"].as_array().unwrap())
        .enumerate()
    {
        assert_eq!(ours, theirs, "row {i}");
    }
    let last = history.last();
    let vals = json!([
        null, -3, 0, 0.04, 0.05, 3.25159, 9.96, 10, 99.5, "12", 1023, 1024, 1536, 1_048_575, 5e9
    ]);
    let vals: Vec<Option<f64>> = vals
        .as_array()
        .unwrap()
        .iter()
        .map(|v| stats::finite(Some(v)))
        .collect();
    let fmt = |f: fn(Option<f64>) -> String| -> Value {
        json!(vals.iter().map(|v| f(*v)).collect::<Vec<_>>())
    };
    let points = |p: Vec<(f32, f32)>| -> Value {
        json!(
            p.iter()
                .map(|(x, y)| [*x as f64, *y as f64])
                .collect::<Vec<_>>()
        )
    };
    let g = gpus.as_array().unwrap();
    let cores: Vec<_> = [json!(0), json!(1), json!(2), json!("1"), json!(7)]
        .iter()
        .map(|id| stats::core_series(&history, id))
        .collect::<Vec<_>>();
    let disks: Vec<_> = [
        ("nvme0n1", "readBps"),
        ("nvme0n1", "writeBps"),
        ("sda", "readBps"),
    ]
    .iter()
    .map(|(n, k)| stats::disk_series(&history, n, k))
    .collect::<Vec<_>>();
    let mem_parts: Vec<_> = [last, None, Some(&json!({"memTotal": 10, "memFree": 8, "memBuffers": 4, "memCached": 0, "memUsed": 3}))]
            .iter()
            .map(|s| stats::mem_parts(*s).iter().map(|(id, label, kb)| json!({"id": id, "label": label, "kb": atmos_ui::js::json_number(*kb)})).collect::<Vec<_>>())
            .collect::<Vec<_>>();
    let ours = json!({
        "series": keys.iter().map(|k| stats::series(&history, k)).collect::<Vec<_>>(),
        "cores": cores,
        "disks": disks,
        "gpuSeries": g.iter().map(|k| stats::gpu_series(&history, k)).collect::<Vec<_>>(),
        "gpuTemp": g.iter().map(|k| json!([stats::gpu_temp_at(last.unwrap(), k), stats::gpu_own_temp(last.unwrap(), k)])).collect::<Vec<_>>(),
        "gpuRows": [stats::gpu_rows(Some(&json!([{"name": "hw"}])), last), stats::gpu_rows(Some(&json!([])), last), stats::gpu_rows(None, None)],
        "percents": fmt(stats::format_percent),
        "temps": fmt(stats::format_temp),
        "bps": fmt(stats::format_bps),
        "mhz": fmt(stats::format_mhz),
        "psi": fmt(stats::format_psi),
        "bytes": vals.iter().map(|v| stats::mem_bytes(v.map(atmos_ui::js::json_number).as_ref())).collect::<Vec<_>>(),
        "texts": [stats::format_net(last), stats::format_net(history.first()), stats::format_load_line(last), stats::format_load_line(history.first()), stats::format_net(None)],
        "memParts": mem_parts,
        "corePercents": [stats::core_percents(last), stats::core_percents(None)],
        "spark": [
            points(stats::sparkline_points(&stats::series(&history, "cpu"), 120.0, 32.0, 2.0)),
            points(stats::sparkline_points(&[5.0, 5.0, 5.0], 100.0, 20.0, 0.0)),
            points(stats::sparkline_points(&[1.0], 10.0, 10.0, 0.0)),
            points(stats::sparkline_points(&[1.0, 2.0], 3.0, 3.0, 2.0)),
        ],
    });
    let ours = norm(ours);
    for key in ours.as_object().unwrap().keys() {
        if key == "spark" {
            continue;
        }
        assert_eq!(ours[key], theirs[key], "{key}");
    }
    // The canvas draws in f32: the points agree to within a pixel's thousandth.
    for (o, t) in ours["spark"]
        .as_array()
        .unwrap()
        .iter()
        .zip(theirs["spark"].as_array().unwrap())
    {
        let (o, t) = (o.as_array().unwrap(), t.as_array().unwrap());
        assert_eq!(o.len(), t.len());
        for (a, b) in o.iter().zip(t) {
            for k in 0..2 {
                assert!(
                    (a[k].as_f64().unwrap() - b[k].as_f64().unwrap()).abs() < 1e-3,
                    "{a} {b}"
                );
            }
        }
    }
}
