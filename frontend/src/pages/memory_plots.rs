//! The Memory page's charts (`memory_charts.rs` and `treemap.rs` lay them out, as `Charts.js`
//! does): the buddy horizon, the RSS treemap, the meminfo Sankey, the cgroup sunburst and the
//! slab icicle, drawn on ODK's canvas as `PrefsChart.qml` paints them. A press on the treemap
//! focuses the process under it and a press on the focus (or off every rectangle) goes back
//! out; a drag pans it and the wheel zooms it, as the QML chart's mouse area and wheel do. Its
//! focus and view are the page's (`data["rss-view"]`), and a focus the tree no longer has is
//! dropped.

use std::cell::Cell;
use std::rc::Rc;

use odk::prelude::*;
use serde_json::{Value, json};

use super::catalog::CatalogPage;
use super::cpu_plots::caption;
use super::meters::latest_list;
use crate::memory_charts::{self, tone_id};
use crate::treemap::{self, Rect, View};

const VIEW: &str = "rss-view";

fn framed(chart: Chart) -> impl IntoElement + use<> {
    v_flex().w_full().h(Space::Xl).child(chart)
}

fn boxes(scene: &mut Scene, rects: &[Rect]) {
    for r in rects {
        let (w, h) = ((r.w as f32).max(1.0), (r.h as f32).max(1.0));
        scene.fill_rect(r.x as f32, r.y as f32, w, h, Ink::Series(tone_id(&r.id)));
        scene.stroke_rect(r.x as f32, r.y as f32, w, h, Ink::Canvas, StrokeWidth::Thin);
    }
    for l in rects.iter().filter_map(|r| r.label.as_ref()) {
        caption(scene, l);
    }
}

/// The treemap's focus and view as the page keeps them.
fn saved(page: &CatalogPage) -> (String, View) {
    let s = page.data.get(VIEW);
    let n = |k: &str, or: f64| s.and_then(|s| s[k].as_f64()).unwrap_or(or);
    let focus = s.and_then(|s| s["focus"].as_str()).unwrap_or_default();
    let view = View {
        scale: n("scale", 1.0),
        x: n("x", 0.0),
        y: n("y", 0.0),
    };
    (focus.to_owned(), view)
}

/// The treemap's focus, for what a test asks.
pub fn treemap_focus(page: &CatalogPage) -> String {
    saved(page).0
}

/// The treemap's focus and view, back at the top when the tree no longer has the focus.
fn state(page: &CatalogPage, tree: Option<&Value>) -> (String, View) {
    let (focus, view) = saved(page);
    if !focus.is_empty() && tree.is_none_or(|t| treemap::find_node(t, &focus).is_none()) {
        return (String::new(), View::default());
    }
    (focus, view)
}

fn keep(page: &mut CatalogPage, focus: &str, v: View, cx: &mut Context<CatalogPage>) {
    page.data.insert(
        VIEW,
        json!({"focus": focus, "scale": v.scale, "x": v.x, "y": v.y}),
    );
    cx.notify();
}

fn rss(page: &CatalogPage, this: &Entity<CatalogPage>, h: &[Value], generation: u64) -> Chart {
    let tree = treemap::rss_tree(&latest_list(h, "processes"));
    let (focus, view) = state(page, tree.as_ref());
    // The box as it was last drawn, which a press, a pan and a zoom are in.
    let size = Rc::new(Cell::new((0.0f64, 0.0f64)));
    let layout = {
        let tree = tree.clone();
        move |focus: &str, view: View, (w, h): (f64, f64)| {
            let (rects, labels) = treemap::layout(tree.as_ref(), w, h, focus);
            treemap::in_view(&rects, &labels, view)
        }
    };
    let painted = (size.clone(), layout.clone(), focus.clone());
    let chart = Chart::new("rss-treemap", "Process RSS treemap")
        .description("Process RSS treemap")
        .transition(generation)
        .paint(move |scene, _| {
            let (size, layout, focus) = &painted;
            size.set((scene.width() as f64, scene.height() as f64));
            let (rects, labels) = layout(focus, view, size.get());
            boxes(scene, &rects);
            for l in &labels {
                caption(scene, l);
            }
        });
    let (pressed, panned, zoomed) = (this.clone(), this.clone(), this.clone());
    let (s1, s2, s3) = (size.clone(), size.clone(), size);
    chart
        .on_press(move |x, y, _, cx| {
            let tree = tree.clone();
            let layout = layout.clone();
            let size = s1.get();
            pressed.update(cx, |p, cx| {
                let Some(t) = tree.as_ref() else { return };
                let (focus, view) = state(p, Some(t));
                let (rects, _) = layout(&focus, view, size);
                let next = match treemap::hit(&rects, x as f64, y as f64) {
                    Some(r) if !r.id.is_empty() => {
                        let id = treemap::focus_id(&r.id);
                        if id == focus {
                            treemap::parent_id(t, &focus)
                        } else {
                            id.to_owned()
                        }
                    }
                    _ if !focus.is_empty() => treemap::parent_id(t, &focus),
                    _ => return,
                };
                if next != focus {
                    keep(p, &next, View::default(), cx);
                }
            })
        })
        .on_pan(move |dx, dy, _, cx| {
            let (w, h) = s2.get();
            panned.update(cx, |p, cx| {
                let (focus, v) = saved(p);
                let v = treemap::clamp_pan(v.scale, v.x + dx as f64, v.y + dy as f64, w, h);
                keep(p, &focus, v, cx);
            })
        })
        .on_zoom(move |factor, x, y, _, cx| {
            let (w, h) = s3.get();
            zoomed.update(cx, |p, cx| {
                let (focus, v) = saved(p);
                let v = treemap::zoom(v, x as f64, y as f64, factor as f64, w, h);
                keep(p, &focus, v, cx);
            })
        })
}

