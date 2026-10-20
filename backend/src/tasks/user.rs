//! Per-user tasks: presentation mode, default apps, audio, launchers, hooks,
//! favorites, process signals, the diagnostics report, and Hyprland resets.

use super::stdin_all;
use crate::error::{Error, Result};
use crate::fsutil::atomic_write_text;
use crate::hyprland::{self, Kind};
use crate::runner::Run;
use regex_lite::Regex;
use serde_json::json;
use std::path::{Path, PathBuf};

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

/// A mistake in how the task was called: exits 2.
fn usage_error(message: impl Into<String>) -> Error {
    Error::bad_request(message)
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn quiet(program: &str, args: &[&str]) -> bool {
    Run::new(program)
        .args(args)
        .output()
        .is_ok_and(|o| o.success())
}

fn text_of(program: &str, args: &[&str]) -> String {
    Run::new(program)
        .args(args)
        .output()
        .map(|o| o.stdout_text().trim_end_matches('\n').to_string())
        .unwrap_or_default()
}

/// Session Presentation Mode: stay awake, DND, screensaver off.
pub fn presentation(on_off: &str, minutes: Option<&str>) -> Result<()> {
    let state = crate::host::env_path(
        "ATMOS_PRESENTATION_FILE",
        format!("{}/.local/state/omarchy/atmos-presentation.json", home()),
    );
    let mut mins: u64 = minutes
        .filter(|m| re(r"^[0-9]+$").is_match(m))
        .and_then(|m| m.parse().ok())
        .unwrap_or(120);
    if mins < 1 {
        mins = 120;
    }
    mins = mins.min(720);
    let dnd_on = || text_of("omarchy-shell", &["notifications", "isDnd"]) == "on";
    match on_off {
        "on" | "true" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            crate::store::with_lock(&state, || {
                atomic_write_text(
                    &state,
                    &(serde_json::to_string_pretty(
                        &json!({"on": true, "until": now + mins * 60, "minutes": mins}),
                    )? + "\n"),
                )
            })?;
            quiet("omarchy", &["toggle", "idle", "stay-awake"]);
            if !dnd_on() {
                quiet("omarchy", &["toggle", "notification", "silencing"]);
            }
            if quiet("omarchy", &["toggle", "enabled", "screensaver"]) {
                quiet("omarchy", &["toggle", "screensaver-off", "on"]);
            }
            Ok(())
        }
        "off" | "false" => {
            crate::store::with_lock(&state, || {
                atomic_write_text(
                    &state,
                    &(serde_json::to_string_pretty(
                        &json!({"on": false, "until": 0, "minutes": 0}),
                    )? + "\n"),
                )
            })?;
            quiet("omarchy", &["toggle", "idle", "allow-idle"]);
            if dnd_on() {
                quiet("omarchy", &["toggle", "notification", "silencing"]);
            }
            if quiet("omarchy", &["toggle", "enabled", "screensaver-off"]) {
                quiet("omarchy", &["toggle", "screensaver-off", "off"]);
            }
            Ok(())
        }
        _ => Err(usage_error("usage: set-presentation on|off [minutes]")),
    }
}

/// Set an XDG default application for a small set of file types.
pub fn mime_default(kind: &str, desktop: &str) -> Result<()> {
    let usage = || bad("Usage: set-mime-default <pdf|image|video> <desktop-id.desktop>");
    let types: &[&str] = match kind {
        "pdf" => &["application/pdf"],
        "image" => &["image/png", "image/jpeg", "image/webp", "image/gif"],
        "video" => &["video/mp4", "video/webm", "video/x-matroska"],
        _ => return Err(usage()),
    };
    if !re(r"^[A-Za-z0-9._-]+\.desktop$").is_match(desktop) {
        return Err(usage());
    }
    let data = crate::host::env_path("XDG_DATA_HOME", format!("{}/.local/share", home()));
    let found = [
        data.join("applications"),
        "/usr/local/share/applications".into(),
        "/usr/share/applications".into(),
    ]
    .iter()
    .any(|d| d.join(desktop).is_file());
    if !found {
        return Err(bad(format!(
            "set-mime-default: desktop file not found: {desktop}"
        )));
    }
    for mime in types {
        let out = Run::new("xdg-mime")
            .args(["default", desktop, mime])
            .output()?;
        if !out.success() {
            return Err(bad(out.stderr_text().trim().to_string()));
        }
    }
    Ok(())
}

