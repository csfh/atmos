//! The Memory page's RSS treemap and the trees it shares with the sunburst and the icicle, as
//! `services/Charts.js` lays them out: the process tree (`rssTree`), a node's value and contents
//! (`treemapNodeValue`, `treemapContents`), the squarified layout (`squarify`), the focus a
//! press drills into and back out of (`treemapFocusId`, `treemapParentId`, `treemapHit`), and
//! the zoomed and panned view (`clampViewScale`, `clampViewPan`, `zoomView`, `mapViewMark`).
//! A tree is the JavaScript's own shape (`id`, `comm`, `value`, `children`).
//! `tests/memory_charts_parity.rs` runs these against the JavaScript.

use std::borrow::Cow;

use serde_json::{Map, Value, json};

use crate::charts::axis_band;
use crate::cpu_charts::Label;
use crate::js;
use crate::stats::{finite, non_neg};

/// `rssTree(processes)`: the processes with a resident set, each under its parent, a parent's
/// own share as a child of its own (`<pid>:self`).
pub fn rss_tree(processes: &[Value]) -> Option<Value> {
    let mut order: Vec<String> = Vec::new();
    let mut by_pid: std::collections::HashMap<String, Map<String, Value>> = Default::default();
    for row in processes {
        let Some(rss) = non_neg(row.get("rssKb")).filter(|r| *r > 0.0) else {
            continue;
        };
        let Some(pid) = row.get("pid").filter(|p| !p.is_null()) else {
            continue;
        };
        let pid = js::string(pid);
        let ppid = row
            .get("ppid")
            .filter(|p| !p.is_null())
            .map_or_else(|| "0".to_owned(), js::string);
        let comm = row
            .get("comm")
            .filter(|c| js::truthy(Some(c)))
            .map_or_else(|| "unknown".to_owned(), js::string);
        let node = json!({"id": pid, "pid": pid, "ppid": ppid, "comm": comm, "value": js::json_number(rss), "children": []});
        if !by_pid.contains_key(&pid) {
            order.push(pid.clone());
        }
        by_pid.insert(pid, node.as_object().cloned().unwrap_or_default());
    }
    // `Object.keys` order: array indices ascending, then the rest as they came.
    let index = |k: &str| {
        k.parse::<u32>()
            .ok()
            .filter(|n| *n != u32::MAX && n.to_string() == k)
    };
    let (mut keys, named): (Vec<String>, Vec<String>) =
        order.into_iter().partition(|k| index(k).is_some());
    keys.sort_by_key(|k| index(k));
    keys.extend(named);
    let parent_of = |id: &str| -> Option<String> {
        let node = by_pid.get(id)?;
        let ppid = node.get("ppid").and_then(Value::as_str).unwrap_or("");
        (!ppid.is_empty() && ppid != id && by_pid.contains_key(ppid)).then(|| ppid.to_owned())
    };
    let mut kids: std::collections::HashMap<String, Vec<String>> = Default::default();
    let mut roots = Vec::new();
    for k in &keys {
        match parent_of(k) {
            Some(p) => kids.entry(p).or_default().push(k.clone()),
            None => roots.push(k.clone()),
        }
    }
    if roots.is_empty() {
        return None;
    }
    fn build(
        id: &str,
        by_pid: &std::collections::HashMap<String, Map<String, Value>>,
        kids: &std::collections::HashMap<String, Vec<String>>,
        seen: &mut std::collections::HashSet<String>,
    ) -> Value {
        let mut node = by_pid.get(id).cloned().unwrap_or_default();
        seen.insert(id.to_owned());
        let children: Vec<Value> = kids
            .get(id)
            .map(|c| {
                c.iter()
                    .filter(|c| !seen.contains(*c))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
            .iter()
            .map(|c| build(c, by_pid, kids, seen))
            .collect();
        let own = finite(node.get("value")).filter(|v| *v > 0.0);
        if let (false, Some(own)) = (children.is_empty(), own) {
            let mut children = children;
            let comm = node
                .get("comm")
                .filter(|c| js::truthy(Some(c)))
                .cloned()
                .unwrap_or_else(|| Value::from(id));
            children.push(json!({"id": format!("{id}:self"), "comm": comm, "value": js::json_number(own), "children": []}));
            node.insert("children".into(), Value::Array(children));
            node.insert("value".into(), json!(0));
        } else {
            node.insert("children".into(), Value::Array(children));
        }
        Value::Object(node)
    }
    let mut seen = std::collections::HashSet::new();
    let roots: Vec<Value> = roots
        .iter()
        .map(|r| build(r, &by_pid, &kids, &mut seen))
        .collect();
    Some(json!({"id": "rss", "children": roots}))
}

fn children(node: &Value) -> Option<&Vec<Value>> {
    node.get("children").and_then(Value::as_array)
}

fn id_of(node: &Value) -> String {
    node.get("id")
        .filter(|i| !i.is_null())
        .map_or_else(String::new, js::string)
}

/// `treemapNodeValue(node)`: a leaf's value, or its children's together.
pub fn node_value(node: &Value) -> f64 {
    if !js::truthy(Some(node)) {
        return 0.0;
    }
    let v = finite(node.get("value"));
    let kids = children(node);
    if let (Some(v), None) = (v, kids)
        && v > 0.0
    {
        return v;
    }
    if let Some(kids) = kids.filter(|k| !k.is_empty()) {
        let s: f64 = kids.iter().map(node_value).sum();
        if s > 0.0 {
            return s;
        }
    }
    v.filter(|v| *v > 0.0).unwrap_or(0.0)
}

/// `treemapContents(node)`: its children, and its own share as one more when it has one.
fn contents(node: &Value) -> Vec<Cow<'_, Value>> {
    let mut kids: Vec<Cow<Value>> = children(node)
        .map(|k| k.iter().map(Cow::Borrowed).collect())
        .unwrap_or_default();
    if let Some(v) = finite(node.get("value")).filter(|v| *v > 0.0)
        && !kids.is_empty()
    {
        let id = js::string(node.get("id").unwrap_or(&Value::Null));
        let comm = node
            .get("comm")
            .filter(|c| js::truthy(Some(c)))
            .cloned()
            .unwrap_or_else(|| Value::from(id.clone()));
        kids.push(Cow::Owned(json!({"id": format!("{id}:self"), "comm": comm, "value": js::json_number(v), "children": []})));
    }
    kids
}

fn caption(node: &Value, id: &str) -> String {
    if let Some(c) = node.get("comm").filter(|c| js::truthy(Some(c))) {
        return js::string(c);
    }
    id.strip_suffix(":self").unwrap_or(id).to_owned()
}

/// `treemapFocusId(id)`: a rect's id as the node it focuses (its own share is its node).
pub fn focus_id(id: &str) -> &str {
    id.strip_suffix(":self").unwrap_or(id)
}

/// A treemap or icicle rectangle: where it is, its node, and its caption when it has room.
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub id: String,
    pub value: f64,
    pub depth: usize,
    pub label: Option<Label>,
}

