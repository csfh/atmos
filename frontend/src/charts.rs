//! The Monitor pages' chart layouts (`services/Charts.js`): what each chart is built from the
//! live rows, and where its marks go, before the canvas draws them in named inks
//! (`pages/charts.rs`). `tests/charts_parity.rs` holds each to its JavaScript.

use serde_json::Value;

use crate::stats::non_neg;

/// `diskStream(history)`: each disk, in the order first seen, with its read and write rate in
/// each row (0 where it was missing).
pub fn disk_stream(history: &[Value]) -> Vec<(String, Vec<f64>)> {
    let disks = |row: &Value| {
        row.get("disks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut names: Vec<String> = Vec::new();
    for row in history {
        for d in disks(row) {
            let name = d
                .get("name")
                .filter(|n| crate::js::truthy(Some(n)))
                .map(crate::js::string);
            if let Some(name) = name
                && !names.contains(&name)
            {
                names.push(name);
            }
        }
    }
    names
        .into_iter()
        .map(|name| {
            let values = history
                .iter()
                .map(|row| {
                    disks(row)
                        .iter()
                        .find(|d| {
                            d.get("name").map(crate::js::string).as_deref() == Some(name.as_str())
                        })
                        .map_or(0.0, |d| {
                            non_neg(d.get("readBps")).unwrap_or(0.0)
                                + non_neg(d.get("writeBps")).unwrap_or(0.0)
                        })
                })
                .collect();
            (name, values)
        })
        .collect()
}

/// Where a layer's name goes: its point, how it is aligned to it, and how wide it may be.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub x: f64,
    pub y: f64,
    /// `left`, `center` or `right`.
    pub align: &'static str,
    pub max_w: f64,
}

/// A streamgraph layer: its top and bottom edges, left to right, and its name's place.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub id: String,
    pub top: Vec<(f64, f64)>,
    pub bottom: Vec<(f64, f64)>,
    pub label: Option<Label>,
}

