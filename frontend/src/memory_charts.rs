//! The Memory page's charts besides the treemap (`treemap.rs`), laid out as `services/Charts.js`
//! lays them out: the buddy horizon (`buddySeries`, `horizonBands`, `horizonLayout`), the
//! meminfo Sankey (`meminfoSankey`, `sankeyLayout`), the cgroup sunburst (`sunburstArcs`,
//! `sunburstLayout`) and the slab icicle (`slabTree`, `icicleRects`), with `PrefsChart.qml`'s
//! colour for an id (`toneId`). `tests/memory_charts_parity.rs` runs these against the
//! JavaScript.

use std::f64::consts::PI;

use serde_json::{Value, json};

use crate::charts::axis_band;
use crate::cpu_charts::Label;
use crate::js;
use crate::stats::{finite, non_neg};
use crate::treemap::{Rect, node_value};

/// `toneId(id)`: the series an id is drawn in, the same for the same id.
pub fn tone_id(id: &str) -> usize {
    id.encode_utf16()
        .fold(0u32, |h, c| h.wrapping_mul(33).wrapping_add(u32::from(c))) as usize
}

/// `buddySeries(history)`: each row's free pages, weighted by order; none for no free pages.
pub fn buddy_series(history: &[Value]) -> Vec<Option<f64>> {
    history
        .iter()
        .map(|s| {
            let orders = s
                .get("buddy")
                .and_then(Value::as_array)
                .map_or(&[][..], Vec::as_slice);
            let t: f64 = orders
                .iter()
                .enumerate()
                .filter_map(|(j, o)| finite(Some(o)).map(|n| n * 2f64.powi(j as i32)))
                .sum();
            (t > 0.0).then_some(t)
        })
        .collect()
}

/// A horizon band: its layer, its fill from 0 to 1, the value at its top, and its line.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    pub layer: usize,
    pub fill: f64,
    pub ceiling: f64,
    pub points: Vec<(f64, f64)>,
}

/// `horizonBands(values, width, height, bandCount)`: the series cut into `bands` slices, each
/// drawn to the full height.
pub fn horizon_bands(values: &[Option<f64>], w: f64, h: f64, bands: usize) -> Vec<Band> {
    let bands = if bands < 1 { 3 } else { bands };
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    let nums: Vec<f64> = values.iter().flatten().map(|n| n.max(0.0)).collect();
    let max = nums.iter().copied().fold(0.0, f64::max);
    if nums.len() < 2 || max <= 0.0 {
        return Vec::new();
    }
    let band_h = max / bands as f64;
    (0..bands)
        .map(|k| {
            let points = nums
                .iter()
                .enumerate()
                .map(|(j, n)| {
                    let slice = (n - k as f64 * band_h).clamp(0.0, band_h);
                    (
                        (w * j as f64) / (nums.len() as f64 - 1.0),
                        h - (slice / band_h) * h,
                    )
                })
                .collect();
            Band {
                layer: k,
                fill: (k as f64 + 1.0) / bands as f64,
                ceiling: (k as f64 + 1.0) * band_h,
                points,
            }
        })
        .collect()
}

/// `foldCaption(n)`: a band's top, whole when it is near enough.
fn fold_caption(n: f64) -> String {
    if !n.is_finite() {
        return String::new();
    }
    let r = js::round(n);
    js::string(&js::json_number(if (n - r).abs() < 1e-6 { r } else { n }))
}

/// `horizonLayout(values, width, height, bandCount)`: the bands right of room for a caption
/// per band.
pub fn horizon_layout(
    values: &[Option<f64>],
    w: f64,
    h: f64,
    bands: usize,
) -> (Vec<Band>, Vec<Label>) {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let mut left = axis_band(w).max(88f64.min(js::round(w * 0.34)));
    let mut plot_w = w - left;
    if plot_w <= w * 0.45 {
        left = axis_band(w);
        plot_w = w - left;
    }
    if plot_w <= 0.0 {
        left = 0.0;
        plot_w = w;
    }
    let mut out = horizon_bands(values, plot_w, h, bands);
    let n = out.len() as f64;
    let mut labels = Vec::new();
    for (i, b) in out.iter_mut().enumerate() {
        for p in &mut b.points {
            p.0 += left;
        }
        let ly = (h - ((i as f64 + 0.5) / n) * h).clamp(0.0, h);
        let label = match fold_caption(b.ceiling) {
            l if l.is_empty() => (i + 1).to_string(),
            l => l,
        };
        labels.push(Label {
            id: b.layer.to_string(),
            label,
            lx: left.max(0.0),
            ly,
            align: if left > 0.0 { "right" } else { "left" },
            baseline: "middle",
            max_w: left.max(8.0),
        });
    }
    (out, labels)
}

