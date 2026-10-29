//! `cpu_charts.rs` is `services/Charts.js`: the CPU page's IRQ heatmap, PSI ridgeline,
//! core-frequency violins and softirq rose, from the same rows and at a few sizes, to within a
//! thousandth of a pixel. Without `node` the test says so and stops.

mod common;

use atmos_ui::cpu_charts::{self, Label, Rates};
use atmos_ui::stats;
use common::qml;
use serde_json::{Map, Value, json};

fn history() -> Vec<Value> {
    let mut h = Vec::new();
    for i in 0..6u64 {
        let f = i as f64;
        let cpus: Vec<Value> = (0..12u64)
            .map(|c| {
                let freq = match (c, i) {
                    (11, _) => json!(null),
                    (3, _) => json!(2400),
                    _ => json!(800 + (c * 97 + i * i * 131) % 3200),
                };
                json!({"id": if c == 10 { json!(10) } else { json!(c) }, "freqMhz": freq, "idle": 1, "total": 2})
            })
            .collect();
        let irq_rows = vec![
            json!({"id": "0", "values": [100 + i * 40, 3, 0, 9 + i]}),
            json!({"id": "NMI", "values": [7, 7, 7, 7]}),
            json!({"id": "LOC", "values": [1000 + i * i * 300, 900 + i * 10, 50, 2]}),
            json!({"id": "RES", "values": [5 + i, 4, 3 + 2 * i, 2]}),
            json!({"id": "CAL", "values": [0, 0, 0, i]}),
        ];
        let mut rows = irq_rows.clone();
        if i == 5 {
            // A row the kernel lists at another place, and one it dropped, are read by id.
            rows.swap(0, 2);
            rows.remove(3);
        }
        let soft = vec![
            json!({"id": "HI", "values": [1 + i, 0]}),
            json!({"id": "TIMER", "values": [100 + i * 50, 80 + i * 20]}),
            json!({"id": "NET_RX", "values": [3, 3 + i * i]}),
            json!({"id": "RCU", "values": [9, 9]}),
        ];
        let psi = if i == 2 {
            json!(null)
        } else {
            json!({"cpu": f * 1.5, "memory": if i == 3 { json!(null) } else { json!(0.2 * f) }, "io": 4.0 - f * 0.5})
        };
        let raw = json!({
            "cpus": cpus, "psi": psi,
            "interrupts": {"cpus": 4, "rows": rows},
            "softirqs": {"cpus": 2, "rows": soft},
        });
        h = stats::push(&h, &stats::parse(&raw).unwrap(), 2000.0 * (f + 1.0));
    }
    h
}

/// Whether `ours` is `theirs` for every field it has, numbers to within a thousandth.
fn near(ours: &Value, theirs: &Value, at: &str) {
    match (ours, theirs) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            assert!((a - b).abs() < 1e-3, "{at}: {a} != {b}");
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{at}: length");
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                near(x, y, &format!("{at}[{i}]"));
            }
        }
        (Value::Object(a), Value::Object(_)) => {
            for (k, v) in a {
                near(v, &theirs[k], &format!("{at}.{k}"));
            }
        }
        _ => assert_eq!(ours, theirs, "{at}"),
    }
}

fn label(l: &Label) -> Map<String, Value> {
    let v = json!({"id": l.id, "label": l.label, "lx": l.lx, "ly": l.ly, "align": l.align,
        "baseline": l.baseline, "maxW": l.max_w});
    v.as_object().unwrap().clone()
}

fn rates(rows: &[Rates]) -> Value {
    json!(
        rows.iter()
            .map(|r| json!({"id": r.id, "values": r.values}))
            .collect::<Vec<_>>()
    )
}

fn series(s: &[cpu_charts::Series]) -> Value {
    json!(
        s.iter()
            .map(|(id, v)| json!({"id": id, "values": v}))
            .collect::<Vec<_>>()
    )
}

fn points(p: &[(f64, f64)]) -> Value {
    json!(p.iter().map(|(x, y)| json!([x, y])).collect::<Vec<_>>())
}

fn heatmap(m: &[Rates], w: f64, h: f64) -> Value {
    let (cells, labels) = cpu_charts::heatmap_layout(m, w, h);
    json!({
        "cells": cells.iter().map(|c| json!({"x": c.x, "y": c.y, "w": c.w, "h": c.h, "value": c.value,
            "fill": c.fill, "row": c.row, "col": c.col})).collect::<Vec<_>>(),
        "labels": labels.iter().map(label).collect::<Vec<_>>(),
    })
}

fn ridgeline(s: &[cpu_charts::Series], w: f64, h: f64) -> Value {
    let (ridges, labels) = cpu_charts::ridgeline_layout(s, w, h);
    json!({
        "ridges": ridges.iter().map(|r| json!({"id": r.id, "points": points(&r.points),
            "baseline": r.baseline})).collect::<Vec<_>>(),
        "labels": labels.iter().map(label).collect::<Vec<_>>(),
    })
}

fn violins(s: &[cpu_charts::Series], w: f64, h: f64) -> Value {
    json!(
        cpu_charts::violin_paths(s, w, h)
            .iter()
            .map(|v| {
                let mut o = label(&v.label);
                o.insert("points".into(), points(&v.points));
                Value::Object(o)
            })
            .collect::<Vec<_>>()
    )
}

