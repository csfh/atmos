//! The hubs the sidebar lists, in the order `services/Hubs.js` gives them, the group each sits
//! in (`services/Layout.js`), what each says of itself, and its pages. `tests/pins.rs` pins the
//! ids to `backend/src/launch.rs` and the rest to `Hubs.js`, so this list cannot drift from the
//! app it replaces.

pub struct Hub {
    pub id: &'static str,
    pub title: &'static str,
    /// The group's id; `GROUPS` has its label.
    pub group: &'static str,
    pub description: &'static str,
    /// What search finds it by besides its title and description.
    pub keywords: &'static [&'static str],
    /// Its pages besides its own, by `<hub>/<page>` id.
    pub children: &'static [Child],
}

/// A page of a hub besides the hub's own.
pub struct Child {
    pub id: &'static str,
    pub title: &'static str,
}

/// A group's id and the heading the sidebar shows over it. Home has none.
pub const GROUPS: &[(&str, &str)] = &[
    ("home", ""),
    ("look", "Desktop"),
    ("input", "Controls"),
    ("device", "Machine"),
    ("apps", "Apps"),
    ("general", "General"),
    ("admin", "Admin"),
];

const fn child(id: &'static str, title: &'static str) -> Child {
    Child { id, title }
}

// Data, one hub to a block and its keywords on one line; `tests/pins.rs` holds it to `Hubs.js`.
#[rustfmt::skip]
pub const HUBS: &[Hub] = &[
    Hub {
        id: "home",
        title: "Home",
        group: "home",
        description: "Live load, memory, temperatures, network, and this user's processes.",
        keywords: &["dashboard", "home", "cpu", "memory", "ram", "network", "bandwidth", "process", "pid", "kill", "sparkline", "chart", "graph", "load", "activity", "thermal", "temperature", "gpu", "heat"],
        children: &[],
    },
    Hub {
        id: "monitor",
        title: "Monitor",
        group: "home",
        description: "Live cores, memory, disk I/O, traffic, sensors, and a full process table.",
        keywords: &["monitor", "htop", "btop", "top", "activity", "process", "pid", "kill", "cpu", "core", "load", "memory", "ram", "swap", "disk", "io", "traffic", "bandwidth", "sensor", "thermal", "temperature", "gpu", "psi", "stall", "sparkline", "histogram"],
        children: &[child("monitor/processes", "Processes"), child("monitor/cpu", "CPU"), child("monitor/memory", "Memory"), child("monitor/disk", "Disk I/O"), child("monitor/traffic", "Traffic"), child("monitor/sensors", "Sensors")],
    },
    Hub {
        id: "sentinel",
        title: "Sentinel",
        group: "home",
        description: "Processes hogging CPU or memory, and ports held open by long-running ones.",
        keywords: &["rogue", "runaway", "hog", "hogging", "stuck", "leak", "leaking", "high", "cpu", "memory", "ram", "listening", "listener", "port", "tcp", "socket", "bind", "open", "exposed", "long", "running", "suspicious", "kill", "pid", "threshold"],
        children: &[],
    },
    Hub {
        id: "favorites",
        title: "Favorites",
        group: "look",
        description: "Settings you starred, in one place.",
        keywords: &["favorite", "favourite", "star", "pin", "bookmark", "saved"],
        children: &[],
    },
    Hub {
        id: "appearance",
        title: "Appearance",
        group: "look",
        description: "Theme, wallpaper, fonts, and how the desktop looks.",
        keywords: &["theme", "background", "wallpaper", "font", "text", "size", "reset", "default", "scale", "aether", "palette", "night", "light", "nightlight", "warmth", "temperature", "kelvin", "schedule", "hyprsunset", "plymouth", "boot", "screen", "unlock", "sddm", "login", "refresh", "reapply", "templates", "switcher", "preview", "thumbnail", "picker", "install", "git", "extra", "clone", "update", "pull", "remove", "uninstall", "delete", "custom", "folder", "file", "image", "path", "directory", "cache", "logo", "png", "render"],
        children: &[child("appearance/background", "Background"), child("appearance/boot", "Boot screen")],
    },
    Hub {
        id: "display",
        title: "Displays",
        group: "look",
        description: "Monitors, scale, and brightness.",
        keywords: &["monitor", "scale", "hidpi", "brightness", "backlight", "ddc", "keyboard", "laptop", "lid", "internal", "mirror", "clone", "edp", "hdmi", "dp", "resolution", "refresh", "touchpad", "trackpad", "pointer", "mouse", "input", "touch", "touchscreen", "tablet", "digitizer"],
        children: &[],
    },
    Hub {
        id: "windows",
        title: "Windows",
        group: "look",
        description: "Gaps, borders, bindings, and window rules.",
        keywords: &["gaps", "border", "rounding", "blur", "shadow", "tiling", "dwindle", "scrolling", "niri", "column", "opacity", "transparency", "fullscreen", "tight", "square", "aspect", "dim", "animations", "cursor", "tearing", "looknfeel", "preserve", "split", "focus", "activate", "swallow", "window", "rule", "float", "tile", "class"],
        children: &[child("windows/rules", "Window rules")],
    },
    Hub {
        id: "workspaces",
        title: "Workspaces",
        group: "look",
        description: "Count, names, monitors, and how you switch.",
        keywords: &["workspace", "named", "persistent", "monitor", "scratch", "special", "wrap", "wheel", "switch", "default", "login", "bar", "plugin", "clone"],
        children: &[],
    },
    Hub {
        id: "bar",
        title: "Bar",
        group: "look",
        description: "Position, clock, and widgets.",
        keywords: &["position", "transparent", "menu", "bar", "top", "bottom", "left", "right", "show", "hide", "visible", "clock", "format", "time", "alternate", "date", "week", "start", "calendar", "sunday", "monday", "birth", "year", "age", "life", "expectancy", "indicators", "always", "status", "icons", "dictation", "recording", "reminder", "night", "light", "dnd", "stay", "awake", "agents", "usage", "refresh", "sync", "snapshot", "file", "hostname", "device", "id", "spacer", "gap", "padding", "tray", "hidden", "unhide", "pin", "unpin", "plugin", "widget"],
        children: &[],
    },
    Hub {
        id: "notifications",
        title: "Notifications",
        group: "look",
        description: "Do not disturb and reminders.",
        keywords: &["do", "not", "disturb", "dnd", "silent", "mute", "quiet", "toast", "reminder", "timer", "notify", "later", "test", "time", "battery", "weather"],
        children: &[],
    },
    Hub {
        id: "profiles",
        title: "Profiles",
        group: "look",
        description: "Apply a bundle of settings at once.",
        keywords: &["profile", "coding", "gaming", "battery", "bundle", "apply", "import", "stay", "awake", "performance"],
        children: &[],
    },
    Hub {
        id: "input",
        title: "Input",
        group: "input",
        description: "Pointer, keyboard, and gestures.",
        keywords: &["mouse", "pointer", "sensitivity", "acceleration", "natural", "scroll", "inertia", "wheel", "high-res", "discrete", "smooth", "touchpad", "clickfinger", "repeat", "delay", "numlock", "follow", "dpms", "gesture", "swipe", "layout", "xkb", "keyboard"],
        children: &[],
    },
    Hub {
        id: "keybindings",
        title: "Keybindings",
        group: "input",
        description: "Search, add, and override Hyprland chords.",
        keywords: &["keybinding", "hotkey", "shortcut", "bind", "unbind", "chord", "record", "conflict", "reset", "window", "workspace", "launcher", "media", "system"],
        children: &[],
    },
    Hub {
        id: "accessibility",
        title: "Accessibility",
        group: "input",
        description: "Motion, text size, and assistive tools.",
        keywords: &["a11y", "motion", "animations", "reduce", "text", "size", "font", "cursor", "pointer", "hide", "typing", "touchscreen", "herdr", "screen", "reader"],
        children: &[],
    },
    Hub {
        id: "sound",
        title: "Sound",
        group: "input",
        description: "Volume, sinks, and sources.",
        keywords: &["audio", "volume", "mute", "speaker", "headphone", "sink", "source", "microphone", "mic", "input", "output", "pipewire", "pactl", "wpctl", "tuning", "dsp", "restart", "voxtype", "dictation", "speech"],
        children: &[],
    },
    Hub {
        id: "capture",
        title: "Capture",
        group: "input",
        description: "Screenshots, recordings, and OCR.",
        keywords: &["screenshot", "record", "recording", "screen", "video", "ocr", "qr", "webcam", "grim", "slurp", "pictures", "videos"],
        children: &[],
    },
    Hub {
        id: "hardware",
        title: "Hardware",
        group: "device",
        description: "CPU, memory, firmware identity, and the rest of the machine.",
        keywords: &["cpu", "processor", "intel", "amd", "memory", "ram", "dimm", "ddr", "chipset", "motherboard", "bios", "uefi", "firmware", "tpm", "npu", "xdna", "neural", "tpu", "ai", "accelerator", "dmi", "chassis", "laptop", "desktop", "framework", "usb", "nic", "thermal", "battery"],
        children: &[],
    },
    Hub {
        id: "drivers",
        title: "Drivers",
        group: "device",
        description: "GPU stack, hybrid switching, and firmware updates.",
        keywords: &["gpu", "graphics", "nvidia", "vulkan", "hybrid", "supergfx", "igpu", "cuda", "radeon", "drm", "amdgpu", "i915", "xe", "driver", "firmware", "fwupd", "bios", "gsp"],
        children: &[],
    },
    Hub {
        id: "disks",
        title: "Disks",
        group: "device",
        description: "Drives, snapshots, and swap.",
        keywords: &["drive", "nvme", "ssd", "lsblk", "luks", "encryption", "snapper", "snapshot", "btrfs", "hibernation", "zram", "swap", "speedtest", "df", "usage", "trim", "fstrim", "timeline", "retention"],
        children: &[],
    },
    Hub {
        id: "network",
        title: "Network",
        group: "device",
        description: "Wi-Fi, DNS, VPN, and speed test.",
        keywords: &["dns", "cloudflare", "google", "dhcp", "custom", "nameserver", "resolver", "radio", "wifi", "band", "ghz", "wlan", "qr", "share", "ssid", "password", "psk", "copy", "ethernet", "status", "ip", "speedtest", "join", "scan", "forget", "restart", "rfkill", "connectivity", "tailscale", "vpn", "tailnet", "localsend", "taildrop", "clipboard", "file", "folder"],
        children: &[child("network/wifi", "Wi-Fi"), child("network/speedtest", "Speed test")],
    },
    Hub {
        id: "bluetooth",
        title: "Bluetooth",
        group: "device",
        description: "Adapter, pairing, and trusted devices.",
        keywords: &["bluetooth", "radio", "pair", "connect", "disconnect", "forget", "trust", "battery", "discoverable", "headset", "mouse"],
        children: &[],
    },
    Hub {
        id: "power",
        title: "Power",
        group: "device",
        description: "Profiles and battery.",
        keywords: &["power", "profile", "performance", "balanced", "battery", "saver", "percentage", "charge", "laptop", "ac", "plugged", "adapter", "charger", "unplugged", "discharging", "status", "notify", "draw"],
        children: &[],
    },
    Hub {
        id: "idle",
        title: "Idle",
        group: "device",
        description: "Screensaver, lock, and lid.",
        keywords: &["screensaver", "lock", "timeout", "idle", "security", "stay", "awake", "caffeine", "allow", "disable", "suspend", "sleep", "branding", "logo", "ascii", "lid", "clamshell"],
        children: &[],
    },
    Hub {
        id: "defaults",
        title: "Defaults",
        group: "apps",
        description: "Browser, terminal, editor, and MIME.",
        keywords: &["browser", "terminal", "editor", "chrome", "firefox", "nvim", "pdf", "mime", "image", "video"],
        children: &[],
    },
    Hub {
        id: "agentic",
        title: "Agentic",
        group: "apps",
        description: "Installed coding agents, the default Omarchy opens, and MCP.",
        keywords: &["agent", "agentic", "coding", "mcp", "claude", "codex", "grok", "cursor", "pi", "omp", "opencode", "gemini", "hermes", "openclaw", "muse", "copilot", "crush", "install", "default", "tools"],
        children: &[child("agentic/mcp", "MCP")],
    },
    Hub {
        id: "applications",
        title: "Applications",
        group: "apps",
        description: "Desktop, TUI, and web launchers.",
        keywords: &["desktop", "app", "tui", "webapp", "web", "add", "install", "create", "remove", "uninstall", "launcher", "shortcut", "delete", "autostart", "startup", "launch"],
        children: &[child("applications/startup", "Startup")],
    },
    Hub {
        id: "software",
        title: "Software",
        group: "apps",
        description: "Packages and extras.",
        keywords: &["install", "remove", "browser", "terminal", "editor", "service", "gaming", "steam", "heroic", "lutris", "docker", "chatgpt", "1password", "dropbox", "tailscale", "nordvpn", "signal", "spotify", "sunshine", "package"],
        children: &[],
    },
    Hub {
        id: "hooks",
        title: "Hooks",
        group: "apps",
        description: "Theme-set and other scripts.",
        keywords: &["hook", "script", "theme-set", "font-set", "post-boot", "post-update", "pacman", "battery-low"],
        children: &[],
    },
    Hub {
        id: "system",
        title: "System",
        group: "general",
        description: "Host, locale, updates, and about.",
        keywords: &["crash", "capture", "diagnostics", "coredump", "weather", "location", "city", "forecast", "coordinates", "latitude", "longitude", "gps", "units", "celsius", "fahrenheit", "metric", "imperial", "refresh", "interval", "about", "logo", "branding", "fastfetch", "timezone", "tz", "utc", "region", "date", "time", "zoneinfo", "timedatectl", "hostname", "computer", "machine", "device", "name", "hostnamectl", "keyboard", "layout", "keymap", "xkb", "qwerty", "language", "input", "localectl", "ntp", "timesync", "synchronize", "automatic", "clock", "network", "locale", "lang", "utf-8", "i18n", "translation", "pacman", "parallel", "downloads", "packages", "mirrors", "aur", "update", "channel", "orphan", "prune", "version", "printer", "cups", "print", "restore", "hyprland", "shell", "restart", "atmos", "git", "pull", "reset", "sentinel", "overrides", "report", "journal", "systemd", "portal", "pipewire", "kernel", "environment", "path", "editor", "history", "preview", "audit", "undo", "changes", "uki"],
        children: &[child("system/environment", "Environment"), child("system/kernel", "Kernel"), child("system/history", "History"), child("system/diagnostics", "Diagnostics")],
    },
    Hub {
        id: "tweaks",
        title: "Tweaks",
        group: "general",
        description: "Overflow settings with a one-click reset.",
        keywords: &["tweak", "acceleration", "natural", "scroll", "paste", "electron", "wayland", "cursor", "sysctl", "swappiness", "nvidia", "amd", "laptop"],
        children: &[],
    },
    Hub {
        id: "export",
        title: "Omafile",
        group: "general",
        description: "Share this Omarchy system as a Markdown file, or apply one.",
        keywords: &["omafile", "import", "export", "backup", "restore", "settings", "file", "markdown", "md", "share", "declare", "machine", "profile", "transfer", "move", "migrate", "copy", "clone", "another", "laptop", "dotfiles", "plan", "dry", "run", "review", "undo", "revert", "apply", "keybindings", "bindings", "rules", "autostart", "omarchy"],
        children: &[],
    },
    Hub {
        id: "accounts",
        title: "Accounts",
        group: "admin",
        description: "Face, password, users, and groups.",
        keywords: &["account", "avatar", "face", "icon", "picture", "sddm", "user", "group", "password", "passwd", "login", "wheel", "admin", "docker", "useradd", "userdel", "groupadd", "usermod", "chfn", "gecos", "full", "name", "real", "display", "chpasswd"],
        children: &[],
    },
    Hub {
        id: "security",
        title: "Security",
        group: "admin",
        description: "Fingerprint, SSH, and sudo.",
        keywords: &["fingerprint", "fido2", "yubikey", "ssh", "sshd", "sudo", "passwordless", "docker", "pam", "u2f"],
        children: &[],
    },
    Hub {
        id: "services",
        title: "Services",
        group: "admin",
        description: "Search, filter, and control user and system units.",
        keywords: &["systemd", "unit", "service", "enable", "disable", "start", "stop", "restart", "failed", "logs", "pipewire", "portal", "bluetooth", "search", "filter"],
        children: &[],
    },
];

