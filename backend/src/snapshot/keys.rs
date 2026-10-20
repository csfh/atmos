//! Reading some snapshot keys and nothing else: the result half of a write.
//!
//! After a write the window is shown what the machine now reports for the keys
//! the write can change, never a value it worked out itself. Each key belongs to
//! one producer, the same code the full snapshot runs, so the value has the
//! shape the snapshot gives it. A producer is run once however many of its keys
//! are asked for. A key no producer owns is reported back as unknown; a test
//! reads every key the window names and fails on an unknown one.

use serde_json::{json, Map, Value};

use std::collections::BTreeMap;

use super::{host, hypr, net, sh, system, theme_fields, toggle_off, Env};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Producer {
    Theme,
    LiveToggles,
    Nightlight,
    Toggles,
    Bar,
    HyprLook,
    Markers,
    Monitors,
    Devices,
    Dnd,
    Reminders,
    Apps,
    Branding,
    Accounts,
    Atmos,
    Software,
    Snapper,
    Network,
    Bluetooth,
    Tailscale,
    Audio,
    Power,
    Weather,
    Hooks,
    MarkerFlags,
    Control,
    Time,
    Locale,
    Keyboard,
    Hostname,
    FullName,
    ParallelDownloads,
    CrashCapture,
    Defaults,
    DefaultApps,
    Keybindings,
    Favorites,
    Input,
    Security,
    OmarchyInfo,
    HardwareFlags,
    GpuExtras,
    FocusPrint,
}

