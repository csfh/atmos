//! Platform documents and the scripts that already own them.
//!
//! List domains go through `hypr-sentinel.py`, so the sentinel body is the
//! same Lua that script writes. Other domains write the file the existing
//! shell script writes, and a live (no `--root`) set also runs that script.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value};

pub fn read_hypr(path: &Path, kind: &str, key: &str) -> Result<Value, String> {
    if !path.is_file() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    match key {
        "workspaceWrapSwitch" => Ok(flag_comment(&text, "wrapSwitch")),
        "workspaceWheelSwitch" => Ok(flag_comment(&text, "wheelSwitch")),
        _ => {
            let begin = sentinel_begin(kind);
            if !text.contains(begin) && !text_has_calls(&text, kind) {
                return Ok(Value::Null);
            }
            hypr_list(kind, path)
        }
    }
}

pub fn write_hypr(path: &Path, kind: &str, key: &str, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let payload = match key {
        "bindings" | "windowRules" | "autostart" | "monitorRules" => {
            serde_json::json!({ "items": keep_managed(value) })
        }
        "workspaces" => {
            let wrap = flag_comment(&text, "wrapSwitch").as_bool().unwrap_or(true);
            let wheel = flag_comment(&text, "wheelSwitch").as_bool().unwrap_or(true);
            serde_json::json!({ "items": keep_managed(value), "wrapSwitch": wrap, "wheelSwitch": wheel })
        }
        "workspaceWrapSwitch" | "workspaceWheelSwitch" => {
            let mut items = if text.contains("-- atmos:workspaces begin") {
                keep_managed(&hypr_list("workspaces", path)?)
            } else {
                Value::Array(Vec::new())
            };
            if !items.is_array() {
                items = Value::Array(Vec::new());
            }
            let wrap = if key == "workspaceWrapSwitch" {
                value.as_bool().unwrap_or(true)
            } else {
                flag_comment(&text, "wrapSwitch").as_bool().unwrap_or(true)
            };
            let wheel = if key == "workspaceWheelSwitch" {
                value.as_bool().unwrap_or(true)
            } else {
                flag_comment(&text, "wheelSwitch").as_bool().unwrap_or(true)
            };
            serde_json::json!({ "items": items, "wrapSwitch": wrap, "wheelSwitch": wheel })
        }
        other => return Err(format!("no hypr block for {other}")),
    };
    hypr_apply(kind, path, &payload)
}

pub fn read_doc(root: Option<&Path>, path: &Path, key: &str) -> Result<Value, String> {
    match key {
        "passwordlessSudo" => {
            let check = if root.is_none() {
                passwordless_live_path()
            } else {
                path.to_path_buf()
            };
            if !check.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(&check).map_err(|err| err.to_string())?;
            Ok(Value::Bool(text.contains("NOPASSWD")))
        }
        "fingerprintConfigured" | "fido2Configured" => {
            if !path.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            let needle = if key == "fingerprintConfigured" {
                "pam_fprintd.so"
            } else {
                "pam_u2f.so"
            };
            Ok(Value::Bool(text.contains(needle)))
        }
        "sshdEnabled" => {
            if root.is_none() {
                return systemctl_enabled("sshd.service");
            }
            if !path.is_file() {
                return Ok(Value::Bool(false));
            }
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            Ok(Value::Bool(text.contains("enable sshd.service")))
        }
        "fstrimEnabled" => {
            if root.is_none() {
                return systemctl_enabled("fstrim.timer");
            }
            if !path.is_file() {
                return Ok(Value::Bool(false));
            }
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            Ok(Value::Bool(text.trim() == "enabled"))
        }
        "directBoot" => {
            if !path.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            Ok(Value::Bool(text.contains("Omarchy")))
        }
        "sudolessDocker" => {
            if !path.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            Ok(Value::Bool(text.contains("docker")))
        }
        _ => {
            if !path.is_file() {
                return Ok(Value::Null);
            }
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            read_doc_text(key, &text)
        }
    }
}

