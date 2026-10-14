//! PCI devices from lspci, with a sysfs + pci.ids fallback, and the chipset pick.

use super::{clean, Sys};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

#[derive(Default, Clone)]
pub struct Device {
    pub slot: String,
    pub class_name: String,
    pub class_id: String,
    pub vendor: String,
    pub name: String,
    pub pci_id: String,
}

pub fn devices(sys: &Sys) -> Vec<Device> {
    let mut raw = sys.run("lspci", &["-nnmm"]);
    if raw.is_empty() {
        raw = sys.run("lspci", &["-nn"]);
    }
    if raw.is_empty() {
        return sysfs_devices(sys);
    }
    raw.lines().filter_map(parse_line).collect()
}

/// `[hhhh]` .. `[hhhhhhhh]` (min..=max hex digits) found anywhere: the hex inside.
fn bracket_id(text: &str, min: usize, max: usize) -> String {
    let mut from = 0;
    while let Some(open) = text[from..].find('[').map(|i| i + from) {
        if let Some(close) = text[open..].find(']').map(|i| i + open) {
            let inner = &text[open + 1..close];
            if (min..=max).contains(&inner.len()) && inner.chars().all(|c| c.is_ascii_hexdigit()) {
                return inner.to_lowercase();
            }
        }
        from = open + 1;
    }
    String::new()
}

