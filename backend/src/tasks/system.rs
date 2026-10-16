//! System-wide settings: hostname, time, locale, keyboard, accounts' names,
//! pacman, sudo, sshd, fstrim, snapper, charge limit, swappiness, DNS.

use super::elevate;
use crate::error::{Error, Result};
use crate::runner::Run;
use regex_lite::Regex;
use std::path::Path;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

fn host_label_ok(name: &str) -> bool {
    let label = r"[A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?";
    !name.is_empty()
        && !name.starts_with('-')
        && name.chars().count() <= 253
        && re(&format!(r"^{label}(\.{label})*$")).is_match(name)
}

pub fn hostname(name: &str) -> Result<()> {
    if !host_label_ok(name) {
        return Err(bad("Usage: set-hostname <hostname>"));
    }
    elevate::run_checked("hostnamectl", &["set-hostname", name], &[]).map(|_| ())
}

pub fn timezone(tz: &str) -> Result<()> {
    if tz.is_empty()
        || tz.starts_with('-')
        || tz.contains("..")
        || !re(r"^[A-Za-z0-9/_+-]+$").is_match(tz)
    {
        return Err(bad("Usage: set-timezone <Area/City>"));
    }
    let zone = Path::new("/usr/share/zoneinfo").join(tz);
    if !zone.exists() || zone.is_dir() {
        return Err(bad(format!("set-timezone: unknown timezone: {tz}")));
    }
    elevate::run_checked("timedatectl", &["set-timezone", tz], &[]).map(|_| ())
}

pub fn locale_ok(locale: &str) -> bool {
    locale == "C.UTF-8" || re(r"^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$").is_match(locale)
}

pub fn locale(locale: &str) -> Result<()> {
    if locale.is_empty() || locale.starts_with('-') || !locale_ok(locale) {
        return Err(bad("Usage: set-locale <xx_YY.UTF-8>"));
    }
    if let Ok(supported) = std::fs::read_to_string("/usr/share/i18n/SUPPORTED") {
        let known = supported.lines().any(|l| {
            let mut cols = l.split_whitespace();
            cols.next() == Some(locale) && cols.next() == Some("UTF-8")
        });
        if !known {
            return Err(bad(format!("set-locale: unknown locale: {locale}")));
        }
    }
    let installed = Run::new("localectl")
        .arg("list-locales")
        .output()
        .map(|o| o.stdout_text().lines().any(|l| l == locale))
        .unwrap_or(false);
    if installed {
        return elevate::run_checked("localectl", &["set-locale", &format!("LANG={locale}")], &[])
            .map(|_| ());
    }
    elevate::root_task("locale-gen", &[locale], &[]).map(|_| ())
}

pub fn keyboard_layout(layout: &str) -> Result<()> {
    if layout.is_empty() || layout.starts_with('-') || !re(r"^[a-z0-9]{1,8}$").is_match(layout) {
        return Err(bad("Usage: set-keyboard-layout <layout>"));
    }
    if !Path::new("/usr/share/X11/xkb/symbols")
        .join(layout)
        .is_file()
    {
        return Err(bad(format!(
            "set-keyboard-layout: unknown layout: {layout}"
        )));
    }
    elevate::run_checked("localectl", &["set-x11-keymap", layout], &[])?;
    let restarted = Run::new("omarchy")
        .args(["restart", "hyprctl"])
        .output()
        .is_ok_and(|o| o.success());
    if !restarted {
        let _ = Run::new("hyprctl").arg("reload").output();
    }
    Ok(())
}

pub fn ntp(on: &str) -> Result<()> {
    if on != "true" && on != "false" {
        return Err(bad("Usage: set-ntp true|false"));
    }
    elevate::run_checked("timedatectl", &["set-ntp", on], &[]).map(|_| ())
}

fn user_name() -> String {
    Run::new("id")
        .arg("-un")
        .output()
        .map(|o| o.stdout_text().trim().to_string())
        .unwrap_or_default()
}

