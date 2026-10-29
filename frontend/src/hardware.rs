//! The Hardware page's logic (`pages/HardwarePage.qml`, `services/Hardware.js`): the `hardware`
//! inventory as `normalize` keeps it, and how each part is summed up. `tests/catalog_hardware.rs`
//! runs `Hardware.js` and the page's own expressions under Node against this.

use serde_json::{Map, Value, json};

use crate::drivers::clean_text;
use crate::js;

const MEMORY_TYPES: &[(u32, &str)] = &[
    (1, "Other"),
    (2, "Unknown"),
    (3, "DRAM"),
    (4, "EDRAM"),
    (5, "VRAM"),
    (6, "SRAM"),
    (7, "RAM"),
    (8, "ROM"),
    (9, "Flash"),
    (10, "EEPROM"),
    (11, "FEPROM"),
    (12, "EPROM"),
    (13, "CDRAM"),
    (14, "3DRAM"),
    (15, "SDRAM"),
    (16, "SGRAM"),
    (17, "RDRAM"),
    (18, "DDR"),
    (19, "DDR2"),
    (20, "DDR2 FB-DIMM"),
    (24, "DDR3"),
    (25, "FBD2"),
    (26, "DDR4"),
    (27, "LPDDR"),
    (28, "LPDDR2"),
    (29, "LPDDR3"),
    (30, "LPDDR4"),
    (31, "Logical"),
    (32, "HBM"),
    (33, "HBM2"),
    (34, "DDR5"),
    (35, "LPDDR5"),
    (36, "LPDDR5X"),
    (37, "HBM3"),
];
const MEMORY_FORMS: &[(u32, &str)] = &[
    (1, "Other"),
    (2, "Unknown"),
    (3, "SIMM"),
    (4, "SIP"),
    (8, "DIMM"),
    (9, "TSOP"),
    (13, "SODIMM"),
    (15, "FB-DIMM"),
    (16, "Die"),
];
const CHASSIS_TYPES: &[(u32, &str)] = &[
    (1, "Other"),
    (2, "Unknown"),
    (3, "Desktop"),
    (4, "Low-profile desktop"),
    (5, "Pizza box"),
    (6, "Mini tower"),
    (7, "Tower"),
    (8, "Portable"),
    (9, "Laptop"),
    (10, "Notebook"),
    (11, "Handheld"),
    (12, "Docking station"),
    (13, "All-in-one"),
    (14, "Sub-notebook"),
    (15, "Space-saving"),
    (16, "Lunch box"),
    (17, "Main server"),
    (18, "Expansion"),
    (19, "Sub-chassis"),
    (20, "Bus expansion"),
    (21, "Peripheral"),
    (22, "RAID"),
    (23, "Rack mount"),
    (24, "Sealed-case PC"),
    (25, "Multi-system"),
    (26, "Compact PCI"),
    (27, "Advanced TCA"),
    (28, "Blade"),
    (29, "Blade enclosure"),
    (30, "Tablet"),
    (31, "Convertible"),
    (32, "Detachable"),
    (33, "IoT gateway"),
    (34, "Embedded PC"),
    (35, "Mini PC"),
    (36, "Stick PC"),
];

const NOTABLE: &[(&str, &str)] = &[
    ("vmx", "VT-x"),
    ("svm", "AMD-V"),
    ("hypervisor", "hypervisor"),
    ("aes", "AES"),
    ("avx", "AVX"),
    ("avx2", "AVX2"),
    ("avx512f", "AVX-512"),
    ("nx", "NX"),
    ("lm", "64-bit"),
    ("sse4_2", "SSE4.2"),
];

fn object(v: Option<&Value>) -> &Map<String, Value> {
    static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
    v.and_then(Value::as_object)
        .unwrap_or_else(|| EMPTY.get_or_init(Map::new))
}

