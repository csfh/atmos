//! One live sample of CPU, memory, network, temps, and user processes. No sudo.

mod gpu;
mod procs;
mod sockets;
mod system;

use crate::inventory::read_text;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where to read from: `ATMOS_SYS_ROOT` for fixtures, `/` otherwise.
pub(super) struct Sys(PathBuf);

impl Sys {
    fn from_env() -> Sys {
        let root = std::env::var("ATMOS_SYS_ROOT").unwrap_or_default();
        Sys(PathBuf::from(if root.is_empty() {
            "/".into()
        } else {
            root
        }))
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.0.join(rel.trim_start_matches('/'))
    }

    pub fn text(&self, rel: &str) -> String {
        self.text_limit(rel, 65536)
    }

    pub fn text_limit(&self, rel: &str, limit: usize) -> String {
        read_text(&self.path(rel), limit)
    }

    pub fn line(&self, rel: &str) -> String {
        line_of(&self.path(rel))
    }

    /// Sorted names in a directory; empty when it cannot be listed.
    pub fn names(&self, rel: &str) -> Vec<String> {
        sorted_names(&self.path(rel))
    }
}

pub(super) fn sorted_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// First line of a small file, trimmed.
pub(super) fn line_of(path: &Path) -> String {
    let text = read_text(path, 4096);
    text.lines()
        .next()
        .map(|l| l.trim().to_string())
        .unwrap_or_default()
}

/// `str.isdigit()` for ASCII sysfs values.
pub(super) fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// `raw.lstrip("-").isdigit()`
pub(super) fn is_int_text(text: &str) -> bool {
    is_digits(text.trim_start_matches('-'))
}

/// hwmon/thermal millidegrees. Missing or non-positive stays unknown.
pub(super) fn milli_to_c(raw: &str) -> Option<i64> {
    if !is_int_text(raw) {
        return None;
    }
    let milli: i64 = raw.parse().ok()?;
    if milli < 0 {
        return None;
    }
    let celsius = (milli + 500) / 1000;
    (celsius > 0).then_some(celsius)
}

fn uid_wanted() -> i64 {
    if let Some(uid) = std::env::var("ATMOS_UID").ok().filter(|v| is_digits(v)) {
        return uid.parse().unwrap_or(-1);
    }
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self").map_or(-1, |m| i64::from(m.uid()))
}

extern "C" {
    fn sysconf(name: i32) -> i64;
}

fn clk_tck() -> i64 {
    const SC_CLK_TCK: i32 = 2;
    // SAFETY: sysconf takes an int and returns a long; it has no pointer arguments.
    let n = unsafe { sysconf(SC_CLK_TCK) };
    if n > 0 {
        n
    } else {
        100
    }
}

fn int_or_null(value: Option<i64>) -> Value {
    value.map_or(Value::Null, Value::from)
}

fn parse_cpu_counts(line: &str) -> Option<(i64, i64)> {
    let mut nums = Vec::new();
    for part in line.split_whitespace().skip(1) {
        nums.push(part.parse::<i64>().ok()?);
    }
    if nums.len() < 4 {
        return None;
    }
    let idle = nums[3] + nums.get(4).copied().unwrap_or(0);
    let total: i64 = if nums.len() >= 8 {
        nums[..8].iter().sum()
    } else {
        nums.iter().sum()
    };
    (total > 0).then_some((idle, total))
}

fn read_cpu(sys: &Sys) -> Option<(i64, i64)> {
    sys.text("proc/stat")
        .lines()
        .find(|l| l.starts_with("cpu "))
        .and_then(parse_cpu_counts)
}