pub fn find(id: &str) -> Option<&'static Hub> {
    HUBS.iter().find(|hub| hub.id == id)
}

/// Whether `id` names a hub or one of its pages (`hubById`).
pub fn known(id: &str) -> bool {
    HUBS.iter()
        .any(|hub| hub.id == id || hub.children.iter().any(|c| c.id == id))
}

/// The title of a hub or one of its pages, then of the hub a page path is under, then the id
/// itself (`hubTitle`).
pub fn title(id: &str) -> &str {
    HUBS.iter()
        .find_map(|hub| {
            (hub.id == id)
                .then_some(hub.title)
                .or_else(|| hub.children.iter().find(|c| c.id == id).map(|c| c.title))
        })
        .or_else(|| find(id.split('/').next().unwrap_or_default()).map(|hub| hub.title))
        .unwrap_or(id)
}

/// The hub a launcher word opens: `atmosd launch`'s aliases (`alias` in `backend/src/launch.rs`),
/// then the hub of a path such as `monitor/cpu`. `None` for a word that names no hub.
pub fn resolve(page: &str) -> Option<&'static Hub> {
    let page = match page {
        "dashboard" => "monitor",
        "machine" | "system/machine" => "hardware",
        "defaults/agents" => "agentic/mcp",
        other if other.starts_with("dashboard/") => "monitor",
        other => other,
    };
    find(page.split('/').next().unwrap_or_default())
}