pub fn full_name(name: &str) -> Result<()> {
    let user = user_name();
    if name.starts_with('-')
        || name.chars().count() > 256
        || name.contains('\n')
        || name.contains('\r')
        || name.contains(':')
        || name.contains(',')
        || !re(r"^[a-z_][a-z0-9_-]*$").is_match(&user)
    {
        return Err(bad("Usage: set-full-name <full-name>"));
    }
    elevate::run_checked("chfn", &["-f", name, &user], &[]).map(|_| ())
}

pub fn parallel_downloads(n: &str) -> Result<()> {
    let ok =
        re(r"^[1-9][0-9]?$").is_match(n) && n.parse::<u32>().is_ok_and(|v| (1..=20).contains(&v));
    if !ok {
        return Err(bad("Usage: set-parallel-downloads <1-20>"));
    }
    elevate::root_task("parallel-downloads", &[n], &[]).map(|_| ())
}

pub fn sshd_disable() -> Result<()> {
    elevate::root_task("sshd-disable", &[], &[]).map(|_| ())
}

pub fn fstrim(value: &str) -> Result<()> {
    if value != "on" && value != "off" {
        return Err(bad("Usage: set-fstrim <on|off>"));
    }
    elevate::root_task("fstrim", &[value], &[]).map(|_| ())
}

pub fn snapper_policy(action: &str, value: &str) -> Result<()> {
    let usage = || bad("Usage: set-snapper-policy number-limit <1-50> | timeline <on|off>");
    if std::fs::read_to_string("/etc/snapper/configs/root").is_err() {
        return Err(bad(
            "set-snapper-policy: /etc/snapper/configs/root is not readable",
        ));
    }
    match action {
        "number-limit" => {
            let ok = re(r"^[1-9][0-9]?$").is_match(value)
                && value.parse::<u32>().is_ok_and(|v| (1..=50).contains(&v));
            if !ok {
                return Err(usage());
            }
            elevate::root_task("snapper-number-limit", &[value], &[]).map(|_| ())
        }
        "timeline" => {
            let want = match value {
                "on" => "yes",
                "off" => "no",
                _ => return Err(usage()),
            };
            elevate::root_task("snapper-timeline", &[want], &[]).map(|_| ())
        }
        _ => Err(usage()),
    }
}

pub fn passwordless_sudo(action: &str, minutes: &str) -> Result<()> {
    let user = user_name();
    let usage = || bad("Usage: set-passwordless-sudo on <minutes> | off");
    if !re(r"^[a-z_][a-z0-9_-]*$").is_match(&user) {
        return Err(usage());
    }
    match action {
        "on" => {
            let ok = re(r"^[1-9][0-9]*$").is_match(minutes)
                && minutes.parse::<u32>().is_ok_and(|m| (1..=240).contains(&m));
            if !ok {
                return Err(usage());
            }
            elevate::root_task("sudo-on", &[&user, minutes], &[]).map(|_| ())
        }
        "off" => elevate::root_task("sudo-off", &[&user], &[]).map(|_| ()),
        _ => Err(usage()),
    }
}

/// Where the charge threshold lives, and the mode file when Custom must be selected first.
fn charge_paths() -> Option<(String, Option<String>)> {
    let dir = std::env::var("ATMOS_POWER_SUPPLY_DIR")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/sys/class/power_supply".into());
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    let found = entries
        .into_iter()
        .map(|p| p.join("charge_control_end_threshold"))
        .find(|p| p.exists())?;
    // Dell exposes thresholds even in Adaptive mode, where they do not apply.
    // The kernel marks the active type in [brackets], so " Custom " means
    // offered but not selected. [Custom] and hardware without Custom stay.
    let types = found.with_file_name("charge_types");
    let mode = std::fs::read_to_string(&types)
        .ok()
        .filter(|t| format!(" {} ", t.trim_end_matches('\n')).contains(" Custom "))
        .map(|_| types.to_string_lossy().into_owned());
    Some((found.to_string_lossy().into_owned(), mode))
}

