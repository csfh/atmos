//! A no-sudo health snapshot for the Atmos Diagnostics page.

use super::time::utc_stamp;
use super::{read_strip, read_text};
use crate::hyprland::Kind;
use crate::runner::Run;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct Env {
    sys: PathBuf,
    skip_cmds: bool,
    home: String,
}

impl Env {
    fn from_env() -> Env {
        let var = |name: &str| std::env::var(name).unwrap_or_default();
        let sys = var("ATMOS_SYS_ROOT");
        let home = [var("ATMOS_HOME"), var("HOME")]
            .into_iter()
            .find(|h| !h.is_empty())
            .unwrap_or_default();
        Env {
            sys: PathBuf::from(if sys.is_empty() { "/".into() } else { sys }),
            skip_cmds: matches!(var("ATMOS_DIAG_SKIP_CMDS").as_str(), "1" | "true" | "yes"),
            home,
        }
    }

    fn root(&self, rel: &str) -> PathBuf {
        self.sys.join(rel.trim_start_matches('/'))
    }

    fn run(&self, program: &str, args: &[&str], secs: u64) -> String {
        if self.skip_cmds {
            return String::new();
        }
        Run::new(program)
            .args(args)
            .timeout(Duration::from_secs(secs))
            .checked()
            .map(|out| out.stdout_text())
            .unwrap_or_default()
    }
}

pub fn collect() -> Value {
    let env = Env::from_env();
    let online = default_route(&env);
    let kind = if online {
        network_kind(&env)
    } else {
        "offline".into()
    };
    json!({
        "generatedAt": utc_stamp(now_secs()),
        "hostname": hostname(&env),
        "kernel": kernel(),
        "omarchy": omarchy_info(&env),
        "atmos": atmos_info(&env),
        "hyprland": hyprland_info(&env),
        "failedUnits": failed_units(&env),
        "disk": disk(&env),
        "memory": memory(&env),
        "gpu": gpu(&env),
        "portals": {
            "xdg-desktop-portal": user_state(&env, "xdg-desktop-portal.service"),
            "xdg-desktop-portal-hyprland": user_state(&env, "xdg-desktop-portal-hyprland.service"),
            "xdg-desktop-portal-gtk": user_state(&env, "xdg-desktop-portal-gtk.service"),
        },
        "pipewire": {
            "pipewire": user_state(&env, "pipewire.service"),
            "pipewire-pulse": user_state(&env, "pipewire-pulse.service"),
            "wireplumber": user_state(&env, "wireplumber.service"),
        },
        "network": { "online": online, "kind": kind },
        "pacman": pacman(&env),
        "recentErrors": recent_errors(&env),
    })
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn looks_secret(text: &str) -> bool {
    let lower = text.to_lowercase();
    ["password", "passwd", "secret", "token", "bearer "]
        .iter()
        .any(|word| lower.contains(word))
}

fn clean_line(text: &str, limit: usize) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.is_empty() || looks_secret(&line) {
        return String::new();
    }
    line.chars().take(limit).collect()
}

fn unit_ok(unit: &str) -> bool {
    !unit.is_empty()
        && unit
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ":_.@\\-".contains(c))
}

fn hostname(env: &Env) -> String {
    let raw = read_strip(&env.root("etc/hostname"), 256);
    let mut name = raw.split_whitespace().next().unwrap_or("").to_string();
    if name.is_empty() {
        name = read_strip(Path::new("/proc/sys/kernel/hostname"), 256);
    }
    name.chars().take(253).collect()
}

fn kernel() -> Value {
    let proc = |name: &str| read_strip(&Path::new("/proc/sys/kernel").join(name), 64);
    json!({
        "sysname": proc("ostype").chars().take(32).collect::<String>(),
        "release": proc("osrelease"),
        "machine": std::env::consts::ARCH,
    })
}

fn kb_to_bytes(raw: &str) -> u64 {
    raw.split_whitespace()
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .map_or(0, |n| n * 1024)
}

fn memory(env: &Env) -> Value {
    let (mut total, mut available, mut swap_total, mut swap_used) = (0u64, 0u64, 0u64, 0u64);
    for line in read_text(&env.root("proc/meminfo"), 4096).lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let value = kb_to_bytes(rest.trim());
        match key {
            "MemTotal" => total = value,
            "MemAvailable" => available = value,
            "SwapTotal" => swap_total = value,
            "SwapFree" => {
                swap_used = if swap_total > 0 {
                    swap_total.saturating_sub(value)
                } else {
                    0
                }
            }
            _ => {}
        }
    }
    if total > 0 && available > total {
        available = total;
    }
    let used = if total > 0 {
        total.saturating_sub(available)
    } else {
        0
    };
    json!({
        "total": total,
        "used": used,
        "available": available,
        "swapTotal": swap_total,
        "swapUsed": swap_used,
    })
}