/// Absolute PipeWire volumes. `omarchy audio output volume` only steps
/// relatively; the Sound page slider needs a percent.
pub fn audio(action: &str, value: &str) -> Result<()> {
    audio_with(
        action,
        value,
        std::env::var("ATMOS_SKIP_LIVE").is_ok_and(|v| v == "1"),
    )
}

/// `skip_live` is a fixture run: validate, then leave the live session alone.
pub fn audio_with(action: &str, value: &str, skip_live: bool) -> Result<()> {
    let usage = || bad("Usage: set-audio output-volume <0-100> | input-volume <0-100>");
    if !re(r"^[0-9]+$").is_match(value) || !value.parse::<u32>().is_ok_and(|v| v <= 100) {
        return Err(usage());
    }
    if skip_live {
        return Ok(());
    }
    let pct = format!("{value}%");
    let run = |program: &str, args: &[&str]| -> Result<()> {
        let out = Run::new(program).args(args).output()?;
        if out.success() {
            Ok(())
        } else {
            Err(bad(out.stderr_text().trim().to_string()))
        }
    };
    match action {
        "output-volume" => {
            let sink = text_of("omarchy", &["audio", "output", "sink"]);
            if sink.is_empty() {
                run("wpctl", &["set-volume", "@DEFAULT_AUDIO_SINK@", &pct])?;
                run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SINK@", "0"])
            } else {
                run("pactl", &["set-sink-mute", &sink, "0"])?;
                run("pactl", &["set-sink-volume", &sink, &pct])
            }
        }
        "input-volume" => {
            run("wpctl", &["set-volume", "@DEFAULT_AUDIO_SOURCE@", &pct])?;
            run("wpctl", &["set-mute", "@DEFAULT_AUDIO_SOURCE@", "0"])
        }
        _ => Err(usage()),
    }
}

fn desktop_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\t', "\\t")
}

/// Write a user desktop launcher. The page already checked the name and command.
pub fn add_desktop_launcher(name: &str, command: &str, icon: &str) -> Result<()> {
    let usage = || bad("Usage: add-desktop-launcher <name> <command> [icon]");
    let icon = if icon.is_empty() {
        "application-x-executable"
    } else {
        icon
    };
    let line_free = |s: &str| !s.contains('\n') && !s.contains('\r');
    if name.is_empty()
        || command.is_empty()
        || name.starts_with('-')
        || name.contains('/')
        || !line_free(name)
        || !line_free(command)
        || !line_free(icon)
        || name.chars().count() > 80
    {
        return Err(usage());
    }
    let dir = crate::host::env_path("XDG_DATA_HOME", format!("{}/.local/share", home()))
        .join("applications");
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{name}.desktop"));
    let body = format!(
        "[Desktop Entry]\nVersion=1.0\nName={n}\nComment={n}\nExec={c}\nTerminal=false\nType=Application\nIcon={i}\nStartupNotify=true\n",
        n = desktop_escape(name),
        c = desktop_escape(command),
        i = desktop_escape(icon)
    );
    std::fs::write(&file, body)?;
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&file)?.permissions().mode() | 0o111;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(mode))?;
    }
    quiet("update-desktop-database", &[&dir.to_string_lossy()]);
    Ok(())
}

/// Where the Favorites list lives.
pub fn favorites_file() -> std::path::PathBuf {
    crate::host::env_path(
        "ATMOS_FAVORITES_FILE",
        format!("{}/.local/state/omarchy/atmos-favorites.json", home()),
    )
}

/// Write the Favorites list. UI chrome, not Omarchy settings.
pub fn favorites_write(raw: &str) -> Result<()> {
    let file = favorites_file();
    let data: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| usage_error("usage: set-favorites write <json>"))?;
    let items = match &data {
        serde_json::Value::Object(map) => map.get("items").cloned(),
        other => Some(other.clone()),
    };
    let Some(items @ serde_json::Value::Array(_)) = items else {
        return Err(usage_error("usage: set-favorites write <json>"));
    };
    atomic_write_text(
        &file,
        &(serde_json::to_string_pretty(&json!({"items": items}))? + "\n"),
    )
}

