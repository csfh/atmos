//! `charts.rs` is `services/Charts.js`: each chart's series from the same rows, and where its
//! marks go, to within a thousandth of a pixel. Without `node` the test says so and stops.

mod common;

use atmos_ui::{charts, layouts, stats};
use common::qml;
use serde_json::{Value, json};

fn history() -> Vec<Value> {
    let mut h = Vec::new();
    for i in 0..6u64 {
        let disks = match i {
            0 => json!([{"name": "nvme0n1", "readSectors": 1000, "writeSectors": 50}]),
            3 => json!([{"name": "sda", "readSectors": 9, "writeSectors": 9}, null, {"name": ""}]),
            _ => json!([
                {"name": "nvme0n1", "readSectors": 1000 + i * i * 2048, "writeSectors": 50 + i * 3},
                {"name": "sda", "readSectors": 9 + i * 700, "writeSectors": 9},
            ]),
        };
        h = stats::push(
            &h,
            &stats::parse(&json!({"disks": disks})).unwrap(),
            2000.0 * (i + 1) as f64,
        );
    }
    h
}

fn close(a: f64, b: &Value) -> bool {
    b.as_f64().is_some_and(|b| (a - b).abs() < 1e-3)
}

#[test]
fn the_disk_stream_is_charts_js() {
    let h = history();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const C = load("services/Charts.js");
const series = C.diskStream({h});
process.stdout.write(JSON.stringify({{
  series,
  layers: C.streamgraphLayers(series, 300, 80),
  edge: [C.streamgraphLayers([{{ id: "a", values: [1] }}], 10, 10), C.streamgraphLayers([{{ values: [0, 0] }}], 10, 10), C.streamgraphLayers([{{ id: "", values: [1, 2] }}, {{ values: ["x", 3] }}], 10, 10)],
}}));
"#,
        h = json!(h)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let series = charts::disk_stream(&h);
    let ours: Vec<Value> = series.iter().map(|(id, v)| json!({"id": id, "values": v.iter().map(|n| atmos_ui::js::json_number(*n)).collect::<Vec<_>>()})).collect();
    assert_eq!(json!(ours), theirs["series"]);
    let check = |layers: Vec<charts::Layer>, theirs: &Value| {
        let theirs = theirs.as_array().unwrap();
        assert_eq!(layers.len(), theirs.len());
        for (l, t) in layers.iter().zip(theirs) {
            assert_eq!(json!(l.id), t["id"]);
            for (edge, key) in [(&l.top, "top"), (&l.bottom, "bottom")] {
                let te = t[key].as_array().unwrap();
                assert_eq!(edge.len(), te.len());
                for ((x, y), p) in edge.iter().zip(te) {
                    assert!(close(*x, &p[0]) && close(*y, &p[1]), "{key} {x},{y} {p}");
                }
            }
            match &l.label {
                None => assert!(t.get("label").is_none()),
                Some(lb) => {
                    assert_eq!(json!(lb.align), t["align"]);
                    assert!(
                        close(lb.x, &t["lx"])
                            && close(lb.y, &t["ly"])
                            && close(lb.max_w, &t["maxW"]),
                        "{t}"
                    );
                }
            }
        }
    };
    check(
        charts::stream_layers(&series, 300.0, 80.0),
        &theirs["layers"],
    );
    check(
        charts::stream_layers(&[("a".into(), vec![1.0])], 10.0, 10.0),
        &theirs["edge"][0],
    );
    check(
        charts::stream_layers(&[("0".into(), vec![0.0, 0.0])], 10.0, 10.0),
        &theirs["edge"][1],
    );
    check(
        charts::stream_layers(
            &[
                (String::new(), vec![1.0, 2.0]),
                ("1".into(), vec![f64::NAN, 3.0]),
            ],
            10.0,
            10.0,
        ),
        &theirs["edge"][2],
    );
}