fn array(v: Option<&Value>) -> &[Value] {
    v.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn text(row: &Map<String, Value>, name: &str, max: usize) -> String {
    clean_text(row.get(name), max)
}

/// `row.a || row.b`.
fn or<'a>(row: &'a Map<String, Value>, a: &str, b: &str) -> Option<&'a Value> {
    row.get(a)
        .filter(|v| js::truthy(Some(v)))
        .or_else(|| row.get(b))
}

/// `intOrZero`: a whole number above 0, else 0.
fn whole(v: Option<&Value>) -> f64 {
    let n = js::round(js::number(v));
    if n.is_finite() && n > 0.0 { n } else { 0.0 }
}

fn num(n: f64) -> Value {
    js::json_number(n)
}

fn named(table: &[(u32, &str)], code: Option<&Value>) -> String {
    let n = whole(code);
    table
        .iter()
        .find(|(c, _)| f64::from(*c) == n)
        .map_or_else(String::new, |(_, name)| (*name).to_owned())
}

fn chassis(code: Option<&Value>) -> String {
    let n = whole(code);
    if let Some(Value::String(s)) = code
        && n == 0.0
        && !s.is_empty()
        && !s.chars().all(|c| c.is_ascii_digit())
    {
        return clean_text(code, 40);
    }
    named(CHASSIS_TYPES, code)
}

fn module(item: &Value) -> Value {
    let row = object(Some(item));
    let kind = or(row, "type", "typeCode");
    let kind = if js::truthy(row.get("type")) {
        clean_text(kind, 20)
    } else {
        clean_text(
            Some(&Value::from(named(MEMORY_TYPES, row.get("typeCode")))),
            20,
        )
    };
    let form = if js::truthy(row.get("form")) {
        text(row, "form", 20)
    } else {
        clean_text(
            Some(&Value::from(named(MEMORY_FORMS, row.get("formCode")))),
            20,
        )
    };
    json!({
        "locator": text(row, "locator", 40), "bank": text(row, "bank", 40),
        "size": num(whole(row.get("size"))), "type": kind, "form": form,
        "speed": num(whole(row.get("speed"))), "manufacturer": text(row, "manufacturer", 80),
        "part": text(row, "part", 80), "rank": num(whole(row.get("rank"))),
    })
}

fn pci(item: &Value) -> Value {
    let row = object(Some(item));
    json!({
        "slot": text(row, "slot", 20), "className": text(row, "className", 80),
        "classId": text(row, "classId", 8), "vendor": text(row, "vendor", 80),
        "name": text(row, "name", 160), "pciId": text(row, "pciId", 20),
    })
}

fn gpu(item: &Value) -> Value {
    let row = object(Some(item));
    json!({
        "name": text(row, "name", 160), "vendor": text(row, "vendor", 80),
        "pciId": text(row, "pciId", 20), "driver": text(row, "driver", 40),
        "vram": num(whole(row.get("vram"))),
    })
}

fn nic(item: &Value) -> Value {
    let row = object(Some(item));
    json!({
        "name": clean_text(or(row, "name", "iface"), 80), "iface": text(row, "iface", 32),
        "mac": text(row, "mac", 20), "driver": text(row, "driver", 40),
        "speed": num(whole(row.get("speed"))),
        "wireless": row.get("wireless") == Some(&Value::Bool(true)),
    })
}

fn battery(item: &Value) -> Value {
    let row = object(Some(item));
    json!({
        "name": text(row, "name", 80), "status": text(row, "status", 20),
        "capacity": num(whole(row.get("capacity"))), "technology": text(row, "technology", 20),
        "energy": num(whole(row.get("energy"))), "voltage": num(whole(row.get("voltage"))),
    })
}

fn thermal(item: &Value) -> (Value, f64) {
    let row = object(Some(item));
    let temp = js::number(row.get("temp"));
    let out = json!({
        "name": clean_text(or(row, "name", "type"), 80), "type": text(row, "type", 40),
        "temp": if temp.is_finite() { num(temp) } else { Value::Null },
    });
    (out, temp)
}

