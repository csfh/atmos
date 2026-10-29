//! The words of a row the QML builds from what the window holds (`description: Omarchy.x ?
//! ... : ...`) where one condition and two fixed texts do not say it (`Line::varies` does): a
//! count, a value inside the text, or the row's job running. Each reads the keys it names, so its
//! page holds them. `tests/catalog_words.rs` runs each row's own QML description against these.

use serde_json::Value;

use super::control::Field;
use crate::js;
use crate::settings::Settings;

/// What a row's words are made of.
pub struct Said<'a> {
    pub settings: &'a Settings,
    /// The job the window runs, while it is not an interactive tool (`Omarchy.jobKind` while
    /// `jobBusy`).
    pub job: Option<&'a str>,
    /// The inventory the words read, as the window holds it.
    pub inventory: Option<&'a Value>,
}

impl Said<'_> {
    fn value(&self, key: &'static str) -> Option<Value> {
        Field::Key(key).read(self.settings).value().cloned()
    }

    fn text(&self, key: &'static str) -> String {
        self.value(key)
            .filter(|v| js::truthy(Some(v)))
            .map_or_else(String::new, |v| js::string(&v))
    }

    fn number(&self, key: &'static str) -> f64 {
        js::number(self.value(key).as_ref())
    }

    fn on(&self, key: &'static str) -> bool {
        self.value(key) == Some(Value::Bool(true))
    }

    fn list(&self, key: &'static str) -> Vec<Value> {
        match self.value(key) {
            Some(Value::Array(a)) => a,
            _ => Vec::new(),
        }
    }

    fn managed(&self, key: &'static str) -> usize {
        self.list(key)
            .iter()
            .filter(|r| r.get("managed") == Some(&Value::Bool(true)))
            .count()
    }
}

/// A row's words, by row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Words {
    /// Appearance: Background.
    Background,
    /// Appearance: Boot screen.
    Boot,
    /// Appearance: Night light.
    NightLight,
    /// Appearance: Night light warmth.
    NightWarmth,
    /// Windows: Keybindings (`overrideCountText`).
    Overrides,
    /// Windows: Window rules (`ruleCountText`).
    Rules,
    /// Notifications: Clear reminders.
    Reminders,
    /// Disks: Create snapshot.
    Snapshot,
    /// Applications: Startup programs.
    Startup,
    /// System: Reset.
    Reset,
    /// System: Health report (`diagnosticsDescription`).
    Health,
    /// System: Restore Hyprland.
    RestoreHyprland,
    /// Security: SSH server.
    Ssh,
    /// Network: Bluetooth (`bluetoothHubDescription`).
    Bluetooth,
    /// Network: Tailscale.
    Tailscale,
    /// Wi-Fi: Band.
    Band,
    /// Wi-Fi: Wi-Fi radio.
    Radio,
}

/// `n` and the word for one or many: "One reminder" or "3 reminders".
fn count(n: f64, one: &str, many: &str) -> String {
    if n == 1.0 {
        one.to_owned()
    } else {
        format!("{}{many}", js::string(&js::json_number(n)))
    }
}