fn read_cpus(sys: &Sys) -> Vec<Value> {
    let mut out = Vec::new();
    for line in sys.text("proc/stat").lines() {
        if !line.starts_with("cpu") || line.starts_with("cpu ") {
            continue;
        }
        let name = line.split_whitespace().next().unwrap_or("");
        if !is_digits(&name[3..]) {
            continue;
        }
        let Some((idle, total)) = parse_cpu_counts(line) else {
            continue;
        };
        let idx: i64 = name[3..].parse().unwrap_or(0);
        let base = format!("sys/devices/system/cpu/cpu{idx}/cpufreq");
        let raw = sys.line(&format!("{base}/scaling_cur_freq"));
        let mhz = is_digits(&raw)
            .then(|| raw.parse::<i64>().unwrap_or(0) / 1000)
            .filter(|n| *n > 0);
        out.push(json!({
            "id": idx,
            "idle": idle,
            "total": total,
            "freqMhz": int_or_null(mhz),
            "governor": sys.line(&format!("{base}/scaling_governor")),
        }));
    }
    out
}

fn read_load(sys: &Sys) -> [Value; 3] {
    let line = sys.line("proc/loadavg");
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return [Value::Null, Value::Null, Value::Null];
    }
    let one = |p: &str| match p.parse::<f64>() {
        Ok(n) if n >= 0.0 => json!(n),
        _ => Value::Null,
    };
    [one(parts[0]), one(parts[1]), one(parts[2])]
}

fn read_mem(sys: &Sys) -> Map<String, Value> {
    let mut fields: BTreeMap<String, i64> = BTreeMap::new();
    for line in sys.text("proc/meminfo").lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        if let Ok(n) = parts[1].parse::<i64>() {
            fields.insert(parts[0].trim_end_matches(':').to_string(), n);
        }
    }
    let field = |key: &str| fields.get(key).copied().filter(|n| *n >= 0);
    let (total, avail) = (field("MemTotal"), field("MemAvailable"));
    let used = match (total, avail) {
        (Some(t), Some(a)) if t > 0 => Some((t - a).max(0)),
        _ => None,
    };
    let (swap_total, swap_free) = (field("SwapTotal"), field("SwapFree"));
    let swap_used = match (swap_total, swap_free) {
        (Some(t), Some(f)) => Some((t - f).max(0)),
        _ => None,
    };
    let (active, inactive) = (field("Active(file)"), field("Inactive(file)"));
    let file = (active.is_some() || inactive.is_some())
        .then(|| active.unwrap_or(0) + inactive.unwrap_or(0));
    let mut out = Map::new();
    for (key, value) in [
        ("memUsed", used),
        ("memTotal", total),
        ("memAvail", avail),
        ("memFree", field("MemFree")),
        ("memBuffers", field("Buffers")),
        ("memCached", field("Cached")),
        ("memShared", field("Shmem")),
        ("memSReclaimable", field("SReclaimable")),
        ("memAnon", field("AnonPages")),
        ("memDirty", field("Dirty")),
        ("memSlab", field("Slab")),
        ("memPageTables", field("PageTables")),
        ("memKernelStack", field("KernelStack")),
        ("memFile", file),
        ("swapUsed", swap_used),
        ("swapTotal", swap_total),
    ] {
        out.insert(key.into(), int_or_null(value));
    }
    out
}

/// One `/proc/net/dev` line as (interface, columns after the colon).
fn net_rows(sys: &Sys) -> Vec<(String, Vec<String>)> {
    sys.text("proc/net/dev")
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once(':')?;
            let iface = name.trim();
            (!iface.is_empty() && iface != "lo").then(|| {
                (
                    iface.to_string(),
                    rest.split_whitespace().map(String::from).collect(),
                )
            })
        })
        .collect()
}

fn read_net(sys: &Sys) -> (Value, Value) {
    if sys.text("proc/net/dev").is_empty() {
        return (Value::Null, Value::Null);
    }
    let (mut rx, mut tx, mut found) = (0i64, 0i64, false);
    for (_, cols) in net_rows(sys) {
        if cols.len() < 9 {
            continue;
        }
        let (Ok(r), Ok(t)) = (cols[0].parse::<i64>(), cols[8].parse::<i64>()) else {
            continue;
        };
        rx += r;
        tx += t;
        found = true;
    }
    if found {
        (json!(rx), json!(tx))
    } else {
        (Value::Null, Value::Null)
    }
}

