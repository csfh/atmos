//! The pages not yet ported (docs/adr/0021), as their QML pages have them: each page's groups in
//! order, with the label, description, hint and keywords of every row whose words are literals in
//! the QML, the rows `search.query` reads. `pages/catalog.rs` draws them. A row is drawn with
//! its control once it is mapped (`Control`), and without one until then.
//! `tests/catalog_parity.rs` holds every page to its QML file and to `search.query`, both ways.
//!
//! A row the QML builds from a model at run time (a Wi-Fi network, a process) is in no page here,
//! as it is in no catalog.

mod admin;
mod apps;
pub mod control;
pub mod custom;
mod device;
mod framed;
mod general;
mod home;
mod input;
mod keys;
mod look;
mod models;
pub mod picks;
mod sample_forms;
mod sample_pages;
pub mod samples;
pub mod setter;
mod wide;
pub mod words;

pub use control::{Control, Field, When};
pub use framed::{FRAMED, framed};
pub use models::MODEL_LISTS;
pub use wide::{WIDE, wide};

use crate::hubs;
use crate::search::Row;
use crate::settings::Held;

/// A page: a hub's own page or one of its pages, by the id `Hubs.js` gives it.
#[derive(Debug)]
pub struct Page {
    pub id: &'static str,
    /// Its QML file under `pages/`.
    pub file: &'static str,
    /// What the QML page says of itself, when that is a literal; empty when it is built at run
    /// time, and the hub's description is shown instead.
    pub description: &'static str,
    pub groups: &'static [Group],
}

/// A group of rows under its title and what it says of them. The rows the QML page has outside
/// every group are in one with no title, first.
#[derive(Debug)]
pub struct Group {
    pub title: &'static str,
    pub detail: &'static str,
    pub rows: &'static [Line],
}

