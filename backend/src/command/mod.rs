//! Typed commands: the writes that go through the daemon's command bus.
//!
//! A command names its lane, an optional coalescing key and the topics it
//! affects in one place (`plan`). The daemon runs commands in order on a lane
//! and holds back reads of the affected topics until they finish (`reads` says
//! which reads those are). The window asks, the daemon orders, and the work
//! itself happens in a `ratmos request` child that carries the caller's
//! environment, so a long-lived daemon never runs a tool with a stale
//! `HYPRLAND_INSTANCE_SIGNATURE`.

mod args;
pub mod entries;
pub mod jobs;
pub mod table;
pub mod writes;

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

use crate::error::{Error, Result};
use crate::request::Request;
use crate::runner::Run;
use crate::snapshot::{hypr, sh};

/// Set in the child the daemon starts for a command, so it runs the work
/// itself instead of asking the daemon again.
pub const DIRECT: &str = "RATMOS_DIRECT";

/// How long a read waits for the writes ahead of it before it answers anyway.
pub const BARRIER_WAIT: Duration = Duration::from_secs(3);

/// The topic that names the monitor list in the look snapshot.
pub const MONITORS: &str = "monitors";

/// Every write names it, and every read of the settings waits on it.
pub const SETTINGS: &str = "settings";

/// Where a command runs and what it touches.
#[derive(Debug, PartialEq)]
pub struct Plan {
    pub lane: String,
    pub key: Option<String>,
    pub affects: Vec<String>,
}

/// None for a request that is not a command.
pub fn plan(request: &Request) -> Option<Plan> {
    match request {
        Request::Write { name, args } => {
            let built = writes::build(name, args).ok()?;
            let mut affects = vec![SETTINGS.to_string()];
            affects.extend(built.affects);
            Some(Plan {
                lane: built.lane,
                key: None,
                affects,
            })
        }
        Request::DisplayBrightness { monitor, .. } => Some(Plan {
            // One panel's brightness is one device: its writes and its reads
            // share a lane, and a dragged slider collapses to its last value.
            lane: format!("monitor:{monitor}"),
            key: Some("brightness".into()),
            affects: vec![MONITORS.into(), SETTINGS.into()],
        }),
        Request::DisplayScale { .. } => Some(Plan {
            // The focused monitor's scale and the monitor list move together.
            lane: "hypr:monitors".into(),
            key: Some("scale".into()),
            affects: vec![MONITORS.into(), SETTINGS.into()],
        }),
        // Every write the server used to make in its own process goes through
        // a lane too (docs/adr/0011). A lane names the whole target, so two
        // different writes never share one and neither is lost.
        Request::SettingsSet { domain, .. } => Some(Plan {
            lane: format!("settings:{domain}"),
            key: None,
            // A setting can start or stop a unit, and its own topic holds it.
            affects: vec![
                SETTINGS.into(),
                "services".into(),
                crate::snapshot::topics::topic_of_domain(domain).into(),
            ],
        }),
        Request::HostWrite(file) => Some(Plan {
            lane: format!("file:{}", file.path),
            key: None,
            affects: Vec::new(),
        }),
        Request::HostOpen { .. } => Some(Plan {
            lane: "open".into(),
            key: None,
            affects: Vec::new(),
        }),
        Request::AgentsMcpSet { agent, .. } => Some(Plan {
            lane: format!("agents:{agent}"),
            key: None,
            affects: Vec::new(),
        }),
        _ => None,
    }
}

/// A read the daemon runs for a window: a lane of its own, nothing affected.
pub fn read_plan() -> Plan {
    static READS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = READS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Plan {
        lane: format!("read:{n}"),
        key: None,
        affects: Vec::new(),
    }
}