fn read_ifaces(sys: &Sys) -> Vec<Value> {
    let mut out = Vec::new();
    for (name, cols) in net_rows(sys) {
        if cols.len() < 9 {
            continue;
        }
        let num = |i: usize| cols.get(i).map(|c| c.parse::<i64>());
        let (Some(Ok(rx)), Some(Ok(rx_packets)), Some(Ok(rx_err)), Some(Ok(rx_drop)), Some(Ok(tx))) =
            (num(0), num(1), num(2), num(3), num(8))
        else {
            continue;
        };
        let opt = |i: usize| match num(i) {
            None => Some(0),
            Some(Ok(n)) => Some(n),
            Some(Err(_)) => None,
        };
        let (Some(tx_packets), Some(tx_err), Some(tx_drop)) = (opt(9), opt(10), opt(11)) else {
            continue;
        };
        if rx < 0 || tx < 0 {
            continue;
        }
        out.push(json!({
            "name": name, "rx": rx, "tx": tx,
            "rxPackets": rx_packets, "txPackets": tx_packets,
            "rxErr": rx_err, "txErr": tx_err, "rxDrop": rx_drop, "txDrop": tx_drop,
        }));
    }
    out
}

/// `\d+` then each separator and `\d+` in turn, to the end of the text.
fn digit_groups(text: &str, seps: &[char]) -> bool {
    let mut rest = text;
    for (i, sep) in std::iter::once(None)
        .chain(seps.iter().map(Some))
        .enumerate()
    {
        if let Some(sep) = sep {
            let Some(after) = rest.strip_prefix(*sep) else {
                return false;
            };
            rest = after;
        }
        let tail = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        if tail.len() == rest.len() {
            return false;
        }
        rest = tail;
        let _ = i;
    }
    rest.is_empty()
}

/// `^(sd[a-z]+|vd[a-z]+|hd[a-z]+|xvd[a-z]+)\d+$|nvme\d+n\d+p\d+$|mmcblk\d+p\d+$`
/// (`re.search`, so the last two may start anywhere in the name).
fn is_partition(name: &str) -> bool {
    let classic = ["sd", "vd", "hd", "xvd"].iter().any(|prefix| {
        name.strip_prefix(prefix).is_some_and(|rest| {
            let digits = rest.trim_start_matches(|c: char| c.is_ascii_lowercase());
            digits.len() < rest.len() && digit_groups(digits, &[])
        })
    });
    let anywhere = |prefix: &str, seps: &[char]| {
        name.match_indices(prefix)
            .any(|(at, _)| digit_groups(&name[at + prefix.len()..], seps))
    };
    classic || anywhere("nvme", &['n', 'p']) || anywhere("mmcblk", &['p'])
}

fn read_disks(sys: &Sys) -> Vec<Value> {
    let mut out = Vec::new();
    for line in sys.text("proc/diskstats").lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 14 {
            continue;
        }
        let name = cols[2];
        if ["ram", "loop", "sr", "zram"]
            .iter()
            .any(|p| name.starts_with(p))
            || is_partition(name)
        {
            continue;
        }
        let (Ok(read), Ok(write)) = (cols[5].parse::<i64>(), cols[9].parse::<i64>()) else {
            continue;
        };
        if read < 0 || write < 0 {
            continue;
        }
        out.push(json!({"name": name, "readSectors": read, "writeSectors": write}));
    }
    out
}

fn psi_file(sys: &Sys, kind: &str) -> Value {
    for line in sys
        .text_limit(&format!("proc/pressure/{kind}"), 4096)
        .lines()
    {
        if !line.starts_with("some ") {
            continue;
        }
        for part in line.split_whitespace() {
            if let Some(value) = part.strip_prefix("avg10=") {
                return match value.parse::<f64>() {
                    Ok(n) if n >= 0.0 => json!(n),
                    _ => Value::Null,
                };
            }
        }
    }
    Value::Null
}