pub fn write_doc(root: Option<&Path>, path: &Path, key: &str, value: &Value) -> Result<(), String> {
    if root.is_none() && privileged(key) {
        return Ok(());
    }
    match key {
        "nightlightTemperature" | "nightlightDay" | "nightlightNight" | "nightlightNightOn" => {
            write_sunset(path, key, value)
        }
        "envVars" | "envPathPrepend" => write_env(path, key, value),
        "mimePdf" | "mimeImage" | "mimeVideo" => write_mime(path, key, value),
        "audioOutputVolume" | "audioInputVolume" | "audioTuningOn" => {
            write_audio_line(path, key, value)
        }
        "bluetooth" | "wifiRadio" | "suspendEnabled" | "crashCapture" => {
            let token = match value.as_bool() {
                Some(true) => "true",
                Some(false) => "false",
                None => return Err(format!("{key} expects a bool")),
            };
            atomic_text(path, &format!("{token}\n"))
        }
        "presentationMode" => {
            let on = value
                .as_bool()
                .ok_or_else(|| format!("{key} expects a bool"))?;
            let body = serde_json::json!({ "on": on, "until": 0, "minutes": 0 });
            let text = serde_json::to_string(&body).map_err(|err| err.to_string())?;
            atomic_text(path, &format!("{text}\n"))
        }
        "chargeLimit" => {
            let number = value
                .as_i64()
                .ok_or_else(|| "chargeLimit expects an int".to_string())?;
            atomic_text(path, &format!("{number}\n"))
        }
        "snapperNumberLimit" | "snapperTimeline" => write_snapper(path, key, value),
        "passwordlessSudo" => {
            let on = value
                .as_bool()
                .ok_or_else(|| "passwordlessSudo expects a bool".to_string())?;
            if on {
                atomic_text(path, "atmos ALL=(ALL) NOPASSWD: ALL\n")
            } else if path.exists() {
                fs::remove_file(path).map_err(|err| err.to_string())
            } else {
                Ok(())
            }
        }
        "fingerprintConfigured" | "fido2Configured" => write_pam(path, key, value),
        "sshdEnabled" => {
            let on = value
                .as_bool()
                .ok_or_else(|| "sshdEnabled expects a bool".to_string())?;
            let line = if on {
                "enable sshd.service\n"
            } else {
                "disable sshd.service\n"
            };
            atomic_text(path, line)
        }
        "fstrimEnabled" => {
            let on = value
                .as_bool()
                .ok_or_else(|| "fstrimEnabled expects a bool".to_string())?;
            atomic_text(path, if on { "enabled\n" } else { "disabled\n" })
        }
        "directBoot" => {
            let on = value
                .as_bool()
                .ok_or_else(|| "directBoot expects a bool".to_string())?;
            if on {
                atomic_text(path, "Boot0001* Omarchy\n")
            } else {
                atomic_text(path, "\n")
            }
        }
        "sudolessDocker" => {
            let on = value
                .as_bool()
                .ok_or_else(|| "sudolessDocker expects a bool".to_string())?;
            atomic_text(path, if on { "docker\n" } else { "\n" })
        }
        other => Err(format!("no platform document for {other}")),
    }
}

