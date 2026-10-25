//! Sentinel: rogue processes from the live sample. A process that stays over its CPU limit for a
//! few samples, one that holds a large share of memory, a port held open by a process that has
//! been up for days. This is `services/Sentinel.js` in Rust; nothing here reads the system, and
//! `tests/sentinel_parity.rs` runs both on the same frames until the QML page goes.
//!
//! The wording and the arithmetic are the JavaScript's, quirks included: `Math.round` rounds a
//! half up, `toFixed(1)` rounds an exact tie up where Rust's `{:.1}` rounds it to even, a missing
//! owner prints as `null`.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::live::{Latest, Process};

/// CPU has to stay over its threshold for this many samples in a row.
pub const SUSTAIN: u32 = 3;
/// The most rows the page lists.
pub const ROW_CAP: usize = 60;
const DAY: f64 = 86_400.0;

/// One of the three limits, by the name the page and the settings know it by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    CpuPercent,
    MemoryPercent,
    PortAgeDays,
}

impl Limit {
    pub const ALL: [Limit; 3] = [Limit::CpuPercent, Limit::MemoryPercent, Limit::PortAgeDays];

    /// The settings domain it is stored in (`backend/src/domain.rs`).
    pub fn domain(self) -> &'static str {
        match self {
            Limit::CpuPercent => "sentinelCpuPercent",
            Limit::MemoryPercent => "sentinelMemoryPercent",
            Limit::PortAgeDays => "sentinelPortAgeDays",
        }
    }

    pub fn default(self) -> i64 {
        match self {
            Limit::CpuPercent => 80,
            Limit::MemoryPercent => 20,
            Limit::PortAgeDays => 7,
        }
    }

    /// The least and the greatest it may be.
    pub fn range(self) -> (i64, i64) {
        match self {
            Limit::CpuPercent => (50, 100),
            Limit::MemoryPercent => (5, 90),
            Limit::PortAgeDays => (1, 90),
        }
    }

    pub fn step(self) -> i64 {
        match self {
            Limit::PortAgeDays => 1,
            _ => 5,
        }
    }
}

/// `clampThreshold`: a stored value brought into its range, or the default when it is not a
/// number (`null`, `""`, text).
pub fn clamp_threshold(limit: Limit, raw: Option<&Value>) -> i64 {
    let Some(n) = crate::live::finite_number(raw) else {
        return limit.default();
    };
    let (low, high) = limit.range();
    (js_round(n) as i64).clamp(low, high)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thresholds {
    pub cpu_percent: i64,
    pub memory_percent: i64,
    pub port_age_days: i64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cpu_percent: Limit::CpuPercent.default(),
            memory_percent: Limit::MemoryPercent.default(),
            port_age_days: Limit::PortAgeDays.default(),
        }
    }
}

impl Thresholds {
    /// `thresholds(src)`: `src` holds the stored values by domain (`sentinelCpuPercent`, ...);
    /// an unset or out-of-range one falls back to the default.
    pub fn from_stored(src: Option<&Value>) -> Self {
        let get = |l: Limit| clamp_threshold(l, src.and_then(|s| s.get(l.domain())));
        Self {
            cpu_percent: get(Limit::CpuPercent),
            memory_percent: get(Limit::MemoryPercent),
            port_age_days: get(Limit::PortAgeDays),
        }
    }

    pub fn get(&self, limit: Limit) -> i64 {
        match limit {
            Limit::CpuPercent => self.cpu_percent,
            Limit::MemoryPercent => self.memory_percent,
            Limit::PortAgeDays => self.port_age_days,
        }
    }
}

/// `Math.round`: a half goes up.
fn js_round(x: f64) -> f64 {
    let floor = x.floor();
    if x - floor >= 0.5 { floor + 1.0 } else { floor }
}

/// `x.toFixed(1)` for `x >= 0`. JavaScript takes the larger of two numbers that are equally
/// near, Rust the even one; a tie is a number that is odd over four, which a double holds
/// exactly, so it is found exactly and not by multiplying (0.15 times ten is 1.5, and 0.15 is
/// below it).
fn to_fixed1(x: f64) -> String {
    let quad = x * 4.0;
    if quad.fract() == 0.0 && quad.abs() < 1e15 && (quad as i64) % 2 != 0 {
        let tenths = (x * 10.0).ceil() as i64;
        return format!("{}.{}", tenths / 10, tenths % 10);
    }
    format!("{x:.1}")
}

/// `humanKb`: kilobytes as the biggest unit they reach.
pub fn human_kb(kb: f64) -> String {
    if !kb.is_finite() || kb < 0.0 {
        return String::new();
    }
    if kb < 1024.0 {
        return format!("{} kB", js_round(kb));
    }
    let mb = kb / 1024.0;
    if mb < 1024.0 {
        let shown = if mb >= 100.0 {
            format!("{}", js_round(mb))
        } else {
            to_fixed1(mb)
        };
        return format!("{shown} MB");
    }
    format!("{} GB", to_fixed1(mb / 1024.0))
}

