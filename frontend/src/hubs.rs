//! The hubs the sidebar lists, in the order `services/Hubs.js` gives them, and the group each sits
//! in (`services/Layout.js`). `tests/hubs.rs` pins the ids to `backend/src/launch.rs` and the
//! titles and groups to `Hubs.js`, so this list cannot drift from the app it replaces.

pub struct Hub {
    pub id: &'static str,
    pub title: &'static str,
    /// The group's id; `GROUPS` has its label.
    pub group: &'static str,
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

const fn hub(id: &'static str, title: &'static str, group: &'static str) -> Hub {
    Hub { id, title, group }
}

pub const HUBS: &[Hub] = &[
    hub("home", "Home", "home"),
    hub("monitor", "Monitor", "home"),
    hub("sentinel", "Sentinel", "home"),
    hub("favorites", "Favorites", "look"),
    hub("appearance", "Appearance", "look"),
    hub("display", "Displays", "look"),
    hub("windows", "Windows", "look"),
    hub("workspaces", "Workspaces", "look"),
    hub("bar", "Bar", "look"),
    hub("notifications", "Notifications", "look"),
    hub("profiles", "Profiles", "look"),
    hub("input", "Input", "input"),
    hub("keybindings", "Keybindings", "input"),
    hub("accessibility", "Accessibility", "input"),
    hub("sound", "Sound", "input"),
    hub("capture", "Capture", "input"),
    hub("hardware", "Hardware", "device"),
    hub("drivers", "Drivers", "device"),
    hub("disks", "Disks", "device"),
    hub("network", "Network", "device"),
    hub("bluetooth", "Bluetooth", "device"),
    hub("power", "Power", "device"),
    hub("idle", "Idle", "device"),
    hub("defaults", "Defaults", "apps"),
    hub("agentic", "Agentic", "apps"),
    hub("applications", "Applications", "apps"),
    hub("software", "Software", "apps"),
    hub("hooks", "Hooks", "apps"),
    hub("system", "System", "general"),
    hub("tweaks", "Tweaks", "general"),
    hub("export", "Omafile", "general"),
    hub("accounts", "Accounts", "admin"),
    hub("security", "Security", "admin"),
    hub("services", "Services", "admin"),
];

pub fn find(id: &str) -> Option<&'static Hub> {
    HUBS.iter().find(|hub| hub.id == id)
}

/// The hub a launcher word opens: `ratmos launch`'s aliases (`alias` in `backend/src/launch.rs`),
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