pub fn after_write(root: Option<&Path>, key: &str, value: &Value) -> Result<(), String> {
    let fixture = root.is_some();
    match key {
        "audioOutputVolume" => audio_script(fixture, "output-volume", value),
        "audioInputVolume" => audio_script(fixture, "input-volume", value),
        "wifiRadio" => {
            let flag = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            run_bash(fixture, "set-wifi-connection.sh", &["radio", flag])
        }
        _ if fixture => Ok(()),
        "bluetooth" => {
            let flag = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            spawn("omarchy", &["bluetooth", "power", flag]);
            Ok(())
        }
        "suspendEnabled" => {
            // The existing writer inverts this onto `omarchy toggle suspend-off`.
            let flag = if value.as_bool() == Some(true) {
                "off"
            } else {
                "on"
            };
            spawn("omarchy", &["toggle", "suspend-off", flag]);
            Ok(())
        }
        "crashCapture" => {
            spawn("omarchy", &["toggle", "crash", "capture"]);
            Ok(())
        }
        "presentationMode" => {
            let flag = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            run_bash(false, "set-presentation.sh", &[flag])
        }
        "chargeLimit" => {
            let number = value
                .as_i64()
                .ok_or_else(|| "chargeLimit expects an int".to_string())?
                .to_string();
            run_bash(false, "set-charge-limit.sh", &[&number])
        }
        "mimePdf" => mime_script("pdf", value),
        "mimeImage" => mime_script("image", value),
        "mimeVideo" => mime_script("video", value),
        "passwordlessSudo" => {
            let action = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            run_bash(false, "set-passwordless-sudo.sh", &[action])
        }
        "sshdEnabled" => {
            if value.as_bool() == Some(false) {
                run_bash(false, "set-sshd.sh", &["disable"])
            } else {
                spawn("systemctl", &["enable", "--now", "sshd.service"]);
                Ok(())
            }
        }
        "snapperNumberLimit" => {
            let number = value
                .as_i64()
                .ok_or_else(|| "snapperNumberLimit expects an int".to_string())?;
            if (1..=50).contains(&number) {
                let rendered = number.to_string();
                run_bash(false, "set-snapper-policy.sh", &["number-limit", &rendered])
            } else {
                Ok(())
            }
        }
        "snapperTimeline" => {
            let flag = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            run_bash(false, "set-snapper-policy.sh", &["timeline", flag])
        }
        "fstrimEnabled" => {
            if value.as_bool() == Some(true) {
                spawn("systemctl", &["enable", "--now", "fstrim.timer"]);
            } else {
                spawn("systemctl", &["disable", "--now", "fstrim.timer"]);
            }
            Ok(())
        }
        "customDns" => {
            let text = value.as_str().unwrap_or("");
            if text.contains('.') || text.contains(':') {
                run_bash(false, "set-dns-custom.sh", &[text])
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

fn privileged(key: &str) -> bool {
    matches!(
        key,
        "chargeLimit"
            | "passwordlessSudo"
            | "sshdEnabled"
            | "snapperNumberLimit"
            | "snapperTimeline"
            | "fstrimEnabled"
            | "fingerprintConfigured"
            | "fido2Configured"
            | "sudolessDocker"
            | "directBoot"
    )
}

fn missing_bool(root: Option<&Path>) -> Value {
    if root.is_some() {
        Value::Bool(false)
    } else {
        Value::Null
    }
}

fn passwordless_live_path() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_default();
    PathBuf::from(format!("/etc/sudoers.d/99-omarchy-nopasswd-{user}"))
}

fn systemctl_enabled(unit: &str) -> Result<Value, String> {
    let output = Command::new("systemctl")
        .args(["is-enabled", unit])
        .output();
    let Ok(output) = output else {
        return Ok(Value::Null);
    };
    let text = String::from_utf8_lossy(&output.stdout);
    if text.trim() == "enabled" {
        Ok(Value::Bool(true))
    } else if text.trim().is_empty() {
        Ok(Value::Null)
    } else {
        Ok(Value::Bool(false))
    }
}

fn read_doc_text(key: &str, text: &str) -> Result<Value, String> {
    match key {
        "nightlight" | "audioOutputMuted" | "audioInputMuted" => Ok(Value::Null),
        "nightlightTemperature" => {
            let state = sunset_state(text);
            if state.saw_temp {
                Ok(Value::from(state.temp))
            } else {
                Ok(Value::Null)
            }
        }
        "nightlightDay" => {
            let state = sunset_state(text);
            if state.saw_day {
                Ok(Value::String(state.day))
            } else {
                Ok(Value::Null)
            }
        }
        "nightlightNight" => {
            let state = sunset_state(text);
            if state.saw_night {
                Ok(Value::String(state.night))
            } else {
                Ok(Value::Null)
            }
        }
        "nightlightNightOn" => Ok(Value::Bool(sunset_state(text).night_on)),
        "envPathPrepend" => Ok(Value::String(env_state(text).0)),
        "envVars" => {
            let vars = env_state(text).1;
            let items = vars
                .into_iter()
                .map(|(key, value)| serde_json::json!({ "key": key, "value": value }))
                .collect();
            Ok(Value::Array(items))
        }
        "mimePdf" => Ok(mime_desktop(text, "application/pdf")),
        "mimeImage" => Ok(mime_desktop(text, "image/png")),
        "mimeVideo" => Ok(mime_desktop(text, "video/mp4")),
        "audioOutputVolume" => audio_number(text, "output-volume "),
        "audioInputVolume" => audio_number(text, "input-volume "),
        "audioTuningOn" => audio_bool(text, "tuning "),
        "bluetooth" | "wifiRadio" | "suspendEnabled" | "crashCapture" => {
            Ok(Value::Bool(text.trim() == "true"))
        }
        "presentationMode" => {
            let parsed: Value = serde_json::from_str(text).map_err(|err| err.to_string())?;
            Ok(Value::Bool(parsed["on"].as_bool().unwrap_or(false)))
        }
        "chargeLimit" => text
            .trim()
            .parse::<i64>()
            .map(Value::from)
            .map_err(|err| err.to_string()),
        "snapperNumberLimit" => Ok(Value::from(snapper_number(text))),
        "snapperTimeline" => Ok(Value::Bool(snapper_timeline(text))),
        other => Err(format!("cannot read {other}")),
    }
}

struct Sunset {
    day: String,
    night: String,
    temp: i64,
    night_on: bool,
    saw_day: bool,
    saw_night: bool,
    saw_temp: bool,
}

fn sunset_state(text: &str) -> Sunset {
    let mut day = "07:00".to_string();
    let mut night = "20:00".to_string();
    let mut temp = 4000_i64;
    let mut saw_day = false;
    let mut saw_night = false;
    let mut saw_temp = false;
    let mut profiles: Vec<Vec<String>> = Vec::new();
    let mut current: Option<Vec<String>> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# atmos:night = ") {
            night = rest.to_string();
            saw_night = true;
        }
        if let Some(rest) = trimmed.strip_prefix("# atmos:temperature = ") {
            if let Ok(number) = rest.parse() {
                temp = number;
                saw_temp = true;
            }
        }
        if trimmed.starts_with("profile") {
            current = Some(Vec::new());
            continue;
        }
        if trimmed == "}" {
            if let Some(body) = current.take() {
                profiles.push(body);
            }
            continue;
        }
        if let Some(body) = current.as_mut() {
            body.push(trimmed.to_string());
        }
    }
    if let Some(first) = profiles.first() {
        for line in first {
            if let Some(rest) = line.strip_prefix("time = ") {
                day = rest.to_string();
                saw_day = true;
            }
        }
    }
    let night_on = profiles.len() > 1;
    if night_on {
        for line in &profiles[1] {
            if let Some(rest) = line.strip_prefix("time = ") {
                night = rest.to_string();
                saw_night = true;
            }
            if let Some(rest) = line.strip_prefix("temperature = ") {
                if let Ok(number) = rest.parse() {
                    temp = number;
                    saw_temp = true;
                }
            }
        }
    }
    Sunset {
        day,
        night,
        temp,
        night_on,
        saw_day,
        saw_night,
        saw_temp,
    }
}

fn write_sunset(path: &Path, key: &str, value: &Value) -> Result<(), String> {
    let existing = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let mut state = sunset_state(&existing);
    match key {
        "nightlightNightOn" => {
            state.night_on = value
                .as_bool()
                .ok_or_else(|| "nightlightNightOn expects a bool".to_string())?;
        }
        "nightlightTemperature" => {
            state.temp = value
                .as_i64()
                .ok_or_else(|| "nightlightTemperature expects an int".to_string())?;
            state.saw_temp = true;
        }
        "nightlightDay" => {
            state.day = clock(
                value
                    .as_str()
                    .ok_or_else(|| "nightlightDay expects a time".to_string())?,
            )?;
            state.saw_day = true;
        }
        "nightlightNight" => {
            state.night = clock(
                value
                    .as_str()
                    .ok_or_else(|| "nightlightNight expects a time".to_string())?,
            )?;
            state.saw_night = true;
        }
        _ => return Err(format!("not a sunset field {key}")),
    }
    // Profiles only. The on/off switch is `omarchy toggle nightlight`.
    let mut lines = vec!["# Written by atmos. Day leaves the screen untinted.".to_string()];
    if state.saw_night {
        lines.push(format!("# atmos:night = {}", state.night));
    }
    if state.saw_temp {
        lines.push(format!("# atmos:temperature = {}", state.temp));
    }
    lines.extend([
        "profile {".to_string(),
        format!("    time = {}", state.day),
        "    identity = true".to_string(),
        "}".to_string(),
    ]);
    if state.night_on {
        lines.push(String::new());
        lines.push("profile {".to_string());
        lines.push(format!("    time = {}", state.night));
        lines.push(format!("    temperature = {}", state.temp));
        lines.push("}".to_string());
    }
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

fn clock(raw: &str) -> Result<String, String> {
    let Some((hour, minute)) = raw.trim().split_once(':') else {
        return Err(format!("{raw} is not HH:MM"));
    };
    let hour: u32 = hour.parse().map_err(|_| format!("{raw} is not HH:MM"))?;
    let minute: u32 = minute.parse().map_err(|_| format!("{raw} is not HH:MM"))?;
    if hour > 23 || minute > 59 {
        return Err(format!("{raw} is not HH:MM"));
    }
    Ok(format!("{hour:02}:{minute:02}"))
}

fn env_state(text: &str) -> (String, Vec<(String, String)>) {
    let mut inside = false;
    let mut prepend = String::new();
    let mut vars = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "# atmos:env begin" {
            inside = true;
            continue;
        }
        if trimmed == "# atmos:env end" {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("PATH=") {
            if let Some(path) = rest.strip_suffix(":$PATH") {
                prepend = path.to_string();
            }
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if !key.is_empty() && key != "PATH" {
                vars.push((key.to_string(), value.to_string()));
            }
        }
    }
    (prepend, vars)
}

fn write_env(path: &Path, key: &str, value: &Value) -> Result<(), String> {
    let existing = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let (mut prepend, mut vars) = env_state(&existing);
    match key {
        "envPathPrepend" => {
            prepend = value
                .as_str()
                .ok_or_else(|| "envPathPrepend expects a string".to_string())?
                .to_string();
        }
        "envVars" => {
            let items = value
                .as_array()
                .ok_or_else(|| "envVars expects a list".to_string())?;
            vars.clear();
            for item in items {
                let Some(obj) = item.as_object() else {
                    continue;
                };
                let Some(name) = obj.get("key").and_then(Value::as_str) else {
                    continue;
                };
                if name.is_empty() || name == "PATH" {
                    continue;
                }
                let stored = obj.get("value").and_then(Value::as_str).unwrap_or("");
                vars.push((name.to_string(), stored.to_string()));
            }
        }
        _ => return Err(format!("not an env field {key}")),
    }
    let mut lines = vec!["# atmos:env begin".to_string()];
    if !prepend.is_empty() {
        lines.push(format!("PATH={prepend}:$PATH"));
    }
    for (name, stored) in vars {
        lines.push(format!("{name}={stored}"));
    }
    lines.push("# atmos:env end".to_string());
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

fn mime_types(key: &str) -> &'static [&'static str] {
    match key {
        "mimePdf" => &["application/pdf"],
        "mimeImage" => &["image/png", "image/jpeg", "image/webp", "image/gif"],
        "mimeVideo" => &["video/mp4", "video/webm", "video/x-matroska"],
        _ => &[],
    }
}

fn mime_map(text: &str) -> Map<String, Value> {
    let mut inside = false;
    let mut map = Map::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[Default Applications]" {
            inside = true;
            continue;
        }
        if trimmed.starts_with('[') {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some((mime, desktop)) = trimmed.split_once('=') {
            map.insert(mime.to_string(), Value::String(desktop.to_string()));
        }
    }
    map
}

fn mime_desktop(text: &str, mime: &str) -> Value {
    mime_map(text)
        .get(mime)
        .cloned()
        .unwrap_or(Value::String(String::new()))
}

fn write_mime(path: &Path, key: &str, value: &Value) -> Result<(), String> {
    let desktop = value
        .as_str()
        .ok_or_else(|| format!("{key} expects a string"))?;
    let existing = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed == "[Default Applications]" {
            skipping = true;
            continue;
        }
        if skipping && trimmed.starts_with('[') {
            skipping = false;
        }
        if !skipping {
            kept.push(line.to_string());
        }
    }
    let mut map = mime_map(&existing);
    for mime in mime_types(key) {
        map.insert((*mime).to_string(), Value::String(desktop.to_string()));
    }
    if !kept.is_empty() && !kept.last().is_some_and(|line| line.is_empty()) {
        kept.push(String::new());
    }
    kept.push("[Default Applications]".to_string());
    let mut names: Vec<_> = map.keys().cloned().collect();
    names.sort();
    for name in names {
        if let Some(desktop) = map.get(&name).and_then(Value::as_str) {
            kept.push(format!("{name}={desktop}"));
        }
    }
    atomic_text(path, &format!("{}\n", kept.join("\n")))
}

