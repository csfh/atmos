mod common;

use std::fs;

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
        assert_eq!(response["result"][field], value, "{kind} dropped the fixture value");
        assert_eq!(response["result"]["platform"], "omarchy");
        assert_eq!(response["result"]["collector"].as_str().unwrap().is_empty(), false);
        println!("ok {kind} {field}={value}");
    }
    cleanup(&root);
}
