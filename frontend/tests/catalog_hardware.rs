//! The Hardware page's rows (`pages/hardware.rs`) are its QML rows: `Hardware.js`'s `normalize`
//! and summaries, and each row's own expressions, run under Node on the same inventories as
//! `hardware.rs`. Without `node` the test says so and stops.

mod common;

use atmos_ui::hardware as hw;
use common::qml;
use serde_json::{Value, json};

/// A full inventory, one of placeholders and odd values, and none.
fn inventories() -> Vec<Value> {
    vec![
        json!({
            "machine": {"vendor": "LENOVO", "name": "ThinkPad X1", "family": "ThinkPad", "version": "Gen 11",
                "serial": "PF3ABC", "sku": "21HM", "chassis": 10},
            "board": {"vendor": "LENOVO", "name": "21HMCTO1WW", "version": "SDK0T76530 WIN"},
            "chipset": {"name": "Raptor Lake-P Host Bridge", "vendor": "Intel", "pciId": "8086:a706",
                "role": "Host bridge", "southbridge": "Raptor Lake LPC"},
            "bios": {"vendor": "LENOVO", "version": "N3XET53W (1.28 )", "date": "07/11/2024", "uefi": true},
            "cpu": {"model": "13th Gen Intel(R) Core(TM) i7-1365U", "vendor": "GenuineIntel", "arch": "x86_64",
                "cores": 10, "threads": 12, "sockets": 1, "mhz": 1200.4, "maxMhz": 5200,
                "caches": "L2 6.5 MiB", "flags": ["vmx", "avx2", "AES", "vmx", "lm", "x"],
                "virtualization": "VT-x"},
            "memory": {"total": 33361223680u64, "modules": [
                {"locator": "Controller0-ChannelA", "size": 17179869184u64, "typeCode": 35, "formCode": 13,
                    "speed": 6400, "manufacturer": "Samsung", "part": "K3KL3L30CM-BGCT", "rank": 2},
                {"locator": "", "size": 1610612736, "type": "DDR4"},
                {"locator": "", "size": 0}]},
            "npus": [{"name": "Meteor Lake NPU", "vendor": "Intel", "pciId": "8086:7d1d"}, {}],
            "nics": [{"iface": "wlan0", "name": "Intel AX211", "mac": "aa:bb:cc:dd:ee:ff", "driver": "iwlwifi",
                "wireless": true, "speed": 0}, {"iface": "enp0s31f6", "speed": 1000, "driver": "e1000e"}, {}],
            "audio": [{"name": "sof-hda-dsp", "driver": "sof"}, {"name": "HDMI"}, {}],
            "usb": [{"name": "Integrated Camera", "vendor": "Chicony", "speed": "480"},
                {"product": "Hub"}, {"vendor": "x"}],
            "batteries": [{"name": "BAT0", "status": "Discharging", "capacity": 87.6, "technology": "Li-poly"},
                {"capacity": 0}],
            "thermals": [{"name": "x86_pkg_temp", "temp": 47.25}, {"type": "acpitz", "temp": 30},
                {"name": "bad", "temp": "hot"}],
            "tpm": {"present": true, "version": "2.0"},
            "secureBoot": {"available": true, "enabled": false},
            "virtualization": {"kvm": true},
        }),
        json!({
            "machine": {"vendor": "To Be Filled By O.E.M.", "name": "System Product Name", "family": "Default string",
                "serial": "0000000", "sku": "SKU-1", "chassis": "Mini PC box"},
            "board": {"vendor": "ASRock", "name": "", "version": "Not Specified"},
            "chipset": {"pciId": "1022:14d8"},
            "bios": {"uefi": true},
            "cpu": {"vendor": "AuthenticAMD", "cores": 1, "threads": 1, "sockets": 2, "mhz": 3000.6,
                "flags": "nope"},
            "memory": {"total": 512, "modules": "nope"},
            "tpm": {"present": true},
            "secureBoot": {"available": true, "enabled": true},
            "virtualization": {"hypervisor": "KVM"},
            "nics": [{"name": "eth0"}],
        }),
        json!(null),
    ]
}

/// The text of `name:` in a QML block: its first line, and the lines after it that are not
/// another property or a child.
fn prop(block: &str, name: &str) -> String {
    let head = format!("{name}: ");
    let mut lines = block
        .lines()
        .map(str::trim)
        .skip_while(|l| !l.starts_with(&head));
    let mut out = vec![lines.next().unwrap()[head.len()..].to_owned()];
    let depth = |text: &[String]| {
        let all = text.concat();
        all.matches('{').count() as i64 - all.matches('}').count() as i64
    };
    for l in lines {
        // The brace that closes the row, not the expression.
        if l.starts_with('}') && depth(&out) <= 0 {
            break;
        }
        let is_prop = l
            .split_once(": ")
            .is_some_and(|(k, _)| !k.is_empty() && k.chars().all(char::is_alphanumeric));
        if is_prop || l.ends_with(" {") || l.starts_with("Prefs") {
            break;
        }
        out.push(l.to_owned());
    }
    out.join("\n")
}

/// The block of the first `SettingRow` after `anchor`.
fn row_after<'a>(src: &'a str, anchor: &str) -> &'a str {
    let rest = src.split(anchor).nth(1).unwrap();
    let start = rest.find("SettingRow {").unwrap();
    &rest[start..]
}

