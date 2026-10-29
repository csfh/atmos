//! The CPU page's charts (`pages/monitor/CpuPage.qml`), laid out as `services/Charts.js` lays
//! them out: the IRQ heatmap (`irqRateMatrix`, `heatmapCells`, `heatmapLayout`), the PSI
//! ridgeline (`psiRidges`, `ridgelinePaths`, `ridgelineLayout`), the core-frequency violins
//! (`freqViolins`, `kdeAt`, `violinPaths`) and the softirq rose (`softirqWedges`,
//! `nightingaleWedges`). `tests/cpu_charts_parity.rs` runs these against the JavaScript.

use std::collections::HashMap;
use std::f64::consts::PI;

use serde_json::Value;

use crate::charts::{axis_band, tile};
use crate::js;
use crate::layouts::top_indices;
use crate::stats::finite;

/// A series: its id and its values, a gap where one is not a number.
pub type Series = (String, Vec<Option<f64>>);

/// A row of `/proc/interrupts` or `/proc/softirqs` rates: its id as the table gives it, and a
/// rate per CPU.
#[derive(Debug, Clone, PartialEq)]
pub struct Rates {
    pub id: Value,
    pub values: Vec<Option<f64>>,
}

/// A caption: its words, where it goes, and how it sits there.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub id: String,
    pub label: String,
    pub lx: f64,
    pub ly: f64,
    pub align: &'static str,
    pub baseline: &'static str,
    pub max_w: f64,
}

fn rows_of(table: Option<&Value>) -> &[Value] {
    table
        .and_then(|t| t.get("rows"))
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// The row of `before` a row of `after` is compared with: the one at its place with its id, or
/// the one with its id.
fn before_of<'a>(
    before: &'a [Value],
    at: usize,
    row: &Value,
    by_id: &mut Option<HashMap<String, &'a Value>>,
) -> Option<&'a Vec<Value>> {
    let id = row.get("id");
    if let Some(hit) = before.get(at)
        && hit.get("id") == id
        && let Some(v) = hit.get("values").and_then(Value::as_array)
    {
        return Some(v);
    }
    let id = id.filter(|i| !i.is_null())?;
    let map = by_id.get_or_insert_with(|| {
        let mut m = HashMap::new();
        for hit in before {
            if let Some(i) = hit.get("id").filter(|i| !i.is_null()) {
                m.insert(js::string(i), hit.get("values").unwrap_or(&Value::Null));
            }
        }
        m
    });
    map.get(&js::string(id)).and_then(|v| v.as_array())
}

/// `irqRateMatrix(prev, next)`: each interrupt's count per CPU since the row before, for the
/// interrupts that rose on any CPU.
pub fn irq_rate_matrix(prev: Option<&Value>, next: Option<&Value>) -> Vec<Rates> {
    let after = rows_of(next);
    if after.is_empty() || prev.zip(next).is_some_and(|(a, b)| std::ptr::eq(a, b)) {
        return Vec::new();
    }
    let before = rows_of(prev);
    let mut by_id = None;
    let mut out = Vec::new();
    for (i, row) in after.iter().enumerate() {
        let Some(cells) = row.get("values").and_then(Value::as_array) else {
            continue;
        };
        if cells.is_empty() {
            continue;
        }
        let Some(pv) = before_of(before, i, row, &mut by_id) else {
            continue;
        };
        let pair = |j: usize| (finite(cells.get(j)), finite(pv.get(j)));
        let hot = (0..cells.len()).any(|j| matches!(pair(j), (Some(n), Some(p)) if n > p));
        if !hot {
            continue;
        }
        let values = (0..cells.len())
            .map(|j| match pair(j) {
                (Some(n), Some(p)) if n >= p => Some(n - p),
                _ => None,
            })
            .collect();
        out.push(Rates {
            id: row.get("id").cloned().unwrap_or(Value::Null),
            values,
        });
    }
    out
}

