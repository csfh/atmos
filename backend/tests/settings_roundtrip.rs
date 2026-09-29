mod common;

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

use serde_json::Value;

use atmos_backend::domain;
use atmos_backend::effect::{self, Effect};

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
        let value = probe_for(index, ty, key);
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
        let on = value.as_bool().unwrap_or(false);
        if let Some(argv) = expected_command(key, on) {
            assert_eq!(
                got["result"], *value,
                "{key} read does not match the write\nwritten {value}\ngot {}",
                got["result"]
            );
            assert_command_logged(&root, key, argv, value);
            assert_no_command_shadow(&root, key);
            println!(
                "ok {key} commands.log {}",
                serde_json::to_string(&serde_json::json!(argv)).unwrap()
            );
            continue;
        }
        assert!(
            round_ok(key, value, &got["result"]),
            "{key} read does not match the write\nwritten {value}\ngot {}",
            got["result"]
        );
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
    assert_command_logged(
        &root,
        "barVisible",
        &["omarchy", "toggle", "bar", "on"],
        &Value::Bool(false),
    );
    let shell: Value =
        serde_json::from_str(&fs::read_to_string(root.join(".config/omarchy/shell.json")).unwrap())
            .unwrap();
    assert!(shell["bar"].get("visible").is_none(), "{shell}");
    assert_eq!(shell["bar"]["position"], "probe-barPosition", "{shell}");
    assert!(shell.get("barVisible").is_none(), "{shell}");
    assert!(shell["idle"].get("stayAwake").is_none(), "{shell}");
    assert!(shell["idle"].get("screensaverEnabled").is_none(), "{shell}");
    let input = fs::read_to_string(root.join(".config/hypr/input.lua")).unwrap_or_default();
    assert!(!input.contains("touchpad_enabled"), "{input}");
    assert!(!input.contains("touchscreen_enabled"), "{input}");
    assert!(!root
        .join(".local/state/omarchy/notifications.json")
        .exists());
    println!(
        "ok barVisible false commands.log {}",
        serde_json::to_string(&serde_json::json!(["omarchy", "toggle", "bar", "on"])).unwrap()
    );

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

fn probe_for(index: usize, ty: &str, domain: &str) -> Value {
    match domain {
        "bindings" => serde_json::json!([{
            "keys": "SUPER+A",
            "label": "Probe",
            "command": "true",
            "unbind": false
        }]),
        "windowRules" => serde_json::json!([{
            "match": "probe-window",
            "placement": "float"
        }]),
        "workspaces" => serde_json::json!([{
            "id": "1",
            "name": "ProbeWs",
            "persistent": true
        }]),
        "autostart" => serde_json::json!([{
            "command": "probe-autostart",
            "delay": 0,
            "enabled": true
        }]),
        "monitorRules" => serde_json::json!([{
            "output": "DP-1",
            "mode": "preferred",
            "position": "auto",
            "scale": 1
        }]),
        "envVars" => serde_json::json!([{
            "key": "ATMOS_PROBE",
            "value": "probe-envVars"
        }]),
        "envPathPrepend" => serde_json::json!("/opt/probe"),
        "nightlightDay" => serde_json::json!("07:11"),
        "nightlightNight" => serde_json::json!("20:11"),
        "nightlightTemperature" => serde_json::json!(4500),
        "audioOutputVolume" | "audioInputVolume" => serde_json::json!(40),
        "mimePdf" => serde_json::json!("mimePdf.desktop"),
        "mimeImage" => serde_json::json!("mimeImage.desktop"),
        "mimeVideo" => serde_json::json!("mimeVideo.desktop"),
        _ => probe(index, ty, domain),
    }
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
        "hypr" => hypr_file_contains(key, text, value),
        "doc" => doc_file_contains(key, text, value),
        other => panic!("unknown encoding {other}"),
    }
}

fn round_ok(key: &str, written: &Value, got: &Value) -> bool {
    match key {
        "bindings" => list_has(got, "keys", "SUPER+A") && list_has(got, "command", "true"),
        "windowRules" => list_has(got, "match", "probe-window"),
        "workspaces" => list_has(got, "name", "ProbeWs"),
        "autostart" => list_has(got, "command", "probe-autostart"),
        "monitorRules" => list_has(got, "output", "DP-1"),
        _ => got == written,
    }
}

fn list_has(value: &Value, field: &str, expect: &str) -> bool {
    value.as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item.get(field).and_then(Value::as_str) == Some(expect))
    })
}