const HOOK_TYPES: &[&str] = &[
    "theme-set",
    "font-set",
    "post-boot",
    "post-update",
    "pre-refresh-pacman",
    "battery-low",
];

/// Write a one-line hook script and install it with `omarchy hook install`.
pub fn create_hook(kind: &str, name: &str, command: &str) -> Result<()> {
    let usage = || bad("Usage: create-hook <type> <name> <command>");
    if kind.starts_with('-') || name.starts_with('-') {
        return Err(usage());
    }
    if !(HOOK_TYPES.contains(&kind) || re(r"^[a-z0-9][a-z0-9-]*$").is_match(kind)) {
        return Err(bad("create-hook: unknown hook type"));
    }
    if name.ends_with(".sample")
        || name.contains('/')
        || name.contains("..")
        || !re(r"^[A-Za-z0-9][A-Za-z0-9._+-]*$").is_match(name)
    {
        return Err(bad("create-hook: bad hook name"));
    }
    let name = if re(r"\.[A-Za-z0-9]+$").is_match(name) {
        name.to_string()
    } else {
        format!("{name}.sh")
    };
    if command.is_empty() || command.len() > 512 || command.contains('\n') || command.contains('\r')
    {
        return Err(bad("create-hook: bad command"));
    }
    let comment = match kind {
        "theme-set" => "# $1 is the snake-cased theme name that was just set.",
        "font-set" => "# $1 is the snake-cased font name that was just set.",
        "battery-low" => "# $1 is the current battery percentage.",
        _ => "# This event has no extra argument.",
    };
    let dir = std::env::temp_dir().join(format!("atmos-hook-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(&name);
    std::fs::write(&file, format!("#!/bin/bash\n{comment}\n{command}\n"))?;
    let out = Run::new("omarchy")
        .args(["hook", "install", kind])
        .arg(&file)
        .output();
    let _ = std::fs::remove_dir_all(&dir);
    let out = out?;
    print!("{}", out.stdout_text());
    if out.success() {
        Ok(())
    } else {
        Err(bad(out.stderr_text().trim().to_string()))
    }
}

/// Enable or disable a hook in ~/.config/omarchy/hooks by renaming `.sample`.
pub fn hook_sample(action: &str, src: &str) -> Result<()> {
    let usage = || bad("Usage: set-hook-sample enable|disable <path>");
    if action != "enable" && action != "disable" {
        return Err(usage());
    }
    if !src.starts_with('/') || src.contains("..") {
        return Err(bad(
            "set-hook-sample: path must be absolute and stay in the hooks folder",
        ));
    }
    let root = format!("{}/.config/omarchy/hooks/", home());
    let Some(rel) = src.strip_prefix(&root) else {
        return Err(bad(
            "set-hook-sample: path must be under ~/.config/omarchy/hooks",
        ));
    };
    if !rel.contains(".d/") || rel.matches('/').count() != 1 {
        return Err(bad(
            "set-hook-sample: only scripts in a <type>.d folder can be renamed",
        ));
    }
    if !Path::new(src).is_file() {
        return Err(bad("set-hook-sample: file not found"));
    }
    let dest = if action == "enable" {
        match src.strip_suffix(".sample") {
            Some(d) => d.to_string(),
            None => return Err(bad("set-hook-sample: already enabled")),
        }
    } else if src.ends_with(".sample") {
        return Err(bad("set-hook-sample: already a sample"));
    } else {
        format!("{src}.sample")
    };
    if Path::new(&dest).exists() {
        return Err(bad(format!("set-hook-sample: {dest} already exists")));
    }
    std::fs::rename(src, dest)?;
    Ok(())
}

/// Signal a process owned by this user. TERM or KILL only. No sudo.
pub fn signal_process(pid: &str, sig: &str) -> Result<()> {
    if !re(r"^[1-9][0-9]*$").is_match(pid) {
        return Err(bad("signal-process: pid must be a positive integer"));
    }
    let pid_n: u32 = pid
        .parse()
        .map_err(|_| bad("signal-process: pid must be a positive integer"))?;
    if pid_n == 1 {
        return Err(bad("signal-process: refusing pid 1"));
    }
    let sig = match sig {
        "TERM" | "SIGTERM" | "15" => "TERM",
        "KILL" | "SIGKILL" | "9" => "KILL",
        _ => return Err(bad("signal-process: signal must be TERM or KILL")),
    };
    let own = std::process::id();
    let parent = std::os::unix::process::parent_id();
    let atmos_self = std::env::var("ATMOS_SELF_PID")
        .ok()
        .and_then(|v| v.parse::<u32>().ok());
    if pid_n == own || pid_n == parent || atmos_self == Some(pid_n) {
        return Err(bad("signal-process: refusing to signal Atmos"));
    }
    let sys = std::env::var("ATMOS_SYS_ROOT")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/".into());
    let status = std::fs::read_to_string(Path::new(&sys).join(format!("proc/{pid}/status")))
        .map_err(|_| bad(format!("signal-process: no such process {pid}")))?;
    // Uid: is real, effective, saved, fs. Ownership for signalling is the real
    // and saved pair, not the effective one: a setuid helper the user launched
    // (fusermount3, passwd, sudo) runs with real=them, effective=0.
    let uids: Vec<&str> = status
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .map(|l| l.split_whitespace().collect())
        .unwrap_or_default();
    let me = std::env::var("ATMOS_UID")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            use std::os::unix::fs::MetadataExt;
            std::fs::metadata("/proc/self").map_or(String::new(), |m| m.uid().to_string())
        });
    let (real, saved) = (
        uids.get(1).copied().unwrap_or(""),
        uids.get(3).copied().unwrap_or(""),
    );
    if real.is_empty() || (real != me && saved != me) {
        return Err(bad(format!(
            "signal-process: pid {pid} is not owned by this user"
        )));
    }
    let lock = crate::host::env_path(
        "XDG_RUNTIME_DIR",
        std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into()),
    )
    .join("atmos-signal.lock");
    crate::store::with_lock(&lock, || {
        if std::env::var("ATMOS_SIGNAL_DRY").is_ok_and(|v| v == "1") {
            println!("kill -s {sig} {pid}");
            return Ok(());
        }
        let out = Run::new("kill").args(["-s", sig, pid]).output()?;
        if out.success() {
            Ok(())
        } else {
            Err(bad(out.stderr_text().trim().to_string()))
        }
    })
}

