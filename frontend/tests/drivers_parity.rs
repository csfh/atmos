//! The Rust Drivers page is the QML one until the QML one goes: `services/Hardware.js` runs under
//! Node and must list and sum up the same GPUs as `drivers.rs` for every inventory the test makes
//! up, the QML page's own expressions for the stack and the hybrid switch (restated here, and
//! checked to be the page's) must say what `Graphics` says, a Refresh must read what
//! `SnapshotGroups` reads, and the failure text of a job must be `Failure.applyBanner`'s. Without
//! `node` the test says so and stops.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use atmos_ui::drivers::{self, Graphics};
use atmos_ui::jobs;
use atmos_ui::settings::{self, Settings};
use serde_json::{Value, json};

const NODE: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const { load } = require(root + "/tests/harness.js");
const H = load("services/Hardware.js");
const G = load("services/SnapshotGroups.js");
const F = load("services/Failure.js");
const input = JSON.parse(fs.readFileSync(0, "utf8"));
// DriversPage.qml's expressions, with Omarchy's properties as `O`.
function stack(O) {
  if (!(O.hwNvidia || O.hwVulkan)) return null;
  const description = O.hwNvidia
    ? (O.hwNvidiaGsp ? "NVIDIA, with GSP firmware (Turing or newer)."
      : (O.hwNvidiaWithoutGsp ? "NVIDIA, without GSP firmware (Maxwell, Pascal, or Volta)." : "NVIDIA."))
    : "Vulkan.";
  const hint = O.hwNvidia ? "omarchy hw nvidia" : "omarchy hw vulkan";
  const copied = O.hwNvidia
    ? (O.hwNvidiaGsp ? "NVIDIA GSP" : (O.hwNvidiaWithoutGsp ? "NVIDIA without GSP" : "NVIDIA"))
    : "Vulkan";
  return [description, hint, copied];
}
function hybrid(O) {
  return O.hybridGpuMode === "Integrated"
    ? "Using the integrated GPU only. Switch to hybrid if you want the dedicated GPU."
    : (O.hybridGpuMode === "Hybrid"
      ? "Hybrid mode. The dedicated GPU can wake for a game or CUDA."
      : "This machine can switch between integrated-only and hybrid.");
}
function confirm(O) {
  return O.hybridGpuMode === "Integrated"
    ? "Turn the dedicated GPU on (hybrid) and reboot."
    : "Use only the integrated GPU and reboot.";
}
const EMPTY = { hwNvidia: false, hwNvidiaGsp: false, hwNvidiaWithoutGsp: false, hwVulkan: false,
  hybridGpuAvailable: false, hybridGpuMode: "" };
process.stdout.write(JSON.stringify({
  gpus: input.hardware.map((raw) => H.normalize(raw).gpus.map((g) => [g.name, g.vendor, g.pciId, g.driver, H.gpuSummary(g)])),
  graphics: input.flags.map((f) => {
    const O = Object.assign({}, EMPTY, f);
    const any = !!(O.hwNvidia || O.hwVulkan || O.hybridGpuAvailable);
    return [stack(O), hybrid(O), confirm(O), any, O.hybridGpuAvailable === true];
  }),
  all: G.topicsForGroups("all"),
  failures: input.failures.map((f) => F.applyBanner(f.summary, f.code, F.commandText(f.err, f.out))),
}));
"#;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn node(input: &Value) -> Option<Value> {
    let Ok(mut child) = Command::new("node")
        .arg("-e")
        .arg(NODE)
        .env("REPO_DIR", repo())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    else {
        eprintln!("skip: node is not installed");
        return None;
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "the JavaScript side failed");
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

fn hardware() -> Vec<Value> {
    let long = "x".repeat(200);
    vec![
        json!({}),
        json!(null),
        json!({"gpus": "no"}),
        json!({"gpus": [
            {"name": "  RTX 4070\n", "vendor": "NVIDIA", "pciId": "10de:2786", "driver": "nvidia"},
            {"name": "Not Specified", "pciId": "0000"},
            {"vendor": "AMD"},
            {"name": "Radeon 780M", "vendor": "Advanced Micro Devices, Inc. [AMD/ATI]", "driver": ""},
            {"name": long, "pciId": "1002:15bf.", "driver": "amdgpu\tkms"},
            {"name": "Iris\r\n\r\nXe", "vendor": "unknown", "pciId": "FFFF", "driver": "i915"},
            {"name": 42, "pciId": true},
            {"name": "\u{a0}Arc A770\u{a0}", "vendor": "None", "pciId": "8086:56a0"},
            {"name": "Ünïcödé 😀", "vendor": "OEM"},
            "junk",
            null,
        ]}),
    ]
}

