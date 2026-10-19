//! The small shell-like helpers the snapshot reads the machine with: run a
//! tool and take its text, test a tool's exit status, find a tool on PATH.

use crate::runner::Run;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(30);

/// A tool's whole result, for callers that must tell a failure from empty output.
pub fn capture(program: &str, args: &[&str]) -> crate::error::Result<crate::runner::Output> {
    Run::new(program).args(args).timeout(TIMEOUT).output()
}

/// `$(cmd 2>/dev/null || true)`: stdout whatever the exit code, trailing newlines gone.
pub fn out(program: &str, args: &[&str]) -> String {
    match Run::new(program).args(args).timeout(TIMEOUT).output() {
        Ok(o) => o.stdout_text().trim_end_matches('\n').to_string(),
        Err(_) => String::new(),
    }
}

/// Like `out` with a tighter limit, for tools that can hang (`timeout 2s ...`).
pub fn out_within(program: &str, args: &[&str], secs: u64) -> String {
    match Run::new(program)
        .args(args)
        .timeout(Duration::from_secs(secs))
        .output()
    {
        Ok(o) => o.stdout_text().trim_end_matches('\n').to_string(),
        Err(_) => String::new(),
    }
}

/// `$(cmd 2>/dev/null)` that only counts when the command exits 0.
pub fn out_ok(program: &str, args: &[&str]) -> Option<String> {
    let o = Run::new(program)
        .args(args)
        .timeout(TIMEOUT)
        .output()
        .ok()?;
    o.success()
        .then(|| o.stdout_text().trim_end_matches('\n').to_string())
}

pub fn ok(program: &str, args: &[&str]) -> bool {
    Run::new(program)
        .args(args)
        .timeout(TIMEOUT)
        .output()
        .is_ok_and(|o| o.success())
}

pub fn omarchy(args: &[&str]) -> String {
    out("omarchy", args)
}

/// `command -v name`
pub fn present(name: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if name.contains('/') {
        return Path::new(name).is_file();
    }
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| {
            let path = Path::new(if dir.is_empty() { "." } else { dir }).join(name);
            path.metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

pub fn lines(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| Value::String(l.into()))
        .collect()
}

pub fn file_has(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|t| t.lines().any(|l| l.contains(needle)))
}

pub fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// First line of a file with the trailing newline cut, or "".
pub fn first_line(path: &Path) -> String {
    read(path).lines().next().unwrap_or("").to_string()
}

/// `^[0-9]+$`
pub fn is_uint(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// `[[ $x =~ ^[a-zA-Z0-9._-]+$ ]]`
pub fn is_ident(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

/// What `jq -r` prints for an unsigned integer, or None when the value is not one.
pub fn jq_uint(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(n) => {
            let f = n.as_f64()?;
            (f >= 0.0 && f.fract() == 0.0 && f < 1.8e19).then_some(f as u64)
        }
        Value::String(s) => is_uint(s).then(|| s.parse().ok()).flatten(),
        _ => None,
    }
}

/// jq's `x // default`: null and false fall through.
pub fn alt(value: Option<&Value>) -> Option<&Value> {
    value.filter(|v| !v.is_null() && **v != Value::Bool(false))
}

/// What `jq -r` prints for a scalar.
pub fn jq_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}
