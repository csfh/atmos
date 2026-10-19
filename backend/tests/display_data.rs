mod common;

use std::fs;
use std::process::{Command, Stdio};

use common::{cleanup, request, temp_root};

#[test]
fn display_kinds_return_fixture_identity() {
    let root = temp_root();
    let samples = [
        ("live", "sampleHost", "fixture-live-host"),
        ("hardware", "cpuModel", "fixture-hw-cpu"),
        ("disks", "diskName", "fixture-disk-name"),
        ("services", "unitName", "fixture-unit-name"),
        ("software", "packageName", "fixture-package-name"),
        ("diagnostics", "reportId", "fixture-report-id"),
    ];
    for (kind, field, value) in samples {
        let rel = format!(".local/state/omarchy/display/{kind}.json");
        let path = root.join(&rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::json!({ field: value }).to_string()).unwrap();
        let response = request(
            &root,
            "omarchy",
            &serde_json::json!({"op": "display.get", "kind": kind}),
        );
        assert_eq!(
            response["result"][field], value,
            "{kind} dropped the fixture value"
        );
        assert_eq!(response["result"]["platform"], "omarchy");
        assert!(!response["result"]["collector"].as_str().unwrap().is_empty());
        println!("ok {kind} {field}={value}");
    }
    cleanup(&root);
}

#[test]
fn display_collects_natively_without_atmos_root_or_scripts() {
    for kind in [
        "live",
        "hardware",
        "disks",
        "services",
        "software",
        "diagnostics",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .env_remove("ATMOS_ROOT")
            .env("PATH", "/nonexistent")
            .args(["--backend", "omarchy", "display", kind])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("spawn ratmos");
        let err = String::from_utf8_lossy(&output.stderr);
        // These two cannot be collected without their tool, and say so rather
        // than answer with a machine that has no disks or services.
        if let Some(tool) = match kind {
            "disks" => Some("lsblk"),
            "services" => Some("systemctl"),
            _ => None,
        } {
            assert!(!output.status.success(), "{kind} succeeded with no {tool}");
            assert!(err.contains(tool), "{kind} did not name {tool}\n{err}");
            continue;
        }
        assert!(output.status.success(), "{kind} failed\n{err}");
        let value: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("display json");
        assert_eq!(
            value["collector"],
            format!(
                "omarchy-{}",
                match kind {
                    "live" => "sampler",
                    other => other,
                }
            )
        );
        assert!(value.is_object(), "{kind} is not an object");
    }
}