/// `softirqWedges(prev, next)`: each softirq's count since the row before, summed over CPUs.
pub fn softirq_wedges(prev: Option<&Value>, next: Option<&Value>) -> Vec<(Value, f64)> {
    let after = rows_of(next);
    let before = rows_of(prev);
    let mut by_id = None;
    let mut out = Vec::new();
    for (i, row) in after.iter().enumerate() {
        if !js::truthy(Some(row)) {
            continue;
        }
        let cells = row
            .get("values")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        let pv = before_of(before, i, row, &mut by_id);
        let mut sum = 0.0;
        if let Some(pv) = pv
            && !cells.is_empty()
        {
            for j in 0..cells.len().min(pv.len()) {
                if let (Some(n), Some(p)) = (finite(cells.get(j)), finite(pv.get(j)))
                    && n >= p
                {
                    sum += n - p;
                }
            }
        }
        out.push((row.get("id").cloned().unwrap_or(Value::Null), sum));
    }
    out
}

/// `psiRidges(history)`: cpu, memory and io stall, a value per row.
pub fn psi_ridges(history: &[Value]) -> Vec<Series> {
    ["cpu", "memory", "io"]
        .into_iter()
        .map(|kind| {
            let values = history
                .iter()
                .map(|s| {
                    s.get("psi")
                        .filter(|p| js::truthy(Some(p)))
                        .and_then(|p| finite(p.get(kind)))
                })
                .collect();
            (kind.to_owned(), values)
        })
        .collect()
}

/// Whether `Object.keys` puts a key among the array indices, which come first and in order.
fn array_index(key: &str) -> Option<u32> {
    let n: u32 = key.parse().ok()?;
    (n != u32::MAX && n.to_string() == key).then_some(n)
}

/// `freqViolins(history)`: each core's frequencies across the rows, cores in `Object.keys`
/// order.
pub fn freq_violins(history: &[Value]) -> Vec<Series> {
    let mut order: Vec<String> = Vec::new();
    let mut by_id: HashMap<String, Vec<Option<f64>>> = HashMap::new();
    for s in history {
        let cpus = s
            .get("cpus")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        for c in cpus {
            if !js::truthy(Some(c)) {
                continue;
            }
            let Some(f) = finite(c.get("freqMhz")) else {
                continue;
            };
            let id = js::string(c.get("id").unwrap_or(&Value::Null));
            by_id
                .entry(id.clone())
                .or_insert_with(|| {
                    order.push(id);
                    Vec::new()
                })
                .push(Some(f));
        }
    }
    let (mut indices, named): (Vec<String>, Vec<String>) =
        order.into_iter().partition(|k| array_index(k).is_some());
    indices.sort_by_key(|k| array_index(k));
    indices
        .into_iter()
        .chain(named)
        .map(|k| {
            let v = by_id.remove(&k).unwrap_or_default();
            (k, v)
        })
        .collect()
}

/// A heatmap cell: where it goes, its value, its fill from 0 to 1, and its row and column.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub value: f64,
    pub fill: f64,
    pub row: String,
    pub col: usize,
}

