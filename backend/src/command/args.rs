//! Checking a write's arguments: a value that is not what it says is refused
//! before anything runs.

use serde_json::Value;

use crate::error::{Error, Result};

pub(super) fn bad(field: &str, why: &str) -> Error {
    Error::bad_request(format!("{field}: {why}")).with_context(field.to_string())
}

/// A string argument that cannot be read as an option or smuggle a line.
pub(super) fn word<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
    let text = args
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(field, "a string is required"))?;
    if text.is_empty() || text.len() > 1024 {
        return Err(bad(field, "empty or too long"));
    }
    if text.starts_with('-') || text.chars().any(|c| c == '\0' || c == '\n' || c == '\r') {
        return Err(bad(field, "not a plain value"));
    }
    Ok(text)
}

pub(super) fn flag(args: &Value, field: &str) -> Result<bool> {
    args.get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| bad(field, "true or false is required"))
}

pub(super) fn uint(args: &Value, field: &str, min: u64, max: u64) -> Result<u64> {
    let n = args
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| bad(field, "a whole number is required"))?;
    if n < min || n > max {
        return Err(bad(field, &format!("between {min} and {max}")));
    }
    Ok(n)
}

pub(super) fn one_of<'a>(args: &'a Value, field: &str, allowed: &[&str]) -> Result<&'a str> {
    let text = word(args, field)?;
    if allowed.contains(&text) {
        Ok(text)
    } else {
        Err(bad(field, &format!("one of {}", allowed.join(", "))))
    }
}

pub(super) fn plain_chars(field: &str, text: &str, extra: &str) -> Result<()> {
    if text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || extra.contains(c))
    {
        Ok(())
    } else {
        Err(bad(field, "unexpected characters"))
    }
}

pub(super) fn address(args: &Value) -> Result<&str> {
    let text = word(args, "address")?;
    let octets: Vec<&str> = text.split(':').collect();
    if octets.len() == 6
        && octets
            .iter()
            .all(|o| o.len() == 2 && o.chars().all(|c| c.is_ascii_hexdigit()))
    {
        Ok(text)
    } else {
        Err(bad("address", "not a device address"))
    }
}

pub(super) fn uuid(args: &Value) -> Result<&str> {
    let text = word(args, "uuid")?;
    if text.len() == 36 && text.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        Ok(text)
    } else {
        Err(bad("uuid", "not a connection id"))
    }
}

/// A path under the Omarchy hooks directory, which is all a hook write may touch.
pub(super) fn hook_path<'a>(args: &'a Value, field: &str) -> Result<&'a str> {
    let text = word(args, field)?;
    let root = format!(
        "{}/.config/omarchy/hooks/",
        std::env::var("HOME").unwrap_or_default()
    );
    if text.starts_with(&root) && !text.split('/').any(|part| part == "..") {
        Ok(text)
    } else {
        Err(bad(field, "not a hook file"))
    }
}

pub(super) fn hook_type(args: &Value) -> Result<&str> {
    let text = word(args, "type")?;
    plain_chars("type", text, "-")?;
    Ok(text)
}

pub(super) fn s(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| p.to_string()).collect()
}

/// A `ratmos task` command line, run by this same binary.
pub(super) fn task(name: &str, rest: &[&str]) -> Vec<String> {
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "ratmos".into());
    let mut argv = vec![exe, "task".into(), name.into()];
    argv.extend(rest.iter().map(|p| p.to_string()));
    argv
}