/// Rows across a few UTC days, with sensors, a package temperature, and RAPL counters, one of
/// which goes back.
fn days() -> Vec<Value> {
    let start = 1_761_600_000_000.0; // 2025-10-27T21:20:00Z, a Monday evening.
    (0..12u64)
        .map(|i| {
            let n = i as f64;
            json!({
                "at": start + n * 9.0 * 3_600_000.0,
                "cpuTemp": if i % 4 == 0 { Value::Null } else { json!(40.0 + n * 1.5) },
                "sensors": [{"kind": "temp", "value": 35.0 + (n * 7.0) % 20.0}, {"kind": "fan", "value": 9000}, {"kind": "temp", "value": "x"}],
                "rapl": [
                    {"id": "package-0", "name": "package", "uj": 1_000_000.0 + n * 15_000_000.0},
                    {"id": "core", "uj": if i == 11 { 5.0 } else { 500_000.0 + n * 4_000_000.0 }},
                    {"id": "dram", "name": "", "uj": 10.0 + n * 900_000.0},
                    {"name": "none"},
                ],
            })
        })
        .collect()
}

#[test]
fn the_waterfall_and_the_calendar_are_charts_js() {
    let h = days();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const C = load("services/Charts.js");
const h = {h};
const steps = [C.raplSteps(h[0], h[1]), C.raplSteps(h[10], h[11]), C.raplSteps(null, h[1]), C.raplSteps(h[1], {{}})];
const days = C.thermalDays(h);
const cal = (d, w, ht) => {{ const l = C.calendarLayout(d, w, ht); return {{ cells: l.cells.map((c) => [c.x, c.y, c.w, c.h, c.date, c.fill]), labels: l.labels.map((x) => [x.label, x.ly, x.maxW]) }}; }};
process.stdout.write(JSON.stringify({{
  steps,
  bars: steps.map((s) => C.waterfallBars(s, 240, 70).map((b) => [b.id, b.value, b.x, b.y, b.w, b.h, b.label || null, b.lx || 0, b.ly || 0, b.maxW || 0])),
  odd: C.waterfallBars([{{ id: "a", value: 3 }}, {{ id: "b", label: "", value: -5 }}, {{ value: "x" }}, {{ value: 1 }}], 100, 10).map((b) => [b.id, b.value, b.x, b.y, b.w, b.h, b.label || null]),
  days,
  cals: [cal(days, 300, 90), cal(days, 30, 90), cal([{{ date: "2025-02-30", value: 1 }}, {{ date: "2025-13-01", value: 2 }}, {{ date: "bad", value: 3 }}], 200, 70), cal([], 200, 70)],
}}));
"#,
        h = json!(h)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let near = |a: &Value, b: &Value| match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => (x - y).abs() < 1e-6,
        _ => a == b,
    };
    let same = |ours: &Value, theirs: &Value, at: &str| {
        let (o, t) = (ours.as_array().unwrap(), theirs.as_array().unwrap());
        assert_eq!(o.len(), t.len(), "{at}");
        for (a, b) in o.iter().zip(t) {
            match (a.as_array(), b.as_array()) {
                (Some(x), Some(y)) => {
                    assert_eq!(x.len(), y.len(), "{at}");
                    for (p, q) in x.iter().zip(y) {
                        assert!(near(p, q), "{at}: {a} {b}");
                    }
                }
                _ => assert!(near(a, b), "{at}: {a} {b}"),
            }
        }
    };
    let steps = [
        charts::rapl_steps(Some(&h[0]), Some(&h[1])),
        charts::rapl_steps(Some(&h[10]), Some(&h[11])),
        charts::rapl_steps(None, Some(&h[1])),
        charts::rapl_steps(Some(&h[1]), Some(&json!({}))),
    ];
    for (i, s) in steps.iter().enumerate() {
        let ours: Vec<Value> = s
            .iter()
            .map(|(id, label, v)| json!({"id": id, "label": label, "value": v}))
            .collect();
        for (o, t) in ours.iter().zip(theirs["steps"][i].as_array().unwrap()) {
            assert_eq!(o["id"], t["id"]);
            assert_eq!(o["label"], t["label"]);
            assert!(near(&o["value"], &t["value"]));
        }
        assert_eq!(ours.len(), theirs["steps"][i].as_array().unwrap().len());
        let bars: Vec<Value> = charts::waterfall(s, 240.0, 70.0)
            .iter()
            .map(|b| {
                let (x, y, w, hh) = b.rect;
                let (label, lx, ly, mw) = b
                    .label
                    .clone()
                    .map_or((Value::Null, 0.0, 0.0, 0.0), |(l, lx, ly, mw)| {
                        (json!(l), lx, ly, mw)
                    });
                json!([b.id, b.value, x, y, w, hh, label, lx, ly, mw])
            })
            .collect();
        same(&json!(bars), &theirs["bars"][i], "bars");
    }
    let odd = vec![
        ("a".to_owned(), "a".to_owned(), 3.0),
        ("b".to_owned(), "b".to_owned(), -5.0),
        ("3".to_owned(), "3".to_owned(), 1.0),
    ];
    let odd: Vec<Value> = charts::waterfall(&odd, 100.0, 10.0)
        .iter()
        .map(|b| {
            let (x, y, w, hh) = b.rect;
            json!([
                b.id,
                b.value,
                x,
                y,
                w,
                hh,
                b.label.as_ref().map(|l| l.0.clone())
            ])
        })
        .collect();
    // The QML labels a bar with its id where its label is empty, and numbers one without an id.
    same(&json!(odd), &theirs["odd"], "odd");
    let days = charts::thermal_days(&h);
    let ours_days: Vec<Value> = days
        .iter()
        .map(|(d, v)| json!({"date": d, "value": v}))
        .collect();
    same(
        &json!(
            ours_days
                .iter()
                .map(|d| json!([d["date"], d["value"]]))
                .collect::<Vec<_>>()
        ),
        &json!(
            theirs["days"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| json!([d["date"], d["value"]]))
                .collect::<Vec<_>>()
        ),
        "days",
    );
    let inputs = [
        (days.clone(), 300.0, 90.0),
        (days.clone(), 30.0, 90.0),
        (
            vec![
                ("2025-02-30".to_owned(), 1.0),
                ("2025-13-01".to_owned(), 2.0),
                ("bad".to_owned(), 3.0),
            ],
            200.0,
            70.0,
        ),
        (Vec::new(), 200.0, 70.0),
    ];
    for (i, (d, w, hh)) in inputs.iter().enumerate() {
        let (cells, labels) = charts::calendar(d, *w, *hh);
        let cells: Vec<Value> = cells
            .iter()
            .map(|c| json!([c.rect.0, c.rect.1, c.rect.2, c.rect.3, c.date, c.fill]))
            .collect();
        let labels: Vec<Value> = if cells.is_empty() {
            Vec::new()
        } else {
            labels.iter().map(|(n, y, mw)| json!([n, y, mw])).collect()
        };
        same(&json!(cells), &theirs["cals"][i]["cells"], "cells");
        same(&json!(labels), &theirs["cals"][i]["labels"], "labels");
    }
}