/// `streamgraphLayers(series, width, height)`: the layers stacked about a centerline, each as
/// thick as its share of the busiest moment.
pub fn stream_layers(series: &[(String, Vec<f64>)], w: f64, h: f64) -> Vec<Layer> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || series.is_empty() {
        return Vec::new();
    }
    let len = series.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
    if len < 2 {
        return Vec::new();
    }
    let rows: Vec<Vec<f64>> = series
        .iter()
        .map(|(_, v)| {
            let mut out: Vec<f64> = v
                .iter()
                .map(|n| if n.is_finite() && *n > 0.0 { *n } else { 0.0 })
                .collect();
            out.resize(len, 0.0);
            out
        })
        .collect();
    let totals: Vec<f64> = (0..len).map(|j| rows.iter().map(|r| r[j]).sum()).collect();
    let max = totals.iter().cloned().fold(0.0, f64::max);
    if max <= 0.0 {
        return Vec::new();
    }
    let baseline: Vec<f64> = totals.iter().map(|t| (h - (t / max) * h) / 2.0).collect();
    rows.iter()
        .enumerate()
        .map(|(i, values)| {
            let mut acc = baseline.clone();
            for below in &rows[..i] {
                for (n, a) in acc.iter_mut().enumerate() {
                    *a += (below[n] / max) * h;
                }
            }
            let x = |j: usize| (w * j as f64) / (len - 1) as f64;
            let top: Vec<(f64, f64)> = (0..len).map(|j| (x(j), acc[j])).collect();
            let bottom: Vec<(f64, f64)> = (0..len)
                .map(|j| (x(j), acc[j] + (values[j] / max) * h))
                .collect();
            let (mut thick, mut best) = (0.0, 0);
            for j in 0..len {
                let t = bottom[j].1 - top[j].1;
                if t >= thick {
                    thick = t;
                    best = j;
                }
            }
            let id = series[i].0.clone();
            let label = (!id.is_empty()).then(|| {
                let lx = top[best].0.clamp(0.0, w);
                let ly = ((top[best].1 + bottom[best].1) / 2.0).clamp(0.0, h);
                let align = if best == 0 {
                    "left"
                } else if best == len - 1 {
                    "right"
                } else {
                    "center"
                };
                let max_w = match align {
                    "left" => (w - lx).max(8.0),
                    "right" => lx.max(8.0),
                    _ => (lx.min(w - lx) * 2.0).max(8.0),
                };
                Label {
                    x: lx,
                    y: ly,
                    align,
                    max_w,
                }
            });
            Layer {
                id,
                top,
                bottom,
                label,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stream_needs_two_samples_and_a_rate() {
        assert!(stream_layers(&[("a".into(), vec![1.0])], 10.0, 10.0).is_empty());
        assert!(stream_layers(&[("a".into(), vec![0.0, 0.0])], 10.0, 10.0).is_empty());
    }
}

/// `axisLabelBand(extent)`: room for a chart's captions.
pub(crate) fn axis_band(e: f64) -> f64 {
    if e.is_nan() || e <= 0.0 {
        return 0.0;
    }
    crate::js::round(e * 0.18).clamp(14.0, 24.0)
}

/// `tileMetrics(extent, count)`: a tile's size, the gap after it, and the two together.
pub(crate) fn tile(e: f64, n: f64) -> (f64, f64, f64) {
    if e.is_nan() || n.is_nan() || e <= 0.0 || n <= 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let slot = e / n;
    let mut gap = 0.0;
    if n > 1.0 && slot >= 4.0 {
        gap = 1.0;
    }
    if n > 1.0 && slot >= 8.0 {
        gap = 2.0;
    }
    let mut size = if n > 1.0 {
        (e - gap * (n - 1.0)) / n
    } else {
        e
    };
    if size.is_nan() || size <= 0.0 {
        gap = 0.0;
        size = slot;
    }
    (size, gap, size + gap)
}

/// A RAPL domain's energy since the row before, in joules: its id, its name, and the joules.
pub type Step = (String, String, f64);

/// `raplSteps(prev, next)`: each domain's energy between two rows, where the counter moved on.
pub fn rapl_steps(prev: Option<&Value>, next: Option<&Value>) -> Vec<Step> {
    let list = |r: Option<&Value>| {
        r.and_then(|r| r.get("rapl"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let present = |v: &Value, k: &str| v.get(k).filter(|x| !x.is_null()).map(crate::js::string);
    let before: Vec<(String, Option<f64>)> = list(prev)
        .iter()
        .filter_map(|r| Some((present(r, "id")?, crate::stats::finite(r.get("uj")))))
        .collect();
    list(next)
        .iter()
        .filter_map(|r| {
            let id = present(r, "id")?;
            let n = crate::stats::finite(r.get("uj"))?;
            let p = before
                .iter()
                .rev()
                .find(|(i, _)| *i == id)
                .and_then(|(_, p)| *p)?;
            if n < p {
                return None;
            }
            let label = present(r, "name")
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| id.clone());
            Some((id, label, (n - p) / 1e6))
        })
        .collect()
}

/// A waterfall bar: its value, its place, and its caption's centre and width.
#[derive(Debug, Clone, PartialEq)]
pub struct Bar {
    pub id: String,
    pub value: f64,
    pub rect: (f64, f64, f64, f64),
    pub label: Option<(String, f64, f64, f64)>,
}

/// `waterfallBars(steps, width, height)`: each step from where the last ended.
pub fn waterfall(steps: &[Step], w: f64, h: f64) -> Vec<Bar> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || steps.is_empty() {
        return Vec::new();
    }
    let (mut min, mut max, mut origin) = (0.0f64, 0.0f64, 0.0);
    let mut rows = Vec::new();
    for (id, label, value) in steps.iter().filter(|s| s.2.is_finite()) {
        let before = origin;
        rows.push((id.clone(), label.clone(), *value, before));
        origin += value;
        max = max.max(origin).max(before);
        min = min.min(origin).min(before);
    }
    if rows.is_empty() {
        return Vec::new();
    }
    let span = if max - min > 0.0 { max - min } else { 1.0 };
    let gap = 4.0;
    let bw = (w - gap * (rows.len() as f64 - 1.0)) / rows.len() as f64;
    if bw.is_nan() || bw <= 0.0 {
        return Vec::new();
    }
    let mut plot = h - axis_band(h);
    if plot.is_nan() || plot <= 0.0 {
        plot = h;
    }
    rows.into_iter()
        .enumerate()
        .map(|(i, (id, label, value, origin))| {
            let y0 = plot - ((origin - min) / span) * plot;
            let y1 = plot - ((origin + value - min) / span) * plot;
            let (top, bot) = (y0.min(y1), y0.max(y1));
            let x = i as f64 * (bw + gap);
            let label = (!label.is_empty()).then(|| (label, x + bw / 2.0, h, bw.max(8.0)));
            Bar {
                id,
                value,
                rect: (x, top, bw, (bot - top).max(1.0)),
                label,
            }
        })
        .collect()
}

/// The UTC day of a time in milliseconds: `YYYY-MM-DD`, and the weekday (0 is Sunday).
pub(crate) fn utc_day(ms: f64) -> (String, i64, i64) {
    let days = (ms / 86_400_000.0).floor() as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (
        format!("{y:04}-{m:02}-{d:02}"),
        days,
        (days + 4).rem_euclid(7),
    )
}

/// `thermalDays(history)`: the hottest temperature of each UTC day the rows span.
pub fn thermal_days(history: &[Value]) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    for row in history {
        let t = crate::js::number(row.get("at"));
        if !t.is_finite() || t <= 0.0 {
            continue;
        }
        let (day, _, _) = utc_day(t);
        let temps = row
            .get("sensors")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|s| s.get("kind").and_then(Value::as_str) == Some("temp"))
            .filter_map(|s| crate::stats::finite(s.get("value")))
            .chain(crate::stats::finite(row.get("cpuTemp")));
        for v in temps {
            match out.iter_mut().find(|(d, _)| *d == day) {
                Some((_, max)) if v > *max => *max = v,
                Some(_) => {}
                None => out.push((day.clone(), v)),
            }
        }
    }
    out
}

/// A calendar cell: its place, its day, and how hot it was from 0 (coolest) to 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub rect: (f64, f64, f64, f64),
    pub date: String,
    pub fill: f64,
}