/// A Sankey node: its id, its words, and its column.
pub type Node = (String, String, usize);
/// A Sankey flow: from, to, and how much.
pub type Flow = (String, String, f64);

/// `meminfoSankey(sample)`: MemTotal flowing into each part of memory that has any.
pub fn meminfo_sankey(sample: Option<&Value>) -> (Vec<Node>, Vec<Flow>) {
    let Some(s) = sample.filter(|s| js::truthy(Some(s))) else {
        return (Vec::new(), Vec::new());
    };
    if !non_neg(s.get("memTotal")).is_some_and(|t| t > 0.0) {
        return (Vec::new(), Vec::new());
    }
    let parts = [
        ("anon", "Anon", "memAnon"),
        ("file", "File", "memFile"),
        ("slab", "Slab", "memSlab"),
        ("pagetables", "PageTables", "memPageTables"),
        ("kernelstack", "KernelStack", "memKernelStack"),
        ("shmem", "Shmem", "memShared"),
        ("swap", "Swap", "swapUsed"),
    ];
    let mut nodes = vec![("total".to_owned(), "MemTotal".to_owned(), 0)];
    let mut flows = Vec::new();
    for (id, label, key) in parts {
        let Some(kb) = non_neg(s.get(key)).filter(|k| *k > 0.0) else {
            continue;
        };
        nodes.push((id.to_owned(), label.to_owned(), 1));
        flows.push(("total".to_owned(), id.to_owned(), kb));
    }
    if flows.is_empty() {
        return (Vec::new(), Vec::new());
    }
    (nodes, flows)
}

/// A Sankey node laid out: where it is, its total, and its caption.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub value: f64,
    pub label: Label,
}

/// A Sankey flow laid out: from where to where, and how wide.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub source: String,
    pub target: String,
    pub value: f64,
    pub width: f64,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// `sankeyLayout(nodes, links, width, height)`: a column of nodes per column, each as tall as
