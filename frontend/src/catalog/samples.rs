//! What the custom rows call that no `Setter` describes, with what each call sends, for
//! `tests/catalog_setters.rs` to run against `Omarchy.qml` (`Custom::samples`).

use serde_json::{Value, json};

use super::custom::Custom;
use super::sample_pages as pages;
use crate::{
    appearance, capture, environment, notifications, rules, software, startup, workspaces,
};

impl Custom {
    /// Calls it makes that no `Setter` describes, with what it sends for each, for
    /// `tests/catalog_setters.rs` to run against `Omarchy.qml`: the setter, the snapshot it runs
    /// on, its arguments, and the request (none when it sends nothing; a list when several).
    pub fn samples(self) -> Vec<(&'static str, Value, Value, Option<Value>)> {
        match self {
            // The Lock slider calls `setIdle(Omarchy.idleScreensaver, next)` when `next` is not
            // the lock already.
            Custom::IdleLock => [
                (300, 600, 900.0),
                (300, 600, 600.0),
                (0, 0, 60.0),
                (120, 3600, 0.0),
            ]
            .into_iter()
            .map(|(saver, lock, next)| {
                let doc = json!({"idleScreensaver": saver, "idleLock": lock});
                let body = crate::system::idle_lock_write(
                    doc.get("idleScreensaver"),
                    doc.get("idleLock"),
                    next,
                );
                (
                    "setIdle",
                    doc,
                    json!([saver, crate::js::json_number(next)]),
                    body,
                )
            })
            // The row skips a value the lock already is, and calls nothing.
            .filter(|s| s.3.is_some())
            .collect(),
            Custom::NewReminder => [
                ("5", "Tea"),
                (" 12 ", "  Stand up  "),
                ("0", "x"),
                ("05", "x"),
                ("1.5", "x"),
                ("", ""),
                ("1440", ""),
            ]
            .into_iter()
            .map(|(m, t)| {
                (
                    "setReminder",
                    json!({}),
                    json!([m, t]),
                    notifications::reminder_write(m, t),
                )
            })
            .collect(),
            // `setEnvVars(vars, prepend)`: the variables, when a list, then the prepend.
            Custom::AddVar => {
                let vars = json!([{"key": "EDITOR", "value": "nvim"}]);
                [
                    (vars.clone(), json!("/opt/bin")),
                    (json!(null), json!("")),
                    (vars, json!(null)),
                ]
                .into_iter()
                .map(|(v, p)| {
                    let bodies = environment::set_env(&v, &p);
                    (
                        "setEnvVars",
                        json!({}),
                        json!([v, p]),
                        Some(Value::Array(bodies)),
                    )
                })
                .collect()
            }
            // The startup setters, on a list with a line of the user's and two of Atmos's.
            Custom::AddAutostart => {
                let list = json!([
                    {"command": "waybar", "managed": false},
                    {"command": "hyprsunset", "delay": 5, "enabled": false, "managed": true},
                    {"command": "mako", "managed": true},
                    null,
                ]);
                let doc = json!({ "autostart": list });
                let a = Some(&list);
                let mut out = Vec::new();
                for (cmd, delay) in [
                    (" foo ", json!(3)),
                    ("mako", json!(0)),
                    ("", json!(1)),
                    ("bar", json!(900)),
                    ("baz", json!(-4)),
                    ("q", json!("x")),
                ] {
                    out.push((
                        "addAutostart",
                        doc.clone(),
                        json!([cmd, delay]),
                        startup::add(a, cmd, &delay),
                    ));
                }
                for (cmd, on) in [("hyprsunset", true), ("mako", false), ("waybar", true)] {
                    out.push((
                        "setAutostartEnabled",
                        doc.clone(),
                        json!([cmd, on]),
                        startup::set_enabled(a, cmd, on),
                    ));
                }
                for cmd in ["mako", "waybar", "nope"] {
                    out.push((
                        "removeAutostart",
                        doc.clone(),
                        json!([cmd]),
                        startup::remove(a, cmd),
                    ));
                }
                out
            }
            // The rule setters, on a list with a rule of the user's and two of Atmos's.
            Custom::AddRule => {
                let list = json!([
                    {"match": "steam", "placement": "float", "managed": false},
                    {"match": "mpv", "placement": "float", "center": true, "width": 1280.4, "height": 720, "managed": true},
                    {"match": "obs", "workspace": "4", "pin": true, "opacity": 0.9, "managed": true},
                    {"match": "", "managed": true},
                    null,
                ]);
                let doc = json!({ "windowRules": list });
                let a = Some(&list);
                let mut out = Vec::new();
                for (m, placement, center, w, h, ws) in [
                    (" foot ", "float", true, json!(0), json!(0), " 2 "),
                    ("mpv", "tile", false, json!(800), json!(600), ""),
                    ("kitty", "sideways", false, json!(99), json!(600), ""),
                    ("kitty", "", false, json!(0), json!(0), ""),
                    ("", "float", true, json!(0), json!(0), ""),
                    ("a]]b", "float", true, json!(0), json!(0), ""),
                    ("zed", "", true, json!("x"), json!(300.6), "special:term"),
                ] {
                    let args = json!([m, placement, center, w, h, ws]);
                    let body = rules::add(a, m, placement, center, &w, &h, ws);
                    out.push(("addWindowRule", doc.clone(), args, body));
                }
                for m in ["obs", "steam", "nope"] {
                    out.push((
                        "removeWindowRule",
                        doc.clone(),
                        json!([m]),
                        rules::remove(a, m),
                    ));
                }
                out
            }
            // `setCount(n)` calls this with the count it comes to.
            Custom::BarCount => [1.0, 5.0, 10.0]
                .into_iter()
                .map(|n| {
                    let body = workspaces::count_write(None, n);
                    (
                        "dispatchSetting",
                        json!({}),
                        json!(["workspaceBarCount", n]),
                        body,
                    )
                })
                .collect(),
            Custom::WorkspaceSwitch(key) => [true, false]
                .into_iter()
                .map(|on| {
                    let body = Some(workspaces::switch_write(key, on));
                    ("dispatchSetting", json!({}), json!([key, on]), body)
                })
                .collect(),
            Custom::TakeShot => [
                ("smart", "slurp"),
                ("fullscreen", "save"),
                ("windows", "copy"),
                ("screen", "slurp"),
                ("region", "print"),
            ]
            .into_iter()
            .map(|(mode, dest)| {
                let body = capture::screenshot(mode, dest).map(|args| {
                    json!({"op": "job", "name": "capture.screenshot", "args": args,
                        "kind": "screenshot", "sudo": false, "interactive": true})
                });
                ("captureScreenshot", json!({}), json!([mode, dest]), body)
            })
            .collect(),
            // Start, with what the switches hold, and Stop once the recorder runs.
            Custom::RecSwitch("recFullscreen") => {
                let mut out: Vec<_> = [
                    (true, false, false, "medium", false),
                    (false, true, true, "large", true),
                    (true, true, true, "huge", false),
                ]
                .into_iter()
                .map(|(d, m, w, size, f)| {
                    let args = capture::recording(d, m, w, size, f);
                    let body = json!({"op": "job", "name": "capture.recording", "args": args,
                        "kind": "recording", "sudo": false, "interactive": true});
                    (
                        "startScreenrecording",
                        json!({}),
                        json!([d, m, w, size, f]),
                        Some(body),
                    )
                })
                .collect();
                let on = json!({ "recordingActive": true });
                out.push(("stopScreenrecording", on, json!([]), Some(capture::stop())));
                out
            }
            Custom::ResizeWebcam => ["smaller", "larger", "reset", "large", "huge", ""]
                .into_iter()
                .map(|a| ("resizeWebcam", json!({}), json!([a]), capture::resize(a)))
                .collect(),
            Custom::SaveFolder(_) => {
                let dirs = ["/home/u/Pictures", "", "rel/dir", "/a/../b", "/a\nb"];
                let mut out: Vec<_> = dirs
                    .into_iter()
                    .map(|d| ("openUserDir", json!({}), json!([d]), capture::open_dir(d)))
                    .collect();
                for d in ["/home/u/Videos", ""] {
                    out.push((
                        "copyText",
                        json!({}),
                        json!([d]),
                        crate::actions::copy_write(d),
                    ));
                }
                out
            }
            // Install… and Remove… of a stack, and of a toolchain.
            Custom::DevEnv => {
                let job = |j: Option<crate::jobs::Job>| j.map(|j| software::job_body(&j));
                let mut out = Vec::new();
                let some = software::ITEMS
                    .iter()
                    .take(3)
                    .chain(software::ITEMS.iter().rev().take(2));
                for item in some {
                    for (argv, remove) in [(item.install, false), (item.remove, true)] {
                        if let Some(argv) = argv {
                            let kind = if remove {
                                "software-remove"
                            } else {
                                "software-install"
                            };
                            let body = job(software::software_job(argv, remove));
                            out.push(("runSoftware", json!({}), json!([argv, kind]), body));
                        }
                    }
                }
                for lang in ["node", "rust", "Node", "", "dot net"] {
                    let (install, remove) = (
                        software::dev_env_job(lang, false),
                        software::dev_env_job(lang, true),
                    );
                    out.push(("installDevEnv", json!({}), json!([lang]), job(install)));
                    out.push(("removeDevEnv", json!({}), json!([lang]), job(remove)));
                }
                out
            }
            Custom::DockerDb => ["PostgreSQL", "MSSQL", "", "Maria DB", "redis2"]
                .into_iter()
                .map(|n| {
                    let body = software::docker_db_job(n).map(|j| software::job_body(&j));
                    ("installDockerDb", json!({}), json!([n]), body)
                })
                .collect(),
            // `addAutostart(command)`, with no delay.
            Custom::AppAutostart => {
                let list = json!([{"command": "waybar", "managed": false}, {"command": "mako", "managed": true}]);
                let doc = json!({ "autostart": list });
                ["swaync", " mako ", ""]
                    .into_iter()
                    .map(|c| {
                        (
                            "addAutostart",
                            doc.clone(),
                            json!([c]),
                            startup::add(Some(&list), c, &Value::Null),
                        )
                    })
                    .collect()
            }
            // What Add's dialog and Remove… call, for each kind.
            Custom::AddLauncher(kind) => {
                use crate::launchers::{self, Draft};
                let mut out = Vec::new();
                for (name, command, url, style, icon) in [
                    (
                        " Notes ",
                        " nvim ~/notes ",
                        "notes.example.com",
                        "float",
                        " ",
                    ),
                    ("Mail", "aerc", "http://mail.example.com", "tile", "mail"),
                ] {
                    let d = Draft {
                        kind: kind.into(),
                        name: name.into(),
                        command: command.into(),
                        url: url.into(),
                        style: style.into(),
                        icon: icon.into(),
                    };
                    let Ok(job) = launchers::submit(&d) else {
                        continue;
                    };
                    let a = &job.args;
                    let (setter, args) = match kind {
                        "web" => ("installWebApp", json!([a["name"], a["url"], icon])),
                        "tui" => (
                            "installTui",
                            json!([a["name"], a["command"], a["style"], a["icon"]]),
                        ),
                        _ => (
                            "installDesktopApp",
                            json!([a["name"], a["command"], a["icon"]]),
                        ),
                    };
                    out.push((setter, json!({}), args, Some(launchers::job_body(&job))));
                }
                for (id, name) in [("notes", "Notes"), ("", "x"), ("mail", "")] {
                    let body = launchers::remove(kind, id, name).map(|(b, _)| b);
                    let (setter, args) = match kind {
                        "web" => ("removeWebApp", json!([id])),
                        "tui" => ("removeTui", json!([id])),
                        _ => ("removeDesktopApp", json!([id, name])),
                    };
                    out.push((setter, json!({}), args, body));
                }
                out
            }
            // Snapper, TRIM, rollback, hibernation and the passphrase, as the page sends them.
            Custom::Disk("hibernation") => pages::disks(),
            Custom::Display("layout") => super::sample_forms::display(),
            Custom::Kb("layouts") => super::sample_forms::keyboard(),
            Custom::Bind("add") => super::sample_forms::bindings(),
            Custom::Net("hotspot") => super::sample_forms::network(),
            Custom::Acct("adduser") => super::sample_forms::accounts(),
            Custom::Wifi("qr") => super::sample_forms::wifi(),
            Custom::Bt("nopaired") => super::sample_forms::bluetooth(),
            Custom::Wifi("nearby") => super::sample_forms::radios(),
            // What the Hooks page's buttons and dialog call.
            Custom::HookAdd => pages::hooks(),
            // What the System page's own rows call.
            Custom::Sys("hostname") => pages::system(),
            // What the Bar page's own rows call.
            Custom::Bar("hidden") => pages::bar(),
            // Copy and Ask send the report the inventory makes.
            Custom::Diag("copy") => pages::diagnostics(),
            // Each time and the switch as its own setting (see `appearance.rs`).
            Custom::NightSchedule | Custom::UseSchedule => [
                ("nightlightDay", json!("06:45")),
                ("nightlightNight", json!("21:30")),
                ("nightlightNightOn", json!(true)),
                ("nightlightNightOn", json!(false)),
            ]
            .into_iter()
            .map(|(key, v)| {
                let body = Some(appearance::schedule_write(key, v.clone()));
                ("dispatchSetting", json!({}), json!([key, v]), body)
            })
            .collect(),
            // `writeWorkspaces(items, wrap, wheel)`, on a list with a line of the user's.
            Custom::AddScratch => {
                let list = json!([
                    {"id": "3", "name": "web", "managed": false},
                    {"id": "1", "name": "", "monitor": "DP-1", "managed": true},
                    {"id": "special:notes", "special": true, "managed": true},
                ]);
                let doc = json!({ "workspaces": list });
                let items = workspaces::add_special(Some(&list), "music").unwrap();
                let body = workspaces::write(Some(&list), items.clone());
                vec![(
                    "writeWorkspaces",
                    doc,
                    json!([items, true, false]),
                    Some(body),
                )]
            }
            _ => Vec::new(),
        }
    }
}