/// The sidebar's groups in order, each with its heading and hubs. Consecutive hubs of one group
/// stay together, as `clusterByGroup` does.
pub fn clusters() -> Vec<(&'static str, Vec<&'static Hub>)> {
    let mut out: Vec<(&'static str, Vec<&'static Hub>)> = Vec::new();
    let mut last = "";
    for hub in HUBS {
        if out.is_empty() || hub.group != last {
            let label = GROUPS
                .iter()
                .find(|(id, _)| *id == hub.group)
                .map_or("", |(_, label)| *label);
            out.push((label, Vec::new()));
            last = hub.group;
        }
        out.last_mut()
            .expect("a cluster was just pushed")
            .1
            .push(hub);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_launcher_word_opens_its_hub() {
        let hub = |page| resolve(page).map(|h| h.id);
        assert_eq!(hub("monitor"), Some("monitor"));
        assert_eq!(hub("appearance/theme"), Some("appearance"));
        assert_eq!(hub("dashboard"), Some("monitor"));
        assert_eq!(hub("dashboard/cpu"), Some("monitor"));
        assert_eq!(hub("system/machine"), Some("hardware"));
        assert_eq!(hub("defaults/agents"), Some("agentic"));
        assert_eq!(hub("nothing"), None);
        assert_eq!(hub(""), None);
    }

    #[test]
    fn clusters_keep_the_order_and_only_home_is_unlabelled() {
        let clusters = clusters();
        let headings: Vec<_> = clusters.iter().map(|(label, _)| *label).collect();
        assert_eq!(
            headings,
            [
                "", "Desktop", "Controls", "Machine", "Apps", "General", "Admin"
            ]
        );
        let flat: Vec<_> = clusters
            .iter()
            .flat_map(|(_, h)| h.iter().map(|h| h.id))
            .collect();
        let listed: Vec<_> = HUBS.iter().map(|h| h.id).collect();
        assert_eq!(flat, listed);
    }
}