/// Which producer owns which keys.
const OWNERS: &[(Producer, &[&str])] = &[
    (
        Producer::Theme,
        &[
            "theme",
            "background",
            "font",
            "themes",
            "extraThemes",
            "fonts",
            "plymouth",
            "plymouthThemes",
        ],
    ),
    (
        Producer::LiveToggles,
        &[
            "textSize",
            "stayAwake",
            "nightlight",
            "nightlightTemperature",
        ],
    ),
    (
        Producer::Nightlight,
        &["nightlightDay", "nightlightNight", "nightlightNightOn"],
    ),
    (
        Producer::Toggles,
        &["barVisible", "screensaverEnabled", "suspendEnabled"],
    ),
    (
        Producer::Bar,
        &[
            "agentsPresent",
            "agentsRefreshIntervalSec",
            "agentsSync",
            "agentsSyncDeviceId",
            "agentsSyncDir",
            "agentsSyncFileName",
            "barPosition",
            "barTransparent",
            "clockBirthYear",
            "clockFormat",
            "clockFormatAlt",
            "clockLifeExpectancy",
            "clockPresent",
            "clockWeekStart",
            "idleLock",
            "idleScreensaver",
            "indicatorsAlwaysShow",
            "indicatorsItems",
            "indicatorsPresent",
            "powerPresent",
            "powerShowPercentage",
            "spacerPresent",
            "spacerSize",
            "trayHidden",
            "trayPinned",
            "trayPresent",
            "weatherPresent",
            "weatherRefreshMinutes",
            "weatherUnit",
            "workspaceBarCount",
            "workspaceBarNames",
        ],
    ),
    (Producer::HyprLook, &["hyprLook"]),
    (
        Producer::Markers,
        &[
            "hyprLookManaged",
            "hyprNoGaps",
            "hyprWorkspaceLayout",
            "hyprSquareAspect",
        ],
    ),
    (Producer::Monitors, &["monitors"]),
    (
        Producer::Devices,
        &[
            "internalPresent",
            "internalEnabled",
            "mirroring",
            "touchpadEnabled",
            "touchpadPresent",
            "touchscreenEnabled",
            "touchscreenPresent",
            "keyboardBrightness",
            "keyboardBacklightPresent",
            "isLaptop",
            "batteryPresent",
            "externalPresent",
        ],
    ),
    (Producer::Dnd, &["doNotDisturb"]),
    (
        Producer::Reminders,
        &["reminderCount", "reminderActive", "reminders"],
    ),
    (Producer::Apps, &["desktopApps", "tuiApps", "webApps"]),
    (Producer::Branding, &["screensaverBranded", "aboutBranded"]),
    (
        Producer::Accounts,
        &["currentUser", "avatarPath", "users", "groups"],
    ),
    (Producer::FullName, &["fullName"]),
    (
        Producer::Atmos,
        &[
            "atmosChannel",
            "atmosVersion",
            "atmosRevision",
            "atmosInstalled",
            "atmosPackaged",
        ],
    ),
    (
        Producer::Software,
        &[
            "plugins",
            "recordingActive",
            "webcamOverlay",
            "browsers",
            "editors",
            "extras",
            "gaming",
            "hasAether",
            "services",
            "terminals",
        ],
    ),
    (
        Producer::Snapper,
        &[
            "snapperNumberLimit",
            "snapperTimeline",
            "fstrimEnabled",
            "hibernationAvailable",
            "hibernationConfigured",
            "hibernationSupported",
            "snapperConfigs",
            "snapperPresent",
        ],
    ),
    (
        Producer::Network,
        &[
            "wifiBandSelected",
            "wifiBand",
            "wifiBands",
            "wifiConnected",
            "netKind",
            "netSsid",
            "wifiConnections",
            "wifiRadio",
            "dns",
            "netDnsServers",
            "netGateway",
            "netIface",
            "netIp",
            "netSignal",
            "netSpeed",
            "wifiHw",
            "wifiIface",
        ],
    ),
    (Producer::Bluetooth, &["bluetooth", "bluetoothDevices"]),
    (
        Producer::Tailscale,
        &["tailscaleInstalled", "tailscalePeers", "tailscaleRunning"],
    ),
    (
        Producer::Audio,
        &[
            "audioOutputVolume",
            "audioOutputMuted",
            "audioInputVolume",
            "audioInputMuted",
            "audioSinks",
            "audioSources",
            "audioTuningMatch",
            "audioTuningOn",
        ],
    ),
    (
        Producer::Power,
        &[
            "powerProfile",
            "powerProfiles",
            "powerProfileAc",
            "powerProfileBattery",
        ],
    ),
    (
        Producer::Weather,
        &["weatherLocation", "weatherAuto", "weatherCoords"],
    ),
    (Producer::Hooks, &["hooks"]),
    (
        Producer::MarkerFlags,
        &[
            "bindings",
            "bindingsManaged",
            "windowRules",
            "windowRulesManaged",
            "windowOpaque",
            "autostart",
            "autostartManaged",
        ],
    ),
    (
        Producer::Control,
        &[
            "workspaces",
            "workspacesManaged",
            "workspaceWrapSwitch",
            "workspaceWheelSwitch",
            "monitorRules",
            "monitorRulesManaged",
            "envVars",
            "envPathPrepend",
            "tweaks",
            "presentationMode",
            "amdPstate",
            "chargeLimit",
            "chargeLimitAvailable",
            "envDetected",
            "powerGovernor",
            "systemdUnits",
        ],
    ),
    (
        Producer::Time,
        &[
            "ntp",
            "ntpAvailable",
            "ntpSynchronized",
            "timezone",
            "timezones",
        ],
    ),
    (Producer::Locale, &["locale", "locales"]),
    (Producer::Keyboard, &["keyboardLayout", "keyboardLayouts"]),
    (Producer::Hostname, &["hostname"]),
    (Producer::ParallelDownloads, &["parallelDownloads"]),
    (Producer::CrashCapture, &["crashCapture"]),
    (
        Producer::Defaults,
        &[
            "mimePdf",
            "mimeImage",
            "mimeVideo",
            "directBoot",
            "directBootAvailable",
            "mimeImageOptions",
            "mimePdfOptions",
            "mimeVideoOptions",
            "picturesDir",
            "videosDir",
        ],
    ),
    (
        Producer::DefaultApps,
        &["browser", "terminal", "editor", "agent"],
    ),
    (Producer::Keybindings, &["keybindings"]),
    (Producer::Favorites, &["favorites"]),
    (
        Producer::Security,
        &[
            "fido2Configured",
            "fingerprintAvailable",
            "fingerprintConfigured",
            "passwordlessSudo",
            "sshdActive",
            "sshdEnabled",
            "sudolessDocker",
        ],
    ),
    (
        Producer::OmarchyInfo,
        &[
            "omarchyChannel",
            "omarchyVersion",
            "updateAvailable",
            "updateSummary",
        ],
    ),
    (
        Producer::HardwareFlags,
        &[
            "cpuIdentity",
            "cpuStat",
            "dmiFamily",
            "dmiProduct",
            "dmiVendor",
            "gpuIdentity",
            "hwAsusRog",
            "hwFramework16",
            "hwIntel",
            "hwIntelPtl",
            "hwNvidia",
            "hwNvidiaGsp",
            "hwNvidiaWithoutGsp",
            "hwSurface",
            "hwVulkan",
            "hwWebcam",
            "memoryStat",
            "npuIdentity",
        ],
    ),
    (
        Producer::GpuExtras,
        &["hybridGpuAvailable", "hybridGpuMode", "voxtypeInstalled"],
    ),
    (
        Producer::FocusPrint,
        &["cupsActive", "focusedClass", "printerSetup"],
    ),
    (
        Producer::Input,
        &[
            "hyprInput",
            "hyprInputManaged",
            "hyprWorkspaceGesture",
            "hyprWorkspaceGestureManaged",
            "hyprWorkspaceGestureUnmanaged",
        ],
    ),
];

