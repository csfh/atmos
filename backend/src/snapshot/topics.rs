//! The settings snapshot as topics (docs/adr/0011).
//!
//! Five topics, each producer's keys in exactly one of them, so a sample of one
//! never overwrites a newer value of another. A topic's document is its keys as
//! the producers read them, with the settings files laid over them, and a
//! `_failed` map of the keys whose tool did not answer (those keys are left out,
//! so a reader keeps the value it had).

use serde_json::{json, Map, Value};

use super::keys;

pub const LOOK: &str = "settings.look";
pub const NETWORK: &str = "settings.network";
pub const ACCOUNTS: &str = "settings.accounts";
pub const SYSTEM: &str = "settings.system";
pub const HOST: &str = "settings.host";

pub const NAMES: &[&str] = &[LOOK, NETWORK, ACCOUNTS, SYSTEM, HOST];

pub fn is_topic(name: &str) -> bool {
    NAMES.contains(&name)
}

/// The key under which a topic names what it could not read.
pub const FAILED: &str = "_failed";

/// The topic a settings file's value lands in: its key's, or the host's for a
/// key no producer reads (the file is the only source).
pub fn topic_of_domain(key: &str) -> &'static str {
    let head = key.split_once('.').map_or(key, |(head, _)| head);
    keys::topic_of_key(head).unwrap_or(HOST)
}

/// What the `settings.snapshot` groups of old name, as topics. A group is a
/// request for some of them; it decides nothing about which key is where.
pub fn for_group(group: &str) -> Vec<&'static str> {
    match group {
        "look" => vec![LOOK],
        "network" => vec![NETWORK],
        "accounts" => vec![ACCOUNTS],
        "system" => vec![SYSTEM],
        "disks" => vec![HOST],
        "rest" => vec![NETWORK, ACCOUNTS, SYSTEM, HOST],
        _ => NAMES.to_vec(),
    }
}

/// One topic's document, read from the machine now.
pub fn collect(name: &str) -> Map<String, Value> {
    let wanted: Vec<String> = keys::keys_in(name).iter().map(|k| k.to_string()).collect();
    let read = keys::read(&wanted);
    let mut doc = read.values;
    for spec in crate::domain::specs() {
        if topic_of_domain(spec.key) != name {
            continue;
        }
        // One unreadable system file must not drop the rest of the topic.
        if let Ok(value) = crate::read_domain(crate::platform::Backend::Omarchy, None, spec.key) {
            super::merge::overlay_domain(&mut doc, spec.key, value);
        }
    }
    doc.retain(|_, value| !value.is_null());
    if !read.failed.is_empty() {
        doc.insert(FAILED.into(), json!(read.failed));
    }
    doc
}

/// Files a topic is read from that no settings key names: `~/` is the home
/// directory. A change to one invalidates the topics listed. A setting's own
/// file is added by `watched`.
const WATCHED: &[(&str, &[&str])] = &[
    (".config/omarchy/shell.json", &[LOOK]),
    ("/usr/share/omarchy/config/omarchy/shell.json", &[LOOK]),
    (".config/omarchy/shell.toml", &[LOOK]),
    (".local/state/omarchy/settings/weather.json", &[HOST]),
    (".local/state/omarchy/notifications.json", &[LOOK]),
    (".local/state/omarchy/current/theme.name", &[LOOK]),
    (".local/state/omarchy/current/background", &[LOOK]),
    (".config/omarchy/branding/screensaver.txt", &[LOOK]),
    ("/usr/share/omarchy/logo.txt", &[LOOK]),
    (".config/omarchy/branding/about.txt", &[LOOK]),
    ("/usr/share/omarchy/icon.txt", &[LOOK]),
    ("/usr/share/plymouth/themes/omarchy/logo.png", &[LOOK]),
    ("/usr/share/omarchy/default/plymouth/logo.png", &[LOOK]),
    ("/usr/share/omarchy/themes", &[LOOK]),
    (".config/omarchy/themes", &[LOOK]),
    (".config/fontconfig/fonts.conf", &[LOOK]),
    (".local/state/omarchy/indicators", &[LOOK]),
    (".config/hypr/looknfeel.lua", &[LOOK]),
    (".config/hypr/hyprsunset.conf", &[LOOK]),
    (".config/hypr/monitors.lua", &[LOOK]),
    (".config/hypr/input.lua", &[LOOK]),
    (".local/state/omarchy/toggles", &[LOOK, SYSTEM]),
    (
        ".local/state/omarchy/atmos-presentation.json",
        &[LOOK, SYSTEM],
    ),
    (".local/state/omarchy/powerprofiles/ac", &[HOST]),
    (".local/state/omarchy/powerprofiles/battery", &[HOST]),
    ("/var/lib/power-profiles-daemon/state.ini", &[HOST]),
    (".local/share/applications", &[HOST]),
    (".config/hypr/autostart.lua", &[HOST]),
    (".config/hypr/bindings.lua", &[HOST]),
    (".config/hypr/atmos.lua", &[HOST]),
    (".local/state/omarchy/defaults/editor", &[SYSTEM]),
    (".config/omarchy/defaults/agent", &[SYSTEM]),
    (".config/xdg-terminals.list", &[SYSTEM]),
    (".config/mimeapps.list", &[SYSTEM]),
    (".config/environment.d/10-atmos.conf", &[SYSTEM]),
    (".config/gtk-4.0/settings.ini", &[SYSTEM]),
    ("/etc/sysctl.d/99-atmos-swappiness.conf", &[SYSTEM]),
    ("/etc/localtime", &[SYSTEM]),
    ("/etc/vconsole.conf", &[SYSTEM]),
    ("/etc/locale.conf", &[SYSTEM]),
    ("/etc/pacman.conf", &[SYSTEM]),
    ("/etc/NetworkManager/conf.d/20-omarchy-dns.conf", &[NETWORK]),
    ("/var/lib/systemd/rfkill", &[NETWORK]),
    ("/run/NetworkManager/devices", &[NETWORK]),
];

