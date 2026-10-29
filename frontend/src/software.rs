//! The Software page's logic (`pages/SoftwarePage.qml`, `services/Software.js`): the optional
//! stacks Omarchy installs, whether each is installed, what Install… and Remove… ask, and the
//! jobs they start, as `Omarchy.runSoftware`, `installDevEnv`, `removeDevEnv` and
//! `installDockerDb` make them. `tests/catalog_software.rs` holds the table to `catalog()` and
//! `tests/catalog_setters.rs` runs the setters, under Node.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::jobs::Job;

/// One stack (`catalog()`'s item).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub id: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub hint: &'static str,
    pub install: Option<&'static [&'static str]>,
    pub remove: Option<&'static [&'static str]>,
    /// The snapshot bag and key that say it is installed (`presentIn`).
    pub present: (&'static str, &'static str),
    /// Remove also deletes the libraries it keeps.
    pub wipe: bool,
}

#[rustfmt::skip]
pub const ITEMS: &[Item] = &[
    Item { id: "chrome", group: "browsers", label: "Chrome", hint: "omarchy install browser chrome", install: Some(&["omarchy", "install", "browser", "chrome"]), remove: Some(&["omarchy", "remove", "browser", "chrome"]), present: ("browsers", "chrome"), wipe: false },
    Item { id: "brave", group: "browsers", label: "Brave", hint: "omarchy install browser brave", install: Some(&["omarchy", "install", "browser", "brave"]), remove: Some(&["omarchy", "remove", "browser", "brave"]), present: ("browsers", "brave"), wipe: false },
    Item { id: "brave-origin", group: "browsers", label: "Brave Origin", hint: "omarchy install browser brave-origin", install: Some(&["omarchy", "install", "browser", "brave-origin"]), remove: Some(&["omarchy", "remove", "browser", "brave-origin"]), present: ("browsers", "brave-origin"), wipe: false },
    Item { id: "edge", group: "browsers", label: "Edge", hint: "omarchy install browser edge", install: Some(&["omarchy", "install", "browser", "edge"]), remove: Some(&["omarchy", "remove", "browser", "edge"]), present: ("browsers", "edge"), wipe: false },
    Item { id: "firefox", group: "browsers", label: "Firefox", hint: "omarchy install browser firefox", install: Some(&["omarchy", "install", "browser", "firefox"]), remove: Some(&["omarchy", "remove", "browser", "firefox"]), present: ("browsers", "firefox"), wipe: false },
    Item { id: "zen", group: "browsers", label: "Zen", hint: "omarchy install browser zen", install: Some(&["omarchy", "install", "browser", "zen"]), remove: Some(&["omarchy", "remove", "browser", "zen"]), present: ("browsers", "zen"), wipe: false },
    Item { id: "alacritty", group: "terminals", label: "Alacritty", hint: "omarchy install terminal alacritty", install: Some(&["omarchy", "install", "terminal", "alacritty"]), remove: None, present: ("terminals", "alacritty"), wipe: false },
    Item { id: "foot", group: "terminals", label: "foot", hint: "omarchy install terminal foot", install: Some(&["omarchy", "install", "terminal", "foot"]), remove: None, present: ("terminals", "foot"), wipe: false },
    Item { id: "ghostty", group: "terminals", label: "Ghostty", hint: "omarchy install terminal ghostty", install: Some(&["omarchy", "install", "terminal", "ghostty"]), remove: None, present: ("terminals", "ghostty"), wipe: false },
    Item { id: "kitty", group: "terminals", label: "Kitty", hint: "omarchy install terminal kitty", install: Some(&["omarchy", "install", "terminal", "kitty"]), remove: None, present: ("terminals", "kitty"), wipe: false },
    Item { id: "emacs", group: "editors", label: "Emacs", hint: "omarchy install editor emacs", install: Some(&["omarchy", "install", "editor", "emacs"]), remove: None, present: ("editors", "emacs"), wipe: false },
    Item { id: "helix", group: "editors", label: "Helix", hint: "omarchy install editor helix", install: Some(&["omarchy", "install", "editor", "helix"]), remove: None, present: ("editors", "helix"), wipe: false },
    Item { id: "vscode", group: "editors", label: "VS Code", hint: "omarchy install editor vscode", install: Some(&["omarchy", "install", "editor", "vscode"]), remove: None, present: ("editors", "code"), wipe: false },
    Item { id: "zed", group: "editors", label: "Zed", hint: "omarchy install editor zed", install: Some(&["omarchy", "install", "editor", "zed"]), remove: None, present: ("editors", "zeditor"), wipe: false },
    Item { id: "1password", group: "services", label: "1Password", hint: "omarchy install service 1password", install: Some(&["omarchy", "install", "service", "1password"]), remove: Some(&["omarchy", "remove", "service", "1password"]), present: ("services", "onepassword"), wipe: false },
    Item { id: "dropbox", group: "services", label: "Dropbox", hint: "omarchy install service dropbox", install: Some(&["omarchy", "install", "service", "dropbox"]), remove: Some(&["omarchy", "remove", "service", "dropbox"]), present: ("services", "dropbox"), wipe: false },
    Item { id: "nordvpn", group: "services", label: "NordVPN", hint: "omarchy install service nordvpn", install: Some(&["omarchy", "install", "service", "nordvpn"]), remove: None, present: ("services", "nordvpn"), wipe: false },
    Item { id: "signal", group: "services", label: "Signal", hint: "omarchy install service signal", install: Some(&["omarchy", "install", "service", "signal"]), remove: None, present: ("services", "signal"), wipe: false },
    Item { id: "spotify", group: "services", label: "Spotify", hint: "omarchy install service spotify", install: Some(&["omarchy", "install", "service", "spotify"]), remove: None, present: ("services", "spotify"), wipe: false },
    Item { id: "sunshine", group: "services", label: "Sunshine", hint: "omarchy install service sunshine", install: Some(&["omarchy", "install", "service", "sunshine"]), remove: Some(&["omarchy", "remove", "service", "sunshine"]), present: ("services", "sunshine"), wipe: false },
    Item { id: "tailscale", group: "services", label: "Tailscale", hint: "omarchy install service tailscale", install: Some(&["omarchy", "install", "service", "tailscale"]), remove: Some(&["omarchy", "remove", "service", "tailscale"]), present: ("services", "tailscale"), wipe: false },
    Item { id: "steam", group: "gaming", label: "Steam", hint: "omarchy install gaming steam", install: Some(&["omarchy", "install", "gaming", "steam"]), remove: Some(&["omarchy", "remove", "gaming", "steam"]), present: ("gaming", "steam"), wipe: true },
    Item { id: "heroic", group: "gaming", label: "Heroic", hint: "omarchy install gaming heroic", install: Some(&["omarchy", "install", "gaming", "heroic"]), remove: Some(&["omarchy", "remove", "gaming", "heroic"]), present: ("gaming", "heroic"), wipe: true },
    Item { id: "lutris", group: "gaming", label: "Lutris", hint: "omarchy install gaming lutris", install: Some(&["omarchy", "install", "gaming", "lutris"]), remove: Some(&["omarchy", "remove", "gaming", "lutris"]), present: ("gaming", "lutris"), wipe: true },
    Item { id: "retroarch", group: "gaming", label: "RetroArch", hint: "omarchy install gaming retroarch", install: Some(&["omarchy", "install", "gaming", "retroarch"]), remove: Some(&["omarchy", "remove", "gaming", "retroarch"]), present: ("gaming", "retroarch"), wipe: true },
    Item { id: "battlenet", group: "gaming", label: "Battle.net", hint: "omarchy install gaming battlenet", install: Some(&["omarchy", "install", "gaming", "battlenet"]), remove: Some(&["omarchy", "remove", "gaming", "battlenet"]), present: ("gaming", "battlenet"), wipe: true },
    Item { id: "geforce-now", group: "gaming", label: "GeForce Now", hint: "omarchy install gaming geforce-now", install: Some(&["omarchy", "install", "gaming", "geforce-now"]), remove: Some(&["omarchy", "remove", "gaming", "geforce-now"]), present: ("gaming", "geforceNow"), wipe: true },
    Item { id: "xbox-cloud", group: "gaming", label: "Xbox Cloud", hint: "omarchy install gaming xbox-cloud", install: Some(&["omarchy", "install", "gaming", "xbox-cloud"]), remove: Some(&["omarchy", "remove", "gaming", "xbox-cloud"]), present: ("gaming", "xboxCloud"), wipe: false },
    Item { id: "xbox-controllers", group: "gaming", label: "Xbox controllers", hint: "omarchy install gaming xbox-controllers", install: Some(&["omarchy", "install", "gaming", "xbox-controllers"]), remove: Some(&["omarchy", "remove", "gaming", "xbox-controllers"]), present: ("gaming", "xboxControllers"), wipe: false },
    Item { id: "chatgpt", group: "development", label: "ChatGPT app", hint: "omarchy install ai chatgpt", install: Some(&["omarchy", "install", "ai", "chatgpt"]), remove: None, present: ("extras", "chatgpt"), wipe: false },
];

