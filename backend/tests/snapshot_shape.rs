mod common;

use common::{cleanup, request, temp_root};
use serde_json::json;

#[test]
fn a_snapshot_is_nested_and_leaves_out_unset_values() {
    let root = temp_root();
    request(
        &root,
        "plain",
        &json!({"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 4}),
    );
    request(
        &root,
        "plain",
        &json!({"op": "settings.set", "domain": "theme", "value": "tokyo"}),
    );
    let snap = request(&root, "plain", &json!({"op": "settings.snapshot"}));
    let doc = &snap["result"];
    assert_eq!(doc["hyprLook"]["gapsIn"], 4, "dotted domains nest");
    assert!(
        doc.get("hyprLook.gapsIn").is_none(),
        "no flat dotted key is left behind"
    );
    assert_eq!(doc["theme"], "tokyo");
    assert!(
        doc["hyprLook"].get("gapsOut").is_none(),
        "an unset nested value is absent, not null"
    );
    assert!(
        doc.as_object().unwrap().values().all(|v| !v.is_null()),
        "no top-level null"
    );
    cleanup(&root);
}

#[test]
fn a_snapshot_can_ask_for_some_keys() {
    let root = temp_root();
    request(
        &root,
        "plain",
        &json!({"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 4}),
    );
    request(
        &root,
        "plain",
        &json!({"op": "settings.set", "domain": "theme", "value": "tokyo"}),
    );
    let snap = request(
        &root,
        "plain",
        &json!({"op": "settings.snapshot", "group": "look", "keys": ["theme"]}),
    );
    let doc = snap["result"].as_object().unwrap();
    assert_eq!(doc["theme"], "tokyo");
    assert!(!doc.contains_key("hyprLook"), "unlisted keys are not read");

    let nested = request(
        &root,
        "plain",
        &json!({"op": "settings.snapshot", "keys": ["hyprLook"]}),
    );
    assert_eq!(nested["result"]["hyprLook"]["gapsIn"], 4);
    assert!(nested["result"].get("theme").is_none());
    cleanup(&root);
}
