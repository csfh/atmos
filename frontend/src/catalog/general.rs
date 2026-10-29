//! The General pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{Buttons, Calls, Custom, Go, Switch};
use super::control::Field::Key;
use super::control::Options::{Fixed, Listed};
use super::control::Shape::{Count, Pick};
use super::control::When::{All, Has, Idle, On};
use super::control::range;
use super::custom::Custom::{
    AddVar, Detected, Diag, DirectBoot, Export, Hist, PathPrepend, RunningKernel, Swappiness, Sys,
};
use super::setter::Args::{Given, Nothing};
use super::setter::Sends::{Job, Setting, WriteText};
use super::setter::{ANY, btn, setter, task};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "system",
        file: "SystemPage.qml",
        description: "This machine's name, language, and clock. Account name and face are on Accounts. Printers, weather, diagnostics, and Omarchy updates are further down.",
        groups: &[
            Group { title: "Machine", detail: "Hostname is how this computer shows up on the network and in prompts.", rows: &[
                line("Hostname", "", "hostnamectl set-hostname", &["hostname", "computer", "machine", "device", "name"]).with(Custom(Sys("hostname"))),
            ] },
            Group { title: "Keyboard", detail: "XKB layout for typing. Hyprland reads it from vconsole.conf after you change it.", rows: &[
                line("Layout", "Key positions for typing. Hyprland picks this up from vconsole.conf.", "localectl set-x11-keymap", &["keyboard", "layout", "keymap", "xkb", "qwerty", "language", "input"]).with(Custom(Sys("layout"))),
            ] },
            Group { title: "Language", detail: "The locale apps and the system use for language, dates, and number formats. New sessions pick this up.", rows: &[
                line("Locale", "Language for the system and apps. Open a new session after you change it.", "localectl set-locale", &["locale", "lang", "language", "utf-8", "i18n", "translation"]).with(Calls(Key("locale"), Pick(Listed(Key("locales"), &[])), setter("setLocale", Given, ANY.row_same().trim().non_empty().same().valid(), Setting("locale")))).enabled(Has(Key("locales"))),
            ] },
            Group { title: "Date and time", detail: "Timezone is what the clock, logs, and timestamps use. Network time keeps that clock honest over NTP.", rows: &[
                line("Timezone", "The zone the clock, logs, and timestamps use.", "timedatectl set-timezone", &["timezone", "tz", "utc", "region", "city", "date", "time", "zoneinfo"]).with(Calls(Key("timezone"), Pick(Listed(Key("timezones"), &[])), setter("setTimezone", Given, ANY.row_same().trim().non_empty().same().valid(), Setting("timezone")))).enabled(Has(Key("timezones"))),
                line("Network time", "", "timedatectl set-ntp", &["ntp", "timesync", "synchronize", "automatic", "clock"]).with(Switch(Key("ntp"))).shown(On(Key("ntpAvailable"))).enabled(On(Key("ntpAvailable"))).varies(All(&[On(Key("ntp")), On(Key("ntpSynchronized"))]), "The clock is in sync over the network.", "Keep the clock in sync over the network (NTP)."),
            ] },
            Group { title: "Updates", detail: "Channel picks which Omarchy package stream you follow. Update runs the usual omarchy update job.", rows: &[
                line("Channel", "Stable is the usual stream. rc and edge move faster. Dev is a source checkout.", "omarchy channel set", &["channel", "stable", "rc", "edge", "dev", "mirror"]).with(Custom(Sys("ochannel"))),
                line("Updates", "", "omarchy update", &["update", "upgrade", "pacman", "check"]).with(Custom(Sys("oupdates"))),
            ] },
            Group { title: "Atmos", detail: "", rows: &[
                line("Channel", "Stable tracks the stable branch. Other channels are not available yet.", "~/.config/atmos/channel", &["atmos", "channel", "stable", "branch"]).with(Custom(Sys("achannel"))),
                line("Updates", "", "atmosd task update-atmos", &["atmos", "update", "upgrade", "git", "pull", "stable"]).with(Custom(Sys("aupdates"))),
                line("Hyprland integration", "", "atmos --setup", &["atmos", "setup", "hyprland", "integration", "hypr.atmos", "drop-in"]).with(Custom(Sys("integration"))),
                line("Reset", "", "atmosd task reset-atmos", &["reset", "clear", "sentinel", "overrides", "atmos", "search", "index", "sqlite", "cache"]).with(Buttons(&[btn("Reset…", setter("resetAtmos", Nothing, ANY, Job(task("reset.atmos", "reset-atmos", "The Atmos reset")))).when(Idle).confirm("Reset Atmos", "Remove Atmos-managed Hyprland overrides (look, input, autostart, bindings, extra window rules) and the search index cache. The Atmos window still floats. Theme, wallpaper, and shell.json stay as they are.", "Reset")])).words(super::words::Words::Reset),
            ] },
            Group { title: "Printers", detail: "CUPS is the print service on this machine. Set up opens the usual printer window. The web UI is the CUPS admin page on this computer.", rows: &[
                line("Printers", "", "system-config-printer", &["printer", "cups", "print", "ipp"]).varies(On(Key("cupsActive")), "CUPS is running. Set up opens the printer window. The web UI is http://127.0.0.1:631.", "CUPS is not running. You can still open the admin page if you start the service.").with(Custom(Sys("printers"))),
            ] },
            Group { title: "Packages", detail: "How many packages pacman fetches at once. Higher can finish a big upgrade sooner on a fast link.", rows: &[
                line("Parallel downloads", "How many packages pacman fetches at once. Bump this if updates feel slow on a good connection.", "/etc/pacman.conf · ParallelDownloads", &["pacman", "downloads", "parallel", "mirrors", "aur", "speed"]).with(Calls(Key("parallelDownloads"), Count(range(1.0, 20.0, 1.0, 0)), setter("setParallelDownloads", Given, ANY.row_same().range(1.0, 20.0).same(), Setting("parallelDownloads")))),
            ] },
            Group { title: "Branding", detail: "ASCII art on the About screen. Choose a picture to turn into that art. Edit opens the text file if you want to write it yourself. Reset puts the Omarchy icon back.", rows: &[
                line("About logo", "", "omarchy branding about", &["ascii", "logo", "fastfetch", "about"]).varies(On(Key("aboutBranded")), "You are using custom ASCII art on the About screen.", "The stock Omarchy icon. Choose a picture to turn into ASCII. Edit opens the text file.").with(Custom(Sys("about"))),
            ] },
            Group { title: "Weather", detail: "The city the weather widget and notifications use. Auto guesses from your IP. Coordinates pin the forecast when a city name is ambiguous.", rows: &[
                line("Location", "", "omarchy weather location", &["weather", "city", "forecast", "wttr"]).with(Custom(Sys("location"))),
                line("Coordinates", "", "omarchy weather location --set name lat,lon", &["latitude", "longitude", "gps", "coords"]).with(Custom(Sys("coords"))),
                line("Units", "Temperature and wind in the weather widget. Auto follows the location you set above.", "omarchy bar set omarchy.weather unit", &["celsius", "fahrenheit", "metric", "imperial", "temperature"]).with(Calls(Key("weatherUnit"), Pick(Fixed(&[("auto", "Auto"), ("metric", "Celsius"), ("imperial", "Fahrenheit")])), setter("setWeatherUnit", Given, ANY.row_same().one_of(&["auto", "metric", "imperial"]).same(), Setting("weatherUnit")))).shown(On(Key("weatherPresent"))).enabled(On(Key("weatherPresent"))),
                line("Refresh", "How often the bar pulls a new forecast. Five minutes is chatty. An hour is plenty for most days.", "omarchy bar set omarchy.weather refreshMinutes", &["interval", "update", "minutes", "forecast"]).with(Calls(Key("weatherRefreshMinutes"), Count(range(5.0, 60.0, 5.0, 0)), setter("setWeatherRefreshMinutes", Given, ANY.row_same().at_least(1.0).same(), Setting("weatherRefreshMinutes")))).shown(On(Key("weatherPresent"))).enabled(On(Key("weatherPresent"))),
            ] },
            Group { title: "Diagnostics", detail: "Health, failed units, Hyprland errors, and a copyable report. Crash capture lives on that page.", rows: &[
                line("Environment", "Detected session values and a user overlay for extra variables.", "~/.config/environment.d/10-atmos.conf", &["environment", "path", "shell", "xdg"]).with(Go("Open…", "system/environment")),
                line("Kernel", "Running image, direct EFI boot, and swappiness.", "omarchy setup direct boot", &["kernel", "uki", "efi", "swappiness", "limine"]).with(Go("Open…", "system/kernel")),
                line("History", "Every change this window made, and a preview mode that shows a change before it happens. In memory for this session, not a disk log.", "", &["history", "undo", "changes", "preview", "audit"]).with(Go("Open…", "system/history")),
                line("Health report", "", "omarchy debug --no-sudo --print", &["diagnostics", "report", "discord", "journal", "systemd", "hyprland", "crash"]).with(Go("Open…", "system/diagnostics")).words(super::words::Words::Health),
            ] },
            Group { title: "Advanced", detail: "Leftover packages, the pacman download cache, and restore for Hyprland Lua or shell.json. Firmware updates are on Drivers.", rows: &[
                line("Orphan packages", "Remove packages that nothing else depends on.", "omarchy update orphan pkgs", &["orphan", "unused", "pacman"]).with(Buttons(&[btn("Remove…", setter("updateOrphanPkgs", Nothing, ANY, Job(task("update.orphans", "update-orphans", "The orphan removal").sudo()))).when(Idle).confirm("Remove orphans", "Remove packages that nothing else depends on.", "Remove")])),
                line("Package cache", "Delete old downloaded packages to free disk.", "omarchy update pkg prune", &["prune", "cache", "pacman"]).with(Buttons(&[btn("Prune…", setter("prunePkgCache", Nothing, ANY, Job(task("update.prune", "update-prune", "The cache prune").sudo()))).when(Idle).confirm("Prune package cache", "Delete old downloaded packages from the pacman cache.", "Prune")])),
                line("Restart shell", "Reload the bar and notifications without touching shell.json.", "omarchy restart shell", &["restart", "reload", "bar", "quickshell"]).with(Buttons(&[btn("Restart", setter("restartShell", Nothing, ANY, WriteText("restart", &[("what", "shell")]))).when(Idle)])),
                line("Restore Hyprland", "", "omarchy refresh hyprland", &["refresh", "hyprland", "restore", "defaults", "bindings", "monitors"]).with(Buttons(&[btn("Restore…", setter("refreshHyprland", Nothing, ANY, Job(task("refresh.hyprland", "refresh-hyprland", "The Hyprland restore")))).when(Idle).confirm("Restore Hyprland defaults", "Replace the Hyprland Lua files in ~/.config/hypr with the shipped Omarchy copies. Your current files are backed up first. The Atmos drop-in require is written back so this window still floats.", "Restore")])).words(super::words::Words::RestoreHyprland),
                line("Restore shell", "Put the shipped shell.json back. Your copy is backed up. The bar restarts.", "omarchy refresh shell", &["refresh", "shell", "restore", "defaults", "bar"]).with(Buttons(&[btn("Restore…", setter("refreshShell", Nothing, ANY, Job(task("refresh.shell", "refresh-shell", "The shell restore")))).when(Idle).confirm("Restore shell defaults", "Replace ~/.config/omarchy/shell.json with the shipped Omarchy copy. Your current file is backed up first. The bar restarts afterward.", "Restore")])),
            ] },
        ],
    },
    Page {
        id: "system/environment",
        file: "system/EnvironmentPage.qml",
        description: "Detected session values are read-only. Extra variables and a PATH prepend write ~/.config/environment.d/10-atmos.conf and apply on the next login.",
        groups: &[
            Group { title: "Detected", detail: "What this session is already using. Defaults for browser, terminal, and editor are on Defaults.", rows: &[
                line("Session", "This login's session type and desktop.", "", &["wayland", "session", "desktop"]).with(Custom(Detected("session"))),
                line("Shell", "The login shell for this account.", "", &["shell", "zsh", "bash"]).with(Custom(Detected("shell"))),
                line("Editor / terminal / browser", "Environment values for those tools. Defaults still live on Defaults.", "", &["editor", "terminal", "browser"]).with(Custom(Detected("tools"))),
                line("PATH", "", "", &["path"]).with(Custom(Detected("path"))),
            ] },
            Group { title: "Overlay", detail: "Written to ~/.config/environment.d/10-atmos.conf. systemd user sessions pick it up on the next login.", rows: &[
                line("PATH prepend", "Absolute directories, colon-separated, added in front of PATH.", "PATH=…:$PATH", &["path", "prepend"]).with(Custom(PathPrepend)),
                line("Add a variable", "", "", &["environment", "variable", "export"]).with(Custom(AddVar)),
            ] },
        ],
    },
    Page {
        id: "system/kernel",
        file: "system/KernelPage.qml",
        description: "The running image, how firmware starts it, and a couple of knobs that change kernel behavior.",
        groups: &[
            Group { title: "This boot", detail: "uname as the kernel reports it. Hardware has firmware and DMI.", rows: &[
                line("Running kernel", "", "uname -sr", &["kernel", "uname", "linux", "arch", "release"]).with(Custom(RunningKernel)),
            ] },
            Group { title: "Direct EFI boot", detail: "A normal start opens Limine, Omarchy's bootloader. That menu is how you pick this install or an older Snapper snapshot after a bad update. Direct EFI boot writes a firmware entry named Omarchy that loads the Unified Kernel Image, a single EFI file under /boot/EFI/Linux that already contains the kernel, initramfs, and command line. Firmware starts that file. Limine never appears, and you go straight to disk unlock. Do this when you never choose a snapshot at power-on and want that pause gone. Snapshots stay on disk. To boot one afterward, open the firmware boot menu (often F12, F10, or Esc), choose Limine, then the snapshot by date. Run this setup again to delete the firmware entry and return to Limine on every start.", rows: &[
                line("Direct EFI boot", "", "omarchy setup direct boot", &["efi", "uki", "efibootmgr", "direct", "boot", "limine", "snapshot"]).varies(On(Key("directBoot")), "Firmware boots the Omarchy UKI with no Limine stop. Remove the entry to get the snapshot menu back at every boot.", "Skip Limine and go straight to disk unlock. Snapshot rollback then starts from the firmware boot list.").with(Custom(DirectBoot)).shown(On(Key("directBootAvailable"))),
            ] },
            Group { title: "Memory", detail: "vm.swappiness. Lower values keep more file cache in RAM and wait longer before using swap.", rows: &[
                line("Lower swappiness", "", "/etc/sysctl.d/99-atmos-swappiness.conf", &["swap", "swappiness", "sysctl", "vm", "memory"]).with(Custom(Swappiness)),
            ] },
        ],
    },
    Page {
        id: "system/history",
        file: "system/HistoryPage.qml",
        description: "Every change this window made, newest first, in memory for the session. Turn on Preview to see a change before it happens instead of after.",
        groups: &[
            Group { title: "Preview", detail: "While Preview is on, Atmos shows you what a control would run and does not run it. Apply releases everything held; Discard throws it away. Nothing held runs on its own. The flag is not saved across a restart.", rows: &[
                line("Preview changes", "", "", &["dry", "run", "diff", "safe", "preview"]).with(Custom(Hist("preview"))),
                line("Held", "", "", &[]).with(Custom(Hist("held"))),
            ] },
            Group { title: "Changes", detail: "Recorded in memory for this window. Atmos keeps the most recent 200. Another window has its own list; nothing is written to disk.", rows: &[
                line("Nothing yet", "Change something and it will show up here with the command it ran.", "", &[]).with(Custom(Hist("none"))),
                line("By you", "Changes made from a control in this window.", "", &[]).with(Custom(Hist("you"))),
            ] },
        ],
    },
    Page {
        id: "system/diagnostics",
        file: "system/DiagnosticsPage.qml",
        description: "What this machine is doing, in one place. Copy report is the text to paste on Discord instead of twelve shell commands.",
        groups: &[
            Group { title: "Settings that could not be read", detail: "A tool that failed leaves the value you were shown where it was, and says so here.", rows: &[
            ] },
            Group { title: "Report", detail: "Copy puts an Atmos summary plus omarchy debug --no-sudo --print on the clipboard. Save writes the same file. Ask my Agent copies it, then opens the default coding agent with the path.", rows: &[
                line("Copy report", "", "omarchy debug --no-sudo --print", &["copy", "report", "discord", "debug", "paste", "clipboard"]).with(Custom(Diag("copy"))),
                line("Save report", "", "~/.cache/atmos/diagnostic-report.txt", &["save", "file", "export", "txt"]).with(Custom(Diag("save"))),
                line("Ask my Agent", "Copy the report, then open the default coding agent with that file.", "omarchy agent prompt", &["agent", "diagnose", "help", "grok", "claude"]).with(Custom(Diag("ask"))),
                line("Refresh", "Read the machine again. Failed units and Hyprland errors change while it runs.", "snapshot", &["reload", "rescan"]).with(Custom(Diag("refresh"))),
            ] },
            Group { title: "Omarchy", detail: "Package version and channel. Copy on System still copies the version string alone.", rows: &[
            ] },
            Group { title: "Atmos", detail: "The drop-in require in hyprland.lua is what keeps this window floating. Sentinels are the blocks Atmos wrote.", rows: &[
            ] },
            Group { title: "Hyprland", detail: "hyprctl version and configerrors. A dirty config still runs; errors here are why a bind or monitor rule did not apply.", rows: &[
                line("Config error", "", "hyprctl configerrors", &["error", "hypr", "lua"]).with(Custom(Diag("listed"))),
            ] },
            Group { title: "Failed units", detail: "systemctl --failed and the same list for the user session. Start and stop stay on a later Services page. This list is read-only.", rows: &[
                line("Failed units", "No failed units.", "systemctl --failed", &["systemd", "failed", "empty"]).with(Custom(Diag("nofailed"))),
            ] },
            Group { title: "Disk", detail: "Use on the root filesystem, from statvfs. The Disks hub has per-drive detail.", rows: &[
                line("Root", "", "/", &["disk", "df", "space", "pressure"]).with(Custom(Diag("root"))),
            ] },
            Group { title: "Memory", detail: "RAM and swap from /proc/meminfo. Hardware has the DIMM list.", rows: &[
                line("RAM", "", "/proc/meminfo", &["memory", "ram", "pressure"]).with(Custom(Diag("ram"))),
                line("Swap", "", "/proc/meminfo", &["swap", "zram"]).with(Custom(Diag("swap"))),
            ] },
            Group { title: "Kernel", detail: "uname. The Hardware hub has firmware and DMI.", rows: &[
                line("Kernel", "", "uname -sr", &["kernel", "uname", "linux", "arch"]).with(Custom(Diag("kernel"))),
            ] },
            Group { title: "GPU", detail: "The DRM driver name. Drivers has the PCI device list and hybrid GPU switching.", rows: &[
                line("Driver", "", "/sys/class/drm", &["gpu", "drm", "amdgpu", "nvidia", "i915", "xe"]).with(Custom(Diag("driver"))),
            ] },
            Group { title: "Portals", detail: "xdg-desktop-portal and the Hyprland and GTK backends. Screen sharing and file pickers need these.", rows: &[
                line("Portals", "", "systemctl --user status xdg-desktop-portal", &["portal", "xdg", "screencast", "pipewire"]).with(Custom(Diag("portals"))),
            ] },
            Group { title: "PipeWire", detail: "The session audio graph. Restart is on Sound.", rows: &[
                line("PipeWire", "", "systemctl --user status pipewire", &["pipewire", "pulse", "wireplumber", "audio"]).with(Custom(Diag("pipewire"))),
            ] },
            Group { title: "Network", detail: "Whether this machine has a default route. Join a network on the Network hub.", rows: &[
                line("Connectivity", "", "/proc/net/route", &["network", "online", "route", "offline"]).with(Custom(Diag("connectivity"))),
            ] },
            Group { title: "Packages", detail: "pacman sync databases under /var/lib/pacman/sync. Updates stay on System.", rows: &[
                line("Package database", "", "/var/lib/pacman/sync", &["pacman", "sync", "mirrors", "database"]).with(Custom(Diag("packages"))),
            ] },
            Group { title: "Recent errors", detail: "journalctl -b -p 3, last fifteen lines. Lines that look like passwords are dropped.", rows: &[
                line("Errors", "No recent error-priority journal lines.", "journalctl -b -p 3", &["journal", "empty"]).with(Custom(Diag("noerrors"))),
                line("Journal", "", "journalctl -b -p 3", &["journal", "error", "log"]).with(Custom(Diag("listed"))),
            ] },
            Group { title: "Crash capture", detail: "When a process dumps core, Omarchy can notify you so a coding agent can look at the crash.", rows: &[
                line("Crash capture", "A notification when a process crashes, so a coding agent can look at the dump.", "omarchy toggle crash capture", &["crash", "coredump", "agent", "watch"]).with(Switch(Key("crashCapture"))),
            ] },
        ],
    },
    Page {
        id: "export",
        file: "ExportPage.qml",
        description: "An Omafile is this Omarchy system's configuration as a Markdown file. Write one to share the desktop, or apply one from another machine. Review every change before anything happens.",
        groups: &[
            Group { title: "Export", detail: "Writes an Omafile of this Omarchy system. Settings live in fenced blocks you can read and edit. Security settings are written as prose and never applied.", rows: &[
                line("Sections", "", "", &["all", "none", "select", "sections", "choose"]).with(Custom(Export("sections"))),
                line("Where to write it", "", "", &["path", "file", "folder", "directory", "markdown", "md", "save", "choose"]).with(Custom(Export("where"))),
                line("Write the file", "", "", &["export", "save", "write", "backup", "open"]).with(Custom(Export("write"))),
            ] },
            Group { title: "Import", detail: "Apply an Omafile from another Omarchy machine. Nothing is written until you have read the plan. Atmos shows every change and keeps a way back.", rows: &[
                line("File to read", "", "", &["path", "file", "import", "load", "browse", "open"]).with(Custom(Export("file"))),
                line("See what it would do", "", "", &["review", "dry run", "preview", "plan", "diff"]).with(Custom(Export("review"))),
                line("This file", "Where it came from.", "", &["origin", "host", "when", "sections", "provenance"]).with(Custom(Export("summary"))),
                line("Changes", "", "", &["changes", "plan"]).with(Custom(Export("changes"))),
                line("Worth knowing", "These still happen.", "", &["warning", "hardware", "skipped", "shadow"]).with(Custom(Export("warnings"))),
                line("Blocked", "Atmos will not do these.", "", &["blocked", "refused", "security"]).with(Custom(Export("blocked"))),
                line("Apply the plan you just read", "", "", &["apply", "import", "run", "password"]).with(Custom(Export("apply"))),
                line("Applying", "", "", &["progress", "applying"]).with(Custom(Export("applying"))),
                line("Put it back", "", "", &["undo", "revert", "restore", "back", "put back"]).with(Custom(Export("putback"))),
            ] },
        ],
    },
];