fn flags() -> Vec<Value> {
    let mut out = vec![json!({})];
    for nvidia in [false, true] {
        for gsp in [false, true] {
            for without in [false, true] {
                for vulkan in [false, true] {
                    out.push(json!({"hwNvidia": nvidia, "hwNvidiaGsp": gsp,
                        "hwNvidiaWithoutGsp": without, "hwVulkan": vulkan}));
                }
            }
        }
    }
    for mode in ["Integrated", "Hybrid", "", "other"] {
        out.push(json!({"hybridGpuAvailable": true, "hybridGpuMode": mode}));
    }
    out
}

fn failures() -> Vec<Value> {
    vec![
        json!({"summary": {"ok": true, "message": "x"}, "code": 0, "err": "", "out": ""}),
        json!({"summary": {"ok": false, "message": " fwupd: no device "}, "code": 1, "err": "", "out": ""}),
        json!({"summary": {"ok": false, "message": "", "warnings": ["noise"]}, "code": 1, "err": "", "out": ""}),
        json!({"summary": {"ok": false, "message": ""}, "code": 2, "err": "", "out": ""}),
        json!({"summary": null, "code": 0, "err": "e", "out": "o"}),
        json!({"summary": null, "code": 3, "err": " e ", "out": "o"}),
        json!({"summary": null, "code": 3, "err": "", "out": " o "}),
        json!({"summary": null, "code": 3, "err": "same", "out": "same"}),
        json!({"summary": null, "code": 3, "err": "", "out": ""}),
    ]
}

#[test]
fn the_rust_drivers_give_what_the_qml_page_gives() {
    let (hardware, flags, failures) = (hardware(), flags(), failures());
    let Some(js) = node(&json!({"hardware": hardware, "flags": flags, "failures": failures}))
    else {
        return;
    };
    for (raw, js) in hardware.iter().zip(js["gpus"].as_array().unwrap()) {
        let ours: Vec<Value> = drivers::gpus(raw)
            .iter()
            .map(|g| {
                json!([
                    g.name,
                    g.vendor,
                    g.pci_id,
                    g.driver,
                    drivers::gpu_summary(g)
                ])
            })
            .collect();
        assert_eq!(&Value::Array(ours), js, "the GPUs of {raw}");
    }
    for (f, js) in flags.iter().zip(js["graphics"].as_array().unwrap()) {
        let mut s = Settings::default();
        s.adopt("settings.host", 1, f);
        let g = Graphics::of(&s);
        let ours = json!([
            g.stack(),
            g.hybrid_text(),
            g.switch_text(),
            g.any(&[]),
            g.hybrid
        ]);
        assert_eq!(&ours, js, "with {f}");
    }
    assert_eq!(json!(settings::ALL_TOPICS), js["all"]);
    for (f, js) in failures.iter().zip(js["failures"].as_array().unwrap()) {
        let summary = f.get("summary").filter(|s| !s.is_null());
        let text = jobs::failure_text(
            summary,
            f["code"].as_i64().unwrap(),
            f["err"].as_str().unwrap(),
            f["out"].as_str().unwrap(),
        );
        assert_eq!(&json!(text), js, "{f}");
    }
}

/// The expressions above are the page's, and the jobs it runs are registry entries.
#[test]
fn the_qml_page_says_what_the_test_restates_and_runs_these_jobs() {
    let qml = std::fs::read_to_string(repo().join("pages/DriversPage.qml")).unwrap();
    for phrase in [
        "NVIDIA, with GSP firmware (Turing or newer).",
        "NVIDIA, without GSP firmware (Maxwell, Pascal, or Volta).",
        "Using the integrated GPU only. Switch to hybrid if you want the dedicated GPU.",
        "Hybrid mode. The dedicated GPU can wake for a game or CUDA.",
        "Turn the dedicated GPU on (hybrid) and reboot.",
        "Omarchy.toggleHybridGpu()",
        "Omarchy.updateFirmware()",
    ] {
        assert!(qml.contains(phrase), "the page says {phrase:?}");
    }
    let omarchy = std::fs::read_to_string(repo().join("services/Omarchy.qml")).unwrap();
    for job in [drivers::switch_job(), drivers::firmware_job()] {
        let call = format!(
            "runJob(Requests.job(\"{}\", {{}}), \"{}\", {{ sudo: true }})",
            job.name, job.kind
        );
        assert!(omarchy.contains(&call), "Omarchy.qml runs {call}");
        assert!(job.sudo);
    }
}
