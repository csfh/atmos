//! The Atmos install itself: XDG paths, the update channel, the Hyprland
//! opt-in, `atmosd install`, and `update check|apply`.

use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text;
use crate::hyprland::require::{self, Action};
use crate::runner::Run;
use regex_lite::Regex;
use std::path::{Path, PathBuf};

const HYPR_SEED: &str = include_str!("../../../packaging/hypr-atmos.lua");
const HYPR_LAYOUT: &str = include_str!("../../../packaging/hypr-atmos-layout.lua");
const DESKTOP: &str = include_str!("../../../packaging/atmos.desktop");
const STAGED: &[&str] = &[
    "bin",
    "components",
    "pages",
    "services",
    "packaging",
    "icons",
    "shell.qml",
];

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn xdg(var: &str, tail: &str) -> PathBuf {
    match std::env::var(var) {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(home()).join(tail),
    }
}

pub fn data_home() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share").join("atmos")
}

fn cache_home() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache").join("atmos")
}

fn config_home() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("atmos")
}

fn bin_home() -> PathBuf {
    xdg("XDG_BIN_HOME", ".local/bin")
}

fn repo() -> String {
    std::env::var("ATMOS_REPO")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "https://github.com/csfh/atmos.git".into())
}

fn hypr_dir() -> PathBuf {
    std::env::var("ATMOS_HYPR_DIR")
        .ok()
        .filter(|v| !v.is_empty())
        .map_or_else(|| PathBuf::from(home()).join(".config/hypr"), PathBuf::from)
}

/// Only `stable` exists. An old `alpha` and anything unknown read as stable.
pub fn channel() -> &'static str {
    "stable"
}

pub fn write_channel(name: &str) -> Result<()> {
    if name != "stable" {
        return Err(Error::bad_request("Usage: set-atmos-channel stable"));
    }
    atomic_write_text(&config_home().join("channel"), "stable\n")
}

pub fn setup_state() -> &'static str {
    let lua = std::fs::read_to_string(hypr_dir().join("hyprland.lua")).unwrap_or_default();
    if lua.contains("hypr.atmos") {
        "on"
    } else {
        "off"
    }
}

fn reload_hypr() {
    if crate::tasks::present("hyprctl") {
        let _ = Run::new("hyprctl").arg("reload").output();
    }
}

/// Remove `atmos` entries from Omarchy's menu extension, which is JSON with comments.
fn strip_omarchy_menu() {
    let path = PathBuf::from(home()).join(".config/omarchy/extensions/omarchy-menu.jsonc");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return;
    };
    let no_block = Regex::new(r"(?s)/\*.*?\*/")
        .map(|re| re.replace_all(&text, "").into_owned())
        .unwrap_or(text);
    let no_line = Regex::new(r"(?m)//.*?$")
        .map(|re| re.replace_all(&no_block, "").into_owned())
        .unwrap_or(no_block);
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&no_line)
    else {
        return;
    };
    let is_atmos = |v: &serde_json::Value| {
        v.get("action").and_then(|a| a.as_str()).is_some_and(|a| {
            let a = a.trim();
            a == "atmos" || a.starts_with("atmos ")
        })
    };
    let kept: serde_json::Map<String, serde_json::Value> = map
        .iter()
        .filter(|(_, v)| !is_atmos(v))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    if kept.len() == map.len() {
        return;
    }
    let body = if kept.is_empty() {
        "{}\n".to_string()
    } else {
        serde_json::to_string_pretty(&serde_json::Value::Object(kept)).unwrap_or_default() + "\n"
    };
    let _ = atomic_write_text(&path, &body);
}

/// Hyprland integration for the current user: the window seed, the layout
/// wrapper, and their requires in hyprland.lua. Idempotent.
pub fn setup_hypr() -> Result<()> {
    let hypr = hypr_dir();
    std::fs::create_dir_all(&hypr)?;
    if !hypr.join("atmos.lua").is_file() {
        std::fs::write(hypr.join("atmos.lua"), HYPR_SEED)?;
    }
    std::fs::write(hypr.join("atmos_layout.lua"), HYPR_LAYOUT)?;
    require::run(
        Action::Enable,
        &hypr.join("hyprland.lua"),
        crate::platform::packaged(),
    )?;
    strip_omarchy_menu();
    reload_hypr();
    Ok(())
}

