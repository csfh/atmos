//! The Monitor pages' rows (`pages/monitor/*Page.qml`): each row's value and the series its
//! sparkline draws, the disk, interface and sensor lists as filtered and sorted, the GPUs' words,
//! the processes each page lists, and the CPU histogram (`Monitor.js`). Their charts beyond these
//! (the heatmap, ridgeline, violin, rose, streamgraph, treemaps, Sankey, sunburst, icicle,
//! beeswarm, radar, calendar and waterfall) are not drawn yet. `tests/meters_parity.rs` runs the
//! pages' own code under Node against these.

use serde_json::Value;

use crate::processes::Filter;
use crate::stats::{self, finite};

/// What a page reads: the rows, and whether none has come yet.
pub struct Read<'a> {
    pub history: &'a [Value],
    pub waiting: bool,
}

/// A sparkline's series, its second, and whether it is hot.
pub type Series = (Vec<f64>, Vec<f64>, bool);

fn hot(v: Option<f64>) -> bool {
    crate::monitor::hot(v)
}

impl Read<'_> {
    fn latest(&self) -> Option<&Value> {
        self.history.last()
    }

    /// `waiting()`.
    pub fn waiting(&self) -> String {
        if self.waiting {
            "waiting for samples"
        } else {
            "unknown"
        }
        .into()
    }

    fn or(&self, text: String) -> String {
        if text.is_empty() {
            self.waiting()
        } else {
            text
        }
    }

    fn at(&self, k: &str) -> Option<f64> {
        self.latest().and_then(|l| finite(l.get(k)))
    }

    fn series(&self, k: &str, hot_too: bool) -> Option<Series> {
        Some((
            stats::series(self.history, k),
            Vec::new(),
            hot_too && hot(self.at(k)),
        ))
    }

    /// `kbText(kb)`.
    fn kb(&self, k: &str) -> String {
        match self
            .latest()
            .and_then(|l| l.get(k))
            .filter(|v| !v.is_null())
        {
            None => "unknown".into(),
            kb => crate::hardware::format_bytes(stats::mem_bytes(kb).unwrap_or(0.0)),
        }
    }

    fn psi(&self, which: &str) -> String {
        match self
            .latest()
            .and_then(|l| l.get("psi"))
            .filter(|p| crate::js::truthy(Some(p)))
        {
            None => self.waiting(),
            Some(psi) => match stats::format_psi(finite(psi.get(which))) {
                t if t.is_empty() => "unknown".into(),
                t => t,
            },
        }
    }

    fn tcp(&self, k: &str) -> String {
        match self
            .latest()
            .and_then(|l| l.get("tcp"))
            .filter(|t| crate::js::truthy(Some(t)))
        {
            None => self.waiting(),
            Some(tcp) => crate::js::string(tcp.get(k).unwrap_or(&Value::Null)),
        }
    }

    /// A row's value, and its sparkline if it has one, by row (`busy`, `load`, `cpupsi`, `used`,
    /// ... `temp`).
    pub fn meter(&self, id: &str) -> (String, Option<Series>) {
        let mem = |k: &str| {
            if self.latest().is_some() {
                self.kb(k)
            } else {
                self.waiting()
            }
        };
        match id {
            "busy" => (
                self.or(stats::format_percent(self.at("cpu"))),
                self.series("cpu", true),
            ),
            "load" => (
                self.or(stats::format_load_line(self.latest())),
                self.series("load1", false),
            ),
            "cpupsi" => (self.psi("cpu"), None),
            "mempsi" => (self.psi("memory"), None),
            "iopsi" => (self.psi("io"), None),
            "used" => (mem("memUsed"), None),
            "avail" => (mem("memAvail"), None),
            "free" => (mem("memFree"), None),
            "buffers" => (mem("memBuffers"), None),
            "cached" => (mem("memCached"), None),
            "anon" => (mem("memAnon"), None),
            "shared" => (mem("memShared"), None),
            "dirty" => (mem("memDirty"), None),
            "swap" => (self.swap(), self.series("swap", true)),
            "bandwidth" => (
                self.or(stats::format_net(self.latest())),
                Some((
                    stats::series(self.history, "rxBps"),
                    stats::series(self.history, "txBps"),
                    false,
                )),
            ),
            "established" => (self.tcp("established"), None),
            "listen" => (self.tcp("listen"), None),
            "timewait" => (self.tcp("timeWait"), None),
            "closewait" => (self.tcp("closeWait"), None),
            "temp" => (
                self.or(stats::format_temp(self.at("cpuTemp"))),
                self.series("cpuTemp", false),
            ),
            _ => (String::new(), None),
        }
    }

    /// The swap row's value: used of all, or off.
    fn swap(&self) -> String {
        let Some(l) = self.latest() else {
            return self.waiting();
        };
        if l.get("swapTotal").and_then(Value::as_f64) == Some(0.0) {
            return "off".into();
        }
        let pct = stats::format_percent(finite(l.get("swap")));
        let tail = if pct.is_empty() {
            String::new()
        } else {
            format!("  {pct}")
        };
        format!("{} / {}{tail}", self.kb("swapUsed"), self.kb("swapTotal"))
    }

    /// `diskLine(row)` and `ifaceLine(row)`: read and written, or received and sent.
    pub fn pair_line(&self, row: &Value, a: &str, b: &str) -> String {
        let (r, w) = (
            stats::format_bps(finite(row.get(a))),
            stats::format_bps(finite(row.get(b))),
        );
        if r.is_empty() && w.is_empty() {
            return self.waiting();
        }
        let or = |s: String| if s.is_empty() { "—".to_owned() } else { s };
        format!("↓ {}  ↑ {}", or(r), or(w))
    }

    /// `sensorText(row)`.
    pub fn sensor_line(&self, row: &Value) -> String {
        let text = if row.get("kind").and_then(Value::as_str) == Some("fan") {
            stats::format_rpm(finite(row.get("value")))
        } else {
            stats::format_temp(finite(row.get("value")))
        };
        if text.is_empty() {
            "unknown".into()
        } else {
            text
        }
    }

    /// A Sensors GPU's value: busy and temperature.
    pub fn gpu_line(&self, gpu: &Value) -> String {
        let latest = self.latest().unwrap_or(&Value::Null);
        let busy = stats::format_percent(stats::gpu_busy_at(latest, gpu));
        let temp = stats::format_temp(stats::gpu_temp_at(latest, gpu));
        match (busy.is_empty(), temp.is_empty()) {
            (false, false) => format!("{busy}  ·  {temp}"),
            (false, true) => busy,
            (true, false) => temp,
            _ => self.waiting(),
        }
    }

    /// A Sensors GPU's words: its driver and the VRAM in use.
    pub fn gpu_says(&self, gpu: &Value) -> String {
        let latest = self.latest().unwrap_or(&Value::Null);
        let live = stats::find_gpu(latest, gpu)
            .or_else(|| gpu.get("card").and_then(|c| stats::find_gpu(latest, c)));
        let vram = live
            .and_then(|l| Some((finite(l.get("vramUsed"))?, finite(l.get("vramTotal"))?)))
            .map(|(u, t)| {
                format!(
                    "{} / {}",
                    crate::hardware::format_bytes(u * 1024.0),
                    crate::hardware::format_bytes(t * 1024.0)
                )
            });
        let driver = gpu
            .get("driver")
            .filter(|d| crate::js::truthy(Some(d)))
            .map(crate::js::string);
        [driver, vram]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

fn name(row: &Value) -> String {
    crate::js::string(row.get("name").unwrap_or(&Value::Null))
}

/// The Disk page's disks, or the Traffic page's interfaces, filtered by name and sorted: by both
/// rates (`io`), by one (`read` over `readBps`, ...), or by name.
pub fn rates(rows: &[Value], filter: &str, sort: &str, keys: [(&str, &str); 2]) -> Vec<Value> {
    let q = filter.to_lowercase();
    let mut out: Vec<Value> = rows
        .iter()
        .filter(|r| {
            crate::js::truthy(Some(r)) && (q.is_empty() || name(r).to_lowercase().contains(&q))
        })
        .cloned()
        .collect();
    let rate = |r: &Value, k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    let [(a, ka), (b, kb)] = keys;
    let by = |x: &Value, y: &Value, f: &dyn Fn(&Value) -> f64| {
        f(y).partial_cmp(&f(x)).unwrap_or(std::cmp::Ordering::Equal)
    };
    out.sort_by(|x, y| match sort {
        s if s == a => by(x, y, &|r| rate(r, ka)),
        s if s == b => by(x, y, &|r| rate(r, kb)),
        "name" => name(x).cmp(&name(y)),
        _ => by(x, y, &|r| rate(r, ka) + rate(r, kb)),
    });
    out
}

/// The Sensors page's sensors: of a kind, filtered, temperatures first, then by label.
pub fn sensors(rows: &[Value], filter: &str, kind: &str) -> Vec<Value> {
    let q = filter.to_lowercase();
    let s = |r: &Value, k: &str| crate::js::string(r.get(k).unwrap_or(&Value::Null));
    let mut out: Vec<Value> = rows
        .iter()
        .filter(|r| crate::js::truthy(Some(r)))
        .filter(|r| kind == "all" || s(r, "kind") == kind)
        .filter(|r| {
            q.is_empty()
                || format!("{} {} {}", s(r, "chip"), s(r, "label"), s(r, "kind"))
                    .to_lowercase()
                    .contains(&q)
        })
        .cloned()
        .collect();
    out.sort_by(|x, y| {
        let (kx, ky) = (s(x, "kind"), s(y, "kind"));
        if kx != ky {
            return if kx == "temp" {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        s(x, "label").cmp(&s(y, "label"))
    });
    out
}

/// What each page lists of the processes.
pub fn procs(page: &str) -> Filter<'static> {
    let base = Filter {
        scope: "all",
        ..Filter::default()
    };
    match page {
        "cpu" => Filter {
            min_cpu: 1.0,
            cap: 12,
            ..base
        },
        "memory" => Filter {
            sort: "memory",
            min_rss_kb: 10240.0,
            cap: 16,
            ..base
        },
        _ => Filter {
            sort: "io",
            cap: 12,
            ..base
        },
    }
}

/// `cpuHistogram(rows)`: how many processes fall in each band of CPU, by label.
pub fn histogram(processes: &[Value]) -> Vec<(&'static str, usize)> {
    let mut bins = [
        ("idle", 0.0, 0.05, 0),
        ("<1%", 0.05, 1.0, 0),
        ("1–5%", 1.0, 5.0, 0),
        ("5–15%", 5.0, 15.0, 0),
        ("15–40%", 15.0, 40.0, 0),
        ("40%+", 40.0, 100_000.0, 0),
    ];
    for p in processes {
        let cpu = finite(p.get("cpu")).unwrap_or(0.0);
        if let Some(b) = bins
            .iter_mut()
            .find(|(_, lo, hi, _)| cpu >= *lo && cpu < *hi)
        {
            b.3 += 1;
        }
    }
    bins.iter().map(|(label, _, _, n)| (*label, *n)).collect()
}

/// `histogramRects(bins, width, height)`: each band's bar, the fullest at the top.
pub fn histogram_bars(
    bins: &[(&str, usize)],
    width: f32,
    height: f32,
) -> Vec<crate::monitor::Rect> {
    let max = bins
        .iter()
        .map(|(_, n)| *n)
        .max()
        .filter(|m| *m > 0)
        .unwrap_or(1) as f64;
    let scaled: Vec<f64> = bins.iter().map(|(_, n)| *n as f64 / max * 100.0).collect();
    let n = scaled.len();
    if n == 0 || width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let gap = 2.0;
    let bw = (width - gap * (n as f32 - 1.0)) / n as f32;
    if bw <= 0.0 {
        return Vec::new();
    }
    scaled
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let bh = (*v / 100.0) as f32 * height;
            (i as f32 * (bw + gap), height - bh, bw, bh)
        })
        .collect()
}