#[test]
fn the_beeswarm_and_the_radar_are_charts_js() {
    let sample = json!({"tcpSockets": [
        {"inode": 77, "state": "ESTABLISHED"}, {"inode": "x9", "state": "LISTEN"}, {"state": "ESTABLISHED"},
        {"inode": null, "state": "TIME_WAIT"}, {"state": ""}, null, {"inode": 5, "state": "ESTABLISHED"},
    ]});
    let ifaces = json!([
        {"name": "wlan0", "rx": 900_000, "tx": 120_000, "rxPackets": 800, "txPackets": 300, "rxDrop": 2, "rxErr": 1},
        {"name": "eth0", "rx": 10, "tx": 4_000_000, "rxPackets": "x", "txPackets": 50, "txDrop": 9},
        {"name": "", "rx": 1}, null, {"name": "tun0"},
    ]);
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const C = load("services/Charts.js");
const bees = C.tcpBees({sample});
const layout = (w, h) => {{ const l = C.beeswarmLayout(bees, w, h); return {{ points: l.points.map((p) => [p.x, p.y, p.r, p.group, p.id]), labels: l.labels.map((x) => [x.label, x.lx, x.maxW]) }}; }};
const series = C.netdevRadar({ifaces});
const axes = [{{ key: "rx", label: "rx" }}, {{ key: "tx", label: "tx" }}, {{ key: "packets", label: "packets" }}, {{ key: "drops", label: "drops" }}, {{ key: "errs", label: "errs" }}];
const radar = (w, h) => {{ const r = C.radarPolygons(series, axes, w, h); return {{
  spokes: r.axes.map((s) => [s.label, s.x, s.y, s.lx, s.ly, s.align, s.baseline, s.cx, s.cy]),
  polygons: r.polygons.map((p) => [p.id, p.points]), labels: r.labels.map((l) => [l.label, l.lx, l.ly]) }}; }};
process.stdout.write(JSON.stringify({{
  bees, hash: ["", "a", "77x", "ç漢", "😀"].map((s) => C.hash01(s)),
  layouts: [layout(300, 80), layout(40, 10)],
  series, radars: [radar(240, 160), radar(20, 20)],
}}));
"#
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let close = |a: f64, b: &Value| b.as_f64().is_some_and(|b| (a - b).abs() < 1e-6);
    let bees = layouts::tcp_bees(Some(&sample));
    let ours: Vec<Value> = bees
        .iter()
        .map(|(id, g)| json!({"id": id, "group": g}))
        .collect();
    assert_eq!(json!(ours), theirs["bees"]);
    for (i, s) in ["", "a", "77x", "ç漢", "😀"].iter().enumerate() {
        assert!(close(layouts::hash01(s), &theirs["hash"][i]), "{s}");
    }
    for (i, (w, h)) in [(300.0, 80.0), (40.0, 10.0)].iter().enumerate() {
        let (points, labels) = layouts::beeswarm(&bees, *w, *h);
        let t = &theirs["layouts"][i];
        assert_eq!(points.len(), t["points"].as_array().unwrap().len());
        for (p, q) in points.iter().zip(t["points"].as_array().unwrap()) {
            assert!(
                close(p.x, &q[0])
                    && close(p.y, &q[1])
                    && close(p.r, &q[2])
                    && json!(p.group) == q[3]
                    && json!(p.id) == q[4],
                "{q}"
            );
        }
        assert_eq!(labels.len(), t["labels"].as_array().unwrap().len());
        for ((g, x, mw), q) in labels.iter().zip(t["labels"].as_array().unwrap()) {
            assert!(
                json!(g) == q[0] && close(*x, &q[1]) && close(*mw, &q[2]),
                "{q}"
            );
        }
    }
    let series = layouts::netdev_radar(ifaces.as_array().unwrap());
    let ours: Vec<Value> = series
        .iter()
        .map(|(id, v)| json!({"id": id, "rx": v[0], "tx": v[1], "packets": v[2], "drops": v[3], "errs": v[4]}))
        .collect();
    for (o, t) in ours.iter().zip(theirs["series"].as_array().unwrap()) {
        for k in ["rx", "tx", "packets", "drops", "errs"] {
            assert!(close(o[k].as_f64().unwrap(), &t[k]), "{k} {t}");
        }
        assert_eq!(o["id"], t["id"]);
    }
    for (i, (w, h)) in [(240.0, 160.0), (20.0, 20.0)].iter().enumerate() {
        let ((cx, cy), spokes, polygons, labels) =
            layouts::radar(&series, layouts::NETDEV_AXES, *w, *h);
        let t = &theirs["radars"][i];
        assert_eq!(spokes.len(), t["spokes"].as_array().unwrap().len());
        for (s, q) in spokes.iter().zip(t["spokes"].as_array().unwrap()) {
            assert!(
                json!(s.label) == q[0]
                    && close(s.end.0, &q[1])
                    && close(s.end.1, &q[2])
                    && close(s.at.0, &q[3])
                    && close(s.at.1, &q[4]),
                "{q}"
            );
            assert!(
                json!(s.align) == q[5]
                    && json!(s.baseline) == q[6]
                    && close(cx, &q[7])
                    && close(cy, &q[8]),
                "{q}"
            );
        }
        assert_eq!(polygons.len(), t["polygons"].as_array().unwrap().len());
        for ((id, pts), q) in polygons.iter().zip(t["polygons"].as_array().unwrap()) {
            assert_eq!(json!(id), q[0]);
            for ((x, y), p) in pts.iter().zip(q[1].as_array().unwrap()) {
                assert!(close(*x, &p[0]) && close(*y, &p[1]), "{p}");
            }
        }
        assert_eq!(labels.len(), t["labels"].as_array().unwrap().len());
        for ((id, x, y), q) in labels.iter().zip(t["labels"].as_array().unwrap()) {
            assert!(
                json!(id) == q[0] && close(*x, &q[1]) && close(*y, &q[2]),
                "{q}"
            );
        }
    }
}

