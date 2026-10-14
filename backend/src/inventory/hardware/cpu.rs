//! CPU model, topology and clocks from /proc/cpuinfo, lscpu and cpufreq.

use super::{clean, key_values, numeric_prefix, Sys};
use serde_json::{json, Value};
use std::collections::BTreeMap;

struct Cpu {
    model: String,
    vendor: String,
    arch: String,
    cores: i64,
    threads: i64,
    sockets: i64,
    mhz: i64,
    max_mhz: i64,
    caches: String,
    flags: Vec<String>,
    virtualization: String,
    hypervisor: String,
}

pub fn collect(sys: &Sys) -> Value {
    let mut cpu = from_cpuinfo(&sys.text("proc/cpuinfo", 1 << 18));
    apply_lscpu(&mut cpu, &sys.run("lscpu", &[]));
    apply_cpufreq(&mut cpu, sys);
    json!({
        "model": cpu.model,
        "vendor": cpu.vendor,
        "arch": cpu.arch,
        "cores": cpu.cores,
        "threads": cpu.threads,
        "sockets": cpu.sockets,
        "mhz": cpu.mhz,
        "maxMhz": cpu.max_mhz,
        "caches": cpu.caches,
        "flags": cpu.flags,
        "virtualization": cpu.virtualization,
        "hypervisor": cpu.hypervisor,
    })
}

/// First non-empty of the candidates, like Python's `a or b or c`.
fn first<'a>(candidates: &[Option<&'a String>]) -> &'a str {
    candidates
        .iter()
        .flatten()
        .map(|s| s.as_str())
        .find(|s| !s.is_empty())
        .unwrap_or("")
}

fn from_cpuinfo(raw: &str) -> Cpu {
    let (mut model, mut vendor) = (String::new(), String::new());
    let mut flags: Vec<String> = Vec::new();
    let mut mhz = 0.0f64;
    let mut physical: BTreeMap<String, i64> = BTreeMap::new();
    let mut threads = 0;
    for block in raw.replace("\r\n", "\n").split("\n\n") {
        let kv = key_values(block);
        if !(kv.contains_key("processor")
            || kv.contains_key("model name")
            || kv.contains_key("Hardware"))
        {
            continue;
        }
        threads += 1;
        if model.is_empty() {
            model = clean(
                first(&[kv.get("model name"), kv.get("Hardware"), kv.get("model")]),
                160,
            );
        }
        if vendor.is_empty() {
            vendor = clean(first(&[kv.get("vendor_id"), kv.get("Hardware")]), 80);
        }
        if flags.is_empty() {
            flags = kv.get("flags").map_or_else(Vec::new, |f| {
                f.split_whitespace().map(String::from).collect()
            });
        }
        let hz = kv
            .get("cpu MHz")
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0);
        if hz > mhz {
            mhz = hz;
        }
        let phys = kv
            .get("physical id")
            .filter(|p| !p.is_empty())
            .cloned()
            .unwrap_or_else(|| "0".into());
        let cores = kv
            .get("cpu cores")
            .and_then(|c| c.trim().parse::<i64>().ok())
            .unwrap_or(0);
        let entry = physical.entry(phys).or_insert(0);
        *entry = (*entry).max(if cores == 0 { 1 } else { cores });
    }
    let total: i64 = physical.values().sum();
    Cpu {
        model,
        vendor,
        arch: String::new(),
        cores: if total > 0 { total } else { threads },
        threads,
        sockets: physical.len().max(1) as i64,
        mhz: mhz.round_ties_even() as i64,
        max_mhz: 0,
        caches: String::new(),
        flags,
        virtualization: String::new(),
        hypervisor: String::new(),
    }
}