fn hypr_file_contains(key: &str, text: &str, value: &Value) -> bool {
    if text.contains("atmos-json") {
        return false;
    }
    match key {
        "bindings" => text.contains("o.bind(\"SUPER+A\", \"Probe\", \"true\")"),
        "windowRules" => text.contains("o.window(\"probe-window\""),
        "workspaces" => text.contains("default_name = \"ProbeWs\""),
        "autostart" => text.contains("o.launch_on_start(\"probe-autostart\")"),
        "monitorRules" => text.contains("output = \"DP-1\""),
        "workspaceWrapSwitch" | "workspaceWheelSwitch" => {
            let name = if key == "workspaceWrapSwitch" {
                "wrapSwitch"
            } else {
                "wheelSwitch"
            };
            let flag = if value.as_bool() == Some(true) {
                "true"
            } else {
                "false"
            };
            text.contains(&format!("-- atmos:{name} = {flag}"))
        }
        _ => false,
    }
}

fn doc_file_contains(key: &str, text: &str, value: &Value) -> bool {
    if text.contains("atmos-json") {
        return false;
    }
    let rendered = scalar(value);
    match key {
        "nightlight" | "audioOutputMuted" | "audioInputMuted" => false,
        "nightlightDay" => text.contains("time = 07:11"),
        "nightlightNight" => text.contains("time = 20:11"),
        "nightlightNightOn" => text.matches("profile {").count() >= 2,
        "nightlightTemperature" => text.contains(&format!("temperature = {rendered}")),
        "envPathPrepend" => text.contains(&format!("PATH={rendered}:$PATH")),
        "envVars" => text.contains("ATMOS_PROBE=probe-envVars"),
        "mimePdf" => text.contains(&format!("application/pdf={rendered}")),
        "mimeImage" => {
            text.contains(&format!("image/png={rendered}"))
                && text.contains(&format!("image/jpeg={rendered}"))
                && text.contains(&format!("image/webp={rendered}"))
                && text.contains(&format!("image/gif={rendered}"))
        }
        "mimeVideo" => {
            text.contains(&format!("video/mp4={rendered}"))
                && text.contains(&format!("video/webm={rendered}"))
                && text.contains(&format!("video/x-matroska={rendered}"))
        }
        "audioOutputVolume" => text.contains(&format!("output-volume {rendered}")),
        "audioInputVolume" => text.contains(&format!("input-volume {rendered}")),
        "audioTuningOn" => text.contains("tuning true"),
        "bluetooth" | "wifiRadio" | "suspendEnabled" | "crashCapture" => text.trim() == rendered,
        "presentationMode" => text.contains("\"on\":true"),
        "chargeLimit" => text.trim() == rendered,
        "snapperNumberLimit" => text.contains(&format!("NUMBER_LIMIT=\"{rendered}\"")),
        "snapperTimeline" => text.contains("TIMELINE_CREATE=\"yes\""),
        "passwordlessSudo" => text.contains("NOPASSWD"),
        "fingerprintConfigured" => text.contains("pam_fprintd.so"),
        "fido2Configured" => text.contains("pam_u2f.so"),
        "sshdEnabled" => text.contains("enable sshd.service"),
        "fstrimEnabled" => text.trim() == "enabled",
        "directBoot" => text.contains("Boot0001* Omarchy"),
        "sudolessDocker" => text.trim() == "docker",
        _ => false,
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

#[test]
fn effect_table_matches_every_settings_domain() {
    let mut rows = effect::row_keys();
    let mut specs: Vec<_> = domain::specs().iter().map(|spec| spec.key).collect();
    let width = rows.len();
    rows.sort_unstable();
    rows.dedup();
    assert_eq!(rows.len(), width, "duplicate effect row");
    specs.sort_unstable();
    assert_eq!(rows, specs, "effect rows and domain specs differ");
    for spec in domain::specs() {
        let row = effect::get(spec.key).unwrap_or_else(|| panic!("{} has no effect row", spec.key));
        let command = expected_command(spec.key, true).is_some();
        match (row, expected_sentinel(spec.key), command) {
            (Effect::Command(form), None, true) => {
                for on in [false, true] {
                    let value = Value::Bool(on);
                    let expected = expected_command(spec.key, on).unwrap_or_else(|| {
                        panic!("{} is Command without the Settings.js argv", spec.key)
                    });
                    assert_eq!(
                        effect::command_argv(form, &value),
                        expected,
                        "{} value={on}",
                        spec.key
                    );
                }
            }
            (Effect::Command(_), _, _) => {
                panic!("{} is Command without the Settings.js argv", spec.key);
            }
            (Effect::Sentinel { kind }, Some(expected), false) => {
                assert_eq!(kind, expected, "{}", spec.key);
            }
            (Effect::Sentinel { .. }, _, _) => {
                panic!("{} is Sentinel without a kind in the test", spec.key);
            }
            (Effect::Document, None, false) => {}
            (Effect::Document, _, _) => {
                panic!(
                    "{} is Document but the test expects a command or sentinel",
                    spec.key
                );
            }
        }
    }
    assert!(effect::get("not-a-domain").is_none());
    println!("ok effect rows {}", specs.len());
}

#[test]
fn effect_table_round_trips_through_the_backend() {
    let plants = [
        Plant {
            domain: "bindings",
            rel: ".config/hypr/bindings.lua",
            outside_field: Some("keys"),
            outside: "SUPER+Q",
            inside_field: Some("keys"),
            inside: "SUPER+Z",
            needle: "o.bind(\"SUPER+Q\", \"Outside\", \"true\")",
            body: "\
-- keep outside
o.bind(\"SUPER+Q\", \"Outside\", \"true\")
-- atmos:bindings begin
o.bind(\"SUPER+Z\", \"Inside\", \"true\")
-- atmos:bindings end
",
        },
        Plant {
            domain: "windowRules",
            rel: ".config/hypr/atmos.lua",
            outside_field: Some("match"),
            outside: "outside-window",
            inside_field: Some("match"),
            inside: "inside-window",
            needle: "o.window(\"outside-window\"",
            body: "\
-- keep outside
o.window(\"outside-window\", { float = true })
-- atmos:windows begin
o.window(\"inside-window\", { tile = true })
-- atmos:windows end
",
        },
        Plant {
            domain: "autostart",
            rel: ".config/hypr/autostart.lua",
            outside_field: Some("command"),
            outside: "outside-start",
            inside_field: Some("command"),
            inside: "inside-start",
            needle: "o.launch_on_start(\"outside-start\")",
            body: "\
-- keep outside
o.launch_on_start(\"outside-start\")
-- atmos:autostart begin
o.launch_on_start(\"inside-start\")
-- atmos:autostart end
",
        },
        Plant {
            domain: "workspaces",
            rel: ".config/hypr/atmos.lua",
            outside_field: None,
            outside: "OutsideWs",
            inside_field: None,
            inside: "InsideWs",
            needle: "default_name = \"OutsideWs\"",
            body: "\
-- keep outside
hl.workspace_rule({ workspace = \"11\", persistent = true, default_name = \"OutsideWs\" })
-- atmos:workspaces begin
-- atmos:wrapSwitch = true
-- atmos:wheelSwitch = true
hl.workspace_rule({ workspace = \"1\", persistent = true, default_name = \"InsideWs\" })
-- atmos:workspaces end
",
        },
        Plant {
            domain: "monitorRules",
            rel: ".config/hypr/monitors.lua",
            outside_field: None,
            outside: "HDMI-A-9",
            inside_field: None,
            inside: "DP-1",
            needle: "output = \"HDMI-A-9\"",
            body: "\
-- keep outside
hl.monitor({ output = \"HDMI-A-9\", mode = \"preferred\", position = \"auto\", scale = 1 })
-- atmos:monitors begin
hl.monitor({ output = \"DP-1\", mode = \"preferred\", position = \"auto\", scale = 1 })
-- atmos:monitors end
",
        },
    ];

    for plant in plants {
        let root = temp_root();
        let file = root.join(plant.rel);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, plant.body).unwrap();
        let got = request(
            &root,
            "omarchy",
            &serde_json::json!({"op": "settings.get", "domain": plant.domain}),
        );
        if let (Some(field), Some(inside_field)) = (plant.outside_field, plant.inside_field) {
            assert_eq!(
                row_managed(&got["result"], field, plant.outside),
                Value::Bool(false),
                "{} dropped managed:false\n{}",
                plant.domain,
                got["result"]
            );
            assert_eq!(
                row_managed(&got["result"], inside_field, plant.inside),
                Value::Bool(true),
                "{} inside row\n{}",
                plant.domain,
                got["result"]
            );
        } else {
            let encoded = got["result"].to_string();
            assert!(
                !encoded.contains(plant.outside),
                "{} get included the outside call\n{encoded}",
                plant.domain
            );
        }
        request(
            &root,
            "omarchy",
            &serde_json::json!({
                "op": "settings.set",
                "domain": plant.domain,
                "value": got["result"]
            }),
        );
        let text = fs::read_to_string(&file).unwrap();
        assert_eq!(
            text.matches(plant.needle).count(),
            1,
            "{} duplicated the outside call\n{text}",
            plant.domain
        );
        assert!(
            text.contains(plant.inside),
            "{} lost the inside call\n{text}",
            plant.domain
        );
        assert!(!text.contains("atmos-json"), "{text}");
        println!("ok {} {} once", plant.domain, plant.needle);
        cleanup(&root);
    }

    let root = temp_root();
    request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "settings.set",
            "domain": "windowRules",
            "value": [{"match": "probe-window", "placement": "float"}]
        }),
    );
    let got = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "windowRules"}),
    );
    assert_eq!(
        row_managed(&got["result"], "match", "dev.csfh.atmos"),
        Value::Bool(false),
        "window seed lost managed:false\n{}",
        got["result"]
    );
    request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "settings.set",
            "domain": "windowRules",
            "value": got["result"]
        }),
    );
    let text = fs::read_to_string(root.join(".config/hypr/atmos.lua")).unwrap();
    assert_eq!(
        text.matches("dev.csfh.atmos").count(),
        1,
        "window seed was copied into the sentinel\n{text}"
    );
    let begin = text
        .find("-- atmos:windows begin")
        .expect("windows sentinel");
    let seed = text.find("dev.csfh.atmos").expect("window seed");
    assert!(seed < begin, "{text}");
    assert!(text.contains("probe-window"), "{text}");
    println!("ok windowRules seed outside once");
    cleanup(&root);

    for key in [
        "nightlight",
        "audioOutputMuted",
        "audioInputMuted",
        "barVisible",
        "screensaverEnabled",
        "stayAwake",
        "touchpadEnabled",
        "touchscreenEnabled",
        "doNotDisturb",
    ] {
        for on in [false, true] {
            let argv = expected_command(key, on)
                .unwrap_or_else(|| panic!("{key} is missing from the Settings.js argv table"));
            let root = temp_root();
            let value = Value::Bool(on);
            let set = request(
                &root,
                "omarchy",
                &serde_json::json!({"op": "settings.set", "domain": key, "value": value}),
            );
            assert_eq!(set["result"]["value"], value, "{set}");
            let got = request(
                &root,
                "omarchy",
                &serde_json::json!({"op": "settings.get", "domain": key}),
            );
            assert_eq!(got["result"], value, "{got}");
            assert_command_logged(&root, key, argv, &value);
            assert_fresh_command_shadow_absent(&root, key);
            println!(
                "ok {key} {on} commands.log {}",
                serde_json::to_string(&serde_json::json!(argv)).unwrap()
            );
            cleanup(&root);
        }
    }
}

struct Plant {
    domain: &'static str,
    rel: &'static str,
    outside_field: Option<&'static str>,
    outside: &'static str,
    inside_field: Option<&'static str>,
    inside: &'static str,
    needle: &'static str,
    body: &'static str,
}

#[test]
fn planted_atmos_lua_keeps_the_other_sentinel() {
    let root = temp_root();
    let file = root.join(".config/hypr/atmos.lua");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        "\
-- keep outside atmos
-- atmos:windows begin
o.window(\"old-window\", { float = true })
-- atmos:windows end
-- atmos:workspaces begin
-- atmos:wrapSwitch = true
-- atmos:wheelSwitch = true
hl.workspace_rule({ workspace = \"1\", persistent = true, default_name = \"Kept\" })
-- atmos:workspaces end
",
    )
    .unwrap();

    request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "settings.set",
            "domain": "windowRules",
            "value": [{"match": "probe-window", "placement": "float"}]
        }),
    );
    let after_window = fs::read_to_string(&file).unwrap();
    assert!(
        after_window.contains("-- keep outside atmos"),
        "{after_window}"
    );
    assert!(
        after_window.contains("o.window(\"probe-window\""),
        "{after_window}"
    );
    assert!(!after_window.contains("old-window"), "{after_window}");
    assert!(
        after_window.contains("default_name = \"Kept\""),
        "{after_window}"
    );
    assert!(!after_window.contains("atmos-json"), "{after_window}");

    request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "settings.set",
            "domain": "workspaces",
            "value": [{"id": "1", "name": "ProbeWs", "persistent": true}]
        }),
    );
    let after_workspace = fs::read_to_string(&file).unwrap();
    assert!(
        after_workspace.contains("-- keep outside atmos"),
        "{after_workspace}"
    );
    assert!(
        after_workspace.contains("o.window(\"probe-window\""),
        "{after_workspace}"
    );
    assert!(
        after_workspace.contains("default_name = \"ProbeWs\""),
        "{after_workspace}"
    );
    assert!(
        !after_workspace.contains("default_name = \"Kept\""),
        "{after_workspace}"
    );
    assert!(!after_workspace.contains("atmos-json"), "{after_workspace}");

    let windows = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "windowRules"}),
    );
    assert!(
        list_has(&windows["result"], "match", "probe-window"),
        "{}",
        windows["result"]
    );
    let workspaces = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "workspaces"}),
    );
    assert!(
        list_has(&workspaces["result"], "name", "ProbeWs"),
        "{}",
        workspaces["result"]
    );

    cleanup(&root);
}

