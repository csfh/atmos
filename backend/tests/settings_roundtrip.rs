mod common;

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
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
        "nightlight" => text.contains("# atmos:nightlight = true"),
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
        "audioOutputMuted" => text.contains("output-muted true"),
        "audioInputMuted" => text.contains("input-muted true"),
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
fn planted_bindings_keep_lines_outside_the_sentinel() {
    let root = temp_root();
    let file = root.join(".config/hypr/bindings.lua");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        "\
-- keep outside
o.bind(\"SUPER+Q\", \"Outside\", \"true\")
-- atmos:bindings begin
o.bind(\"SUPER+Z\", \"Old\", \"false\")
-- atmos:bindings end
",
    )
    .unwrap();

    request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "settings.set",
            "domain": "bindings",
            "value": [{
                "keys": "SUPER+A",
                "label": "Probe",
                "command": "true",
                "unbind": false
            }]
        }),
    );

    let text = fs::read_to_string(&file).unwrap();
    let begin = text
        .find("-- atmos:bindings begin")
        .expect("bindings sentinel");
    let outside = text.find("SUPER+Q").expect("outside bind");
    let inside = text.find("SUPER+A").expect("new bind");
    assert!(text.contains("-- keep outside"), "{text}");
    assert!(outside < begin, "{text}");
    assert!(inside > begin, "{text}");
    assert!(
        text.contains("o.bind(\"SUPER+Q\", \"Outside\", \"true\")"),
        "{text}"
    );
    assert!(
        text.contains("o.bind(\"SUPER+A\", \"Probe\", \"true\")"),
        "{text}"
    );
    assert!(!text.contains("SUPER+Z"), "{text}");
    assert!(!text.contains("atmos-json"), "{text}");

    let got = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "bindings"}),
    );
    assert!(
        list_has(&got["result"], "keys", "SUPER+Q"),
        "{}",
        got["result"]
    );
    assert!(
        list_has(&got["result"], "keys", "SUPER+A"),
        "{}",
        got["result"]
    );
    assert!(
        !list_has(&got["result"], "keys", "SUPER+Z"),
        "{}",
        got["result"]
    );

    cleanup(&root);
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
