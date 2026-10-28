//! The registry of typed writes.
//!
//! A write is a name and arguments. The entry for the name checks the
//! arguments, builds the command line, and says what the write touches: its
//! lane, a coalescing key, the snapshot keys to read back afterwards and, for
//! a write the machine finishes a moment after the command returns, what the
//! read-back must show before it is believed. The window sends the name and
//! the arguments and never a command line, and shows what is read back, never
//! a value it worked out itself.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Map, Value};

use super::args::*;
use crate::error::{Error, Kind, Result};
use crate::runner::Run;

type Snap = Map<String, Value>;

/// What the read-back must show: given the keys before the write and now.
pub type Check = Box<dyn Fn(&Snap, &Snap) -> bool + Send + Sync>;

pub struct Built {
    /// The program and its arguments, run as they are.
    pub argv: Vec<String>,
    pub lane: String,
    /// Snapshot keys read back once the command has run.
    pub readback: Vec<String>,
    /// Waited for before the read-back is returned; None when the command
    /// returns after the machine has changed.
    pub settle: Option<Check>,
    /// The check needs the keys as they were before the write.
    pub before: bool,
    /// How long the read-back is repeated waiting for `settle`.
    pub settle_for: Duration,
    /// What the command reads on stdin.
    pub stdin: Option<String>,
    /// Starts something that stays open, as a unit of its own, and does not wait.
    pub launch: bool,
    /// Display kinds it changes (see `table::Entry::affects`).
    pub affects: Vec<String>,
}

impl Built {
    /// Writes on a lane run in order and are never collapsed: two toggles are
    /// two toggles. (`key` names the control, for the log.)
    pub(super) fn new(lane: &str, _key: &str, argv: Vec<String>, readback: &[&str]) -> Built {
        Built {
            argv,
            lane: lane.into(),
            readback: readback.iter().map(|k| k.to_string()).collect(),
            settle: None,
            before: false,
            settle_for: SETTLE_FAST,
            stdin: None,
            launch: false,
            affects: Vec::new(),
        }
    }

    /// Display kinds the command changes, dropped when it ends.
    fn touches(mut self, kinds: &[&str]) -> Built {
        self.affects = kinds.iter().map(|k| k.to_string()).collect();
        self
    }

    pub(super) fn settle(mut self, check: Check) -> Built {
        self.settle = Some(check);
        self
    }

    fn flipped(mut self, key: &'static str) -> Built {
        self.before = true;
        self.settle = Some(Box::new(
            move |before, now| matches!((before.get(key), now.get(key)), (Some(a), Some(b)) if a != b),
        ));
        self
    }

    /// A change that takes seconds to show: a radio associating, a device
    /// connecting, the bar reloading.
    pub(super) fn slow(mut self) -> Built {
        self.settle_for = SETTLE_SLOW;
        self
    }
}

// ---- what a settled read-back looks like --------------------------------

pub(super) fn equals(key: &'static str, want: Value) -> Check {
    Box::new(move |_, now| now.get(key) == Some(&want))
}

/// A row in the list under `key` whose `field` is `id`.
fn row<'a>(now: &'a Snap, key: &str, field: &str, id: &str) -> Option<&'a Value> {
    now.get(key)?
        .as_array()?
        .iter()
        .find(|row| row.get(field).and_then(Value::as_str) == Some(id))
}

pub(super) fn row_has(
    key: &'static str,
    field: &'static str,
    id: String,
    prop: &'static str,
    want: Value,
) -> Check {
    Box::new(move |_, now| row(now, key, field, &id).is_some_and(|r| r.get(prop) == Some(&want)))
}

pub(super) fn row_gone(key: &'static str, field: &'static str, id: String) -> Check {
    Box::new(move |_, now| now.contains_key(key) && row(now, key, field, &id).is_none())
}

fn names_gone(key: &'static str, name: String) -> Check {
    Box::new(move |_, now| {
        now.get(key).and_then(Value::as_array).is_some_and(|rows| {
            !rows.iter().any(|row| {
                row.as_str() == Some(&name)
                    || row.get("id").and_then(Value::as_str) == Some(&name)
                    || row.get("name").and_then(Value::as_str) == Some(&name)
            })
        })
    })
}

// ---- the registry -------------------------------------------------------