/// Write the Atmos diagnostic report, then copy, save, or hand it to an agent.
/// stdin is the Atmos header from Diagnostics.reportText.
pub fn diag_report(mode: &str, dest: &str) -> Result<()> {
    match mode {
        "copy" | "agent" => {}
        "save" => {
            if dest.is_empty()
                || dest.contains('\n')
                || dest.contains('\r')
                || !dest.starts_with('/')
            {
                return Err(bad("diag-report: save needs an absolute path"));
            }
        }
        _ => return Err(usage_error("usage: diag-report copy|save|agent [path]")),
    }
    let cache = crate::host::env_path("XDG_CACHE_HOME", format!("{}/.cache", home())).join("atmos");
    std::fs::create_dir_all(&cache)?;
    let file = cache.join("diagnostic-report.txt");
    let header = String::from_utf8_lossy(&stdin_all()).into_owned();
    let mut body = header.clone();
    if !header.ends_with('\n') {
        body.push('\n');
    }
    if super::present("omarchy-debug") {
        body.push_str("\n== omarchy debug ==\n\n");
        if let Ok(out) = Run::new("omarchy-debug")
            .args(["--no-sudo", "--print"])
            .output()
        {
            body.push_str(&out.stdout_text());
        }
    }
    std::fs::write(&file, &body)?;
    let copy = || {
        if super::present("wl-copy") {
            let _ = Run::new("wl-copy").arg("-n").input(body.clone()).output();
        }
    };
    match mode {
        "copy" => {
            copy();
            println!("{}", file.display());
        }
        "save" => {
            std::fs::copy(&file, dest)?;
            println!("{dest}");
        }
        _ => {
            copy();
            if super::present("omarchy") {
                let prompt = format!(
                    "A diagnostic report from Atmos is on the clipboard and in {}. Read that file and diagnose this Omarchy machine.",
                    file.display()
                );
                let _ = Run::new("omarchy")
                    .args(["agent", "prompt", &prompt])
                    .detach();
            }
            println!("{}", file.display());
        }
    }
    Ok(())
}

