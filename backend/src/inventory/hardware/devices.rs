//! The sysfs-backed device lists: GPUs, NICs, audio, USB, power, thermals, security.

use super::pci::{is_gpu, is_npu, Device};
use super::{clean, Sys};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// `basename(realpath(path))`, or the path's own name when it does not resolve.
fn link_name(path: &Path) -> String {
    let real = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    real.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn drm_driver(sys: &Sys, pci_id: &str) -> String {
    for name in sys.dir_names("sys/class/drm") {
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let dir = format!("sys/class/drm/{name}/device");
        let hex = |file: &str| {
            sys.strip(&format!("{dir}/{file}"))
                .replace("0x", "")
                .to_lowercase()
        };
        if !pci_id.is_empty() && format!("{}:{}", hex("vendor"), hex("device")) != pci_id {
            continue;
        }
        let driver = link_name(&sys.path(&format!("{dir}/driver")));
        if !driver.is_empty() && driver != "driver" {
            return driver;
        }
    }
    String::new()
}

pub fn gpus(sys: &Sys, devices: &[Device]) -> Value {
    let found: Vec<Value> = devices
        .iter()
        .filter(|d| is_gpu(d))
        .map(|d| {
            json!({
                "name": clean(&d.name, 160),
                "vendor": clean(&d.vendor, 80),
                "pciId": d.pci_id,
                "driver": drm_driver(sys, &d.pci_id),
                "vram": 0,
            })
        })
        .collect();
    if !found.is_empty() || !sys.path("sys/class/drm").is_dir() {
        return Value::Array(found);
    }
    let rows = sys
        .dir_names("sys/class/drm")
        .into_iter()
        .filter(|n| n.starts_with("card") && !n.contains('-'))
        .map(|name| {
            let driver = link_name(&sys.path(&format!("sys/class/drm/{name}/device/driver")));
            let driver = if driver == "driver" {
                String::new()
            } else {
                driver
            };
            json!({"name": name, "vendor": "", "pciId": "", "driver": driver, "vram": 0})
        })
        .collect();
    Value::Array(rows)
}

pub fn npus(devices: &[Device]) -> Value {
    Value::Array(
        devices
            .iter()
            .filter(|d| is_npu(d))
            .map(|d| json!({"name": clean(&d.name, 160), "vendor": clean(&d.vendor, 80), "pciId": d.pci_id}))
            .collect(),
    )
}

const VIRTUAL_NET: &[&str] = &[
    "lo",
    "docker",
    "veth",
    "br-",
    "virbr",
    "cni",
    "flannel",
    "tun",
    "tap",
    "wg",
    "tailscale",
    "zt",
];

pub fn nics(sys: &Sys) -> Value {
    let mut rows = Vec::new();
    for name in sys.dir_names("sys/class/net") {
        let dir = format!("sys/class/net/{name}");
        if VIRTUAL_NET.iter().any(|p| name.starts_with(p)) || !sys.path(&dir).is_dir() {
            continue;
        }
        if sys.strip(&format!("{dir}/type")) == "772" {
            continue;
        }
        let driver_path = sys.path(&format!("{dir}/device/driver"));
        let driver = if driver_path.exists() {
            link_name(&driver_path)
        } else {
            String::new()
        };
        let driver = if driver == "driver" {
            String::new()
        } else {
            driver
        };
        rows.push(json!({
            "name": name,
            "iface": name,
            "mac": clean(&sys.strip(&format!("{dir}/address")), 20),
            "driver": driver,
            "speed": sys.int(&format!("{dir}/speed")).max(0),
            "wireless": sys.path(&format!("{dir}/wireless")).is_dir(),
        }));
    }
    Value::Array(rows)
}

/// ` 0 [PCH            ]: HDA-Intel - HDA Intel PCH` -> (id, bracket, rest)
fn card_line(line: &str) -> Option<(&str, &str, &str)> {
    let line = line.trim_start();
    let digits = line
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(line.len());
    if digits == 0 {
        return None;
    }
    let rest = line[digits..].trim_start();
    if rest.len() == line[digits..].len() {
        return None;
    }
    let inner = rest.strip_prefix('[')?;
    let close = inner.find(']')?;
    let after = inner[close + 1..].strip_prefix(':')?;
    let tail = after
        .strip_prefix(char::is_whitespace)
        .map(str::trim_start)?;
    Some((&line[..digits], &inner[..close], tail))
}

pub fn audio(sys: &Sys) -> Value {
    let cards = sys.text("proc/asound/cards", 8192);
    let rows: Vec<Value> = cards
        .lines()
        .filter_map(card_line)
        .map(|(id, short, long)| {
            let name = if long.is_empty() { short } else { long };
            json!({"id": id, "name": clean(name, 160), "driver": clean(short, 40)})
        })
        .collect();
    if !rows.is_empty() || !sys.path("sys/class/sound").is_dir() {
        return Value::Array(rows);
    }
    let rows = sys
        .dir_names("sys/class/sound")
        .into_iter()
        .filter(|n| n.starts_with("card") && !n.ends_with('c'))
        .filter_map(|name| {
            let ident = sys.strip(&format!("sys/class/sound/{name}/id"));
            (!ident.is_empty()).then(|| json!({"id": name, "name": ident, "driver": ""}))
        })
        .collect();
    Value::Array(rows)
}

pub fn usb(sys: &Sys) -> Value {
    let mut out = Vec::new();
    for name in sys.dir_names("sys/bus/usb/devices") {
        let dir = format!("sys/bus/usb/devices/{name}");
        let product = clean(&sys.strip(&format!("{dir}/product")), 160);
        let manufacturer = clean(&sys.strip(&format!("{dir}/manufacturer")), 80);
        if (product.is_empty() && manufacturer.is_empty()) || name.starts_with("usb") {
            continue;
        }
        out.push(json!({
            "name": if product.is_empty() { manufacturer.clone() } else { product.clone() },
            "vendor": manufacturer,
            "product": product,
            "speed": clean(&sys.strip(&format!("{dir}/speed")), 20),
        }));
    }
    Value::Array(out)
}

pub fn batteries(sys: &Sys) -> Value {
    let mut out = Vec::new();
    for name in sys.dir_names("sys/class/power_supply") {
        let dir = format!("sys/class/power_supply/{name}");
        let kind = sys.strip(&format!("{dir}/type")).to_lowercase();
        if (!kind.is_empty() && kind != "battery")
            || !sys.path(&format!("{dir}/capacity")).is_file()
        {
            continue;
        }
        let model = sys.strip(&format!("{dir}/model_name"));
        out.push(json!({
            "name": clean(if model.is_empty() { &name } else { &model }, 80),
            "status": clean(&sys.strip(&format!("{dir}/status")), 20),
            "capacity": sys.int(&format!("{dir}/capacity")),
            "technology": clean(&sys.strip(&format!("{dir}/technology")), 20),
            "energy": sys.int(&format!("{dir}/energy_now")),
            "voltage": sys.int(&format!("{dir}/voltage_now")),
        }));
    }
    Value::Array(out)
}

pub fn thermals(sys: &Sys) -> Value {
    let mut out = Vec::new();
    for name in sys.dir_names("sys/class/thermal") {
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let dir = format!("sys/class/thermal/{name}");
        let milli = sys.int(&format!("{dir}/temp"));
        if milli <= 0 {
            continue;
        }
        let kind = clean(&sys.strip(&format!("{dir}/type")), 40);
        out.push(json!({
            "name": if kind.is_empty() { name.clone() } else { kind.clone() },
            "type": kind,
            "temp": milli as f64 / 1000.0,
        }));
    }
    Value::Array(out)
}

pub fn tpm(sys: &Sys) -> Value {
    if sys.path("sys/class/tpm/tpm0").is_dir() {
        let version = clean(&sys.strip("sys/class/tpm/tpm0/tpm_version_major"), 12);
        return json!({"present": true, "version": version});
    }
    let present = sys.path("dev/tpm0").exists() || sys.path("dev/tpmrm0").exists();
    json!({"present": present, "version": ""})
}

pub fn secure_boot(sys: &Sys) -> Value {
    if !sys.path("sys/firmware/efi").is_dir() {
        return json!({"available": false, "enabled": false});
    }
    let mut enabled = false;
    let mut vars = sys.dir_names("sys/firmware/efi/efivars");
    vars.retain(|n| n.starts_with("SecureBoot-"));
    if let Some(name) = vars.first() {
        use std::io::Read;
        let mut blob = Vec::new();
        if let Ok(file) = fs::File::open(sys.path(&format!("sys/firmware/efi/efivars/{name}"))) {
            let _ = file.take(8).read_to_end(&mut blob);
        }
        enabled = blob.len() >= 5 && blob[4] == 1;
    }
    json!({"available": true, "enabled": enabled})
}

pub fn virt(sys: &Sys, cpu: &Value) -> Value {
    let mut hypervisor = clean(cpu["hypervisor"].as_str().unwrap_or(""), 40);
    let has_flag = cpu["flags"]
        .as_array()
        .is_some_and(|f| f.iter().any(|x| x == "hypervisor"));
    let mut guest = !hypervisor.is_empty() || has_flag;
    let kvm = sys.path("dev/kvm").exists();
    if hypervisor.is_empty() {
        for rel in ["sys/hypervisor/type", "sys/class/dmi/id/product_name"] {
            let text = clean(&sys.strip(rel), 40);
            const KNOWN: &[&str] = &[
                "kvm",
                "qemu",
                "vmware",
                "virtualbox",
                "xen",
                "microsoft",
                "bhyve",
                "parallels",
            ];
            if KNOWN.contains(&text.to_lowercase().as_str()) {
                hypervisor = text;
                guest = true;
                break;
            }
        }
    }
    let detect = sys.run("systemd-detect-virt", &[]);
    if let Some(first) = detect.lines().next() {
        let virt = clean(first, 40);
        if !virt.is_empty() && virt.to_lowercase() != "none" {
            if hypervisor.is_empty() {
                hypervisor = virt;
            }
            guest = true;
        }
    }
    json!({"hypervisor": hypervisor, "guest": guest, "kvm": kvm})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_cards_parse_from_proc_asound() {
        let (id, short, long) =
            card_line(" 0 [PCH            ]: HDA-Intel - HDA Intel PCH").unwrap();
        assert_eq!(
            (id, short.trim(), long),
            ("0", "PCH", "HDA-Intel - HDA Intel PCH")
        );
        assert!(card_line("                      HDA Intel PCH at 0xf7210000").is_none());
    }
}
