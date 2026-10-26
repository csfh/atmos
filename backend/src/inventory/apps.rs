//! User-installed desktop, TUI, and web launchers from `~/.local/share/applications`.

use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn collect() -> Value {
    let home = std::env::var("HOME").unwrap_or_default();
    let data = match std::env::var("XDG_DATA_HOME") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(home).join(".local/share"),
    };
    from_dir(&data.join("applications"))
}

pub fn from_dir(root: &Path) -> Value {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut buckets: BTreeMap<&str, Vec<(String, Value)>> = BTreeMap::new();
    for kind in ["desktop", "tui", "web"] {
        buckets.insert(kind, Vec::new());
    }
    let mut seen = HashSet::new();
    for path in files {
        let entry = parse_desktop(&path);
        if truthy(entry.get("Hidden")) || truthy(entry.get("NoDisplay")) {
            continue;
        }
        let kind_type = entry
            .get("Type")
            .map(String::as_str)
            .filter(|t| !t.is_empty());
        if kind_type.unwrap_or("Application") != "Application" {
            continue;
        }
        let exec = entry.get("Exec").cloned().unwrap_or_default();
        if exec.is_empty() {
            continue;
        }
        let kind = classify(&exec);
        let id = file_id(&path);
        if !seen.insert((kind, id.clone())) {
            continue;
        }
        let name = entry
            .get("Name")
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| id.clone());
        let detail = match kind {
            "web" => web_url(&exec),
            "tui" => tui_command(&exec),
            _ => String::new(),
        };
        let sort = name.to_lowercase();
        buckets
            .get_mut(kind)
            .expect("bucket")
            .push((sort, json!({ "id": id, "name": name, "detail": detail })));
    }
    let mut out = Map::new();
    for (kind, mut items) in buckets {
        items.sort_by(|a, b| a.0.cmp(&b.0));
        out.insert(
            kind.into(),
            Value::Array(items.into_iter().map(|(_, item)| item).collect()),
        );
    }
    Value::Object(out)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "desktop") {
            out.push(path);
        }
    }
}

fn parse_desktop(path: &Path) -> BTreeMap<String, String> {
    let mut entry = BTreeMap::new();
    let Ok(raw) = fs::read(path) else {
        return entry;
    };
    let text = String::from_utf8_lossy(&raw);
    let mut in_entry = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_entry = &line[1..line.len() - 1] == "Desktop Entry";
            continue;
        }
        if !in_entry {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            entry
                .entry(key.to_string())
                .or_insert_with(|| value.to_string());
        }
    }
    entry
}

fn truthy(value: Option<&String>) -> bool {
    matches!(
        value.map(|v| v.trim().to_lowercase()).as_deref(),
        Some("true" | "1" | "yes")
    )
}

fn is_space(c: char) -> bool {
    c.is_whitespace()
}

fn classify(exec: &str) -> &'static str {
    if exec.contains("omarchy-launch-webapp") || exec.contains("omarchy-webapp-handler") {
        return "web";
    }
    if is_tui(exec) {
        "tui"
    } else {
        "desktop"
    }
}

/// `(^|[\s=])(\$TERMINAL|xdg-terminal-exec)(\s|$).*-e(\s|$)`
fn is_tui(exec: &str) -> bool {
    for term in ["$TERMINAL", "xdg-terminal-exec"] {
        for (at, _) in exec.match_indices(term) {
            let before = exec[..at].chars().next_back();
            if !before.is_none_or(|c| is_space(c) || c == '=') {
                continue;
            }
            let after = &exec[at + term.len()..];
            let rest = match after.chars().next() {
                None => "",
                Some(c) if is_space(c) => &after[c.len_utf8()..],
                Some(_) => continue,
            };
            if has_dash_e(rest) {
                return true;
            }
        }
    }
    false
}

fn has_dash_e(text: &str) -> bool {
    text.match_indices("-e")
        .any(|(at, _)| text[at + 2..].chars().next().is_none_or(is_space))
}

fn web_url(exec: &str) -> String {
    exec.split_whitespace()
        .find(|part| part.starts_with("http://") || part.starts_with("https://"))
        .unwrap_or("")
        .to_string()
}

/// `(?:^|\s)-e\s+(.+)$`
fn tui_command(exec: &str) -> String {
    for (at, _) in exec.match_indices("-e") {
        if !exec[..at].chars().next_back().is_none_or(is_space) {
            continue;
        }
        let rest = &exec[at + 2..];
        let trimmed = rest.trim_start_matches(is_space);
        if trimmed.len() < rest.len() && !trimmed.is_empty() {
            return trimmed.trim().to_string();
        }
    }
    String::new()
}

fn file_id(path: &Path) -> String {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    unquote(name.strip_suffix(".desktop").unwrap_or(name))
}

fn unquote(text: &str) -> String {
    let bytes = text.as_bytes();
    let hex = |b: u8| (b as char).to_digit(16);
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launchers_are_classified_like_the_script_did() {
        assert_eq!(classify("omarchy-launch-webapp https://x.com"), "web");
        assert_eq!(classify("xdg-terminal-exec -e btop"), "tui");
        assert_eq!(classify("env TERM=$TERMINAL -e htop"), "tui");
        assert_eq!(classify("x$TERMINAL -e htop"), "desktop");
        assert_eq!(classify("FOO=$TERMINAL -e htop"), "tui");
        assert_eq!(classify("xdg-terminal-exec"), "desktop");
        assert_eq!(classify("firefox %u"), "desktop");
        assert_eq!(
            tui_command("xdg-terminal-exec -e btop --tree"),
            "btop --tree"
        );
        assert_eq!(tui_command("xdg-terminal-exec -e"), "");
        assert_eq!(
            web_url("omarchy-launch-webapp https://x.com/a"),
            "https://x.com/a"
        );
    }

    #[test]
    fn ids_decode_percent_escapes() {
        assert_eq!(unquote("chrome-x%2Ecom"), "chrome-x.com");
        assert_eq!(unquote("100%"), "100%");
        assert_eq!(unquote("a%zzb"), "a%zzb");
    }

    #[test]
    fn a_directory_is_bucketed_sorted_and_deduplicated() {
        let dir = std::env::temp_dir().join(format!("atmosd-apps-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let write = |name: &str, body: &str| fs::write(dir.join(name), body).unwrap();
        write("b.desktop", "[Desktop Entry]\nName=Beta\nExec=firefox\n");
        write("a.desktop", "[Desktop Entry]\nName=alpha\nExec=firefox\n");
        write(
            "h.desktop",
            "[Desktop Entry]\nName=Hidden\nExec=x\nNoDisplay=true\n",
        );
        write(
            "t.desktop",
            "[Desktop Entry]\nName=Top\nExec=xdg-terminal-exec -e btop\n",
        );
        write("l.desktop", "[Desktop Entry]\nType=Link\nName=L\nExec=x\n");
        let got = from_dir(&dir);
        let names: Vec<_> = got["desktop"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["alpha", "Beta"]);
        assert_eq!(got["tui"][0]["detail"], "btop");
        assert_eq!(got["web"], json!([]));
        fs::remove_dir_all(&dir).unwrap();
    }
}