fn hypr_file(var: &str, name: &str) -> PathBuf {
    crate::host::env_path(var, format!("{}/.config/hypr/{name}", home()))
}

/// Restore Hyprland Lua defaults, then put back the drop-in require.
pub fn refresh_hyprland() -> Result<()> {
    let out = Run::new("omarchy").args(["refresh", "hyprland"]).output()?;
    print!("{}", out.stdout_text());
    if !out.success() {
        return Err(bad(out.stderr_text().trim().to_string()));
    }
    hyprland::require::run(
        hyprland::require::Action::Apply,
        &hypr_file("ATMOS_HYPRLAND_FILE", "hyprland.lua"),
        crate::platform::packaged(),
    )
}

/// `hyprctl reload`, then report what Hyprland says about the config.
pub fn hypr_reload(label: &str, check_errors: bool) -> Result<()> {
    if std::env::var("ATMOS_SKIP_HYPR").is_ok_and(|v| v == "1") || !super::present("hyprctl") {
        return Ok(());
    }
    quiet("hyprctl", &["reload"]);
    if check_errors {
        let errors = text_of("hyprctl", &["configerrors"]);
        if !errors.is_empty() && errors != "nothing" {
            return Err(bad(format!("{label}: hyprctl configerrors:\n{errors}")));
        }
    }
    Ok(())
}

/// `default_name` only applies when Hyprland creates a workspace. Persistent
/// 1–N already exist, so rename them or the bar keeps showing the number.
pub fn rename_workspaces(items: &serde_json::Value) {
    if std::env::var("ATMOS_SKIP_HYPR").is_ok_and(|v| v == "1") || !super::present("hyprctl") {
        return;
    }
    let Some(items) = items.as_array() else {
        return;
    };
    for item in items {
        let text = |key: &str| {
            item.get(key)
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .unwrap_or("")
        };
        let id = text("id");
        if id.is_empty() || id.starts_with("special:") {
            continue;
        }
        let name = if text("name").is_empty() {
            id
        } else {
            text("name")
        };
        let code = format!(
            "hl.dispatch(hl.dsp.workspace.rename({{ workspace = {}, name = {} }}))",
            hyprland::lua::lua_string(id),
            hyprland::lua::lua_string(name)
        );
        quiet("hyprctl", &["eval", &code]);
    }
}

/// Apply a `{_patch, _full}` (or whole-block) payload to the look or input block, then reload.
pub fn hypr_apply(kind: &str, json: &str) -> Result<()> {
    let (kind, file) = match kind {
        "look" => (Kind::Look, hypr_file("ATMOS_LOOK_FILE", "looknfeel.lua")),
        "input" => (Kind::Input, hypr_file("ATMOS_INPUT_FILE", "input.lua")),
        _ => return Err(usage_error("usage: hypr-apply look|input <json>")),
    };
    let payload: serde_json::Value = serde_json::from_str(json)
        .map_err(|_| usage_error("usage: hypr-apply look|input <json>"))?;
    hyprland::apply_file(kind, &file, &payload, false)?;
    hypr_reload("hypr-apply", false)
}

/// Strip one managed block (look or input) and reload.
pub fn reset_block(kind: &str) -> Result<()> {
    let (kind, file) = match kind {
        "look" => (Kind::Look, hypr_file("ATMOS_LOOK_FILE", "looknfeel.lua")),
        "input" => (Kind::Input, hypr_file("ATMOS_INPUT_FILE", "input.lua")),
        _ => return Err(bad("usage: reset-hypr look|input")),
    };
    hyprland::apply_file(kind, &file, &serde_json::Value::Null, true)?;
    hypr_reload("reset-hypr", false)
}

