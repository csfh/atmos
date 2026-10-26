//! `atmosd launch [page]`: open an Atmos window. Every launch is its own
//! instance, with its own engine and queue. Instances converge through the
//! files they share: writers serialize on a lock and rename into place, and
//! file watchers re-read what another window wrote.

use crate::error::{Error, Result};
use std::os::unix::process::CommandExt;

const HUBS: &[&str] = &[
    "home",
    "monitor",
    "sentinel",
    "favorites",
    "appearance",
    "display",
    "windows",
    "workspaces",
    "bar",
    "notifications",
    "profiles",
    "input",
    "keybindings",
    "accessibility",
    "sound",
    "capture",
    "hardware",
    "drivers",
    "disks",
    "network",
    "bluetooth",
    "power",
    "idle",
    "defaults",
    "agentic",
    "applications",
    "software",
    "hooks",
    "system",
    "tweaks",
    "export",
    "accounts",
    "security",
    "services",
];

const USAGE: &str = "Usage: atmos [home|monitor|sentinel|favorites|appearance|display|hardware|drivers|windows|workspaces|input|keybindings|accessibility|sound|capture|disks|bar|notifications|profiles|defaults|agentic|applications|software|network|bluetooth|power|idle|security|accounts|hooks|tweaks|services|system|export][/theme|background|boot|gpu|cpu|npu|speedtest|wifi|bindings|rules|startup|environment|kernel|diagnostics|history|processes|memory|disk|traffic|sensors|mcp]";

/// Launcher words that older versions accepted, mapped to where they live now.
fn alias(page: &str) -> String {
    if page == "dashboard" || page.starts_with("dashboard/") {
        return "monitor".into();
    }
    // Machine used to be its own System page. Identity lives on Hardware.
    if page == "machine" || page == "system/machine" {
        return "hardware".into();
    }
    // Agent tools moved under Agentic.
    if page == "defaults/agents" {
        return "agentic/mcp".into();
    }
    page.into()
}

pub fn page_ok(page: &str) -> bool {
    let hub = page.split('/').next().unwrap_or("");
    page.is_empty() || HUBS.contains(&hub)
}

pub fn run(args: &[String]) -> Result<i32> {
    let root = crate::platform::app_root();
    if std::env::var("ATMOS_PRINT_ROOT").is_ok_and(|v| v == "1") {
        println!("{}", root.display());
        return Ok(0);
    }
    let first = args.first().map_or("", String::as_str);
    if first == "mcp" {
        let (mut backend, mut fixture) = (crate::platform::Backend::Omarchy, None);
        let mut it = args[1..].iter();
        while let Some(flag) = it.next() {
            match flag.as_str() {
                "--backend" => {
                    backend =
                        crate::platform::Backend::parse(it.next().ok_or("--backend needs a name")?)?
                }
                "--root" => fixture = it.next().map(std::path::PathBuf::from),
                _ => {}
            }
        }
        crate::mcp::serve(
            backend,
            fixture.as_deref(),
            std::io::stdin().lock(),
            std::io::stdout(),
        );
        return Ok(0);
    }
    // Per-user Hyprland integration, opt-in. This does not open a window.
    if first == "--setup" {
        crate::tasks::app::setup(args.get(1).map_or("on", String::as_str))?;
        return Ok(0);
    }
    if !root.join("shell.qml").is_file() {
        return Err(Error::from(format!(
            "atmos: missing {}/shell.qml",
            root.display()
        )));
    }
    if !crate::tasks::present("quickshell") {
        return Err(Error::from(
            "atmos: quickshell is not installed".to_string(),
        ));
    }
    let page = alias(first);
    if !page_ok(&page) {
        return Err(Error::from(USAGE.to_string()));
    }
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new("quickshell");
    cmd.arg("-p").arg(&root);
    if !page.is_empty() {
        cmd.env("ATMOS_PAGE", &page);
    }
    cmd.env("ATMOS_BACKEND", &exe)
        .env(
            "ATMOS_BACKEND_ID",
            std::env::var("ATMOS_BACKEND_ID").unwrap_or_else(|_| "omarchy".into()),
        )
        .env("ATMOS_ROOT", &root)
        .env("QS_APP_ID", "dev.csfh.atmos")
        // Qt logs this on every launch when xdg-desktop-portal already has a
        // connection for the desktop file name. It does not stop the window.
        .env(
            "QT_LOGGING_RULES",
            std::env::var("QT_LOGGING_RULES").unwrap_or_else(|_| "qt.qpa.services=false".into()),
        );
    // Every picker in Atmos is a native dialog, and Omarchy sets
    // QT_QPA_PLATFORMTHEME=gtk3 for the whole session. That combination
    // dlopens libgtk-3 into a quickshell linked against jemalloc, and the
    // first picker aborts the process in g_malloc, taking every page that opens
    // a picker down with it. The portal dialog runs out of process, so the
    // allocators never meet. Upstream: quickshell-mirror/quickshell#1010.
    if std::env::var("QT_QPA_PLATFORMTHEME").is_ok_and(|v| v == "gtk3") {
        cmd.env("QT_QPA_PLATFORMTHEME", "xdgdesktopportal");
    }
    // No duplicate refusal and no focus-stealing: each launch owns its window.
    // The shared QS_APP_ID keeps the Hyprland window rule matching.
    let err = cmd.exec();
    Err(Error::from(format!("atmos: quickshell: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_launcher_words_still_land_somewhere() {
        assert_eq!(alias("dashboard"), "monitor");
        assert_eq!(alias("dashboard/cpu"), "monitor");
        assert_eq!(alias("machine"), "hardware");
        assert_eq!(alias("defaults/agents"), "agentic/mcp");
        assert_eq!(alias("windows/rules"), "windows/rules");
    }

    #[test]
    fn only_known_hubs_open() {
        assert!(page_ok(""));
        assert!(page_ok("network/wifi"));
        assert!(!page_ok("nope"));
        assert!(!page_ok("../etc"));
    }
}