fn has(v: &Value, names: &[&str]) -> bool {
    names.iter().any(|n| js::truthy(v.get(*n)))
}

fn list(raw: Option<&Value>, f: fn(&Value) -> Value, keep: &[&str]) -> Vec<Value> {
    array(raw).iter().map(f).filter(|v| has(v, keep)).collect()
}

/// `normalize(raw)`: the inventory, each field cleaned and each list kept to what names itself.
pub fn normalize(raw: Option<&Value>) -> Value {
    let data = object(raw);
    let part = |name: &str| object(data.get(name));
    let (m, b, c, bios, cpu, mem) = (
        part("machine"),
        part("board"),
        part("chipset"),
        part("bios"),
        part("cpu"),
        part("memory"),
    );
    let chassis = match chassis(m.get("chassis")) {
        s if s.is_empty() => text(m, "chassis", 40),
        s => s,
    };
    let flags: Vec<Value> = array(cpu.get("flags"))
        .iter()
        .map(|f| clean_text(Some(f), 32))
        .filter(|f| !f.is_empty())
        .map(Value::from)
        .collect();
    let sockets = match whole(cpu.get("sockets")) {
        0.0 => 1.0,
        n => n,
    };
    let modules: Vec<Value> = array(mem.get("modules"))
        .iter()
        .map(module)
        .filter(|m| m["size"].as_f64().is_some_and(|s| s > 0.0) || has(m, &["locator"]))
        .collect();
    let npu = |item: &Value| {
        let row = object(Some(item));
        json!({ "name": text(row, "name", 160), "vendor": text(row, "vendor", 80), "pciId": text(row, "pciId", 20) })
    };
    let thermals: Vec<Value> = array(data.get("thermals"))
        .iter()
        .map(thermal)
        .filter(|(t, temp)| has(t, &["name"]) && temp.is_finite())
        .map(|(t, _)| t)
        .collect();
    let (tpm, sb, virt) = (part("tpm"), part("secureBoot"), part("virtualization"));
    let hypervisor = text(virt, "hypervisor", 40);
    let yes = |row: &Map<String, Value>, n: &str| row.get(n) == Some(&Value::Bool(true));
    json!({
        "machine": {
            "vendor": text(m, "vendor", 80), "name": text(m, "name", 160),
            "family": text(m, "family", 80), "version": text(m, "version", 80),
            "serial": text(m, "serial", 80), "sku": text(m, "sku", 80), "chassis": chassis,
        },
        "board": {
            "vendor": text(b, "vendor", 80), "name": text(b, "name", 160),
            "version": text(b, "version", 40), "serial": text(b, "serial", 80),
        },
        "chipset": {
            "name": text(c, "name", 160), "vendor": text(c, "vendor", 80),
            "pciId": text(c, "pciId", 20), "role": text(c, "role", 40),
            "southbridge": text(c, "southbridge", 160),
        },
        "bios": {
            "vendor": text(bios, "vendor", 80), "version": text(bios, "version", 80),
            "date": text(bios, "date", 40), "uefi": yes(bios, "uefi"),
        },
        "cpu": {
            "model": text(cpu, "model", 160), "vendor": text(cpu, "vendor", 80),
            "arch": text(cpu, "arch", 40), "cores": num(whole(cpu.get("cores"))),
            "threads": num(whole(cpu.get("threads"))), "sockets": num(sockets),
            "mhz": num(whole(cpu.get("mhz"))), "maxMhz": num(whole(cpu.get("maxMhz"))),
            "caches": text(cpu, "caches", 200), "flags": flags,
            "virtualization": text(cpu, "virtualization", 40),
            "hypervisor": text(cpu, "hypervisor", 40),
        },
        "memory": {
            "total": num(whole(mem.get("total"))), "used": num(whole(mem.get("used"))),
            "available": num(whole(mem.get("available"))),
            "swapTotal": num(whole(mem.get("swapTotal"))),
            "swapUsed": num(whole(mem.get("swapUsed"))), "modules": modules,
        },
        "gpus": list(data.get("gpus"), gpu, &["name", "pciId"]),
        "npus": array(data.get("npus")).iter().map(npu).filter(|n| has(n, &["name", "pciId"])).collect::<Vec<_>>(),
        "nics": list(data.get("nics"), nic, &["iface", "name"]),
        "audio": list(data.get("audio"), |i| {
            let row = object(Some(i));
            json!({ "name": text(row, "name", 160), "driver": text(row, "driver", 40), "id": text(row, "id", 20) })
        }, &["name"]),
        "usb": list(data.get("usb"), |i| {
            let row = object(Some(i));
            json!({
                "name": clean_text(or(row, "name", "product"), 160), "vendor": text(row, "vendor", 80),
                "product": text(row, "product", 80), "speed": text(row, "speed", 20),
            })
        }, &["name", "product"]),
        "batteries": list(data.get("batteries"), battery, &["name", "capacity"]),
        "thermals": thermals,
        "tpm": { "present": yes(tpm, "present"), "version": text(tpm, "version", 12) },
        "secureBoot": { "available": yes(sb, "available"), "enabled": yes(sb, "enabled") },
        "virtualization": {
            "guest": yes(virt, "guest") || !hypervisor.is_empty(),
            "hypervisor": hypervisor, "kvm": yes(virt, "kvm"),
        },
        "pci": list(data.get("pci"), pci, &["name", "className"]),
    })
}

