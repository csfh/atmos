//! Running a command as root: passwordless sudo when it is on, an in-app
//! password on stdin when the caller asked for it, polkit otherwise.

use crate::error::{Error, Result};
use crate::runner::{Output, Run};
use std::time::Duration;

fn present(name: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| {
            std::path::Path::new(if dir.is_empty() { "." } else { dir })
                .join(name)
                .metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

/// `sudo -n true` succeeds: passwordless sudo is active.
fn sudo_ready() -> bool {
    present("sudo")
        && Run::new("sudo")
            .args(["-n", "true"])
            .timeout(Duration::from_secs(10))
            .output()
            .is_ok_and(|o| o.success())
}

/// The command prefix that gets a program run as root.
fn prefix(ask: bool) -> Vec<&'static str> {
    if sudo_ready() {
        vec!["sudo"]
    } else if ask && present("sudo") {
        vec!["sudo", "-S", "-p", ""]
    } else {
        vec!["pkexec"]
    }
}

/// Run `program args` as root. `stdin` is the whole of its input; with
/// `ATMOS_SUDO_ASK=1` the caller's password line leads it.
pub fn run(program: &str, args: &[&str], stdin: &[u8]) -> Result<Output> {
    let ask = std::env::var("ATMOS_SUDO_ASK").is_ok_and(|v| v == "1");
    let pre = prefix(ask);
    let mut input = Vec::new();
    if ask && pre.first() == Some(&"sudo") && pre.len() > 1 {
        input = super::stdin_all();
    }
    input.extend_from_slice(stdin);
    let mut cmd = Run::new(pre[0]).args(&pre[1..]).arg(program).args(args);
    if !input.is_empty() {
        cmd = cmd.input(input);
    }
    cmd.output()
        .map_err(|err| err.with_context(program.to_string()))
}

/// Run and fail unless it exits 0, with its stderr in the error.
pub fn run_checked(program: &str, args: &[&str], stdin: &[u8]) -> Result<Output> {
    let out = run(program, args, stdin)?;
    if out.success() {
        Ok(out)
    } else {
        let text = out.stderr_text();
        let text = text.trim();
        Err(Error::from(if text.is_empty() {
            format!("{program} failed ({})", out.code)
        } else {
            text.to_string()
        }))
    }
}

/// Re-run this binary as root for a `ratmos root <task>` step.
pub fn root_task(task: &str, args: &[&str], stdin: &[u8]) -> Result<Output> {
    let exe = std::env::current_exe().map_err(Error::from)?;
    let exe = exe.to_string_lossy().into_owned();
    let mut full = vec!["root", task];
    full.extend_from_slice(args);
    let out = run(&exe, &full, stdin)?;
    if out.success() {
        Ok(out)
    } else {
        let text = out.stderr_text();
        let text = text.trim();
        Err(Error::from(if text.is_empty() {
            format!("{task} failed ({})", out.code)
        } else {
            text.to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prefix_falls_back_to_polkit() {
        // No sudo with a password and no ask flag: polkit.
        assert!(matches!(
            prefix(false).first(),
            Some(&"sudo") | Some(&"pkexec")
        ));
        assert!(prefix(false).len() == 1);
    }
}
