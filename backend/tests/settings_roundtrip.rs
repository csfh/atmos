mod common;

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

use common::{cleanup, request, temp_root};

#[test]
fn every_domain_roundtrips_through_the_backend_process() {
    let root = temp_root();
    let listed = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.list"}),
    );
    let domains = listed["result"].as_array().expect("domain list");
    assert!(
        domains.len() > 100,
        "the GUI exposes more settings than this"
    );

    let mut probes = Vec::new();
    for (index, domain) in domains.iter().enumerate() {
        let key = domain["domain"].as_str().unwrap();
        let ty = domain["type"].as_str().unwrap();
        let file = domain["file"].as_str().unwrap();
        assert!(!file.starts_with('/'), "{key} file is inside the fixture");
        assert!(!file.contains(".."), "{key} file stays inside the fixture");
        assert!(
            !file.ends_with(".sqlite"),
            "{key} is not a private database"
        );
        assert!(
            file != "atmos-settings.json",
            "{key} is not a private prefs file"
        );
        let value = probe(index, ty, key);
        let set = request(
            &root,
            "omarchy",
            &serde_json::json!({"op": "settings.set", "domain": key, "value": value}),
        );
        assert_eq!(
            set["result"]["file"], domain["file"],
            "{key} reports its platform file"
        );
        probes.push((domain.clone(), value));
    }

    for (domain, value) in &probes {
        let key = domain["domain"].as_str().unwrap();
        let got = request(
            &root,
            "omarchy",
            &serde_json::json!({"op": "settings.get", "domain": key}),
        );
        assert_eq!(&got["result"], value, "{key} read does not match the write");
        let rel = domain["file"].as_str().unwrap();
        let text =
            fs::read_to_string(root.join(rel)).unwrap_or_else(|err| panic!("read {rel}: {err}"));
        assert!(
            file_contains(&text, domain, value),
            "{key} file {rel} does not contain {value}\n{text}"
        );
        println!("ok {key} {rel}");
    }

    let hidden = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "barVisible", "value": false}),
    );
    assert_eq!(hidden["result"]["value"], Value::Bool(false));
    let shell: Value =
        serde_json::from_str(&fs::read_to_string(root.join(".config/omarchy/shell.json")).unwrap())
            .unwrap();
    assert_eq!(shell["bar"]["visible"], Value::Bool(false), "{shell}");
    assert_eq!(shell["bar"]["position"], "probe-barPosition", "{shell}");
    assert!(shell.get("barVisible").is_none(), "{shell}");

    assert_pinned(&listed["result"], "theme", ".config/omarchy/theme.json");
    assert_pinned(
        &listed["result"],
        "hyprLook.gapsIn",
        ".config/hypr/looknfeel.lua",
    );
    assert_pinned(
        &listed["result"],
        "hyprInput.sensitivity",
        ".config/hypr/input.lua",
    );
    assert_pinned(
        &listed["result"],
        "barPosition",
        ".config/omarchy/shell.json",
    );
    assert_pinned(&listed["result"], "bindings", ".config/hypr/bindings.lua");
    assert_pinned(&listed["result"], "windowRules", ".config/hypr/atmos.lua");
    assert_pinned(&listed["result"], "workspaces", ".config/hypr/atmos.lua");
    assert_pinned(&listed["result"], "autostart", ".config/hypr/autostart.lua");
    assert_pinned(
        &listed["result"],
        "monitorRules",
        ".config/hypr/monitors.lua",
    );
    assert_pinned(&listed["result"], "hostname", "etc/hostname");
    assert_pinned(&listed["result"], "locale", "etc/locale.conf");
    assert_pinned(&listed["result"], "parallelDownloads", "etc/pacman.conf");
    assert_pinned(
        &listed["result"],
        "nightlight",
        ".config/hypr/hyprsunset.conf",
    );
    assert_pinned(
        &listed["result"],
        "favorites",
        ".local/state/omarchy/atmos-favorites.json",
    );
    assert_pinned(&listed["result"], "atmosChannel", ".config/atmos/channel");
    assert_pinned(
        &listed["result"],
        "displayBrightness",
        "sys/class/backlight/acpi_video0/brightness",
    );

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
    let key = domain["domain"].as_str().unwrap();
    match domain["encoding"].as_str().unwrap() {
        "map" | "sentinel" => text.contains(&format!("\"{key}\":{encoded}")),
        "items" => text.contains(&format!("\"items\":{encoded}")),
        "whole" => text.trim() == encoded,
        "line" => {
            let prefix = domain["prefix"].as_str().unwrap_or("");
            text.contains(&format!("{prefix}{}", scalar(value)))
        }
        "shell" | "nested" => {
            let doc: Value = serde_json::from_str(text).unwrap_or(Value::Null);
            doc.get(key).is_none() && json_contains(&doc, value)
        }
        "lua" => !text.contains("atmos-json") && text.contains(&scalar(value)),
        "flag" => text.contains("hl.config"),
        other => panic!("unknown encoding {other}"),
    }
}

fn scalar(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(true) => "true".into(),
        Value::Bool(false) => "false".into(),
        Value::Number(number) => number.to_string(),
        other => other.to_string(),
    }
}

