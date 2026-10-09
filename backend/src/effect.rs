//! What a settings domain does when it is written, besides patching its file.
//! The effect lives on the domain's row in `domain::SPECS`; this module defines
//! the types and runs the command form.
//!
//! Omarchy `settings.set` dispatches on the effect. `Command` renders the
//! Settings.js argv, including the on/off flag when that writer takes one.
//! Under `--root` the rendered argv is appended to `commands.log` and the
//! platform file is left alone. A live set spawns the same argv. Live reads
//! of those keys stay null so `snapshot.sh` keeps the toggle status.

use crate::error::{Error, Result};
use crate::runner::{Run, COMMAND_TIMEOUT};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Patch the platform file this domain already uses.
    Document,
    /// Hyprland sentinel. Apply keeps rows whose `managed` flag is not false.
    Sentinel { kind: &'static str },
    /// Settings.js argv. Fixture logs the rendered argv. Live spawns it.
    Command(CommandForm),
}

/// How a command row turns the written value into argv.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandForm {
    /// The argv is the same for both bools.
    Exact(&'static [&'static str]),
    /// Settings.js `onOff`. The flag is `on` when the bool differs from `invert`.
    OnOff {
        prefix: &'static [&'static str],
        invert: bool,
    },
    /// `true` appends `stay-awake`. `false` appends `allow-idle`.
    Idle(&'static [&'static str]),
}

/// The effect a domain has, from its row in `domain::SPECS`.
pub fn get(key: &str) -> Option<Effect> {
    crate::domain::find(key).map(|spec| spec.effect)
}

pub fn command_argv(form: CommandForm, value: &Value) -> Vec<&'static str> {
    match form {
        CommandForm::Exact(argv) => argv.to_vec(),
        CommandForm::OnOff { prefix, invert } => {
            let flag = if (value.as_bool() == Some(true)) ^ invert {
                "on"
            } else {
                "off"
            };
            let mut argv = prefix.to_vec();
            argv.push(flag);
            argv
        }
        CommandForm::Idle(prefix) => {
            let flag = if value.as_bool() == Some(true) {
                "stay-awake"
            } else {
                "allow-idle"
            };
            let mut argv = prefix.to_vec();
            argv.push(flag);
            argv
        }
    }
}

pub fn apply_command(
    root: Option<&Path>,
    key: &str,
    form: CommandForm,
    value: &Value,
) -> Result<()> {
    let argv = command_argv(form, value);
    if argv.is_empty() {
        return Err(format!("{key} command is empty").into());
    }
    if let Some(dir) = root {
        return append_command_log(dir, key, &argv, value);
    }
    Run::new(argv[0])
        .args(&argv[1..])
        .timeout(COMMAND_TIMEOUT)
        .checked()
        .map(|_| ())
}

/// Fixture reads return the value recorded with the argv. Live reads stay null
/// so a file or this log cannot cover `snapshot.sh`.
pub fn read_command(root: Option<&Path>, key: &str) -> Result<Value> {
    let Some(dir) = root else {
        return Ok(Value::Null);
    };
    let path = dir.join("commands.log");
    if !path.is_file() {
        return Ok(Value::Null);
    }
    let text = std::fs::read_to_string(&path)?;
    let mut found = Value::Null;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: Value = serde_json::from_str(trimmed)?;
        if parsed.get("domain").and_then(Value::as_str) == Some(key) {
            found = parsed.get("value").cloned().unwrap_or(Value::Null);
        }
    }
    Ok(found)
}

fn append_command_log(dir: &Path, key: &str, argv: &[&str], value: &Value) -> Result<()> {
    let line = serde_json::json!({
        "domain": key,
        "argv": argv,
        "value": value,
    });
    let text = serde_json::to_string(&line)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("commands.log"))?;
    writeln!(file, "{text}").map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const PREFIX: &[&str] = &["omarchy", "toggle", "thing"];

    #[test]
    fn exact_ignores_the_value() {
        let form = CommandForm::Exact(PREFIX);
        assert_eq!(command_argv(form, &json!(true)), PREFIX);
        assert_eq!(command_argv(form, &json!(false)), PREFIX);
        assert_eq!(command_argv(form, &Value::Null), PREFIX);
    }

    #[test]
    fn on_off_follows_the_bool() {
        let form = CommandForm::OnOff {
            prefix: PREFIX,
            invert: false,
        };
        assert_eq!(command_argv(form, &json!(true)).last(), Some(&"on"));
        assert_eq!(command_argv(form, &json!(false)).last(), Some(&"off"));
        assert_eq!(command_argv(form, &json!(true))[..3], *PREFIX);
    }

    #[test]
    fn on_off_inverts_when_asked() {
        let form = CommandForm::OnOff {
            prefix: PREFIX,
            invert: true,
        };
        assert_eq!(command_argv(form, &json!(true)).last(), Some(&"off"));
        assert_eq!(command_argv(form, &json!(false)).last(), Some(&"on"));
    }

    #[test]
    fn a_non_bool_counts_as_false() {
        let plain = CommandForm::OnOff {
            prefix: PREFIX,
            invert: false,
        };
        let inverted = CommandForm::OnOff {
            prefix: PREFIX,
            invert: true,
        };
        for value in [json!("true"), json!(1), Value::Null] {
            assert_eq!(command_argv(plain, &value).last(), Some(&"off"), "{value}");
            assert_eq!(
                command_argv(inverted, &value).last(),
                Some(&"on"),
                "{value}"
            );
        }
    }

    #[test]
    fn idle_maps_the_bool_to_the_two_words() {
        let form = CommandForm::Idle(PREFIX);
        assert_eq!(command_argv(form, &json!(true)).last(), Some(&"stay-awake"));
        assert_eq!(
            command_argv(form, &json!(false)).last(),
            Some(&"allow-idle")
        );
        assert_eq!(command_argv(form, &Value::Null).last(), Some(&"allow-idle"));
    }

    #[test]
    fn get_reads_the_effect_off_the_spec_row() {
        assert_eq!(get("theme"), Some(Effect::Document));
        assert_eq!(get("no.such.key"), None);
    }

    #[test]
    fn every_command_row_renders_a_nonempty_argv_for_both_bools() {
        for spec in crate::domain::specs() {
            if let Effect::Command(form) = spec.effect {
                for value in [json!(true), json!(false)] {
                    let argv = command_argv(form, &value);
                    assert!(!argv.is_empty(), "{}", spec.key);
                    assert!(argv.iter().all(|word| !word.is_empty()), "{}", spec.key);
                }
            }
        }
    }
}
