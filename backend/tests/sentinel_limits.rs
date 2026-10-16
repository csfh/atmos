mod common;

use std::fs;

use common::{cleanup, request, temp_root};
use serde_json::json;

const LIMITS: [(&str, i64); 3] = [
    ("sentinelCpuPercent", 90),
    ("sentinelMemoryPercent", 35),
    ("sentinelPortAgeDays", 14),
];

/// An unmapped shell key falls back to one shared slot, so three limits written
/// without their own paths would overwrite each other.
#[test]
fn the_three_limits_keep_their_own_values() {
    for backend in ["omarchy", "plain"] {
        let root = temp_root();
        for (domain, _) in LIMITS {
            let unset = request(
                &root,
                backend,
                &json!({"op": "settings.get", "domain": domain}),
            );
            assert!(unset["result"].is_null(), "{backend} {domain} starts unset");
        }
        for (domain, value) in LIMITS {
            request(
                &root,
                backend,
                &json!({"op": "settings.set", "domain": domain, "value": value}),
            );
        }
        for (domain, value) in LIMITS {
            let read = request(
                &root,
                backend,
                &json!({"op": "settings.get", "domain": domain}),
            );
            assert_eq!(read["result"], json!(value), "{backend} {domain}");
        }
        cleanup(&root);
    }
}

#[test]
fn the_omarchy_limits_live_under_the_atmos_key_in_shell_json() {
    let root = temp_root();
    for (domain, value) in LIMITS {
        request(
            &root,
            "omarchy",
            &json!({"op": "settings.set", "domain": domain, "value": value}),
        );
    }
    let text = fs::read_to_string(root.join(".config/omarchy/shell.json")).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        doc["atmos"]["sentinel"],
        json!({"cpuPercent": 90, "memoryPercent": 35, "portAgeDays": 14}),
        "{text}"
    );
    assert!(doc.get("sentinel").is_none(), "nothing beside bar: {text}");
    cleanup(&root);
}