/// Drop a trailing `[hhhh]` (or `[hhhh:hhhh]`) and the whitespace around it.
fn strip_id(text: &str, min: usize, max: usize, colon: bool) -> String {
    let trimmed = text.trim_end();
    if let Some(body) = trimmed.strip_suffix(']') {
        if let Some(open) = body.rfind('[') {
            let inner = &body[open + 1..];
            let ok = if colon {
                inner.split_once(':').is_some_and(|(a, b)| {
                    a.len() == 4
                        && b.len() == 4
                        && a.chars().chain(b.chars()).all(|c| c.is_ascii_hexdigit())
                })
            } else {
                (min..=max).contains(&inner.len()) && inner.chars().all(|c| c.is_ascii_hexdigit())
            };
            if ok {
                return body[..open].trim_end().to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Three quoted fields after the slot: `^(\S+)\s+"([^"]*)"\s+"([^"]*)"\s+"([^"]*)"`.
fn parse_mm(line: &str) -> Option<(&str, [&str; 3])> {
    let (slot, mut rest) = line.split_once(char::is_whitespace)?;
    if slot.is_empty() {
        return None;
    }
    let mut fields = [""; 3];
    for field in &mut fields {
        rest = rest.trim_start();
        let inner = rest.strip_prefix('"')?;
        let end = inner.find('"')?;
        *field = &inner[..end];
        rest = &inner[end + 1..];
    }
    Some((slot, fields))
}

fn parse_line(line: &str) -> Option<Device> {
    if let Some((slot, [cls, ven, dev])) = parse_mm(line) {
        let class_id = bracket_id(cls, 4, 4);
        let vendor_id = bracket_id(ven, 4, 4);
        let device_id = bracket_id(dev, 4, 8);
        return Some(Device {
            slot: slot.into(),
            class_name: clean(&strip_id(cls, 4, 4, false), 80),
            class_id,
            vendor: clean(&strip_id(ven, 4, 4, false), 80),
            name: clean(&strip_id(dev, 4, 8, false), 160),
            pci_id: if !vendor_id.is_empty() && !device_id.is_empty() {
                format!("{vendor_id}:{device_id}")
            } else {
                String::new()
            },
        });
    }
    // `^(\S+)\s+([^:]+):\s+(.*)$`
    let (slot, rest) = line.split_once(char::is_whitespace)?;
    let rest = rest.trim_start();
    let colon = rest.find(':')?;
    if colon == 0 {
        return None;
    }
    let after = &rest[colon + 1..];
    let tail = after
        .strip_prefix(char::is_whitespace)
        .map(str::trim_start)?;
    let ids = classic_ids(tail);
    Some(Device {
        slot: slot.into(),
        class_name: clean(&rest[..colon], 80),
        class_id: String::new(),
        vendor: String::new(),
        name: clean(&strip_id(tail, 0, 0, true), 160),
        pci_id: ids,
    })
}

/// First `[hhhh:hhhh]` in the text, lower-cased.
fn classic_ids(text: &str) -> String {
    let bytes = text.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'['
            && i + 11 <= bytes.len()
            && bytes[i + 10] == b']'
            && bytes[i + 5] == b':'
        {
            let inner = &text[i + 1..i + 10];
            let (a, b) = (&inner[..4], &inner[5..]);
            if a.chars().chain(b.chars()).all(|c| c.is_ascii_hexdigit()) {
                return inner.to_lowercase();
            }
        }
    }
    String::new()
}

fn class_label(id: &str) -> &'static str {
    match id {
        "0600" => "Host bridge",
        "0601" => "ISA bridge",
        "0300" => "VGA compatible controller",
        "0302" => "3D controller",
        "0380" => "Display controller",
        "0200" => "Ethernet controller",
        "0280" => "Network controller",
        "0403" => "Audio device",
        "0c03" => "USB controller",
        "0c05" => "SMBus",
        _ => "",
    }
}

fn sysfs_devices(sys: &Sys) -> Vec<Device> {
    let hex = |rel: String| sys.strip(&rel).replace("0x", "").to_lowercase();
    let mut rows = Vec::new();
    for name in sys.dir_names("sys/bus/pci/devices") {
        let vendor = hex(format!("sys/bus/pci/devices/{name}/vendor"));
        let device = hex(format!("sys/bus/pci/devices/{name}/device"));
        let class = hex(format!("sys/bus/pci/devices/{name}/class"));
        if vendor.len() < 4 || device.len() < 4 {
            continue;
        }
        rows.push(Device {
            slot: name.replace("0000:", ""),
            class_id: class.chars().take(4).collect(),
            pci_id: format!("{vendor}:{device}"),
            ..Device::default()
        });
    }
    let pairs: Vec<(String, String)> = rows
        .iter()
        .filter_map(|r| {
            r.pci_id
                .split_once(':')
                .map(|(v, d)| (v.to_string(), d.to_string()))
        })
        .collect();
    let names = lookup_names(sys, &pairs);
    for row in &mut rows {
        let vendor = row.pci_id.split(':').next().unwrap_or("");
        row.vendor = names.get(vendor).cloned().unwrap_or_default();
        row.name = names.get(&row.pci_id).cloned().unwrap_or_default();
        row.class_name = class_label(&row.class_id).into();
    }
    rows
}

fn lookup_names(sys: &Sys, pairs: &[(String, String)]) -> HashMap<String, String> {
    let mut needed: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (vendor, device) in pairs {
        needed.entry(vendor).or_default().insert(device);
    }
    let candidates = [
        sys.path("usr/share/hwdata/pci.ids"),
        sys.path("usr/share/misc/pci.ids"),
        "/usr/share/hwdata/pci.ids".into(),
        "/usr/share/misc/pci.ids".into(),
    ];
    let mut names = HashMap::new();
    let Some(path) = candidates.iter().find(|p| p.is_file()) else {
        return names;
    };
    let Ok(raw) = std::fs::read(path) else {
        return names;
    };
    let mut vendor = String::new();
    for line in String::from_utf8_lossy(&raw).lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("C ") {
            continue;
        }
        if !line.starts_with('\t') {
            let mut parts = line.splitn(2, char::is_whitespace);
            let (Some(id), Some(name)) = (parts.next(), parts.next()) else {
                continue;
            };
            vendor = id.to_lowercase();
            if needed.contains_key(vendor.as_str()) {
                names.insert(vendor.clone(), name.trim().to_string());
            }
            continue;
        }
        if line.starts_with("\t\t") {
            continue;
        }
        let Some(wanted) = needed.get(vendor.as_str()) else {
            continue;
        };
        let mut parts = line.trim().splitn(2, char::is_whitespace);
        let (Some(id), Some(name)) = (parts.next(), parts.next()) else {
            continue;
        };
        let device = id.to_lowercase();
        if wanted.contains(device.as_str()) {
            names.insert(format!("{vendor}:{device}"), name.trim().to_string());
        }
    }
    names
}

pub fn is_host_bridge(d: &Device) -> bool {
    d.class_id == "0600" || d.class_name.to_lowercase().contains("host bridge")
}

pub fn is_isa_bridge(d: &Device) -> bool {
    let name = d.class_name.to_lowercase();
    d.class_id == "0601" || name.contains("isa bridge") || name.contains("lpc")
}

pub fn is_gpu(d: &Device) -> bool {
    if matches!(d.class_id.as_str(), "0300" | "0302" | "0380") {
        return true;
    }
    let name = d.class_name.to_lowercase();
    name.contains("vga") || name.contains("3d controller") || name.contains("display controller")
}

pub fn is_npu(d: &Device) -> bool {
    let blob = format!("{} {} {}", d.class_name, d.name, d.vendor).to_lowercase();
    let padded = format!(" {blob} ");
    blob.contains("neural processing")
        || padded.contains(" npu")
        || blob.contains("xdna")
        || padded.contains(" tpu")
        || blob.contains("ai accelerator")
        || blob.contains("habana")
        || blob.contains("hailo")
        || blob.contains("coral")
}

pub fn chipset(devices: &[Device]) -> Value {
    let host = devices.iter().find(|d| is_host_bridge(d));
    let south = devices.iter().find(|d| is_isa_bridge(d));
    let Some(primary) = host.or(south) else {
        return json!({"name": "", "vendor": "", "pciId": "", "role": "", "southbridge": ""});
    };
    let south_name = match (host, south) {
        (Some(h), Some(s)) if !s.name.is_empty() && s.name != h.name => s.name.as_str(),
        _ => "",
    };
    json!({
        "name": clean(&primary.name, 160),
        "vendor": clean(&primary.vendor, 80),
        "pciId": clean(&primary.pci_id, 20),
        "role": if host.is_some() { "Host bridge" } else { "ISA bridge" },
        "southbridge": clean(south_name, 160),
    })
}

pub fn summary(devices: &[Device]) -> Value {
    Value::Array(
        devices
            .iter()
            .filter(|d| {
                is_host_bridge(d)
                    || is_isa_bridge(d)
                    || is_gpu(d)
                    || matches!(d.class_id.as_str(), "0200" | "0280" | "0403" | "0c05")
            })
            .map(|d| {
                json!({
                    "slot": d.slot,
                    "className": d.class_name,
                    "classId": d.class_id,
                    "vendor": d.vendor,
                    "name": d.name,
                    "pciId": d.pci_id,
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_readable_lspci_lines_split_into_ids_and_names() {
        let line = r#"00:02.0 "VGA compatible controller [0300]" "Intel Corporation [8086]" "Alder Lake-P GT2 [Iris Xe Graphics] [46a6]" -r0c -p00 "Dell [1028]" "Device [0a5c]""#;
        let d = parse_line(line).unwrap();
        assert_eq!(d.slot, "00:02.0");
        assert_eq!(d.class_id, "0300");
        assert_eq!(d.class_name, "VGA compatible controller");
        assert_eq!(d.vendor, "Intel Corporation");
        assert_eq!(d.name, "Alder Lake-P GT2 [Iris Xe Graphics]");
        assert_eq!(d.pci_id, "8086:46a6");
        assert!(is_gpu(&d));
    }

    #[test]
    fn classic_lspci_lines_still_parse() {
        let d =
            parse_line("00:00.0 Host bridge [0600]: Intel Corporation Device [8086:9a14] (rev 01)")
                .unwrap();
        assert_eq!(d.class_name, "Host bridge [0600]");
        assert_eq!(d.pci_id, "8086:9a14");
        assert!(parse_line("garbage").is_none());
    }
}