fn centred(id: &str, text: String, x: f64, y: f64, w: f64, h: f64) -> Label {
    Label {
        id: id.to_owned(),
        label: text,
        lx: x + w / 2.0,
        ly: y + h / 2.0,
        align: "center",
        baseline: "middle",
        max_w: (w - 2.0).max(8.0),
    }
}

fn worst_aspect(row: &[(&Value, f64)], side: f64) -> f64 {
    if side.is_nan() || side <= 0.0 || row.is_empty() {
        return f64::INFINITY;
    }
    let total: f64 = row.iter().map(|r| r.1).sum();
    let mn = row.iter().map(|r| r.1).fold(f64::INFINITY, f64::min);
    let mx = row.iter().map(|r| r.1).fold(0.0, f64::max);
    if total.is_nan() || total <= 0.0 || mn.is_nan() || mn <= 0.0 {
        return f64::INFINITY;
    }
    let s2 = side * side;
    ((s2 * mx) / (total * total)).max((total * total) / (s2 * mn))
}

#[allow(clippy::too_many_arguments)]
fn layout_row(
    row: &[(&Value, f64)],
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    out: &mut Vec<Rect>,
    depth: usize,
) {
    let total: f64 = row.iter().map(|r| r.1).sum();
    if total.is_nan() || total <= 0.0 {
        return;
    }
    let horizontal = w >= h;
    let mut a = 0.0;
    for (node, value) in row {
        let (nx, ny, rw, rh) = if horizontal {
            let rw = w * (value / total);
            let at = (x + a, y, rw, h);
            a += rw;
            at
        } else {
            let rh = h * (value / total);
            let at = (x, y + a, w, rh);
            a += rh;
            at
        };
        push_rect(node, nx, ny, rw, rh, out, depth);
    }
}

fn push_rect(node: &Value, x: f64, y: f64, w: f64, h: f64, out: &mut Vec<Rect>, depth: usize) {
    let id = id_of(node);
    let text = caption(node, &id);
    let kids = contents(node);
    let label = (kids.is_empty() && !text.is_empty() && w >= 16.0 && h >= 10.0)
        .then(|| centred(&id, text, x, y, w, h));
    out.push(Rect {
        x,
        y,
        w,
        h,
        id,
        value: node_value(node),
        depth,
        label,
    });
    if !kids.is_empty() {
        let kids: Vec<&Value> = kids.iter().map(|k| k.as_ref()).collect();
        squarify(&kids, x, y, w, h, depth + 1, out);
    }
}

