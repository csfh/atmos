//! The Traffic and Processes pages' chart layouts (`services/Charts.js`): the TCP beeswarm,
//! the netdev radar, and the processes' parallel coordinates. `tests/charts_parity.rs` holds each
//! to its JavaScript.

use serde_json::Value;

use crate::charts::axis_band;
use crate::stats::non_neg;

/// `tcpBees(sample)`: each TCP socket, by its inode, grouped by its state.
pub fn tcp_bees(sample: Option<&Value>) -> Vec<(String, String)> {
    let sockets = sample
        .and_then(|s| s.get("tcpSockets"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    sockets
        .iter()
        .enumerate()
        .filter(|(_, s)| crate::js::truthy(Some(s)) && crate::js::truthy(s.get("state")))
        .map(|(i, s)| {
            let id = s
                .get("inode")
                .filter(|v| !v.is_null())
                .map_or_else(|| i.to_string(), crate::js::string);
            (id, crate::js::string(&s["state"]))
        })
        .collect()
}

/// `hash01(s)`: FNV-1a of the text's UTF-16 units, from 0 to 1.
pub fn hash01(s: &str) -> f64 {
    let h = s.encode_utf16().fold(2_166_136_261u32, |h, u| {
        (h ^ u32::from(u)).wrapping_mul(16_777_619)
    });
    f64::from(h) / 4_294_967_295.0
}

/// A beeswarm mark: its centre, its half-width, and its group.
#[derive(Debug, Clone, PartialEq)]
pub struct Bee {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub group: String,
    pub id: String,
}

/// `beeswarmLayout(items, width, height)`: a mark per item, jittered within its group's slot,
/// and each group's caption under its slot (centre and width).
pub fn beeswarm(items: &[(String, String)], w: f64, h: f64) -> (Vec<Bee>, Vec<(String, f64, f64)>) {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || items.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let mut groups: Vec<String> = Vec::new();
    for (_, g) in items {
        if !groups.contains(g) {
            groups.push(g.clone());
        }
    }
    let slot = w / groups.len() as f64;
    let r = (slot * 0.08).min(h * 0.06).clamp(2.0, 6.0);
    let pad = r + 1.0;
    let top = pad;
    let mut bot = axis_band(h) + pad;
    if top + bot >= h {
        bot = pad;
    }
    let jitter = (slot - 2.0 * pad).max(0.0) * 0.7;
    let span = (h - top - bot).max(0.0);
    let bees = items
        .iter()
        .map(|(id, g)| {
            let at = groups.iter().position(|x| x == g).unwrap_or(0) as f64;
            Bee {
                x: (at + 0.5) * slot + (hash01(&format!("{id}x")) - 0.5) * jitter,
                y: top + hash01(&format!("{id}y")) * span,
                r,
                group: g.clone(),
                id: id.clone(),
            }
        })
        .collect();
    let labels = groups
        .iter()
        .enumerate()
        .map(|(i, g)| (g.clone(), (i as f64 + 0.5) * slot, (slot - 2.0).max(8.0)))
        .collect();
    (bees, labels)
}

/// `netdevRadar(ifaces)`: each interface's bytes received and sent, packets, drops and errors.
pub fn netdev_radar(ifaces: &[Value]) -> Vec<(String, Vec<f64>)> {
    let n = |r: &Value, k: &str| non_neg(r.get(k)).unwrap_or(0.0);
    ifaces
        .iter()
        .filter(|r| crate::js::truthy(Some(r)) && crate::js::truthy(r.get("name")))
        .map(|r| {
            let values = vec![
                n(r, "rx"),
                n(r, "tx"),
                n(r, "rxPackets") + n(r, "txPackets"),
                n(r, "rxDrop") + n(r, "txDrop"),
                n(r, "rxErr") + n(r, "txErr"),
            ];
            (crate::js::string(&r["name"]), values)
        })
        .collect()
}

/// The axes `netdevRadar` is drawn on, in its order.
pub const NETDEV_AXES: &[&str] = &["rx", "tx", "packets", "drops", "errs"];

/// A radar spoke: its end, and where its caption goes and how it is aligned.
#[derive(Debug, Clone, PartialEq)]
pub struct Spoke {
    pub label: &'static str,
    pub end: (f64, f64),
    pub at: (f64, f64),
    pub align: &'static str,
    pub baseline: &'static str,
}

/// `radarPolygons(series, axes, width, height)`: the centre, the spokes, each series' polygon,
/// each scaled to the largest on its axis, and each polygon's caption.
#[allow(clippy::type_complexity)]
pub fn radar(
    series: &[(String, Vec<f64>)],
    axes: &[&'static str],
    w: f64,
    h: f64,
) -> (
    (f64, f64),
    Vec<Spoke>,
    Vec<(String, Vec<(f64, f64)>)>,
    Vec<(String, f64, f64)>,
) {
    let none = ((0.0, 0.0), Vec::new(), Vec::new(), Vec::new());
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || series.is_empty() || axes.len() < 3 {
        return none;
    }
    let (cx, cy) = (w / 2.0, h / 2.0);
    let mut r = w.min(h) / 2.0 - axis_band(w.min(h));
    if r.is_nan() || r <= 0.0 {
        r = w.min(h) / 2.0 - 4.0;
    }
    if r.is_nan() || r <= 0.0 {
        return none;
    }
    let value = |s: &(String, Vec<f64>), j: usize| s.1.get(j).copied().filter(|n| n.is_finite());
    let maxs: Vec<f64> = (0..axes.len())
        .map(|j| {
            series
                .iter()
                .filter_map(|s| value(s, j))
                .fold(0.0, f64::max)
        })
        .collect();
    let angle = |j: usize| {
        -std::f64::consts::PI / 2.0 + (j as f64 * 2.0 * std::f64::consts::PI) / axes.len() as f64
    };
    let spokes = axes
        .iter()
        .enumerate()
        .map(|(j, label)| {
            let (ox, oy) = (angle(j).cos(), angle(j).sin());
            Spoke {
                label,
                end: (cx + ox * r, cy + oy * r),
                at: (
                    (cx + ox * (r + 3.0)).clamp(0.0, w),
                    (cy + oy * (r + 3.0)).clamp(0.0, h),
                ),
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
            }
        })
        .collect();
    let mut polygons = Vec::new();
    let mut labels = Vec::new();
    for s in series {
        let pts: Vec<(f64, f64)> = (0..axes.len())
            .map(|j| {
                let n = value(s, j).filter(|n| *n >= 0.0).unwrap_or(0.0);
                let rr = if maxs[j] > 0.0 {
                    (n / maxs[j]) * r
                } else {
                    0.0
                };
                (cx + angle(j).cos() * rr, cy + angle(j).sin() * rr)
            })
            .collect();
        let (mut best, mut best_d) = (0, -1.0);
        for (j, (x, y)) in pts.iter().enumerate() {
            let d = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d > best_d {
                best_d = d;
                best = j;
            }
        }
        let (mx, my) = (
            pts.iter().map(|p| p.0).sum::<f64>() / pts.len() as f64,
            pts.iter().map(|p| p.1).sum::<f64>() / pts.len() as f64,
        );
        if !s.0.is_empty() && best_d >= 64.0 {
            let lx = (pts[best].0 * 0.55 + mx * 0.45).clamp(0.0, w);
            let ly = (pts[best].1 * 0.55 + my * 0.45).clamp(0.0, h);
            labels.push((s.0.clone(), lx, ly));
        }
        polygons.push((s.0.clone(), pts));
    }
    ((cx, cy), spokes, polygons, labels)
}

/// `processParallel(processes)`: each process with every field the chart reads, by pid, in the
/// order of `PARALLEL_AXES`.
pub fn process_parallel(processes: &[Value]) -> Vec<(String, Vec<f64>)> {
    use crate::stats::finite;
    processes
        .iter()
        .filter(|r| crate::js::truthy(Some(r)))
        .filter_map(|r| {
            let values = vec![
                finite(r.get("cpu"))?,
                non_neg(r.get("rssKb"))?,
                non_neg(r.get("fds"))?,
                non_neg(r.get("threads"))?,
                finite(r.get("nice"))?,
            ];
            Some((
                crate::js::string(r.get("pid").unwrap_or(&Value::Null)),
                values,
            ))
        })
        .collect()
}

/// The axes the processes' parallel coordinates are drawn on, in their order.
pub const PARALLEL_AXES: &[&str] = &["cpu", "rss", "fds", "threads", "nice"];

/// `topIndices(scores, limit)`: the indices of the highest scores, in their order.
pub(crate) fn top_indices(scores: &[f64], limit: usize) -> Vec<usize> {
    let cap = limit.max(1);
    let mut order: Vec<usize> = (0..scores.len()).collect();
    if scores.len() <= cap {
        return order;
    }
    order.sort_by(|a, b| {
        scores[*b]
            .partial_cmp(&scores[*a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(b))
    });
    order.truncate(cap);
    order.sort_unstable();
    order
}

/// `parallelPolylines(rows, axes, width, height)`: a line per row across the axes, each axis
/// scaled to its own range, keeping the most extreme rows when there are more than fit.
pub fn parallel_lines(
    rows: &[(String, Vec<f64>)],
    axes: usize,
    w: f64,
    h: f64,
) -> Vec<(String, Vec<(f64, f64)>)> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || rows.is_empty() || axes < 2 {
        return Vec::new();
    }
    let (mut top, mut plot) = (axis_band(h), h - axis_band(h));
    if plot.is_nan() || plot <= 0.0 {
        top = 0.0;
        plot = h;
    }
    let value = |r: &(String, Vec<f64>), j: usize| r.1.get(j).copied().filter(|n| n.is_finite());
    let range = |j: usize| {
        let all: Vec<f64> = rows.iter().filter_map(|r| value(r, j)).collect();
        let lo = all.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = all.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if all.is_empty() { (0.0, 0.0) } else { (lo, hi) }
    };
    let ranges: Vec<(f64, f64)> = (0..axes).map(range).collect();
    let complete: Vec<(String, Vec<f64>, f64)> = rows
        .iter()
        .filter_map(|r| {
            let vals: Option<Vec<f64>> = (0..axes).map(|j| value(r, j)).collect();
            let vals = vals?;
            let score = vals
                .iter()
                .zip(&ranges)
                .map(|(n, (lo, hi))| {
                    if hi - lo > 0.0 {
                        (n - lo) / (hi - lo)
                    } else {
                        0.0
                    }
                })
                .sum();
            Some((r.0.clone(), vals, score))
        })
        .collect();
    let keep = top_indices(
        &complete.iter().map(|c| c.2).collect::<Vec<_>>(),
        ((h / 4.0).floor() as usize).max(1),
    );
    keep.iter()
        .map(|&i| {
            let (id, vals, _) = &complete[i];
            let pts = vals
                .iter()
                .enumerate()
                .map(|(j, n)| {
                    let (lo, hi) = ranges[j];
                    let x = (w * j as f64) / (axes - 1) as f64;
                    let y = if hi - lo == 0.0 {
                        top + plot / 2.0
                    } else {
                        top + (1.0 - (n - lo) / (hi - lo)) * plot
                    };
                    (x, y)
                })
                .collect();
            (id.clone(), pts)
        })
        .collect()
}

/// `parallelRails(axes, width, height)`: each axis's rail (its x and top) and caption alignment.
pub fn parallel_rails(
    axes: &[&'static str],
    w: f64,
    h: f64,
) -> Vec<(&'static str, f64, f64, &'static str)> {
    if w.is_nan() || h.is_nan() || w <= 0.0 || h <= 0.0 || axes.len() < 2 {
        return Vec::new();
    }
    let top = if h - axis_band(h) > 0.0 {
        axis_band(h)
    } else {
        0.0
    };
    let last = axes.len() - 1;
    axes.iter()
        .enumerate()
        .map(|(j, a)| {
            let align = if j == 0 {
                "left"
            } else if j == last {
                "right"
            } else {
                "center"
            };
            (*a, (w * j as f64) / last as f64, top, align)
        })
        .collect()
}
