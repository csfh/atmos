//! User hook scripts under ~/.config/omarchy/hooks.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const TYPES: &[&str] = &[
    "theme-set",
    "font-set",
    "post-boot",
    "post-update",
    "pre-refresh-pacman",
    "battery-low",
];

fn hook_id_ok(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn sorted_children(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    paths.sort();
    paths
}

fn hook_types(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = TYPES.iter().map(|t| t.to_string()).collect();
    if !root.is_dir() {
        return found;
    }
    for child in sorted_children(root) {
        let name = child
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let id = if child.is_dir() && name.ends_with(".d") {
            name[..name.len() - 2].to_string()
        } else if child.is_file() && !name.ends_with(".sample") {
            name
        } else {
            continue;
        };
        if found.contains(&id) || !hook_id_ok(&id) {
            continue;
        }
        found.push(id);
    }
    found
}

pub fn collect() -> Value {
    let root = Path::new(&std::env::var("HOME").unwrap_or_default()).join(".config/omarchy/hooks");
    let mut items = Vec::new();
    for kind in hook_types(&root) {
        let flat = root.join(&kind);
        if flat.is_file() {
            items.push(json!({
                "type": kind, "name": kind, "path": flat.to_string_lossy(),
                "sample": false, "flat": true,
            }));
        }
        let folder = root.join(format!("{kind}.d"));
        if !folder.is_dir() {
            continue;
        }
        for path in sorted_children(&folder) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !path.is_file() || name.contains('/') || name.contains("..") {
                continue;
            }
            items.push(json!({
                "type": kind, "name": name, "path": path.to_string_lossy(),
                "sample": name.ends_with(".sample"), "flat": false,
            }));
        }
    }
    Value::Array(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_ids_follow_the_old_pattern() {
        assert!(hook_id_ok("post-boot"));
        assert!(hook_id_ok("9lives"));
        assert!(!hook_id_ok("-x"));
        assert!(!hook_id_ok("Up"));
        assert!(!hook_id_ok(""));
    }
}