/// Every name `build` knows.
#[cfg(test)]
pub const NAMES: &[&str] = &[
    "audio.output.mute-toggle",
    "audio.input.mute-toggle",
    "audio.output.default",
    "audio.input.default",
    "power.profile",
    "wifi.band",
    "wifi.connection",
    "bluetooth.device",
    "bluetooth.discoverable",
    "plugin.set",
    "bar.spacer",
    "monitor.internal",
    "monitor.mirror",
    "keyboard.backlight",
    "display.text-size-reset",
    "theme.remove",
    "branding.reset",
    "hypr.reset",
    "hypr.workspace-layout-toggle",
    "hypr.apply",
    "atmos.channel",
    "weather.coordinates",
    "reminder.set",
    "reminder.clear",
    "recording.stop",
    "hook.install",
    "hook.create",
    "hook.remove",
    "hook.sample",
    "tui.remove",
    "webapp.remove",
    "app.remove-desktop",
    "avatar.set",
    "avatar.clear",
    "snapper.number-limit",
    "snapper.timeline",
    "fstrim",
];

/// Check the arguments of write `name` and build it.
pub fn build(name: &str, args: &Value) -> Result<Built> {
    let args = if args.is_null() { &Value::Null } else { args };
    Ok(match name {
        "audio.output.mute-toggle" => Built::new(
            "audio",
            "audioOutputMuted",
            s(&["omarchy", "audio", "output", "volume", "mute-toggle"]),
            &["audioOutputMuted"],
        )
        .flipped("audioOutputMuted"),
        "audio.input.mute-toggle" => Built::new(
            "audio",
            "audioInputMuted",
            s(&["omarchy", "audio", "input", "mute"]),
            &["audioInputMuted"],
        )
        .flipped("audioInputMuted"),
        "audio.output.default" | "audio.input.default" => {
            let (dir, key, sinks) = if name == "audio.output.default" {
                ("output", "audioSink", "audioSinks")
            } else {
                ("input", "audioSource", "audioSources")
            };
            let id = word(args, "id")?;
            let chosen = word(args, "name")?;
            let want = chosen.to_string();
            let list: &'static str = sinks;
            Built::new(
                "audio",
                key,
                s(&["omarchy", "audio", dir, "set", "default", id, chosen]),
                &[sinks],
            )
            .settle(Box::new(move |_, now| {
                row(now, list, "name", &want)
                    .is_some_and(|r| r.get("default") == Some(&json!(true)))
            }))
        }
        "power.profile" => {
            let profile = word(args, "name")?;
            Built::new(
                "power",
                "powerProfile",
                s(&["omarchy", "powerprofiles", "set", "autodetect", profile]),
                &["powerProfile"],
            )
            .settle(equals("powerProfile", json!(profile)))
        }
        "wifi.band" | "wifi.connection" | "bluetooth.device" | "bluetooth.discoverable" => {
            super::radio::build(name, args)?
        }
        "plugin.set" => {
            let id = word(args, "id")?;
            let on = flag(args, "on")?;
            Built::new(
                "plugin",
                &format!("plugin:{id}"),
                s(&[
                    "omarchy",
                    "plugin",
                    if on { "enable" } else { "disable" },
                    id,
                ]),
                &["plugins"],
            )
            .settle(row_has(
                "plugins",
                "id",
                id.to_string(),
                "enabled",
                json!(on),
            ))
            .slow()
        }
        "bar.spacer" => {
            let on = flag(args, "on")?;
            let argv = if on {
                s(&["omarchy", "bar", "put", "omarchy.spacer"])
            } else {
                s(&["omarchy", "plugin", "disable", "omarchy.spacer"])
            };
            Built::new("plugin", "spacerPresent", argv, &["spacerPresent"])
                .settle(equals("spacerPresent", json!(on)))
                .slow()
        }
        "monitor.internal" => {
            let on = flag(args, "on")?;
            Built::new(
                "hypr:monitors",
                "internalEnabled",
                s(&[
                    "omarchy",
                    "hyprland",
                    "monitor",
                    "internal",
                    if on { "on" } else { "off" },
                ]),
                &["internalEnabled", "internalPresent", "mirroring"],
            )
            .settle(equals("internalEnabled", json!(on)))
        }
        "monitor.mirror" => {
            let on = flag(args, "on")?;
            Built::new(
                "hypr:monitors",
                "mirroring",
                s(&[
                    "omarchy",
                    "hyprland",
                    "monitor",
                    "internal",
                    "mirror",
                    if on { "on" } else { "off" },
                ]),
                &["mirroring", "internalEnabled", "internalPresent"],
            )
            .settle(equals("mirroring", json!(on)))
        }
        "keyboard.backlight" => {
            let direction = one_of(args, "direction", &["up", "down", "off"])?;
            Built::new(
                "keyboard",
                "keyboardBrightness",
                s(&["omarchy", "brightness", "keyboard", "--no-osd", direction]),
                &["keyboardBrightness"],
            )
        }
        "display.text-size-reset" => Built::new(
            "display",
            "textSize",
            s(&["omarchy", "display", "text", "size", "reset"]),
            &["textSize"],
        )
        .settle(equals("textSize", json!(12))),
        "theme.remove" => {
            let theme = word(args, "name")?;
            if theme.contains('/') || theme.starts_with('.') {
                return Err(bad("name", "not a theme name"));
            }
            Built::new(
                "theme",
                &format!("theme-remove:{theme}"),
                s(&["omarchy", "theme", "remove", theme]),
                &["extraThemes", "themes", "theme"],
            )
            .settle(names_gone("extraThemes", theme.to_string()))
        }
        "branding.reset" => {
            let which = one_of(args, "which", &["screensaver", "about"])?;
            let key = if which == "about" {
                "aboutBranded"
            } else {
                "screensaverBranded"
            };
            Built::new(
                "branding",
                key,
                s(&["omarchy", "branding", which, "reset"]),
                &[key],
            )
            .settle(equals(key, json!(false)))
        }
        "hypr.reset" => {
            let group = one_of(args, "group", &["look", "input"])?;
            let (key, managed) = if group == "look" {
                ("hyprLook", "hyprLookManaged")
            } else {
                ("hyprInput", "hyprInputManaged")
            };
            Built::new(
                "hypr:config",
                managed,
                task("reset-hypr", &[group]),
                &[key, managed],
            )
            .settle(equals(managed, json!(false)))
        }
        "hypr.workspace-layout-toggle" => Built::new(
            "hypr:config",
            "hyprWorkspaceLayout",
            s(&["omarchy", "hyprland", "workspace", "layout", "toggle"]),
            &["hyprWorkspaceLayout"],
        )
        .flipped("hyprWorkspaceLayout"),
        "hypr.apply" => {
            let group = one_of(args, "group", &["look", "input"])?;
            let payload = args
                .get("payload")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("payload", "a string is required"))?;
            if payload.len() > 64 * 1024 {
                return Err(bad("payload", "too long"));
            }
            let (key, managed) = if group == "look" {
                ("hyprLook", "hyprLookManaged")
            } else {
                ("hyprInput", "hyprInputManaged")
            };
            Built::new(
                "hypr:config",
                key,
                task("hypr-apply", &[group, payload]),
                &[key, managed],
            )
        }
        "atmos.channel" => {
            let channel = one_of(args, "channel", &["stable"])?;
            Built::new(
                "atmos",
                "atmosChannel",
                task("set-atmos-channel", &[channel]),
                &["atmosChannel"],
            )
            .settle(equals("atmosChannel", json!(channel)))
        }
        "weather.coordinates" => {
            let place = word(args, "name")?;
            let coords = coords(args, "coords")?;
            Built::new(
                "weather",
                "weatherCoords",
                s(&["omarchy", "weather", "location", "--set", place, coords]),
                &["weatherCoords", "weatherLocation", "weatherAuto"],
            )
        }
        "reminder.set" => {
            let minutes = uint(args, "minutes", 1, 100_000)?.to_string();
            let mut argv = s(&["omarchy", "reminder", &minutes]);
            if args
                .get("message")
                .and_then(Value::as_str)
                .is_some_and(|m| !m.is_empty())
            {
                argv.push(text(args, "message")?.to_string());
            }
            Built::new(
                "reminder",
                "reminder",
                argv,
                &["reminderActive", "reminderCount", "reminders"],
            )
            .settle(equals("reminderActive", json!(true)))
        }
        "reminder.clear" => Built::new(
            "reminder",
            "reminder",
            s(&["omarchy", "reminder", "clear"]),
            &["reminderActive", "reminderCount", "reminders"],
        )
        .settle(equals("reminderActive", json!(false))),
        "recording.stop" => Built::new(
            "recording",
            "recordingActive",
            s(&["omarchy", "capture", "screenrecording", "--stop-recording"]),
            &["recordingActive"],
        )
        .settle(equals("recordingActive", json!(false))),
        "hook.install" => {
            let kind = hook_type(args)?;
            let source = word(args, "path")?;
            if !source.starts_with('/') || source.split('/').any(|part| part == "..") {
                return Err(bad("path", "an absolute path is required"));
            }
            Built::new(
                "hooks",
                "hooks",
                s(&["omarchy", "hook", "install", kind, source]),
                &["hooks"],
            )
        }
        "hook.create" => {
            let kind = hook_type(args)?;
            let hook = word(args, "name")?;
            let command = text(args, "command")?;
            if hook.contains('/') || hook.contains("..") {
                return Err(bad("name", "not a hook name"));
            }
            Built::new(
                "hooks",
                "hooks",
                task("create-hook", &[kind, hook, command]),
                &["hooks"],
            )
        }
        "hook.remove" => {
            let path = hook_path(args, "path")?;
            if path.ends_with(".sample") {
                return Err(bad("path", "a sample is not removed"));
            }
            // Removed in-process (see `execute`): nothing but a file inside
            // the hooks directory can be named.
            Built::new("hooks", "hooks", s(&["rm", "-f", path]), &["hooks"]).settle(row_gone(
                "hooks",
                "path",
                path.to_string(),
            ))
        }
        "hook.sample" => {
            let path = hook_path(args, "path")?;
            let enabled = flag(args, "enabled")?;
            Built::new(
                "hooks",
                "hooks",
                task(
                    "set-hook-sample",
                    &[if enabled { "enable" } else { "disable" }, path],
                ),
                &["hooks"],
            )
        }
        "tui.remove" | "webapp.remove" => {
            let target = word(args, "name")?;
            let (cmd, key) = if name == "tui.remove" {
                (["omarchy", "tui", "remove"], "tuiApps")
            } else {
                (["omarchy", "webapp", "remove"], "webApps")
            };
            let mut argv = s(&cmd);
            argv.push(target.to_string());
            Built::new("apps", &format!("{key}:{target}"), argv, &[key])
                .settle(names_gone(key, target.to_string()))
        }
        "app.remove-desktop" => {
            let id = word(args, "id")?;
            let shown = args
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.is_empty())
                .unwrap_or(id);
            if shown.starts_with('-') || shown.contains('\0') {
                return Err(bad("name", "not plain text"));
            }
            Built::new(
                "apps",
                &format!("desktop-remove:{id}"),
                s(&["omarchy", "remove", "launcher", "entry", id, shown]),
                &["desktopApps"],
            )
            .settle(names_gone("desktopApps", id.to_string()))
            .touches(&["software"])
        }
        "avatar.set" => {
            let user = word(args, "user")?;
            let path = word(args, "path")?;
            if !path.starts_with('/') || path.contains("..") {
                return Err(bad("path", "an absolute path is required"));
            }
            Built::new(
                "avatar",
                "avatar",
                task("set-avatar", &["set", user, path]),
                &["avatarPath"],
            )
            .settle(equals("avatarPath", json!(path)))
        }
        "avatar.clear" => {
            let user = word(args, "user")?;
            Built::new(
                "avatar",
                "avatar",
                task("set-avatar", &["clear", user]),
                &["avatarPath"],
            )
            .settle(equals("avatarPath", json!("")))
        }
        "snapper.number-limit" => {
            let n = uint(args, "n", 1, 50)?;
            Built::new(
                "snapper",
                "snapperNumberLimit",
                task("set-snapper-policy", &["number-limit", &n.to_string()]),
                &["snapperNumberLimit"],
            )
            .settle(equals("snapperNumberLimit", json!(n)))
        }
        "snapper.timeline" => {
            let on = flag(args, "on")?;
            Built::new(
                "snapper",
                "snapperTimeline",
                task(
                    "set-snapper-policy",
                    &["timeline", if on { "on" } else { "off" }],
                ),
                &["snapperTimeline"],
            )
            .settle(equals("snapperTimeline", json!(on)))
        }
        "fstrim" => {
            let on = flag(args, "on")?;
            Built::new(
                "snapper",
                "fstrimEnabled",
                task("set-fstrim", &[if on { "on" } else { "off" }]),
                &["fstrimEnabled"],
            )
            .settle(equals("fstrimEnabled", json!(on)))
        }
        other => match super::entries::find(other).filter(|entry| {
            matches!(
                entry.kind,
                super::table::Kind::Command | super::table::Kind::Launch
            )
        }) {
            Some(entry) => from_table(entry, args)?,
            None => {
                return Err(
                    Error::new(Kind::BadRequest, format!("unknown write {other}"))
                        .with_context("name"),
                )
            }
        },
    })
}