#[repr(C)]
struct StatVfs {
    bsize: u64,
    frsize: u64,
    blocks: u64,
    bfree: u64,
    bavail: u64,
    rest: [u64; 10],
}

extern "C" {
    fn statvfs(path: *const std::os::raw::c_char, buf: *mut StatVfs) -> i32;
}

fn disk(env: &Env) -> Value {
    use std::os::unix::ffi::OsStrExt;
    let zero = json!({"path": "/", "total": 0, "used": 0, "available": 0, "percent": 0});
    let Ok(path) = std::ffi::CString::new(env.sys.as_os_str().as_bytes()) else {
        return zero;
    };
    let mut st = StatVfs {
        bsize: 0,
        frsize: 0,
        blocks: 0,
        bfree: 0,
        bavail: 0,
        rest: [0; 10],
    };
    // SAFETY: `st` is a writable struct at least as large as glibc's LP64 statvfs.
    if unsafe { statvfs(path.as_ptr(), &mut st) } != 0 {
        return zero;
    }
    let total = st.frsize * st.blocks;
    let available = st.frsize * st.bavail;
    let used = if total > 0 {
        total.saturating_sub(st.frsize * st.bfree)
    } else {
        0
    };
    let percent = if total > 0 {
        ((used as f64 / total as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u64
    } else {
        0
    };
    json!({"path": "/", "total": total, "used": used, "available": available, "percent": percent})
}

fn gpu(env: &Env) -> Value {
    let drm = env.root("sys/class/drm");
    let mut driver = String::new();
    let mut names: Vec<String> = fs::read_dir(&drm)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    for name in names {
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let Ok(target) = fs::read_link(drm.join(&name).join("device/driver")) else {
            continue;
        };
        driver = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !driver.is_empty() {
            break;
        }
    }
    let mut identity = String::new();
    for line in read_text(&env.root("sys/class/drm/card0/device/uevent"), 2048).lines() {
        if let Some(value) = line.strip_prefix("DRIVER=") {
            if driver.is_empty() {
                driver = value.trim().to_string();
            }
        }
        if let Some(value) = line.strip_prefix("PCI_ID=") {
            identity = value.trim().to_string();
        }
    }
    json!({
        "driver": driver.chars().take(64).collect::<String>(),
        "identity": identity.chars().take(160).collect::<String>(),
    })
}

fn default_route(env: &Env) -> bool {
    read_text(&env.root("proc/net/route"), 8192)
        .lines()
        .skip(1)
        .any(|line| line.split_whitespace().nth(1) == Some("00000000"))
}

fn network_kind(env: &Env) -> String {
    let net = env.root("sys/class/net");
    let Ok(entries) = fs::read_dir(&net) else {
        return String::new();
    };
    let (mut wired, mut wifi) = (false, false);
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "lo" || read_strip(&net.join(&name).join("operstate"), 32) != "up" {
            continue;
        }
        if net.join(&name).join("wireless").is_dir() {
            wifi = true;
        } else {
            wired = true;
        }
    }
    match (wifi, wired) {
        (true, _) => "wifi",
        (_, true) => "ethernet",
        _ => "offline",
    }
    .into()
}

fn pacman(env: &Env) -> Value {
    let Ok(entries) = fs::read_dir(env.root("var/lib/pacman/sync")) else {
        return json!({"syncOk": false, "lastSync": "", "dbCount": 0});
    };
    let (mut count, mut last) = (0u32, 0i64);
    for entry in entries.flatten() {
        if !entry.file_name().to_string_lossy().ends_with(".db") {
            continue;
        }
        let Some(mtime) = entry.metadata().ok().and_then(|m| m.modified().ok()) else {
            continue;
        };
        count += 1;
        let secs = mtime
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        last = last.max(secs);
    }
    json!({
        "syncOk": count > 0,
        "lastSync": if last > 0 { utc_stamp(last) } else { String::new() },
        "dbCount": count,
    })
}

fn omarchy_info(env: &Env) -> Value {
    let omarchy_path = std::env::var("OMARCHY_PATH")
        .ok()
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "/usr/share/omarchy".into());
    let path = if Path::new(&omarchy_path).is_dir() {
        omarchy_path
    } else {
        String::new()
    };
    json!({
        "version": clean_line(&env.run("omarchy", &["version"], 2), 64),
        "channel": clean_line(&env.run("omarchy", &["channel", "current"], 2), 32),
        "path": path.chars().take(240).collect::<String>(),
    })
}

fn file_has(path: &Path, needle: &str) -> bool {
    read_text(path, 65536).contains(needle)
}