fn audio_prefix(key: &str) -> &'static str {
    match key {
        "audioOutputVolume" => "output-volume ",
        "audioInputVolume" => "input-volume ",
        "audioTuningOn" => "tuning ",
        _ => "",
    }
}

fn audio_number(text: &str, prefix: &str) -> Result<Value, String> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix(prefix) {
            let number: i64 = rest
                .parse()
                .map_err(|err: std::num::ParseIntError| err.to_string())?;
            return Ok(Value::from(number));
        }
    }
    Ok(Value::Null)
}

fn audio_bool(text: &str, prefix: &str) -> Result<Value, String> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix(prefix) {
            return Ok(Value::Bool(rest == "true"));
        }
    }
    Ok(Value::Null)
}

fn write_audio_line(path: &Path, key: &str, value: &Value) -> Result<(), String> {
    let prefix = audio_prefix(key);
    let line = match key {
        "audioOutputVolume" | "audioInputVolume" => {
            let number = value
                .as_i64()
                .ok_or_else(|| format!("{key} expects an int"))?;
            format!("{prefix}{number}")
        }
        _ => {
            let flag = value
                .as_bool()
                .ok_or_else(|| format!("{key} expects a bool"))?;
            format!("{prefix}{flag}")
        }
    };
    let existing = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let mut found = false;
    let mut lines = Vec::new();
    for current in existing.lines() {
        if current.trim().starts_with(prefix) {
            lines.push(line.clone());
            found = true;
        } else if !current.trim().is_empty() {
            lines.push(current.to_string());
        }
    }
    if !found {
        lines.push(line);
    }
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

fn write_snapper(path: &Path, key: &str, value: &Value) -> Result<(), String> {
    let existing = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let mut number = snapper_number(&existing);
    let mut timeline = snapper_timeline(&existing);
    match key {
        "snapperNumberLimit" => {
            number = value
                .as_i64()
                .ok_or_else(|| "snapperNumberLimit expects an int".to_string())?;
        }
        "snapperTimeline" => {
            timeline = value
                .as_bool()
                .ok_or_else(|| "snapperTimeline expects a bool".to_string())?;
        }
        _ => return Err(format!("not a snapper field {key}")),
    }
    let want = if timeline { "yes" } else { "no" };
    let mut lines = Vec::new();
    let mut saw_number = false;
    let mut saw_timeline = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("NUMBER_LIMIT=") {
            lines.push(format!("NUMBER_LIMIT=\"{number}\""));
            saw_number = true;
        } else if trimmed.starts_with("TIMELINE_CREATE=") {
            lines.push(format!("TIMELINE_CREATE=\"{want}\""));
            saw_timeline = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !saw_number {
        lines.push(format!("NUMBER_LIMIT=\"{number}\""));
    }
    if !saw_timeline {
        lines.push(format!("TIMELINE_CREATE=\"{want}\""));
    }
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

fn snapper_number(text: &str) -> i64 {
    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches("NUMBER_LIMIT=").trim();
        if line.trim().starts_with("NUMBER_LIMIT=") {
            let bare = trimmed.trim_matches('"');
            if let Ok(number) = bare.parse() {
                return number;
            }
        }
    }
    5
}

fn snapper_timeline(text: &str) -> bool {
    for line in text.lines() {
        if line.trim().starts_with("TIMELINE_CREATE=") {
            return line.contains("yes");
        }
    }
    false
}

fn write_pam(path: &Path, key: &str, value: &Value) -> Result<(), String> {
    let on = value
        .as_bool()
        .ok_or_else(|| format!("{key} expects a bool"))?;
    let needle = if key == "fingerprintConfigured" {
        "pam_fprintd.so"
    } else {
        "pam_u2f.so"
    };
    let existing = if path.is_file() {
        fs::read_to_string(path).map_err(|err| err.to_string())?
    } else {
        String::new()
    };
    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();
    let present = lines.iter().any(|line| line.contains(needle));
    if on && !present {
        lines.push(format!("auth sufficient {needle}"));
    }
    if !on {
        lines.retain(|line| !line.contains(needle));
    }
    let body = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    atomic_text(path, &body)
}

fn audio_script(fixture: bool, action: &str, value: &Value) -> Result<(), String> {
    let number = value
        .as_i64()
        .ok_or_else(|| format!("{action} expects an int"))?
        .to_string();
    run_bash(fixture, "set-audio.sh", &[action, &number])
}

fn mime_script(kind: &str, value: &Value) -> Result<(), String> {
    let desktop = value.as_str().unwrap_or("");
    if desktop.is_empty() {
        return Ok(());
    }
    let _ = run_bash(false, "set-mime-default.sh", &[kind, desktop]);
    Ok(())
}

fn flag_comment(text: &str, name: &str) -> Value {
    let prefix = format!("-- atmos:{name} = ");
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix(&prefix) {
            return Value::Bool(rest == "true");
        }
    }
    Value::Null
}

fn sentinel_begin(kind: &str) -> &'static str {
    match kind {
        "bindings" => "-- atmos:bindings begin",
        "windows" => "-- atmos:windows begin",
        "workspaces" => "-- atmos:workspaces begin",
        "autostart" => "-- atmos:autostart begin",
        "monitors" => "-- atmos:monitors begin",
        _ => "",
    }
}