/// A list a group draws besides its rows: what the QML builds with a `Repeater`, which no
/// catalog row holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    /// The built-in profiles, each with Apply (`profiles.rs`).
    Profiles,
    /// The waiting reminders, each with Copy and Show (`notifications.rs`).
    Reminders,
    /// Agentic: the coding agents, each with Use or Install (`agentic.rs`).
    Agents,
    /// MCP: the installed agents, each with its entry's switch and Replace.
    McpAgents,
    /// Environment: the variables Atmos writes, each with Remove.
    EnvVars,
    /// Startup: the commands Hyprland launches, each with its switch and Remove.
    Autostart,
    /// Startup: the units that failed this boot.
    Failures,
    /// Window rules: each rule, with Remove.
    Rules,
    /// Workspaces: each numbered one, with its name and monitor.
    WorkspaceNames,
    /// Workspaces: each scratch one, with Remove.
    Scratch,
    /// Capture: Start, or Stop while a recording runs (see `pages/capture.rs`).
    Recording,
    /// Hardware: a part's rows, by part (`machine`, `board`, `modules`, ...); the machine's leads
    /// its group.
    Hardware(&'static str),
    /// Software: a group's stacks, by group, each with Install… or Remove….
    Software(&'static str),
    /// Services: the summary, the search and filters, and each unit (`pages/units.rs`).
    Units,
    /// Applications: the launch-on-start commands, each with Remove….
    AppAutostart,
    /// Applications: the launchers of a kind (`desktop`, `tui`, `web`), each with Remove….
    Launchers(&'static str),
    /// History: what Preview holds.
    Held,
    /// History: each change this window made.
    Changes,
    /// Disks: a list of the disks inventory, by name (`drives`, `luks`, `snapshots`, `swap`).
    Disk(&'static str),
    /// Monitor's pages: a list drawn from the live samples, by name (`corerows`, `cpuhist`,
    /// `cpuhot`, `memmix`, `fat`, `disks`, `io`, `ifaces`, `sgpus`, `sensors`).
    Meters(&'static str),
    /// Home and the Monitor hub: a group drawn from the live samples, by name (`quick`, `hero`,
    /// `gpus`, `procs`; `sampling`, `now`, `cores`, `pages`, `hottest`).
    Live(&'static str),
    /// Accounts: this login's face (`face`), each user (`users`), each group (`groups`), or the
    /// members of the group Manage opens, which its dialog draws (`members`).
    Acct(&'static str),
    /// Bluetooth: the paired devices (`paired`), or the nearby ones a scan found (`nearby`).
    Bt(&'static str),
    /// Export: a row per section an Omafile can carry.
    Export,
    /// Wi-Fi: the saved connections (`saved`), or the nearby networks a scan found (`nearby`).
    Wifi(&'static str),
    /// Network: the connection's row, whose label is what it is.
    Link,
    /// Background: the current wallpaper's row, with its preview.
    Wallpaper,
    /// Keybindings: the overrides (`overrides`), or what Hyprland has bound (`bound`).
    Bindings(&'static str),
    /// Displays: each monitor as a group of its own (`monitors`), or the row that says there is
    /// none (`empty`).
    Display(&'static str),
    /// Diagnostics: a group's rows, by group (`stale`, `omarchy`, `atmos`, `hyprland`, `failed`,
    /// `errors`).
    Diag(&'static str),
    /// Bar: the plugins, each with its switch.
    Plugins,
    /// System: a version's row, whose label is the version (`omarchy`, `atmos`).
    Sys(&'static str),
    /// Hooks: each event's group, with its scripts.
    Hooks,
}

impl List {
    /// Whether it reads the live samples.
    pub fn live(self) -> bool {
        matches!(self, List::Meters(_))
            || matches!(self, List::Live(name) if !matches!(name, "quick" | "pages"))
    }

    /// The row of the group its rows follow, when they go between the group's own (the
    /// sections an Omafile carries come after Sections).
    pub fn follows(self) -> Option<&'static str> {
        match self {
            List::Export => Some("Sections"),
            _ => None,
        }
    }

    /// Whether its rows come before the group's own (the machine's row, then Identity).
    pub fn leads(self) -> bool {
        matches!(
            self,
            List::Hardware("machine")
                | List::Sys(_)
                | List::Wallpaper
                | List::Link
                | List::Acct("face")
                | List::Live("hero" | "procs" | "hottest")
                | List::Meters(_)
        )
    }
}

impl List {
    /// The snapshot keys it reads.
    pub fn fields(self) -> Vec<Field> {
        match self {
            List::Profiles => Vec::new(),
            List::Reminders => vec![crate::notifications::REMINDERS],
            List::Agents => vec![Field::Key("agent")],
            List::McpAgents => Vec::new(),
            List::EnvVars => vec![crate::environment::VARS, crate::environment::PREPEND],
            List::Autostart => vec![crate::startup::AUTOSTART],
            List::Failures => Vec::new(),
            List::Rules => vec![crate::rules::RULES],
            List::WorkspaceNames => vec![crate::workspaces::ITEMS, crate::workspaces::MONITORS],
            List::Scratch => vec![crate::workspaces::ITEMS],
            List::Recording => vec![crate::capture::RECORDING],
            List::Hardware("batteries") => vec![Field::Key("batteryPresent")],
            List::Hardware(_) => Vec::new(),
            List::Software(_) => crate::software::BAGS
                .iter()
                .map(|b| Field::Key(b))
                .collect(),
            List::Units => vec![crate::units::UNITS],
            List::AppAutostart => vec![crate::startup::AUTOSTART],
            List::Launchers(kind) => vec![crate::launchers::list(kind)],
            List::Held | List::Changes => Vec::new(),
            List::Disk(_) => vec![Field::Key("snapperPresent")],
            List::Display(_) => vec![crate::display::MONITORS, crate::display::RULES],
            List::Bindings(_) => vec![crate::bindings::BINDINGS, crate::bindings::KEYBINDINGS],
            List::Wallpaper => vec![Field::Key("background")],
            List::Live(_) | List::Meters(_) => Vec::new(),
            List::Wifi(_) => vec![Field::Key("wifiConnections")],
            List::Export => Vec::new(),
            List::Bt(_) => vec![Field::Key("bluetoothDevices")],
            List::Acct(_) => vec![
                crate::accounts::USERS,
                crate::accounts::GROUPS,
                crate::accounts::CURRENT,
                crate::accounts::AVATAR,
            ],
            List::Link => ["netKind", "netSsid", "netIface", "netSpeed", "netSignal"]
                .into_iter()
                .map(Field::Key)
                .collect(),
            List::Plugins => vec![Field::Key("plugins")],
            List::Hooks => ["hooks", "theme", "font"]
                .into_iter()
                .map(Field::Key)
                .collect(),
            List::Sys(_) => [
                "omarchyVersion",
                "omarchyChannel",
                "atmosPackaged",
                "atmosVersion",
                "atmosRevision",
                "atmosInstalled",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
            List::Diag(_) => [
                "omarchyVersion",
                "atmosVersion",
                "atmosRevision",
                "atmosPackaged",
            ]
            .into_iter()
            .map(Field::Key)
            .collect(),
        }
    }

    /// The inventories it reads.
    pub fn inventories(self) -> &'static [&'static str] {
        match self {
            List::Meters("sgpus") => &["hardware"],
            List::Live("gpus") => &["hardware"],
            List::Failures => &["services"],
            List::Hardware(_) => &["hardware"],
            List::Disk(_) => &["disks"],
            List::Diag(_) => &["diagnostics"],
            _ => &[],
        }
    }
}

/// The groups that draw a list, by page and group title.
pub const LISTS: &[(&str, &str, List)] = &[
    ("profiles", "Built-in", List::Profiles),
    ("notifications", "Reminders", List::Reminders),
    ("agentic", "Agents", List::Agents),
    ("agentic/mcp", "Installed agents", List::McpAgents),
    ("system/environment", "Overlay", List::EnvVars),
    ("applications/startup", "Launch on start", List::Autostart),
    ("applications/startup", "Failures", List::Failures),
    ("windows/rules", "Your rules", List::Rules),
    ("workspaces", "Names and monitors", List::WorkspaceNames),
    ("workspaces", "Scratch workspaces", List::Scratch),
    ("capture", "Recording", List::Recording),
    ("hardware", "Machine", List::Hardware("machine")),
    ("hardware", "Motherboard", List::Hardware("board")),
    ("hardware", "Chipset", List::Hardware("chipset")),
    ("hardware", "Processor", List::Hardware("cpu")),
    ("hardware", "Memory", List::Hardware("modules")),
    ("hardware", "NPU", List::Hardware("npus")),
    ("hardware", "Network adapters", List::Hardware("nics")),
    ("hardware", "Audio", List::Hardware("audio")),
    ("hardware", "USB", List::Hardware("usb")),
    ("hardware", "Battery", List::Hardware("batteries")),
    ("hardware", "Thermal", List::Hardware("thermals")),
    ("hardware", "Virtualization", List::Hardware("virt")),
    ("software", "Browsers", List::Software("browsers")),
    ("software", "Terminals", List::Software("terminals")),
    ("software", "Editors", List::Software("editors")),
    ("software", "Services", List::Software("services")),
    ("software", "Gaming", List::Software("gaming")),
    ("software", "Development", List::Software("development")),
    ("services", "", List::Units),
    ("applications", "Advanced", List::AppAutostart),
    ("applications", "Desktop", List::Launchers("desktop")),
    ("applications", "Terminal", List::Launchers("tui")),
    ("applications", "Web", List::Launchers("web")),
    ("system/history", "Preview", List::Held),
    ("system/history", "Changes", List::Changes),
    ("disks", "", List::Disk("drives")),
    ("display", "", List::Display("monitors")),
    ("display", "Displays", List::Display("empty")),
    ("appearance/background", "Wallpaper", List::Wallpaper),
    ("network", "Connection", List::Link),
    ("export", "Export", List::Export),
    ("network/wifi", "Networks", List::Wifi("nearby")),
    ("bluetooth", "Paired", List::Bt("paired")),
    ("bluetooth", "Nearby", List::Bt("nearby")),
    ("network/wifi", "Saved connections", List::Wifi("saved")),
    ("accounts", "This account", List::Acct("face")),
    ("home", "Quick access", List::Live("quick")),
    ("home", "Activity", List::Live("hero")),
    ("home", "Thermal", List::Live("gpus")),
    ("home", "Processes", List::Live("procs")),
    ("monitor", "Sampling", List::Live("sampling")),
    ("monitor", "Cores", List::Live("cores")),
    ("monitor", "Pages", List::Live("pages")),
    ("monitor", "Hottest tasks", List::Live("hottest")),
    ("monitor/cpu", "IRQ land", List::Meters("heatmap")),
    ("monitor/cpu", "PSI ridgeline", List::Meters("ridgeline")),
    ("monitor/cpu", "Cores", List::Meters("corerows")),
    ("monitor/cpu", "Core frequency", List::Meters("violin")),
    ("monitor/cpu", "Softirq rose", List::Meters("rose")),
    ("monitor/cpu", "Histogram", List::Meters("cpuhist")),
    ("monitor/cpu", "Hottest", List::Meters("cpuhot")),
    ("monitor/memory", "Composition", List::Meters("memmix")),
    ("monitor/memory", "Buddy horizon", List::Meters("horizon")),
    ("monitor/memory", "RSS treemap", List::Meters("treemap")),
    ("monitor/memory", "Meminfo Sankey", List::Meters("sankey")),
    (
        "monitor/memory",
        "Cgroup sunburst",
        List::Meters("sunburst"),
    ),
    ("monitor/memory", "Slab icicle", List::Meters("icicle")),
    ("monitor/memory", "Largest", List::Meters("fat")),
    ("monitor/disk", "I/O streamgraph", List::Meters("stream")),
    ("monitor/disk", "Devices", List::Meters("disks")),
    ("monitor/disk", "Busiest tasks", List::Meters("io")),
    (
        "monitor/processes",
        "Parallel coordinates",
        List::Meters("parallel"),
    ),
    ("monitor/processes", "", List::Meters("table")),
    ("monitor/traffic", "TCP beeswarm", List::Meters("bees")),
    ("monitor/traffic", "Netdev radar", List::Meters("radar")),
    ("monitor/traffic", "Interfaces", List::Meters("ifaces")),
    (
        "monitor/sensors",
        "Thermal calendar",
        List::Meters("calendar"),
    ),
    (
        "monitor/sensors",
        "RAPL waterfall",
        List::Meters("waterfall"),
    ),
    ("monitor/sensors", "Graphics", List::Meters("sgpus")),
    ("monitor/sensors", "hwmon", List::Meters("sensors")),
    ("accounts", "Users", List::Acct("users")),
    ("accounts", "Groups", List::Acct("groups")),
    ("accounts", "", List::Acct("members")),
    ("keybindings", "Your overrides", List::Bindings("overrides")),
    ("keybindings", "What is bound", List::Bindings("bound")),
    ("disks", "Encryption", List::Disk("luks")),
    ("disks", "Snapshots", List::Disk("snapshots")),
    ("disks", "Swap", List::Disk("swap")),
    (
        "system/diagnostics",
        "Settings that could not be read",
        List::Diag("stale"),
    ),
    ("system/diagnostics", "Omarchy", List::Diag("omarchy")),
    ("system/diagnostics", "Atmos", List::Diag("atmos")),
    ("system/diagnostics", "Hyprland", List::Diag("hyprland")),
    ("system/diagnostics", "Failed units", List::Diag("failed")),
    ("system/diagnostics", "Recent errors", List::Diag("errors")),
    ("bar", "Advanced", List::Plugins),
    ("system", "Updates", List::Sys("omarchy")),
    ("system", "Atmos", List::Sys("atmos")),
    ("hooks", "", List::Hooks),
];

/// The list a group draws, if any.
pub fn list(page: &str, group: &str) -> Option<List> {
    LISTS
        .iter()
        .find(|(p, g, _)| *p == page && *g == group)
        .map(|(_, _, l)| *l)
}

/// The groups a QML page keeps out of the catalog (`catalog: false`) that the window draws all
/// the same: by page, the group they follow, their title and words, and the list that draws
/// them. `tests/catalog_parity.rs` holds this to the QML.
pub const UNLISTED: &[(&str, &str, &str, &str, List)] = &[
    (
        "monitor",
        "Sampling",
        "Now",
        "Meters share the live sample. Hot ink is 90% and up.",
        List::Live("now"),
    ),
    (
        "monitor/processes",
        "Parallel coordinates",
        "",
        "Filters stay in this window. They are not written to disk.",
        List::Meters("table"),
    ),
];

/// What a group of `SHOWN` asks: a condition on the snapshot, or one the page module answers
/// from an inventory or a list of its own, by name (`pages/custom.rs` `group_shows`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shown {
    When(When),
    Page(&'static str),
}

/// The groups whose QML says `query: cond ? root.query : "."`, by page and group title, with
/// the condition as the QML has it: while it is false the group is drawn only when one of its
/// rows is (`PrefsGroup`'s `visible`). `tests/catalog_parity.rs` holds this to the QML both
/// ways, and runs each `When` against the QML's condition under Node.
#[rustfmt::skip]
pub const SHOWN: &[(&str, &str, &str, Shown)] = &[
    ("hardware", "Memory", "root.hw.memory.total > 0 || root.hw.memory.modules.length > 0", Shown::Page("memory")),
    ("display", "Displays", "Omarchy.monitors.length === 0", Shown::Page("nomonitors")),
    ("display", "Layouts", "Omarchy.monitors.length", Shown::When(When::Has(Field::Key("monitors")))),
    ("display", "Laptop", "(Omarchy.internalPresent || Omarchy.keyboardBacklightPresent || Omarchy.touchpadPresent || Omarchy.touchscreenPresent)", Shown::When(When::Any(&[When::On(Field::Key("internalPresent")), When::On(Field::Key("keyboardBacklightPresent")), When::On(Field::Key("touchpadPresent")), When::On(Field::Key("touchscreenPresent"))]))),
    ("disks", "Encryption", "Omarchy.luksDevices.length > 0", Shown::Page("luks")),
    ("disks", "Snapshots", "Omarchy.snapperPresent", Shown::When(When::On(Field::Key("snapperPresent")))),
    ("disks", "Swap", "Omarchy.swapDevices.length > 0", Shown::Page("swap")),
    ("power", "Hardware", "(Omarchy.powerGovernor || Omarchy.amdPstate || Omarchy.chargeLimitAvailable)", Shown::When(When::Any(&[When::Has(Field::Key("powerGovernor")), When::Has(Field::Key("amdPstate")), When::On(Field::Key("chargeLimitAvailable"))]))),
    ("power", "Battery", "(Omarchy.batteryPresent || (Omarchy.powerPresent && Omarchy.isLaptop))", Shown::When(When::Any(&[When::On(Field::Key("batteryPresent")), When::All(&[When::On(Field::Key("powerPresent")), When::On(Field::Key("isLaptop"))])]))),
    ("network", "VPN and hotspot", "Omarchy.wifiHw", Shown::When(When::On(Field::Key("wifiHw")))),
    ("idle", "Advanced", "Omarchy.isLaptop", Shown::When(When::On(Field::Key("isLaptop")))),
    ("applications/startup", "Failures", "root.failed.length", Shown::Page("failures")),
    ("sound", "Tuning", "Omarchy.audioTuningMatch", Shown::When(When::On(Field::Key("audioTuningMatch")))),
    ("system/diagnostics", "Disk", "root.diag.disk.total > 0", Shown::Page("disk")),
    ("system/diagnostics", "Memory", "root.diag.memory.total > 0", Shown::Page("memory")),
    ("system/kernel", "Direct EFI boot", "Omarchy.directBootAvailable", Shown::When(When::On(Field::Key("directBootAvailable")))),
];

/// What decides whether a group whose rows are all hidden is drawn, if anything does.
pub fn shown(page: &str, group: &str) -> Option<Shown> {
    SHOWN
        .iter()
        .find(|(p, g, _, _)| *p == page && *g == group)
        .map(|(_, _, _, s)| *s)
}

/// A row, with the words `SettingRow` has, its control, and when it is shown (`available`) and
/// its control can be used (`enabled`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub label: &'static str,
    pub description: &'static str,
    pub hint: &'static str,
    pub keywords: &'static [&'static str],
    pub control: Control,
    pub available: When,
    pub enabled: When,
    /// A description the QML picks by a condition (`Omarchy.isLaptop ? "..." : "..."`), which
    /// search reads as none.
    pub varies: Option<Varies>,
    /// Words the QML builds from what the window holds, beyond one condition's two texts.
    pub words: Option<words::Words>,
}

/// One text while a condition holds, the other while it does not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Varies {
    pub when: When,
    pub then: &'static str,
    pub otherwise: &'static str,
}

/// A row with no control yet.
pub const fn line(
    label: &'static str,
    description: &'static str,
    hint: &'static str,
    keywords: &'static [&'static str],
) -> Line {
    Line {
        label,
        description,
        hint,
        keywords,
        control: Control::Pending,
        available: When::Always,
        enabled: When::Always,
        varies: None,
        words: None,
    }
}

impl Line {
    /// The row with `control`.
    pub const fn with(self, control: Control) -> Line {
        Line { control, ..self }
    }

    /// The row, shown only while `when` holds (`available:`).
    pub const fn shown(self, when: When) -> Line {
        Line {
            available: when,
            ..self
        }
    }

    /// The row, described by `then` while `when` holds and by `otherwise` while it does not.
    pub const fn varies(self, when: When, then: &'static str, otherwise: &'static str) -> Line {
        Line {
            varies: Some(Varies {
                when,
                then,
                otherwise,
            }),
            ..self
        }
    }

    /// The row, described by `words` (`catalog/words.rs`).
    pub const fn words(self, words: words::Words) -> Line {
        Line {
            words: Some(words),
            ..self
        }
    }

    /// The row, its control usable only while `when` holds (`enabled:`).
    pub const fn enabled(self, when: When) -> Line {
        Line {
            enabled: when,
            ..self
        }
    }

    pub fn mapped(&self) -> bool {
        self.control != Control::Pending
    }
}

impl Page {
    pub fn title(&self) -> &'static str {
        hubs::title(self.id)
    }

    /// What the page says of itself: its own words, or the hub's.
    pub fn description(&self) -> &'static str {
        if !self.description.is_empty() {
            return self.description;
        }
        hubs::find(self.id).map_or("", |hub| hub.description)
    }

    pub fn lines(&self) -> impl Iterator<Item = &'static Line> + use<> {
        self.groups.iter().flat_map(|g| g.rows.iter())
    }

    /// The snapshot keys its rows read, each once, in order.
    pub fn held(&self) -> Vec<Held> {
        let mut out: Vec<Held> = Vec::new();
        let lists = self
            .groups
            .iter()
            .filter_map(|g| list(self.id, g.title))
            .flat_map(List::fields);
        let groups = self
            .groups
            .iter()
            .filter_map(|g| match shown(self.id, g.title) {
                Some(Shown::When(w)) => Some(w.fields()),
                _ => None,
            });
        let lists = lists.chain(groups.flatten());
        let fields = self.lines().flat_map(|l| {
            let gates = l.available.fields().into_iter().chain(l.enabled.fields());
            let gates = gates.chain(l.varies.map(|v| v.when.fields()).unwrap_or_default());
            let said = l
                .words
                .map_or(&[][..], |w| w.reads())
                .iter()
                .map(|k| Field::Key(k));
            let gates = gates.chain(said);
            l.control.fields().into_iter().chain(gates)
        });
        let custom = self.lines().flat_map(|l| match l.control {
            Control::Custom(c) => c.held(),
            _ => Vec::new(),
        });
        for held in fields.chain(lists).map(|f| *f.held()).chain(custom) {
            if !out.contains(&held) {
                out.push(held);
            }
        }
        out
    }

    /// The inventories its rows read, each once.
    pub fn inventories(&self) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::new();
        let lists = self.groups.iter().filter_map(|g| list(self.id, g.title));
        for kind in lists.flat_map(List::inventories) {
            if !out.contains(kind) {
                out.push(kind);
            }
        }
        for line in self.lines() {
            let custom = match line.control {
                Control::Custom(c) => c.inventories(),
                _ => &[],
            };
            let said = line.words.and_then(|w| w.inventory());
            for kind in custom.iter().copied().chain(said) {
                if !out.contains(&kind) {
                    out.push(kind);
                }
            }
        }
        out
    }

    /// Whether the page reads the live samples (`feed.rs`), so the window holds them for it.
    pub fn live(&self) -> bool {
        self.groups
            .iter()
            .filter_map(|g| list(self.id, g.title))
            .any(List::live)
            || self
                .lines()
                .any(|l| matches!(l.control, Control::Custom(c) if c.live()))
    }

    /// Whether the page is whole: drawn, and blocked on nothing (`BLOCKED`).
    pub fn mapped(&self) -> bool {
        self.drawn() && blocked(self.id).is_none()
    }

    /// Whether every row has its control, and every list the QML builds from a model is drawn.
    /// A group with no row of its own lists what the QML builds at run time, so a page with one
    /// is not drawn unless it draws that list, and neither is a page with no group the catalog
    /// could read.
    pub fn drawn(&self) -> bool {
        // A model is drawn by its group's list, or by the custom row whose control it is.
        let drawn = |title: &str| {
            self.groups
                .iter()
                .filter(|g| g.title == title)
                .flat_map(|g| g.rows.iter())
                .any(|l| matches!(l.control, Control::Custom(c) if c.draws_model()))
        };
        let listed = MODEL_LISTS
            .iter()
            .filter(|(page, _)| *page == self.id)
            .all(|(_, group)| list(self.id, group).is_some() || drawn(group));
        !self.groups.is_empty()
            && listed
            && self
                .groups
                .iter()
                .all(|g| !g.rows.is_empty() || list(self.id, g.title).is_some())
            && self.lines().all(Line::mapped)
    }
}

/// The pages drawn whole but for a part ODK or the backend cannot give yet, with what is missing:
/// they are not mapped, and say why.
pub const BLOCKED: &[(&str, &str)] = &[];

/// What a page is blocked on, if anything.
pub fn blocked(page: &str) -> Option<&'static str> {
    BLOCKED
        .iter()
        .find(|(p, _)| *p == page)
        .map(|(_, why)| *why)
}

/// Every page, in the order of the hubs and of each hub's pages.
pub fn pages() -> impl Iterator<Item = &'static Page> {
    [
        home::PAGES,
        look::PAGES,
        input::PAGES,
        device::PAGES,
        apps::PAGES,
        general::PAGES,
        admin::PAGES,
    ]
    .into_iter()
    .flatten()
}

pub fn page(id: &str) -> Option<&'static Page> {
    pages().find(|p| p.id == id)
}

/// The pages' rows as search finds them: each group's heading, then its rows.
pub fn rows() -> Vec<Row> {
    let mut out = Vec::new();
    for page in pages() {
        for group in page.groups {
            if !group.title.is_empty() {
                out.push(Row::group(page.id, group.title, group.detail));
            }
            out.extend(
                group
                    .rows
                    .iter()
                    .map(|l| Row::setting(page.id, l.label, l.description, l.hint, l.keywords)),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_is_a_hub_or_a_hub_page_and_is_listed_once() {
        let ids: Vec<_> = pages().map(|p| p.id).collect();
        for id in &ids {
            assert!(hubs::known(id), "{id} is not a hub or a hub's page");
            assert_eq!(ids.iter().filter(|i| *i == id).count(), 1, "{id}");
        }
    }

    #[test]
    fn a_page_built_at_run_time_says_what_its_hub_says() {
        let power = page("power").unwrap();
        assert!(power.description.is_empty());
        assert_eq!(
            power.description(),
            hubs::find("power").unwrap().description
        );
        assert_eq!(page("monitor/cpu").unwrap().title(), "CPU");
    }
}
