//! SMBIOS: the sysfs dmi/id strings and the raw DMI table.

use super::{clean, Hw};
use crate::inventory::read_strip;
use serde_json::{json, Value};
use std::path::Path;

pub fn chassis_name(value: &str) -> String {
    let text = clean(value, 40);
    if text.is_empty() {
        return String::new();
    }
    if text.chars().all(|c| c.is_ascii_digit()) {
        return text
            .parse::<u32>()
            .ok()
            .and_then(chassis_type)
            .unwrap_or("")
            .into();
    }
    text
}

fn chassis_type(n: u32) -> Option<&'static str> {
    Some(match n {
        1 => "Other",
        2 => "Unknown",
        3 => "Desktop",
        4 => "Low-profile desktop",
        5 => "Pizza box",
        6 => "Mini tower",
        7 => "Tower",
        8 => "Portable",
        9 => "Laptop",
        10 => "Notebook",
        11 => "Handheld",
        12 => "Docking station",
        13 => "All-in-one",
        14 => "Sub-notebook",
        15 => "Space-saving",
        16 => "Lunch box",
        17 => "Main server",
        18 => "Expansion",
        19 => "Sub-chassis",
        20 => "Bus expansion",
        21 => "Peripheral",
        22 => "RAID",
        23 => "Rack mount",
        24 => "Sealed-case PC",
        25 => "Multi-system",
        26 => "Compact PCI",
        27 => "Advanced TCA",
        28 => "Blade",
        29 => "Blade enclosure",
        30 => "Tablet",
        31 => "Convertible",
        32 => "Detachable",
        33 => "IoT gateway",
        34 => "Embedded PC",
        35 => "Mini PC",
        36 => "Stick PC",
        _ => return None,
    })
}

fn memory_type(n: u8) -> &'static str {
    match n {
        1 => "Other",
        2 => "Unknown",
        3 => "DRAM",
        18 => "DDR",
        19 => "DDR2",
        24 => "DDR3",
        26 => "DDR4",
        27 => "LPDDR",
        29 => "LPDDR3",
        30 => "LPDDR4",
        32 => "HBM",
        33 => "HBM2",
        34 => "DDR5",
        35 => "LPDDR5",
        36 => "LPDDR5X",
        37 => "HBM3",
        _ => "",
    }
}

fn memory_form(n: u8) -> &'static str {
    match n {
        8 => "DIMM",
        13 => "SODIMM",
        15 => "FB-DIMM",
        16 => "Die",
        _ => "",
    }
}

fn decode_memory_size(word: u32, extended: u32) -> u64 {
    match word {
        0 | 0xFFFF => 0,
        0x7FFF => u64::from(extended) * 1024 * 1024,
        _ => {
            let value = u64::from(word & 0x7FFF);
            if word & 0x8000 != 0 {
                value * 1024
            } else {
                value * 1024 * 1024
            }
        }
    }
}

struct Entry<'a> {
    typ: u8,
    data: &'a [u8],
    strings: Vec<String>,
}

impl Entry<'_> {
    fn string(&self, index: u8) -> String {
        let index = usize::from(index);
        if index == 0 || index > self.strings.len() {
            return String::new();
        }
        clean(&self.strings[index - 1], 160)
    }

    fn at(&self, i: usize) -> u8 {
        self.data[i]
    }

    fn word(&self, from: usize) -> u32 {
        u32::from(u16::from_le_bytes([self.data[from], self.data[from + 1]]))
    }
}

fn parse_table(blob: &[u8]) -> Vec<Entry<'_>> {
    let mut out = Vec::new();
    let (mut i, n) = (0usize, blob.len());
    while i + 4 <= n {
        let typ = blob[i];
        let length = usize::from(blob[i + 1]);
        if length < 4 || i + length > n {
            break;
        }
        let data = &blob[i..i + length];
        let mut j = i + length;
        let mut strings = Vec::new();
        if j < n && blob[j] == 0 {
            j += 1;
            if j < n && blob[j] == 0 {
                j += 1;
            }
        } else {
            let mut start = j;
            while j < n {
                if blob[j] == 0 {
                    if j > start {
                        strings.push(blob[start..j].iter().map(|&b| b as char).collect());
                    }
                    j += 1;
                    start = j;
                    if j < n && blob[j] == 0 {
                        j += 1;
                        break;
                    }
                } else {
                    j += 1;
                }
            }
        }
        out.push(Entry { typ, data, strings });
        i = j;
        if typ == 127 {
            break;
        }
    }
    out
}

