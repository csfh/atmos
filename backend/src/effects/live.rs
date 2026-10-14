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
            let _ = flag;
            crate::tasks::net::radio(value.as_bool() == Some(true), fixture)
        }
        _ if fixture => Ok(()),
        "workspaceBarNames" | "workspaceBarCount" => {
            crate::wsbar::after_write(root, key);
            Ok(())
        }
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
            crate::tasks::user::presentation(flag, None)
        }
        "chargeLimit" => {
            let number = value
                .as_i64()
                .ok_or_else(|| "chargeLimit expects an int".to_string())?
                .to_string();
            crate::tasks::system::charge_limit(&number)
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
            crate::tasks::system::passwordless_sudo(action, "15")
        }
        "sshdEnabled" => {
            if value.as_bool() == Some(false) {
                crate::tasks::system::sshd_disable()
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
                crate::tasks::system::snapper_policy("number-limit", &rendered)
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
            crate::tasks::system::snapper_policy("timeline", flag)
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
                crate::tasks::system::dns_custom(&[text])
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

pub(super) fn spawn(program: &str, args: &[&str]) {
    // Fire and forget, but not silently: a command that cannot start is logged.
    if let Err(err) = Run::new(program).args(args).detach() {
        eprintln!("ratmos: {err}");
    }
}