/// `joinParts`: each part cleaned, the empty ones left out.
fn join(parts: &[String], sep: &str) -> String {
    parts
        .iter()
        .map(|p| clean_text(Some(&Value::from(p.as_str())), 200))
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(sep)
}

/// `sentence`: the parts joined by ". ", ending in a stop.
fn sentence(parts: &[String]) -> String {
    let text = join(parts, ". ");
    if text.is_empty() || text.ends_with(['.', '!', '?']) {
        text
    } else {
        format!("{text}.")
    }
}

fn s(v: &Value, name: &str) -> String {
    v.get(name)
        .filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

fn n(v: &Value, name: &str) -> f64 {
    v.get(name).and_then(Value::as_f64).unwrap_or(0.0)
}

/// `if (row.x) "<prefix>" + row.x`.
fn labelled(v: &Value, name: &str, prefix: &str) -> String {
    match s(v, name) {
        x if x.is_empty() => x,
        x => format!("{prefix}{x}"),
    }
}

pub fn machine_summary(m: &Value) -> String {
    let family = match s(m, "family") {
        f if f != s(m, "name") => f,
        _ => String::new(),
    };
    sentence(&[
        join(&[s(m, "vendor"), s(m, "name")], " "),
        family,
        s(m, "chassis"),
        labelled(m, "version", "Version "),
    ])
}

pub fn board_summary(b: &Value) -> String {
    sentence(&[
        join(&[s(b, "vendor"), s(b, "name")], " "),
        labelled(b, "version", "Version "),
    ])
}

pub fn chipset_summary(c: &Value) -> String {
    sentence(&[
        join(&[s(c, "vendor"), s(c, "name")], " "),
        s(c, "role"),
        labelled(c, "southbridge", "Southbridge "),
        s(c, "pciId"),
    ])
}

pub fn bios_summary(b: &Value) -> String {
    let uefi = if js::truthy(b.get("uefi")) {
        "UEFI"
    } else {
        ""
    };
    sentence(&[
        join(&[s(b, "vendor"), s(b, "version")], " "),
        s(b, "date"),
        uefi.into(),
    ])
}

pub fn cpu_summary(c: &Value) -> String {
    let mut bits = vec![s(c, "model")];
    let mut topo = Vec::new();
    let (cores, threads, sockets) = (n(c, "cores"), n(c, "threads"), n(c, "sockets"));
    if cores > 0.0 {
        topo.push(format!(
            "{cores} {}",
            if cores == 1.0 { "core" } else { "cores" }
        ));
    }
    if threads > 0.0 && threads != cores {
        topo.push(format!("{threads} threads"));
    }
    if sockets > 1.0 {
        topo.push(format!("{sockets} sockets"));
    }
    let arch = s(c, "arch");
    if !arch.is_empty() {
        topo.push(arch);
    }
    bits.push(topo.join(", "));
    let (max, mhz) = (n(c, "maxMhz"), n(c, "mhz"));
    bits.push(if max > 0.0 {
        format!("{} MHz max", js::round(max))
    } else if mhz > 0.0 {
        format!("{} MHz", js::round(mhz))
    } else {
        String::new()
    });
    bits.push(s(c, "caches"));
    let mut feats: Vec<String> = Vec::new();
    for f in array(c.get("flags")) {
        let key = js::string(f).to_lowercase();
        if let Some((_, label)) = NOTABLE.iter().find(|(k, _)| *k == key)
            && !feats.iter().any(|x| x == label)
        {
            feats.push((*label).to_owned());
        }
    }
    let virt = s(c, "virtualization");
    if !virt.is_empty() {
        feats.insert(0, virt);
    }
    bits.push(feats.join(", "));
    sentence(&bits)
}

/// `toFixed(1)`.
fn fixed(n: f64, places: usize) -> String {
    format!("{n:.places$}")
}

fn module_size(bytes: f64) -> String {
    if !bytes.is_finite() || bytes <= 0.0 {
        return String::new();
    }
    let gb = bytes / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        if (gb - js::round(gb)).abs() < 0.05 {
            return format!("{} GB", js::round(gb));
        }
        return format!("{} GB", fixed(gb, 1));
    }
    format!("{} MB", js::round(bytes / (1024.0 * 1024.0)))
}