/// `squarify(nodes, x, y, w, h, depth, out)`: rows of rectangles kept as near square as the
/// values allow, largest first.
fn squarify(nodes: &[&Value], x: f64, y: f64, w: f64, h: f64, depth: usize, out: &mut Vec<Rect>) {
    if nodes.is_empty() || w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return;
    }
    let mut items: Vec<(&Value, f64)> = nodes
        .iter()
        .map(|n| (*n, node_value(n)))
        .filter(|(_, v)| *v > 0.0)
        .collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    match items.len() {
        0 => return,
        1 => return push_rect(items[0].0, x, y, w, h, out, depth),
        _ => {}
    }
    let (mut x, mut y, mut w, mut h) = (x, y, w, h);
    let mut side = w.min(h);
    let mut row: Vec<(&Value, f64)> = Vec::new();
    let mut rest = items.as_slice();
    while let Some((item, tail)) = rest.split_first() {
        if row.is_empty() {
            row.push(*item);
            rest = tail;
            continue;
        }
        let mut longer = row.clone();
        longer.push(*item);
        if worst_aspect(&longer, side) <= worst_aspect(&row, side) {
            row = longer;
            rest = tail;
            continue;
        }
        let row_val: f64 = row.iter().map(|r| r.1).sum();
        let rest_val: f64 = rest.iter().map(|r| r.1).sum();
        let frac = row_val / (row_val + rest_val);
        if w >= h {
            layout_row(&row, x, y, w * frac, h, out, depth);
            x += w * frac;
            w -= w * frac;
        } else {
            layout_row(&row, x, y, w, h * frac, out, depth);
            y += h * frac;
            h -= h * frac;
        }
        row.clear();
        side = w.min(h);
    }
    if !row.is_empty() {
        layout_row(&row, x, y, w, h, out, depth);
    }
}

/// `findTreeNode(tree, id)`: the node with that id, or the root for none.
pub fn find_node<'a>(tree: &'a Value, id: &str) -> Option<&'a Value> {
    if id.is_empty() || (tree.get("id").is_some_and(|i| !i.is_null()) && id_of(tree) == id) {
        return Some(tree);
    }
    children(tree)?.iter().find_map(|k| find_node(k, id))
}

fn find_parent(node: &Value, id: &str, parent: &str) -> Option<String> {
    if node.get("id").is_some_and(|i| !i.is_null()) && id_of(node) == id {
        return Some(parent.to_owned());
    }
    let own = id_of(node);
    children(node)?
        .iter()
        .find_map(|k| find_parent(k, id, &own))
}

/// `treemapParentId(tree, id)`: the focus a zoom out goes to, none for the top.
pub fn parent_id(tree: &Value, id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    let top = tree
        .get("id")
        .is_some_and(|i| !i.is_null())
        .then(|| id_of(tree));
    match find_parent(tree, id, "") {
        Some(p) if !p.is_empty() && top.as_ref() != Some(&p) => p,
        _ => String::new(),
    }
}

/// `treemapRects(tree, width, height, focusId)`: the focused node's contents, laid out.
pub fn rects(tree: &Value, w: f64, h: f64, focus: &str) -> Vec<Rect> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || !js::truthy(Some(tree)) {
        return Vec::new();
    }
    let root = find_node(tree, focus).unwrap_or(tree);
    let mut out = Vec::new();
    let kids = contents(root);
    let kids: Vec<&Value> = if kids.is_empty() {
        vec![root]
    } else {
        kids.iter().map(|k| k.as_ref()).collect()
    };
    squarify(&kids, 0.0, 0.0, w, h, 0, &mut out);
    out
}

/// `treemapLayout(tree, width, height, focusId)`: the rectangles under a breadcrumb naming the
/// focus, when there is one.
pub fn layout(tree: Option<&Value>, w: f64, h: f64, focus: &str) -> (Vec<Rect>, Vec<Label>) {
    let Some(tree) = tree.filter(|t| js::truthy(Some(t))) else {
        return (Vec::new(), Vec::new());
    };
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let focused = find_node(tree, focus).unwrap_or(tree);
    let crumb = match focused.get("id").filter(|i| !i.is_null()) {
        Some(id) if !focus.is_empty() => {
            match focused.get("comm").filter(|c| js::truthy(Some(c))) {
                Some(c) => format!("{} {}", js::string(c), js::string(id)),
                None => js::string(id),
            }
        }
        _ => String::new(),
    };
    let mut band = if crumb.is_empty() { 0.0 } else { axis_band(h) };
    if band > h * 0.28 {
        band = 14f64.max(js::round(h * 0.2));
    }
    let mut plot_h = h - band;
    if plot_h <= 0.0 {
        band = 0.0;
        plot_h = h;
    }
    let mut out = rects(tree, w, plot_h, focus);
    for r in &mut out {
        r.y += band;
        if let Some(l) = &mut r.label {
            l.ly += band;
        }
    }
    let labels = if crumb.is_empty() {
        Vec::new()
    } else {
        vec![Label {
            id: "__crumb".into(),
            label: crumb,
            lx: 0.0,
            ly: 0.0,
            align: "left",
            baseline: "top",
            max_w: 0.0,
        }]
    };
    (out, labels)
}