/// Seconds since boot, the first column of /proc/uptime.
fn read_uptime(sys: &Sys) -> Value {
    sys.line("proc/uptime")
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|n| n.is_finite() && *n >= 0.0)
        .map_or(Value::Null, |n| json!(n))
}

pub fn collect() -> Value {
    collect_from(&Sys::from_env())
}

fn collect_from(sys: &Sys) -> Value {
    let uid = uid_wanted();
    let cpu = read_cpu(sys);
    let (rx, tx) = read_net(sys);
    let [load1, load5, load15] = read_load(sys);
    let mut out = Map::new();
    out.insert("uid".into(), json!(uid));
    out.insert("cpuIdle".into(), int_or_null(cpu.map(|c| c.0)));
    out.insert("cpuTotal".into(), int_or_null(cpu.map(|c| c.1)));
    out.insert("cpus".into(), Value::Array(read_cpus(sys)));
    out.insert("load1".into(), load1);
    out.insert("load5".into(), load5);
    out.insert("load15".into(), load15);
    out.extend(read_mem(sys));
    out.insert("netRx".into(), rx);
    out.insert("netTx".into(), tx);
    out.insert("ifaces".into(), Value::Array(read_ifaces(sys)));
    out.insert("disks".into(), Value::Array(read_disks(sys)));
    out.insert(
        "psi".into(),
        json!({"cpu": psi_file(sys, "cpu"), "memory": psi_file(sys, "memory"), "io": psi_file(sys, "io")}),
    );
    out.insert("tcp".into(), sockets::read_tcp(sys));
    out.insert(
        "tcpSockets".into(),
        Value::Array(sockets::read_tcp_sockets(sys)),
    );
    out.insert("uptimeSec".into(), read_uptime(sys));
    out.insert(
        "interrupts".into(),
        system::irq_table(&sys.text("proc/interrupts")),
    );
    out.insert(
        "softirqs".into(),
        system::irq_table(&sys.text("proc/softirqs")),
    );
    out.insert("buddy".into(), system::buddy(sys));
    out.insert("cgroups".into(), system::cgroups(sys));
    out.insert("rapl".into(), Value::Array(system::rapl(sys)));
    out.insert("slabs".into(), Value::Array(system::slabs(sys)));
    out.insert("clkTck".into(), json!(clk_tck()));
    out.insert("cpuTemp".into(), int_or_null(system::cpu_temp(sys)));
    out.insert("gpus".into(), Value::Array(gpu::gpus(sys)));
    out.insert("sensors".into(), Value::Array(system::sensors(sys)));
    out.insert("processes".into(), Value::Array(procs::processes(sys, uid)));
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partitions_are_told_from_whole_disks() {
        for part in ["sda1", "vdb12", "nvme0n1p2", "mmcblk0p1", "xvda3"] {
            assert!(is_partition(part), "{part}");
        }
        for disk in ["sda", "nvme0n1", "mmcblk0", "dm-0", "md127"] {
            assert!(!is_partition(disk), "{disk}");
        }
    }

    #[test]
    fn cpu_counts_follow_the_script() {
        assert_eq!(parse_cpu_counts("cpu  1 2 3 4 5 6 7 8 9 10"), Some((9, 36)));
        assert_eq!(parse_cpu_counts("cpu  1 2 3 4"), Some((4, 10)));
        assert_eq!(parse_cpu_counts("cpu  1 2 3"), None);
        assert_eq!(parse_cpu_counts("cpu  0 0 0 0"), None);
    }

    #[test]
    fn millidegrees_round_and_hide_nonsense() {
        assert_eq!(milli_to_c("45500"), Some(46));
        assert_eq!(milli_to_c("0"), None);
        assert_eq!(milli_to_c("-5000"), None);
        assert_eq!(milli_to_c("x"), None);
    }
}