/// `calendarLayout(days, width, height)`: a cell per day in a week × weekday grid after the
/// weekday names, and each name's height and width.
pub fn calendar(
    days: &[(String, f64)],
    w: f64,
    h: f64,
) -> (Vec<Cell>, Vec<(&'static str, f64, f64)>) {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let mut band = axis_band(w).max(crate::js::round(w * 0.3).min(64.0));
    let mut plot = w - band;
    if plot.is_nan() || plot <= w * 0.5 {
        band = axis_band(w);
        plot = w - band;
    }
    if plot.is_nan() || plot <= 0.0 {
        band = 0.0;
        plot = w;
    }
    // The days that parse, each as its day number since the epoch.
    let rows: Vec<(String, i64, f64)> = days
        .iter()
        .filter(|(_, v)| v.is_finite())
        .filter_map(|(date, v)| parse_day(date).map(|d| (date.clone(), d, *v)))
        .collect();
    if rows.is_empty() || plot <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let (min_t, max_t) = rows
        .iter()
        .fold((i64::MAX, i64::MIN), |(lo, hi), (_, t, _)| {
            (lo.min(*t), hi.max(*t))
        });
    let (min_v, max_v) = rows
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), (_, _, v)| {
            (lo.min(*v), hi.max(*v))
        });
    let origin = min_t - (min_t + 4).rem_euclid(7);
    let weeks = ((max_t - origin) / 7 + 1).max(1) as f64;
    let (xs, _, xstep) = tile(plot, weeks);
    let (ys, _, ystep) = tile(h, 7.0);
    if xs <= 0.0 || ys <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let span = max_v - min_v;
    let cells = rows
        .iter()
        .map(|(date, t, v)| {
            let week = ((t - origin) / 7) as f64;
            let weekday = (t + 4).rem_euclid(7) as f64;
            let fill = if span == 0.0 { 0.5 } else { (v - min_v) / span };
            Cell {
                rect: (week * xstep + band, weekday * ystep, xs, ys),
                date: date.clone(),
                fill,
            }
        })
        .collect();
    let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let labels = names
        .iter()
        .enumerate()
        .map(|(i, n)| {
            (
                *n,
                (i as f64 * ystep + ys / 2.0).clamp(0.0, h),
                band.max(8.0),
            )
        })
        .collect();
    (cells, labels)
}

/// `parseDay(s)`: a `YYYY-MM-DD` day's number since the epoch.
fn parse_day(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    let ok = b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && s.chars()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if !ok {
        return None;
    }
    let (y, m, d): (i64, i64, i64) = (
        s[..4].parse().ok()?,
        s[5..7].parse().ok()?,
        s[8..].parse().ok()?,
    );
    // `Date.UTC` rolls a month or day over; the days from the civil date do the same.
    let (y, m) = (y + (m - 1).div_euclid(12), (m - 1).rem_euclid(12) + 1);
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468 + d - 1)
}
