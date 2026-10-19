//! The systemd user unit that runs `ratmos daemon`.
//!
//! `ratmos install` and `update` write it and restart it, so the daemon is
//! never older than the app. A packaged install turns it on with
//! `atmos --setup`, like the Hyprland drop-in. Everything here is best
//! effort: without a systemd user session Atmos still works, it just collects
//! each inventory itself.

use crate::error::Result;
use crate::fsutil::atomic_write_text;
use crate::runner::Run;
use std::path::{Path, PathBuf};

const UNIT: &str = include_str!("../../../packaging/ratmos.service");
const NAME: &str = "ratmos.service";

pub fn unit_path() -> PathBuf {
    let config = match std::env::var("XDG_CONFIG_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config"),
    };
    config.join("systemd/user").join(NAME)
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

/// The unit text with `ExecStart` pointing at the installed ratmos.
pub fn render(exe: &Path) -> String {
    UNIT.replacen(
        "ExecStart=ratmos daemon\n",
        &format!("ExecStart={} daemon\n", exec_word(exe)),
        1,
    )
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

/// Write the unit, enable it, and (re)start it on the binary at `exe`.
pub fn enable(exe: &Path) -> Result<()> {
    let path = unit_path();
    let text = render(exe);
    if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        atomic_write_text(&path, &text)?;
        systemctl(&["daemon-reload"]);
    }
    systemctl(&["enable", NAME]);
    // Restart rather than start: after an update the old process runs old code.
    systemctl(&["restart", NAME]);
    Ok(())
}

/// Stop and remove the unit. Cached samples live in tmpfs and go on their own.
pub fn disable() {
    systemctl(&["disable", "--now", NAME]);
    if std::fs::remove_file(unit_path()).is_ok() {
        systemctl(&["daemon-reload"]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_has_the_line_install_rewrites() {
        assert!(UNIT.contains("ExecStart=ratmos daemon\n"));
    }

    #[test]
    fn the_unit_runs_at_the_lowest_priority() {
        for line in ["Nice=15", "CPUWeight=20", "IOSchedulingPriority=7"] {
            assert!(UNIT.contains(line), "{line}");
        }
        assert!(UNIT.contains("WantedBy=graphical-session.target"));
    }

    #[test]
    fn the_unit_never_runs_idle_because_it_also_answers_windows() {
        assert!(!UNIT.contains("CPUSchedulingPolicy=idle"));
        assert!(!UNIT.contains("IOSchedulingClass=idle"));
    }

    #[test]
    fn render_points_exec_start_at_the_installed_binary() {
        let text = render(Path::new("/home/me/.local/share/atmos/bin/ratmos"));
        assert!(text.contains("ExecStart=\"/home/me/.local/share/atmos/bin/ratmos\" daemon\n"));
        assert!(!text.contains("ExecStart=ratmos"));
    }

    #[test]
    fn render_survives_spaces_and_specifiers() {
        let text = render(Path::new("/home/a b/50%/ra$mos"));
        assert!(
            text.contains("ExecStart=\"/home/a b/50%%/ra$$mos\" daemon\n"),
            "{text}"
        );
    }
}