#[test]
fn audio_volume_runs_set_audio_without_a_private_map() {
    let home = temp_root();
    let bin = home.join("bin");
    let log = home.join("pactl.log");
    fs::create_dir_all(&bin).unwrap();
    let log_path = log.display().to_string();
    fs::write(
        bin.join("omarchy"),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log_path}'\nif [ \"$1\" = audio ] && [ \"$2\" = output ] && [ \"$3\" = sink ]; then\n  printf '%s\\n' fake-sink\nfi\nexit 0\n"
        ),
    )
    .unwrap();
    fs::write(
        bin.join("pactl"),
        format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log_path}'\nexit 0\n"),
    )
    .unwrap();
    fs::write(
        bin.join("wpctl"),
        format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log_path}'\nexit 0\n"),
    )
    .unwrap();
    for name in ["omarchy", "pactl", "wpctl"] {
        let path = bin.join(name);
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let set = request_at_home(
        &home,
        &repo,
        &path,
        &serde_json::json!({
            "op": "settings.set",
            "domain": "audioOutputVolume",
            "value": 40
        }),
    );
    assert_eq!(set["result"]["value"], 40, "{set}");
    let recorded = fs::read_to_string(&log).unwrap_or_default();
    assert!(
        recorded.contains("set-sink-volume fake-sink 40%"),
        "{recorded}"
    );
    assert!(!home.join(".config/omarchy/audio.json").exists());
    let level = fs::read_to_string(home.join(".local/state/omarchy/audio-level")).unwrap();
    assert!(level.contains("output-volume 40"), "{level}");

    let got = request_at_home(
        &home,
        &repo,
        &path,
        &serde_json::json!({"op": "settings.get", "domain": "audioOutputVolume"}),
    );
    assert_eq!(got["result"], 40, "{got}");

    cleanup(&home);
}

