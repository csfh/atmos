//! Every command, launcher, job and interactive tool the window can start,
//! by name. Each is the command line the window used to build itself, with the
//! parts that vary named and checked (see `table`).

use super::table::Kind::{Command, Interactive, Job, Launch};
use super::table::{Entry, In, A, T};
use T::{Rest, Task, B, F, J, L, O, V, Z};

use crate::snapshot::topics::{ACCOUNTS, HOST, LOOK, NETWORK};

pub static ENTRIES: &[Entry] = &[
    // ---- short commands -------------------------------------------------
    Entry::new("theme.refresh", Command, &[L("omarchy"), L("theme"), L("refresh")]).lane("theme").affects(&[LOOK]),
    Entry::new("theme.bg-next", Command, &[L("omarchy"), L("theme"), L("bg"), L("next")]).lane("theme").affects(&[LOOK]),
    Entry::new("theme.bg-cache", Command, &[L("omarchy"), L("theme"), L("bg"), L("cache")]).lane("theme"),
    Entry::new(
        "keyboard.backlight-restore",
        Command,
        &[L("omarchy"), L("brightness"), L("keyboard"), L("--no-osd"), L("restore")],
    )
    .lane("keyboard"),
    Entry::new(
        "account.member",
        Command,
        &[Task("manage-account"), L("set-member"), V("group", A::Ident), V("user", A::Ident), B("on", "on", "off")],
    )
    .lane("accounts")
    .affects(&[ACCOUNTS])
    .root(),
    Entry::new(
        "window.transparency-toggle",
        Command,
        &[L("omarchy"), L("hyprland"), L("window"), L("transparency"), L("toggle")],
    )
    .lane("hypr:config"),
    Entry::new(
        "window.tiled-fullscreen-toggle",
        Command,
        &[L("omarchy"), L("hyprland"), L("window"), L("tiled"), L("fullscreen"), L("toggle")],
    )
    .lane("hypr:config"),
    Entry::new(
        "wifi.password-copy",
        Command,
        &[L("bash"), L("-c"), L("omarchy network password \"$1\" | wl-copy -n"), L("wifi-password"), V("iface", A::Ident)],
    ),
    Entry::new("clipboard.copy", Command, &[L("wl-copy"), L("-n")]).stdin(In::Field("text")),
    Entry::new(
        "wifi.metered",
        Command,
        &[Task("set-wifi-connection"), L("metered"), V("uuid", A::Uuid), V("value", A::Is(&["yes", "no", "unknown"]))],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new(
        "wifi.priority",
        Command,
        &[Task("set-wifi-connection"), L("priority"), V("uuid", A::Uuid), V("value", A::Uint(0, 999))],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new(
        "wifi.mac",
        Command,
        &[
            Task("set-wifi-connection"),
            L("mac"),
            V("uuid", A::Uuid),
            V("value", A::Is(&["default", "random", "stable", "permanent", "preserve"])),
        ],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new(
        "wifi.ipv4-auto",
        Command,
        &[Task("set-wifi-connection"), L("ipv4"), V("uuid", A::Uuid), L("auto")],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new(
        "wifi.ipv4-manual",
        Command,
        &[
            Task("set-wifi-connection"),
            L("ipv4"),
            V("uuid", A::Uuid),
            L("manual"),
            V("address", A::Word),
            V("prefix", A::Uint(1, 32)),
            Z("gateway", A::Word),
            O("dns", A::Text),
        ],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new(
        "wifi.wireguard-import",
        Command,
        &[Task("set-wifi-connection"), L("wireguard-import"), V("path", A::Path)],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new(
        "wifi.hotspot-on",
        Command,
        &[Task("set-wifi-connection"), L("hotspot"), L("on"), V("ssid", A::Text)],
    )
    .lane("network")
    .stdin(In::Line("password"))
    .affects(&[NETWORK]),
    Entry::new("wifi.hotspot-off", Command, &[Task("set-wifi-connection"), L("hotspot"), L("off")]).lane("network")
    .affects(&[NETWORK]),
    Entry::new("favorites.write", Command, &[Task("set-favorites"), L("write"), V("json", A::Json)])
        .lane("favorites")
        .readback(&["favorites"]),
    Entry::new(
        "process.signal",
        Command,
        &[Task("signal-process"), V("pid", A::Uint(2, 4_194_304)), V("signal", A::Is(&["TERM", "KILL"]))],
    ),
    Entry::new(
        "systemd.user",
        Command,
        &[
            L("systemctl"),
            L("--user"),
            V("action", A::Is(&["start", "stop", "restart", "enable", "disable"])),
            V("unit", A::Ident),
        ],
    )
    .lane("systemd")
    .affects(&["services"]),
    Entry::new(
        "systemd.system",
        Command,
        &[
            L("sudo"),
            L("-n"),
            L("systemctl"),
            V("action", A::Is(&["start", "stop", "restart", "enable", "disable"])),
            V("unit", A::Ident),
        ],
    )
    .lane("systemd")
    .affects(&["services"])
    .root(),
    Entry::new(
        "wifi.ssid",
        Command,
        &[Task("set-wifi-connection"), V("action", A::Is(&["down-ssid", "delete-ssid"])), V("ssid", A::Text)],
    )
    .lane("network")
    .affects(&[NETWORK]),
    Entry::new("restart", Command, &[L("omarchy"), L("restart"), V("what", A::Is(&["wifi", "bluetooth", "audio", "shell"]))])
    .affects(&[NETWORK]),
    Entry::new(
        "bluetooth.pair",
        Command,
        &[L("omarchy"), L("bluetooth"), L("device"), L("pair"), V("address", A::Addr)],
    )
    .lane("bluetooth")
    .affects(&[NETWORK]),
    Entry::new("bluetooth.trust", Command, &[L("bluetoothctl"), L("trust"), V("address", A::Addr)]).lane("bluetooth")
    .affects(&[NETWORK]),
    Entry::new("audio.output-switch", Command, &[L("omarchy"), L("audio"), L("output"), L("switch")]).lane("audio")
    .affects(&[NETWORK]),
    Entry::new("notification", Command, &[L("omarchy"), L("notification"), V("kind", A::Is(&["battery", "time", "weather"]))]),
    Entry::new(
        "notification.test",
        Command,
        &[L("omarchy"), L("notification"), L("send"), L("Atmos"), L("This is a test toast.")],
    ),
    Entry::new("reminder.show", Command, &[L("omarchy"), L("reminder"), L("show")]),
    Entry::new(
        "webcam.resize",
        Command,
        &[
            L("omarchy"),
            L("capture"),
            L("webcam"),
            L("resize"),
            V("action", A::Is(&["smaller", "larger", "reset", "small", "medium", "large"])),
        ],
    ),
    Entry::new("share.clipboard", Command, &[L("omarchy"), L("share"), L("clipboard")]),
    Entry::new("share.file", Command, &[L("omarchy"), L("share"), L("file"), V("path", A::Path)]),
    Entry::new("share.folder", Command, &[L("omarchy"), L("share"), L("folder"), V("path", A::Path)]),
    Entry::new("hook.run", Command, &[L("omarchy"), L("hook"), V("name", A::Ident), O("arg", A::Text)]),
    // ---- launchers: something that stays open, as a unit of its own ------
    Entry::new(
        "theme.folder",
        Launch,
        &[
            L("bash"),
            L("-c"),
            L("dir=$(omarchy theme dir \"$1\") && [[ -d \"$dir\" ]] && xdg-open \"$dir\" >/dev/null 2>&1 &"),
            L("theme-dir"),
            V("theme", A::Ident),
        ],
    ),
    Entry::new(
        "agent.prompt",
        Launch,
        &[L("bash"), L("-c"), L("omarchy agent prompt \"$1\" >/dev/null 2>&1 &"), L("agent-prompt"), V("text", A::Text)],
    ),
    Entry::new(
        "hook.folder",
        Launch,
        &[
            L("bash"),
            L("-c"),
            L("dir=\"$HOME/.config/omarchy/hooks/$1.d\"; mkdir -p \"$dir\"; xdg-open \"$dir\" >/dev/null 2>&1 &"),
            L("prefs-hooks"),
            V("type", A::Ident),
        ],
    ),
    Entry::new("theme.bg-folder", Launch, &[L("omarchy"), L("theme"), L("bg"), L("install")]),
    Entry::new("aether", Launch, &[L("aether")]),
    Entry::new("open.path", Launch, &[L("xdg-open"), V("path", A::Path)]),
    Entry::new("cups.admin", Launch, &[L("xdg-open"), L("http://127.0.0.1:631")]),
    Entry::new("printers.setup", Launch, &[L("system-config-printer")]),
    Entry::new("terminal.herdr", Launch, &[L("omarchy"), L("launch"), L("terminal"), L("herdr")]),
    // ---- jobs: they stream output and may read input ---------------------
    Entry::new("theme.install", Job, &[L("omarchy"), L("theme"), L("install"), V("url", A::Url)]).lane("theme"),
    Entry::new("theme.update", Job, &[L("omarchy"), L("theme"), L("update")]).lane("theme"),
    Entry::new(
        "app.desktop-install",
        Job,
        &[Task("add-desktop-launcher"), V("name", A::Label), V("command", A::Text), V("icon", A::Text)],
    )
    .lane("apps"),
    Entry::new(
        "app.tui-install",
        Job,
        &[
            L("omarchy"),
            L("tui"),
            L("install"),
            V("name", A::Label),
            V("command", A::Text),
            V("style", A::Is(&["float", "tile"])),
            V("icon", A::Text),
        ],
    )
    .lane("apps"),
    Entry::new(
        "app.webapp-install",
        Job,
        &[L("omarchy"), L("webapp"), L("install"), V("name", A::Label), V("url", A::Text), Z("icon", A::Text)],
    )
    .lane("apps"),
    Entry::new("dns.custom", Job, &[Task("set-dns-custom"), V("servers", A::Text)]).lane("network"),
    Entry::new(
        "account.add-user",
        Job,
        &[Task("manage-account"), L("add-user"), V("name", A::Ident), V("full", A::Text), B("wheel", "true", "false")],
    )
    .lane("accounts")
    .stdin(In::Line("password"))
    .root(),
    Entry::new("account.remove-user", Job, &[Task("manage-account"), L("remove-user"), V("name", A::Ident)])
        .lane("accounts")
        .root(),
    Entry::new("account.set-password", Job, &[Task("manage-account"), L("set-password"), V("name", A::Ident)])
        .lane("accounts")
        .stdin(In::Line("password"))
        .root(),
    Entry::new("account.add-group", Job, &[Task("manage-account"), L("add-group"), V("name", A::Ident)])
        .lane("accounts")
        .root(),
    Entry::new("account.remove-group", Job, &[Task("manage-account"), L("remove-group"), V("name", A::Ident)])
        .lane("accounts")
        .root(),
    Entry::new("sshd.disable", Job, &[Task("set-sshd"), L("disable")]).lane("security").root(),
    Entry::new(
        "sudo.enable",
        Job,
        &[Task("set-passwordless-sudo"), L("on"), V("minutes", A::Uint(1, 240)), L("--sudo-ask")],
    )
    .lane("security")
    .stdin(In::Line("password")),
    Entry::new("sudo.disable", Job, &[Task("set-passwordless-sudo"), L("off")]).lane("security").root(),
    Entry::new("update.omarchy-check", Job, &[L("omarchy"), L("update"), L("available")]),
    Entry::new("update.atmos-check", Job, &[Task("update-atmos"), L("check")]),
    Entry::new("update.atmos-apply", Job, &[Task("update-atmos"), L("apply")]).lane("atmos"),
    Entry::new("wifi.qr", Job, &[L("omarchy"), L("network"), L("qr"), L("--meta"), O("iface", A::Ident)]),
    Entry::new(
        "diag.report",
        Job,
        &[Task("diag-report"), V("action", A::Is(&["copy", "save", "agent"])), O("dest", A::Path)],
    )
    .stdin(In::Field("header")),
    Entry::new("wifi.enterprise", Job, &[Task("enterprise-wifi-connect"), V("ssid", A::Text), V("identity", A::Text)])
        .lane("network")
        .stdin(In::Line("password")),
    Entry::new("wifi.join", Job, &[Task("set-wifi-connection"), L("join"), V("ssid", A::Text)])
        .lane("network")
        .stdin(In::LineOpt("password")),
    Entry::new("luks.change-key", Job, &[Task("luks-change-key")])
        .lane("disks")
        .stdin(In::Lines(&["device", "current", "new"]))
        .root(),
    Entry::new("snapshot.create", Job, &[L("omarchy"), L("snapshot"), L("create")]).lane("disks").root(),
    Entry::new(
        "snapshot.rollback",
        Job,
        &[Task("rollback-snapshot"), V("config", A::Ident), V("id", A::Ident)],
    )
    .lane("disks")
    .root(),
    Entry::new("hibernation.setup", Job, &[L("omarchy"), L("hibernation"), L("setup"), L("--force")]).lane("disks").root(),
    Entry::new("hibernation.remove", Job, &[L("omarchy"), L("hibernation"), L("remove")]).lane("disks").gum().root(),
    Entry::new("refresh.hyprland", Job, &[Task("refresh-hyprland")]).lane("hypr:config"),
    Entry::new("refresh.shell", Job, &[L("omarchy"), L("refresh"), L("shell")]),
    Entry::new("reset.atmos", Job, &[Task("reset-atmos")]).lane("atmos"),
    Entry::new("plymouth.reset", Job, &[L("omarchy"), L("plymouth"), L("reset")]).lane("boot").root(),
    Entry::new(
        "plymouth.set",
        Job,
        &[
            L("bash"),
            L("-c"),
            L("bg=$(omarchy theme color background); text=$(omarchy theme color foreground); omarchy plymouth set \"$bg\" \"$text\" \"$1\""),
            L("plymouth-set"),
            V("path", A::Path),
        ],
    )
    .lane("boot")
    .root(),
    Entry::new(
        "plymouth.preview",
        Job,
        &[
            L("bash"),
            L("-c"),
            L("bg=$(omarchy theme color background); text=$(omarchy theme color foreground); out=$(mktemp --suffix=.png); omarchy plymouth preview \"$bg\" \"$text\" \"$1\" \"$out\"; echo \"$out\""),
            L("plymouth-preview"),
            V("path", A::Path),
        ],
    )
    .lane("boot"),
    Entry::new("settings.apply", Job, &[Task("settings-apply"), L("--progress")])
        .lane("settings")
        .stdin(In::Field("plan")),
    Entry::new("settings.undo", Job, &[Task("settings-apply"), L("--no-backup"), L("--plan"), V("plan", A::Path)])
        .lane("settings"),
    // ---- jobs that show their progress (gum) and need root ---------------
    Entry::new("security.fingerprint-setup", Job, &[L("omarchy"), L("setup"), L("security"), L("fingerprint")]).lane("security").gum().root(),
    Entry::new("security.fingerprint-remove", Job, &[L("omarchy"), L("remove"), L("security"), L("fingerprint")]).lane("security").gum().root(),
    Entry::new("security.fido2-setup", Job, &[L("omarchy"), L("setup"), L("security"), L("fido2")]).lane("security").gum().root(),
    Entry::new("security.fido2-remove", Job, &[L("omarchy"), L("remove"), L("security"), L("fido2")]).lane("security").gum().root(),
    Entry::new("security.sshd-setup", Job, &[L("omarchy"), L("setup"), L("security"), L("sshd"), J("--key=", "key", A::Text)]).lane("security").gum().root(),
    Entry::new("security.docker-setup", Job, &[L("omarchy"), L("setup"), L("security"), L("sudoless"), L("docker")]).lane("security").gum().root(),
    Entry::new("security.docker-remove", Job, &[L("omarchy"), L("remove"), L("security"), L("sudoless"), L("docker")]).lane("security").gum().root(),
    Entry::new("channel.set", Job, &[L("omarchy"), L("channel"), L("set"), V("name", A::Is(&["stable", "rc", "edge", "dev"]))]).lane("pacman").gum().root(),
    Entry::new("update.omarchy", Job, &[L("omarchy"), L("update")]).lane("pacman").gum().root(),
    Entry::new("update.firmware", Job, &[L("omarchy"), L("update"), L("firmware")]).lane("pacman").gum().root(),
    Entry::new("update.orphans", Job, &[L("omarchy"), L("update"), L("orphan"), L("pkgs")]).lane("pacman").gum().root(),
    Entry::new("update.prune", Job, &[L("omarchy"), L("update"), L("pkg"), L("prune")]).lane("pacman").gum().root(),
    Entry::new("voxtype.install", Job, &[L("omarchy"), L("voxtype"), L("install")]).lane("pacman").gum().root(),
    Entry::new("voxtype.remove", Job, &[L("omarchy"), L("voxtype"), L("remove")]).lane("pacman").gum().root(),
    Entry::new("hybrid-gpu.toggle", Job, &[L("omarchy"), L("toggle"), L("hybrid"), L("gpu")]).lane("boot").gum().root(),
    Entry::new("tailscale.install", Job, &[L("omarchy"), L("install"), L("service"), L("tailscale")]).lane("pacman").gum().root(),
    Entry::new("tailscale.remove", Job, &[L("omarchy"), L("remove"), L("service"), L("tailscale")]).lane("pacman").gum().root(),
    Entry::new("direct-boot.setup", Job, &[L("omarchy"), L("setup"), L("direct"), L("boot")]).lane("boot").gum().root(),
    Entry::new(
        "software.run",
        Job,
        &[L("omarchy"), V("action", A::Is(&["install", "remove"])), Rest("words", A::Ident, 2, 3)],
    )
    .lane("pacman")
    .gum()
    .root(),
    Entry::new("dev-env.install", Job, &[L("omarchy"), L("install"), L("dev"), L("env"), V("lang", A::Ident)]).lane("pacman").gum().root(),
    Entry::new("dev-env.remove", Job, &[L("omarchy"), L("remove"), L("dev"), L("env"), V("lang", A::Ident)]).lane("pacman").gum().root(),
    Entry::new("docker-db.install", Job, &[L("omarchy"), L("install"), L("docker"), L("dbs"), V("name", A::Ident)]).lane("pacman").gum().root(),
    // ---- interactive tools: they need the user's session -----------------
    // `omarchy theme set` runs on after the tool has ended, so what shows the
    // new theme is the daemon's look at `theme.name` (docs/adr/0015).
    Entry::new(
        "theme.switcher",
        Interactive,
        &[L("bash"), L("-c"), L("theme=$(omarchy theme switcher || true); [[ -n $theme ]] && omarchy theme set \"$theme\" >/dev/null 2>&1 &")],
    )
    .affects(&[LOOK]),
    Entry::new("theme.bg-switcher", Interactive, &[L("omarchy"), L("theme"), L("bg-switcher")]).affects(&[LOOK]),
    Entry::new(
        "theme.bg-file",
        Interactive,
        &[L("bash"), L("-c"), L("path=$(omarchy file select --title \"Set background\" --extensions \"jpg jpeg png gif webp bmp\" || true); [[ -n $path ]] && omarchy theme bg set \"$path\"")],
    )
    .affects(&[LOOK]),
    Entry::new("branding.screensaver", Interactive, &[L("omarchy"), L("branding"), L("screensaver"), V("action", A::Is(&["image", "text"]))]).affects(&[LOOK]),
    Entry::new("branding.about", Interactive, &[L("omarchy"), L("branding"), L("about"), V("action", A::Is(&["image", "text"]))]).affects(&[LOOK]),
    Entry::new(
        "capture.screenshot",
        Interactive,
        &[
            L("omarchy"),
            L("capture"),
            L("screenshot"),
            V("mode", A::Is(&["smart", "region", "windows", "fullscreen"])),
            V("dest", A::Is(&["slurp", "copy", "save"])),
        ],
    ),
    Entry::new(
        "capture.recording",
        Interactive,
        &[
            L("omarchy"),
            L("capture"),
            L("screenrecording"),
            F("fullscreen", "--fullscreen"),
            F("desktopAudio", "--with-desktop-audio"),
            F("microphone", "--with-microphone-audio"),
            F("webcam", "--with-webcam"),
            J("--webcam-size=", "webcamSize", A::Is(&["small", "medium", "large"])),
        ],
    )
    // `recordingActive` is the host topic's.
    .affects(&[HOST]),
    Entry::new("capture.text", Interactive, &[L("omarchy"), L("capture"), L("text")]),
    Entry::new("capture.qr", Interactive, &[L("omarchy"), L("capture"), L("qr")]),
    Entry::new("tailscale.send", Interactive, &[L("omarchy"), L("tailscale"), L("send"), V("machine", A::Ident), O("path", A::Path)]),
    Entry::new("tailscale.receive", Interactive, &[L("omarchy"), L("tailscale"), L("receive"), L("--once")]),
];

pub fn find(name: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|entry| entry.name == name)
}

/// Every name of a kind, for the documentation check and the window check.
#[cfg(test)]
pub fn names(kinds: &[super::table::Kind]) -> Vec<&'static str> {
    ENTRIES
        .iter()
        .filter(|entry| kinds.contains(&entry.kind))
        .map(|entry| entry.name)
        .collect()
}

#[cfg(test)]
#[path = "entries_tests.rs"]
mod tests;