/// The topic a producer's keys are held in (docs/adr/0011). A match with no
/// wildcard, so a new producer has to say.
fn topic(producer: Producer) -> &'static str {
    use Producer::*;
    match producer {
        Theme | LiveToggles | Nightlight | Toggles | Bar | HyprLook | Markers | Monitors
        | Devices | Dnd | Reminders | Branding | Input => super::topics::LOOK,
        Network | Bluetooth | Tailscale | Audio => super::topics::NETWORK,
        Accounts | FullName => super::topics::ACCOUNTS,
        Time | Locale | Keyboard | Hostname | ParallelDownloads | CrashCapture | Control
        | Defaults | DefaultApps => super::topics::SYSTEM,
        Apps | Atmos | Software | Snapper | Power | Weather | Hooks | MarkerFlags | Security
        | OmarchyInfo | HardwareFlags | GpuExtras | FocusPrint | Keybindings | Favorites => {
            super::topics::HOST
        }
    }
}

/// The topic that holds `key`, None for a key nobody owns.
pub fn topic_of_key(key: &str) -> Option<&'static str> {
    owner_of(key).map(topic)
}

/// Every key held in `name`.
pub fn keys_in(name: &str) -> Vec<&'static str> {
    OWNERS
        .iter()
        .filter(|(producer, _)| topic(*producer) == name)
        .flat_map(|(_, keys)| keys.iter().copied())
        .collect()
}

fn owner_of(key: &str) -> Option<Producer> {
    OWNERS
        .iter()
        .find(|(_, keys)| keys.contains(&key))
        .map(|(producer, _)| *producer)
}

/// Every key some producer owns.
pub fn known(key: &str) -> bool {
    owner_of(key).is_some()
}