/// Undo `setup_hypr`. atmos.lua is the user's file once edited, so it goes only
/// while it is still the seed. Sentinel blocks stay; Reset in Atmos strips those.
pub fn unsetup_hypr() -> Result<()> {
    let hypr = hypr_dir();
    require::run(
        Action::Reset,
        &hypr.join("hyprland.lua"),
        crate::platform::packaged(),
    )?;
    let _ = std::fs::remove_file(hypr.join("atmos_layout.lua"));
    if std::fs::read_to_string(hypr.join("atmos.lua")).is_ok_and(|t| t == HYPR_SEED) {
        let _ = std::fs::remove_file(hypr.join("atmos.lua"));
    }
    reload_hypr();
    Ok(())
}

/// `atmos --setup [on|off|status]`
pub fn setup(mode: &str) -> Result<()> {
    match mode {
        "on" => {
            setup_hypr()?;
            super::sampler_unit::enable(&std::env::current_exe()?)?;
            println!("Atmos is set up. Undo it with: atmos --setup off");
        }
        "off" => {
            unsetup_hypr()?;
            super::sampler_unit::disable();
            println!("Atmos is set down. Settings you changed in Atmos stay; Reset in Atmos strips them.");
        }
        "status" => println!("{}", setup_state()),
        _ => return Err(Error::bad_request("Usage: atmos --setup [on|off|status]")),
    }
    Ok(())
}