pub fn module_summary(item: &Value) -> String {
    let m = module(item);
    let mut bits = Vec::new();
    if n(&m, "size") > 0.0 {
        bits.push(module_size(n(&m, "size")));
    }
    bits.extend([s(&m, "type"), s(&m, "form")]);
    if n(&m, "speed") > 0.0 {
        bits.push(format!("{} MT/s", n(&m, "speed")));
    }
    bits.extend([s(&m, "manufacturer"), s(&m, "part")]);
    if n(&m, "rank") > 0.0 {
        bits.push(format!("rank {}", n(&m, "rank")));
    }
    bits.retain(|b| !b.is_empty());
    bits.join(" · ")
}

pub fn nic_summary(item: &Value) -> String {
    let r = nic(item);
    let mut bits = Vec::new();
    let name = s(&r, "name");
    if !name.is_empty() && name != s(&r, "iface") {
        bits.push(name);
    }
    if js::truthy(r.get("wireless")) {
        bits.push("Wireless".into());
    }
    bits.push(labelled(&r, "driver", "Driver "));
    if n(&r, "speed") > 0.0 {
        bits.push(format!("{} Mb/s", n(&r, "speed")));
    }
    bits.push(s(&r, "mac"));
    sentence(&bits)
}

pub fn battery_summary(item: &Value) -> String {
    let r = battery(item);
    let capacity = n(&r, "capacity");
    let capacity = if capacity > 0.0 {
        format!("{capacity}%")
    } else {
        String::new()
    };
    sentence(&[s(&r, "status"), capacity, s(&r, "technology")])
}

pub fn thermal_summary(item: &Value) -> String {
    let (_, temp) = thermal(item);
    if !temp.is_finite() {
        return String::new();
    }
    let t = js::round(temp * 10.0) / 10.0;
    format!("{} °C", js::string(&js::json_number(t)))
}