fn expected_command(key: &str, on: bool) -> Option<&'static [&'static str]> {
    match key {
        "nightlight" => Some(&["omarchy", "toggle", "nightlight"]),
        "audioOutputMuted" => Some(&["omarchy", "audio", "output", "volume", "mute-toggle"]),
        "audioInputMuted" => Some(&["omarchy", "audio", "input", "mute"]),
        "doNotDisturb" => Some(&["omarchy", "toggle", "notification", "silencing"]),
        "barVisible" => Some(if on {
            &["omarchy", "toggle", "bar", "off"]
        } else {
            &["omarchy", "toggle", "bar", "on"]
        }),
        "screensaverEnabled" => Some(if on {
            &["omarchy", "toggle", "screensaver-off", "off"]
        } else {
            &["omarchy", "toggle", "screensaver-off", "on"]
        }),
        "stayAwake" => Some(if on {
            &["omarchy", "toggle", "idle", "stay-awake"]
        } else {
            &["omarchy", "toggle", "idle", "allow-idle"]
        }),
        "touchpadEnabled" => Some(if on {
            &["omarchy", "toggle", "touchpad", "on"]
        } else {
            &["omarchy", "toggle", "touchpad", "off"]
        }),
        "touchscreenEnabled" => Some(if on {
            &["omarchy", "toggle", "touchscreen", "on"]
        } else {
            &["omarchy", "toggle", "touchscreen", "off"]
        }),
        _ => None,
    }
}

