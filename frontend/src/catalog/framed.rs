//! The groups that are drawn in a box.

/// The groups whose QML says `framed: true`, by page and group title: a group that lists things
/// (users, paired devices, networks, the units on Hardware), which the window draws as an outline
/// group, in a box with its title inside. `tests/catalog_parity.rs` holds this to the QML.
#[rustfmt::skip]
pub const FRAMED: &[(&str, &str)] = &[
    ("accounts", "Groups"),
    ("accounts", "Users"),
    ("applications", "Advanced"),
    ("applications", "Desktop"),
    ("applications", "Terminal"),
    ("applications", "Web"),
    ("applications/startup", "Launch on start"),
    ("bar", "Indicators"),
    ("bluetooth", "Nearby"),
    ("bluetooth", "Paired"),
    ("disks", "Encryption"),
    ("disks", "Snapshots"),
    ("disks", "Swap"),
    ("export", "Export"),
    ("export", "Import"),
    ("hardware", "Audio"),
    ("hardware", "Battery"),
    ("hardware", "Memory"),
    ("hardware", "NPU"),
    ("hardware", "Network adapters"),
    ("hardware", "Thermal"),
    ("hardware", "USB"),
    ("keybindings", "What is bound"),
    ("keybindings", "Your overrides"),
    ("network/wifi", "Networks"),
    ("network/wifi", "Saved connections"),
    ("notifications", "Reminders"),
    ("software", "Browsers"),
    ("software", "Development"),
    ("software", "Editors"),
    ("software", "Gaming"),
    ("software", "Services"),
    ("software", "Terminals"),
    ("system", "Advanced"),
    ("system/diagnostics", "Failed units"),
    ("system/diagnostics", "Recent errors"),
    ("system/environment", "Overlay"),
    ("windows/rules", "Your rules"),
    ("workspaces", "Scratch workspaces"),
];

/// Whether `page`'s group `title` is drawn in a box.
pub fn framed(page: &str, title: &str) -> bool {
    FRAMED.contains(&(page, title))
}