/// Strip Atmos-managed Hyprland sentinels and drop the search index cache.
/// Keep the Atmos window seed and require.
pub fn reset_atmos() -> Result<()> {
    let windows = hypr_file("ATMOS_WINDOWS_FILE", "atmos.lua");
    for (kind, file) in [
        (Kind::Look, hypr_file("ATMOS_LOOK_FILE", "looknfeel.lua")),
        (Kind::Input, hypr_file("ATMOS_INPUT_FILE", "input.lua")),
        (
            Kind::Autostart,
            hypr_file("ATMOS_AUTOSTART_FILE", "autostart.lua"),
        ),
        (
            Kind::Bindings,
            hypr_file("ATMOS_BINDINGS_FILE", "bindings.lua"),
        ),
        (Kind::Windows, windows.clone()),
    ] {
        hyprland::apply_file(kind, &file, &serde_json::Value::Null, true)?;
    }
    if crate::platform::packaged() && super::app::setup_state() == "off" {
        return hypr_reload("reset-atmos", true);
    }
    const SEED: &str = include_str!("../../../packaging/hypr-atmos.lua");
    let text = std::fs::read_to_string(&windows).unwrap_or_default();
    if !text.contains(r#"o.window("dev.csfh.atmos""#) {
        let mut next = text;
        next.push_str(SEED);
        atmos_write(&windows, &next)?;
    }
    let layout = windows
        .parent()
        .unwrap_or(Path::new("."))
        .join("atmos_layout.lua");
    atmos_write(
        &layout,
        include_str!("../../../packaging/hypr-atmos-layout.lua"),
    )?;
    hyprland::require::run(
        hyprland::require::Action::Apply,
        &hypr_file("ATMOS_HYPRLAND_FILE", "hyprland.lua"),
        crate::platform::packaged(),
    )?;
    hypr_reload("reset-atmos", true)
}

/// Run a command whose `gum confirm` prompts the window already answered: put a
/// `gum` that says yes ahead of the real one on PATH. The stub is this binary
/// under another name (see `gum_stub`), linked into a private directory.
pub fn gum_job(args: &[String]) -> Result<i32> {
    let Some((program, rest)) = args.split_first() else {
        return Err(bad("Usage: gum-job <command> [args...]"));
    };
    let base = crate::host::env_path(
        "XDG_RUNTIME_DIR",
        std::env::temp_dir().to_string_lossy().into_owned(),
    );
    let dir = base.join(format!("atmos-gum-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let exe = std::env::current_exe()?;
    std::os::unix::fs::symlink(&exe, dir.join("gum"))?;
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let status = std::process::Command::new(program)
        .args(rest)
        .env("PATH", path)
        .status();
    let _ = std::fs::remove_dir_all(&dir);
    Ok(status
        .map_err(|err| bad(format!("{program}: {err}")))?
        .code()
        .unwrap_or(1))
}

/// Invoked as `gum`: confirm prompts are answered yes, everything else goes to the real gum.
pub fn gum_stub(args: &[String]) -> ! {
    use std::os::unix::process::CommandExt;
    if args.first().map(String::as_str) == Some("confirm") {
        std::process::exit(0);
    }
    let err = std::process::Command::new("/usr/bin/gum").args(args).exec();
    eprintln!("gum: {err}");
    std::process::exit(127);
}

fn atmos_write(path: &Path, text: &str) -> Result<()> {
    atomic_write_text(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_fields_are_escaped_for_desktop_files() {
        assert_eq!(desktop_escape("a\\b\tc"), "a\\\\b\\tc");
    }

    #[test]
    fn hooks_and_signals_are_checked_before_they_run() {
        assert!(create_hook("-x", "n", "c").is_err());
        assert!(create_hook("post-boot", "a/b", "c").is_err());
        assert!(create_hook("post-boot", "n", "line\nbreak").is_err());
        assert!(hook_sample("enable", "relative").is_err());
        assert!(signal_process("1", "TERM").is_err());
        assert!(signal_process("12", "HUP").is_err());
        assert!(signal_process("abc", "TERM").is_err());
    }

    #[test]
    fn audio_values_are_percentages() {
        assert!(audio("output-volume", "101").is_err());
        assert!(audio("output-volume", "x").is_err());
    }
}
