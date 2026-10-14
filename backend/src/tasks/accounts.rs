//! Local users and groups, login avatars, LUKS passphrases, snapper rollback.

use super::elevate;
use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text;
use crate::runner::Run;
use regex_lite::Regex;
use std::io::Write;
use std::path::Path;

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

const RESERVED: &[&str] = &[
    "root",
    "bin",
    "daemon",
    "mail",
    "ftp",
    "http",
    "nobody",
    "dbus",
    "systemd-coredump",
    "systemd-network",
    "systemd-oom",
    "systemd-journal-remote",
    "systemd-resolve",
    "systemd-timesync",
    "tss",
    "uuidd",
    "alpm",
    "git",
    "avahi",
    "cups",
    "cups-browsed",
    "lp",
    "_talkd",
    "polkitd",
    "rtkit",
    "qemu",
    "brltty",
    "gluster",
    "rpc",
    "libvirt-qemu",
    "pcscd",
    "nvidia-persistenced",
    "sddm",
];

fn name_shape(n: &str) -> bool {
    re(r"^[a-z_][a-z0-9_-]{0,31}$").is_match(n) && !n.ends_with('-') && !n.contains("..")
}

fn ok_user(n: &str) -> bool {
    name_shape(n) && !RESERVED.contains(&n)
}

fn ok_group(n: &str) -> bool {
    name_shape(n) && !matches!(n, "root" | "nobody" | "nogroup")
}

fn self_user() -> String {
    Run::new("id")
        .arg("-un")
        .output()
        .map(|o| o.stdout_text().trim().to_string())
        .unwrap_or_default()
}

/// One line of stdin, which must be present and not empty.
fn password_line(stdin: &[u8]) -> Result<String> {
    let text = String::from_utf8_lossy(stdin);
    let line = text
        .split('\n')
        .next()
        .unwrap_or("")
        .trim_end_matches('\r')
        .to_string();
    if line.is_empty() {
        return Err(bad("manage-account: password cannot be empty"));
    }
    Ok(line)
}

pub fn manage(args: &[String], stdin: &[u8]) -> Result<()> {
    let usage = || {
        bad("Usage: manage-account add-user <name> <full-name> <wheel:true|false> | remove-user <name> | set-password <name> | set-full-name <name> <full-name> | add-group <name> | remove-group <name> | set-member <group> <user> <on|off>")
    };
    let arg = |i: usize| args.get(i).map_or("", String::as_str);
    let me = self_user();
    match arg(0) {
        "add-user" => {
            let (name, full, wheel) = (
                arg(1),
                arg(2),
                if args.len() > 3 { arg(3) } else { "false" },
            );
            if !ok_user(name) || (wheel != "true" && wheel != "false") || full.chars().count() > 256
            {
                return Err(usage());
            }
            if full.starts_with('-') || full.contains(':') || full.contains(',') {
                return Err(usage());
            }
            let pass = password_line(stdin)?;
            elevate::root_task(
                "add-user",
                &[name, full, wheel],
                format!("{name}:{pass}\n").as_bytes(),
            )
            .map(|_| ())
        }
        "remove-user" => {
            if !ok_user(arg(1)) {
                return Err(usage());
            }
            if arg(1) == me {
                return Err(bad("manage-account: will not remove the logged-in user"));
            }
            elevate::run_checked("userdel", &["-r", arg(1)], &[]).map(|_| ())
        }
        "set-password" => {
            if !ok_user(arg(1)) {
                return Err(usage());
            }
            let pass = password_line(stdin)?;
            elevate::run_checked("chpasswd", &[], format!("{}:{pass}\n", arg(1)).as_bytes())
                .map(|_| ())
        }
        "set-full-name" => {
            let (name, full) = (arg(1), arg(2));
            if !ok_user(name)
                || full.chars().count() > 256
                || full.starts_with('-')
                || full.contains('\n')
                || full.contains('\r')
                || full.contains(':')
                || full.contains(',')
            {
                return Err(usage());
            }
            elevate::run_checked("chfn", &["-f", full, name], &[]).map(|_| ())
        }
        "add-group" => {
            if !ok_group(arg(1)) {
                return Err(usage());
            }
            elevate::run_checked("groupadd", &[arg(1)], &[]).map(|_| ())
        }
        "remove-group" => {
            let name = arg(1);
            if !ok_group(name) {
                return Err(usage());
            }
            if name == "wheel" || name == "docker" {
                return Err(bad(format!("manage-account: will not remove {name}")));
            }
            let group = Run::new("getent")
                .args(["group", name])
                .output()
                .map(|o| o.stdout_text())
                .unwrap_or_default();
            let gid: Option<u32> = group.trim().split(':').nth(2).and_then(|g| g.parse().ok());
            let Some(gid) = gid else {
                return Err(usage());
            };
            if !(1000..65534).contains(&gid) {
                return Err(bad(format!("manage-account: {name} is a system group")));
            }
            elevate::run_checked("groupdel", &[name], &[]).map(|_| ())
        }
        "set-member" => {
            let (group, name, on) = (arg(1), arg(2), arg(3));
            if !ok_group(group) || !ok_user(name) || (on != "on" && on != "off") {
                return Err(usage());
            }
            if on == "off" && group == "wheel" && name == me {
                return Err(bad(
                    "manage-account: will not drop the logged-in user from wheel",
                ));
            }
            if on == "on" {
                elevate::run_checked("usermod", &["-aG", group, name], &[]).map(|_| ())
            } else {
                elevate::run_checked("gpasswd", &["-d", name, group], &[]).map(|_| ())
            }
        }
        _ => Err(usage()),
    }
}

