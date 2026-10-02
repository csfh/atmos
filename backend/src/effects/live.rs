use super::*;

pub fn after_write(root: Option<&Path>, key: &str, value: &Value) -> Result<()> {
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

pub(super) fn run_bash(skip_live: bool, name: &str, args: &[&str]) -> Result<()> {
    let script = crate::scripts::repo_script(name)?;
    let mut run = Run::new("bash").arg(&script).args(args);
    if skip_live {
        run = run.env("ATMOS_SKIP_LIVE", "1");
    }
    run.timeout(COMMAND_TIMEOUT)
        .checked()
        .map(|_| ())
        .map_err(|err| err.with_context(name))
}

pub(super) fn spawn(program: &str, args: &[&str]) {
    // Fire and forget, but not silently: a command that cannot start is logged.
    if let Err(err) = Run::new(program).args(args).detach() {
        eprintln!("ratmos: {err}");
    }
}
