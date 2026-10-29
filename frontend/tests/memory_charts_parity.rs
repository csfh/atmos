//! `treemap.rs` and `memory_charts.rs` are `services/Charts.js`: the Memory page's buddy
//! horizon, RSS treemap (with its focus, hit and zoomed view), meminfo Sankey, cgroup sunburst
//! and slab icicle, from the same rows and at a few sizes, to within a thousandth of a pixel; and
//! the colour an id takes is `PrefsChart.qml`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::cpu_charts::Label;
use atmos_ui::treemap::{self, Rect, View};
use atmos_ui::{memory_charts, stats};
use common::qml;
use serde_json::{Map, Value, json};

fn history() -> Vec<Value> {
    let mut h = Vec::new();
    for i in 0..5u64 {
        let f = i as f64;
        let procs = json!([
            {"pid": 1, "ppid": 0, "comm": "systemd", "rssKb": 12000 + i * 10, "cpu": 0, "uid": 0},
            {"pid": 400, "ppid": 1, "comm": "dbus", "rssKb": 3000, "cpu": 0, "uid": 0},
            {"pid": 900, "ppid": 1, "comm": "Hyprland", "rssKb": 250000 + i * 999, "cpu": 2, "uid": 1000},
            {"pid": 950, "ppid": 900, "comm": "kitty", "rssKb": 80000, "cpu": 1, "uid": 1000},
            {"pid": 951, "ppid": 950, "comm": "zsh", "rssKb": 6000, "cpu": 0, "uid": 1000},
            {"pid": 960, "ppid": 900, "comm": "firefox", "rssKb": 600000, "cpu": 9, "uid": 1000},
            {"pid": 961, "ppid": 960, "comm": "", "rssKb": 300000, "cpu": 3, "uid": 1000},
            {"pid": 962, "ppid": 960, "comm": "Web Content", "rssKb": 220000, "cpu": 1, "uid": 1000},
            {"pid": 3000, "ppid": 2999, "comm": "orphan", "rssKb": 4000, "cpu": 0, "uid": 1000},
            {"pid": 3001, "ppid": 3001, "comm": "selfish", "rssKb": 500, "cpu": 0, "uid": 1000},
            {"pid": 3002, "ppid": 1, "comm": "idle", "rssKb": 0, "cpu": 0, "uid": 1000},
        ]);
        let cgroups = json!({"id": "/", "children": [
            {"id": "user.slice", "value": 900000 + i * 1000, "children": [
                {"id": "user-1000.slice", "value": 800000, "children": [
                    {"id": "app.slice", "value": 600000},
                    {"id": "session.slice", "value": 150000},
                ]},
            ]},
            {"id": "system.slice", "value": 120000, "children": [
                {"id": "dbus.service", "value": 3000},
                {"id": "x.service", "value": 0},
            ]},
            {"id": "init.scope", "value": 12000},
        ]});
        let slabs = json!([
            {"name": "kmalloc-64", "active": 9000 + i, "num": 9100},
            {"name": "kmalloc-128", "active": 4000, "num": 4100},
            {"name": "dentry", "active": 30000, "num": 31000},
            {"name": "inode_cache", "active": 12000, "num": 12500},
            {"name": "ext4_inode_cache", "active": 0, "num": 10},
            {"name": "ext4_extent_status", "active": 700, "num": 800},
            {"name": "9p-fcall", "active": 5, "num": 5},
            {"name": "", "active": 99, "num": 99},
        ]);
        let raw = json!({
            "memTotal": 16_000_000, "memUsed": 6_000_000 + i * 1000, "memAvail": 9_000_000,
            "memAnon": 3_000_000 + i * 500, "memFile": 2_500_000, "memSlab": 400_000,
            "memPageTables": 60_000, "memKernelStack": 20_000, "memShared": 0, "swapUsed": 1_000,
            "buddy": if i == 2 { json!([]) } else { json!([120.0 + f * 10.0, 80, 40, 9, 3, 1]) },
            "processes": procs, "cgroups": cgroups, "slabs": slabs,
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

/// A caption's fields, laid flat on its mark as the JavaScript lays them, or `label: null`.
fn flat(o: &mut Map<String, Value>, l: Option<&Label>) {
    match l {
        None => {
            o.insert("label".into(), Value::Null);
        }
        Some(l) => {
            for (k, v) in [
                ("label", json!(l.label)),
                ("lx", json!(l.lx)),
                ("ly", json!(l.ly)),
                ("align", json!(l.align)),
                ("baseline", json!(l.baseline)),
            ] {
                o.insert(k.into(), v);
            }
            if l.id != "__crumb" {
                o.insert("maxW".into(), json!(l.max_w));
            }
        }
    }
}

fn label(l: &Label) -> Value {
    let mut o = Map::new();
    o.insert("id".into(), json!(l.id));
    flat(&mut o, Some(l));
    Value::Object(o)
}

fn rects(rs: &[Rect]) -> Value {
    json!(
        rs.iter()
            .map(|r| {
                let mut o = json!({"x": r.x, "y": r.y, "w": r.w, "h": r.h, "id": r.id,
                    "value": r.value, "depth": r.depth})
                .as_object()
                .unwrap()
                .clone();
                flat(&mut o, r.label.as_ref());
                Value::Object(o)
            })
            .collect::<Vec<_>>()
    )
}

fn points(p: &[(f64, f64)]) -> Value {
    json!(p.iter().map(|(x, y)| json!([x, y])).collect::<Vec<_>>())
}

const SIZES: [(f64, f64); 5] = [
    (300.0, 80.0),
    (600.0, 140.0),
    (40.0, 12.0),
    (12.0, 300.0),
    (1.0, 1.0),
];

#[test]
fn the_memory_charts_are_charts_js() {
    let h = history();
    let latest = h.last().unwrap();
    let foci = ["", "1", "900", "960:self", "9999"];
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("memory_charts_parity.json");
    std::fs::write(
        &input,
        json!({"h": h, "sizes": SIZES, "foci": foci}).to_string(),
    )
    .unwrap();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const C = load("services/Charts.js");
const {{ h, sizes, foci }} = JSON.parse(require("fs").readFileSync({input}, "utf8"));
const latest = h[h.length - 1];
const buddy = C.buddySeries(h);
const rss = C.rssTree(latest.processes);
const sk = C.meminfoSankey(latest);
const slab = C.slabTree(latest.slabs);
const views = [];
let v = C.clampViewPan(1, 0, 0, 300, 80);
views.push(v);
v = C.zoomView(v.scale, v.x, v.y, 100, 40, 1.12, 300, 80); views.push(v);
v = C.zoomView(v.scale, v.x, v.y, 10, 10, 3, 300, 80); views.push(v);
v = C.clampViewPan(v.scale, v.x + 50, v.y - 500, 300, 80); views.push(v);
v = C.zoomView(v.scale, v.x, v.y, 0, 0, 1 / 50, 300, 80); views.push(v);
v = C.zoomView(v.scale, v.x, v.y, 150, 40, 1000, 300, 80); views.push(v);
const tone = (s) => {{ let t = 0; for (let n = 0; n < s.length; n++) t = (t * 33 + s.charCodeAt(n)) >>> 0; return t; }};
process.stdout.write(JSON.stringify({{
  buddy, rss, sankey: sk, slab,
  horizon: sizes.map(([w, hh]) => [1, 3, 0].map((b) => C.horizonLayout(buddy, w, hh, b))),
  treemap: sizes.map(([w, hh]) => foci.map((f) => C.treemapLayout(rss, w, hh, f))),
  sankeyLayout: sizes.map(([w, hh]) => C.sankeyLayout(sk.nodes, sk.links, w, hh)),
  sunburst: sizes.map(([w, hh]) => C.sunburstLayout(latest.cgroups, w, hh)),
  icicle: sizes.map(([w, hh]) => C.icicleRects(slab, w, hh)),
  parents: ["", "1", "900", "961", "rss", "9999"].map((id) => C.treemapParentId(rss, id)),
  focusIds: ["960:self", "960", ""].map(C.treemapFocusId),
  views,
  mapped: views.map((vv) => C.mapViewScene(C.treemapLayout(rss, 300, 80, "900"), vv.scale, vv.x, vv.y)),
  hits: [[5, 5], [150, 40], [299, 79], [-1, 3], [200, 20]].map(([x, y]) => {{ const r = C.treemapHit(C.treemapLayout(rss, 300, 80, "").rects, x, y); return r ? r.id : null; }}),
  tones: ["", "firefox", "user.slice", "Ünïcødé"].map(tone),
}}));
"#,
        input = json!(input.to_string_lossy()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };

    let buddy = memory_charts::buddy_series(&h);
    near(&json!(buddy), &theirs["buddy"], "buddy");
    let procs = latest["processes"].as_array().unwrap();
    let rss = treemap::rss_tree(procs).unwrap();
    near(&rss, &theirs["rss"], "rss");
    let (nodes, flows) = memory_charts::meminfo_sankey(Some(latest));
    let sk = json!({
        "nodes": nodes.iter().map(|(id, l, c)| json!({"id": id, "label": l, "col": c})).collect::<Vec<_>>(),
        "links": flows.iter().map(|(s, t, v)| json!({"source": s, "target": t, "value": v})).collect::<Vec<_>>(),
    });
    near(&sk, &theirs["sankey"], "sankey");
    let slab = memory_charts::slab_tree(latest["slabs"].as_array().unwrap());
    near(&json!(slab), &theirs["slab"], "slab");
    let cgroups = latest.get("cgroups");

    for (i, (w, hh)) in SIZES.iter().enumerate() {
        let at = format!("{w}x{hh}");
        for (j, bands) in [1usize, 3, 0].iter().enumerate() {
            let (bs, labels) = memory_charts::horizon_layout(&buddy, *w, *hh, *bands);
            let ours = json!({
                "bands": bs.iter().map(|b| json!({"id": b.layer.to_string(), "layer": b.layer, "fill": b.fill,
                    "ceiling": b.ceiling, "points": points(&b.points)})).collect::<Vec<_>>(),
                "labels": labels.iter().map(label).collect::<Vec<_>>(),
            });
            near(
                &ours,
                &theirs["horizon"][i][j],
                &format!("horizon {at} {bands}"),
            );
        }
        for (j, focus) in foci.iter().enumerate() {
            let (rs, labels) = treemap::layout(Some(&rss), *w, *hh, focus);
            let ours = json!({"rects": rects(&rs), "labels": labels.iter().map(label).collect::<Vec<_>>()});
            near(
                &ours,
                &theirs["treemap"][i][j],
                &format!("treemap {at} {focus:?}"),
            );
        }
        let (blocks, links) = memory_charts::sankey_layout(&nodes, &flows, *w, *hh);
        let ours = json!({
            "nodes": blocks.iter().map(|b| {
                let mut o = json!({"id": b.label.id, "x": b.x, "y": b.y, "w": b.w, "h": b.h, "value": b.value})
                    .as_object().unwrap().clone();
                flat(&mut o, Some(&b.label));
                Value::Object(o)
            }).collect::<Vec<_>>(),
            "links": links.iter().map(|l| json!({"source": l.source, "target": l.target, "value": l.value,
                "width": l.width, "x0": l.x0, "y0": l.y0, "x1": l.x1, "y1": l.y1})).collect::<Vec<_>>(),
        });
        near(&ours, &theirs["sankeyLayout"][i], &format!("sankey {at}"));
        let arcs = memory_charts::sunburst_arcs(cgroups, *w, *hh);
        let ours = json!({
            "arcs": arcs.iter().map(|a| {
                let mut o = json!({"id": a.id, "value": a.value, "depth": a.depth, "cx": a.cx, "cy": a.cy,
                    "innerR": a.inner, "outerR": a.outer, "start": a.start, "end": a.end})
                    .as_object().unwrap().clone();
                flat(&mut o, a.label.as_ref());
                Value::Object(o)
            }).collect::<Vec<_>>(),
            "labels": arcs.iter().filter_map(|a| a.label.as_ref()).map(label).collect::<Vec<_>>(),
        });
        near(&ours, &theirs["sunburst"][i], &format!("sunburst {at}"));
        let ice = memory_charts::icicle_rects(slab.as_ref(), *w, *hh);
        near(&rects(&ice), &theirs["icicle"][i], &format!("icicle {at}"));
    }
    assert!(
        !theirs["treemap"][0][2]["labels"]
            .as_array()
            .unwrap()
            .is_empty(),
        "a focus has its crumb"
    );
    assert!(
        !theirs["sunburst"][1]["labels"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let parents: Vec<String> = ["", "1", "900", "961", "rss", "9999"]
        .iter()
        .map(|id| treemap::parent_id(&rss, id))
        .collect();
    near(&json!(parents), &theirs["parents"], "parents");
    let ids: Vec<&str> = ["960:self", "960", ""]
        .into_iter()
        .map(treemap::focus_id)
        .collect();
    near(&json!(ids), &theirs["focusIds"], "focus ids");

    let mut views = vec![treemap::clamp_pan(1.0, 0.0, 0.0, 300.0, 80.0)];
    let step = |v: View| -> Vec<View> {
        let a = treemap::zoom(v, 100.0, 40.0, 1.12, 300.0, 80.0);
        let b = treemap::zoom(a, 10.0, 10.0, 3.0, 300.0, 80.0);
        let c = treemap::clamp_pan(b.scale, b.x + 50.0, b.y - 500.0, 300.0, 80.0);
        let d = treemap::zoom(c, 0.0, 0.0, 1.0 / 50.0, 300.0, 80.0);
        let e = treemap::zoom(d, 150.0, 40.0, 1000.0, 300.0, 80.0);
        vec![a, b, c, d, e]
    };
    views.extend(step(views[0]));
    let ours: Vec<Value> = views
        .iter()
        .map(|v| json!({"scale": v.scale, "x": v.x, "y": v.y}))
        .collect();
    near(&json!(ours), &theirs["views"], "views");
    let (rs, labels) = treemap::layout(Some(&rss), 300.0, 80.0, "900");
    for (i, v) in views.iter().enumerate() {
        let (rs, labels) = treemap::in_view(&rs, &labels, *v);
        let ours =
            json!({"rects": rects(&rs), "labels": labels.iter().map(label).collect::<Vec<_>>()});
        near(&ours, &theirs["mapped"][i], &format!("mapped {i}"));
    }
    let (rs, _) = treemap::layout(Some(&rss), 300.0, 80.0, "");
    let hits: Vec<Option<String>> = [
        (5.0, 5.0),
        (150.0, 40.0),
        (299.0, 79.0),
        (-1.0, 3.0),
        (200.0, 20.0),
    ]
    .iter()
    .map(|(x, y)| treemap::hit(&rs, *x, *y).map(|r| r.id.clone()))
    .collect();
    near(&json!(hits), &theirs["hits"], "hits");

    let tones: Vec<usize> = ["", "firefox", "user.slice", "Ünïcødé"]
        .into_iter()
        .map(memory_charts::tone_id)
        .collect();
    near(&json!(tones), &theirs["tones"], "tones");
    let chart = std::fs::read_to_string(qml::repo().join("components/PrefsChart.qml")).unwrap();
    assert!(chart.contains("for (n = 0; n < s.length; n++) h = (h * 33 + s.charCodeAt(n)) >>> 0"));
}