pub fn tpm_summary(t: &Value) -> String {
    if !js::truthy(t.get("present")) {
        return String::new();
    }
    let version = s(t, "version");
    let said = if version.is_empty() {
        "TPM is present".to_owned()
    } else {
        format!("TPM {version}")
    };
    sentence(&[said])
}

pub fn virt_summary(v: &Value) -> String {
    let mut bits = Vec::new();
    let hypervisor = s(v, "hypervisor");
    if !hypervisor.is_empty() {
        bits.push(format!("Running on {hypervisor}"));
    } else if js::truthy(v.get("guest")) {
        bits.push("This machine is a guest".into());
    }
    if js::truthy(v.get("kvm")) {
        bits.push("KVM is available for guests".into());
    }
    sentence(&bits)
}

pub fn npu_summary(item: &Value) -> String {
    if js::truthy(item.get("npuIdentity")) {
        return clean_text(item.get("npuIdentity"), 160);
    }
    sentence(&[
        join(&[s(item, "vendor"), s(item, "name")], " "),
        s(item, "pciId"),
    ])
}

/// `gpuSummary(item)`: its vendor and name, driver and PCI id.
pub fn gpu_summary(item: &Value) -> String {
    let g = gpu(item);
    let t = |k: &str| field(&g, k);
    let driver = t("driver");
    sentence(&[
        join(&[t("vendor"), t("name")], " "),
        if driver.is_empty() {
            String::new()
        } else {
            format!("Driver {driver}")
        },
        t("pciId"),
    ])
}

/// `RichUi.formatBytes`.
pub fn format_bytes(bytes: f64) -> String {
    let mut n = if bytes.is_finite() && bytes >= 0.0 {
        bytes
    } else {
        0.0
    };
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut i = 0;
    while n >= 1024.0 && i < units.len() - 1 {
        n /= 1024.0;
        i += 1;
    }
    if i == 0 {
        return format!("{} {}", js::round(n), units[i]);
    }
    format!("{} {}", fixed(n, if n >= 10.0 { 1 } else { 2 }), units[i])
}

/// `hasText(...)`: whether any of the fields has text.
pub fn has_text(v: &Value, names: &[&str]) -> bool {
    names.iter().any(|name| !s(v, name).is_empty())
}

/// The first of the texts that is not empty, else `or`.
pub fn first(texts: &[String], or: &str) -> String {
    texts
        .iter()
        .find(|t| !t.is_empty())
        .cloned()
        .unwrap_or_else(|| or.to_owned())
}

/// A field as text, "" when it is not truthy.
pub fn field(v: &Value, name: &str) -> String {
    s(v, name)
}

/// A row of the page's own, by part: its label and what it says, none while it does not show.
pub fn row(section: &str, h: &Value) -> Option<(String, String)> {
    let (m, b, c) = (&h["machine"], &h["bios"], &h["cpu"]);
    let on = |v: &Value, name: &str| js::truthy(v.get(name));
    let says = |label: &str, said: String| Some((label.to_owned(), said));
    match section {
        "machine" if has_text(m, &["vendor", "name", "family", "chassis"]) => Some((
            first(&[s(m, "name"), s(m, "family")], "This machine"),
            first(
                &[machine_summary(m)],
                "The firmware did not name this chassis.",
            ),
        )),
        "identity" if has_text(m, &["serial", "sku"]) => says(
            "Identity",
            if has_text(m, &["serial"]) {
                format!("Serial {}{}.", s(m, "serial"), labelled(m, "sku", ". SKU "))
            } else {
                format!("SKU {}.", s(m, "sku"))
            },
        ),
        "board" if has_text(&h["board"], &["vendor", "name"]) => Some((
            first(&[s(&h["board"], "name")], "Board"),
            board_summary(&h["board"]),
        )),
        "chipset" if has_text(&h["chipset"], &["name", "vendor", "pciId"]) => Some((
            first(&[s(&h["chipset"], "name")], "Host bridge"),
            chipset_summary(&h["chipset"]),
        )),
        "bios" if has_text(b, &["vendor", "version", "date"]) || on(b, "uefi") => {
            let uefi = if on(b, "uefi") { "UEFI firmware." } else { "" };
            says("BIOS", first(&[bios_summary(b)], uefi))
        }
        "secureBoot" if on(&h["secureBoot"], "available") => says(
            "Secure Boot",
            format!(
                "The firmware is {}verifying boot loaders. Change this in UEFI setup, not here.",
                if on(&h["secureBoot"], "enabled") {
                    ""
                } else {
                    "not "
                }
            ),
        ),
        "tpm" if on(&h["tpm"], "present") => says("TPM", tpm_summary(&h["tpm"])),
        "cpu" if has_text(c, &["model", "vendor"]) => {
            Some((first(&[s(c, "model")], "CPU"), cpu_summary(c)))
        }
        "virt" => {
            let v = &h["virtualization"];
            let shows = on(v, "guest") || on(v, "kvm") || has_text(v, &["hypervisor"]);
            let label = if on(v, "guest") { "Guest" } else { "Host" };
            let said = first(&[virt_summary(v)], "No hypervisor was reported.");
            shows.then(|| (label.to_owned(), said))
        }
        _ => None,
    }
}

