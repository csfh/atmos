//! CPU, memory, chipset, and the rest of the machine. No sudo.

mod cpu;
mod devices;
mod dmi;
mod pci;

use super::{capture, read_int, read_strip, read_text};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;

/// Where to read from: `ATMOS_SYS_ROOT` for fixtures, `/` otherwise.
pub(super) struct Sys {
    root: PathBuf,
    skip_cmds: bool,
}

impl Sys {
    fn from_env() -> Sys {
        let root = std::env::var("ATMOS_SYS_ROOT").unwrap_or_default();
        Sys {
            root: PathBuf::from(if root.is_empty() { "/".into() } else { root }),
            skip_cmds: std::env::var("ATMOS_HW_SKIP_CMDS").is_ok_and(|v| !v.is_empty()),
        }
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel.trim_start_matches('/'))
    }

    pub fn run(&self, program: &str, args: &[&str]) -> String {
        if self.skip_cmds {
            return String::new();
        }
        capture(program, args)
    }

    pub fn strip(&self, rel: &str) -> String {
        read_strip(&self.path(rel), 512)
    }

    pub fn int(&self, rel: &str) -> i64 {
        read_int(&self.path(rel))
    }

    pub fn text(&self, rel: &str, limit: usize) -> String {
        read_text(&self.path(rel), limit)
    }

    pub fn dir_names(&self, rel: &str) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.path(rel))
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }
}

pub(super) struct Hw;

impl Hw {
    /// Keep a value already found; fill an empty field from a later source.
    pub fn fill(hw: &mut Value, section: &str, key: &str, value: &str) {
        let empty = hw[section][key].as_str().is_none_or(str::is_empty);
        if empty && !value.is_empty() {
            hw[section][key] = json!(value);
        }
    }
}

const PLACEHOLDERS: &[&str] = &[
    "not specified",
    "unknown",
    "none",
    "to be filled by o.e.m.",
    "default string",
    "fill by oem",
    "system product name",
    "system manufacturer",
    "oem",
    "1234567890",
];

/// A firmware string, or empty when it is a vendor placeholder.
pub(super) fn clean(value: &str, limit: usize) -> String {
    let text: String = {
        let mut out = String::new();
        let mut in_run = false;
        for c in value.chars() {
            if matches!(c, '\r' | '\n' | '\t') {
                if !in_run {
                    out.push(' ');
                }
                in_run = true;
            } else {
                in_run = false;
                out.push(c);
            }
        }
        out.trim().to_string()
    };
    if text.is_empty() {
        return String::new();
    }
    let lower = text.to_lowercase();
    if PLACEHOLDERS.contains(&lower.as_str())
        || text.chars().all(|c| c == '0')
        || text.chars().all(|c| c == 'f' || c == 'F')
    {
        return String::new();
    }
    text.chars().take(limit).collect()
}

pub(super) fn key_values(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in raw.lines() {
        let Some(cut) = line.find(':').filter(|c| *c >= 1) else {
            continue;
        };
        let key = line[..cut].trim();
        if !key.is_empty() {
            out.insert(key.to_string(), line[cut + 1..].trim().to_string());
        }
    }
    out
}

/// The leading `[0-9.]*` of a value, the way `re.sub(r"[^0-9.].*", "", v)` leaves it.
pub(super) fn numeric_prefix(value: &str) -> &str {
    let end = value
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(value.len());
    &value[..end]
}

pub(super) fn kb_to_bytes(value: Option<&String>) -> i64 {
    value
        .and_then(|v| numeric_prefix(v).parse::<f64>().ok())
        .map_or(0, |kb| (kb * 1024.0) as i64)
}

fn meminfo(sys: &Sys) -> Value {
    let kv = key_values(&sys.text("proc/meminfo", 16384));
    let total = kb_to_bytes(kv.get("MemTotal"));
    let mut available = kb_to_bytes(kv.get("MemAvailable"));
    if available == 0 {
        available = kb_to_bytes(kv.get("MemFree"))
            + kb_to_bytes(kv.get("Cached"))
            + kb_to_bytes(kv.get("Buffers"));
    }
    available = available.min(total);
    let swap_total = kb_to_bytes(kv.get("SwapTotal"));
    let swap_free = kb_to_bytes(kv.get("SwapFree"));
    json!({
        "total": total,
        "used": (total - available).max(0),
        "available": available,
        "swapTotal": swap_total,
        "swapUsed": (swap_total - swap_free).max(0),
        "modules": [],
    })
}

fn empty(sys: &Sys) -> Value {
    json!({
        "machine": {"vendor": "", "name": "", "family": "", "version": "", "serial": "", "sku": "", "chassis": ""},
        "board": {"vendor": "", "name": "", "version": "", "serial": ""},
        "chipset": {"name": "", "vendor": "", "pciId": "", "role": "", "southbridge": ""},
        "bios": {"vendor": "", "version": "", "date": "", "uefi": sys.path("sys/firmware/efi").is_dir()},
        "memory": {"total": 0, "used": 0, "available": 0, "swapTotal": 0, "swapUsed": 0, "modules": []},
        "tpm": {"present": false, "version": ""},
        "secureBoot": {"available": false, "enabled": false},
        "virtualization": {"hypervisor": "", "guest": false, "kvm": false},
    })
}

pub fn collect() -> Value {
    collect_from(&Sys::from_env())
}

fn collect_from(sys: &Sys) -> Value {
    let mut hw = empty(sys);
    hw["memory"] = meminfo(sys);
    hw["cpu"] = cpu::collect(sys);
    dmi::apply_id(&mut hw, &sys.path(""));
    dmi::apply_table(&mut hw, &sys.path(""));
    let found = pci::devices(sys);
    hw["chipset"] = pci::chipset(&found);
    hw["gpus"] = devices::gpus(sys, &found);
    hw["npus"] = devices::npus(&found);
    hw["pci"] = pci::summary(&found);
    hw["nics"] = devices::nics(sys);
    hw["audio"] = devices::audio(sys);
    hw["usb"] = devices::usb(sys);
    hw["batteries"] = devices::batteries(sys);
    hw["thermals"] = devices::thermals(sys);
    hw["tpm"] = devices::tpm(sys);
    hw["secureBoot"] = devices::secure_boot(sys);
    hw["virtualization"] = devices::virt(sys, &hw["cpu"]);
    hw
}
