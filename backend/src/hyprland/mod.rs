//! Atmos's managed blocks in the Hyprland Lua files: parse them, write them,
//! and keep the `hypr.atmos` requires in hyprland.lua.

pub mod autostart;
pub mod bindings;
pub mod input;
pub mod look;
pub mod lua;
pub mod monitors;
pub mod require;
pub mod windows;
pub mod workspaces;

use crate::error::Result;
use lua::{replace_sentinel, strip_sentinel};
use serde_json::{Map, Value};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Look,
    Input,
    Autostart,
    Bindings,
    Windows,
    Workspaces,
    Monitors,
}

impl Kind {
    pub fn parse(name: &str) -> Option<Kind> {
        Some(match name {
            "look" => Kind::Look,
            "input" => Kind::Input,
            "autostart" => Kind::Autostart,
            "bindings" => Kind::Bindings,
            "windows" => Kind::Windows,
            "workspaces" => Kind::Workspaces,
            "monitors" => Kind::Monitors,
            _ => return None,
        })
    }

    pub fn begin(self) -> &'static str {
        match self {
            Kind::Look => look::BEGIN,
            Kind::Input => input::BEGIN,
            Kind::Autostart => autostart::BEGIN,
            Kind::Bindings => bindings::BEGIN,
            Kind::Windows => windows::BEGIN,
            Kind::Workspaces => workspaces::BEGIN,
            Kind::Monitors => monitors::BEGIN,
        }
    }

    pub fn end(self) -> &'static str {
        match self {
            Kind::Look => look::END,
            Kind::Input => input::END,
            Kind::Autostart => autostart::END,
            Kind::Bindings => bindings::END,
            Kind::Windows => windows::END,
            Kind::Workspaces => workspaces::END,
            Kind::Monitors => monitors::END,
        }
    }
}

/// What `list` prints for a kind: the rows, or the gesture state for input.
pub fn list(kind: Kind, text: &str) -> Value {
    match kind {
        Kind::Bindings => bindings::parse(text),
        Kind::Windows => windows::parse(text),
        Kind::Autostart => autostart::parse(text),
        Kind::Workspaces => workspaces::parse(text),
        Kind::Monitors => monitors::parse(text),
        Kind::Input => input::gesture_state(text),
        Kind::Look => Value::Array(Vec::new()),
    }
}

/// A `_patch` payload's own fields, without the `_`-prefixed bookkeeping keys.
fn split_patch(payload: &Map<String, Value>) -> Option<Map<String, Value>> {
    let patch = payload.get("_patch")?.as_object()?;
    Some(
        patch
            .iter()
            .filter(|(k, _)| !k.starts_with('_'))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}

/// The file text after applying (or resetting) one kind's block.
pub fn apply_text(kind: Kind, text: &str, payload: &Value, reset: bool) -> String {
    let empty = Map::new();
    let payload = payload.as_object().unwrap_or(&empty);
    let mut text = text.to_string();
    if kind == Kind::Windows {
        if text.trim().is_empty() {
            text = format!("{}\n", windows::prefs_seed());
        }
        text = windows::normalize_seed(&text);
    }
    match kind {
        Kind::Look => text = strip_sentinel(&text, look::LEGACY_BEGIN, look::LEGACY_END),
        Kind::Input => text = strip_sentinel(&text, input::LEGACY_BEGIN, input::LEGACY_END),
        _ => {}
    }
    let (begin, end) = (kind.begin(), kind.end());
    if reset {
        return strip_sentinel(&text, begin, end);
    }
    // A _patch payload carries only the fields one write changed. Merge those
    // onto what is on disk right now (under the caller's lock) so two windows
    // editing different fields do not clobber each other. A plain object
    // without _patch is a full managed block.
    let block = match kind {
        Kind::Input => {
            let src = match split_patch(payload) {
                Some(patch) => {
                    let mut base = input::parse(&text);
                    base.extend(patch.clone());
                    if !patch.contains_key("workspaceGesture")
                        && !base.contains_key("workspaceGesture")
                    {
                        base.insert(
                            "workspaceGesture".into(),
                            Value::Bool(input::has_workspace_gesture(&text)),
                        );
                    }
                    base
                }
                None => payload.clone(),
            };
            input::serialize(&src, &text)
        }
        Kind::Look => {
            let src = match split_patch(payload) {
                Some(patch) => {
                    let mut base = look::parse(&text);
                    base.extend(patch);
                    base
                }
                None => payload.clone(),
            };
            look::serialize(&src)
        }
        Kind::Windows => {
            // A write carries the rule list, the opaque flag, or both.
            // Whatever it leaves out stays as it is on disk.
            let mut src = payload.clone();
            if !src.contains_key("items") {
                let managed: Vec<Value> = windows::parse(&text)
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|r| r["managed"] == Value::Bool(true))
                    .cloned()
                    .collect();
                src.insert("items".into(), Value::Array(managed));
            }
            if !src.contains_key(windows::OPAQUE_KEY) {
                src.insert(
                    windows::OPAQUE_KEY.into(),
                    Value::Bool(windows::read_opaque(&text)),
                );
            }
            windows::serialize(&src)
        }
        Kind::Autostart => autostart::serialize(payload),
        Kind::Bindings => bindings::serialize(payload),
        Kind::Workspaces => workspaces::serialize(payload),
        Kind::Monitors => monitors::serialize(payload),
    };
    replace_sentinel(&text, begin, end, &block)
}

