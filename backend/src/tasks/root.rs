//! `atmosd root <task>`: the privileged half of a few settings. It runs only
//! when polkit or sudo started this binary as root, and every task checks its
//! own arguments again, because a caller cannot be trusted past that door.

use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text;
use crate::runner::Run;
use regex_lite::Regex;
use std::path::Path;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

fn is_root() -> bool {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self").is_ok_and(|m| m.uid() == 0)
}

fn must(program: &str, args: &[&str]) -> Result<()> {
    let out = Run::new(program).args(args).output()?;
    if out.success() {
        Ok(())
    } else {
        let text = out.stderr_text();
        Err(bad(if text.trim().is_empty() {
            format!("{program} failed ({})", out.code)
        } else {
            text.trim().to_string()
        }))
    }
}

fn quiet(program: &str, args: &[&str]) {
    let _ = Run::new(program).args(args).output();
}

pub fn run(task: &str, args: &[String], stdin: &[u8]) -> Result<()> {
    if !is_root() && std::env::var("ATMOS_ROOT_TEST").is_err() {
        return Err(bad("root: this task runs as root"));
    }
    let arg = |i: usize| args.get(i).map_or("", String::as_str);
    match task {
        "locale-gen" => locale_gen(arg(0)),
        "parallel-downloads" => parallel_downloads(arg(0)),
        "sshd-disable" => {
            quiet("systemctl", &["disable", "--now", "sshd.service"]);
            if Run::new("ufw").arg("version").output().is_ok() {
                quiet("ufw", &["--force", "delete", "limit", "22/tcp"]);
                quiet("ufw", &["reload"]);
            }
            Ok(())
        }
        "fstrim" => match arg(0) {
            "on" => must("systemctl", &["enable", "--now", "fstrim.timer"]),
            "off" => must("systemctl", &["disable", "--now", "fstrim.timer"]),
            _ => Err(bad("root fstrim: on or off")),
        },
        "snapper-number-limit" => snapper_set("NUMBER_LIMIT", arg(0), |v| {
            re(r"^[1-9][0-9]?$").is_match(v)
                && v.parse::<u32>().is_ok_and(|n| (1..=50).contains(&n))
        }),
        "snapper-timeline" => {
            snapper_set("TIMELINE_CREATE", arg(0), |v| v == "yes" || v == "no")?;
            if arg(0) == "yes" {
                must("systemctl", &["enable", "--now", "snapper-timeline.timer"])?;
                must("systemctl", &["enable", "--now", "snapper-cleanup.timer"])
            } else {
                quiet("systemctl", &["disable", "--now", "snapper-timeline.timer"]);
                Ok(())
            }
        }
        "sudo-on" => sudo_on(arg(0), arg(1)),
        "sudo-off" => sudo_off(arg(0)),
        "charge-limit" => charge_limit(arg(0), arg(1), arg(2)),
        other => super::accounts::root_task(other, args, stdin),
    }
}

fn locale_gen(locale: &str) -> Result<()> {
    if !super::system::locale_ok(locale) {
        return Err(bad("root locale-gen: not a UTF-8 locale"));
    }
    let gen = Path::new("/etc/locale.gen");
    if locale != "C.UTF-8" && gen.is_file() {
        let text = std::fs::read_to_string(gen)?;
        if !text
            .lines()
            .any(|l| l.split_whitespace().next() == Some(locale))
        {
            let mut next = text;
            if !next.is_empty() && !next.ends_with('\n') {
                next.push('\n');
            }
            next.push_str(&format!("{locale} UTF-8\n"));
            atomic_write_text(gen, &next)?;
        }
        must("locale-gen", &[])?;
    }
    must("localectl", &["set-locale", &format!("LANG={locale}")])
}

/// Replace every `KEY=` line in a text, or append one when none exists.
fn set_key(text: &str, key_pattern: &Regex, line: &str, append: &str) -> String {
    if text.lines().any(|l| key_pattern.is_match(l)) {
        let mut out: Vec<String> = text
            .lines()
            .map(|l| {
                if key_pattern.is_match(l) {
                    line.to_string()
                } else {
                    l.to_string()
                }
            })
            .collect();
        if text.ends_with('\n') {
            out.push(String::new());
        }
        out.join("\n")
    } else {
        format!("{text}{append}")
    }
}

