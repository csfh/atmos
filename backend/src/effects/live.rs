use super::*;

pub fn after_write(root: Option<&Path>, key: &str, value: &Value) -> Result<()> {
    let fixture = root.is_some();
    match key {
        "audioOutputVolume" => audio_script(fixture, "output-volume", value),
        "audioInputVolume" => audio_script(fixture, "input-volume", value),
        "wifiRadio" => crate::tasks::net::radio(value.as_bool() == Some(true), fixture),
        _ if fixture => Ok(()),
        "workspaceBarNames" | "workspaceBarCount" => {
            crate::wsbar::after_write(root, key);
            Ok(())
        }
        "bluetooth" => {
            spawn(
                "omarchy",
                &["bluetooth", "power", crate::effect::on_off(value, false)],
            );
            Ok(())
        }
        "suspendEnabled" => {
            // The existing writer inverts this onto `omarchy toggle suspend-off`.
            spawn(
                "omarchy",
                &["toggle", "suspend-off", crate::effect::on_off(value, true)],
            );
            Ok(())
        }
        "crashCapture" => {
            // The tool only toggles, so it runs only when the flag differs.
            if crate::snapshot::crash_capture() != (value.as_bool() == Some(true)) {
                run("omarchy", &["toggle", "crash", "capture"])?;
            }
            Ok(())
        }
        // A local write of one small file: it runs to its end, so the
        // read-back that follows sees it and a failure reaches the reply.
        "weatherLocation" => match value.as_str().unwrap_or("") {
            "" => run("omarchy", &["weather", "location", "--clear"]),
            name => run("omarchy", &["weather", "location", "--set", name]),
        },
        "presentationMode" => {
            crate::tasks::user::presentation(crate::effect::on_off(value, false), None)
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
            crate::tasks::system::passwordless_sudo(crate::effect::on_off(value, false), "15")
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
            crate::tasks::system::snapper_policy("timeline", crate::effect::on_off(value, false))
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
        // Group-file keys. The fixture arm above has already returned.
        _ => {
            run_argv(key, value);
            Ok(())
        }
    }
}

fn run_argv(key: &str, value: &Value) {
    let Ok(rendered) = crate::scalar_arg(value) else {
        return;
    };
    let args: &[&str] = match key {
        "theme" => &["theme", "set"],
        "background" => &["theme", "bg", "set"],
        "barPosition" => &["bar", "position"],
        "barTransparent" => &["bar", "transparent"],
        "font" => &["font", "set"],
        "textSize" => &["display", "text", "size"],
        "browser" => &["default", "browser"],
        "terminal" => &["default", "terminal"],
        "editor" => &["default", "editor"],
        "agent" => &["default", "agent"],
        "dns" => &["dns"],
        "powerProfile" => &["powerprofiles", "set", "autodetect"],
        "powerProfileAc" => &["powerprofiles", "set", "ac"],
        "powerProfileBattery" => &["powerprofiles", "set", "battery"],
        "audioTuningOn" => {
            spawn(
                "omarchy",
                &["audio", "tuning", crate::effect::on_off(value, false)],
            );
            return;
        }
        _ => return,
    };
    let mut owned: Vec<&str> = args.to_vec();
    owned.push(&rendered);
    spawn("omarchy", &owned);
}

fn run(program: &str, args: &[&str]) -> Result<()> {
    Run::new(program)
        .args(args)
        .timeout(crate::runner::COMMAND_TIMEOUT)
        .checked()
        .map(|_| ())
}

pub(crate) fn spawn(program: &str, args: &[&str]) {
    // Fire and forget, but not silently: a command that cannot start is logged.
    if let Err(err) = Run::new(program).args(args).detach() {
        eprintln!("ratmos: {err}");
    }
}
