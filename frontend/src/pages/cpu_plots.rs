//! The CPU page's charts (`cpu_charts.rs` lays them out, as `Charts.js` does): the IRQ heatmap,
//! the PSI ridgeline, the core-frequency violins and the softirq rose, drawn on ODK's canvas in
//! named inks with their captions haloed, as `PrefsChart.qml` paints them.

use std::f64::consts::PI;

use odk::prelude::*;
use serde_json::Value;

use crate::cpu_charts::{self, Label};

pub(super) fn caption(scene: &mut Scene, l: &Label) {
    let anchor = match l.align {
        "left" => TextAnchor::Start,
        "right" => TextAnchor::End,
        _ => TextAnchor::Middle,
    };
    let baseline = match l.baseline {
        "top" => TextBaseline::Top,
        "bottom" => TextBaseline::Bottom,
        _ => TextBaseline::Middle,
    };
    let (x, y) = (
        l.lx.clamp(0.0, scene.width() as f64),
        l.ly.clamp(0.0, scene.height() as f64),
    );
    scene.text(
        x as f32,
        y as f32,
        l.label.clone(),
        Ink::Text,
        anchor,
        baseline,
        true,
    );
}

fn outline(points: &[(f64, f64)], closed: bool) -> Option<ChartPath> {
    let ((x, y), rest) = points.split_first()?;
    let path = rest.iter().fold(
        ChartPath::new().move_to(*x as f32, *y as f32),
        |p, (x, y)| p.line_to(*x as f32, *y as f32),
    );
    Some(if closed { path.close() } else { path })
}

/// A wedge as a path: the centre, then the arc in short steps.
fn wedge(w: &cpu_charts::Wedge) -> ChartPath {
    let steps = ((w.end - w.start) / (PI / 36.0)).ceil().max(2.0) as usize;
    (0..=steps)
        .map(|k| w.start + (w.end - w.start) * k as f64 / steps as f64)
        .fold(
            ChartPath::new().move_to(w.cx as f32, w.cy as f32),
            |p, a| p.line_to((w.cx + w.r * a.cos()) as f32, (w.cy + w.r * a.sin()) as f32),
        )
        .close()
}
/// The chart `which` names, from the live rows `h`.
pub fn plot(which: &str, h: &[Value], generation: u64) -> impl IntoElement + use<> {
    let last_two = match h {
        [.., prev, next] => Some((prev, next)),
        _ => None,
    };
    match which {
        "heatmap" => {
            let m = last_two.map_or_else(Vec::new, |(p, n)| {
                cpu_charts::irq_rate_matrix(p.get("interrupts"), n.get("interrupts"))
            });
            Chart::new("irq-heatmap", "IRQ land heatmap")
                .height(ChartHeight::Regular)
                .description("IRQ land heatmap")
                .transition(generation)
                .paint(move |scene, _| {
                    let (w, hh) = (scene.width() as f64, scene.height() as f64);
                    let (cells, labels) = cpu_charts::heatmap_layout(&m, w, hh);
                    for c in &cells {
                        let level = (c.fill * 4.0).round().clamp(0.0, 4.0) as usize;
                        scene.fill_rect(
                            c.x as f32,
                            c.y as f32,
                            (c.w as f32).max(1.0),
                            (c.h as f32).max(1.0),
                            Ink::Heat(level),
                        );
                    }
                    for l in &labels {
                        caption(scene, l);
                    }
                })
                .into_any_element()
        }
        "ridgeline" => {
            let series = cpu_charts::psi_ridges(h);
            Chart::new("psi-ridgeline", "PSI stall ridgeline")
                .height(ChartHeight::Regular)
                .description("PSI stall ridgeline")
                .transition(generation)
                .paint(move |scene, _| {
                    let (w, hh) = (scene.width() as f64, scene.height() as f64);
                    let (ridges, labels) = cpu_charts::ridgeline_layout(&series, w, hh);
                    for (i, r) in ridges.iter().enumerate() {
                        if let Some(path) = outline(&r.points, false) {
                            scene.stroke(&path, Ink::Series(i), StrokeWidth::Thin);
                        }
                    }
                    for l in &labels {
                        caption(scene, l);
                    }
                })
                .into_any_element()
        }
        "violin" => {
            let series = cpu_charts::freq_violins(h);
            Chart::new("freq-violin", "Core-freq violin")
                .height(ChartHeight::Regular)
                .description("Core-freq violin")
                .transition(generation)
                .paint(move |scene, _| {
                    let (w, hh) = (scene.width() as f64, scene.height() as f64);
                    let violins = cpu_charts::violin_paths(&series, w, hh);
                    for (i, v) in violins.iter().enumerate() {
                        if let Some(path) = outline(&v.points, true) {
                            scene.fill(&path, Ink::SeriesFill(i));
                            scene.stroke(&path, Ink::Series(i), StrokeWidth::Thin);
                        }
                    }
                    for v in &violins {
                        caption(scene, &v.label);
                    }
                })
                .into_any_element()
        }
        "rose" => {
            let items = last_two.map_or_else(Vec::new, |(p, n)| {
                cpu_charts::softirq_wedges(p.get("softirqs"), n.get("softirqs"))
            });
            Chart::new("softirq-rose", "Softirq Nightingale rose")
                .height(ChartHeight::Regular)
                .description("Softirq Nightingale rose")
                .transition(generation)
                .paint(move |scene, _| {
                    let (w, hh) = (scene.width() as f64, scene.height() as f64);
                    let wedges = cpu_charts::nightingale_wedges(&items, w, hh);
                    for (i, wd) in wedges.iter().enumerate() {
                        let path = wedge(wd);
                        scene.fill(&path, Ink::Series(i));
                        scene.stroke(&path, Ink::Canvas, StrokeWidth::Thin);
                    }
                    for wd in &wedges {
                        caption(scene, &wd.label);
                    }
                })
                .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}
