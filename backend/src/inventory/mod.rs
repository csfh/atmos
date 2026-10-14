//! The host inventories the Quickshell pages show, collected natively.

use crate::runner::Run;
use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

pub mod accounts;
pub mod apps;
pub mod diagnostics;
pub mod disks;
pub mod hardware;
pub mod hooks;
pub mod live;
pub mod services;
pub mod time;

/// Native collector for a display kind, or None while one still runs a script.
pub fn collect(kind: &str) -> Option<Value> {
    match kind {
        "software" => Some(apps::collect()),
        "disks" => Some(disks::collect()),
        "live" => Some(live::collect()),
        "hardware" => Some(hardware::collect()),
        "diagnostics" => Some(diagnostics::collect()),
        "services" => Some(services::collect()),
        _ => None,
    }
}

/// Stdout of a tool that may be missing or fail; empty then, like a script's `|| true`.
pub(crate) fn capture(program: &str, args: &[&str]) -> String {
    Run::new(program)
        .args(args)
        .timeout(Duration::from_secs(20))
        .checked()
        .map(|out| out.stdout_text())
        .unwrap_or_default()
}

/// A JSON scalar as the text a script's `str()` would give; null is empty.
pub(crate) fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        other => other.to_string(),
    }
}

/// `int(x or 0)` for a number or a numeric string.
pub(crate) fn number(value: &Value) -> i64 {
    match value {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .unwrap_or(0),
        Value::String(s) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// Up to `limit` bytes of a file as text; empty when it cannot be read.
pub(crate) fn read_text(path: &Path, limit: usize) -> String {
    let Ok(file) = std::fs::File::open(path) else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = file.take(limit as u64).read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

pub(crate) fn read_strip(path: &Path, limit: usize) -> String {
    read_text(path, limit).trim().to_string()
}

/// `int(text.split()[0], 0)` of a sysfs file, 0 when absent or not a number.
pub(crate) fn read_int(path: &Path) -> i64 {
    let text = read_strip(path, 512);
    let Some(word) = text.split_whitespace().next() else {
        return 0;
    };
    let (neg, digits) = match word.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, word.strip_prefix('+').unwrap_or(word)),
    };
    let value = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(oct) = digits
        .strip_prefix("0o")
        .or_else(|| digits.strip_prefix("0O"))
    {
        i64::from_str_radix(oct, 8).ok()
    } else if let Some(bin) = digits
        .strip_prefix("0b")
        .or_else(|| digits.strip_prefix("0B"))
    {
        i64::from_str_radix(bin, 2).ok()
    } else if digits.len() > 1 && digits.starts_with('0') {
        digits.chars().all(|c| c == '0').then_some(0)
    } else {
        digits.parse().ok()
    };
    value.map_or(0, |v| if neg { -v } else { v })
}