fn text_has_calls(text: &str, kind: &str) -> bool {
    match kind {
        "bindings" => text.contains("o.bind(") || text.contains("hl.unbind("),
        "windows" => text.contains("o.window("),
        "autostart" => text.contains("o.launch_on_start("),
        "workspaces" => text.contains("hl.workspace_rule("),
        "monitors" => text.contains("hl.monitor("),
        _ => false,
    }
}

fn keep_managed(value: &Value) -> Value {
    let Some(items) = value.as_array() else {
        return value.clone();
    };
    Value::Array(
        items
            .iter()
            .filter(|item| item.get("managed").and_then(Value::as_bool) != Some(false))
            .cloned()
            .collect(),
    )
}

fn hypr_apply(kind: &str, path: &Path, payload: &Value) -> Result<(), String> {
    let script = crate::scripts::repo_script("hypr-sentinel.py")?;
    let json = serde_json::to_string(payload).map_err(|err| err.to_string())?;
    let output = Command::new("python3")
        .arg(&script)
        .arg(kind)
        .arg("apply")
        .arg(path)
        .arg(&json)
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "hypr-sentinel.py {kind} exited {}: {}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn hypr_list(kind: &str, path: &Path) -> Result<Value, String> {
    let script = crate::scripts::repo_script("hypr-sentinel.py")?;
    let output = Command::new("python3")
        .arg(&script)
        .arg(kind)
        .arg("list")
        .arg(path)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "hypr-sentinel.py {kind} list exited {}: {}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

fn run_bash(skip_live: bool, name: &str, args: &[&str]) -> Result<(), String> {
    let script = crate::scripts::repo_script(name)?;
    let mut command = Command::new("bash");
    command.arg(&script).args(args);
    if skip_live {
        command.env("ATMOS_SKIP_LIVE", "1");
    }
    let output = command.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{name} exited {}: {}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn spawn(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn atomic_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(format!(".tmp-{}-{nanos}", std::process::id()));
    let tmp = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(tmp_name);
    fs::write(&tmp, text).map_err(|err| err.to_string())?;
    fs::rename(&tmp, path).map_err(|err| {
        let _ = fs::remove_file(&tmp);
        err.to_string()
    })
}
