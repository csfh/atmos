//! The marks the Monitor and Home pages draw on ODK's chart canvas, in named inks: a sparkline
//! (one series, or two, the first filled under unless asked not to), the core bars, and the
//! memory bar. Each takes the height of a `Space::Xl`: ODK has no chart height of its own yet.

use odk::prelude::*;

use crate::{monitor, stats};

/// What a sparkline draws: its series, a second one, whether it is hot, and whether the first is
/// filled under.
pub struct Spark {
    pub values: Vec<f64>,
    pub second: Vec<f64>,
    pub hot: bool,
    pub fill: bool,
}

fn line(values: &[f64], w: f32, h: f32) -> Option<ChartPath> {
    let points = stats::sparkline_points(values, w, h, 1.0);
    let ((x, y), rest) = points.split_first()?;
    Some(
        rest.iter()
            .fold(ChartPath::new().move_to(*x, *y), |p, (x, y)| {
                p.line_to(*x, *y)
            }),
    )
}

fn frame(chart: Chart) -> impl IntoElement {
    v_flex().w_full().h(Space::Xl).child(chart)
}

/// A sparkline of the last minute, named `name` and described by `says`.
pub fn spark(
    id: String,
    name: &'static str,
    says: String,
    s: Spark,
    generation: u64,
) -> impl IntoElement {
    let chart = Chart::new(SharedString::from(id), name)
        .description(says)
        .transition(generation)
        .paint(move |scene, _| {
            let (w, h) = (scene.width(), scene.height());
            let ink = if s.hot { Ink::Heat(4) } else { Ink::Series(0) };
            if let Some(path) = line(&s.values, w, h) {
                if s.fill {
                    let first = stats::sparkline_points(&s.values, w, h, 1.0);
                    let (x0, x1) = (
                        first.first().map_or(0.0, |p| p.0),
                        first.last().map_or(w, |p| p.0),
                    );
                    scene.fill(
                        &path.clone().line_to(x1, h).line_to(x0, h).close(),
                        Ink::SeriesFill(0),
                    );
                }
                scene.stroke(&path, ink, StrokeWidth::Regular);
            }
            if let Some(path) = line(&s.second, w, h) {
                scene.stroke(&path, Ink::Series(1), StrokeWidth::Regular);
            }
        });
    frame(chart)
}

/// A bar per core, from 0 to 100% busy; a hot one in the ramp's top ink.
pub fn core_bars(id: String, says: String, values: Vec<f64>, generation: u64) -> impl IntoElement {
    let chart = Chart::new(SharedString::from(id), "Busy share of each core")
        .description(says)
        .transition(generation)
        .paint(move |scene, _| {
            for ((x, y, w, h), hot) in monitor::bars(&values, scene.width(), scene.height()) {
                scene.fill_rect(x, y, w, h, if hot { Ink::Heat(4) } else { Ink::Series(0) });
            }
        });
    frame(chart)
}

/// Memory as one bar: used, buffers, cached and free, each its share.
pub fn memory(
    id: String,
    says: String,
    parts: Vec<(&'static str, &'static str, f64)>,
    generation: u64,
) -> impl IntoElement {
    let chart = Chart::new(SharedString::from(id), "Memory by use")
        .description(says)
        .transition(generation)
        .paint(move |scene, _| {
            for (i, (x, y, w, h)) in monitor::stacked(&parts, scene.width(), scene.height())
                .into_iter()
                .enumerate()
            {
                scene.fill_rect(x, y, w, h, Ink::Series(i));
            }
        });
    frame(chart)
}