fn json_contains(doc: &Value, needle: &Value) -> bool {
    if doc == needle {
        return true;
    }
    match doc {
        Value::Object(map) => map.values().any(|value| json_contains(value, needle)),
        Value::Array(items) => items.iter().any(|value| json_contains(value, needle)),
        _ => false,
    }
}

#[test]
fn request_without_root_writes_user_files_under_home() {
    let home = temp_root();
    let body = serde_json::to_string(&serde_json::json!({
        "op": "settings.set",
        "domain": "clockFormat",
        "value": "HH:mm"
    }))
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_atmos-backend"))
        .env("HOME", &home)
        .env_remove("ATMOS_ROOT")
        .args(["--backend", "omarchy", "request"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout {}\nstderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let shell: Value =
        serde_json::from_str(&fs::read_to_string(home.join(".config/omarchy/shell.json")).unwrap())
            .unwrap();
    assert!(shell.get("clockFormat").is_none(), "{shell}");
    let text = serde_json::to_string(&shell).unwrap();
    assert!(text.contains("HH:mm"), "{shell}");

    let mut plain = Command::new(env!("CARGO_BIN_EXE_atmos-backend"))
        .env("HOME", &home)
        .env_remove("ATMOS_ROOT")
        .args(["--backend", "plain", "request"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    plain
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"op":"platform"}"#)
        .unwrap();
    let plain_out = plain.wait_with_output().unwrap();
    assert!(
        plain_out.status.success(),
        "{}",
        String::from_utf8_lossy(&plain_out.stderr)
    );
    let platform: Value = serde_json::from_slice(&plain_out.stdout).unwrap();
    assert_eq!(platform["platform"]["id"], "plain");
    assert_eq!(platform["version"], "0.1.0");

    cleanup(&home);
}

#[test]
fn planted_documents_keep_their_shape() {
    let root = temp_root();
    fs::create_dir_all(root.join(".config/omarchy")).unwrap();
    fs::create_dir_all(root.join(".config/hypr")).unwrap();
    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(
        root.join(".config/omarchy/shell.json"),
        "{\n  \"bar\": { \"position\": \"top\", \"transparent\": false },\n  \"idle\": { \"lock\": 300 },\n  \"plugins\": []\n}\n",
    )
    .unwrap();
    fs::write(
        root.join(".config/hypr/looknfeel.lua"),
        "-- keep outside\n-- atmos:look begin\nhl.config({\n  general = {\n    gaps_in = 5,\n    gaps_out = 10,\n  },\n})\n-- atmos:look end\n",
    )
    .unwrap();
    fs::write(
        root.join("etc/pacman.conf"),
        "HoldPkg = pacman\nParallelDownloads = 5\nColor\n",
    )
    .unwrap();

    let bar = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "barPosition", "value": "left"}),
    );
    assert_eq!(bar["result"]["value"], "left");
    let gaps = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 12}),
    );
    assert_eq!(gaps["result"]["value"], 12);
    let downloads = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "parallelDownloads", "value": 8}),
    );
    assert_eq!(downloads["result"]["value"], 8);

    let shell: Value =
        serde_json::from_str(&fs::read_to_string(root.join(".config/omarchy/shell.json")).unwrap())
            .unwrap();
    assert_eq!(shell["bar"]["position"], "left", "{shell}");
    assert_eq!(shell["bar"]["transparent"], false, "{shell}");
    assert_eq!(shell["idle"]["lock"], 300, "{shell}");
    assert_eq!(shell["plugins"], serde_json::json!([]), "{shell}");
    assert!(shell.get("barPosition").is_none(), "{shell}");

    let lua = fs::read_to_string(root.join(".config/hypr/looknfeel.lua")).unwrap();
    assert!(lua.contains("-- keep outside"), "{lua}");
    assert!(lua.contains("gaps_in = 12"), "{lua}");
    assert!(lua.contains("gaps_out = 10"), "{lua}");
    assert!(!lua.contains("atmos-json"), "{lua}");

    let pacman = fs::read_to_string(root.join("etc/pacman.conf")).unwrap();
    assert!(pacman.contains("HoldPkg = pacman"), "{pacman}");
    assert!(pacman.contains("Color"), "{pacman}");
    assert!(pacman.contains("ParallelDownloads = 8"), "{pacman}");
    assert!(!pacman.contains("ParallelDownloads = 5"), "{pacman}");

    let again = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "barPosition"}),
    );
    assert_eq!(again["result"], "left");
    let gaps_again = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "hyprLook.gapsIn"}),
    );
    assert_eq!(gaps_again["result"], 12);
    let downloads_again = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "parallelDownloads"}),
    );
    assert_eq!(downloads_again["result"], 8);

    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "hyprLook.cursorSize", "value": 24}),
    );
    let resized = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "hyprLook.cursorSize", "value": 36}),
    );
    assert_eq!(resized["result"]["value"], 36);
    let lua = fs::read_to_string(root.join(".config/hypr/looknfeel.lua")).unwrap();
    assert!(lua.contains("gaps_in = 12"), "{lua}");
    assert!(lua.contains("HYPRCURSOR_SIZE\", \"36\""), "{lua}");
    assert!(lua.contains("XCURSOR_SIZE\", \"36\""), "{lua}");
    assert!(!lua.contains("\"24\""), "{lua}");

    cleanup(&root);
}