/// `heatmapCells(matrix, width, height)`: a cell per value, the hottest rows and columns when
/// there are more than pixels.
pub fn heatmap_cells(matrix: &[Rates], w: f64, h: f64) -> Vec<Cell> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || matrix.is_empty() {
        return Vec::new();
    }
    let mut rows: Vec<(String, Vec<Option<f64>>)> = Vec::new();
    let mut row_score: Vec<f64> = Vec::new();
    for (i, r) in matrix.iter().enumerate() {
        let Some(score) = r.values.iter().flatten().copied().reduce(f64::max) else {
            continue;
        };
        let id = if r.id.is_null() {
            i.to_string()
        } else {
            js::string(&r.id)
        };
        rows.push((id, r.values.clone()));
        row_score.push(score);
    }
    if rows.is_empty() || rows.iter().all(|r| r.1.is_empty()) {
        return Vec::new();
    }
    let fit = h.floor();
    if rows.len() as f64 > fit {
        let keep = top_indices(&row_score, fit as usize);
        rows = keep.into_iter().map(|i| rows[i].clone()).collect();
    }
    let cols = rows.iter().map(|r| r.1.len()).max().unwrap_or(0);
    if cols == 0 {
        return Vec::new();
    }
    let col_score: Vec<Option<f64>> = (0..cols)
        .map(|j| {
            rows.iter()
                .filter_map(|r| r.1.get(j).copied().flatten())
                .reduce(f64::max)
        })
        .collect();
    let mut keep: Vec<usize> = (0..cols).filter(|j| col_score[*j].is_some()).collect();
    if keep.len() as f64 > w.floor() {
        let scores: Vec<f64> = keep.iter().map(|j| col_score[*j].unwrap_or(0.0)).collect();
        keep = top_indices(&scores, w.floor() as usize)
            .into_iter()
            .map(|i| keep[i])
            .collect();
    }
    let cpu_cols = keep.clone();
    let mut cols = cols;
    if keep.len() < cols {
        for r in &mut rows {
            r.1 = keep
                .iter()
                .map(|j| r.1.get(*j).copied().flatten())
                .collect();
        }
        cols = keep.len();
    }
    if cols == 0 {
        return Vec::new();
    }
    let all = || rows.iter().flat_map(|r| r.1.iter().flatten().copied());
    let (Some(min), Some(max)) = (all().reduce(f64::min), all().reduce(f64::max)) else {
        return Vec::new();
    };
    let span = max - min;
    let (xs, _, xstep) = tile(w, cols as f64);
    let (ys, _, ystep) = tile(h, rows.len() as f64);
    if xs.is_nan() || ys.is_nan() || xs <= 0.0 || ys <= 0.0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (i, (id, values)) in rows.iter().enumerate() {
        for (j, col) in cpu_cols.iter().enumerate().take(cols) {
            let Some(n) = values.get(j).copied().flatten() else {
                continue;
            };
            out.push(Cell {
                x: j as f64 * xstep,
                y: i as f64 * ystep,
                w: xs,
                h: ys,
                value: n,
                fill: if span == 0.0 { 0.5 } else { (n - min) / span },
                row: id.clone(),
                col: *col,
            });
        }
    }
    out
}

/// `heatmapLayout(matrix, width, height)`: the cells inside room for the row and column
/// captions, and the captions.
pub fn heatmap_layout(matrix: &[Rates], w: f64, h: f64) -> (Vec<Cell>, Vec<Label>) {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || matrix.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut left = axis_band(w).max(88f64.min(js::round(w * 0.34)));
    let mut top = axis_band(h);
    let mut plot_w = w - left;
    let mut plot_h = h - top;
    if plot_w.is_nan() || plot_w <= w * 0.45 {
        left = axis_band(w);
        plot_w = w - left;
    }
    if plot_h.is_nan() || plot_h <= h * 0.5 {
        top = top.min(12f64.max(js::round(h * 0.2)));
        plot_h = h - top;
    }
    if plot_w.is_nan() || plot_w <= 0.0 {
        left = 0.0;
        plot_w = w;
    }
    if plot_h.is_nan() || plot_h <= 0.0 {
        top = 0.0;
        plot_h = h;
    }
    let mut cells = heatmap_cells(matrix, plot_w, plot_h);
    if cells.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut labels = Vec::new();
    let mut rows_seen = std::collections::HashSet::new();
    let mut cols_seen = std::collections::HashSet::new();
    for c in &mut cells {
        c.x += left;
        c.y += top;
        if rows_seen.insert(c.row.clone()) {
            let ly = (c.y + c.h / 2.0).max(0.0).min(h);
            labels.push(Label {
                id: c.row.clone(),
                label: c.row.clone(),
                lx: left.max(0.0),
                ly,
                align: if left > 0.0 { "right" } else { "left" },
                baseline: "middle",
                max_w: left.max(8.0),
            });
        }
        if cols_seen.insert(c.col) {
            let lx = (c.x + c.w / 2.0).max(0.0).min(w);
            labels.push(Label {
                id: format!("cpu{}", c.col),
                label: c.col.to_string(),
                lx,
                ly: 0.0,
                align: "center",
                baseline: "top",
                max_w: c.w.max(8.0),
            });
        }
    }
    (cells, labels)
}

/// A ridge: its line, and the baseline it rises from.
#[derive(Debug, Clone, PartialEq)]
pub struct Ridge {
    pub id: String,
    pub points: Vec<(f64, f64)>,
    pub baseline: f64,
}