/// A diagnostics inventory with something in every part.
pub static DIAGNOSTICS: std::sync::LazyLock<Value> = std::sync::LazyLock::new(|| {
    json!({
        "generatedAt": "2026-10-28T10:00:00Z", "hostname": "box",
        "kernel": {"sysname": "Linux", "release": "6.12.1-arch1", "machine": "x86_64"},
        "omarchy": {"version": "3.1.0", "channel": "stable", "path": "/home/u/.local/share/omarchy"},
        "atmos": {"revision": "abc1234", "installed": true, "hyprAtmos": true,
            "sentinels": {"look": true, "input": false, "bindings": true}},
        "hyprland": {"version": "0.52.0", "configErrors": ["bad bind on line 3", "", "token=secret"]},
        "failedUnits": [{"unit": "foo.service", "scope": "user", "sub": "failed", "description": "Foo"},
            {"unit": "bad unit"}, {"unit": "bar.service"}],
        "disk": {"total": 1000, "used": 950, "available": 50, "percent": 95},
        "memory": {"total": 100, "used": 96, "available": 4, "swapTotal": 10, "swapUsed": 2},
        "gpu": {"driver": "amdgpu", "identity": "Radeon 780M"},
        "portals": {"xdg-desktop-portal": "active", "xdg-desktop-portal-hyprland": "failed", "xdg-desktop-portal-gtk": "x"},
        "pipewire": {"pipewire": "active", "pipewire-pulse": "active", "wireplumber": "active"},
        "network": {"online": true, "kind": "wifi"},
        "pacman": {"syncOk": true, "lastSync": "2026-10-27", "dbCount": 4},
        "recentErrors": ["kernel: oops", "sshd: password for root"],
    })
});
