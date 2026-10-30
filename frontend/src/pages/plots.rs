//! The Monitor pages' charts beyond sparklines and bars (`charts.rs` lays them out, as
//! `Charts.js` does): the CPU histogram, the disks' streamgraph, the thermal calendar, the RAPL
//! waterfall, the TCP beeswarm and the netdev radar, each drawn on ODK's canvas in named inks.

use odk::prelude::*;
use serde_json::Value;

use super::meters::latest_list;
use crate::meters;

/// The chart `which` names, from the live rows `h`.
pub fn plot(which: &str, h: &[Value], generation: u64) -> impl IntoElement + use<> {
    let latest = h.last();
    match which {
        "cpuhist" => {
            let bins = meters::histogram(&latest_list(h, "processes"));
            let chart = Chart::new("cpu-histogram", "Processes by CPU share")
                .height(ChartHeight::Regular)
                .description("CPU histogram")
                .transition(generation)
                .paint(move |scene, _| {
                    for (x, y, w, hh) in
                        meters::histogram_bars(&bins, scene.width(), scene.height())
                    {
                        scene.fill_rect(x, y, w, hh, Ink::Series(0));
                    }
                });
            chart.into_any_element()
        }
        "stream" => {
            let series = crate::charts::disk_stream(h);
            let chart = Chart::new("disk-stream", "Blockdev I/O streamgraph")
                .height(ChartHeight::Regular)
                .description("Blockdev I/O streamgraph")
                .transition(generation)
                .paint(move |scene, _| {
                    let layers = crate::charts::stream_layers(
                        &series,
                        scene.width() as f64,
                        scene.height() as f64,
                    );
                    for (i, l) in layers.iter().enumerate() {
                        let ((x0, y0), rest) = l.top.split_first().expect("a layer has two points");
                        let path = rest.iter().chain(l.bottom.iter().rev()).fold(
                            ChartPath::new().move_to(*x0 as f32, *y0 as f32),
                            |p, (x, y)| p.line_to(*x as f32, *y as f32),
                        );
                        scene.fill(&path.close(), Ink::Series(i));
                    }
                    for l in &layers {
                        if let Some(label) = &l.label {
                            let anchor = match label.align {
                                "left" => TextAnchor::Start,
                                "right" => TextAnchor::End,
                                _ => TextAnchor::Middle,
                            };
                            scene.text(
                                label.x as f32,
                                label.y as f32,
                                l.id.clone(),
                                Ink::Text,
                                anchor,
                                TextBaseline::Middle,
                                true,
                            );
                        }
                    }
                });
            chart.into_any_element()
        }
        "calendar" => {
            let days = crate::charts::thermal_days(h);
            let chart = Chart::new("thermal-calendar", "Hwmon thermal calendar")
                .height(ChartHeight::Tall)
                .description("Hwmon thermal calendar")
                .transition(generation)
                .paint(move |scene, _| {
                    let (cells, labels) =
                        crate::charts::calendar(&days, scene.width() as f64, scene.height() as f64);
                    for c in &cells {
                        let (x, y, w, hh) = c.rect;
                        let level = (c.fill * 4.0).round().clamp(0.0, 4.0) as usize;
                        scene.fill_rect(
                            x as f32,
                            y as f32,
                            (w as f32).max(1.0),
                            (hh as f32).max(1.0),
                            Ink::Heat(level),
                        );
                    }
                    if !cells.is_empty() {
                        for (name, y, _) in labels {
                            scene.text(
                                0.0,
                                y as f32,
                                name,
                                Ink::TextMuted,
                                TextAnchor::Start,
                                TextBaseline::Middle,
                                false,
                            );
                        }
                    }
                });
            chart.into_any_element()
        }
        "waterfall" => {
            let steps = match h {
                [.., prev, next] => crate::charts::rapl_steps(Some(prev), Some(next)),
                _ => Vec::new(),
            };
            let chart = Chart::new("rapl-waterfall", "RAPL energy waterfall")
                .height(ChartHeight::Regular)
                .description("RAPL energy waterfall")
                .transition(generation)
                .paint(move |scene, _| {
                    for (i, b) in crate::charts::waterfall(
                        &steps,
                        scene.width() as f64,
                        scene.height() as f64,
                    )
                    .iter()
                    .enumerate()
                    {
                        let (x, y, w, hh) = b.rect;
                        let ink = if b.value < 0.0 {
                            Ink::Heat(4)
                        } else {
                            Ink::Series(i)
                        };
                        scene.fill_rect(
                            x as f32,
                            y as f32,
                            (w as f32).max(1.0),
                            (hh as f32).max(1.0),
                            ink,
                        );
                        if let Some((label, lx, ly, _)) = &b.label {
                            scene.text(
                                *lx as f32,
                                *ly as f32,
                                label.clone(),
                                Ink::Text,
                                TextAnchor::Middle,
                                TextBaseline::Bottom,
                                true,
                            );
                        }
                    }
                });
            chart.into_any_element()
        }
        "bees" => {
            let items = crate::layouts::tcp_bees(latest);
            let chart = Chart::new("tcp-beeswarm", "TCP state beeswarm")
                .height(ChartHeight::Tall)
                .description("TCP state beeswarm")
                .transition(generation)
                .paint(move |scene, _| {
                    let (w, hh) = (scene.width() as f64, scene.height() as f64);
                    let (bees, labels) = crate::layouts::beeswarm(&items, w, hh);
                    for b in &bees {
                        let ink = Ink::Series(
                            labels.iter().position(|(g, ..)| *g == b.group).unwrap_or(0),
                        );
                        let (x, y, side) =
                            ((b.x - b.r) as f32, (b.y - b.r) as f32, (b.r * 2.0) as f32);
                        scene.fill_rect(x, y, side, side, ink);
                        scene.stroke_rect(x, y, side, side, Ink::Canvas, StrokeWidth::Thin);
                    }
                    for (group, x, _) in labels {
                        scene.text(
                            x as f32,
                            hh as f32,
                            group,
                            Ink::Text,
                            TextAnchor::Middle,
                            TextBaseline::Bottom,
                            true,
                        );
                    }
                });
            chart.into_any_element()
        }
        "radar" => {
            let series = crate::layouts::netdev_radar(&latest_list(h, "ifaces"));
            let chart = Chart::new("netdev-radar", "Netdev counter radar")
                .height(ChartHeight::Tall)
                .description("Netdev counter radar")
                .transition(generation)
                .paint(move |scene, _| {
                    let (w, hh) = (scene.width() as f64, scene.height() as f64);
                    let ((cx, cy), spokes, polygons, labels) =
                        crate::layouts::radar(&series, crate::layouts::NETDEV_AXES, w, hh);
                    let point = |p: (f64, f64)| (p.0 as f32, p.1 as f32);
                    let ring = |pts: &[(f64, f64)]| {
                        let (first, rest) = pts.split_first()?;
                        let (x, y) = point(*first);
                        Some(
                            rest.iter()
                                .fold(ChartPath::new().move_to(x, y), |p, q| {
                                    let (x, y) = point(*q);
                                    p.line_to(x, y)
                                })
                                .close(),
                        )
                    };
                    for s in &spokes {
                        let (x, y) = point(s.end);
                        scene.stroke(
                            &ChartPath::new().move_to(cx as f32, cy as f32).line_to(x, y),
                            Ink::Axis,
                            StrokeWidth::Thin,
                        );
                    }
                    if let Some(path) = ring(&spokes.iter().map(|s| s.end).collect::<Vec<_>>()) {
                        scene.stroke(&path, Ink::Axis, StrokeWidth::Thin);
                    }
                    for (i, (_, pts)) in polygons.iter().enumerate() {
                        if let Some(path) = ring(pts) {
                            scene.fill(&path, Ink::SeriesFill(i));
                            scene.stroke(&path, Ink::Series(i), StrokeWidth::Thin);
                        }
                    }
                    for s in &spokes {
                        let anchor = match s.align {
                            "left" => TextAnchor::Start,
                            "right" => TextAnchor::End,
                            _ => TextAnchor::Middle,
                        };
                        let baseline = match s.baseline {
                            "top" => TextBaseline::Top,
                            "bottom" => TextBaseline::Bottom,
                            _ => TextBaseline::Middle,
                        };
                        let (x, y) = point(s.at);
                        scene.text(x, y, s.label, Ink::TextMuted, anchor, baseline, false);
                    }
                    for (id, x, y) in labels {
                        scene.text(
                            x as f32,
                            y as f32,
                            id,
                            Ink::Text,
                            TextAnchor::Middle,
                            TextBaseline::Middle,
                            true,
                        );
                    }
                });
            chart.into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}