/// `humanAge`: seconds as the unit that reads best.
pub fn human_age(sec: f64) -> String {
    if !sec.is_finite() || sec < 0.0 {
        return String::new();
    }
    if sec < 90.0 {
        return format!("{} s", js_round(sec));
    }
    if sec < 5400.0 {
        return format!("{} min", js_round(sec / 60.0));
    }
    if sec < DAY * 2.0 {
        return format!("{} h", js_round(sec / 3600.0));
    }
    format!("{} d", js_round(sec / DAY))
}

fn usable(p: &Process) -> bool {
    !p.kthread && p.pid > 1
}

/// When a process first went over the CPU limit and for how many samples in a row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Over {
    pub since: f64,
    pub n: u32,
}

/// `track`'s state: per pid, how long it has been over the limit. A pid that drops under it, or
/// is gone, is forgotten.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tracked {
    at: Option<f64>,
    over: HashMap<u64, Over>,
}

impl Tracked {
    pub fn over(&self, pid: u64) -> Option<Over> {
        self.over.get(&pid).copied()
    }

    /// The state after `latest`. The same sample passed twice leaves it alone, and no sample
    /// forgets everything.
    pub fn track(&self, latest: Option<&Latest>, th: &Thresholds) -> Tracked {
        let Some(latest) = latest else {
            return Tracked::default();
        };
        if self.at == Some(latest.at) {
            return self.clone();
        }
        let limit = th.cpu_percent as f64;
        let mut over = HashMap::new();
        for row in &latest.processes {
            let Some(cpu) = row.cpu else { continue };
            if !usable(row) || cpu < limit {
                continue;
            }
            let next = match self.over.get(&row.pid) {
                Some(old) => Over {
                    since: old.since,
                    n: old.n + 1,
                },
                None => Over {
                    since: latest.at,
                    n: 1,
                },
            };
            over.insert(row.pid, next);
        }
        Tracked {
            at: Some(latest.at),
            over,
        }
    }

    pub fn to_json(&self) -> Value {
        let over: serde_json::Map<String, Value> = self
            .over
            .iter()
            .map(|(pid, o)| (pid.to_string(), json!({ "since": o.since, "n": o.n })))
            .collect();
        json!({ "at": self.at, "over": over })
    }
}

/// What a finding is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Cpu,
    Memory,
    Port,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Cpu => "cpu",
            Kind::Memory => "memory",
            Kind::Port => "port",
        }
    }
}

/// A port held open, as far as `collapse` needs it.
#[derive(Debug, Clone, PartialEq)]
struct PortInfo {
    port: f64,
    reach: String,
    age: Option<f64>,
}

/// One row of the page: a process over a limit, or a listening port.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub id: String,
    pub kind: Kind,
    pub pid: Option<u64>,
    pub comm: String,
    pub cmdline: String,
    pub detail: String,
    pub ratio: f64,
    pub flagged: bool,
    pub proc: Option<Process>,
    /// Text that finds the row beside what it shows: the ports of a collapsed row.
    pub search: Option<String>,
    port: Option<PortInfo>,
}

impl Finding {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id, "kind": self.kind.name(), "pid": self.pid, "comm": self.comm,
            "cmdline": self.cmdline, "detail": self.detail, "ratio": self.ratio,
            "flagged": self.flagged, "search": self.search,
        })
    }
}

/// `reach`: "local only" matters more than the address, since it decides whether another
/// machine can reach the port.
fn reach(addr: Option<&str>) -> String {
    let a = addr.unwrap_or("");
    if a == "0.0.0.0" || a == "::" {
        "all interfaces".into()
    } else if a.starts_with("127.") || a == "::1" {
        "local only".into()
    } else {
        a.to_owned()
    }
}

/// A listener, once per process and port. One whose process is unreadable keeps its row with no
/// pid.
struct Listener<'a> {
    port: f64,
    reach: String,
    uid: Option<f64>,
    pid: Option<u64>,
    proc: Option<&'a Process>,
    age: Option<f64>,
}

/// `s.pid || null`, as a pid: zero and nothing are both nothing.
fn truthy_pid(pid: Option<f64>) -> Option<f64> {
    pid.filter(|p| *p != 0.0)
}

fn number_text(n: f64) -> String {
    format!("{n}")
}

fn optional_text(n: Option<f64>) -> String {
    n.map_or_else(|| "null".into(), number_text)
}