fn atmos_info(env: &Env) -> Value {
    let data = match std::env::var("XDG_DATA_HOME") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => Path::new(&env.home).join(".local/share"),
    }
    .join("atmos");
    let mut revision = read_strip(&data.join("REVISION"), 80);
    let root = crate::platform::app_root();
    let packaged = root.join("PACKAGED").is_file();
    if packaged {
        revision = read_strip(&root.join("PACKAGED"), 80);
    }
    let hypr = (!env.home.is_empty()).then(|| Path::new(&env.home).join(".config/hypr"));
    let has = |file: &str, needle: &str| {
        hypr.as_ref()
            .is_some_and(|h| file_has(&h.join(file), needle))
    };
    json!({
        "revision": revision.chars().take(80).collect::<String>(),
        "packaged": packaged,
        "installed": packaged || !revision.is_empty() || data.is_dir(),
        "hyprAtmos": has("hyprland.lua", r#"require("hypr.atmos")"#),
        "hyprAtmosLayout": has("hyprland.lua", r#"require("hypr.atmos_layout")"#),
        "sentinels": {
            "look": has("looknfeel.lua", Kind::Look.begin()),
            "input": has("input.lua", Kind::Input.begin()),
            "autostart": has("autostart.lua", Kind::Autostart.begin()),
            "bindings": has("bindings.lua", Kind::Bindings.begin()),
            "windows": has("atmos.lua", Kind::Windows.begin()),
        },
    })
}

fn hyprland_info(env: &Env) -> Value {
    let raw = env.run("hyprctl", &["version"], 2);
    let mut version = String::new();
    if let Some(first) = raw.lines().next() {
        let mut words = first.split_whitespace();
        let named = words.by_ref().skip_while(|w| *w != "Hyprland").nth(1);
        version = match named {
            Some(v) => v.to_string(),
            None => clean_line(first, 80),
        };
    }
    let errors: Vec<String> = env
        .run("hyprctl", &["configerrors"], 2)
        .lines()
        .map(|l| clean_line(l, 240))
        .filter(|l| !l.is_empty())
        .take(20)
        .collect();
    json!({"version": version.chars().take(80).collect::<String>(), "configErrors": errors})
}

fn parse_failed(raw: &str, scope: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let cols = split_n(line, 5);
        let Some(unit) = cols.first().filter(|u| unit_ok(u)) else {
            continue;
        };
        out.push(json!({
            "unit": unit,
            "scope": scope,
            "sub": clean_line(cols.get(3).map_or("", |s| s), 32),
            "description": clean_line(cols.get(4).map_or("", |s| s), 160),
        }));
        if out.len() >= 12 {
            break;
        }
    }
    out
}

/// Python's `str.split(None, n - 1)`: at most `n` fields, the last keeps the rest.
fn split_n(line: &str, n: usize) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = line.trim_start();
    while !rest.is_empty() {
        if out.len() + 1 == n {
            out.push(rest.trim_end());
            break;
        }
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        out.push(&rest[..end]);
        rest = rest[end..].trim_start();
    }
    out
}

fn failed_units(env: &Env) -> Vec<Value> {
    let base = ["--failed", "--plain", "--no-legend", "--no-pager"];
    let mut out = parse_failed(&env.run("systemctl", &base, 2), "system");
    let user = ["--user", "--failed", "--plain", "--no-legend", "--no-pager"];
    out.extend(parse_failed(&env.run("systemctl", &user, 2), "user"));
    out
}

fn user_state(env: &Env, unit: &str) -> String {
    let raw = env.run("systemctl", &["--user", "is-active", unit], 2);
    let raw = raw.trim();
    match raw {
        "active" | "inactive" | "failed" | "activating" | "deactivating" => raw.into(),
        "" => String::new(),
        _ => "unknown".into(),
    }
}

fn recent_errors(env: &Env) -> Vec<String> {
    env.run(
        "journalctl",
        &["-b", "-p", "3", "-n", "15", "--no-pager", "-o", "cat"],
        3,
    )
    .lines()
    .map(|l| clean_line(l, 240))
    .filter(|l| !l.is_empty())
    .take(15)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_n_matches_python_split_none() {
        assert_eq!(split_n("a b  c d e f", 5), ["a", "b", "c", "d", "e f"]);
        assert_eq!(split_n("  a b", 5), ["a", "b"]);
        assert!(split_n("   ", 5).is_empty());
    }

    #[test]
    fn secrets_never_reach_the_page() {
        assert_eq!(clean_line("login with password hunter2", 240), "");
        assert_eq!(clean_line("  a   b ", 240), "a b");
        assert_eq!(clean_line("abcdef", 3), "abc");
    }

    #[test]
    fn unit_names_are_restricted() {
        assert!(unit_ok("foo@bar.service"));
        assert!(!unit_ok("foo bar"));
        assert!(!unit_ok("a;b"));
    }
}
