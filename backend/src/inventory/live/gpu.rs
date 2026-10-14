//! GPUs from sysfs, with nvidia-smi filling in what the NVIDIA driver hides.

use super::{is_digits, milli_to_c, sorted_names, Sys};
use crate::runner::Run;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;

fn vendor_brand(id: &str) -> &'static str {
    match id {
        "0x1002" => "AMD",
        "0x10de" => "NVIDIA",
        "0x8086" => "Intel",
        "0x1a03" => "ASPEED",
        _ => "",
    }
}

fn driver_of(dev: &Path) -> String {
    std::fs::read_link(dev.join("driver"))
        .ok()
        .and_then(|t| t.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

fn pci_id_of(sys: &Sys, rel: &str) -> String {
    let hex = |file: &str| {
        sys.line(&format!("{rel}/{file}"))
            .to_lowercase()
            .replace("0x", "")
    };
    let (vendor, device) = (hex("vendor"), hex("device"));
    if vendor.is_empty() || device.is_empty() {
        String::new()
    } else {
        format!("{vendor}:{device}")
    }
}

fn slot_of(sys: &Sys, rel: &str) -> String {
    sys.text(&format!("{rel}/uevent"))
        .lines()
        .find_map(|l| {
            l.strip_prefix("PCI_SLOT_NAME=")
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_default()
}

fn slots_match(left: &str, right: &str) -> bool {
    let (a, b) = (left.to_lowercase(), right.to_lowercase());
    !a.is_empty() && !b.is_empty() && (a == b || a.ends_with(&b) || b.ends_with(&a))
}

fn drm_temp(dev: &Path) -> Option<i64> {
    let base = dev.join("hwmon");
    sorted_names(&base)
        .into_iter()
        .filter(|hw| hw.starts_with("hwmon"))
        .find_map(|hw| milli_to_c(&super::line_of(&base.join(hw).join("temp1_input"))))
}

#[derive(Clone)]
struct Smi {
    slot: String,
    temp: Option<i64>,
    busy: Option<i64>,
    vram_used: Option<i64>,
    vram_total: Option<i64>,
    name: String,
}

fn smi_int(raw: &str) -> Option<i64> {
    let s = raw.trim();
    is_digits(s).then(|| s.parse().ok()).flatten()
}

fn nvidia_rows() -> Vec<Smi> {
    let Ok(out) = Run::new("nvidia-smi")
        .args([
            "--query-gpu=pci.bus_id,temperature.gpu,utilization.gpu,memory.used,memory.total,name",
            "--format=csv,noheader,nounits",
        ])
        .timeout(Duration::from_secs(2))
        .output()
    else {
        return Vec::new();
    };
    out.stdout_text()
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').map(str::trim).collect();
            if parts.len() < 5 {
                return None;
            }
            Some(Smi {
                slot: parts[0].to_string(),
                temp: smi_int(parts[1]).filter(|t| *t > 0),
                busy: smi_int(parts[2]),
                vram_used: smi_int(parts[3]).map(|n| n * 1024),
                vram_total: smi_int(parts[4]).map(|n| n * 1024),
                name: if parts.len() > 5 {
                    parts[5..].join(",").trim().to_string()
                } else {
                    String::new()
                },
            })
        })
        .collect()
}

/// The row whose slot matches, else the only row, as `nvidia_field_for` did.
fn row_for<'a>(slot: &str, rows: &'a [Smi]) -> Option<&'a Smi> {
    rows.iter()
        .find(|r| slots_match(slot, &r.slot))
        .or_else(|| (rows.len() == 1).then(|| &rows[0]))
}

pub fn gpus(sys: &Sys) -> Vec<Value> {
    struct Card {
        value: Value,
        slot: String,
        nvidia: bool,
    }
    let mut cards = Vec::new();
    for name in sys.names("sys/class/drm") {
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let rel = format!("sys/class/drm/{name}/device");
        let dev = sys.path(&rel);
        if !dev.is_dir() {
            continue;
        }
        let driver = driver_of(&dev);
        let vendor_id = sys.line(&format!("{rel}/vendor")).to_lowercase();
        let brand = match vendor_brand(&vendor_id) {
            "" if driver == "asahi" => "Apple",
            b => b,
        };
        let slot = slot_of(sys, &rel);
        let bus = if slot.matches(':').count() >= 2 {
            slot.split(':').nth(1).unwrap_or("")
        } else {
            ""
        };
        let integrated = driver == "i915" || driver == "asahi" || (driver == "xe" && bus == "00");
        let nvidia = driver == "nvidia";
        let (mut busy, mut used, mut total) = (None, None, None);
        if !nvidia {
            let raw = sys.line(&format!("{rel}/gpu_busy_percent"));
            busy = is_digits(&raw)
                .then(|| raw.parse::<i64>().ok())
                .flatten()
                .filter(|n| (0..=100).contains(n));
            let kib = |file: &str| {
                let raw = sys.line(&format!("{rel}/{file}"));
                is_digits(&raw)
                    .then(|| raw.parse::<i64>().ok().map(|n| n / 1024))
                    .flatten()
            };
            used = kib("mem_info_vram_used");
            total = kib("mem_info_vram_total");
        }
        let label = [brand, driver.as_str(), name.as_str()]
            .into_iter()
            .find(|s| !s.is_empty())
            .unwrap_or("");
        let vendor = if brand.is_empty() {
            vendor_id.as_str()
        } else {
            brand
        };
        cards.push(Card {
            value: json!({
                "card": name,
                "pciId": pci_id_of(sys, &rel),
                "name": label,
                "vendor": vendor,
                "driver": driver,
                "integrated": integrated,
                "temp": if nvidia { None } else { drm_temp(&dev) },
                "busy": busy,
                "vramUsed": used,
                "vramTotal": total,
            }),
            slot,
            nvidia,
        });
    }
    let rows = if cards.iter().any(|c| c.nvidia) {
        nvidia_rows()
    } else {
        Vec::new()
    };
    cards
        .into_iter()
        .map(|mut card| {
            if card.nvidia {
                let row = row_for(&card.slot, &rows);
                card.value["temp"] = json!(row.and_then(|r| r.temp));
                card.value["busy"] = json!(row.and_then(|r| r.busy));
                card.value["vramUsed"] = json!(row.and_then(|r| r.vram_used));
                card.value["vramTotal"] = json!(row.and_then(|r| r.vram_total));
                let smi_name = rows
                    .iter()
                    .find(|r| slots_match(&card.slot, &r.slot) && !r.name.trim().is_empty())
                    .or_else(|| (rows.len() == 1).then(|| &rows[0]))
                    .map(|r| r.name.trim().to_string())
                    .unwrap_or_default();
                if !smi_name.is_empty() {
                    card.value["name"] = json!(smi_name);
                }
            }
            card.value
        })
        .collect()
}