#[test]
fn the_hardware_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/HardwarePage.qml")).unwrap();
    // The rows of the page's own: label, description, and whether it shows.
    let own = [
        ("machine", "title: \"Machine\""),
        ("identity", "label: \"Identity\""),
        ("board", "title: \"Motherboard\""),
        ("chipset", "title: \"Chipset\""),
        ("bios", "label: \"BIOS\""),
        ("secureBoot", "label: \"Secure Boot\""),
        ("tpm", "label: \"TPM\""),
        ("cpu", "title: \"Processor\""),
        ("virt", "title: \"Virtualization\""),
    ];
    let mut exprs = serde_json::Map::new();
    for (name, anchor) in own {
        let anchor_at = src.find(anchor).unwrap();
        let block = if anchor.starts_with("label:") {
            &src[src[..anchor_at].rfind("SettingRow {").unwrap()..]
        } else {
            row_after(&src, anchor)
        };
        let get = |p: &str| {
            block
                .split("PrefsButton")
                .next()
                .unwrap()
                .contains(&format!("{p}: "))
                .then(|| prop(block, p))
        };
        exprs.insert(name.into(), json!({ "label": get("label"), "description": get("description"), "available": get("available") }));
    }
    // The rows each list draws, from its model.
    let lists = [
        ("modules", "root.hw.memory.modules"),
        ("npus", "root.hw.npus"),
        ("nics", "root.hw.nics"),
        ("audio", "root.hw.audio"),
        ("usb", "root.hw.usb"),
        ("batteries", "root.hw.batteries"),
        ("thermals", "root.hw.thermals"),
    ];
    for (name, model) in lists {
        let block = src.split(&format!("model: {model}")).nth(1).unwrap();
        exprs.insert(name.into(), json!({ "model": model, "label": prop(block, "label"), "description": prop(block, "description") }));
    }
    let installed = prop(
        src.split("label: \"Installed\"").nth(1).unwrap(),
        "valueText",
    );
    let script = format!(
        r#"{root}
const HardwareJs = load("services/Hardware.js");
const RichUi = load("services/RichUi.js");
const e = {exprs};
const run = (expr, root, modelData) => {{
  const body = expr.trim().startsWith("{{") ? expr.trim().slice(1, -1) : "return (" + expr + ");";
  return new Function("root", "HardwareJs", "RichUi", "modelData", body)(root, HardwareJs, RichUi, modelData);
}};
process.stdout.write(JSON.stringify({inventories}.map((raw) => {{
  const r = pageRoot("HardwarePage.qml", {{ hardware: raw }}, {{}});
  const h = HardwareJs.normalize(raw);
  const own = {{}};
  for (const [k, x] of Object.entries(e)) {{
    if (x.model) {{
      own[k] = run(x.model, r).map((m) => [String(run(x.label, r, m)), String(run(x.description, r, m))]);
    }} else {{
      const shows = x.available ? !!run(x.available, r) : true;
      own[k] = shows ? [x.label ? String(run(x.label, r)) : null, String(run(x.description, r))] : null;
    }}
  }}
  return {{
    hw: h, own,
    installed: run({installed}, r),
    summaries: [HardwareJs.machineSummary(h.machine), HardwareJs.boardSummary(h.board),
      HardwareJs.chipsetSummary(h.chipset), HardwareJs.biosSummary(h.bios), HardwareJs.cpuSummary(h.cpu),
      HardwareJs.tpmSummary(h.tpm), HardwareJs.virtSummary(h.virtualization)],
    bytes: [0, 512, 1536, 1048576, 33361223680, 1e13].map(RichUi.formatBytes),
  }};
}})));
"#,
        root = qml::PAGE_ROOT,
        exprs = Value::Object(exprs.clone()),
        inventories = json!(inventories()),
        installed = json!(installed),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    for (raw, theirs) in inventories().iter().zip(theirs.as_array().unwrap()) {
        let h = hw::normalize(Some(raw));
        assert_eq!(h, theirs["hw"], "normalize {raw}");
        let summaries = json!([
            hw::machine_summary(&h["machine"]),
            hw::board_summary(&h["board"]),
            hw::chipset_summary(&h["chipset"]),
            hw::bios_summary(&h["bios"]),
            hw::cpu_summary(&h["cpu"]),
            hw::tpm_summary(&h["tpm"]),
            hw::virt_summary(&h["virtualization"]),
        ]);
        assert_eq!(summaries, theirs["summaries"]);
        let total = h["memory"]["total"].as_f64().unwrap();
        assert_eq!(json!(hw::format_bytes(total)), theirs["installed"]);
        let bytes: Vec<String> = [0.0, 512.0, 1536.0, 1048576.0, 33361223680.0, 1e13]
            .into_iter()
            .map(hw::format_bytes)
            .collect();
        assert_eq!(json!(bytes), theirs["bytes"]);
        assert_eq!(atmos_ui_rows(&h), theirs["own"], "rows of {raw}");
    }
}

/// What `pages/hardware.rs` draws of each row, as the test reads the QML's.
fn atmos_ui_rows(h: &Value) -> Value {
    hw::described(h)
}
