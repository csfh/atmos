//! The rows a page module draws itself (docs/adr/0021): what their QML does is page logic the
//! generic controls do not hold, such as a description built from an inventory or a button
//! whose label and confirm follow what the snapshot says. Each names the fields and inventories
//! it reads, so its page holds them, and the setters it calls, so `tests/catalog_setters.rs`
//! runs each against `Omarchy.qml`. `pages/custom.rs` draws them, and `tests/catalog_custom.rs`
//! holds each to its QML.

use super::control::{Field, When};
use super::setter::Setter;
use crate::{appearance, capture, environment, kernel, rules, startup, workspaces};

/// `resetPlymouth()`: none while the stock theme is set.
pub const BOOT_RESET: Setter = super::setter::setter(
    "resetPlymouth",
    super::setter::Args::Nothing,
    super::setter::ANY.only(When::IsNot(Field::Key("plymouth"), "default")),
    super::setter::Sends::Job(
        super::setter::task("plymouth.reset", "plymouth-reset", "The boot screen reset").sudo(),
    ),
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Custom {
    /// Kernel: the running kernel, from the diagnostics inventory, and Copy.
    RunningKernel,
    /// Kernel: Set up… or Remove…, with the confirm that goes with each.
    DirectBoot,
    /// Kernel: the lower swappiness tweak's switch and Reset.
    Swappiness,
    /// The Export page: a row of its own, by part (`pages/export.rs`).
    Export(&'static str),
    /// Idle's Lock timeout, which sends what `setIdle` sends (`system::idle_lock_write`).
    IdleLock,
    /// Notifications: minutes, a message, and Set (`setReminder`).
    NewReminder,
    /// MCP: install for every installed agent, and Check.
    McpInstall,
    /// MCP: "None" while no agent is installed.
    McpNone,
    /// Environment: a detected value (`session`, `shell`, `tools`, `path`).
    Detected(&'static str),
    /// Environment: the PATH prepend and Set.
    PathPrepend,
    /// Environment: a name, a value, and Add.
    AddVar,
    /// Startup: a command, a delay, and Add.
    AddAutostart,
    /// Startup: "No failed units reported." while none failed.
    NoFailures,
    /// Window rules: Add…, which opens its dialog.
    AddRule,
    /// Window rules: "No personal window rules." while there are none.
    NoRules,
    /// Window rules: Add's Center switch, which the dialog draws.
    RuleCenter,
    /// Workspaces: how many the bar shows (`setCount`).
    BarCount,
    /// Workspaces: Wrap switching or Super + mouse wheel, by its key (see `workspaces.rs`).
    WorkspaceSwitch(&'static str),
    /// Workspaces: which one Hyprland opens on login.
    LoginWorkspace,
    /// Workspaces: a name, and Add (`addSpecial`).
    AddScratch,
    /// Appearance: a button for each theme (see `pages/appearance.rs` for what the cards lack).
    Themes,
    /// Appearance: "No extra themes installed." while there are none.
    NoExtraThemes,
    /// Appearance: the extra themes, with Update all and Remove….
    ExtraThemes,
    /// Appearance: Add…, which opens its dialog.
    AddTheme,
    /// Appearance: the night light's day and night times, and Set.
    NightSchedule,
    /// Appearance: whether the schedule is used.
    UseSchedule,
    /// Capture: the screenshot's mode, its place, or the webcam's size, which the page holds.
    CapturePick(&'static str),
    /// Capture: Capture, with the mode and place picked.
    TakeShot,
    /// Capture: a recording switch the page holds, by its name there (`recDesktopAudio`, ...).
    /// Start, which the Recording list draws, sends what they hold.
    RecSwitch(&'static str),
    /// Capture: the webcam overlay's Smaller, Larger and Reset.
    ResizeWebcam,
    /// Capture: a save folder, by its key, with Open folder and Copy.
    SaveFolder(&'static str),
    /// Hardware: a row of the page's own, by part (`identity`, `refresh`, `bios`, `secureBoot`,
    /// `tpm`, `installed`), from the hardware inventory.
    Hw(&'static str),
    /// Software: a language, Install… and Remove… (`askDev`).
    DevEnv,
    /// Software: a Docker database, and Install, while Docker is installed.
    DockerDb,
    /// Speed test: a row of the run, by part (`run`, `progress`, `error`, `down`, `up`).
    Speed(&'static str),
    /// Applications: a command, and Add (`addAutostart(command)`).
    AppAutostart,
    /// Applications: "No launch-on-start commands." while there are none.
    AppNoAutostart,
    /// Applications: Add… for a kind of launcher (`desktop`, `tui`, `web`), in its dialog.
    AddLauncher(&'static str),
    /// Applications: a kind's empty row, while it has no launchers.
    NoLaunchers(&'static str),
    /// History: a row of the page's own, by part (`preview`, `held`, `none`, `you`), from the
    /// window's log of changes.
    Hist(&'static str),
    /// Disks: a row of the page's own, by part (`listed` for one each drive's group draws,
    /// `snapper`, `nosnapshots`, `hibernation`, `limit`, `timeline`, `fstrim`).
    Disk(&'static str),
    /// Displays: a row of the page's own, by part (`listed` for one each monitor's group draws,
    /// `layout`, `touchpad`, `backlight`).
    Display(&'static str),
    /// Monitor's pages: a row's value and sparkline, by row (`busy`, `used`, `iopsi`, ...), or a
    /// list's empty row (`nohot`, `nodisks`, ...), from the live samples.
    Meter(&'static str),
    /// Home: an activity row's caption and sparkline, by what it draws (`cpu`, `mem`, `net`,
    /// `temp`), from the live samples.
    Live(&'static str),
    /// Accounts: a row of the page's own, by part (`fullname`, `password`, `adduser`,
    /// `nologins`, `addgroup`, `nogroups`).
    Acct(&'static str),
    /// Bluetooth: a row of the page's own, by part (`nopaired`, `discoverable`, `scan`,
    /// `nonearby`).
    Bt(&'static str),
    /// Wi-Fi: a row of the page's own, by part (`qr`, `nosaved`, `nearby`, `joinerror`; `listed`
    /// is drawn by the saved list).
    Wifi(&'static str),
    /// Network: a row of the page's own, by part (`address`, `gateway`, `dns`, `wifi`,
    /// `wireguard`, `hotspot`, `provider`, `servers`, `lsfile`, `lsfolder`, `taildrop`).
    Net(&'static str),
    /// Keybindings: a row of the page's own, by part (`add`, `none`, `empty`, `filter`,
    /// `nomatch`, and `unbind` and `more`, which the dialog and the bound list draw).
    Bind(&'static str),
    /// Idle: the screensaver's logo, with Choose…, Edit, and Reset while it is custom.
    Logo(&'static str),
    /// Input: a Hyprland layouts row, by part (`layouts`, `variants`, `toggle` for Alt+Alt).
    Kb(&'static str),
    /// Diagnostics: a row of the page's own, by part (`listed` for one its group's list draws
    /// once per item, `copy`, `ask`, `refresh`, `nofailed`, `root`, `ram`, `swap`, `kernel`,
    /// `driver`, `portals`, `pipewire`, `connectivity`, `packages`, `noerrors`).
    Diag(&'static str),
    /// Bar: a row of the page's own, by part (`hiddenNone`, `hidden`, `pinnedNone`, `pinned`,
    /// `indicators`, `refresh`, `syncfile`, `device`, `clock`, `clockAlt`, `week`, `birth`,
    /// `life`, `pluginsNone`).
    Bar(&'static str),
    /// System: a row of the page's own, by part (`hostname`, `ochannel`, `oupdates`, `achannel`,
    /// `aupdates`, `integration`, `printers`, `about`, `location`, `coords`).
    Sys(&'static str),
    /// A button that opens the file chooser and hands the path to a setter, by id
    /// (`catalog/picks.rs`).
    Choose(&'static str),
    /// Boot: Reset…, while a custom unlock theme is set.
    BootReset,
    /// A row its group's list draws, once for each item (`hidden here`).
    InList,
    /// Hooks: Add…, which opens its dialog.
    HookAdd,
}

impl Custom {
    /// Whether it builds its row's words from what the window holds (`pages/custom.rs`
    /// `description` gives words to no other).
    pub fn speaks(self) -> bool {
        matches!(
            self,
            Custom::RunningKernel
                | Custom::Swappiness
                | Custom::Export(
                    "sections"
                        | "where"
                        | "write"
                        | "file"
                        | "review"
                        | "changes"
                        | "apply"
                        | "applying"
                        | "putback"
                )
                | Custom::Detected("path")
                | Custom::AddVar
                | Custom::AddScratch
                | Custom::ExtraThemes
                | Custom::AddTheme
                | Custom::NightSchedule
                | Custom::UseSchedule
                | Custom::SaveFolder(_)
                | Custom::Hw(_)
                | Custom::Speed(_)
                | Custom::Hist(_)
                | Custom::Disk(_)
                | Custom::Kb(_)
                | Custom::Bind(_)
                | Custom::Net(_)
                | Custom::Wifi("qr" | "nearby" | "joinerror")
                | Custom::Bt("scan")
                | Custom::Acct(_)
                | Custom::Meter("nodisks" | "noifaces" | "nosensors")
                | Custom::Logo(_)
                | Custom::Display("backlight")
                | Custom::Diag(_)
                | Custom::Bar(_)
                | Custom::Sys(_)
        )
    }

    /// Whether it reads the live samples.
    pub fn live(self) -> bool {
        matches!(self, Custom::Live(_) | Custom::Meter(_))
    }

    /// The keys it reads that no field names: the Export page's, the whole snapshot an Omafile
    /// is written from and planned against.
    pub fn held(self) -> Vec<crate::settings::Held> {
        match self {
            Custom::Export(_) => crate::export::held(),
            _ => Vec::new(),
        }
    }

    /// Whether its control draws its group's model (the QML row's own `Repeater`).
    pub fn draws_model(self) -> bool {
        matches!(
            self,
            Custom::Themes | Custom::Bar("indicators") | Custom::Wifi("qr")
        )
    }

    /// The snapshot keys it reads.
    pub fn fields(self) -> Vec<Field> {
        match self {
            Custom::RunningKernel => Vec::new(),
            Custom::DirectBoot => vec![kernel::DIRECT, kernel::AVAILABLE],
            Custom::Swappiness => vec![Field::Key("tweaks")],
            Custom::IdleLock => vec![Field::Key("idleLock"), Field::Key("idleScreensaver")],
            Custom::Export(_) => Vec::new(),
            Custom::NewReminder | Custom::McpInstall | Custom::McpNone => Vec::new(),
            Custom::Detected(_) => vec![environment::DETECTED],
            Custom::PathPrepend | Custom::AddVar => vec![environment::VARS, environment::PREPEND],
            Custom::AddAutostart => vec![startup::AUTOSTART],
            Custom::NoFailures | Custom::RuleCenter => Vec::new(),
            Custom::AddRule => vec![rules::RULES, rules::FOCUSED],
            Custom::NoRules => vec![rules::RULES],
            Custom::BarCount => vec![workspaces::COUNT],
            Custom::WorkspaceSwitch(key) => vec![Field::Key(key)],
            Custom::LoginWorkspace | Custom::AddScratch => vec![workspaces::ITEMS],
            Custom::Themes => vec![appearance::THEME, appearance::THEMES],
            Custom::NoExtraThemes => vec![appearance::EXTRAS],
            Custom::ExtraThemes => vec![appearance::EXTRAS, appearance::THEME],
            Custom::AddTheme => Vec::new(),
            Custom::NightSchedule | Custom::UseSchedule => {
                vec![appearance::DAY, appearance::NIGHT, appearance::NIGHT_ON]
            }
            Custom::CapturePick(_) | Custom::RecSwitch(_) => vec![capture::RECORDING],
            Custom::TakeShot => Vec::new(),
            Custom::ResizeWebcam => vec![capture::OVERLAY],
            Custom::SaveFolder(key) => vec![Field::Key(key)],
            Custom::Hw(_) => Vec::new(),
            Custom::DevEnv => Vec::new(),
            Custom::DockerDb => vec![Field::Key("extras")],
            Custom::Speed(_) => vec![crate::speedtest::NET],
            Custom::AppAutostart | Custom::AppNoAutostart => vec![startup::AUTOSTART],
            Custom::AddLauncher(_) => Vec::new(),
            Custom::NoLaunchers(kind) => vec![crate::launchers::list(kind)],
            Custom::Hist(_) => Vec::new(),
            Custom::Diag(_) => vec![Field::Key("hostname")],
            Custom::Choose(_) | Custom::InList | Custom::HookAdd => Vec::new(),
            Custom::BootReset => vec![Field::Key("plymouth")],
            Custom::Sys(_) => [
                "hostname",
                "omarchyChannel",
                "updateAvailable",
                "updateSummary",
                "atmosChannel",
                "atmosPackaged",
                "cupsActive",
                "printerSetup",
                "aboutBranded",
                "weatherAuto",
                "weatherLocation",
                "weatherCoords",
                "keyboardLayout",
                "keyboardLayouts",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
            Custom::Bar(_) => [
                "trayPresent",
                "trayHidden",
                "trayPinned",
                "indicatorsPresent",
                "indicatorsItems",
                "agentsPresent",
                "agentsRefreshIntervalSec",
                "agentsSyncDir",
                "agentsSyncFileName",
                "agentsSyncDeviceId",
                "clockPresent",
                "clockFormat",
                "clockFormatAlt",
                "clockWeekStart",
                "clockBirthYear",
                "clockLifeExpectancy",
                "barPosition",
                "plugins",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
            Custom::Logo(_) => vec![Field::Key("screensaverBranded")],
            Custom::Live(_) | Custom::Meter(_) => Vec::new(),
            Custom::Bt(_) => ["bluetooth", "bluetoothDevices", "bluetoothDiscoverable"]
                .into_iter()
                .map(Field::Key)
                .collect(),
            Custom::Wifi(_) => [
                "wifiConnected",
                "wifiIface",
                "wifiConnections",
                "wifiHw",
                "wifiRadio",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
            Custom::Acct(_) => vec![
                crate::accounts::USERS,
                crate::accounts::GROUPS,
                crate::accounts::CURRENT,
                crate::accounts::FULL_NAME,
            ],
            Custom::Net(_) => [
                "netKind",
                "netSsid",
                "netIface",
                "netSpeed",
                "netSignal",
                "netIp",
                "netGateway",
                "netDnsServers",
                "wifiHw",
                "wifiRadio",
                "dns",
                "extras",
                "tailscaleInstalled",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
            Custom::Bind(_) => vec![crate::bindings::BINDINGS, crate::bindings::KEYBINDINGS],
            Custom::Kb(_) => vec![
                crate::keyboard::INPUT_KEY,
                crate::keyboard::GESTURE_UNMANAGED,
            ],
            Custom::Display(part) => match part {
                "layout" => vec![
                    crate::display::MONITORS,
                    crate::display::RULES,
                    Field::Key("internalPresent"),
                    Field::Key("externalPresent"),
                ],
                "touchpad" => vec![Field::Key("touchpadEnabled"), Field::Key("touchpadPresent")],
                "backlight" => vec![
                    Field::Key("keyboardBacklightPresent"),
                    Field::Key("keyboardBrightness"),
                ],
                _ => vec![crate::display::MONITORS, crate::display::RULES],
            },
            Custom::Disk(_) => [
                "snapperPresent",
                "snapperConfigs",
                "hibernationConfigured",
                "hibernationSupported",
                "snapperNumberLimit",
                "snapperTimeline",
                "fstrimEnabled",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
        }
    }

    /// The inventories it reads.
    pub fn inventories(self) -> &'static [&'static str] {
        match self {
            Custom::RunningKernel => &["diagnostics"],
            Custom::DirectBoot
            | Custom::Swappiness
            | Custom::IdleLock
            | Custom::Export(_)
            | Custom::NewReminder
            | Custom::McpInstall
            | Custom::McpNone
            | Custom::Detected(_)
            | Custom::PathPrepend
            | Custom::AddVar
            | Custom::AddAutostart
            | Custom::AddRule
            | Custom::NoRules
            | Custom::RuleCenter
            | Custom::BarCount
            | Custom::WorkspaceSwitch(_)
            | Custom::LoginWorkspace
            | Custom::AddScratch
            | Custom::Themes
            | Custom::NoExtraThemes
            | Custom::ExtraThemes
            | Custom::AddTheme
            | Custom::NightSchedule
            | Custom::UseSchedule
            | Custom::CapturePick(_)
            | Custom::TakeShot
            | Custom::RecSwitch(_)
            | Custom::ResizeWebcam
            | Custom::SaveFolder(_) => &[],
            Custom::NoFailures => &["services"],
            Custom::Hw(_) => &["hardware"],
            Custom::DevEnv | Custom::DockerDb | Custom::Speed(_) => &[],
            Custom::AppAutostart
            | Custom::AppNoAutostart
            | Custom::AddLauncher(_)
            | Custom::NoLaunchers(_)
            | Custom::Hist(_) => &[],
            Custom::Bind(_) => &[],
            Custom::Bt(_) => &[],
            Custom::Wifi(_) => &[],
            Custom::Meter(_) => &[],
            Custom::Live(_) => &[],
            Custom::Acct(_) => &[],
            Custom::Net(_) => &[],
            Custom::Logo(_) => &[],
            Custom::Kb(_) => &[],
            Custom::Display(_) => &[],
            Custom::Disk(_) => &["disks"],
            Custom::Diag(_) => &["diagnostics"],
            Custom::Bar(_) => &[],
            Custom::Sys(_) => &["diagnostics"],
            Custom::Choose(_) | Custom::BootReset | Custom::InList | Custom::HookAdd => &[],
        }
    }

    /// The setters it calls.
    pub fn setters(self) -> Vec<Setter> {
        match self {
            Custom::RunningKernel => Vec::new(),
            Custom::DirectBoot => vec![kernel::SETUP_DIRECT_BOOT],
            Custom::Swappiness => vec![kernel::SET_SWAPPINESS],
            Custom::IdleLock | Custom::Export(_) => Vec::new(),
            Custom::Themes => vec![appearance::SET_THEME],
            Custom::ExtraThemes => vec![
                appearance::SET_THEME,
                appearance::UPDATE_THEMES,
                appearance::REMOVE_THEME,
            ],
            Custom::AddTheme => vec![appearance::INSTALL_THEME],
            Custom::Choose(id) => vec![super::picks::pick(id).setter],
            Custom::BootReset => vec![BOOT_RESET],
            _ => Vec::new(),
        }
    }
}