fn with_final_newline(text: &str) -> String {
    if text.is_empty() || text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}

fn read_or_empty(path: &Path) -> Result<String> {
    if path.exists() {
        Ok(std::fs::read_to_string(path)?)
    } else {
        Ok(String::new())
    }
}

/// Apply or reset a block in a file, under the file's writer lock.
pub fn apply_file(kind: Kind, path: &Path, payload: &Value, reset: bool) -> Result<()> {
    crate::store::with_lock(path, || {
        let text = read_or_empty(path)?;
        let updated = apply_text(kind, &text, payload, reset);
        if updated != text {
            crate::fsutil::atomic_write_text(path, &with_final_newline(&updated))?;
        }
        Ok(())
    })
}

pub fn list_file(kind: Kind, path: &Path) -> Result<Value> {
    Ok(list(kind, &read_or_empty(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn windows_flag_and_rules_do_not_overwrite_each_other() {
        let rules = json!({"items": [{"match": "^x$", "placement": "float", "center": true}]});
        let base = apply_text(Kind::Windows, "", &rules, false);
        let on = apply_text(Kind::Windows, &base, &json!({"windowOpaque": true}), false);
        assert!(on.contains("-- atmos:windowOpaque = true"));
        assert!(on.contains("o.window(\"^x$\""));
        assert_eq!(
            apply_text(Kind::Windows, &on, &rules, false),
            on,
            "rules keep the flag"
        );
        assert_eq!(
            apply_text(Kind::Windows, &on, &json!({"windowOpaque": false}), false),
            base
        );
        let reset = apply_text(Kind::Windows, &on, &Value::Null, true);
        assert!(!reset.contains("atmos:windows") && !reset.contains("default-opacity"));
        assert!(reset.contains("dev.csfh.atmos"), "the tile seed stays");
    }

    #[test]
    fn a_look_patch_merges_onto_the_block_on_disk() {
        let first = apply_text(
            Kind::Look,
            "",
            &json!({"gapsIn": 7, "dimInactive": true}),
            false,
        );
        let next = apply_text(
            Kind::Look,
            &first,
            &json!({"_patch": {"gapsOut": 20}}),
            false,
        );
        let parsed = look::parse(&next);
        assert_eq!(parsed["gapsIn"], 7);
        assert_eq!(parsed["gapsOut"], 20);
        assert_eq!(parsed["dimInactive"], true);
    }

    #[test]
    fn autostart_round_trips_with_disabled_rows() {
        let rows = json!({"items": [{"command": "foot", "delay": 3}, {"command": "mako", "enabled": false}]});
        let text = apply_text(Kind::Autostart, "", &rows, false);
        let listed = list(Kind::Autostart, &text);
        assert_eq!(listed[0]["command"], "foot");
        assert_eq!(listed[0]["delay"], 3);
        assert_eq!(listed[1]["enabled"], false);
    }
}