/// `ridgelinePaths(series, width, height)`: each series on one time scale, offset down the
/// chart, all to one height scale.
pub fn ridgeline_paths(series: &[Series], w: f64, h: f64) -> Vec<Ridge> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || series.is_empty() {
        return Vec::new();
    }
    let rows: Vec<(&str, Vec<f64>)> = series
        .iter()
        .map(|(id, v)| {
            (
                id.as_str(),
                v.iter().flatten().map(|n| n.max(0.0)).collect(),
            )
        })
        .collect();
    let max_len = rows.iter().map(|r| r.1.len()).max().unwrap_or(0);
    let max_v = rows
        .iter()
        .flat_map(|r| r.1.iter().copied())
        .fold(0.0, f64::max);
    if max_len < 2 || max_v <= 0.0 {
        return Vec::new();
    }
    let mut band = axis_band(w).max(64f64.min(js::round(w * 0.3)));
    let mut plot_w = w - band;
    if plot_w <= w * 0.5 {
        band = axis_band(w);
        plot_w = w - band;
    }
    if plot_w <= 0.0 {
        band = 0.0;
        plot_w = w;
    }
    let ridge_h = h / (series.len() as f64 + 0.35);
    rows.iter()
        .enumerate()
        .filter(|(_, r)| r.1.len() >= 2)
        .map(|(i, (id, values))| {
            let y0 = ridge_h * (i as f64 + 1.0);
            let points = values
                .iter()
                .enumerate()
                .map(|(j, v)| {
                    (
                        band + (plot_w * j as f64) / (max_len as f64 - 1.0),
                        y0 - (v / max_v) * ridge_h * 0.9,
                    )
                })
                .collect();
            Ridge {
                id: (*id).to_owned(),
                points,
                baseline: y0,
            }
        })
        .collect()
}

/// `ridgelineLayout(series, width, height)`: the ridges and a caption at each baseline.
pub fn ridgeline_layout(series: &[Series], w: f64, h: f64) -> (Vec<Ridge>, Vec<Label>) {
    let ridges = ridgeline_paths(series, w, h);
    let band = ridges
        .first()
        .and_then(|r| r.points.first())
        .map_or(0.0, |p| p.0);
    let labels = ridges
        .iter()
        .map(|r| Label {
            id: r.id.clone(),
            label: r.id.clone(),
            lx: 0.0,
            ly: if h > 0.0 {
                r.baseline.max(0.0).min(h)
            } else {
                r.baseline.max(0.0)
            },
            align: "left",
            baseline: "middle",
            max_w: band.max(8.0),
        })
        .collect();
    (ridges, labels)
}

/// `kdeAt(values, x, h)`: a Gaussian kernel density estimate at `x`.
pub fn kde_at(values: &[f64], x: f64, h: f64) -> f64 {
    let s: f64 = values
        .iter()
        .map(|v| {
            let z = (x - v) / h;
            (-0.5 * z * z).exp()
        })
        .sum();
    s / (values.len() as f64 * h * (2.0 * PI).sqrt())
}

/// A violin: its closed outline and its caption under it.
#[derive(Debug, Clone, PartialEq)]
pub struct Violin {
    pub points: Vec<(f64, f64)>,
    pub label: Label,
}