impl Words {
    /// The keys it reads.
    pub fn reads(self) -> &'static [&'static str] {
        match self {
            Words::Background => &["background"],
            Words::Boot => &["plymouth"],
            Words::NightLight | Words::NightWarmth => &["nightlightTemperature"],
            Words::Overrides => &["bindings", "keybindings"],
            Words::Rules => &["windowRules"],
            Words::Reminders => &["reminderCount"],
            Words::Startup => &["autostart"],
            Words::Ssh => &["sshdActive", "sshdEnabled"],
            Words::Bluetooth => &["bluetooth", "bluetoothDevices"],
            Words::Tailscale => &["tailscaleRunning", "tailscaleInstalled"],
            Words::Band => &["wifiBand"],
            Words::Radio => &["wifiHw", "wifiRadio", "netKind", "netSsid"],
            Words::Snapshot | Words::Reset | Words::Health | Words::RestoreHyprland => &[],
        }
    }

    /// The inventory it reads, if any.
    pub fn inventory(self) -> Option<&'static str> {
        (self == Words::Health).then_some("diagnostics")
    }

    /// The row's words.
    pub fn say(self, s: &Said) -> String {
        let job = |kind: &str, running: &str, idle: &str| {
            if s.job == Some(kind) { running } else { idle }.to_owned()
        };
        match self {
            Words::Background => match s.text("background") {
                b if b.is_empty() => "No wallpaper is set for this theme yet.".into(),
                b => format!(
                    "Current file: {}.",
                    b.rsplit('/').next().unwrap_or_default()
                ),
            },
            Words::Boot => match s.text("plymouth") {
                p if p.is_empty() => {
                    "The unlock animation and logo you see before the desktop.".into()
                }
                p => format!("Unlock theme: {p}. Logo and preview are on the next page."),
            },
            Words::NightLight => match s.number("nightlightTemperature") {
                k if k > 0.0 => format!(
                    "Shift colors toward amber at night. Right now that is {} K.",
                    js::string(&js::json_number(k))
                ),
                _ => "Shift colors toward amber at night.".into(),
            },
            Words::NightWarmth => match s.number("nightlightTemperature") {
                k if k > 0.0 => format!(
                    "The screen is at {} K. 4000 is the usual amber. 6500 is daylight.",
                    js::string(&js::json_number(k))
                ),
                _ => "Pick a color temperature. 4000 is amber. 6500 is daylight.".into(),
            },
            Words::Overrides => match (s.managed("bindings"), s.list("keybindings").len()) {
                (0, 0) => "No chords listed yet.".into(),
                (0, total) => count(
                    total as f64,
                    "One chord Hyprland is running.",
                    " chords Hyprland is running.",
                ),
                (n, _) => count(
                    n as f64,
                    "One Atmos override in bindings.lua.",
                    " Atmos overrides in bindings.lua.",
                ),
            },
            Words::Rules => match s.managed("windowRules") {
                0 => "No Atmos rules yet.".into(),
                n => count(n as f64, "One Atmos rule.", " Atmos rules."),
            },
            Words::Reminders => count(
                s.number("reminderCount"),
                "One reminder is waiting.",
                " reminders are waiting.",
            ),
            Words::Startup => count(
                s.list("autostart").len() as f64,
                "One command at login.",
                " commands at login.",
            ),
            Words::Snapshot => job(
                "snapshot-create",
                "Creating snapshots…",
                "Take a numbered snapshot of each Snapper config, then run cleanup.",
            ),
            Words::Reset => job(
                "reset-atmos",
                "Resetting Atmos…",
                "Strip look, input, autostart, bindings, extra window rules, and the search cache. This window still floats.",
            ),
            Words::Health => {
                let d = crate::diagnostics::normalize(s.inventory);
                let failed = d["failedUnits"].as_array().map_or(0, Vec::len);
                match failed {
                    0 if !crate::diagnostics::hypr_ok(&d) => {
                        "Hyprland has config errors. Copy a report for Discord or an agent.".into()
                    }
                    0 => "Copy a report for Discord or an agent.".into(),
                    n => {
                        count(n as f64, "One failed unit.", " failed units.")
                            + " Copy a report for Discord or an agent."
                    }
                }
            }
            Words::RestoreHyprland => job(
                "refresh-hyprland",
                "Restoring Hyprland Lua…",
                "Put the shipped Hyprland Lua files back. Your copies are backed up.",
            ),
            Words::Bluetooth => {
                crate::network::bluetooth_says(s.on("bluetooth"), s.list("bluetoothDevices").len())
            }
            Words::Tailscale => if s.on("tailscaleRunning") {
                "tailscaled is running."
            } else if s.on("tailscaleInstalled") {
                "Tailscale is installed but the daemon is not running."
            } else {
                "Install Tailscale if you want this machine on a tailnet."
            }
            .into(),
            Words::Radio => {
                let ssid = s.text("netSsid");
                if !s.on("wifiHw") {
                    "No Wi-Fi adapter.".into()
                } else if !s.on("wifiRadio") {
                    "Scan and join nearby networks.".into()
                } else if s.text("netKind") == "wifi" && !ssid.is_empty() {
                    format!("Connected to {ssid}. Nearby networks stay below.")
                } else {
                    "Scanning for access points below.".into()
                }
            }
            Words::Band => match s.text("wifiBand") {
                b if b.is_empty() => "Stay on one band for this network, or let it pick.".into(),
                b => format!("Stay on {b} GHz for this network, or let it pick."),
            },
            Words::Ssh => {
                if s.on("sshdActive") {
                    "sshd is running. Other machines can log in with an authorized key.".into()
                } else if s.on("sshdEnabled") {
                    "sshd is enabled but not running right now.".into()
                } else {
                    "The OpenSSH server is stopped. Turn it on with a public key if you want remote login.".into()
                }
            }
        }
    }
}