pub fn charge_limit(limit: &str) -> Result<()> {
    if !limit.chars().all(|c| c.is_ascii_digit()) || limit.is_empty() {
        return Err(Error::bad_request("usage: set-charge-limit <50-100>"));
    }
    if !limit.parse::<u32>().is_ok_and(|n| (50..=100).contains(&n)) {
        return Err(Error::bad_request("set-charge-limit: limit must be 50-100"));
    }
    let Some((found, mode)) = charge_paths() else {
        return Err(bad(
            "set-charge-limit: no charge_control_end_threshold on this machine",
        ));
    };
    let writable = |path: &str| std::fs::OpenOptions::new().write(true).open(path).is_ok();
    if writable(&found) && mode.as_deref().is_none_or(writable) {
        if let Some(mode) = &mode {
            std::fs::write(mode, "Custom\n")?;
        }
        std::fs::write(&found, format!("{limit}\n"))?;
        return Ok(());
    }
    // One elevation for both writes, so polkit asks once. Custom goes first
    // and a rejected mode write leaves the threshold alone.
    elevate::root_task(
        "charge-limit",
        &[mode.as_deref().unwrap_or(""), limit, &found],
        &[],
    )
    .map(|_| ())
}

fn sysctl_file() -> String {
    std::env::var("ATMOS_SYSCTL_FILE")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/etc/sysctl.d/99-atmos-swappiness.conf".into())
}

fn dir_writable(dest: &str) -> bool {
    let parent = Path::new(dest).parent().unwrap_or(Path::new("."));
    let probe = parent.join(format!(".atmos-probe-{}", std::process::id()));
    let ok = std::fs::File::create(&probe).is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

/// Swappiness writes /etc/sysctl.d. Atmos's sudo flow only unlocks
/// passwordless sudo, then re-runs the job unprivileged, so the write itself
/// has to elevate.
pub fn swappiness(mode: &str) -> Result<()> {
    swappiness_at(&sysctl_file(), mode)
}

fn swappiness_at(dest: &str, mode: &str) -> Result<()> {
    let dest = dest.to_string();
    if mode == "off" {
        if dir_writable(&dest) {
            let _ = std::fs::remove_file(&dest);
            return Ok(());
        }
        return elevate::run_checked("rm", &["-f", &dest], &[]).map(|_| ());
    }
    let body = "vm.swappiness = 10\n";
    if dir_writable(&dest) {
        return crate::fsutil::atomic_write_text(Path::new(&dest), body);
    }
    elevate::run_checked("tee", &[&dest], body.as_bytes()).map(|_| ())
}

fn tweak_path(var: &str, tail: &str) -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    crate::host::env_path(var, std::path::PathBuf::from(home).join(tail))
}

/// Drop every line that starts with `prefix`, then add `extra` when given. A
/// missing file is seeded with an Atmos env block holding just `extra`.
fn env_overlay(dest: &Path, prefix: &str, extra: Option<&str>) -> Result<()> {
    crate::store::with_lock(dest, || {
        let text = std::fs::read_to_string(dest).ok();
        let next = match (text, extra) {
            (None, Some(line)) => format!("# atmos:env begin\n{line}\n# atmos:env end\n"),
            (None, None) => return Ok(()),
            (Some(text), extra) => {
                let mut lines: Vec<&str> =
                    text.lines().filter(|l| !l.starts_with(prefix)).collect();
                if let Some(line) = extra {
                    lines.push(line);
                }
                format!("{}\n", lines.join("\n"))
            }
        };
        crate::fsutil::atomic_write_text(dest, &next)
    })
}

/// Apply or reset one of the file-backed tweaks. `on` leaves the overlay in
/// place for the GTK paste flag and takes it away for the two env opt-outs.
pub fn tweak(action: &str, mode: &str) -> Result<()> {
    let off = mode == "off";
    match action {
        "gtk-middle-paste" => {
            let dest = tweak_path("ATMOS_GTK4_FILE", ".config/gtk-4.0/settings.ini");
            if off {
                let _ = std::fs::remove_file(&dest);
                return Ok(());
            }
            crate::store::with_lock(&dest, || {
                crate::fsutil::atomic_write_text(&dest, "[Settings]\ngtk-enable-primary-paste=false\n")
            })
        }
        "electron-wayland" | "force-zero-scaling" => {
            let dest = tweak_path("ATMOS_ENV_FILE", ".config/environment.d/10-atmos.conf");
            let line = if action == "electron-wayland" {
                "ELECTRON_OZONE_PLATFORM_HINT=auto"
            } else {
                "ATMOS_XWAYLAND_ZERO_SCALING=0"
            };
            let prefix = format!("{}=", line.split('=').next().unwrap_or(""));
            env_overlay(&dest, &prefix, off.then_some(line))
        }
        "swappiness" => swappiness(mode),
        _ => Err(Error::bad_request("usage: set-tweaks gtk-middle-paste|electron-wayland|force-zero-scaling|swappiness on|off")),
    }
}

