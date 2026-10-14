//! NetworkManager: the Wi-Fi radio, saved connections, hotspot, 802.1X.

use crate::error::{Error, Result};
use crate::runner::Run;
use regex_lite::Regex;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

fn uuid_ok(text: &str) -> bool {
    re(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .is_match(text)
}

fn ipv4_ok(text: &str) -> bool {
    re(r"^([0-9]{1,3}\.){3}[0-9]{1,3}$").is_match(text)
}

fn nmcli(args: &[&str], stdin: Option<String>) -> Result<String> {
    let mut run = Run::new("nmcli").args(args);
    if let Some(input) = stdin {
        run = run.input(input);
    }
    let out = run.output()?;
    if out.success() {
        Ok(out.stdout_text())
    } else {
        let text = out.stderr_text();
        let text = if text.trim().is_empty() {
            out.stdout_text()
        } else {
            text
        };
        Err(bad(text.trim().to_string()))
    }
}

fn say(text: String) {
    print!("{text}");
}

pub fn connection(args: &[String], stdin: &[u8]) -> Result<()> {
    // Fixture runs set this so the writer still runs without touching the live radio.
    if std::env::var("ATMOS_SKIP_LIVE").is_ok_and(|v| v == "1") {
        return Ok(());
    }
    connection_live(args, stdin)
}

/// The radio switch the settings path flips; fixture runs leave NetworkManager alone.
pub fn radio(on: bool, skip_live: bool) -> Result<()> {
    if skip_live {
        return Ok(());
    }
    connection_live(
        &[
            "radio".to_string(),
            if on { "on" } else { "off" }.to_string(),
        ],
        &[],
    )
}

fn connection_live(args: &[String], stdin: &[u8]) -> Result<()> {
    let usage = || {
        bad("Usage: set-wifi-connection radio <on|off> | up <uuid> | down <uuid> | delete <uuid> | join <ssid> | down-ssid <ssid> | delete-ssid <ssid> | metered <uuid> <yes|no|unknown> | priority <uuid> <n> | mac <uuid> <mode> | ipv4 <uuid> auto | ipv4 <uuid> manual <addr> <prefix> [gateway] [dns] | wireguard-import <file> | hotspot on <ssid> <password> | hotspot off")
    };
    let arg = |i: usize| args.get(i).map_or("", String::as_str);
    let (action, target) = (arg(0), arg(1));
    if !super::present("nmcli") {
        return Err(bad("set-wifi-connection: nmcli is not installed"));
    }
    let name_ok = |s: &str| !s.is_empty() && !s.starts_with('-');
    match action {
        "radio" => {
            if target != "on" && target != "off" {
                return Err(usage());
            }
            nmcli(&["radio", "wifi", target], None).map(say)
        }
        "up" | "down" => {
            if !uuid_ok(target) {
                return Err(usage());
            }
            nmcli(&["connection", action, "uuid", target], None).map(say)
        }
        "delete" => {
            if !uuid_ok(target) {
                return Err(usage());
            }
            nmcli(&["connection", "delete", "uuid", target], None).map(say)
        }
        "join" => {
            if !name_ok(target) {
                return Err(usage());
            }
            let text = String::from_utf8_lossy(stdin).into_owned();
            let pw = text.split('\n').next().unwrap_or("").to_string();
            if pw.is_empty() {
                nmcli(&["--wait", "15", "device", "wifi", "connect", target], None).map(say)
            } else {
                nmcli(
                    &[
                        "--wait", "15", "device", "wifi", "connect", target, "password", &pw,
                    ],
                    None,
                )
                .map(say)
            }
        }
        "down-ssid" => {
            if !name_ok(target) {
                return Err(usage());
            }
            nmcli(&["connection", "down", "id", target], None).map(say)
        }
        "delete-ssid" => {
            if !name_ok(target) {
                return Err(usage());
            }
            nmcli(&["connection", "delete", "id", target], None).map(say)
        }
        "metered" => {
            let value = arg(2);
            if !uuid_ok(target) || !["yes", "no", "unknown"].contains(&value) {
                return Err(usage());
            }
            nmcli(
                &[
                    "connection",
                    "modify",
                    "uuid",
                    target,
                    "connection.metered",
                    value,
                ],
                None,
            )
            .map(say)
        }
        "priority" => {
            let value = arg(2);
            if !uuid_ok(target)
                || !re(r"^-?[0-9]+$").is_match(value)
                || !value
                    .parse::<i32>()
                    .is_ok_and(|n| (-999..=999).contains(&n))
            {
                return Err(usage());
            }
            nmcli(
                &[
                    "connection",
                    "modify",
                    "uuid",
                    target,
                    "connection.autoconnect-priority",
                    value,
                ],
                None,
            )
            .map(say)
        }
        "mac" => {
            let value = arg(2);
            if !uuid_ok(target)
                || !["default", "random", "stable", "permanent", "preserve"].contains(&value)
            {
                return Err(usage());
            }
            nmcli(
                &[
                    "connection",
                    "modify",
                    "uuid",
                    target,
                    "wifi.cloned-mac-address",
                    value,
                ],
                None,
            )
            .map(say)
        }
        "ipv4" => {
            if !uuid_ok(target) {
                return Err(usage());
            }
            match arg(2) {
                "auto" => nmcli(
                    &[
                        "connection",
                        "modify",
                        "uuid",
                        target,
                        "ipv4.method",
                        "auto",
                        "ipv4.addresses",
                        "",
                        "ipv4.gateway",
                        "",
                    ],
                    None,
                )
                .map(say),
                "manual" => {
                    let (addr, prefix, gateway, dns) = (arg(3), arg(4), arg(5), arg(6));
                    if !ipv4_ok(addr)
                        || !re(r"^[0-9]+$").is_match(prefix)
                        || !prefix.parse::<u32>().is_ok_and(|p| (1..=32).contains(&p))
                    {
                        return Err(usage());
                    }
                    nmcli(
                        &[
                            "connection",
                            "modify",
                            "uuid",
                            target,
                            "ipv4.method",
                            "manual",
                            "ipv4.addresses",
                            &format!("{addr}/{prefix}"),
                        ],
                        None,
                    )?;
                    if !gateway.is_empty() {
                        if !ipv4_ok(gateway) {
                            return Err(usage());
                        }
                        nmcli(
                            &[
                                "connection",
                                "modify",
                                "uuid",
                                target,
                                "ipv4.gateway",
                                gateway,
                            ],
                            None,
                        )?;
                    }
                    if !dns.is_empty() {
                        nmcli(
                            &["connection", "modify", "uuid", target, "ipv4.dns", dns],
                            None,
                        )?;
                    }
                    Ok(())
                }
                _ => Err(usage()),
            }
        }
        "wireguard-import" => {
            let ok = target.starts_with('/')
                && !target.contains("..")
                && (target.ends_with(".conf") || target.ends_with(".nmconnection"));
            if !ok {
                return Err(usage());
            }
            if !std::path::Path::new(target).is_file() {
                return Err(bad(format!("set-wifi-connection: missing {target}")));
            }
            nmcli(
                &["connection", "import", "type", "wireguard", "file", target],
                None,
            )
            .map(say)
        }
        "hotspot" => {
            if target != "on" && target != "off" {
                return Err(usage());
            }
            if target == "off" {
                let _ = nmcli(&["connection", "down", "id", "Hotspot"], None);
                return Ok(());
            }
            let (ssid, password) = (arg(2), arg(3));
            if ssid.is_empty() || ssid.contains('\n') || ssid.chars().count() > 32 {
                return Err(usage());
            }
            if !(8..=63).contains(&password.chars().count()) || password.contains('\n') {
                return Err(usage());
            }
            nmcli(
                &[
                    "device", "wifi", "hotspot", "ssid", ssid, "password", password,
                ],
                None,
            )
            .map(say)
        }
        _ => Err(usage()),
    }
}

/// Create an 802.1X Wi-Fi profile. SSID and identity are argv; the password is
/// one line on stdin so it never appears in /proc.
pub fn enterprise_connect(args: &[String], stdin: &[u8]) -> Result<()> {
    let (ssid, identity) = (
        args.first().map_or("", String::as_str),
        args.get(1).map_or("", String::as_str),
    );
    if ssid.is_empty() || identity.is_empty() {
        return Err(bad("Usage: enterprise-wifi-connect <ssid> <identity>"));
    }
    let text = String::from_utf8_lossy(stdin).into_owned();
    let pw = text.split('\n').next().unwrap_or("").to_string();
    if pw.is_empty() {
        return Err(bad("enterprise-wifi-connect: password cannot be empty"));
    }
    let uuid = std::fs::read_to_string("/proc/sys/kernel/random/uuid")
        .map(|u| u.trim().to_string())
        .unwrap_or_default();
    if !uuid_ok(&uuid) {
        return Err(bad(
            "enterprise-wifi-connect: could not make a connection id",
        ));
    }
    nmcli(
        &[
            "connection",
            "add",
            "type",
            "wifi",
            "con-name",
            ssid,
            "ssid",
            ssid,
            "connection.uuid",
            &uuid,
            "wifi-sec.key-mgmt",
            "wpa-eap",
            "802-1x.eap",
            "peap",
            "802-1x.phase2-auth",
            "mschapv2",
            "802-1x.identity",
            identity,
            "802-1x.auth-timeout",
            "8",
        ],
        None,
    )?;
    let script = format!("set 802-1x.password {pw}\nsave\nquit\n");
    let cleanup = || {
        let _ = nmcli(&["connection", "delete", "uuid", &uuid], None);
    };
    if let Err(err) = nmcli(&["connection", "edit", "uuid", &uuid], Some(script)) {
        cleanup();
        return Err(err);
    }
    match nmcli(&["connection", "up", "uuid", &uuid], None) {
        Ok(out) => {
            say(out);
            Ok(())
        }
        Err(err) => {
            cleanup();
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_are_checked_before_nmcli_runs() {
        assert!(uuid_ok("123e4567-e89b-12d3-a456-426614174000"));
        assert!(!uuid_ok("not-a-uuid"));
        assert!(ipv4_ok("192.168.1.10"));
        assert!(!ipv4_ok("1.2.3"));
        std::env::remove_var("ATMOS_SKIP_LIVE");
    }
}
