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

pub const HUBS: &[Hub] = &[
    Hub {
        id: "home",
        title: "Home",
        group: "home",
        description: "Live load, memory, temperatures, network, and this user's processes.",
        children: &[],
    },
    Hub {
        id: "monitor",
        title: "Monitor",
        group: "home",
        description: "Live cores, memory, disk I/O, traffic, sensors, and a full process table.",
        children: &[
            child("monitor/processes", "Processes"),
            child("monitor/cpu", "CPU"),
            child("monitor/memory", "Memory"),
            child("monitor/disk", "Disk I/O"),
            child("monitor/traffic", "Traffic"),
            child("monitor/sensors", "Sensors"),
        ],
    },
    Hub {
        id: "sentinel",
        title: "Sentinel",
        group: "home",
        description: "Processes hogging CPU or memory, and ports held open by long-running ones.",
        children: &[],
    },
    Hub {
        id: "favorites",
        title: "Favorites",
        group: "look",
        description: "Settings you starred, in one place.",
        children: &[],
    },
    Hub {
        id: "appearance",
        title: "Appearance",
        group: "look",
        description: "Theme, wallpaper, fonts, and how the desktop looks.",
        children: &[
            child("appearance/background", "Background"),
            child("appearance/boot", "Boot screen"),
        ],
    },
    Hub {
        id: "display",
        title: "Displays",
        group: "look",
        description: "Monitors, scale, and brightness.",
        children: &[],
    },
    Hub {
        id: "windows",
        title: "Windows",
        group: "look",
        description: "Gaps, borders, bindings, and window rules.",
        children: &[child("windows/rules", "Window rules")],
    },
    Hub {
        id: "workspaces",
        title: "Workspaces",
        group: "look",
        description: "Count, names, monitors, and how you switch.",
        children: &[],
    },
    Hub {
        id: "bar",
        title: "Bar",
        group: "look",
        description: "Position, clock, and widgets.",
        children: &[],
    },
    Hub {
        id: "notifications",
        title: "Notifications",
        group: "look",
        description: "Do not disturb and reminders.",
        children: &[],
    },
    Hub {
        id: "profiles",
        title: "Profiles",
        group: "look",
        description: "Apply a bundle of settings at once.",
        children: &[],
    },
    Hub {
        id: "input",
        title: "Input",
        group: "input",
        description: "Pointer, keyboard, and gestures.",
        children: &[],
    },
    Hub {
        id: "keybindings",
        title: "Keybindings",
        group: "input",
        description: "Search, add, and override Hyprland chords.",
        children: &[],
    },
    Hub {
        id: "accessibility",
        title: "Accessibility",
        group: "input",
        description: "Motion, text size, and assistive tools.",
        children: &[],
    },
    Hub {
        id: "sound",
        title: "Sound",
        group: "input",
        description: "Volume, sinks, and sources.",
        children: &[],
    },
    Hub {
        id: "capture",
        title: "Capture",
        group: "input",
        description: "Screenshots, recordings, and OCR.",
        children: &[],
    },
    Hub {
        id: "hardware",
        title: "Hardware",
        group: "device",
        description: "CPU, memory, firmware identity, and the rest of the machine.",
        children: &[],
    },
    Hub {
        id: "drivers",
        title: "Drivers",
        group: "device",
        description: "GPU stack, hybrid switching, and firmware updates.",
        children: &[],
    },
    Hub {
        id: "disks",
        title: "Disks",
        group: "device",
        description: "Drives, snapshots, and swap.",
        children: &[],
    },
    Hub {
        id: "network",
        title: "Network",
        group: "device",
        description: "Wi-Fi, DNS, VPN, and speed test.",
        children: &[
            child("network/wifi", "Wi-Fi"),
            child("network/speedtest", "Speed test"),
        ],
    },
    Hub {
        id: "bluetooth",
        title: "Bluetooth",
        group: "device",
        description: "Adapter, pairing, and trusted devices.",
        children: &[],
    },
    Hub {
        id: "power",
        title: "Power",
        group: "device",
        description: "Profiles and battery.",
        children: &[],
    },
    Hub {
        id: "idle",
        title: "Idle",
        group: "device",
        description: "Screensaver, lock, and lid.",
        children: &[],
    },
    Hub {
        id: "defaults",
        title: "Defaults",
        group: "apps",
        description: "Browser, terminal, editor, and MIME.",
        children: &[],
    },
    Hub {
        id: "agentic",
        title: "Agentic",
        group: "apps",
        description: "Installed coding agents, the default Omarchy opens, and MCP.",
        children: &[child("agentic/mcp", "MCP")],
    },
    Hub {
        id: "applications",
        title: "Applications",
        group: "apps",
        description: "Desktop, TUI, and web launchers.",
        children: &[child("applications/startup", "Startup")],
    },
    Hub {
        id: "software",
        title: "Software",
        group: "apps",
        description: "Packages and extras.",
        children: &[],
    },
    Hub {
        id: "hooks",
        title: "Hooks",
        group: "apps",
        description: "Theme-set and other scripts.",
        children: &[],
    },
    Hub {
        id: "system",
        title: "System",
        group: "general",
        description: "Host, locale, updates, and about.",
        children: &[
            child("system/environment", "Environment"),
            child("system/kernel", "Kernel"),
            child("system/history", "History"),
            child("system/diagnostics", "Diagnostics"),
        ],
    },
    Hub {
        id: "tweaks",
        title: "Tweaks",
        group: "general",
        description: "Overflow settings with a one-click reset.",
        children: &[],
    },
    Hub {
        id: "export",
        title: "Omafile",
        group: "general",
        description: "Share this Omarchy system as a Markdown file, or apply one.",
        children: &[],
    },
    Hub {
        id: "accounts",
        title: "Accounts",
        group: "admin",
        description: "Face, password, users, and groups.",
        children: &[],
    },
    Hub {
        id: "security",
        title: "Security",
        group: "admin",
        description: "Fingerprint, SSH, and sudo.",
        children: &[],
    },
    Hub {
        id: "services",
        title: "Services",
        group: "admin",
        description: "Search, filter, and control user and system units.",
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
