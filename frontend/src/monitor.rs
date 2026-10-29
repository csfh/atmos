//! The Monitor hub's and Home's words and marks (`pages/MonitorPage.qml`, `pages/HomePage.qml`,
//! `services/Monitor.js`): each meter's and row's caption, Home's hero line and GPU rows, the
//! Monitor pages' cards, and where the core bars and the memory bar are drawn.
//! `tests/monitor_parity.rs` runs the pages' own functions under Node against these.

use serde_json::Value;

use crate::stats::{self, finite};

/// `pageCards()`: each Monitor page's id, title and words.
pub const PAGES: &[(&str, &str, &str)] = &[
    (
        "processes",
        "Processes",
        "Search, sort, and signal this machine's tasks.",
    ),
    ("cpu", "CPU", "Per-core load, frequency, and pressure."),
    ("memory", "Memory", "Used, cache, buffers, swap, and stall."),
    ("disk", "Disk I/O", "Read and write rates from diskstats."),
    (
        "traffic",
        "Traffic",
        "Per-interface bandwidth and TCP counts.",
    ),
    ("sensors", "Sensors", "Temperatures, fans, and GPU load."),
];

fn f(row: Option<&Value>, k: &str) -> Option<f64> {
    row.and_then(|r| finite(r.get(k)))
}

fn bytes(kb: Option<&Value>) -> String {
    crate::hardware::format_bytes(stats::mem_bytes(kb).unwrap_or(f64::NAN))
}

/// `alertLevel(pct)`: hot at 90 and up.
pub fn hot(pct: Option<f64>) -> bool {
    pct.is_some_and(|p| p >= 90.0)
}

/// The Monitor hub's captions, given the newest row and whether none has come yet.
pub struct Captions<'a> {
    pub latest: Option<&'a Value>,
    pub waiting: bool,
}

impl Captions<'_> {
    fn or(&self, text: String) -> String {
        if !text.is_empty() {
            text
        } else if self.waiting {
            "waiting for samples".into()
        } else {
            "unknown".into()
        }
    }

    /// `cpuCaption`.
    pub fn cpu(&self) -> String {
        self.or(stats::format_percent(f(self.latest, "cpu")))
    }

    /// `memCaption`: used, of what is fitted.
    pub fn mem(&self) -> String {
        let l = self.latest;
        if f(l, "memUsed").is_none() || f(l, "memTotal").is_none() {
            return self.or(String::new());
        }
        let l = l.unwrap();
        let pct = stats::format_percent(f(Some(l), "mem"));
        let tail = if pct.is_empty() {
            String::new()
        } else {
            format!("  {pct}")
        };
        format!(
            "{} / {}{tail}",
            bytes(l.get("memUsed")),
            bytes(l.get("memTotal"))
        )
    }

    /// The Memory meter's caption: what is available.
    pub fn available(&self) -> String {
        f(self.latest, "memAvail").map_or_else(String::new, |_| {
            format!("{} available", bytes(self.latest.unwrap().get("memAvail")))
        })
    }

    /// `netCaption`.
    pub fn net(&self) -> String {
        self.or(stats::format_net(self.latest))
    }

    /// `diskCaption`: read and written across the disks.
    pub fn disk(&self) -> String {
        let disks = self
            .latest
            .and_then(|l| l.get("disks"))
            .and_then(Value::as_array);
        let (mut r, mut w, mut any) = (0.0, 0.0, false);
        for d in disks.into_iter().flatten() {
            if let Some(n) = d.get("readBps").and_then(Value::as_f64) {
                r += n;
                any = true;
            }
            if let Some(n) = d.get("writeBps").and_then(Value::as_f64) {
                w += n;
                any = true;
            }
        }
        if !any {
            return self.or(String::new());
        }
        format!(
            "↓ {}  ↑ {}",
            stats::format_bps(Some(r)),
            stats::format_bps(Some(w))
        )
    }

    /// `tempCaption`.
    pub fn temp(&self) -> String {
        self.or(stats::format_temp(f(self.latest, "cpuTemp")))
    }

    /// `loadCaption`.
    pub fn load(&self) -> String {
        self.or(stats::format_load_line(self.latest))
    }

    /// `coresCaption`.
    pub fn cores(&self) -> String {
        match stats::core_percents(self.latest).len() {
            0 if self.waiting => "waiting for samples".into(),
            0 => "no cores yet".into(),
            n => format!("{n} cores"),
        }
    }

    /// The Load meter's caption.
    pub fn load_cores(&self) -> String {
        match stats::core_percents(self.latest).len() {
            0 => String::new(),
            n => format!("1 / 5 / 15  ·  {n} cores"),
        }
    }

    /// `psiCaption`: the stall of each, where the kernel says.
    pub fn psi(&self) -> String {
        let Some(psi) = self
            .latest
            .and_then(|l| l.get("psi"))
            .filter(|p| p.is_object())
        else {
            return String::new();
        };
        [("cpu", "cpu"), ("memory", "mem"), ("io", "io")]
            .iter()
            .filter_map(|(k, name)| {
                finite(psi.get(*k)).map(|v| format!("{name} {}", stats::format_psi(Some(v))))
            })
            .collect::<Vec<_>>()
            .join("   ")
    }

    /// `tcpCaption`.
    pub fn tcp(&self) -> String {
        let Some(tcp) = self
            .latest
            .and_then(|l| l.get("tcp"))
            .filter(|t| t.is_object())
        else {
            return String::new();
        };
        let n = |k: &str| crate::js::string(tcp.get(k).unwrap_or(&Value::Null));
        format!(
            "{} established  ·  {} listen  ·  {} sockets",
            n("established"),
            n("listen"),
            n("total")
        )
    }
}

