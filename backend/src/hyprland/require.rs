//! The `require("hypr.atmos")` lines in hyprland.lua and the layout wrapper file.

use crate::error::Result;
use std::path::Path;

const REQUIRE_LINE: &str = r#"require("hypr.atmos")"#;
const LAYOUT_REQUIRE: &str = r#"require("hypr.atmos_layout")"#;
const OMARCHY_LINE: &str = r#"require("default.hypr.omarchy")"#;
const TOGGLES_LINE: &str = r#"require("default.hypr.toggles")"#;
const LAYOUT_LUA: &str = include_str!("../../../packaging/hypr-atmos-layout.lua");

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Apply,
    Enable,
    Reset,
}

fn ensure_atmos_require(text: &str) -> String {
    if text.contains("hypr.atmos") || text.trim().is_empty() {
        return text.to_string();
    }
    match text.find(TOGGLES_LINE) {
        Some(at) => format!("{}{REQUIRE_LINE}\n{}", &text[..at], &text[at..]),
        None => format!("{}\n\n{REQUIRE_LINE}\n", text.trim_end()),
    }
}

fn ensure_layout_require(text: &str) -> String {
    if text.contains("hypr.atmos_layout") || text.trim().is_empty() {
        return text.to_string();
    }
    for anchor in [OMARCHY_LINE, REQUIRE_LINE] {
        if let Some(at) = text.find(anchor) {
            return format!("{}{LAYOUT_REQUIRE}\n{}", &text[..at], &text[at..]);
        }
    }
    format!("{}\n\n{LAYOUT_REQUIRE}\n", text.trim_end())
}

fn strip_requires(text: &str) -> String {
    text.split('\n')
        .filter(|line| !matches!(line.trim(), REQUIRE_LINE | LAYOUT_REQUIRE))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Heal, enable or remove the requires. A package install never switches the
/// integration on by itself: only `Enable` adds it.
pub fn run(action: Action, hyprland: &Path, packaged: bool) -> Result<()> {
    if !hyprland.exists() {
        return Ok(());
    }
    crate::store::with_lock(hyprland, || {
        let text = std::fs::read_to_string(hyprland)?;
        if text.trim().is_empty() {
            return Ok(());
        }
        if action == Action::Apply && packaged && !text.contains("hypr.atmos") {
            return Ok(());
        }
        if action == Action::Reset {
            let updated = strip_requires(&text);
            if updated != text {
                crate::fsutil::atomic_write_text(hyprland, &updated)?;
            }
            return Ok(());
        }
        let layout = hyprland
            .parent()
            .unwrap_or(Path::new("."))
            .join("atmos_layout.lua");
        if std::fs::read_to_string(&layout).ok().as_deref() != Some(LAYOUT_LUA) {
            crate::fsutil::atomic_write_text(&layout, LAYOUT_LUA)?;
        }
        let updated = ensure_layout_require(&ensure_atmos_require(&text));
        if updated != text {
            crate::fsutil::atomic_write_text(hyprland, &updated)?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_land_before_the_anchors_they_follow_in_omarchy() {
        let text = "require(\"default.hypr.omarchy\")\nrequire(\"default.hypr.toggles\")\n";
        let out = ensure_layout_require(&ensure_atmos_require(text));
        assert_eq!(
            out,
            "require(\"hypr.atmos_layout\")\nrequire(\"default.hypr.omarchy\")\nrequire(\"hypr.atmos\")\nrequire(\"default.hypr.toggles\")\n"
        );
        assert_eq!(strip_requires(&out), text);
    }
}
