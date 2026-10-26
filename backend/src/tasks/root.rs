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

/// The three things one passwordless-sudo grant puts in place: the drop-in that grants, the
/// transient timer that removes it, and the boot rule that removes it if the timer is lost.
struct Grant {
    file: String,
    boot_rule: String,
    timer: String,
}

impl Grant {
    /// `etc` is `/etc`; a test gives a directory of its own.
    fn under(etc: &str, user: &str) -> Result<Grant> {
        if !re(r"^[a-z_][a-z0-9_-]*$").is_match(user) {
            return Err(bad("root sudo: bad user name"));
        }
        Ok(Grant {
            file: format!("{etc}/sudoers.d/99-omarchy-nopasswd-{user}"),
            boot_rule: format!("{etc}/tmpfiles.d/atmos-nopasswd-{user}.conf"),
            timer: format!("omarchy-nopasswd-expire-{user}"),
        })
    }

    /// Transient units do not survive a reboot, so the timer alone would leave the grant on
    /// after one while the window reads the missing timer as off (ADR 0019). `r!` runs only
    /// when `systemd-tmpfiles` is given `--boot`, which is early boot, so a later
    /// `--remove` cannot cut a live grant short; the rule removes itself with the grant.
    fn boot_rule_text(&self) -> String {
        format!(
            "# Atmos grants passwordless sudo for a while and arms a transient timer to take it\n\
             # back. Transient units do not survive a reboot, so remove the grant in early boot.\n\
             r! {}\nr! {}\n",
            self.file, self.boot_rule
        )
    }

    /// The boot rule first, so that a grant never exists without one. A failure leaves neither.
    fn write(&self, user: &str) -> Result<()> {
        let written = self.write_files(user);
        if written.is_err() {
            self.remove();
        }
        written
    }

    fn write_files(&self, user: &str) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let set_mode = |path: &str, mode| {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
                .map_err(|err| Error::io(err, Path::new(path)))
        };
        atomic_write_text(Path::new(&self.boot_rule), &self.boot_rule_text())?;
        set_mode(&self.boot_rule, 0o644)?;
        atomic_write_text(
            Path::new(&self.file),
            &format!("{user} ALL=(ALL) NOPASSWD: ALL\n"),
        )?;
        set_mode(&self.file, 0o440)
    }

    fn remove(&self) {
        let _ = std::fs::remove_file(&self.file);
        let _ = std::fs::remove_file(&self.boot_rule);
    }
}

fn sudo_on(user: &str, minutes: &str) -> Result<()> {
    let grant = Grant::under("/etc", user)?;
    if !(re(r"^[1-9][0-9]*$").is_match(minutes)
        && minutes.parse::<u32>().is_ok_and(|m| (1..=240).contains(&m)))
    {
        return Err(bad("root sudo: 1-240 minutes"));
    }
    grant.write(user)?;
    quiet("systemctl", &["stop", &format!("{}.timer", grant.timer)]);
    // The timer takes the rule with the grant: the grant is over, and the rule has nothing left
    // to do.
    let armed = must(
        "systemd-run",
        &[
            &format!("--on-active={minutes}m"),
            "--timer-property=AccuracySec=1s",
            &format!("--unit={}", grant.timer),
            "rm",
            "-f",
            "--",
            &grant.file,
            &grant.boot_rule,
        ],
    );
    if armed.is_err() {
        // A grant with no way to end is not left on.
        grant.remove();
    }
    armed
}

fn sudo_off(user: &str) -> Result<()> {
    let grant = Grant::under("/etc", user)?;
    grant.remove();
    quiet("systemctl", &["stop", &format!("{}.timer", grant.timer)]);
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
        assert!(Grant::under("/etc", "ann").is_ok());
        assert!(Grant::under("/etc", "../x").is_err());
        assert!(Grant::under("/etc", "Ann").is_err());
    }

    fn scratch(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("atmosd-root-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sudoers.d")).unwrap();
        std::fs::create_dir_all(dir.join("tmpfiles.d")).unwrap();
        dir.to_str().unwrap().to_string()
    }

    #[test]
    fn a_grant_comes_with_a_rule_that_removes_it_at_boot_and_itself() {
        use std::os::unix::fs::PermissionsExt;
        let etc = scratch("grant");
        let grant = Grant::under(&etc, "ann").unwrap();
        grant.write("ann").unwrap();
        let rule = std::fs::read_to_string(&grant.boot_rule).unwrap();
        let lines: Vec<&str> = rule.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(
            lines,
            [
                format!("r! {etc}/sudoers.d/99-omarchy-nopasswd-ann"),
                format!("r! {etc}/tmpfiles.d/atmos-nopasswd-ann.conf"),
            ]
        );
        let mode = |p: &str| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&grant.file), 0o440);
        assert_eq!(mode(&grant.boot_rule), 0o644);
        assert_eq!(
            std::fs::read_to_string(&grant.file).unwrap(),
            "ann ALL=(ALL) NOPASSWD: ALL\n"
        );
        grant.remove();
        assert!(!Path::new(&grant.file).exists() && !Path::new(&grant.boot_rule).exists());
    }

    /// The rule is read by the real `systemd-tmpfiles`, in a root of its own: a run without
    /// `--boot` leaves the grant alone, and the boot run removes it and the rule.
    #[test]
    fn systemd_tmpfiles_removes_the_grant_at_boot_and_not_before() {
        if !Run::new("systemd-tmpfiles")
            .arg("--version")
            .output()
            .is_ok_and(|out| out.success())
        {
            eprintln!("skipped: no systemd-tmpfiles");
            return;
        }
        let root = scratch("tmpfiles");
        let grant = Grant::under("/etc", "ann").unwrap();
        let in_root = |path: &str| format!("{root}{path}");
        std::fs::create_dir_all(in_root("/etc/sudoers.d")).unwrap();
        std::fs::create_dir_all(in_root("/etc/tmpfiles.d")).unwrap();
        std::fs::write(in_root(&grant.file), "ann ALL=(ALL) NOPASSWD: ALL\n").unwrap();
        std::fs::write(in_root(&grant.boot_rule), grant.boot_rule_text()).unwrap();
        let tmpfiles = |boot: bool| {
            let mut run =
                Run::new("systemd-tmpfiles").args(["--create", "--remove", "--root", &root]);
            if boot {
                run = run.arg("--boot");
            }
            assert!(run.output().unwrap().success());
        };
        tmpfiles(false);
        assert!(Path::new(&in_root(&grant.file)).exists());
        tmpfiles(true);
        assert!(!Path::new(&in_root(&grant.file)).exists());
        assert!(!Path::new(&in_root(&grant.boot_rule)).exists());
    }

    #[test]
    fn a_grant_that_cannot_be_written_leaves_nothing() {
        let etc = scratch("nodir");
        std::fs::remove_dir_all(format!("{etc}/sudoers.d")).unwrap();
        std::fs::write(format!("{etc}/sudoers.d"), "in the way").unwrap();
        let grant = Grant::under(&etc, "ann").unwrap();
        assert!(grant.write("ann").is_err());
        assert!(!Path::new(&grant.boot_rule).exists());
    }
}