/// The topics a read has to see current before it answers.
pub fn reads(request: &Request) -> Vec<String> {
    match request {
        Request::SettingsSnapshot { group, .. } => {
            let mut topics = vec![SETTINGS.to_string()];
            if matches!(group.as_str(), "look" | "rest" | "all") {
                topics.push(MONITORS.into());
            }
            topics
        }
        Request::SettingsGet { .. } | Request::SnapshotKeys { .. } => vec![SETTINGS.into()],
        // A read of a topic waits for the writes ahead of it, as every read does.
        Request::SnapshotInvalidate { .. } => vec![SETTINGS.into()],
        Request::DisplayGet { kind } => vec![kind.clone()],
        Request::DisplaySnapshot => crate::display::KINDS
            .iter()
            .map(|k| k.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// `snapshot.keys`: what the machine reports now for some keys. With `expect`,
/// the read is repeated for a moment until every expected value shows.
pub fn snapshot_keys(
    root: Option<&Path>,
    keys: &[String],
    expect: &serde_json::Map<String, Value>,
) -> Value {
    if root.is_some() {
        let unknown: Vec<&String> = keys
            .iter()
            .filter(|k| !crate::snapshot::keys::known(k))
            .collect();
        return json!({"values": {}, "unknown": unknown, "settled": true});
    }
    let mut wanted: Vec<String> = keys.to_vec();
    for key in expect.keys() {
        if !wanted.contains(key) {
            wanted.push(key.clone());
        }
    }
    let began = std::time::Instant::now();
    let (mut last, mut settled);
    loop {
        last = crate::snapshot::keys::read(&wanted);
        settled = expect.iter().all(|(k, v)| last.values.get(k) == Some(v));
        if settled || began.elapsed() >= writes::SETTLE_FAST {
            break;
        }
        std::thread::sleep(writes::SETTLE_GAP);
    }
    json!({"values": last.values, "unknown": last.unknown, "failed": last.failed, "settled": settled})
}

/// `omarchy brightness display`, then the monitor list as the machine now
/// reports it, so the window shows what is true instead of what it guessed.
pub fn brightness(root: Option<&Path>, monitor: &str, percent: u32) -> Result<Value> {
    if !sh::is_ident(monitor) {
        return Err(Error::bad_request("monitor: not a monitor name").with_context("monitor"));
    }
    if percent > 100 {
        return Err(Error::bad_request("percent: at most 100").with_context("percent"));
    }
    // A fixture root only answers, like the other writes.
    if root.is_some() {
        return Ok(json!({"monitor": monitor, "percent": percent}));
    }
    let shown = format!("{percent}%");
    let done = Run::new("omarchy")
        .args([
            "brightness",
            "display",
            "--no-osd",
            "--monitor",
            monitor,
            &shown,
        ])
        .timeout(Duration::from_secs(30))
        .output()?;
    if !done.success() {
        let why = done.stderr_text();
        return Err(Error::new(
            crate::error::Kind::Command,
            format!("omarchy brightness: {}", why.trim()),
        )
        .with_context("omarchy brightness display"));
    }
    // A panel on DDC/CI answers a read right after a write with the old level.
    // Wait until it reports the new one (a panel may round by a point), so the
    // window is shown what is true and never has to guess.
    let settled = settle(monitor, u64::from(percent));
    let mut result = json!({"monitor": monitor, "percent": percent});
    // No list when hyprctl failed: the window keeps the one it has.
    if let Some(mut rows) = hypr::monitors() {
        if let Some(level) = settled {
            for row in rows.iter_mut().filter(|row| row["name"] == monitor) {
                row["brightness"] = json!(level);
                row["brightnessAvailable"] = json!(true);
            }
        }
        result["snapshot"] = json!({"monitors": rows});
    }
    Ok(result)
}

/// `omarchy hyprland monitor scaling`, then the monitors as the machine now
/// reports them.
pub fn scale(root: Option<&Path>, scale: &str) -> Result<Value> {
    let valid = scale
        .parse::<f64>()
        .is_ok_and(|n| n.is_finite() && n > 0.0 && n <= 8.0)
        && scale.chars().all(|c| c.is_ascii_digit() || c == '.');
    if !valid {
        return Err(
            Error::bad_request("scale: a number above 0 and at most 8").with_context("scale")
        );
    }
    if root.is_some() {
        return Ok(json!({"scale": scale}));
    }
    let done = Run::new("omarchy")
        .args(["hyprland", "monitor", "scaling", scale])
        .timeout(Duration::from_secs(30))
        .output()?;
    if !done.success() {
        return Err(Error::new(
            crate::error::Kind::Command,
            format!(
                "omarchy hyprland monitor scaling: {}",
                done.stderr_text().trim()
            ),
        )
        .with_context("omarchy hyprland monitor scaling"));
    }
    let mut result = json!({"scale": scale});
    if let Some(rows) = hypr::monitors() {
        result["snapshot"] = json!({"monitors": rows});
    }
    Ok(result)
}

/// Reads of a monitor made after a write, and how long between them.
const SETTLE_READS: u32 = 8;
const SETTLE_GAP: Duration = Duration::from_millis(120);

/// The level the panel reports once it agrees with `want` (within a point), or
/// None if it never does in the time allowed.
fn settle(monitor: &str, want: u64) -> Option<u64> {
    for read in 0..SETTLE_READS {
        if read > 0 {
            std::thread::sleep(SETTLE_GAP);
        }
        if let Some(level) = hypr::brightness(monitor) {
            if level.abs_diff(want) <= 1 {
                return Some(level);
            }
        }
    }
    None
}

/// What the daemon's lane worker runs: this binary as a one-shot `request`
/// with the caller's environment. The envelope it prints is the answer.
pub fn run_child(backend_id: &str, body: &Value, env: &[(String, String)]) -> Value {
    let failed = |message: String| json!({"ok": false, "error": {"code": "command", "message": message, "context": null}, "result": null});
    let Ok(exe) = std::env::current_exe() else {
        return failed("cannot find the ratmos binary".into());
    };
    let mut run = Run::new(exe)
        .args(["--backend", backend_id, "request"])
        .input(body.to_string())
        .clear_env()
        .env(DIRECT, "1")
        .timeout(Duration::from_secs(120));
    for (key, value) in env {
        run = run.env(key, value);
    }
    match run.output() {
        Ok(out) => serde_json::from_str::<Value>(out.stdout_text().trim())
            .unwrap_or_else(|_| failed(format!("no answer from the command (exit {})", out.code))),
        Err(err) => failed(err.to_string()),
    }
}

/// An envelope's error as a backend error, so a command that failed in the
/// child fails here the way it would have inline.
pub fn error_of(envelope: &Value) -> Error {
    let error = &envelope["error"];
    let kind = crate::error::Kind::from_code(error["code"].as_str().unwrap_or(""));
    let mut out = Error::new(
        kind,
        error["message"].as_str().unwrap_or("the command failed"),
    );
    if let Some(context) = error["context"].as_str() {
        out = out.with_context(context);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(body: Value) -> Request {
        Request::parse(body).unwrap()
    }

    #[test]
    fn brightness_is_one_lane_per_monitor_with_a_coalescing_key() {
        let plan = plan(&parse(
            json!({"op": "display.brightness", "monitor": "DP-3", "percent": 40}),
        ))
        .unwrap();
        assert_eq!(plan.lane, "monitor:DP-3");
        assert_eq!(plan.key.as_deref(), Some("brightness"));
        assert_eq!(plan.affects, ["monitors", "settings"]);
        let other = super::plan(&parse(
            json!({"op": "display.brightness", "monitor": "eDP-1", "percent": 40}),
        ))
        .unwrap();
        assert_ne!(plan.lane, other.lane);
    }

    #[test]
    fn scale_is_one_lane_for_the_monitor_list() {
        let plan = plan(&parse(json!({"op": "display.scale", "scale": "1.5"}))).unwrap();
        assert_eq!(plan.lane, "hypr:monitors");
        assert_eq!(plan.key.as_deref(), Some("scale"));
        assert_eq!(plan.affects, ["monitors", "settings"]);
    }

    #[test]
    fn a_bad_scale_is_refused_before_anything_runs() {
        for bad in ["", "0", "-1", "1.5x", "9", "abc", "1e2", " 2"] {
            let err = scale(None, bad).unwrap_err();
            assert_eq!(err.kind, crate::error::Kind::BadRequest, "{bad:?}");
        }
        assert_eq!(
            scale(Some(Path::new("/nonexistent")), "1.25").unwrap(),
            json!({"scale": "1.25"})
        );
    }

    #[test]
    fn what_serve_used_to_write_itself_has_a_lane_named_for_its_whole_target() {
        let set = |domain: &str| {
            plan(&parse(
                json!({"op": "settings.set", "domain": domain, "value": 1}),
            ))
            .unwrap()
        };
        assert_eq!(set("theme").lane, "settings:theme");
        assert_ne!(set("theme").lane, set("font").lane);
        assert_eq!(set("theme").key, None, "a setting is never coalesced");
        assert_eq!(
            set("theme").affects,
            ["settings", "services", "settings.look"],
            "a setting invalidates the topic that holds it"
        );
        assert_eq!(set("hostname").affects[2], "settings.system");
        let file = |path: &str| {
            plan(&parse(
                json!({"op": "host.write", "path": path, "text": "x"}),
            ))
            .unwrap()
        };
        assert_ne!(file("/a").lane, file("/b").lane);
        let mcp = |agent: &str| {
            plan(&parse(
                json!({"op": "agents.mcp.set", "agent": agent, "on": true}),
            ))
            .unwrap()
        };
        assert_ne!(mcp("claude").lane, mcp("codex").lane);
        assert!(plan(&parse(json!({"op": "host.open", "path": "/a"}))).is_some());
    }

    #[test]
    fn a_systemd_write_and_an_app_removal_drop_the_inventory_they_change() {
        let plan_of = |name: &str, args: Value| {
            plan(&parse(json!({"op": "write", "name": name, "args": args}))).unwrap()
        };
        let unit = plan_of(
            "systemd.user",
            json!({"action": "restart", "unit": "waybar.service"}),
        );
        assert_eq!(unit.affects, ["settings", "services"]);
        let app = plan_of("app.remove-desktop", json!({"id": "x.desktop"}));
        assert!(app.affects.contains(&"software".to_string()));
        let toggle = plan_of("audio.output.mute-toggle", json!({}));
        assert_eq!(toggle.affects, ["settings"]);
    }

    #[test]
    fn only_commands_have_a_plan() {
        assert!(plan(&parse(json!({"op": "version"}))).is_none());
        assert!(plan(&parse(json!({"op": "settings.get", "domain": "theme"}))).is_none());
    }

    #[test]
    fn the_reads_that_see_monitors_wait_for_brightness() {
        for group in ["look", "rest", "all"] {
            assert_eq!(
                reads(&parse(json!({"op": "settings.snapshot", "group": group}))),
                ["settings", "monitors"],
                "{group}"
            );
        }
        assert_eq!(
            reads(&parse(
                json!({"op": "settings.snapshot", "group": "network"})
            )),
            ["settings"]
        );
        assert_eq!(
            reads(&parse(json!({"op": "display.get", "kind": "disks"}))),
            ["disks"]
        );
        assert_eq!(reads(&parse(json!({"op": "display.snapshot"}))).len(), 6);
        assert!(reads(&parse(json!({"op": "version"}))).is_empty());
    }

    #[test]
    fn a_bad_monitor_or_percent_is_refused_before_anything_runs() {
        assert!(brightness(None, "DP 3; rm", 10).is_err());
        assert!(brightness(None, "", 10).is_err());
        let err = brightness(None, "DP-3", 101).unwrap_err();
        assert_eq!(err.kind, crate::error::Kind::BadRequest);
    }

    #[test]
    fn a_fixture_root_only_answers() {
        let out = brightness(Some(Path::new("/nonexistent")), "DP-3", 40).unwrap();
        assert_eq!(out, json!({"monitor": "DP-3", "percent": 40}));
    }

    #[test]
    fn an_envelope_error_keeps_its_kind_and_context() {
        let envelope =
            json!({"ok": false, "error": {"code": "denied", "message": "no", "context": "x"}});
        let err = error_of(&envelope);
        assert_eq!(err.kind, crate::error::Kind::Denied);
        assert_eq!(err.context.as_deref(), Some("x"));
        let odd = error_of(&json!({"ok": false}));
        assert_eq!(odd.kind, crate::error::Kind::Failed);
    }
}
