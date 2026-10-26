//! Making sure a daemon exists. `atmosd serve` calls `ensure` when it needs
//! one and the socket is not there, so a window never runs without the
//! authority just because nobody enabled a unit.
//!
//! In order: the installed units, then a transient user unit that carries the
//! daemon's own priority, then a bare detached process. The first two keep
//! the daemon outside the window's cgroup, so closing the window that started
//! it does not end it for every other window.

use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::cache;
use crate::runner::Run;

/// How long to wait for the socket after asking for a daemon.
const WAIT: Duration = Duration::from_secs(3);

/// The environment variable that forbids starting anything. Tests set it, so a
/// test never reaches the real user manager.
pub const OFF: &str = "ATMOS_NO_AUTOSTART";

/// Which file this process runs: inode, size and mtime of its executable, as
/// found at `path`. `VERSION` stays put between dev builds, so this is what
/// tells two builds apart.
pub fn identity(path: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(path).ok()?;
    Some(format!(
        "{:x}-{:x}-{:x}-{:x}",
        meta.ino(),
        meta.len(),
        meta.mtime(),
        meta.mtime_nsec()
    ))
}

/// The identity of the running binary, at the path it was started from.
pub fn build_id() -> Option<String> {
    identity(&std::env::current_exe().ok()?)
}

pub fn reachable(dir: &Path) -> bool {
    UnixStream::connect(cache::socket(dir)).is_ok()
}

pub fn allowed() -> bool {
    std::env::var_os(OFF).is_none_or(|value| value.is_empty())
}

fn wait_for(dir: &Path) -> bool {
    let until = Instant::now() + WAIT;
    while Instant::now() < until {
        if reachable(dir) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(40));
    }
    reachable(dir)
}

/// True when a daemon answers on the socket afterwards.
pub fn ensure(dir: &Path) -> bool {
    if reachable(dir) {
        return true;
    }
    if !allowed() {
        return false;
    }
    let short = Duration::from_secs(5);
    let started = Run::new("systemctl")
        .args([
            "--user",
            "start",
            "--no-block",
            "atmosd.service",
            "atmosd-sampler.service",
        ])
        .timeout(short)
        .output()
        .is_ok_and(|out| out.success());
    if started && wait_for(dir) {
        return true;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    // The daemon must use this window's runtime dir, not the manager's.
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
    let transient = Run::new("systemd-run")
        .args([
            "--user",
            "--quiet",
            "--collect",
            "--unit=atmosd-adhoc",
            "--property=Nice=15",
            "--property=CPUWeight=20",
        ])
        .arg(format!("--setenv=XDG_RUNTIME_DIR={runtime}"))
        .arg("--")
        .arg(&exe)
        .arg("daemon")
        .timeout(short)
        .output();
    if let Err(err) = &transient {
        eprintln!("atmosd: systemd-run did not start a daemon: {err}");
    } else if let Ok(out) = &transient {
        if !out.success() {
            eprintln!(
                "atmosd: systemd-run did not start a daemon: {}",
                out.stderr_text().trim()
            );
        }
    }
    let transient = transient.is_ok_and(|out| out.success());
    if transient && wait_for(dir) {
        return true;
    }
    let _ = Command::new(&exe)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();
    wait_for(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::cache::tests::scratch;
    use crate::sampler::daemon::tests::serve;

    #[test]
    fn a_running_daemon_is_reachable_and_an_empty_dir_is_not() {
        let (dir, _daemon, _calls) = serve("reach", Duration::ZERO);
        assert!(reachable(&dir));
        assert!(!reachable(&scratch("unreachable")));
    }

    #[test]
    fn ensure_finds_a_running_daemon_without_starting_anything() {
        let (dir, _daemon, _calls) = serve("ensure", Duration::ZERO);
        assert!(ensure(&dir));
    }
}