/// The lists of parts, by part, as the page's lists hold them.
pub fn parts<'a>(section: &str, h: &'a Value) -> &'a [Value] {
    let list = match section {
        "modules" => &h["memory"]["modules"],
        other => &h[other],
    };
    array(Some(list))
}

/// A row of a list of parts: its label and what it says.
pub fn part(section: &str, r: &Value) -> (String, String) {
    match section {
        "modules" => (first(&[s(r, "locator")], "DIMM"), module_summary(r)),
        "npus" => (first(&[s(r, "name")], "NPU"), npu_summary(r)),
        "nics" => (first(&[s(r, "iface"), s(r, "name")], "NIC"), nic_summary(r)),
        "audio" => (
            first(&[s(r, "name")], "Audio"),
            match s(r, "driver") {
                d if d.is_empty() => "ALSA card. Volume and sinks are on Sound.".to_owned(),
                d => format!("ALSA {d}. Volume and sinks are on Sound."),
            },
        ),
        "usb" => {
            let speed = labelled(r, "speed", "");
            let speed = if speed.is_empty() {
                speed
            } else {
                format!("{speed} Mb/s")
            };
            let bits: Vec<String> = [s(r, "vendor"), speed]
                .into_iter()
                .filter(|b| !b.is_empty())
                .collect();
            let end = if bits.is_empty() { "" } else { "." };
            (
                first(&[s(r, "name")], "USB"),
                format!("{}{end}", bits.join(". ")),
            )
        }
        "batteries" => {
            let said = battery_summary(r);
            let said = if said.is_empty() {
                "Charge profiles stay on Power.".to_owned()
            } else {
                format!("{said} Charge profiles stay on Power.")
            };
            (first(&[s(r, "name")], "Battery"), said)
        }
        _ => (
            first(&[s(r, "name"), s(r, "type")], "Sensor"),
            thermal_summary(r),
        ),
    }
}

/// What the page's rows say, by part, for `tests/catalog_hardware.rs`.
pub fn described(h: &Value) -> Value {
    let mut out = Map::new();
    for section in [
        "machine",
        "identity",
        "board",
        "chipset",
        "bios",
        "secureBoot",
        "tpm",
        "cpu",
        "virt",
    ] {
        out.insert(section.into(), json!(row(section, h)));
    }
    for section in [
        "modules",
        "npus",
        "nics",
        "audio",
        "usb",
        "batteries",
        "thermals",
    ] {
        let rows: Vec<_> = parts(section, h).iter().map(|r| part(section, r)).collect();
        out.insert(section.into(), json!(rows));
    }
    Value::Object(out)
}