/// The Sampling line: how many samples, the stall, and the last error.
pub fn sampling(count: usize, psi: &str, error: &str) -> String {
    let mut out = if count > 0 {
        format!("{count} samples")
    } else {
        "No samples yet".into()
    };
    if !psi.is_empty() {
        out.push_str(&format!("  ·  stall {psi}"));
    }
    if !error.is_empty() {
        out.push_str(&format!("  ·  {error}"));
    }
    out
}

/// Home's captions, given the newest row and how many rows there are.
pub struct Home<'a> {
    pub latest: Option<&'a Value>,
    pub rows: usize,
}

const WAITING: &str = "waiting for samples";

impl Home<'_> {
    /// `cpuCaption`.
    pub fn cpu(&self) -> String {
        match stats::format_percent(f(self.latest, "cpu")) {
            p if p.is_empty() => WAITING.into(),
            p => p,
        }
    }

    /// `memCaption`.
    pub fn mem(&self) -> String {
        let l = self.latest;
        if f(l, "memUsed").is_none() || f(l, "memTotal").is_none() {
            return WAITING.into();
        }
        let l = l.unwrap();
        let pct = stats::format_percent(f(Some(l), "mem"));
        let tail = if pct.is_empty() {
            String::new()
        } else {
            format!(" ({pct})")
        };
        format!(
            "{} of {}{tail}",
            bytes(l.get("memUsed")),
            bytes(l.get("memTotal"))
        )
    }

    /// `netCaption`.
    pub fn net(&self) -> String {
        match stats::format_net(self.latest) {
            n if n.is_empty() => WAITING.into(),
            n => n,
        }
    }

    /// `cpuTempCaption`.
    pub fn temp(&self) -> String {
        if self.rows == 0 {
            return WAITING.into();
        }
        match stats::format_temp(f(self.latest, "cpuTemp")) {
            t if t.is_empty() => "unknown".into(),
            t => t,
        }
    }

    /// `gpuCaption(gpu)`: its own temperature, or the package's marked so.
    pub fn gpu(&self, gpu: &Value) -> String {
        if self.rows == 0 {
            return WAITING.into();
        }
        let Some(latest) = self.latest else {
            return "unknown".into();
        };
        match stats::gpu_temp_at(latest, gpu) {
            None => "unknown".into(),
            Some(t) if stats::gpu_own_temp(latest, gpu).is_none() => {
                format!("{} (package)", stats::format_temp(Some(t)))
            }
            Some(t) => stats::format_temp(Some(t)),
        }
    }

    /// `heroLine()`.
    pub fn hero(&self) -> String {
        if self.rows == 0 {
            return "Sampling live stats…".into();
        }
        let mut parts = Vec::new();
        let cpu = stats::format_percent(f(self.latest, "cpu"));
        if !cpu.is_empty() {
            parts.push(format!("CPU {cpu}"));
        }
        let mem = stats::format_percent(f(self.latest, "mem"));
        if !mem.is_empty() {
            parts.push(format!("MEM {mem}"));
        }
        let net = stats::format_net(self.latest);
        if !net.is_empty() {
            parts.push(net);
        }
        let temp = stats::format_temp(f(self.latest, "cpuTemp"));
        if !temp.is_empty() {
            parts.push(temp);
        }
        if parts.is_empty() {
            "Sampling live stats…".into()
        } else {
            parts.join(" · ")
        }
    }
}

/// `gpuName(gpu)`.
pub fn gpu_name(gpu: &Value) -> String {
    gpu.get("name")
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(|| "GPU".into(), crate::js::string)
}

/// `gpuDescription(gpu)`: the inventory's summary, else its vendor and driver.
pub fn gpu_says(gpu: &Value) -> String {
    let summary = crate::hardware::gpu_summary(gpu);
    if !summary.is_empty() {
        return summary;
    }
    ["vendor", "driver"]
        .iter()
        .filter_map(|k| {
            gpu.get(*k)
                .filter(|v| crate::js::truthy(Some(v)))
                .map(crate::js::string)
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// A mark's place: x, y, width, height.
pub type Rect = (f32, f32, f32, f32);

/// `barRects(values, width, height, 1)`: a bar per value from 0 to 100, and whether it is hot.
pub fn bars(values: &[f64], width: f32, height: f32) -> Vec<(Rect, bool)> {
    let n = values.len();
    if n == 0 || width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let gap = 1.0;
    let bw = (width - gap * (n as f32 - 1.0).max(0.0)) / n as f32;
    if bw <= 0.0 {
        return Vec::new();
    }
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let v = if v.is_finite() {
                v.clamp(0.0, 100.0)
            } else {
                0.0
            };
            let bh = (v / 100.0) as f32 * height;
            ((i as f32 * (bw + gap), height - bh, bw, bh), hot(Some(v)))
        })
        .collect()
}

/// `stackedRects(parts, width, height)`: each part's share of the bar.
pub fn stacked(parts: &[(&'static str, &'static str, f64)], width: f32, height: f32) -> Vec<Rect> {
    let total: f64 = parts.iter().map(|(_, _, kb)| kb.max(0.0)).sum();
    if width <= 0.0 || height <= 0.0 || total <= 0.0 {
        return Vec::new();
    }
    let mut x = 0.0;
    parts
        .iter()
        .map(|(_, _, kb)| {
            let w = (kb.max(0.0) / total) as f32 * width;
            let r = (x, 0.0, w, height);
            x += w;
            r
        })
        .collect()
}