/// The bags whether a stack is installed is read from.
pub const BAGS: &[&str] = &[
    "browsers",
    "terminals",
    "editors",
    "services",
    "gaming",
    "extras",
];

pub const DEV_ENVS: &[(&str, &str)] = &[
    ("ruby", "Ruby"),
    ("node", "Node"),
    ("bun", "Bun"),
    ("deno", "Deno"),
    ("go", "Go"),
    ("php", "PHP"),
    ("laravel", "Laravel"),
    ("symfony", "Symfony"),
    ("python", "Python"),
    ("elixir", "Elixir"),
    ("phoenix", "Phoenix"),
    ("rust", "Rust"),
    ("java", "Java"),
    ("zig", "Zig"),
    ("ocaml", "OCaml"),
    ("dotnet", ".NET"),
    ("clojure", "Clojure"),
    ("scala", "Scala"),
];

pub const DOCKER_DBS: &[(&str, &str)] = &[
    ("MySQL", "MySQL"),
    ("PostgreSQL", "PostgreSQL"),
    ("Redis", "Redis"),
    ("MongoDB", "MongoDB"),
    ("MariaDB", "MariaDB"),
    ("MSSQL", "MSSQL"),
];

/// Where the toolchain and database start (`devLang`, `dockerDb`).
pub const DEV_LANG: &str = "node";
pub const DOCKER_DB: &str = "PostgreSQL";