fn produce(producer: Producer, env: &Env) -> Map<String, Value> {
    let mut f = Map::new();
    match producer {
        Producer::Theme => theme_fields(&mut f, true),
        Producer::LiveToggles => super::live_toggles(&mut f, true),
        Producer::Nightlight => {
            let (day, night, on) = hypr::nightlight_conf(&env.home);
            f.insert("nightlightDay".into(), json!(day));
            f.insert("nightlightNight".into(), json!(night));
            f.insert("nightlightNightOn".into(), json!(on));
        }
        Producer::Toggles => {
            f.insert("barVisible".into(), json!(toggle_off("bar-off")));
            f.insert(
                "screensaverEnabled".into(),
                json!(toggle_off("screensaver-off")),
            );
            f.insert("suspendEnabled".into(), json!(toggle_off("suspend-off")));
        }
        Producer::Bar => {
            f.extend(super::bar::read_bar(&env.home, &env.omarchy_path, &env.user).fields)
        }
        Producer::HyprLook => {
            f.insert("hyprLook".into(), hypr::look());
        }
        Producer::Markers => f.extend(hypr::markers(&env.home)),
        Producer::Monitors => {
            if let Some(rows) = hypr::monitors() {
                f.insert("monitors".into(), json!(rows));
            }
        }
        Producer::Devices => {
            if let Some(rows) = hypr::monitors_with(false) {
                f.extend(hypr::devices(&env.home, &rows).fields);
            }
        }
        Producer::Dnd => {
            f.insert("doNotDisturb".into(), json!(hypr::dnd(&env.home)));
        }
        Producer::Reminders => {
            let (count, active, list) = hypr::reminders();
            f.insert("reminderCount".into(), json!(count));
            f.insert("reminderActive".into(), json!(active));
            f.insert("reminders".into(), list);
        }
        Producer::Apps => {
            let apps = crate::inventory::apps::collect();
            f.insert("desktopApps".into(), apps["desktop"].clone());
            f.insert("tuiApps".into(), apps["tui"].clone());
            f.insert("webApps".into(), apps["web"].clone());
        }
        Producer::Branding => host::branding(&mut f, &env.home, &env.omarchy_path),
        Producer::Accounts => {
            let inv = crate::inventory::accounts::collect();
            for key in ["currentUser", "avatarPath", "users", "groups"] {
                f.insert(key.into(), inv.get(key).cloned().unwrap_or(Value::Null));
            }
        }
        Producer::FullName => {
            f.insert("fullName".into(), json!(system::full_name()));
        }
        Producer::Atmos => host::atmos(&mut f, &env.home),
        Producer::Software => host::software(&mut f, &env.home, net::tailscale_installed()),
        Producer::Snapper => host::snapper(&mut f),
        Producer::Network => f.extend(net::network()),
        Producer::Bluetooth => f.extend(net::bluetooth_fields()),
        Producer::Tailscale => f.extend(net::tailscale_fields()),
        Producer::Audio => f.extend(net::audio()),
        Producer::Power => host::power(&mut f),
        Producer::Weather => host::weather(&mut f, &env.home),
        Producer::Hooks => {
            f.insert("hooks".into(), crate::inventory::hooks::collect());
        }
        Producer::MarkerFlags => host::marker_flags(&mut f, &env.home),
        Producer::Control => f.extend(system::control(&env.home).fields),
        Producer::Time => f.extend(system::time()),
        Producer::Locale => {
            let (locale, locales) = system::locale();
            f.insert("locale".into(), json!(locale));
            f.insert("locales".into(), locales);
        }
        Producer::Keyboard => {
            let (layout, layouts) = system::keyboard();
            f.insert("keyboardLayout".into(), json!(layout));
            f.insert("keyboardLayouts".into(), layouts);
        }
        Producer::Hostname => {
            f.insert("hostname".into(), json!(system::hostname()));
        }
        Producer::ParallelDownloads => {
            f.insert(
                "parallelDownloads".into(),
                json!(system::parallel_downloads()),
            );
        }
        Producer::CrashCapture => {
            f.insert("crashCapture".into(), json!(system::crash_capture()));
        }
        Producer::Defaults => host::boot_and_defaults(&mut f, &env.home),
        Producer::DefaultApps => {
            for key in ["browser", "terminal", "editor", "agent"] {
                f.insert(key.into(), super::text(&["default", key]));
            }
        }
        Producer::Keybindings => {
            f.insert("keybindings".into(), host::keybindings());
        }
        Producer::Favorites => host::favorites(&mut f),
        Producer::Input => super::input_surface(env, &mut f),
        Producer::Security => host::security(&mut f, &env.user),
        Producer::OmarchyInfo => host::omarchy_info(&mut f),
        Producer::HardwareFlags => host::hardware_flags(&mut f),
        Producer::GpuExtras => host::gpu_extras(&mut f),
        Producer::FocusPrint => host::focus_and_print(&mut f),
    }
    f
}