#[test]
fn the_parallel_coordinates_are_charts_js() {
    // Rows with every field the chart reads (`fds` too), more than a short chart keeps, and some
    // without one.
    let rows: Vec<Value> = (0..30u64)
        .map(|i| {
            let n = i as f64;
            json!({"pid": 100 + i, "cpu": (n * 7.0) % 60.0, "rssKb": 1000.0 + n * n * 50.0, "fds": if i == 4 { Value::Null } else { json!(3 + i % 9) },
                "threads": 1 + i % 5, "nice": if i == 9 { json!("x") } else { json!(i as i64 % 3 - 1) }})
        })
        .collect();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const C = load("services/Charts.js");
const rows = C.processParallel({rows});
const axes = ["cpu", "rss", "fds", "threads", "nice"];
const at = (w, h) => {{ const l = C.parallelLayout(rows, axes, w, h); return {{ lines: l.polylines.map((p) => [p.id, p.points]), rails: l.rails.map((r) => [r.label, r.x0, r.y0, r.align]) }}; }};
process.stdout.write(JSON.stringify({{ rows, layouts: [at(300, 120), at(300, 30), at(10, 4)] }}));
"#,
        rows = json!(rows)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let close = |a: f64, b: &Value| b.as_f64().is_some_and(|b| (a - b).abs() < 1e-6);
    let ours = layouts::process_parallel(&rows);
    let theirs_rows = theirs["rows"].as_array().unwrap();
    assert_eq!(ours.len(), theirs_rows.len());
    for ((id, v), t) in ours.iter().zip(theirs_rows) {
        assert_eq!(json!(id), t["id"]);
        for (k, key) in ["cpu", "rss", "fds", "threads", "nice"].iter().enumerate() {
            assert!(close(v[k], &t[*key]), "{key} {t}");
        }
    }
    for (i, (w, h)) in [(300.0, 120.0), (300.0, 30.0), (10.0, 4.0)]
        .iter()
        .enumerate()
    {
        let t = &theirs["layouts"][i];
        let lines = layouts::parallel_lines(&ours, 5, *w, *h);
        assert_eq!(lines.len(), t["lines"].as_array().unwrap().len(), "{w}x{h}");
        for ((id, pts), q) in lines.iter().zip(t["lines"].as_array().unwrap()) {
            assert_eq!(json!(id), q[0]);
            for ((x, y), p) in pts.iter().zip(q[1].as_array().unwrap()) {
                assert!(close(*x, &p[0]) && close(*y, &p[1]), "{p}");
            }
        }
        let rails = layouts::parallel_rails(layouts::PARALLEL_AXES, *w, *h);
        assert_eq!(rails.len(), t["rails"].as_array().unwrap().len());
        for ((label, x, top, align), q) in rails.iter().zip(t["rails"].as_array().unwrap()) {
            assert!(
                json!(label) == q[0]
                    && close(*x, &q[1])
                    && close(*top, &q[2])
                    && json!(align) == q[3],
                "{q}"
            );
        }
    }
}