/// `groupItems(group)`.
pub fn group_items(group: &str) -> impl Iterator<Item = &'static Item> + use<'_> {
    ITEMS.iter().filter(move |i| i.group == group)
}

/// `presentIn(item, bags)`: whether its bag says it is installed.
pub fn installed(item: &Item, bag: Option<&Value>) -> bool {
    bag.and_then(|b| b.get(item.present.1)) == Some(&Value::Bool(true))
}

/// A stack's description.
pub fn description(item: &Item, installed: bool) -> &'static str {
    if installed {
        "Installed"
    } else if item.wipe {
        "Not installed. Remove later also deletes its libraries."
    } else {
        "Not installed"
    }
}

/// The confirm's title and message, for Install or Remove of `label`.
pub fn confirm(remove: bool, label: &str, wipe: bool) -> (String, String) {
    if !remove {
        return (
            format!("Install {label}"),
            format!("Install {label} with the Omarchy installer."),
        );
    }
    let message = if wipe {
        format!("Remove {label}? That also deletes the libraries and prefixes this launcher keeps.")
    } else {
        format!("Remove {label} and the packages Omarchy added for it.")
    };
    (format!("Remove {label}"), message)
}

/// `runSoftware(argv, kind)`: Omarchy's installer with the words after `omarchy`.
pub fn software_job(argv: &[&str], remove: bool) -> Option<Job> {
    if argv.len() < 2 || argv[0] != "omarchy" {
        return None;
    }
    let args = json!({ "action": argv[1], "words": &argv[2..] });
    let kind = if remove {
        "software-remove"
    } else {
        "software-install"
    };
    let what = if remove { "The removal" } else { "The install" };
    Some(Job::new("software.run", args, kind, what).sudo())
}

/// `installDevEnv(lang)` or `removeDevEnv(lang)`.
pub fn dev_env_job(lang: &str, remove: bool) -> Option<Job> {
    if lang.is_empty() || !lang.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    let (name, kind) = if remove {
        ("dev-env.remove", "dev-env-remove")
    } else {
        ("dev-env.install", "dev-env-install")
    };
    Some(Job::new(name, json!({ "lang": lang }), kind, "The toolchain").sudo())
}

/// `installDockerDb(name)`.
pub fn docker_db_job(name: &str) -> Option<Job> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let args = json!({ "name": name });
    Some(
        Job::new(
            "docker-db.install",
            args,
            "docker-db-install",
            "The database",
        )
        .sudo(),
    )
}

/// Whether the Docker database row shows: Docker is installed.
pub fn has_docker(extras: Option<&Value>) -> bool {
    extras.and_then(|e| e.get("docker")) == Some(&Value::Bool(true))
}

/// The bag a stack is read from.
pub fn bag(item: &Item) -> Field {
    Field::Key(item.present.0)
}

/// A job as the setters' tests record it.
pub fn job_body(job: &Job) -> Value {
    json!({
        "op": "job", "name": job.name, "args": job.args, "kind": job.kind,
        "sudo": job.sudo, "interactive": job.interactive,
    })
}

/// `String(value)` of a select's choice, for the drafts the page keeps.
pub fn choice(v: Option<&String>, or: &str) -> String {
    v.cloned().unwrap_or_else(|| or.to_owned())
}