/// what flows through it, and a band per flow.
pub fn sankey_layout(nodes: &[Node], flows: &[Flow], w: f64, h: f64) -> (Vec<Block>, Vec<Link>) {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || nodes.is_empty() || flows.is_empty() {
        return (Vec::new(), Vec::new());
    }
    // `map`, in `Object.keys` order (no id here is an array index).
    let mut map: Vec<(String, String, usize, f64)> = Vec::new();
    let mut cols = 1;
    for (id, label, col) in nodes {
        match map.iter_mut().find(|m| m.0 == *id) {
            Some(m) => *m = (id.clone(), label.clone(), *col, 0.0),
            None => map.push((id.clone(), label.clone(), *col, 0.0)),
        }
        cols = cols.max(col + 1);
    }
    let mut edges: Vec<Flow> = Vec::new();
    for (source, target, value) in flows {
        if !value.is_finite() || *value <= 0.0 {
            continue;
        }
        let (Some(s), Some(t)) = (
            map.iter().position(|m| m.0 == *source),
            map.iter().position(|m| m.0 == *target),
        ) else {
            continue;
        };
        map[s].3 += value;
        map[t].3 += value;
        edges.push((source.clone(), target.clone(), *value));
    }
    if edges.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let node_w = 8f64.max(w * 0.06);
    let mut band = axis_band(w);
    let mut inner_w = w - 2.0 * band;
    if inner_w <= node_w {
        band = 0.0;
        inner_w = w;
    }
    const GAP: f64 = 6.0;
    let last = cols as f64 - 1.0;
    let mut blocks: Vec<(String, Block)> = Vec::new();
    for c in 0..cols {
        let column: Vec<&(String, String, usize, f64)> = map.iter().filter(|m| m.2 == c).collect();
        let total: f64 = column.iter().map(|m| m.3).sum();
        let avail = h - GAP * (column.len() as f64 - 1.0).max(0.0);
        let nx = if cols <= 1 {
            band
        } else {
            band + (c as f64 * (inner_w - node_w)) / last
        };
        let (lx, align, max_w) = if c == 0 {
            (nx, "right", nx.max(8.0))
        } else if c as f64 == last {
            (nx + node_w, "left", (w - (nx + node_w)).max(8.0))
        } else {
            (nx + node_w / 2.0, "center", node_w.max(8.0))
        };
        let mut y = 0.0;
        for (id, label, _, value) in column {
            let nh = if total > 0.0 {
                (value / total) * avail
            } else {
                0.0
            };
            blocks.push((
                id.clone(),
                Block {
                    x: nx,
                    y,
                    w: node_w,
                    h: nh,
                    value: *value,
                    label: Label {
                        id: id.clone(),
                        label: label.clone(),
                        lx,
                        ly: y + nh / 2.0,
                        align,
                        baseline: "middle",
                        max_w,
                    },
                },
            ));
            y += nh + GAP;
        }
    }
    let geom = |id: &str| blocks.iter().find(|b| b.0 == id).map(|b| &b.1);
    let y_of = |id: &str| geom(id).map_or(0.0, |b| b.y);
    edges.sort_by(|p, q| {
        let dy = y_of(&p.0) - y_of(&q.0);
        let dy = if dy != 0.0 {
            dy
        } else {
            y_of(&p.1) - y_of(&q.1)
        };
        dy.partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut out_y: Vec<(String, f64)> = Vec::new();
    let mut in_y: Vec<(String, f64)> = Vec::new();
    let mut links = Vec::new();
    for (source, target, value) in &edges {
        let (Some(a), Some(b)) = (geom(source), geom(target)) else {
            continue;
        };
        let share = |n: &Block| {
            if n.value > 0.0 {
                (value / n.value) * n.h
            } else {
                0.0
            }
        };
        let width = share(a).min(share(b));
        if width.is_nan() || width <= 0.0 {
            continue;
        }
        let at = |list: &mut Vec<(String, f64)>, id: &str, start: f64| -> f64 {
            let i = list.iter().position(|e| e.0 == id).unwrap_or_else(|| {
                list.push((id.to_owned(), start));
                list.len() - 1
            });
            let now = list[i].1;
            list[i].1 += width;
            now
        };
        let (oy, iy) = (at(&mut out_y, source, a.y), at(&mut in_y, target, b.y));
        links.push(Link {
            source: source.clone(),
            target: target.clone(),
            value: *value,
            width,
            x0: a.x + a.w,
            y0: oy + width / 2.0,
            x1: b.x,
            y1: iy + width / 2.0,
        });
    }
    (blocks.into_iter().map(|b| b.1).collect(), links)
}

fn kids(node: &Value) -> &[Value] {
    node.get("children")
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// `treeDepth(node)`: how many rings or rows the tree needs.
fn depth(node: &Value) -> usize {
    1 + kids(node).iter().map(depth).max().unwrap_or(0)
}

fn id_of(node: &Value) -> String {
    node.get("id")
        .filter(|i| !i.is_null())
        .map_or_else(String::new, js::string)
}

/// A sunburst ring segment: its node, its radii and angles, and its caption when it has room.
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    pub id: String,
    pub value: f64,
    pub depth: usize,
    pub cx: f64,
    pub cy: f64,
    pub inner: f64,
    pub outer: f64,
    pub start: f64,
    pub end: f64,
    pub label: Option<Label>,
}

/// `sunburstArcs(tree, width, height)`: a ring per depth, each node's angle its share.
pub fn sunburst_arcs(tree: Option<&Value>, w: f64, h: f64) -> Vec<Arc> {
    let Some(tree) = tree.filter(|t| js::truthy(Some(t))) else {
        return Vec::new();
    };
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    let (cx, cy) = (w / 2.0, h / 2.0);
    let small = w.min(h);
    let mut max_r = small / 2.0 - axis_band(small);
    if max_r <= 0.0 {
        max_r = small / 2.0 - 4.0;
    }
    if max_r <= 0.0 {
        return Vec::new();
    }
    let max_depth = depth(tree);
    let mut out = Vec::new();
    let label_of = |arc: &Arc, has_kids: bool| -> Option<Label> {
        let span = arc.end - arc.start;
        let thick = arc.outer - arc.inner;
        let mid_r = (arc.inner + arc.outer) / 2.0;
        let arc_len = span.abs()
            * if arc.depth == 0 {
                arc.outer.max(1.0)
            } else {
                mid_r
            };
        if arc.id.is_empty() || thick < 10.0 || arc_len < 16.0 {
            return None;
        }
        let mid = arc.start + span / 2.0;
        let (ox, oy) = (mid.cos(), mid.sin());
        let (lx, ly, align, baseline, max_w) = if arc.depth == 0 {
            (cx, cy, "center", "middle", (arc.outer * 1.4).max(8.0))
        } else if arc.depth == max_depth - 1 || !has_kids {
            let r = arc.outer + 3.0;
            let align = if ox > 0.35 {
                "left"
            } else if ox < -0.35 {
                "right"
            } else {
                "center"
            };
            let baseline = if oy > 0.35 {
                "top"
            } else if oy < -0.35 {
                "bottom"
            } else {
                "middle"
            };
            (
                cx + ox * r,
                cy + oy * r,
                align,
                baseline,
                (span.abs() * r).max(8.0),
            )
        } else {
            (
                cx + ox * mid_r,
                cy + oy * mid_r,
                "center",
                "middle",
                arc_len.max(8.0),
            )
        };
        Some(Label {
            id: arc.id.clone(),
            label: arc.id.clone(),
            lx: lx.max(0.0).min(w),
            ly: ly.max(0.0).min(h),
            align,
            baseline,
            max_w,
        })
    };
    fn walk(
        node: &Value,
        depth: usize,
        start: f64,
        end: f64,
        ring: &dyn Fn(usize, f64, f64, &Value) -> Arc,
        out: &mut Vec<Arc>,
    ) {
        if !js::truthy(Some(node)) {
            return;
        }
        let arc = ring(depth, start, end, node);
        let value = arc.value;
        out.push(arc);
        let children = kids(node);
        if children.is_empty() || value.is_nan() || value <= 0.0 {
            return;
        }
        let span = end - start;
        let mut a = start;
        for k in children {
            let v = node_value(k);
            walk(k, depth + 1, a, a + span * (v / value), ring, out);
            a += span * (v / value);
        }
    }
    let ring = |depth: usize, start: f64, end: f64, node: &Value| {
        let mut arc = Arc {
            id: id_of(node),
            value: node_value(node),
            depth,
            cx,
            cy,
            inner: (depth as f64 / max_depth as f64) * max_r,
            outer: ((depth as f64 + 1.0) / max_depth as f64) * max_r,
            start,
            end,
            label: None,
        };
        arc.label = label_of(&arc, !kids(node).is_empty());
        arc
    };
    walk(tree, 0, -PI / 2.0, -PI / 2.0 + PI * 2.0, &ring, &mut out);
    out
}

/// `slabTree(slabs)`: the slab caches with active objects, grouped by the prefix of their name.
pub fn slab_tree(slabs: &[Value]) -> Option<Value> {
    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    for s in slabs {
        let name = s
            .get("name")
            .filter(|n| js::truthy(Some(n)))
            .map_or_else(String::new, js::string);
        if name.is_empty() {
            continue;
        }
        let Some(active) = non_neg(s.get("active")).filter(|a| *a > 0.0) else {
            continue;
        };
        let prefix = match name.split(['-', '_']).next() {
            Some(p) if !p.is_empty() => p.to_owned(),
            _ => name.clone(),
        };
        let leaf = json!({"id": name, "value": js::json_number(active)});
        match groups.iter_mut().find(|g| g.0 == prefix) {
            Some(g) => g.1.push(leaf),
            None => groups.push((prefix, vec![leaf])),
        }
    }
    // `Object.keys` order: array indices ascending, then the rest as they came.
    let index = |k: &str| {
        k.parse::<u32>()
            .ok()
            .filter(|n| *n != u32::MAX && n.to_string() == k)
    };
    let (mut indexed, named): (Vec<_>, Vec<_>) =
        groups.into_iter().partition(|g| index(&g.0).is_some());
    indexed.sort_by_key(|g| index(&g.0));
    let kids: Vec<Value> = indexed
        .into_iter()
        .chain(named)
        .map(|(id, children)| json!({"id": id, "children": children}))
        .collect();
    (!kids.is_empty()).then(|| json!({"id": "slab", "children": kids}))
}

/// `icicleRects(tree, width, height)`: a row per depth, each node as wide as its share.
pub fn icicle_rects(tree: Option<&Value>, w: f64, h: f64) -> Vec<Rect> {
    let Some(tree) = tree.filter(|t| js::truthy(Some(t))) else {
        return Vec::new();
    };
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return Vec::new();
    }
    let row_h = h / depth(tree) as f64;
    fn walk(node: &Value, x: f64, ww: f64, depth: usize, row_h: f64, out: &mut Vec<Rect>) {
        if !js::truthy(Some(node)) || ww.is_nan() || ww <= 0.0 {
            return;
        }
        let value = node_value(node);
        let id = id_of(node);
        let y = depth as f64 * row_h;
        let label = (!id.is_empty() && ww >= 16.0 && row_h >= 10.0).then(|| Label {
            id: id.clone(),
            label: id.clone(),
            lx: x + ww / 2.0,
            ly: y + row_h / 2.0,
            align: "center",
            baseline: "middle",
            max_w: (ww - 2.0).max(8.0),
        });
        out.push(Rect {
            x,
            y,
            w: ww,
            h: row_h,
            id,
            value,
            depth,
            label,
        });
        let children = kids(node);
        if children.is_empty() || value.is_nan() || value <= 0.0 {
            return;
        }
        let mut cx = x;
        for k in children {
            let cw = ww * (node_value(k) / value);
            walk(k, cx, cw, depth + 1, row_h, out);
            cx += cw;
        }
    }
    let mut out = Vec::new();
    walk(tree, 0.0, w, 0, row_h, &mut out);
    out
}