fn apply_lscpu(cpu: &mut Cpu, raw: &str) {
    if raw.is_empty() {
        return;
    }
    let kv = key_values(raw);
    let num = |key: &str| -> i64 {
        let prefix = numeric_prefix(kv.get(key).map_or("", |v| v));
        prefix.parse::<f64>().map_or(0, |n| n as i64)
    };
    if cpu.model.is_empty() {
        cpu.model = clean(first(&[kv.get("Model name"), kv.get("Model Name")]), 160);
    }
    if cpu.vendor.is_empty() {
        cpu.vendor = clean(first(&[kv.get("Vendor ID"), kv.get("BIOS Vendor ID")]), 80);
    }
    cpu.arch = clean(kv.get("Architecture").map_or("", |v| v), 40);
    let sockets = [num("Socket(s)"), cpu.sockets, 1]
        .into_iter()
        .find(|n| *n != 0)
        .unwrap_or(1);
    let per = num("Core(s) per socket");
    cpu.sockets = sockets;
    if per != 0 {
        cpu.cores = per * sockets;
    }
    cpu.threads = [num("CPU(s)"), cpu.threads]
        .into_iter()
        .find(|n| *n != 0)
        .unwrap_or(0);
    if cpu.mhz == 0 {
        cpu.mhz = num("CPU MHz");
    }
    cpu.max_mhz = num("CPU max MHz");
    let caches: Vec<String> = [
        ("L1d cache", "L1d"),
        ("L1i cache", "L1i"),
        ("L2 cache", "L2"),
        ("L3 cache", "L3"),
    ]
    .iter()
    .filter_map(|(key, label)| {
        kv.get(*key)
            .filter(|v| !v.is_empty())
            .map(|v| format!("{label} {v}"))
    })
    .collect();
    cpu.caches = caches.join(", ");
    if cpu.flags.is_empty() {
        cpu.flags = kv.get("Flags").map_or_else(Vec::new, |f| {
            f.split_whitespace().map(String::from).collect()
        });
    }
    cpu.virtualization = clean(kv.get("Virtualization").map_or("", |v| v), 40);
    cpu.hypervisor = clean(kv.get("Hypervisor vendor").map_or("", |v| v), 40);
}

fn apply_cpufreq(cpu: &mut Cpu, sys: &Sys) {
    let max_hz = sys.int("sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq");
    let cur_hz = sys.int("sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq");
    if max_hz != 0 && cpu.max_mhz == 0 {
        cpu.max_mhz = max_hz.div_euclid(1000);
    }
    if cur_hz != 0 && cpu.mhz == 0 {
        cpu.mhz = cur_hz.div_euclid(1000);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpuinfo_blocks_become_threads_and_cores() {
        let raw = "processor\t: 0\nmodel name\t: Fancy CPU\nvendor_id\t: Acme\nflags\t\t: a b c\ncpu MHz\t\t: 800.5\nphysical id\t: 0\ncpu cores\t: 2\n\nprocessor\t: 1\nmodel name\t: Fancy CPU\ncpu MHz\t\t: 3000.4\nphysical id\t: 0\ncpu cores\t: 2\n";
        let cpu = from_cpuinfo(raw);
        assert_eq!((cpu.threads, cpu.cores, cpu.sockets), (2, 2, 1));
        assert_eq!(cpu.model, "Fancy CPU");
        assert_eq!(cpu.mhz, 3000);
        assert_eq!(cpu.flags, ["a", "b", "c"]);
    }

    #[test]
    fn lscpu_fills_what_cpuinfo_lacks() {
        let mut cpu = from_cpuinfo("processor : 0\n");
        apply_lscpu(&mut cpu, "Architecture: x86_64\nSocket(s): 1\nCore(s) per socket: 8\nCPU(s): 16\nL2 cache: 8 MiB\nCPU max MHz: 5000.0000\n");
        assert_eq!((cpu.cores, cpu.threads, cpu.max_mhz), (8, 16, 5000));
        assert_eq!(cpu.caches, "L2 8 MiB");
        assert_eq!(cpu.arch, "x86_64");
    }
}