fn home_of(user: &str) -> Option<String> {
    let entry = Run::new("getent")
        .args(["passwd", user])
        .output()
        .ok()?
        .stdout_text();
    let home = entry.trim().split(':').nth(5)?.to_string();
    (home.starts_with('/') && !home.contains("..")).then_some(home)
}

fn image_kind(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])
        || bytes.starts_with(&[0xff, 0xd8, 0xff])
}

fn install_face(home: &str, file: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(home)?;
    for name in [".face", ".face.icon"] {
        let dest = Path::new(home).join(name);
        std::fs::copy(file, &dest)?;
        let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o644));
    }
    let _ = Run::new("setfacl").args(["-m", "u:sddm:x", home]).output();
    let _ = Run::new("setfacl")
        .args(["-m", "u:sddm:r"])
        .arg(Path::new(home).join(".face.icon"))
        .output();
    Ok(())
}

pub fn avatar(args: &[String]) -> Result<()> {
    let usage = || bad("Usage: set-avatar set <user> <png-or-jpg> | clear <user>");
    let arg = |i: usize| args.get(i).map_or("", String::as_str);
    let (action, user, src) = (arg(0), arg(1), arg(2));
    if (action != "set" && action != "clear") || !name_shape(user) {
        return Err(usage());
    }
    if user == "root" {
        return Err(bad("set-avatar: will not change root's face"));
    }
    let Some(home) = home_of(user) else {
        return Err(bad(format!("set-avatar: no home for {user}")));
    };
    let mine = user == self_user();
    if action == "clear" {
        if mine {
            let _ = std::fs::remove_file(Path::new(&home).join(".face"));
            let _ = std::fs::remove_file(Path::new(&home).join(".face.icon"));
        } else {
            elevate::run_checked(
                "rm",
                &[
                    "-f",
                    &format!("{home}/.face"),
                    &format!("{home}/.face.icon"),
                ],
                &[],
            )?;
        }
        return elevate::root_task("avatar-clear", &[user], &[]).map(|_| ());
    }
    if src.is_empty() || !src.starts_with('/') || src.contains("..") {
        return Err(usage());
    }
    if !Path::new(src).is_file() {
        return Err(bad(format!("set-avatar: not a file: {src}")));
    }
    // Privileged processes cannot read user-only FUSE mounts such as rclone
    // mounts. Stage the selected file while we still have the desktop user's
    // access, then use that local copy for every write.
    let bytes = std::fs::read(src)?;
    if bytes.is_empty() || bytes.len() > 5_000_000 {
        return Err(bad("set-avatar: image must be 1 byte to 5 MB"));
    }
    if !image_kind(&bytes) {
        return Err(bad("set-avatar: need a PNG or JPEG"));
    }
    let staged = std::env::temp_dir().join(format!("atmos-avatar.{}", std::process::id()));
    std::fs::write(&staged, &bytes)?;
    let staged_path = staged.to_string_lossy().into_owned();
    let result = (|| {
        if mine {
            install_face(&home, &staged)?;
        } else {
            elevate::root_task("face-install", &[&home, &staged_path], &[])?;
        }
        elevate::root_task("avatar-install", &[user, &staged_path], &[]).map(|_| ())
    })();
    let _ = std::fs::remove_file(&staged);
    result
}

