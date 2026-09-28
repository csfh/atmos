mod common;

use std::fs;

use serde_json::Value;

use common::{cleanup, request, temp_root};

#[test]
fn every_domain_roundtrips_through_the_backend_process() {
    let root = temp_root();
    let listed = request(&root, "omarchy", &serde_json::json!({"op": "settings.list"}));
    let domains = listed["result"].as_array().expect("domain list");
    assert!(domains.len() > 100, "the GUI exposes more settings than this");

    let mut probes = Vec::new();
    for (index, domain) in domains.iter().enumerate() {
        let key = domain["domain"].as_str().unwrap();
        let ty = domain["type"].as_str().unwrap();
        let file = domain["file"].as_str().unwrap();
        assert!(!file.starts_with('/'), "{key} file is inside the fixture");
        assert!(!file.contains(".."), "{key} file stays inside the fixture");
        assert!(!file.ends_with(".sqlite"), "{key} is not a private database");
        assert!(file != "atmos-settings.json", "{key} is not a private prefs file");
        let value = probe(index, ty, key);
        let set = request(
            &root,
            "omarchy",
            &serde_json::json!({"op": "settings.set", "domain": key, "value": value}),
        );
        assert_eq!(set["result"]["file"], domain["file"], "{key} reports its platform file");
        probes.push((domain.clone(), value));
    }

    for (domain, value) in &probes {
        let key = domain["domain"].as_str().unwrap();
        let got = request(&root, "omarchy", &serde_json::json!({"op": "settings.get", "domain": key}));
        assert_eq!(&got["result"], value, "{key} read does not match the write");
        let rel = domain["file"].as_str().unwrap();
        let text = fs::read_to_string(root.join(rel)).unwrap_or_else(|err| panic!("read {rel}: {err}"));
        assert!(file_contains(&text, domain, value), "{key} file {rel} does not contain {value}\n{text}");
        println!("ok {key} {rel}");
    }

    let hidden = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "barVisible", "value": false}),
    );
    assert_eq!(hidden["result"]["value"], Value::Bool(false));
    let text = fs::read_to_string(root.join(".config/omarchy/shell.json")).unwrap();
    assert!(text.contains("\"barVisible\":false"), "{text}");

    assert_pinned(&listed["result"], "theme", ".config/omarchy/theme.json");
    assert_pinned(&listed["result"], "hyprLook.gapsIn", ".config/hypr/looknfeel.lua");
    assert_pinned(&listed["result"], "hyprInput.sensitivity", ".config/hypr/input.lua");
    assert_pinned(&listed["result"], "barPosition", ".config/omarchy/shell.json");
    assert_pinned(&listed["result"], "bindings", ".config/hypr/bindings.lua");
    assert_pinned(&listed["result"], "windowRules", ".config/hypr/atmos.lua");
    assert_pinned(&listed["result"], "workspaces", ".config/hypr/atmos.lua");
    assert_pinned(&listed["result"], "autostart", ".config/hypr/autostart.lua");
    assert_pinned(&listed["result"], "monitorRules", ".config/hypr/monitors.lua");
    assert_pinned(&listed["result"], "hostname", "etc/hostname");
    assert_pinned(&listed["result"], "locale", "etc/locale.conf");
    assert_pinned(&listed["result"], "parallelDownloads", "etc/pacman.conf");
    assert_pinned(&listed["result"], "nightlight", ".config/hypr/hyprsunset.conf");
    assert_pinned(&listed["result"], "favorites", ".local/state/omarchy/atmos-favorites.json");
    assert_pinned(&listed["result"], "atmosChannel", ".config/atmos/channel");
    assert_pinned(&listed["result"], "displayBrightness", "sys/class/backlight/acpi_video0/brightness");

    cleanup(&root);
}

fn assert_pinned(domains: &Value, key: &str, file: &str) {
    let found = domains
        .as_array()
        .unwrap()
        .iter()
        .find(|domain| domain["domain"] == key)
        .unwrap_or_else(|| panic!("missing {key}"));
    assert_eq!(found["file"], file, "{key}");
}

fn probe(index: usize, ty: &str, domain: &str) -> Value {
    let token = format!("probe-{}", domain.replace('.', "-"));
    match ty {
        "string" => Value::String(token),
        "int" => Value::from(20_000 + index as i64),
        "number" => {
            let text = format!("{}.25", 40 + index);
            Value::Number(text.parse().unwrap())
        }
        "bool" => Value::Bool(true),
        "list" => serde_json::json!([token]),
        "object" => serde_json::json!({"id": token}),
        other => panic!("unknown type {other} for {domain}"),
    }
}

fn file_contains(text: &str, domain: &Value, value: &Value) -> bool {
    let encoded = serde_json::to_string(value).unwrap();
    match domain["encoding"].as_str().unwrap() {
        "map" | "sentinel" => {
            let key = domain["domain"].as_str().unwrap();
            text.contains(&format!("\"{key}\":{encoded}"))
        }
        "items" => text.contains(&format!("\"items\":{encoded}")),
        "whole" => text.trim() == encoded,
        "line" => {
            let prefix = domain["prefix"].as_str().unwrap_or("");
            let rendered = match value {
                Value::String(s) => s.clone(),
                Value::Bool(true) => "true".into(),
                Value::Bool(false) => "false".into(),
                Value::Number(number) => number.to_string(),
                other => panic!("line value {other}"),
            };
            text.contains(&format!("{prefix}{rendered}"))
        }
        other => panic!("unknown encoding {other}"),
    }
}
