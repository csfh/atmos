//! Everything the Quickshell UI used to run as a helper script, as one Rust
//! entry point: `atmosd task <name> [args...]`. Each task keeps the argument
//! shape its script had, prints what the page shows, and exits non-zero with
//! the reason on stderr when it fails. `atmosd root <task>` is the privileged
//! half that polkit or sudo starts, never a command anyone types.

pub mod accounts;
pub mod app;
pub mod elevate;
pub mod net;
pub mod root;
pub mod sampler_unit;
pub mod settings_apply;
pub mod system;
pub mod user;

use crate::error::Result;
use std::io::{Read, Write};

/// Everything on stdin, read at most once per process.
pub fn stdin_all() -> Vec<u8> {
    static CACHE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut buf = Vec::new();
            let _ = std::io::stdin().read_to_end(&mut buf);
            buf
        })
        .clone()
}

/// `command -v name`
pub fn present(name: &str) -> bool {
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

/// The task names, for usage text and the guard test.
pub const NAMES: &[&str] = &[
    "add-desktop-launcher",
    "create-hook",
    "diag-report",
    "enterprise-wifi-connect",
    "gum-job",
    "hypr-apply",
    "luks-change-key",
    "manage-account",
    "refresh-hyprland",
    "reset-atmos",
    "reset-hypr",
    "rollback-snapshot",
    "set-atmos-channel",
    "set-audio",
    "set-avatar",
    "set-charge-limit",
    "set-dns-custom",
    "set-favorites",
    "set-fstrim",
    "set-full-name",
    "set-hook-sample",
    "set-hostname",
    "set-keyboard-layout",
    "set-locale",
    "set-mime-default",
    "set-ntp",
    "set-parallel-downloads",
    "set-passwordless-sudo",
    "set-presentation",
    "set-snapper-policy",
    "set-sshd",
    "set-timezone",
    "set-tweaks",
    "set-wifi-connection",
    "settings-apply",
    "settings-set",
    "setup-atmos",
    "signal-process",
    "update-atmos",
];

/// Run one task. Returns the process exit code.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let name = args.first().map_or("", String::as_str);
    let mut rest: Vec<String> = args[args.len().min(1)..].to_vec();
    // The in-app password prompt answers sudo on stdin.
    if let Some(at) = rest.iter().position(|a| a == "--sudo-ask") {
        rest.remove(at);
        std::env::set_var("ATMOS_SUDO_ASK", "1");
    }
    let rest = &rest[..];
    let arg = |i: usize| rest.get(i).map_or("", String::as_str);
    let result: Result<i32> = (|| {
        let done = |r: Result<()>| r.map(|()| 0);
        match name {
            "add-desktop-launcher" => done(user::add_desktop_launcher(arg(0), arg(1), arg(2))),
            "create-hook" => done(user::create_hook(arg(0), arg(1), arg(2))),
            "diag-report" => done(user::diag_report(
                if arg(0).is_empty() { "copy" } else { arg(0) },
                arg(1),
            )),
            "enterprise-wifi-connect" => done(net::enterprise_connect(rest, &stdin_all())),
            "gum-job" => user::gum_job(rest),
            "hypr-apply" => done(user::hypr_apply(arg(0), arg(1))),
            "luks-change-key" => done(accounts::luks_change_key(&stdin_all())),
            "manage-account" => done(accounts::manage(rest, &stdin_all())),
            "refresh-hyprland" => done(user::refresh_hyprland()),
            "reset-atmos" => {
                if rest.is_empty() {
                    done(user::reset_atmos())
                } else {
                    Err(crate::error::Error::bad_request("Usage: reset-atmos"))
                }
            }
            "reset-hypr" => done(user::reset_block(arg(0))),
            "rollback-snapshot" => done(accounts::rollback(arg(0), arg(1))),
            "set-atmos-channel" => done(app::write_channel(arg(0))),
            "set-audio" => done(user::audio(arg(0), arg(1))),
            "set-avatar" => done(accounts::avatar(rest)),
            "set-charge-limit" => done(system::charge_limit(arg(0))),
            "set-full-name" => done(system::full_name(arg(0))),
            "set-hostname" => done(system::hostname(arg(0))),
            "set-keyboard-layout" => done(system::keyboard_layout(arg(0))),
            "set-locale" => done(system::locale(arg(0))),
            "set-mime-default" => done(user::mime_default(arg(0), arg(1))),
            "set-ntp" => done(system::ntp(arg(0))),
            "set-parallel-downloads" => done(system::parallel_downloads(arg(0))),
            "set-timezone" => done(system::timezone(arg(0))),
            "set-tweaks" => done(system::tweak(arg(0), arg(1))),
            "set-dns-custom" => done(system::dns_custom(
                &rest.iter().map(String::as_str).collect::<Vec<_>>(),
            )),
            "set-favorites" => {
                if arg(0) != "write" || rest.len() != 2 {
                    Err(crate::error::Error::bad_request(
                        "usage: set-favorites write <json>",
                    ))
                } else {
                    done(user::favorites_write(arg(1)))
                }
            }
            "set-fstrim" => done(system::fstrim(arg(0))),
            "set-hook-sample" => done(user::hook_sample(arg(0), arg(1))),
            "set-passwordless-sudo" => done(system::passwordless_sudo(
                arg(0),
                if arg(1).is_empty() { "15" } else { arg(1) },
            )),
            "set-presentation" => done(user::presentation(arg(0), rest.get(1).map(String::as_str))),
            "set-snapper-policy" => done(system::snapper_policy(arg(0), arg(1))),
            "set-sshd" => {
                if arg(0) != "disable" {
                    Err("Usage: set-sshd disable".to_string().into())
                } else {
                    done(system::sshd_disable())
                }
            }
            "settings-apply" => settings_apply::run(rest, stdout, stderr),
            "settings-set" => {
                let value: serde_json::Value = serde_json::from_str(arg(1))
                    .map_err(|_| "usage: settings-set <domain> <json>".to_string())?;
                crate::write_domain(crate::platform::Backend::Omarchy, None, arg(0), &value)
                    .map(|()| 0)
            }
            "set-wifi-connection" => done(net::connection(rest, &stdin_all())),
            "setup-atmos" => done(app::setup(if arg(0).is_empty() { "on" } else { arg(0) })),
            "signal-process" => done(user::signal_process(arg(0), arg(1))),
            "update-atmos" => app::update(arg(0)),
            _ => Err(format!(
                "atmosd task: unknown task {name}. Tasks: {}",
                NAMES.join(", ")
            )
            .into()),
        }
    })();
    finish(result, stdout, stderr)
}

/// `atmosd root <task> [args...]`: run as root by `elevate`.
pub fn run_root(args: &[String], stderr: &mut dyn Write) -> i32 {
    let name = args.first().map_or("", String::as_str);
    let stdin = if matches!(name, "add-user") {
        stdin_all()
    } else {
        Vec::new()
    };
    let result = root::run(name, &args[args.len().min(1)..], &stdin).map(|()| 0);
    finish(result, &mut std::io::sink(), stderr)
}

fn finish(result: Result<i32>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let _ = stdout.flush();
    match result {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(stderr, "{}", err.message);
            // A usage mistake exits 2; a failure to do the work exits 1.
            if err.kind == crate::error::Kind::BadRequest {
                2
            } else {
                1
            }
        }
    }
}