fn expected_sentinel(key: &str) -> Option<&'static str> {
    match key {
        "bindings" => Some("bindings"),
        "windowRules" => Some("windows"),
        "autostart" => Some("autostart"),
        "monitorRules" => Some("monitors"),
        "workspaces" | "workspaceWrapSwitch" | "workspaceWheelSwitch" => Some("workspaces"),
        _ => None,
    }
}

fn row_managed(list: &Value, field: &str, expect: &str) -> Value {
    list.as_array()
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get(field).and_then(Value::as_str) == Some(expect))
        })
        .and_then(|item| item.get("managed"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn assert_command_logged(root: &std::path::Path, key: &str, argv: &[&str], value: &Value) {
    let text = fs::read_to_string(root.join("commands.log")).unwrap_or_default();
    let want = serde_json::json!(argv);
    let found = text.lines().any(|line| {
        let Ok(parsed) = serde_json::from_str::<Value>(line) else {
            return false;
        };
        parsed.get("domain").and_then(Value::as_str) == Some(key)
            && parsed.get("argv") == Some(&want)
            && parsed.get("value") == Some(value)
    });
    assert!(found, "{key} argv missing from commands.log\n{text}");
}

fn assert_no_command_shadow(root: &std::path::Path, key: &str) {
    assert!(!root.join(".config/omarchy/audio.json").exists(), "{key}");
    assert!(!root.join(".config/omarchy/env.json").exists(), "{key}");
    if let Ok(text) = fs::read_to_string(root.join(".config/hypr/hyprsunset.conf")) {
        assert!(
            !text.contains("# atmos:nightlight"),
            "{key} wrote a nightlight comment\n{text}"
        );
    }
    if matches!(key, "audioOutputMuted" | "audioInputMuted") {
        if let Ok(text) = fs::read_to_string(root.join(".local/state/omarchy/audio-level")) {
            assert!(!text.contains("output-muted"), "{key}\n{text}");
            assert!(!text.contains("input-muted"), "{key}\n{text}");
        }
    }
    if let Ok(text) = fs::read_to_string(root.join(".config/omarchy/shell.json")) {
        let shell: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        match key {
            "barVisible" => assert!(shell.pointer("/bar/visible").is_none(), "{key}\n{shell}"),
            "stayAwake" => assert!(shell.pointer("/idle/stayAwake").is_none(), "{key}\n{shell}"),
            "screensaverEnabled" => {
                assert!(
                    shell.pointer("/idle/screensaverEnabled").is_none(),
                    "{key}\n{shell}"
                );
            }
            _ => {}
        }
    }
    if matches!(key, "touchpadEnabled" | "touchscreenEnabled") {
        if let Ok(text) = fs::read_to_string(root.join(".config/hypr/input.lua")) {
            let needle = if key == "touchpadEnabled" {
                "touchpad_enabled"
            } else {
                "touchscreen_enabled"
            };
            assert!(!text.contains(needle), "{key}\n{text}");
        }
    }
    if key == "doNotDisturb" {
        assert!(
            !root
                .join(".local/state/omarchy/notifications.json")
                .exists(),
            "{key} wrote notifications.json"
        );
    }
}

fn assert_fresh_command_shadow_absent(root: &std::path::Path, key: &str) {
    assert!(!root.join(".config/omarchy/audio.json").exists(), "{key}");
    assert!(!root.join(".config/omarchy/env.json").exists(), "{key}");
    assert!(
        !root.join(".config/hypr/hyprsunset.conf").exists(),
        "{key} wrote hyprsunset.conf"
    );
    assert!(
        !root.join(".local/state/omarchy/audio-level").exists(),
        "{key} wrote audio-level"
    );
    match key {
        "barVisible" | "screensaverEnabled" | "stayAwake" => {
            assert!(
                !root.join(".config/omarchy/shell.json").exists(),
                "{key} wrote shell.json"
            );
        }
        "touchpadEnabled" | "touchscreenEnabled" => {
            assert!(
                !root.join(".config/hypr/input.lua").exists(),
                "{key} wrote input.lua"
            );
        }
        "doNotDisturb" => {
            assert!(
                !root
                    .join(".local/state/omarchy/notifications.json")
                    .exists(),
                "{key} wrote notifications.json"
            );
        }
        _ => {}
    }
}

#[test]
fn live_command_keys_spawn_the_settings_js_argv() {
    let home = temp_root();
    let bin = home.join("bin");
    let log = home.join("omarchy.log");
    fs::create_dir_all(&bin).unwrap();
    let log_path = log.display().to_string();
    fs::write(
        bin.join("omarchy"),
        format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log_path}'\nexit 0\n"),
    )
    .unwrap();
    let mut perms = fs::metadata(bin.join("omarchy")).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(bin.join("omarchy"), perms).unwrap();
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    for (key, on) in [
        ("nightlight", true),
        ("audioOutputMuted", true),
        ("audioInputMuted", true),
        ("barVisible", false),
        ("barVisible", true),
        ("screensaverEnabled", false),
        ("screensaverEnabled", true),
        ("stayAwake", true),
        ("stayAwake", false),
        ("touchpadEnabled", true),
        ("touchpadEnabled", false),
        ("touchscreenEnabled", true),
        ("touchscreenEnabled", false),
        ("doNotDisturb", true),
        ("doNotDisturb", false),
    ] {
        let set = request_at_home(
            &home,
            &repo,
            &path,
            &serde_json::json!({"op": "settings.set", "domain": key, "value": on}),
        );
        assert_eq!(set["result"]["value"], Value::Null, "{set}");
        let got = request_at_home(
            &home,
            &repo,
            &path,
            &serde_json::json!({"op": "settings.get", "domain": key}),
        );
        assert_eq!(got["result"], Value::Null, "{got}");
        let argv = expected_command(key, on).unwrap();
        println!("ok live {key} {on} {}", argv.join(" "));
    }
    let recorded = fs::read_to_string(&log).unwrap_or_default();
    assert!(recorded.contains("toggle nightlight"), "{recorded}");
    assert!(
        recorded.contains("audio output volume mute-toggle"),
        "{recorded}"
    );
    assert!(recorded.contains("audio input mute"), "{recorded}");
    assert!(recorded.contains("toggle bar on"), "{recorded}");
    assert!(recorded.contains("toggle bar off"), "{recorded}");
    assert!(recorded.contains("toggle screensaver-off on"), "{recorded}");
    assert!(
        recorded.contains("toggle screensaver-off off"),
        "{recorded}"
    );
    assert!(recorded.contains("toggle idle stay-awake"), "{recorded}");
    assert!(recorded.contains("toggle idle allow-idle"), "{recorded}");
    assert!(recorded.contains("toggle touchpad on"), "{recorded}");
    assert!(recorded.contains("toggle touchpad off"), "{recorded}");
    assert!(recorded.contains("toggle touchscreen on"), "{recorded}");
    assert!(recorded.contains("toggle touchscreen off"), "{recorded}");
    assert!(
        recorded.contains("toggle notification silencing"),
        "{recorded}"
    );
    assert!(
        recorded
            .lines()
            .all(|line| !line.starts_with("toggle notification silencing ")),
        "{recorded}"
    );
    assert!(!recorded.contains("restart hyprsunset"), "{recorded}");
    assert!(!home.join(".config/omarchy/shell.json").exists());
    assert!(!home.join(".config/hypr/input.lua").exists());
    assert!(!home
        .join(".local/state/omarchy/notifications.json")
        .exists());
    let sunset = home.join(".config/hypr/hyprsunset.conf");
    if let Ok(text) = fs::read_to_string(&sunset) {
        assert!(!text.contains("# atmos:nightlight"), "{text}");
    }
    assert!(!home.join(".config/omarchy/audio.json").exists());
    assert!(!home.join(".config/omarchy/env.json").exists());
    if let Ok(text) = fs::read_to_string(home.join(".local/state/omarchy/audio-level")) {
        assert!(!text.contains("output-muted"), "{text}");
        assert!(!text.contains("input-muted"), "{text}");
    }
    println!("ok live commands.log\n{recorded}");
    cleanup(&home);
}

fn request_at_home(
    home: &std::path::Path,
    repo: &std::path::Path,
    path: &str,
    body: &Value,
) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_atmos-backend"))
        .env("HOME", home)
        .env("PATH", path)
        .env("ATMOS_ROOT", repo)
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
        .write_all(serde_json::to_string(body).unwrap().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout {}\nstderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("response json");
    assert_eq!(value["ok"], Value::Bool(true), "{value}");
    assert_eq!(value["version"], "0.1.0", "{value}");
    value
}