fn rose(items: &[(Value, f64)], w: f64, h: f64) -> Value {
    json!(
        cpu_charts::nightingale_wedges(items, w, h)
            .iter()
            .map(|wd| {
                let mut o = label(&wd.label);
                for (k, v) in [
                    ("value", wd.value),
                    ("cx", wd.cx),
                    ("cy", wd.cy),
                    ("r", wd.r),
                    ("start", wd.start),
                    ("end", wd.end),
                ] {
                    o.insert(k.into(), json!(v));
                }
                Value::Object(o)
            })
            .collect::<Vec<_>>()
    )
}

#[test]
fn the_cpu_charts_are_charts_js() {
    let h = history();
    let edges = json!([
        [null, {"rows": [{"id": "a", "values": [1]}]}],
        [{"rows": [{"id": "a", "values": [5, "x"]}]}, {"rows": [{"id": "a", "values": [5, 2]}]}],
        [{"rows": [null, {"id": "b", "values": [1, 9]}, {"id": "a", "values": [0]}]},
         {"rows": [null, {"id": "a", "values": [3, 4]}, {"values": [1]}, {"id": "b", "values": [2, 8]}, {"id": "c", "values": []}]}],
        [{"rows": [{"id": "a", "values": [1, 1]}]}, {"rows": [{"id": "a", "values": [1, 1]}]}],
    ]);
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("cpu_charts_parity.json");
    std::fs::write(&input, json!({"h": h, "edges": edges}).to_string()).unwrap();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const C = load("services/Charts.js");
const {{ h, edges }} = JSON.parse(require("fs").readFileSync({input}, "utf8"));
const irq = C.irqRateMatrix(h[4].interrupts, h[5].interrupts);
const psi = C.psiRidges(h);
const freq = C.freqViolins(h);
const soft = C.softirqWedges(h[4].softirqs, h[5].softirqs);
const sizes = [[300, 80], [20, 3], [40, 10], [2, 1], [600, 140]];
process.stdout.write(JSON.stringify({{
  irq, psi, freq, soft,
  heat: sizes.map(([w, hh]) => C.heatmapLayout(irq, w, hh)),
  ridge: sizes.map(([w, hh]) => C.ridgelineLayout(psi, w, hh)),
  violin: sizes.map(([w, hh]) => C.violinPaths(freq, w, hh)),
  rose: sizes.map(([w, hh]) => C.nightingaleWedges(soft, w, hh)),
  irqEdges: edges.map(([a, b]) => C.irqRateMatrix(a, b)),
  softEdges: edges.map(([a, b]) => C.softirqWedges(a, b)),
  kde: [C.kdeAt([1, 2, 3], 2, 0.5), C.kdeAt([800, 2400, 2401], 1500, 133.3)],
}}));
"#,
        input = json!(input.to_string_lossy()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let irq = cpu_charts::irq_rate_matrix(h[4].get("interrupts"), h[5].get("interrupts"));
    let psi = cpu_charts::psi_ridges(&h);
    let freq = cpu_charts::freq_violins(&h);
    let soft = cpu_charts::softirq_wedges(h[4].get("softirqs"), h[5].get("softirqs"));
    assert!(!irq.is_empty() && freq.len() == 11 && soft.len() == 4);
    near(&rates(&irq), &theirs["irq"], "irq");
    near(&series(&psi), &theirs["psi"], "psi");
    near(&series(&freq), &theirs["freq"], "freq");
    let soft_json = json!(
        soft.iter()
            .map(|(id, v)| json!({"id": id, "value": v}))
            .collect::<Vec<_>>()
    );
    near(&soft_json, &theirs["soft"], "soft");
    let sizes = [
        (300.0, 80.0),
        (20.0, 3.0),
        (40.0, 10.0),
        (2.0, 1.0),
        (600.0, 140.0),
    ];
    for (i, (w, hh)) in sizes.iter().enumerate() {
        let at = format!("{w}x{hh}");
        near(
            &heatmap(&irq, *w, *hh),
            &theirs["heat"][i],
            &format!("heat {at}"),
        );
        near(
            &ridgeline(&psi, *w, *hh),
            &theirs["ridge"][i],
            &format!("ridge {at}"),
        );
        near(
            &violins(&freq, *w, *hh),
            &theirs["violin"][i],
            &format!("violin {at}"),
        );
        near(
            &rose(&soft, *w, *hh),
            &theirs["rose"][i],
            &format!("rose {at}"),
        );
    }
    assert!(!theirs["heat"][0]["cells"].as_array().unwrap().is_empty());
    assert!(!theirs["violin"][0].as_array().unwrap().is_empty());
    for (i, e) in edges.as_array().unwrap().iter().enumerate() {
        let (a, b) = (Some(&e[0]), Some(&e[1]));
        near(
            &rates(&cpu_charts::irq_rate_matrix(a, b)),
            &theirs["irqEdges"][i],
            &format!("irq edge {i}"),
        );
        let soft: Vec<Value> = cpu_charts::softirq_wedges(a, b)
            .iter()
            .map(|(id, v)| {
                let mut o = Map::new();
                if !id.is_null() {
                    o.insert("id".into(), id.clone());
                }
                o.insert("value".into(), json!(v));
                Value::Object(o)
            })
            .collect();
        near(
            &json!(soft),
            &theirs["softEdges"][i],
            &format!("soft edge {i}"),
        );
    }
    near(
        &json!([
            cpu_charts::kde_at(&[1.0, 2.0, 3.0], 2.0, 0.5),
            cpu_charts::kde_at(&[800.0, 2400.0, 2401.0], 1500.0, 133.3)
        ]),
        &theirs["kde"],
        "kde",
    );
}