fn copy_all(src: &Path, dest: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(src)?;
    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(src)?;
        let _ = std::fs::remove_file(dest);
        std::os::unix::fs::symlink(target, dest)?;
    } else if meta.is_dir() {
        std::fs::create_dir_all(dest)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            copy_all(&entry.path(), &dest.join(entry.file_name()))?;
        }
        std::fs::set_permissions(dest, meta.permissions())?;
    } else {
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

fn build_backend(src: &Path, dest: &Path) -> Result<()> {
    if !src.join("backend/Cargo.toml").is_file() {
        return Err(bad(format!(
            "atmos: missing {}/backend/Cargo.toml",
            src.display()
        )));
    }
    if !crate::tasks::present("cargo") {
        return Err(bad("atmos: cargo is not installed"));
    }
    let out = Run::new("cargo")
        .args(["build", "--release", "--manifest-path"])
        .arg(src.join("backend/Cargo.toml"))
        .output()?;
    if !out.success() {
        return Err(bad(out.stderr_text().trim().to_string()));
    }
    std::fs::create_dir_all(dest.join("bin"))?;
    let built = src.join("backend/target/release/atmosd");
    std::fs::copy(&built, dest.join("bin/atmosd"))?;
    Ok(())
}

/// Prepare the whole app before touching the installed copy. A failed Rust
/// build must leave the existing launcher, backend and QML together.
pub fn stage(src: &Path, dest: &Path) -> Result<()> {
    if !(src.is_dir() && src.join("bin/atmos").is_file() && src.join("shell.qml").is_file()) {
        return Err(bad("atmos: not an Atmos source tree"));
    }
    let staging = std::env::temp_dir().join(format!("atmos-stage-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging)?;
    let result = (|| {
        for item in STAGED {
            copy_all(&src.join(item), &staging.join(item))?;
        }
        build_backend(src, &staging)?;
        std::fs::create_dir_all(dest)?;
        for item in STAGED {
            let target = dest.join(item);
            let _ = std::fs::remove_dir_all(&target);
            let _ = std::fs::remove_file(&target);
            copy_all(&staging.join(item), &target)?;
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&staging);
    result
}

fn write_revision(dest: &Path, sha: &str) {
    if Regex::new(r"^[0-9a-f]{4,40}$").is_ok_and(|re| re.is_match(sha)) {
        let _ = std::fs::write(dest.join("REVISION"), format!("{sha}\n"));
    }
}

fn write_desktop(dest: &Path) -> Result<()> {
    let exe = dest.join("bin/atmos");
    if !exe.is_file() {
        return Err(bad("atmos: missing the launcher"));
    }
    let out = xdg("XDG_DATA_HOME", ".local/share").join("applications/atmos.desktop");
    if !DESKTOP.contains("Exec=atmos\n") {
        return Err(bad("atmos.desktop missing Exec=atmos"));
    }
    let text = DESKTOP
        .replacen("Exec=atmos\n", &format!("Exec={}\n", exe.display()), 1)
        .replacen(
            "TryExec=atmos\n",
            &format!("TryExec={}\n", exe.display()),
            1,
        );
    atomic_write_text(&out, &text)
}

fn link_launcher(dest: &Path) -> Result<()> {
    let bin = bin_home();
    std::fs::create_dir_all(&bin)?;
    let link = bin.join("atmos");
    // A leftover file or symlink here must not be written through.
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(dest.join("bin/atmos"), &link)?;
    Ok(())
}

pub fn link_xdg(dest: &Path) -> Result<()> {
    std::fs::create_dir_all(PathBuf::from(home()).join(".local/share/applications"))?;
    std::fs::create_dir_all(PathBuf::from(home()).join(".config/hypr"))?;
    link_launcher(dest)?;
    write_desktop(dest)?;
    let apps = xdg("XDG_DATA_HOME", ".local/share").join("applications");
    let _ = Run::new("update-desktop-database").arg(apps).output();
    setup_hypr()?;
    super::sampler_unit::enable(&dest.join("bin/atmosd"))
}

fn git_head(dir: &Path) -> String {
    Run::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .map(|o| o.stdout_text().trim().to_string())
        .unwrap_or_default()
}

/// `atmosd install --source DIR`: what install.sh runs once it has built atmosd.
pub fn install(source: &Path) -> Result<()> {
    let dest = data_home();
    stage(source, &dest)?;
    write_channel("stable")?;
    let sha = if Run::new("git")
        .arg("-C")
        .arg(source)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .is_ok_and(|o| o.success())
    {
        git_head(source)
    } else {
        String::new()
    };
    write_revision(&dest, &sha);
    link_xdg(&dest)?;
    println!("Atmos is installed. Run: atmos");
    Ok(())
}

fn say(key: &str, value: &str) {
    println!("{key} {value}");
}

fn sync_cache(channel: &str) -> bool {
    let cache = cache_home().join("src");
    let repo = repo();
    if let Some(parent) = cache.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let git = |args: &[&str]| {
        Run::new("git")
            .args(args)
            .output()
            .is_ok_and(|o| o.success())
    };
    let cache_s = cache.to_string_lossy().into_owned();
    if !cache.join(".git").is_dir() {
        return git(&[
            "clone", "--depth", "1", "--branch", channel, &repo, &cache_s,
        ]);
    }
    git(&["-C", &cache_s, "remote", "set-url", "origin", &repo])
        && git(&["-C", &cache_s, "fetch", "--depth", "1", "origin", channel])
        && git(&["-C", &cache_s, "checkout", "-B", channel, "FETCH_HEAD"])
}

/// `update check|apply`: prints `key value` lines for the Software page.
pub fn update(action: &str) -> Result<i32> {
    if action != "check" && action != "apply" {
        return Err(Error::bad_request("Usage: update-atmos check|apply"));
    }
    if crate::platform::packaged() {
        say("status", "packaged");
        say(
            "summary",
            "Atmos is managed by pacman. Update it with omarchy update.",
        );
        return Ok(0);
    }
    let channel = channel();
    say("channel", channel);
    if !crate::tasks::present("git") {
        say("status", "fetch-failed");
        say("summary", "git is not installed.");
        return Ok(1);
    }
    if !sync_cache(channel) {
        say("status", "fetch-failed");
        say("summary", "Could not fetch the stable branch.");
        return Ok(1);
    }
    let cache = cache_home().join("src");
    let remote = git_head(&cache);
    let short = Run::new("git")
        .arg("-C")
        .arg(&cache)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| o.stdout_text().trim().to_string())
        .unwrap_or_default();
    say("remote", &remote);
    say("short", &short);
    let dest = data_home();
    let mut local = std::fs::read_to_string(dest.join("REVISION"))
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    if Regex::new(r"^[0-9a-f]{4,40}$").is_ok_and(|re| re.is_match(&local)) {
        say("local", &local);
    } else {
        local.clear();
    }
    let complete = dest.join("bin/atmos").is_file()
        && dest.join("bin/atmosd").is_file()
        && dest.join("shell.qml").is_file();
    if !local.is_empty() && local == remote && complete {
        say("status", "current");
        say("summary", "Atmos is up to date.");
        return Ok(0);
    }
    say("status", "behind");
    say(
        "summary",
        if local == remote {
            "Atmos installation is incomplete. Update to repair it."
        } else {
            "A newer Atmos is on stable."
        },
    );
    if action != "apply" {
        return Ok(0);
    }
    stage(&cache, &dest)?;
    write_revision(&dest, &remote);
    link_xdg(&dest)?;
    say("status", "current");
    say("short", &short);
    say(
        "summary",
        "Updated. If the window looks the same, quit Atmos and open it again.",
    );
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_stable_is_a_channel() {
        assert_eq!(channel(), "stable");
        assert!(write_channel("alpha").is_err());
    }

    #[test]
    fn the_desktop_template_has_the_lines_install_rewrites() {
        assert!(DESKTOP.contains("Exec=atmos\n"));
    }
}
