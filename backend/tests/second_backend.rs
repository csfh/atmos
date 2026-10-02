mod common;

use std::fs;

use common::{cleanup, request, temp_root};

#[test]
fn the_same_requests_hit_plain_with_a_different_platform() {
    let omarchy_root = temp_root();
    let plain_root = temp_root();
    let set_body = serde_json::json!({
        "op": "settings.set",
        "domain": "theme",
        "value": "shared-theme"
    });
    let set_text = serde_json::to_string(&set_body).unwrap();
    let omarchy_set = request(
        &omarchy_root,
        "omarchy",
        &serde_json::from_str(&set_text).unwrap(),
    );
    let plain_set = request(
        &plain_root,
        "plain",
        &serde_json::from_str(&set_text).unwrap(),
    );
    assert_eq!(set_text, serde_json::to_string(&set_body).unwrap());
    assert_eq!(omarchy_set["platform"]["id"], "omarchy");
    assert_eq!(omarchy_set["platform"]["compositor"], "hyprland");
    assert_eq!(plain_set["platform"]["id"], "plain");
    assert_eq!(plain_set["platform"]["compositor"], "none");
    assert_ne!(omarchy_set["platform"]["id"], plain_set["platform"]["id"]);
    assert_eq!(omarchy_set["result"]["value"], "shared-theme");
    assert_eq!(plain_set["result"]["value"], "shared-theme");
    assert_ne!(omarchy_set["result"]["file"], plain_set["result"]["file"]);

    let get_body = r#"{"op":"settings.get","domain":"theme"}"#;
    let omarchy_get = request(
        &omarchy_root,
        "omarchy",
        &serde_json::from_str(get_body).unwrap(),
    );
    let plain_get = request(
        &plain_root,
        "plain",
        &serde_json::from_str(get_body).unwrap(),
    );
    assert_eq!(omarchy_get["result"], "shared-theme");
    assert_eq!(plain_get["result"], "shared-theme");

    let omarchy_hw = omarchy_root.join(".local/state/omarchy/display/hardware.json");
    let plain_hw = plain_root.join(".local/state/plain/display/hardware.json");
    fs::create_dir_all(omarchy_hw.parent().unwrap()).unwrap();
    fs::create_dir_all(plain_hw.parent().unwrap()).unwrap();
    let omarchy_cpu = "omarchy-fixture-cpu";
    let plain_cpu = "plain-fixture-cpu";
    fs::write(
        &omarchy_hw,
        serde_json::json!({"cpuModel": omarchy_cpu}).to_string(),
    )
    .unwrap();
    fs::write(
        &plain_hw,
        serde_json::json!({"cpuModel": plain_cpu}).to_string(),
    )
    .unwrap();

    let display_body = r#"{"op":"display.get","kind":"hardware"}"#;
    assert_eq!(display_body, r#"{"op":"display.get","kind":"hardware"}"#);
    let omarchy_display = request(
        &omarchy_root,
        "omarchy",
        &serde_json::from_str(display_body).unwrap(),
    );
    let plain_display = request(
        &plain_root,
        "plain",
        &serde_json::from_str(display_body).unwrap(),
    );
    assert_eq!(omarchy_display["result"]["cpuModel"], omarchy_cpu);
    assert_eq!(plain_display["result"]["cpuModel"], plain_cpu);
    assert_ne!(
        omarchy_display["result"]["cpuModel"],
        plain_display["result"]["cpuModel"]
    );
    assert_eq!(omarchy_display["result"]["collector"], "hw-inventory.py");
    assert_eq!(plain_display["result"]["collector"], "plain-hardware");
    assert_ne!(
        omarchy_display["result"]["platform"],
        plain_display["result"]["platform"]
    );
    println!(
        "ok platforms {} vs {} payload {} vs {}",
        omarchy_display["platform"]["id"],
        plain_display["platform"]["id"],
        omarchy_display["result"]["cpuModel"],
        plain_display["result"]["cpuModel"]
    );
    cleanup(&omarchy_root);
    cleanup(&plain_root);
}
