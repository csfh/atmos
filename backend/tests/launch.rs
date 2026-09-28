mod common;

use std::fs;
use std::process::Output;

use common::{cleanup, invoke, temp_root};

#[test]
fn snapshot_launches_match_on_one_fixture() {
    let root = temp_root();
    let root_arg = root.display().to_string();
    let set = invoke(
        &["--backend", "omarchy", "--root", &root_arg, "request"],
        r#"{"op":"settings.set","domain":"theme","value":"fixture-theme-alpha"}"#,
    );
    assert!(set.status.success(), "{}", String::from_utf8_lossy(&set.stderr));

    let samples = [
        ("live", "sampleHost", "fixture-live-host"),
        ("hardware", "cpuModel", "fixture-hw-cpu"),
        ("disks", "diskName", "fixture-disk-name"),
        ("services", "unitName", "fixture-unit-name"),
        ("software", "packageName", "fixture-package-name"),
        ("diagnostics", "reportId", "fixture-report-id"),
    ];
    for (kind, field, value) in samples {
        let path = root.join(format!(".local/state/omarchy/display/{kind}.json"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::json!({ field: value }).to_string()).unwrap();
    }

    let first = snapshot(&root_arg);
    let second = snapshot(&root_arg);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout).unwrap();
    assert!(text.contains("\"version\": \"0.1.0\""), "{text}");
    assert!(text.contains("fixture-theme-alpha"), "{text}");
    for (_, _, value) in samples {
        assert!(text.contains(value), "{value} missing from snapshot");
    }
    cleanup(&root);
}

fn snapshot(root: &str) -> Output {
    invoke(&["--backend", "omarchy", "--root", root, "snapshot"], "")
}
