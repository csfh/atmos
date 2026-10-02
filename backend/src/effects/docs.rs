use super::*;

pub fn read_doc(root: Option<&Path>, path: &Path, key: &str) -> Result<Value> {
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
            let text = fs::read_to_string(&check)?;
            Ok(Value::Bool(text.contains("NOPASSWD")))
        }
        "fingerprintConfigured" | "fido2Configured" => {
            if !path.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(path)?;
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
            let text = fs::read_to_string(path)?;
            Ok(Value::Bool(text.contains("enable sshd.service")))
        }
        "fstrimEnabled" => {
            if root.is_none() {
                return systemctl_enabled("fstrim.timer");
            }
            if !path.is_file() {
                return Ok(Value::Bool(false));
            }
            let text = fs::read_to_string(path)?;
            Ok(Value::Bool(text.trim() == "enabled"))
        }
        "directBoot" => {
            if !path.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(path)?;
            Ok(Value::Bool(text.contains("Omarchy")))
        }
        "sudolessDocker" => {
            if !path.is_file() {
                return Ok(missing_bool(root));
            }
            let text = fs::read_to_string(path)?;
            Ok(Value::Bool(text.contains("docker")))
        }
        _ => {
            if !path.is_file() {
                return Ok(Value::Null);
            }
            let text = fs::read_to_string(path)?;
            read_doc_text(key, &text)
        }
    }
}

pub fn write_doc(root: Option<&Path>, path: &Path, key: &str, value: &Value) -> Result<()> {
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
                None => return Err(format!("{key} expects a bool").into()),
            };
            atomic_text(path, &format!("{token}\n"))
        }
        "presentationMode" => {
            let on = value
                .as_bool()
                .ok_or_else(|| format!("{key} expects a bool"))?;
            let body = serde_json::json!({ "on": on, "until": 0, "minutes": 0 });
            let text = serde_json::to_string(&body)?;
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
                fs::remove_file(path).map_err(Error::from)
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
        other => Err(format!("no platform document for {other}").into()),
    }
}

pub(super) fn privileged(key: &str) -> bool {
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

pub(super) fn missing_bool(root: Option<&Path>) -> Value {
    if root.is_some() {
        Value::Bool(false)
    } else {
        Value::Null
    }
}

pub(super) fn passwordless_live_path() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_default();
    PathBuf::from(format!("/etc/sudoers.d/99-omarchy-nopasswd-{user}"))
}

pub(super) fn systemctl_enabled(unit: &str) -> Result<Value> {
    let output = Run::new("systemctl")
        .args(["is-enabled", unit])
        .timeout(Duration::from_secs(10))
        .output();
    let Ok(output) = output else {
        return Ok(Value::Null);
    };
    let text = output.stdout_text();
    if text.trim() == "enabled" {
        Ok(Value::Bool(true))
    } else if text.trim().is_empty() {
        Ok(Value::Null)
    } else {
        Ok(Value::Bool(false))
    }
}

pub(super) fn read_doc_text(key: &str, text: &str) -> Result<Value> {
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
            let parsed: Value = serde_json::from_str(text)?;
            Ok(Value::Bool(parsed["on"].as_bool().unwrap_or(false)))
        }
        "chargeLimit" => text
            .trim()
            .parse::<i64>()
            .map(Value::from)
            .map_err(|err| Error::bad_request(err.to_string())),
        "snapperNumberLimit" => Ok(Value::from(snapper_number(text))),
        "snapperTimeline" => Ok(Value::Bool(snapper_timeline(text))),
        other => Err(format!("cannot read {other}").into()),
    }
}
