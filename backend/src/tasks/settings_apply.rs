//! `ratmos settings-apply`: apply a settings plan from `planImport()`.
//!
//! The plan arrives on stdin (or `--plan F`) as
//!   { "schema": 1, "changes": [ { "key": ..., "value": ..., "from": ... } ] }
//! and every change goes through the writer that already owns that setting,
//! the same one `settings.set` uses. A failing change is reported and the rest
//! still run: stopping halfway leaves the most confusing state possible.
//!
//!   --dry-run     print each change and write nothing
//!   --progress    name each change before it runs
//!   --no-backup   leave no undo plan and no file copies
//!   --plan F      read the plan from F instead of stdin
//!   --backup DIR  where to leave the undo plan and file copies

use crate::error::{Error, Result};
use crate::inventory::time::utc_stamp;
use crate::platform::Backend;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::PathBuf;

fn bad(message: impl Into<String>) -> Error {
    Error::from(message.into())
}

/// The file a writer may rewrite, so a copy exists before it does.
fn backup_file(key: &str, home: &str) -> Option<PathBuf> {
    let hypr = |name: &str| PathBuf::from(home).join(".config/hypr").join(name);
    let head = key.split('.').next().unwrap_or(key);
    Some(match head {
        "hyprLook" => hypr("looknfeel.lua"),
        "hyprInput" => hypr("input.lua"),
        "bindings" => hypr("bindings.lua"),
        "autostart" => hypr("autostart.lua"),
        "windowRules" | "windowOpaque" | "workspaces" => hypr("atmos.lua"),
        "monitorRules" => hypr("monitors.lua"),
        "nightlightDay" | "nightlightNight" | "nightlightNightOn" | "nightlightTemperature" => {
            std::env::var("ATMOS_HYPRSUNSET_FILE")
                .ok()
                .filter(|v| !v.is_empty())
                .map_or_else(|| hypr("hyprsunset.conf"), PathBuf::from)
        }
        k if k.starts_with("clock")
            || k.starts_with("bar")
            || k.starts_with("workspaceBar")
            || k.starts_with("idle") =>
        {
            PathBuf::from(home).join(".config/omarchy/shell.json")
        }
        _ => return None,
    })
}

fn compact_stamp(secs: i64) -> String {
    // 2026-10-14 09:05 UTC -> 20261014T090500Z (seconds are not kept, so add :00)
    let text = utc_stamp(secs);
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    format!("{}T{}00Z", &digits[..8], &digits[8..12])
}

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let (mut dry, mut progress, mut no_backup) = (false, false, false);
    let (mut plan_file, mut backup_dir): (Option<String>, Option<String>) = (None, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => dry = true,
            "--progress" => progress = true,
            "--no-backup" => no_backup = true,
            "--plan" => {
                i += 1;
                plan_file = args.get(i).cloned();
            }
            "--backup" => {
                i += 1;
                backup_dir = args.get(i).cloned();
            }
            _ => return Err(bad("Usage: settings-apply [--dry-run] [--progress] [--no-backup] [--plan <file>] [--backup <dir>]")),
        }
        i += 1;
    }
    let raw = match &plan_file {
        Some(file) => std::fs::read_to_string(file)
            .map_err(|_| bad(format!("settings-apply: cannot read {file}")))?,
        None => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            text
        }
    };
    let plan: Value = serde_json::from_str(&raw)
        .map_err(|_| bad("settings-apply: the plan needs a changes array"))?;
    let Some(changes) = plan.get("changes").and_then(Value::as_array) else {
        return Err(bad("settings-apply: the plan needs a changes array"));
    };
    let schema = plan.get("schema").and_then(Value::as_i64).unwrap_or(0);
    if schema != 1 {
        return Err(bad(format!(
            "settings-apply: this plan is schema {schema} and this Atmos applies 1"
        )));
    }
    if changes.is_empty() {
        writeln!(stdout, "Nothing to apply.")?;
        return Ok(0);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let state = std::env::var("XDG_STATE_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{home}/.local/state"));
    let explicit = backup_dir.is_some();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let dir = PathBuf::from(
        backup_dir.unwrap_or_else(|| format!("{state}/atmos/imports/{}", compact_stamp(now))),
    );
    // A dry run writes the undo plan only when you name a directory, so you can
    // read the way back before you commit to the way forward. Undoing must not
    // leave a way back of its own, so --no-backup skips it.
    if !no_backup && (!dry || explicit) {
        std::fs::create_dir_all(&dir)?;
        let undo: Vec<Value> = changes
            .iter()
            .filter(|c| !c.get("from").is_none_or(Value::is_null))
            .map(|c| json!({"key": c["key"], "value": c["from"], "from": c["value"]}))
            .collect();
        std::fs::write(
            dir.join("undo.json"),
            serde_json::to_string_pretty(&json!({"schema": plan["schema"], "changes": undo}))?
                + "\n",
        )?;
    }
    if !dry && !no_backup {
        let mut copied: Vec<PathBuf> = Vec::new();
        for change in changes {
            let key = change.get("key").and_then(Value::as_str).unwrap_or("");
            if let Some(file) = backup_file(key, &home) {
                if file.is_file() && !copied.contains(&file) {
                    let _ = std::fs::copy(&file, dir.join(file.file_name().unwrap_or_default()));
                    copied.push(file);
                }
            }
        }
    }
    let mut results: Vec<Value> = Vec::new();
    let mut status = 0;
    let total = changes.len();
    for (n, change) in changes.iter().enumerate() {
        let key = change
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let value = change.get("value").cloned().unwrap_or(Value::Null);
        if progress && !dry {
            writeln!(stdout, "progress\t{}\t{total}\t{key}", n + 1)?;
        }
        if dry {
            let shown = value.to_string();
            let shown = if shown.chars().count() > 400 {
                format!("{}…", shown.chars().take(400).collect::<String>())
            } else {
                shown
            };
            writeln!(stdout, "{key}\t{shown}")?;
            results.push(json!({"key": key, "status": "dry-run"}));
            continue;
        }
        let current = crate::read_domain(Backend::Omarchy, None, &key);
        if matches!(&current, Ok(c) if !c.is_null() && c == &value && !value.is_array() && !value.is_object())
        {
            results.push(json!({"key": key, "status": "skipped"}));
            continue;
        }
        match crate::write_domain(Backend::Omarchy, None, &key, &value) {
            Ok(()) => results.push(json!({"key": key, "status": "applied"})),
            Err(err) => {
                status = 1;
                writeln!(stderr, "settings-apply: {key} failed: {}", err.message)?;
                results.push(json!({"key": key, "status": "failed", "error": err.message}));
            }
        }
    }
    if !dry {
        let backup = if no_backup {
            String::new()
        } else {
            dir.display().to_string()
        };
        writeln!(
            stdout,
            "{}",
            serde_json::to_string(&json!({"backup": backup, "results": results}))?
        )?;
    }
    Ok(status)
}