/// A command or launcher from the table: no read-back, no settling.
fn from_table(entry: &super::table::Entry, args: &Value) -> Result<Built> {
    use super::table;
    let lane = if entry.lane.is_empty() {
        entry.name
    } else {
        entry.lane
    };
    let mut built = Built::new(lane, entry.name, table::argv(entry, args)?, &[]);
    built.stdin = table::stdin(entry, args)?;
    built.launch = entry.kind == table::Kind::Launch;
    built.affects = entry.affects.iter().map(|k| k.to_string()).collect();
    built.readback = entry.readback.iter().map(|k| k.to_string()).collect();
    Ok(built)
}

/// Start `built` as a unit of its own and do not wait for it. Without a user
/// manager it is a detached child, which does not outlive a daemon restart but
/// does not hold the lane either.
fn launch(built: &Built) -> Result<()> {
    use crate::sampler::jobs::{unit_argv, Runner};
    static LAUNCHED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = LAUNCHED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let env: Vec<(String, String)> = std::env::vars().collect();
    if Runner::detect() == Runner::Systemd {
        let unit = format!("atmos-launch-{}-{n}", std::process::id());
        let argv = unit_argv(&unit, "Atmos launch", &built.argv, &env, false);
        let (program, rest) = argv
            .split_first()
            .ok_or_else(|| Error::new(Kind::Failed, "empty"))?;
        let done = Run::new(program)
            .args(rest)
            .timeout(Duration::from_secs(20))
            .output()?;
        if done.success() {
            return Ok(());
        }
        return Err(Error::new(
            Kind::Command,
            format!("{}: {}", built.argv.join(" "), done.stderr_text().trim()),
        ));
    }
    use std::os::unix::process::CommandExt;
    let (program, rest) = built
        .argv
        .split_first()
        .ok_or_else(|| Error::new(Kind::Failed, "empty"))?;
    std::process::Command::new(program)
        .args(rest)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .process_group(0)
        .spawn()
        .map(|_| ())
        .map_err(|err| Error::from(err).with_context(program.clone()))
}