/// Apply the file-backed tweak a `tweaks.*` setting key stands for, if any.
pub fn tweak_setting(key: &str, value: &serde_json::Value) {
    let action = match key {
        "tweaks.middlePaste" => "gtk-middle-paste",
        "tweaks.electronWayland" => "electron-wayland",
        "tweaks.forceZeroScaling" => "force-zero-scaling",
        "tweaks.swappiness" => "swappiness",
        _ => return,
    };
    let _ = tweak(
        action,
        if value.as_bool() == Some(true) {
            "on"
        } else {
            "off"
        },
    );
}

/// Apply custom DNS servers through `omarchy dns Custom` (stdin).
pub fn dns_custom(servers: &[&str]) -> Result<()> {
    let joined = servers.join(" ");
    let usage = || bad("Usage: set-dns-custom <ipv4-or-ipv6> [more...]");
    if joined.trim().is_empty() {
        return Err(usage());
    }
    if joined.starts_with('-') {
        return Err(bad("set-dns-custom: DNS servers cannot start with -"));
    }
    let normalized: Vec<&str> = joined
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|t| !t.is_empty())
        .collect();
    if normalized.is_empty() {
        return Err(usage());
    }
    for token in &normalized {
        if token.contains('.') {
            if !re(r"^([0-9]{1,3}\.){3}[0-9]{1,3}$").is_match(token) {
                return Err(bad(format!("set-dns-custom: not an IPv4 address: {token}")));
            }
        } else if token.contains(':') {
            if !re(r"^[0-9a-fA-F:]+$").is_match(token) {
                return Err(bad(format!("set-dns-custom: not an IPv6 address: {token}")));
            }
        } else {
            return Err(bad(format!("set-dns-custom: not a DNS address: {token}")));
        }
    }
    let out = Run::new("omarchy")
        .args(["dns", "Custom"])
        .input(format!("{}\n", normalized.join(" ")))
        .output()?;
    if out.success() {
        Ok(())
    } else {
        Err(bad(out.stderr_text().trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostnames_follow_the_dns_label_rules() {
        assert!(host_label_ok("omarchy"));
        assert!(host_label_ok("a.b-c.example"));
        assert!(!host_label_ok("-bad"));
        assert!(!host_label_ok("bad-"));
        assert!(!host_label_ok(&"a".repeat(64)));
        assert!(!host_label_ok(""));
    }

    #[test]
    fn locales_are_the_utf8_forms_only() {
        assert!(locale_ok("en_US.UTF-8"));
        assert!(locale_ok("C.UTF-8"));
        assert!(locale_ok("sr_RS.UTF-8@latin"));
        assert!(!locale_ok("en_US.ISO-8859-1"));
        assert!(!locale_ok("../etc"));
    }

    #[test]
    fn swappiness_writes_and_removes_the_drop_in_without_elevating_when_it_can() {
        let dir = std::env::temp_dir().join(format!("ratmos-sysctl-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("99-atmos-swappiness.conf");
        swappiness_at(&file.to_string_lossy(), "on").unwrap();
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "vm.swappiness = 10\n"
        );
        swappiness_at(&file.to_string_lossy(), "off").unwrap();
        assert!(!file.exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn charge_limits_and_dns_are_checked_before_anything_runs() {
        assert!(charge_limit("49").is_err());
        assert!(charge_limit("abc").is_err());
        assert!(dns_custom(&["1.1.1.1", "bad"]).is_err());
        assert!(dns_custom(&["-x"]).is_err());
        assert!(dns_custom(&[""]).is_err());
    }
}