fn listeners(latest: &Latest) -> Vec<Listener<'_>> {
    let by_pid: HashMap<u64, &Process> = latest.processes.iter().map(|p| (p.pid, p)).collect();
    let clk = if latest.clk_tck > 0.0 {
        latest.clk_tck
    } else {
        100.0
    };
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for s in &latest.tcp_sockets {
        let Some(port) = s.port else { continue };
        if s.state != "LISTEN" {
            continue;
        }
        let pid = truthy_pid(s.pid);
        let owner = match pid {
            Some(p) => number_text(p),
            None => format!("?{}", optional_text(s.uid)),
        };
        if !seen.insert(format!("{owner}|{}", number_text(port))) {
            continue;
        }
        let proc = pid.and_then(|p| by_pid.get(&(p as u64)).copied());
        let age = proc.and_then(|p| {
            let start = p.start_ticks?;
            let age = latest.uptime_sec? - start / clk;
            (age >= 0.0).then_some(age)
        });
        out.push(Listener {
            port,
            reach: reach(s.addr.as_deref()),
            uid: s.uid,
            pid: pid.map(|p| p as u64),
            proc,
            age,
        });
    }
    out
}

fn port_detail(l: &Listener) -> String {
    let mut bits = vec![format!(":{} · {}", number_text(l.port), l.reach)];
    if l.pid.is_none() {
        bits.push(format!("owner unknown (uid {})", optional_text(l.uid)));
    } else if let Some(age) = l.age {
        bits.push(format!("process up {}", human_age(age)));
    }
    bits.join(" · ")
}

fn port_row(l: &Listener, th: &Thresholds, flagged: bool) -> Finding {
    let limit = th.port_age_days as f64 * DAY;
    let owner = match l.pid {
        Some(p) => p.to_string(),
        None => format!("u{}", optional_text(l.uid)),
    };
    Finding {
        id: format!("port:{}:{owner}", number_text(l.port)),
        kind: Kind::Port,
        pid: l.pid,
        comm: l
            .proc
            .map_or_else(|| format!(":{}", number_text(l.port)), |p| p.comm.clone()),
        cmdline: l.proc.map_or_else(String::new, |p| p.cmdline.clone()),
        detail: port_detail(l),
        ratio: l.age.map_or(0.0, |age| age / limit),
        flagged,
        proc: l.proc.cloned(),
        search: None,
        port: Some(PortInfo {
            port: l.port,
            reach: l.reach.clone(),
            age: l.age,
        }),
    }
}

/// `rows`: every finding of a sample, flagged or not (a port is listed whatever its age).
pub fn rows(latest: Option<&Latest>, tracked: &Tracked, th: &Thresholds) -> Vec<Finding> {
    let Some(latest) = latest else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in latest.processes.iter().filter(|p| usable(p)) {
        if let Some(st) = tracked.over(p.pid)
            && st.n >= SUSTAIN
        {
            let cpu = p.cpu.unwrap_or(0.0);
            out.push(Finding {
                id: format!("cpu:{}", p.pid),
                kind: Kind::Cpu,
                pid: Some(p.pid),
                comm: p.comm.clone(),
                cmdline: p.cmdline.clone(),
                detail: format!(
                    "{}% CPU for {}",
                    js_round(cpu),
                    human_age((latest.at - st.since) / 1000.0)
                ),
                ratio: cpu / th.cpu_percent as f64,
                flagged: true,
                proc: Some(p.clone()),
                search: None,
                port: None,
            });
        }
        if let (Some(total), Some(rss)) = (latest.mem_total, p.rss_kb)
            && total > 0.0
        {
            let pct = 100.0 * rss / total;
            if pct >= th.memory_percent as f64 {
                out.push(Finding {
                    id: format!("memory:{}", p.pid),
                    kind: Kind::Memory,
                    pid: Some(p.pid),
                    comm: p.comm.clone(),
                    cmdline: p.cmdline.clone(),
                    detail: format!("{}, {}% of memory", human_kb(rss), js_round(pct)),
                    ratio: pct / th.memory_percent as f64,
                    flagged: true,
                    proc: Some(p.clone()),
                    search: None,
                    port: None,
                });
            }
        }
    }
    let limit = th.port_age_days as f64 * DAY;
    for l in listeners(latest) {
        let flagged = l.age.is_some_and(|age| age >= limit);
        out.push(port_row(&l, th, flagged));
    }
    out
}

fn by_ratio_desc(a: &Finding, b: &Finding) -> std::cmp::Ordering {
    if b.ratio != a.ratio {
        return b
            .ratio
            .partial_cmp(&a.ratio)
            .unwrap_or(std::cmp::Ordering::Equal);
    }
    a.id.cmp(&b.id)
}