fn parallel_downloads(n: &str) -> Result<()> {
    if !(re(r"^[1-9][0-9]?$").is_match(n) && n.parse::<u32>().is_ok_and(|v| (1..=20).contains(&v)))
    {
        return Err(bad("root parallel-downloads: 1-20"));
    }
    let conf = Path::new("/etc/pacman.conf");
    let text = std::fs::read_to_string(conf)?;
    let next = set_key(
        &text,
        &re(r"^\s*#?\s*ParallelDownloads\s*="),
        &format!("ParallelDownloads = {n}"),
        &format!("\nParallelDownloads = {n}\n"),
    );
    atomic_write_text(conf, &next)
}

fn snapper_set(key: &str, value: &str, valid: impl Fn(&str) -> bool) -> Result<()> {
    if !valid(value) {
        return Err(bad(format!("root snapper: bad {key}")));
    }
    let conf = Path::new("/etc/snapper/configs/root");
    let text = std::fs::read_to_string(conf).map_err(|_| bad("root snapper: no root config"))?;
    let next = set_key(
        &text,
        &re(&format!(r"^\s*{key}=")),
        &format!("{key}=\"{value}\""),
        &format!("\n{key}=\"{value}\"\n"),
    );
    atomic_write_text(conf, &next)
}

fn sudoers(user: &str) -> Result<(String, String)> {
    if !re(r"^[a-z_][a-z0-9_-]*$").is_match(user) {
        return Err(bad("root sudo: bad user name"));
    }
    Ok((
        format!("/etc/sudoers.d/99-omarchy-nopasswd-{user}"),
        format!("omarchy-nopasswd-expire-{user}"),
    ))
}

fn sudo_on(user: &str, minutes: &str) -> Result<()> {
    let (file, timer) = sudoers(user)?;
    if !(re(r"^[1-9][0-9]*$").is_match(minutes)
        && minutes.parse::<u32>().is_ok_and(|m| (1..=240).contains(&m)))
    {
        return Err(bad("root sudo: 1-240 minutes"));
    }
    atomic_write_text(
        Path::new(&file),
        &format!("{user} ALL=(ALL) NOPASSWD: ALL\n"),
    )?;
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o440))?;
    }
    quiet("systemctl", &["stop", &format!("{timer}.timer")]);
    must(
        "systemd-run",
        &[
            &format!("--on-active={minutes}m"),
            "--timer-property=AccuracySec=1s",
            &format!("--unit={timer}"),
            "rm",
            &file,
        ],
    )
}

fn sudo_off(user: &str) -> Result<()> {
    let (file, timer) = sudoers(user)?;
    let _ = std::fs::remove_file(&file);
    quiet("systemctl", &["stop", &format!("{timer}.timer")]);
    Ok(())
}

/// Custom goes first, and a rejected mode write leaves the threshold alone.
fn charge_limit(mode_file: &str, limit: &str, found: &str) -> Result<()> {
    if !limit.parse::<u32>().is_ok_and(|n| (50..=100).contains(&n))
        || !found.ends_with("/charge_control_end_threshold")
        || found.contains("..")
    {
        return Err(bad("root charge-limit: bad arguments"));
    }
    if !mode_file.is_empty() {
        if !mode_file.ends_with("/charge_types") || mode_file.contains("..") {
            return Err(bad("root charge-limit: bad mode file"));
        }
        std::fs::write(mode_file, "Custom\n")?;
    }
    std::fs::write(found, format!("{limit}\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_key_replaces_every_matching_line_or_appends() {
        let key = re(r"^\s*#?\s*ParallelDownloads\s*=");
        let text = "[options]\n#ParallelDownloads = 5\nColor\n";
        assert_eq!(
            set_key(
                text,
                &key,
                "ParallelDownloads = 9",
                "\nParallelDownloads = 9\n"
            ),
            "[options]\nParallelDownloads = 9\nColor\n"
        );
        assert_eq!(
            set_key(
                "Color\n",
                &key,
                "ParallelDownloads = 9",
                "\nParallelDownloads = 9\n"
            ),
            "Color\n\nParallelDownloads = 9\n"
        );
    }

    #[test]
    fn sudoers_names_are_checked() {
        assert!(sudoers("ann").is_ok());
        assert!(sudoers("../x").is_err());
        assert!(sudoers("Ann").is_err());
    }
}