/// What `read` found.
#[derive(Default)]
pub struct Read {
    pub values: Map<String, Value>,
    /// Keys nobody owns.
    pub unknown: Vec<String>,
    /// Keys whose producer could not read the machine, and why. They are not in
    /// `values`: a reader keeps the value it had, never one made up here.
    pub failed: BTreeMap<String, String>,
}

fn keys_of(producer: Producer) -> &'static [&'static str] {
    OWNERS
        .iter()
        .find(|(owner, _)| *owner == producer)
        .map_or(&[], |(_, keys)| keys)
}

/// What the machine reports now for `keys`.
pub fn read(keys: &[String]) -> Read {
    let mut wanted: Vec<Producer> = Vec::new();
    let mut found = Read::default();
    for key in keys {
        match owner_of(key) {
            Some(producer) if !wanted.contains(&producer) => wanted.push(producer),
            Some(_) => {}
            None => found.unknown.push(key.clone()),
        }
    }
    if wanted.is_empty() {
        return found;
    }
    let env = Env::read();
    // Producers are independent, and some take a while (a DDC panel, pacman),
    // so they run side by side.
    let made: Vec<(Map<String, Value>, Vec<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = wanted
            .iter()
            .map(|producer| {
                let env = &env;
                scope.spawn(move || sh::watching(|| produce(*producer, env)))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or_default())
            .collect()
    });
    for (producer, (map, faults)) in wanted.iter().zip(made) {
        if !faults.is_empty() {
            let why = faults.join("; ");
            for key in keys_of(*producer)
                .iter()
                .filter(|k| keys.iter().any(|w| w == *k))
            {
                found.failed.insert(key.to_string(), why.clone());
            }
            continue;
        }
        for (key, value) in map {
            if keys.contains(&key) {
                found.values.insert(key, value);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_has_one_owner() {
        let mut seen = std::collections::HashSet::new();
        for (_, keys) in OWNERS {
            for key in *keys {
                assert!(seen.insert(*key), "{key} is owned twice");
            }
        }
    }

    #[test]
    fn unknown_keys_are_reported_and_known_ones_are_not() {
        let found = read(&["nope".into(), "also.nope".into()]);
        assert_eq!(found.unknown, ["nope", "also.nope"]);
        assert!(known("audioOutputMuted"));
        assert!(known("plugins"));
        assert!(!known("tweaks.middlePaste"));
    }

    #[test]
    fn nothing_is_run_for_no_keys() {
        let found = read(&[]);
        assert!(found.values.is_empty() && found.unknown.is_empty() && found.failed.is_empty());
    }
}

#[cfg(test)]
mod probe {
    use super::*;

    /// Prints the keys each producer really yields on this machine, to check
    /// `OWNERS` against: `cargo test probe -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn print_what_each_producer_yields() {
        let env = Env::read();
        for (producer, listed) in OWNERS {
            let made = produce(*producer, &env);
            let found: Vec<&String> = made.keys().collect();
            let missing: Vec<&&str> = listed.iter().filter(|k| !made.contains_key(**k)).collect();
            let extra: Vec<&&String> = found
                .iter()
                .filter(|k| !listed.contains(&k.as_str()))
                .collect();
            println!("{producer:?} not produced: {missing:?} unlisted: {extra:?}");
        }
    }
}