/// `treemapHit(rects, x, y)`: the deepest rectangle under a point, the last of those.
pub fn hit(rects: &[Rect], x: f64, y: f64) -> Option<&Rect> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    let mut best: Option<&Rect> = None;
    for r in rects {
        if x < r.x || y < r.y || x > r.x + r.w || y > r.y + r.h {
            continue;
        }
        if best.is_none_or(|b| r.depth >= b.depth) {
            best = Some(r);
        }
    }
    best
}

/// A treemap's view: how far it is zoomed in, and where its top left corner is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub scale: f64,
    pub x: f64,
    pub y: f64,
}

impl Default for View {
    fn default() -> Self {
        View {
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

/// `clampViewScale(s)`: between 1 and 24.
pub fn clamp_scale(s: f64) -> f64 {
    if !s.is_finite() || s <= 0.0 {
        return 1.0;
    }
    s.clamp(1.0, 24.0)
}

fn or_zero(n: f64) -> f64 {
    if n.is_finite() { n } else { 0.0 }
}

/// `clampViewPan(scale, panX, panY, viewW, viewH)`: a view that keeps the chart over the box,
/// or centred in it when it is smaller.
pub fn clamp_pan(scale: f64, x: f64, y: f64, vw: f64, vh: f64) -> View {
    let s = clamp_scale(scale);
    let w = if vw.is_finite() && vw > 0.0 { vw } else { 0.0 };
    let h = if vh.is_finite() && vh > 0.0 { vh } else { 0.0 };
    let along = |pan: f64, view: f64| {
        let sized = view * s;
        if sized <= view {
            (view - sized) / 2.0
        } else {
            let pan = pan.min(0.0);
            if pan + sized < view {
                view - sized
            } else {
                pan
            }
        }
    };
    View {
        scale: s,
        x: along(or_zero(x), w),
        y: along(or_zero(y), h),
    }
}

/// `zoomView(...)`: the view zoomed by `factor` about the point `(cx, cy)` of the box.
pub fn zoom(view: View, cx: f64, cy: f64, factor: f64, vw: f64, vh: f64) -> View {
    let s0 = clamp_scale(view.scale);
    let k = if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    };
    let s1 = clamp_scale(s0 * k);
    let (x, y, px, py) = (or_zero(view.x), or_zero(view.y), or_zero(cx), or_zero(cy));
    let (wx, wy) = ((px - x) / s0, (py - y) / s0);
    clamp_pan(s1, px - wx * s1, py - wy * s1, vw, vh)
}

/// `mapViewMark` over a rectangle: where it is drawn in the view.
pub fn view_rect(r: &Rect, v: View) -> Rect {
    let s = clamp_scale(v.scale);
    let (ox, oy) = (or_zero(v.x), or_zero(v.y));
    Rect {
        x: r.x * s + ox,
        y: r.y * s + oy,
        w: r.w * s,
        h: r.h * s,
        label: r.label.as_ref().map(|l| view_label(l, v)),
        ..r.clone()
    }
}

/// `mapViewMark` over a caption.
pub fn view_label(l: &Label, v: View) -> Label {
    let s = clamp_scale(v.scale);
    Label {
        lx: l.lx * s + or_zero(v.x),
        ly: l.ly * s + or_zero(v.y),
        max_w: l.max_w * s,
        ..l.clone()
    }
}

/// `mapViewScene`: the layout in the view; as it is when the view is the whole chart.
pub fn in_view(rects: &[Rect], labels: &[Label], v: View) -> (Vec<Rect>, Vec<Label>) {
    if clamp_scale(v.scale) == 1.0 && v.x == 0.0 && v.y == 0.0 {
        return (rects.to_vec(), labels.to_vec());
    }
    (
        rects.iter().map(|r| view_rect(r, v)).collect(),
        labels.iter().map(|l| view_label(l, v)).collect(),
    )
}