/// `collapse`: a process with several old listeners is one finding, not one per port.
fn collapse(src: Vec<Finding>) -> Vec<Finding> {
    let mut out: Vec<Finding> = Vec::new();
    // Per pid: the row first seen, where it is in `out`, and every port row of the process.
    let mut groups: Vec<(u64, usize, Vec<Finding>)> = Vec::new();
    for r in src {
        let Some(pid) = r.pid.filter(|_| r.kind == Kind::Port && r.flagged) else {
            out.push(r);
            continue;
        };
        match groups.iter_mut().find(|g| g.0 == pid) {
            Some(g) => g.2.push(r),
            None => {
                groups.push((pid, out.len(), vec![r.clone()]));
                out.push(r);
            }
        }
    }
    for (pid, at, ports) in groups {
        if ports.len() < 2 {
            continue;
        }
        let first = &ports[0];
        let info = |f: &Finding| f.port.clone().expect("a port row has its port");
        let mut shown: Vec<String> = ports
            .iter()
            .take(3)
            .map(|p| {
                let i = info(p);
                format!(":{} {}", number_text(i.port), i.reach)
            })
            .collect();
        if ports.len() > 3 {
            shown.push(format!("+{} more", ports.len() - 3));
        }
        let age = info(first).age.unwrap_or(0.0);
        out[at] = Finding {
            id: format!("ports:{pid}"),
            kind: Kind::Port,
            pid: first.pid,
            comm: first.comm.clone(),
            cmdline: first.cmdline.clone(),
            detail: format!("process up {} · {}", human_age(age), shown.join(", ")),
            ratio: first.ratio,
            flagged: true,
            proc: first.proc.clone(),
            search: Some(
                ports
                    .iter()
                    .map(|p| format!(":{}", number_text(info(p).port)))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            port: first.port.clone(),
        };
    }
    out
}

fn flagged_rows(all: &[Finding]) -> Vec<Finding> {
    collapse(all.iter().filter(|r| r.flagged).cloned().collect())
}

/// What the page shows: a row of the kind the chip names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// Everything over a threshold.
    #[default]
    Flagged,
    Cpu,
    Memory,
    /// Every listener, long-running ones first.
    Port,
}

impl Filter {
    pub const ALL: [Filter; 4] = [Filter::Flagged, Filter::Cpu, Filter::Memory, Filter::Port];

    pub fn id(self) -> &'static str {
        match self {
            Filter::Flagged => "flagged",
            Filter::Cpu => "cpu",
            Filter::Memory => "memory",
            Filter::Port => "port",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Filter::Flagged => "Flagged",
            Filter::Cpu => "CPU",
            Filter::Memory => "Memory",
            Filter::Port => "Ports",
        }
    }

    pub fn from_id(id: &str) -> Option<Filter> {
        Filter::ALL.into_iter().find(|f| f.id() == id)
    }
}

/// `list`: the rows to show for `filter` and a search `query`, most over their limit first,
/// at most [`ROW_CAP`].
pub fn list(all: &[Finding], filter: Filter, query: &str) -> Vec<Finding> {
    let q = query.to_lowercase();
    let mut out: Vec<Finding> = match filter {
        Filter::Flagged => flagged_rows(all),
        other => {
            let kind = match other {
                Filter::Cpu => Kind::Cpu,
                Filter::Memory => Kind::Memory,
                _ => Kind::Port,
            };
            all.iter()
                .filter(|r| r.kind == kind && (kind == Kind::Port || r.flagged))
                .cloned()
                .collect()
        }
    };
    if !q.is_empty() {
        out.retain(|r| {
            let text = format!(
                "{} {} {} {} {}",
                r.comm,
                r.cmdline,
                r.detail,
                r.pid.map_or_else(|| "null".to_owned(), |p| p.to_string()),
                r.search.as_deref().unwrap_or("")
            );
            text.to_lowercase().contains(&q)
        });
    }
    out.sort_by(by_ratio_desc);
    out.truncate(ROW_CAP);
    out
}

/// How many rows each chip holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub flagged: usize,
    pub cpu: usize,
    pub memory: usize,
    pub port: usize,
}

impl Counts {
    pub fn of(all: &[Finding]) -> Self {
        let mut c = Counts::default();
        for r in all {
            match r.kind {
                Kind::Port => c.port += 1,
                Kind::Cpu if r.flagged => c.cpu += 1,
                Kind::Memory if r.flagged => c.memory += 1,
                _ => {}
            }
        }
        c.flagged = flagged_rows(all).len();
        c
    }

    pub fn of_filter(&self, filter: Filter) -> usize {
        match filter {
            Filter::Flagged => self.flagged,
            Filter::Cpu => self.cpu,
            Filter::Memory => self.memory,
            Filter::Port => self.port,
        }
    }
}

#[cfg(test)]
#[path = "sentinel_tests.rs"]
mod tests;
