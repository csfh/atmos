//! Live stats and the inventories the Quickshell pages already show.

use crate::error::{Error, Result};
use crate::platform::Backend;
use crate::runner::Run;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value};

pub const KINDS: &[&str] = &[
    "live",
    "hardware",
    "disks",
    "services",
    "software",
    "diagnostics",
];

pub fn is_kind(kind: &str) -> bool {
    KINDS.contains(&kind)
}

pub fn display_rel(platform: Backend, kind: &str) -> String {
    format!(".local/state/{}/display/{kind}.json", platform.id())
}

pub fn collector(platform: Backend, kind: &str) -> &'static str {
    if platform == Backend::Plain {
        return match kind {
            "live" => "plain-sampler",
            "hardware" => "plain-hardware",
            "disks" => "plain-disks",
            "services" => "plain-services",
            "software" => "plain-software",
            "diagnostics" => "plain-diagnostics",
            _ => "plain",
        };
    }
    match kind {
        "live" => "live-stats.py",
        "hardware" => "hw-inventory.py",
        "disks" => "disk-inventory.py",
        "services" => "systemd-inventory.py",
        "software" => "list-apps.py",
        "diagnostics" => "diag-inventory.py",
        _ => "omarchy",
    }
}

/// Read one display document.
///
/// A fixture file under the config root wins, so tests stay deterministic.
/// Without a root, the Omarchy backend runs the inventory script the GUI
/// already used. The plain backend has no host scripts.
pub fn load(
    root: Option<&Path>,
    platform: Backend,
    kind: &str,
    sampler: Option<&Path>,
) -> Result<Value> {
    if !is_kind(kind) {
        return Err(format!("unknown display kind {kind}").into());
    }
    if let Some(root) = root {
        let path = root.join(display_rel(platform, kind));
        if path.is_file() {
            let text = std::fs::read_to_string(&path)?;
            let value: Value = serde_json::from_str(&text)?;
            return Ok(stamp(value, platform, collector(platform, kind)));
        }
        return Ok(stamp(
            Value::Object(Map::new()),
            platform,
            collector(platform, kind),
        ));
    }
    if platform != Backend::Omarchy {
        return Ok(stamp(
            Value::Object(Map::new()),
            platform,
            collector(platform, kind),
        ));
    }
    let output = run_script(kind, sampler)?;
    Ok(stamp(output, platform, collector(platform, kind)))
}

fn run_script(kind: &str, sampler: Option<&Path>) -> Result<Value> {
    let path = if kind == "live" {
        if let Some(sampler) = sampler {
            sampler.to_path_buf()
        } else {
            script_path(kind)?
        }
    } else {
        script_path(kind)?
    };
    if !path.is_file() {
        return Err(format!("missing {}", path.display()).into());
    }
    let output = Run::new("python3")
        .arg(&path)
        .timeout(Duration::from_secs(if kind == "live" { 10 } else { 60 }))
        .checked()
        .map_err(|err| err.with_context(path.display().to_string()))?;
    serde_json::from_slice(&output.stdout)
        .map_err(|err| Error::from(format!("{}: {err}", path.display())))
}

fn script_path(kind: &str) -> Result<PathBuf> {
    crate::scripts::repo_script(collector(Backend::Omarchy, kind))
}

fn stamp(value: Value, platform: Backend, collector: &str) -> Value {
    let mut map = match value {
        Value::Object(map) => map,
        Value::Array(items) => {
            let mut map = Map::new();
            map.insert("items".into(), Value::Array(items));
            map
        }
        other => {
            let mut map = Map::new();
            map.insert("value".into(), other);
            map
        }
    };
    map.insert("platform".into(), Value::String(platform.id().into()));
    map.insert("collector".into(), Value::String(collector.into()));
    Value::Object(map)
}