// ---- running one --------------------------------------------------------

/// Reads of the touched keys made after the command, and the gap between them.
pub(super) const SETTLE_GAP: Duration = Duration::from_millis(150);
/// How long a read-back is waited on: most writes show at once or within a
/// moment, a radio or a device takes seconds.
pub(super) const SETTLE_FAST: Duration = Duration::from_millis(1500);
const SETTLE_SLOW: Duration = Duration::from_secs(10);

/// Run the write and answer with what the machine reports afterwards:
/// `{snapshot, settled}`. Under a fixture root nothing runs and the command
/// line is returned instead, so tests can see what would have.
pub fn execute(root: Option<&Path>, built: &Built) -> Result<Value> {
    if root.is_some() {
        return Ok(json!({"argv": built.argv, "snapshot": {}, "settled": true}));
    }
    let before = if built.before {
        crate::snapshot::topics::read_keys(&built.readback).values
    } else {
        Map::new()
    };
    run_command(built)?;
    let (mut now, mut failed);
    let mut settled = built.settle.is_none();
    let began = std::time::Instant::now();
    loop {
        // Read as the topic is collected, so what is patched in is what a collection shows.
        let read = crate::snapshot::topics::read_keys(&built.readback);
        (now, failed) = (read.values, read.failed);
        match &built.settle {
            None => break,
            Some(check) if check(&before, &now) => {
                settled = true;
                break;
            }
            Some(_) if began.elapsed() >= built.settle_for => break,
            Some(_) => std::thread::sleep(SETTLE_GAP),
        }
    }
    // A key the machine could not be asked about is named, and left out of the
    // snapshot, so the window keeps what it showed.
    let mut reply = json!({"snapshot": now, "settled": settled});
    if !failed.is_empty() {
        reply["failed"] = json!(failed);
    }
    Ok(reply)
}

fn run_command(built: &Built) -> Result<()> {
    let (program, rest) = built
        .argv
        .split_first()
        .ok_or_else(|| Error::new(Kind::Failed, "an empty command"))?;
    // A hook is removed here, not by `rm`: the path was already confined.
    if program == "rm" {
        if let Some(path) = rest.last() {
            return match std::fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(err) => Err(Error::from(err).with_context(path.clone())),
            };
        }
    }
    if built.launch {
        return launch(built);
    }
    let mut run = Run::new(program)
        .args(rest)
        .timeout(Duration::from_secs(120));
    if let Some(input) = &built.stdin {
        run = run.input(input.clone());
    }
    let done = run.output()?;
    if done.success() {
        return Ok(());
    }
    Err(Error::new(
        Kind::Command,
        format!("{}: {}", built.argv.join(" "), done.stderr_text().trim()),
    )
    .with_context(program.clone()))
}

#[cfg(test)]
#[path = "writes_tests.rs"]
mod tests;