/// Every file whose change invalidates a topic, and which: the settings files
/// the keys are read from, and the files in `WATCHED`. The daemon looks at
/// these, so no window has to say what to watch.
pub fn watched() -> Vec<(std::path::PathBuf, Vec<&'static str>)> {
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    let mut table: Vec<(std::path::PathBuf, Vec<&'static str>)> = WATCHED
        .iter()
        .map(|(path, topics)| {
            let full = if path.starts_with('/') {
                std::path::PathBuf::from(path)
            } else {
                home.join(path)
            };
            (full, topics.to_vec())
        })
        .collect();
    for spec in crate::domain::specs() {
        let Ok(place) = crate::domain::locate(crate::platform::Backend::Omarchy, spec) else {
            continue;
        };
        let path = crate::store::place_path(None, &place.rel);
        let topic = topic_of_domain(spec.key);
        match table.iter_mut().find(|(known, _)| *known == path) {
            Some((_, topics)) if !topics.contains(&topic) => topics.push(topic),
            Some(_) => {}
            None => table.push((path, vec![topic])),
        }
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_is_in_one_topic_and_the_topics_are_disjoint() {
        let mut seen = std::collections::HashMap::new();
        for name in NAMES {
            for key in keys::keys_in(name) {
                assert_eq!(seen.insert(key, *name), None, "{key} is in two topics");
                assert_eq!(keys::topic_of_key(key), Some(*name));
            }
        }
        assert!(seen.len() > 200, "{} keys", seen.len());
    }

    #[test]
    fn every_topic_holds_something() {
        for name in NAMES {
            assert!(!keys::keys_in(name).is_empty(), "{name}");
        }
    }

    #[test]
    fn the_keys_the_window_reads_most_are_where_their_tool_is() {
        assert_eq!(keys::topic_of_key("monitors"), Some(LOOK));
        assert_eq!(keys::topic_of_key("netSsid"), Some(NETWORK));
        assert_eq!(keys::topic_of_key("bluetoothDevices"), Some(NETWORK));
        assert_eq!(keys::topic_of_key("timezone"), Some(SYSTEM));
        assert_eq!(keys::topic_of_key("passwordlessSudo"), Some(HOST));
        assert_eq!(keys::topic_of_key("keybindings"), Some(HOST));
        assert_eq!(keys::topic_of_key("nope"), None);
    }

    #[test]
    fn a_settings_file_lands_with_the_key_it_belongs_to() {
        assert_eq!(topic_of_domain("hyprLook.gapsIn"), LOOK);
        assert_eq!(topic_of_domain("theme"), LOOK);
        assert_eq!(topic_of_domain("favorites"), HOST);
        assert_eq!(keys::topic_of_key("favorites"), Some(HOST));
        assert_eq!(
            topic_of_domain("some.unowned.key"),
            HOST,
            "no producer reads it"
        );
    }

    #[test]
    fn the_watched_files_name_real_topics_and_include_the_settings_files() {
        let table = watched();
        assert!(table.len() > 60, "{}", table.len());
        for (path, topics) in &table {
            assert!(path.is_absolute(), "{path:?}");
            assert!(!topics.is_empty());
            assert!(topics.iter().all(|t| is_topic(t)), "{path:?} {topics:?}");
        }
        assert!(table.iter().any(
            |(path, topics)| path.ends_with(".config/hypr/looknfeel.lua") && topics.contains(&LOOK)
        ));
    }

    #[test]
    fn a_group_is_a_request_for_topics() {
        assert_eq!(for_group("look"), [LOOK]);
        assert_eq!(for_group("all").len(), NAMES.len());
        assert!(!for_group("rest").contains(&LOOK));
    }
}