/// The chart `which` names, from the live rows `h`.
pub fn plot(
    which: &str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    h: &[Value],
    generation: u64,
) -> impl IntoElement + use<> {
    match which {
        "horizon" => {
            let series = memory_charts::buddy_series(h);
            framed(
                Chart::new("buddy-horizon", "Buddy-order horizon")
                    .description("Buddy-order horizon")
                    .transition(generation)
                    .paint(move |scene, _| {
                        let (w, hh) = (scene.width() as f64, scene.height() as f64);
                        let (bands, labels) = memory_charts::horizon_layout(&series, w, hh, 3);
                        for b in &bands {
                            let (Some(first), Some(last)) = (b.points.first(), b.points.last())
                            else {
                                continue;
                            };
                            let path = b
                                .points
                                .iter()
                                .fold(
                                    ChartPath::new().move_to(first.0 as f32, hh as f32),
                                    |p, (x, y)| p.line_to(*x as f32, *y as f32),
                                )
                                .line_to(last.0 as f32, hh as f32)
                                .close();
                            let level = (b.fill * 4.0).round().clamp(0.0, 4.0) as usize;
                            scene.fill(&path, Ink::Heat(level));
                        }
                        for l in &labels {
                            caption(scene, l);
                        }
                    }),
            )
            .into_any_element()
        }
        "treemap" => framed(rss(page, this, h, generation)).into_any_element(),
        "sankey" => {
            let (nodes, flows) = memory_charts::meminfo_sankey(h.last());
            framed(
                Chart::new("meminfo-sankey", "Meminfo composition Sankey")
                    .description("Meminfo composition Sankey")
                    .transition(generation)
                    .paint(move |scene, _| {
                        let (w, hh) = (scene.width() as f64, scene.height() as f64);
                        let (blocks, links) = memory_charts::sankey_layout(&nodes, &flows, w, hh);
                        for (i, b) in blocks.iter().enumerate() {
                            scene.fill_rect(
                                b.x as f32,
                                b.y as f32,
                                (b.w as f32).max(1.0),
                                (b.h as f32).max(1.0),
                                Ink::Series(i),
                            );
                        }
                        // A flow as the band its stroke would cover: its top edge out, its
                        // bottom edge back.
                        for l in &links {
                            let half = (l.width.max(1.0) / 2.0) as f32;
                            let (x0, y0, x1, y1) =
                                (l.x0 as f32, l.y0 as f32, l.x1 as f32, l.y1 as f32);
                            let mid = (x0 + x1) / 2.0;
                            let path = ChartPath::new()
                                .move_to(x0, y0 - half)
                                .curve_to(mid, y0 - half, mid, y1 - half, x1, y1 - half)
                                .line_to(x1, y1 + half)
                                .curve_to(mid, y1 + half, mid, y0 + half, x0, y0 + half)
                                .close();
                            scene.fill(&path, Ink::SeriesFill(tone_id(&l.source)));
                        }
                        for b in &blocks {
                            caption(scene, &b.label);
                        }
                    }),
            )
            .into_any_element()
        }
        "sunburst" => {
            let tree = h.last().and_then(|l| l.get("cgroups")).cloned();
            framed(
                Chart::new("cgroup-sunburst", "Cgroup memory sunburst")
                    .description("Cgroup memory sunburst")
                    .transition(generation)
                    .paint(move |scene, _| {
                        let (w, hh) = (scene.width() as f64, scene.height() as f64);
                        let arcs = memory_charts::sunburst_arcs(tree.as_ref(), w, hh);
                        for a in &arcs {
                            let (cx, cy) = (a.cx as f32, a.cy as f32);
                            let path = ChartPath::new()
                                .arc(cx, cy, a.outer as f32, a.start as f32, a.end as f32)
                                .arc(cx, cy, a.inner as f32, a.end as f32, a.start as f32)
                                .close();
                            scene.fill(&path, Ink::Series(tone_id(&a.id)));
                            scene.stroke(&path, Ink::Canvas, StrokeWidth::Thin);
                        }
                        for l in arcs.iter().filter_map(|a| a.label.as_ref()) {
                            caption(scene, l);
                        }
                    }),
            )
            .into_any_element()
        }
        "icicle" => {
            let tree = memory_charts::slab_tree(&latest_list(h, "slabs"));
            framed(
                Chart::new("slab-icicle", "Slab cache icicle")
                    .description("Slab cache icicle")
                    .transition(generation)
                    .paint(move |scene, _| {
                        let (w, hh) = (scene.width() as f64, scene.height() as f64);
                        boxes(scene, &memory_charts::icicle_rects(tree.as_ref(), w, hh));
                    }),
            )
            .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}