/// `violinPaths(series, width, height)`: each core's density, mirrored, in a slot of its own;
/// the widest-spread cores when there are more than fit.
pub fn violin_paths(series: &[Series], w: f64, h: f64) -> Vec<Violin> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || series.is_empty() {
        return Vec::new();
    }
    let mut rows: Vec<(&str, Vec<f64>)> = series
        .iter()
        .map(|(id, v)| (id.as_str(), v.iter().flatten().copied().collect::<Vec<_>>()))
        .filter(|r| r.1.len() >= 2)
        .collect();
    if rows.is_empty() {
        return Vec::new();
    }
    let max_n = 1usize.max((w / 16.0).floor() as usize);
    if rows.len() > max_n {
        let ranges: Vec<f64> = rows
            .iter()
            .map(|r| {
                let lo = r.1.iter().copied().fold(r.1[0], f64::min);
                let hi = r.1.iter().copied().fold(r.1[0], f64::max);
                hi - lo
            })
            .collect();
        rows = top_indices(&ranges, max_n)
            .into_iter()
            .map(|i| rows[i].clone())
            .collect();
    }
    let all = || rows.iter().flat_map(|r| r.1.iter().copied());
    let (Some(min), Some(max)) = (all().reduce(f64::min), all().reduce(f64::max)) else {
        return Vec::new();
    };
    if max == min {
        return Vec::new();
    }
    let slot = w / rows.len() as f64;
    let span = max - min;
    let bw = span / 12.0;
    if bw.is_nan() || bw <= 0.0 {
        return Vec::new();
    }
    let plot_h = match h - axis_band(h) {
        p if p > 0.0 => p,
        _ => h,
    };
    const STEPS: usize = 24;
    let mut out = Vec::new();
    for (i, (id, values)) in rows.iter().enumerate() {
        let dens: Vec<f64> = (0..=STEPS)
            .map(|j| kde_at(values, min + (span * j as f64) / STEPS as f64, bw))
            .collect();
        let peak = dens.iter().copied().fold(0.0, f64::max);
        if peak.is_nan() || peak <= 0.0 {
            continue;
        }
        let mid = (i as f64 + 0.5) * slot;
        let at = |j: usize, side: f64| {
            let y = plot_h - (j as f64 / STEPS as f64) * plot_h;
            (mid + side * (dens[j] / peak) * (slot * 0.42), y)
        };
        let points = (0..=STEPS)
            .map(|j| at(j, -1.0))
            .chain((0..=STEPS).rev().map(|j| at(j, 1.0)))
            .collect();
        out.push(Violin {
            points,
            label: Label {
                id: (*id).to_owned(),
                label: (*id).to_owned(),
                lx: mid,
                ly: h,
                align: "center",
                baseline: "bottom",
                max_w: slot.max(8.0),
            },
        });
    }
    out
}

/// A wedge of the rose: its centre, radius and angles, and its caption.
#[derive(Debug, Clone, PartialEq)]
pub struct Wedge {
    pub value: f64,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub start: f64,
    pub end: f64,
    pub label: Label,
}

/// `nightingaleWedges(items, width, height)`: equal angles, each wedge's area its value.
pub fn nightingale_wedges(items: &[(Value, f64)], w: f64, h: f64) -> Vec<Wedge> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || items.is_empty() {
        return Vec::new();
    }
    let rows: Vec<(String, f64)> = items
        .iter()
        .enumerate()
        .filter(|(_, (_, n))| n.is_finite() && *n >= 0.0)
        .map(|(i, (id, n))| {
            let id = if id.is_null() {
                i.to_string()
            } else {
                js::string(id)
            };
            (id, *n)
        })
        .collect();
    if rows.is_empty() {
        return Vec::new();
    }
    let max = rows.iter().map(|r| r.1).fold(0.0, f64::max);
    let (cx, cy) = (w / 2.0, h / 2.0);
    let small = w.min(h);
    let mut max_r = small / 2.0 - axis_band(small);
    if max_r <= 0.0 {
        max_r = small / 2.0 - 4.0;
    }
    if max_r <= 0.0 {
        return Vec::new();
    }
    let theta = (PI * 2.0) / rows.len() as f64;
    let max_area = 0.5 * max_r * max_r * theta;
    rows.into_iter()
        .enumerate()
        .map(|(i, (id, value))| {
            let area = if max > 0.0 {
                (value / max) * max_area
            } else {
                0.0
            };
            let start = i as f64 * theta - PI / 2.0;
            let end = (i as f64 + 1.0) * theta - PI / 2.0;
            let mid = start + theta / 2.0;
            let (ox, oy) = (mid.cos(), mid.sin());
            Wedge {
                value,
                cx,
                cy,
                r: ((2.0 * area) / theta).sqrt(),
                start,
                end,
                label: Label {
                    id: id.clone(),
                    label: id,
                    lx: (cx + ox * (max_r + 3.0)).max(0.0).min(w),
                    ly: (cy + oy * (max_r + 3.0)).max(0.0).min(h),
                    align: if ox > 0.35 {
                        "left"
                    } else if ox < -0.35 {
                        "right"
                    } else {
                        "center"
                    },
                    baseline: if oy > 0.35 {
                        "top"
                    } else if oy < -0.35 {
                        "bottom"
                    } else {
                        "middle"
                    },
                    max_w: (max_r * theta).max(8.0),
                },
            }
        })
        .collect()
}
