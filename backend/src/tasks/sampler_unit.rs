//! The systemd user units that run `atmosd daemon` and `atmosd sampler`, and
//! the socket that starts the daemon when a window first connects.
//!
//! `atmosd install` and `update` write them and restart them, so the daemon is
//! never older than the app. A packaged install turns them on with
//! `atmos --setup`, like the Hyprland drop-in. Everything here is best
//! effort: without a systemd user session Atmos still works, it just collects
//! each inventory itself.

use crate::error::Result;
use crate::fsutil::atomic_write_text;
use crate::runner::Run;
use std::path::{Path, PathBuf};

struct Unit {
    name: &'static str,
    template: &'static str,
    /// The `ExecStart=` line in the template, which install points at the binary.
    exec: &'static str,
}

/// The socket first, then the daemon that answers on it, then the sampler,
/// which is `PartOf` the daemon and connects to its socket.
const UNITS: [Unit; 3] = [
    Unit {
        name: "atmosd.socket",
        template: include_str!("../../../packaging/atmosd.socket"),
        // No program: systemd listens, the daemon is started on a connection.
        exec: "",
    },
    Unit {
        name: "atmosd.service",
        template: include_str!("../../../packaging/atmosd.service"),
        exec: "ExecStart=atmosd daemon\n",
    },
    Unit {
        name: "atmosd-sampler.service",
        template: include_str!("../../../packaging/atmosd-sampler.service"),
        exec: "ExecStart=atmosd sampler\n",
    },
];

pub fn unit_path(name: &str) -> PathBuf {
    let config = match std::env::var("XDG_CONFIG_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"),
    };
    config.join("systemd/user").join(name)
}

/// A path as one ExecStart word: quoted, with the characters systemd reads escaped.
fn exec_word(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let escaped = raw
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%")
        .replace('$', "$$");
    format!("\"{escaped}\"")
}

/// The unit text with `ExecStart` pointing at the installed atmosd.
fn render(unit: &Unit, exe: &Path) -> String {
    if unit.exec.is_empty() {
        return unit.template.to_string();
    }
    let subcommand = unit.exec.trim_start_matches("ExecStart=atmosd ");
    unit.template.replacen(
        unit.exec,
        &format!("ExecStart={} {subcommand}", exec_word(exe)),
        1,
    )
}

/// A daemon a window started on its own (`sampler::launch`). It holds the
/// socket and the lock, so the unit's daemon could not start beside it.
const ADHOC: &str = "atmosd-adhoc.service";

/// Run `systemctl --user` and say nothing when it fails: stopping a unit that
/// is not loaded is the usual case.
fn systemctl_quiet(args: &[&str]) {
    if crate::tasks::present("systemctl") {
        let _ = Run::new("systemctl").arg("--user").args(args).output();
    }
}

fn systemctl(args: &[&str]) {
    if !crate::tasks::present("systemctl") {
        return;
    }
    let out = Run::new("systemctl").arg("--user").args(args).output();
    if let Ok(out) = out {
        if !out.success() {
            eprintln!(
                "atmos: systemctl --user {} failed: {}",
                args.join(" "),
                out.stderr_text().trim()
            );
        }
    }
}

/// Write the units, enable them, and (re)start them on the binary at `exe`.
pub fn enable(exe: &Path) -> Result<()> {
    let mut changed = false;
    for unit in &UNITS {
        let path = unit_path(unit.name);
        let text = render(unit, exe);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            atomic_write_text(&path, &text)?;
            changed = true;
        }
    }
    if changed {
        systemctl(&["daemon-reload"]);
    }
    systemctl_quiet(&["stop", ADHOC]);
    for unit in &UNITS {
        systemctl(&["enable", unit.name]);
        // Restart rather than start: after an update the old process runs old code.
        systemctl(&["restart", unit.name]);
    }
    Ok(())
}

/// Stop and remove the units. Cached samples live in tmpfs and go on their own.
pub fn disable() {
    let mut removed = false;
    systemctl_quiet(&["stop", ADHOC]);
    for unit in UNITS.iter().rev() {
        systemctl(&["disable", "--now", unit.name]);
        removed |= std::fs::remove_file(unit_path(unit.name)).is_ok();
    }
    if removed {
        systemctl(&["daemon-reload"]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(name: &str) -> &'static Unit {
        UNITS.iter().find(|u| u.name == name).unwrap()
    }

    #[test]
    fn each_template_has_the_line_install_rewrites() {
        for unit in &UNITS {
            assert!(unit.template.contains(unit.exec), "{}", unit.name);
        }
    }

    #[test]
    fn the_socket_is_the_one_the_daemon_serves_and_is_private() {
        let text = unit("atmosd.socket").template;
        assert!(text.contains("ListenStream=%t/atmos/atmosd.sock"));
        assert!(text.contains("SocketMode=0600") && text.contains("DirectoryMode=0700"));
        assert!(text.contains("WantedBy=sockets.target"));
        let service = unit("atmosd.service").template;
        assert!(service.contains("Requires=atmosd.socket"));
        assert!(service.contains("After=atmosd.socket"));
        assert_eq!(
            render(unit("atmosd.socket"), Path::new("/x/atmosd")),
            text,
            "a socket has no ExecStart to point at the binary"
        );
    }

    #[test]
    fn the_daemon_unit_never_runs_idle_because_it_answers_windows() {
        let text = unit("atmosd.service").template;
        for line in ["Nice=15", "CPUWeight=20", "IOSchedulingPriority=7"] {
            assert!(text.contains(line), "{line}");
        }
        assert!(text.contains("WantedBy=graphical-session.target"));
        assert!(!text.contains("CPUSchedulingPolicy=idle"));
        assert!(!text.contains("IOSchedulingClass=idle"));
    }

    #[test]
    fn the_sampler_unit_is_idle_weightless_and_capped() {
        let text = unit("atmosd-sampler.service").template;
        for line in [
            "CPUSchedulingPolicy=idle",
            "IOSchedulingClass=idle",
            "Nice=19",
            "CPUWeight=1",
            "CPUQuota=",
        ] {
            assert!(text.contains(line), "{line}");
        }
    }

    #[test]
    fn the_sampler_lives_and_dies_with_the_daemon() {
        let text = unit("atmosd-sampler.service").template;
        assert!(text.contains("After=atmosd.service"));
        assert!(text.contains("PartOf=atmosd.service"));
        assert!(text.contains("WantedBy=atmosd.service"));
    }

    #[test]
    fn render_points_each_exec_start_at_the_installed_binary() {
        let exe = Path::new("/home/me/.local/share/atmos/bin/atmosd");
        let daemon = render(unit("atmosd.service"), exe);
        assert!(daemon.contains("ExecStart=\"/home/me/.local/share/atmos/bin/atmosd\" daemon\n"));
        let sampler = render(unit("atmosd-sampler.service"), exe);
        assert!(sampler.contains("ExecStart=\"/home/me/.local/share/atmos/bin/atmosd\" sampler\n"));
        assert!(!daemon.contains("ExecStart=atmosd") && !sampler.contains("ExecStart=atmosd"));
    }

    #[test]
    fn render_survives_spaces_and_specifiers() {
        let text = render(unit("atmosd.service"), Path::new("/home/a b/50%/ra$mos"));
        assert!(
            text.contains("ExecStart=\"/home/a b/50%%/ra$$mos\" daemon\n"),
            "{text}"
        );
    }
}