pub fn apply_table(hw: &mut Value, sys: &Path) {
    let blob = {
        use std::io::Read;
        let mut buf = Vec::new();
        let path = sys.join("sys/firmware/dmi/tables/DMI");
        if let Ok(file) = std::fs::File::open(path) {
            let _ = file.take(1 << 20).read_to_end(&mut buf);
        }
        buf
    };
    for e in parse_table(&blob) {
        let d = e.data;
        match e.typ {
            0 if d.len() >= 9 => {
                Hw::fill(hw, "bios", "vendor", &e.string(e.at(4)));
                Hw::fill(hw, "bios", "version", &e.string(e.at(5)));
                Hw::fill(hw, "bios", "date", &e.string(e.at(8)));
            }
            1 if d.len() >= 8 => {
                Hw::fill(hw, "machine", "vendor", &e.string(e.at(4)));
                Hw::fill(hw, "machine", "name", &e.string(e.at(5)));
                Hw::fill(hw, "machine", "version", &e.string(e.at(6)));
                Hw::fill(hw, "machine", "serial", &e.string(e.at(7)));
                if d.len() > 0x19 {
                    Hw::fill(hw, "machine", "sku", &e.string(e.at(0x19)));
                }
                if d.len() > 0x1A {
                    Hw::fill(hw, "machine", "family", &e.string(e.at(0x1A)));
                }
            }
            2 if d.len() >= 8 => {
                Hw::fill(hw, "board", "vendor", &e.string(e.at(4)));
                Hw::fill(hw, "board", "name", &e.string(e.at(5)));
                Hw::fill(hw, "board", "version", &e.string(e.at(6)));
                Hw::fill(hw, "board", "serial", &e.string(e.at(7)));
            }
            3 if d.len() >= 6 => {
                // `chassis_name(int)` goes through clean(), so a zero type is empty.
                let kind = e.at(5) & 0x7F;
                let name = if kind == 0 {
                    String::new()
                } else {
                    chassis_name(&kind.to_string())
                };
                Hw::fill(hw, "machine", "chassis", &name);
            }
            4 if d.len() >= 0x11 => {
                Hw::fill(hw, "cpu", "model", &e.string(e.at(0x10)));
                Hw::fill(hw, "cpu", "vendor", &e.string(e.at(7)));
            }
            17 if d.len() >= 0x13 => {
                let extended = if d.len() >= 0x20 {
                    u32::from_le_bytes([d[0x1C], d[0x1D], d[0x1E], d[0x1F]])
                } else {
                    0
                };
                let size = decode_memory_size(e.word(0x0C), extended);
                let locator = e.string(e.at(0x10));
                if size == 0 && locator.is_empty() {
                    continue;
                }
                let speed = if d.len() >= 0x17 { e.word(0x15) } else { 0 };
                let string_at = |i: usize| {
                    if d.len() > i {
                        e.string(e.at(i))
                    } else {
                        String::new()
                    }
                };
                let module = json!({
                    "locator": locator,
                    "bank": string_at(0x11),
                    "size": size,
                    "type": memory_type(e.at(0x12)),
                    "form": memory_form(e.at(0x0E)),
                    "speed": if speed == 0 || speed == 0xFFFF { 0 } else { speed },
                    "manufacturer": string_at(0x17),
                    "part": string_at(0x1A),
                    "rank": if d.len() > 0x1B { e.at(0x1B) & 0x0F } else { 0 },
                });
                if let Some(list) = hw["memory"]["modules"].as_array_mut() {
                    list.push(module);
                }
            }
            _ => {}
        }
    }
}

pub fn apply_id(hw: &mut Value, sys: &Path) {
    let base = sys.join("sys/class/dmi/id");
    if !base.is_dir() {
        return;
    }
    let get = |key: &str| clean(&read_strip(&base.join(key), 512), 160);
    for (sec, key, file) in [
        ("bios", "vendor", "bios_vendor"),
        ("bios", "version", "bios_version"),
        ("bios", "date", "bios_date"),
        ("machine", "vendor", "sys_vendor"),
        ("machine", "name", "product_name"),
        ("machine", "family", "product_family"),
        ("machine", "version", "product_version"),
        ("machine", "serial", "product_serial"),
        ("machine", "sku", "product_sku"),
        ("board", "vendor", "board_vendor"),
        ("board", "name", "board_name"),
        ("board", "version", "board_version"),
        ("board", "serial", "board_serial"),
    ] {
        Hw::fill(hw, sec, key, &get(file));
    }
    Hw::fill(
        hw,
        "machine",
        "chassis",
        &chassis_name(&get("chassis_type")),
    );
}