/// Privileged steps of the avatar and account tasks.
pub fn root_task(task: &str, args: &[String], stdin: &[u8]) -> Result<()> {
    let arg = |i: usize| args.get(i).map_or("", String::as_str);
    match task {
        "add-user" => {
            let (name, full, wheel) = (arg(0), arg(1), arg(2));
            if !ok_user(name)
                || (wheel != "true" && wheel != "false")
                || full.contains(':')
                || full.contains(',')
                || full.starts_with('-')
            {
                return Err(bad("root add-user: bad arguments"));
            }
            let exists = Run::new("getent")
                .args(["passwd", name])
                .output()
                .is_ok_and(|o| o.success());
            if exists {
                return Err(bad(format!("manage-account: {name} already exists")));
            }
            run_ok("useradd", &["-m", "-s", "/bin/bash", name])?;
            if !full.is_empty() {
                run_ok("chfn", &["-f", full, name])?;
            }
            if wheel == "true" {
                run_ok("usermod", &["-aG", "wheel", name])?;
            }
            let out = Run::new("chpasswd").input(stdin.to_vec()).output()?;
            if out.success() {
                Ok(())
            } else {
                Err(bad(out.stderr_text().trim().to_string()))
            }
        }
        "avatar-install" => {
            let (user, file) = (arg(0), arg(1));
            if !name_shape(user) || user == "root" || !Path::new(file).is_file() {
                return Err(bad("root avatar-install: bad arguments"));
            }
            use std::os::unix::fs::PermissionsExt;
            std::fs::create_dir_all("/var/lib/AccountsService/icons")?;
            std::fs::create_dir_all("/var/lib/AccountsService/users")?;
            let icon = format!("/var/lib/AccountsService/icons/{user}");
            std::fs::copy(file, &icon)?;
            std::fs::set_permissions(&icon, std::fs::Permissions::from_mode(0o644))?;
            let cfg = format!("/var/lib/AccountsService/users/{user}");
            crate::store::with_lock(Path::new(&cfg), || {
                let next = match std::fs::read_to_string(&cfg) {
                    Ok(text) if text.lines().any(|l| l.starts_with("Icon=")) => {
                        text.lines()
                            .map(|l| {
                                if l.starts_with("Icon=") {
                                    format!("Icon={icon}")
                                } else {
                                    l.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                            + "\n"
                    }
                    Ok(text) => format!("{text}Icon={icon}\n"),
                    Err(_) => format!("[User]\nIcon={icon}\n"),
                };
                atomic_write_text(Path::new(&cfg), &next)
            })
        }
        "avatar-clear" => {
            let user = arg(0);
            if !name_shape(user) || user == "root" {
                return Err(bad("root avatar-clear: bad arguments"));
            }
            let _ = std::fs::remove_file(format!("/var/lib/AccountsService/icons/{user}"));
            let cfg = format!("/var/lib/AccountsService/users/{user}");
            if Path::new(&cfg).is_file() {
                crate::store::with_lock(Path::new(&cfg), || {
                    let text = std::fs::read_to_string(&cfg)?;
                    let kept: Vec<&str> =
                        text.lines().filter(|l| !l.starts_with("Icon=")).collect();
                    if kept
                        .iter()
                        .any(|l| l.chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
                    {
                        atomic_write_text(Path::new(&cfg), &(kept.join("\n") + "\n"))
                    } else {
                        let _ = std::fs::remove_file(&cfg);
                        Ok(())
                    }
                })?;
            }
            Ok(())
        }
        "face-install" => {
            let (home, file) = (arg(0), arg(1));
            if !home.starts_with('/') || home.contains("..") || !Path::new(file).is_file() {
                return Err(bad("root face-install: bad arguments"));
            }
            install_face(home, Path::new(file))
        }
        other => Err(bad(format!("root: unknown task {other}"))),
    }
}

fn run_ok(program: &str, args: &[&str]) -> Result<()> {
    let out = Run::new(program).args(args).output()?;
    if out.success() {
        Ok(())
    } else {
        Err(bad(out.stderr_text().trim().to_string()))
    }
}

/// Change a LUKS passphrase. Three stdin lines: device, current, new.
pub fn luks_change_key(stdin: &[u8]) -> Result<()> {
    let text = String::from_utf8_lossy(stdin).into_owned();
    let mut lines = text.split('\n').map(|l| l.trim_end_matches('\r'));
    let (device, old, new) = (
        lines.next().unwrap_or(""),
        lines.next().unwrap_or(""),
        lines.next().unwrap_or(""),
    );
    if device.is_empty() || !device.starts_with("/dev/") || device.contains("..") {
        return Err(bad("luks-change-key: device must be a /dev path"));
    }
    if !re(r"^/dev/[A-Za-z0-9/_-]+$").is_match(device) {
        return Err(bad(
            "luks-change-key: device path has unexpected characters",
        ));
    }
    if old.is_empty() || new.is_empty() {
        return Err(bad("luks-change-key: passphrases cannot be empty"));
    }
    let listed = Run::new("blkid")
        .args(["-t", "TYPE=crypto_LUKS", "-o", "device"])
        .output()
        .map(|o| o.stdout_text())
        .unwrap_or_default();
    if !listed.lines().any(|l| l == device) {
        return Err(bad(format!(
            "luks-change-key: {device} is not a LUKS device"
        )));
    }
    let dir = std::path::PathBuf::from("/dev/shm");
    let mk = |name: &str, body: &str| -> Result<std::path::PathBuf> {
        use std::os::unix::fs::OpenOptionsExt;
        let path = dir.join(format!("atmos-luks-{}-{name}", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(body.as_bytes())?;
        Ok(path)
    };
    let (oldf, newf) = (mk("old", old)?, mk("new", new)?);
    let result = elevate::run_checked(
        "cryptsetup",
        &[
            "luksChangeKey",
            "--pbkdf",
            "argon2id",
            "--iter-time",
            "2000",
            "--key-file",
            &oldf.to_string_lossy(),
            device,
            &newf.to_string_lossy(),
        ],
        &[],
    );
    let _ = std::fs::remove_file(&oldf);
    let _ = std::fs::remove_file(&newf);
    result.map(|_| ())
}

pub fn rollback(config: &str, id: &str) -> Result<()> {
    if !re(r"^[A-Za-z0-9_-]+$").is_match(config) || !re(r"^[0-9]+$").is_match(id) {
        return Err(bad("Usage: rollback-snapshot <config> <id>"));
    }
    elevate::run_checked("snapper", &["-c", config, "rollback", id], &[]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_and_odd_names_are_refused() {
        assert!(ok_user("ann"));
        assert!(!ok_user("root"));
        assert!(!ok_user("sddm"));
        assert!(!ok_user("a-"));
        assert!(!ok_user("A"));
        assert!(!ok_group("root"));
        assert!(ok_group("sddm"));
    }

    #[test]
    fn avatar_images_are_checked_by_magic_bytes() {
        assert!(image_kind(&[
            0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0
        ]));
        assert!(image_kind(&[0xff, 0xd8, 0xff, 0xe0]));
        assert!(!image_kind(b"GIF89a"));
    }

    #[test]
    fn the_password_line_must_not_be_empty() {
        assert_eq!(password_line(b"hunter2\nmore").unwrap(), "hunter2");
        assert!(password_line(b"\n").is_err());
        assert!(password_line(b"").is_err());
    }
}
