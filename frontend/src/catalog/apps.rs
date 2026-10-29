//! The Apps pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{Custom, Go};
use super::custom::Custom::{
    AddAutostart, AddLauncher, AppAutostart, AppNoAutostart, DevEnv, DockerDb, HookAdd, InList,
    McpInstall, McpNone, NoFailures, NoLaunchers,
};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "agentic",
        file: "AgenticPage.qml",
        description: "The coding agents on this machine, and which one Omarchy opens.",
        groups: &[
            Group { title: "Agents", detail: "", rows: &[
            ] },
            Group { title: "Tools", detail: "MCP lets an installed agent read this machine and change settings through Atmos.", rows: &[
                line("MCP", "Install the Atmos command into Grok, Claude Code, and Cursor.", "atmos mcp", &["mcp", "claude", "grok", "cursor"]).with(Go("Open…", "agentic/mcp")),
            ] },
        ],
    },
    Page {
        id: "agentic/mcp",
        file: "agentic/McpPage.qml",
        description: "Let installed coding agents read this machine and change settings through Atmos.",
        groups: &[
            Group { title: "Installed agents", detail: "", rows: &[
                line("Install", "Write the Atmos command into each installed agent that Atmos knows how to edit.", "", &["mcp", "install", "check"]).with(Custom(McpInstall)),
                line("No agents", "Install a coding agent on the Agentic page, then come back.", "", &[]).with(Custom(McpNone)),
            ] },
        ],
    },
    Page {
        id: "applications",
        file: "ApplicationsPage.qml",
        description: "Launchers you added yourself under ~/.local/share/applications. Remove deletes that desktop file. Packages from the repos stay on the system.",
        groups: &[
            Group { title: "Startup", detail: "Enable, delay, and failures live on the Startup page. The list below is the Atmos block in autostart.lua.", rows: &[
                line("Startup programs", "", "~/.config/hypr/autostart.lua", &["autostart", "startup", "delay"]).with(Go("Configure…", "applications/startup")).words(super::words::Words::Startup),
            ] },
            Group { title: "Advanced", detail: "Autostart writes a managed block at the end of ~/.config/hypr/autostart.lua. Lines you typed yourself stay. Remove only deletes a line Atmos added.", rows: &[
                line("Add a command", "A program name or command Omarchy should launch on start. Same form as o.launch_on_start.", "~/.config/hypr/autostart.lua", &["autostart", "startup", "launch", "hypr"]).with(Custom(AppAutostart)),
                line("Launch on start", "No launch-on-start commands.", "", &["autostart", "empty"]).with(Custom(AppNoAutostart)),
            ] },
            Group { title: "Add", detail: "Desktop writes a .desktop file for a command. Terminal uses omarchy tui install. Web uses omarchy webapp install and can fetch the site icon.", rows: &[
                line("Desktop", "A launcher for a command on this machine.", "~/.local/share/applications", &["add", "install", "create", "desktop", "launcher"]).with(Custom(AddLauncher("desktop"))),
                line("Terminal", "A TUI that opens in your default terminal.", "omarchy tui install", &["add", "install", "create", "tui", "terminal"]).with(Custom(AddLauncher("tui"))),
                line("Web", "A site in its own window.", "omarchy webapp install", &["add", "install", "create", "webapp", "web"]).with(Custom(AddLauncher("web"))),
            ] },
            Group { title: "Desktop", detail: "Regular desktop launchers in ~/.local/share/applications. Web apps and terminal UIs have their own sections below. Remove deletes that .desktop file.", rows: &[
                line("Desktop launchers", "No desktop launchers.", "", &["empty", "desktop"]).with(Custom(NoLaunchers("desktop"))),
            ] },
            Group { title: "Terminal", detail: "Terminal UIs you installed with omarchy tui install. They open in your default terminal. Remove deletes the launcher.", rows: &[
                line("Terminal launchers", "No terminal launchers.", "", &["empty", "tui"]).with(Custom(NoLaunchers("tui"))),
            ] },
            Group { title: "Web", detail: "Site wrappers you installed with omarchy webapp install. Remove deletes the launcher. The site itself stays online.", rows: &[
                line("Web apps", "No web apps.", "", &["empty", "webapp"]).with(Custom(NoLaunchers("web"))),
            ] },
        ],
    },
    Page {
        id: "applications/startup",
        file: "applications/StartupPage.qml",
        description: "Programs Hyprland launches at login. Disable comments the line out. Delay prefixes sleep N &&. Remove only deletes a row Atmos added.",
        groups: &[
            Group { title: "Launch on start", detail: "Writes o.launch_on_start in ~/.config/hypr/autostart.lua. Lines you typed yourself stay.", rows: &[
                line("Add a command", "A program name or command. Delay waits that many seconds after login.", "o.launch_on_start", &["autostart", "startup", "delay"]).with(Custom(AddAutostart)),
            ] },
            Group { title: "Failures", detail: "User or system units that failed this boot. Startup commands that never launched often show up here.", rows: &[
                line("Startup failures", "No failed units reported.", "", &["failed", "empty"]).with(Custom(NoFailures)),
            ] },
        ],
    },
    Page {
        id: "software",
        file: "SoftwarePage.qml",
        description: "Optional stacks Omarchy can install. Remove asks first. Gaming remove also deletes libraries those launchers keep.",
        groups: &[
            Group { title: "Browsers", detail: "Install an extra browser, then set the default on Defaults if you want Omarchy to open it.", rows: &[
            ] },
            Group { title: "Terminals", detail: "Omarchy can install another terminal. There is no matching remove command, so these rows only install.", rows: &[
            ] },
            Group { title: "Editors", detail: "Optional editors with Omarchy theme wiring. Remove is not a separate Omarchy command here.", rows: &[
            ] },
            Group { title: "Services", detail: "Optional apps Omarchy packages as services. Tailscale also has a shortcut on Network.", rows: &[
            ] },
            Group { title: "Gaming", detail: "Launchers and a couple of cloud clients. Remove for Steam and the others wipes their libraries.", rows: &[
            ] },
            Group { title: "Development", detail: "Language toolchains through mise, a Docker database, and the ChatGPT desktop app.", rows: &[
                line("Language toolchain", "Install or remove a dev environment Omarchy knows.", "omarchy install dev env", &["dev", "mise", "ruby", "node", "python", "rust"]).with(Custom(DevEnv)),
                line("Docker database", "Start a supported database in Docker.", "omarchy install docker dbs", &["docker", "postgres", "mysql", "redis", "mongo"]).with(Custom(DockerDb)),
            ] },
        ],
    },
    Page {
        id: "hooks",
        file: "HooksPage.qml",
        description: "Scripts Omarchy runs after a theme or font change, at boot, after an update, before a pacman refresh, and on low battery. Each event is a folder under ~/.config/omarchy/hooks.",
        groups: &[
            Group { title: "", detail: "", rows: &[
                line("This event", "", "omarchy hook ", &["hook", ""]).with(Custom(InList)),
                line("Scripts", "No scripts.", "", &["hook", "empty"]).with(Custom(InList)),
            ] },
            Group { title: "Install", detail: "Add copies a file you already have, or writes a one-line script, into ~/.config/omarchy/hooks/<event>.d/. Enable a sample by dropping .sample from its name. Disable puts .sample back so the runner skips it.", rows: &[
                line("Add a hook", "Pick the event, then copy a file or write a one-line command.", "omarchy hook install", &["hook", "install", "script", "add"]).with(Custom(HookAdd)),
            ] },
        ],
    },
];
