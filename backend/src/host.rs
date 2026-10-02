//! Host reads Quickshell used to do itself. Fixture roots stay off the live machine.

use crate::error::{Error, Kind, Result};
use crate::fsutil::atomic_write;
use crate::platform::Backend;
use crate::request::{Accounts, Paths, SpeedDisk, SpeedNet, ThemePack, UnitOutput, WriteFile};
use crate::runner::Run;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use serde_json::{json, Map, Value};

const TEXT_CAP: usize = 1_000_000;
const WALK_CAP: usize = 4000;

pub fn chrome(backend: Backend, root: Option<&Path>) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({
            "themeName": "plain",
            "colors": "plain-colors",
            "themeShell": "plain-shell",
            "userShell": "plain-user-shell",
        }));
    }
    Ok(json!({
        "themeName": read_capped(&host_path(root, ".local/state/omarchy/current/theme.name"))?,
        "colors": read_capped(&host_path(root, ".local/state/omarchy/current/theme/colors.toml"))?,
        "themeShell": read_capped(&host_path(root, ".local/state/omarchy/current/theme/shell.toml"))?,
        "userShell": read_capped(&host_path(root, ".config/omarchy/shell.toml"))?,
    }))
}

pub fn theme_pack(backend: Backend, root: Option<&Path>, request: &ThemePack) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({
            "colors": "plain-theme-colors",
            "shell": "plain-theme-shell",
        }));
    }
    let slug = slug(&request.name);
    let home = request.home.as_str();
    let part = request.part.as_str();
    let colors = if part == "shell.toml" {
        String::new()
    } else {
        first_text(root, &theme_candidates(home, &slug, "colors.toml"))?
    };
    let shell = if part == "colors.toml" {
        String::new()
    } else {
        first_text(root, &theme_candidates(home, &slug, "shell.toml"))?
    };
    Ok(json!({ "colors": colors, "shell": shell }))
}

pub fn accounts(backend: Backend, root: Option<&Path>, request: &Accounts) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({
            "hostname": "plain-host",
            "passwd": "plain:x:1000:1000:Plain:/home/plain:/bin/sh\n",
            "group": "plain:x:1000:\n",
            "exists": {},
        }));
    }
    let user = request.user.as_str();
    let home = request.home.as_str();
    Ok(json!({
        "hostname": read_capped(&host_path(root, "/etc/hostname"))?,
        "passwd": read_capped(&host_path(root, "/etc/passwd"))?,
        "group": read_capped(&host_path(root, "/etc/group"))?,
        "exists": exists_map(root, user, home)?,
    }))
}

pub fn stamp(root: Option<&Path>, request: &Paths) -> Result<Value> {
    let paths = check_paths(&request.paths)?;
    let mut items = Vec::new();
    for path in paths {
        let resolved = host_path(root, &path);
        let sig = signature(&resolved);
        let text = if resolved.is_file() {
            read_capped(&resolved)?
        } else {
            String::new()
        };
        items.push(json!({
            "path": path,
            "sig": sig,
            "text": text,
        }));
    }
    Ok(json!({ "items": items }))
}

pub fn read_files(root: Option<&Path>, request: &Paths) -> Result<Value> {
    let paths = check_paths(&request.paths)?;
    let mut files = Vec::new();
    for path in paths {
        let resolved = confine(root, &path)?;
        files.push(json!({
            "path": path,
            "exists": resolved.is_file(),
            "text": read_capped(&resolved)?,
        }));
    }
    Ok(json!({ "files": files }))
}

pub fn write_file(root: Option<&Path>, request: &WriteFile) -> Result<Value> {
    let resolved = confine(root, &request.path)?;
    atomic_write(&resolved, request.text.as_bytes())?;
    Ok(json!({ "path": request.path, "bytes": request.text.len() }))
}

pub fn open_file(root: Option<&Path>, path: &str) -> Result<Value> {
    let web = path.starts_with("http://") || path.starts_with("https://");
    let target = if web {
        path.to_string()
    } else {
        let resolved = confine(root, path)?;
        if !resolved.exists() {
            return Err(Error::new(Kind::NotFound, "nothing to open").with_context(path));
        }
        resolved.display().to_string()
    };
    if root.is_some() {
        return Ok(json!({ "opened": path }));
    }
    // The opener may stay up as long as the app it launches, so do not wait.
    Run::new("xdg-open").arg(&target).detach()?;
    Ok(json!({ "opened": path }))
}

pub fn speed_disk(backend: Backend, root: Option<&Path>, request: &SpeedDisk) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({ "exit": 0, "stdout": "plain-disk\n", "stderr": "" }));
    }
    if root.is_some() {
        let text = read_capped(&host_path(root, ".local/state/omarchy/speed/disk.txt"))?;
        return Ok(json!({ "exit": 0, "stdout": text, "stderr": "" }));
    }
    let dir = request.dir.as_str();
    let mut argv = vec!["disk", "speedtest"];
    if !dir.is_empty() {
        argv.push(dir);
    }
    // A disk test writes and reads a real file, so it gets room to finish.
    run_omarchy(&argv, Duration::from_secs(300))
}

pub fn speed_net(backend: Backend, root: Option<&Path>, request: &SpeedNet) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({ "exit": 0, "stdout": "plain-mbps\n", "stderr": "" }));
    }
    let phase = request.phase.as_str();
    if phase != "down" && phase != "up" {
        return Err("speedtest phase must be down or up".into());
    }
    if root.is_some() {
        let rel = format!(".local/state/omarchy/speed/net-{phase}.txt");
        let text = read_capped(&host_path(root, &rel))?;
        return Ok(json!({ "exit": 0, "stdout": text, "stderr": "" }));
    }
    run_omarchy(&["network", "speedtest", phase], Duration::from_secs(90))
}

pub fn unit_output(backend: Backend, root: Option<&Path>, request: &UnitOutput) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({ "exit": 0, "text": "plain-unit\n" }));
    }
    let kind = request.kind.as_str();
    let scope = request.scope.as_str();
    let unit = request.unit.as_str();
    if kind != "status" && kind != "logs" {
        return Err("unit kind must be status or logs".into());
    }
    if scope != "system" && scope != "user" {
        return Err("unit scope must be system or user".into());
    }
    if !unit_name_ok(unit) {
        return Err("unit name is not safe to pass through".into());
    }
    if root.is_some() {
        let rel = format!(".local/state/omarchy/units/{kind}-{unit}.txt");
        let text = read_capped(&host_path(root, &rel))?;
        return Ok(json!({ "exit": 0, "text": text }));
    }
    let mut run = if kind == "status" {
        let mut run = Run::new("systemctl");
        if scope == "user" {
            run = run.arg("--user");
        }
        run.args(["--no-pager", "--full", "status", unit])
    } else {
        let mut run = Run::new("journalctl");
        run = run.arg(if scope == "user" {
            "--user"
        } else {
            "--system"
        });
        run.args(["-u", unit, "-n", "80", "--no-pager"])
    };
    run = run.timeout(Duration::from_secs(15));
    let output = run.output()?;
    let mut text = output.stdout_text();
    if text.trim().is_empty() {
        text = output.stderr_text();
    }
    Ok(json!({
        "exit": output.code,
        "text": text,
    }))
}

fn theme_candidates(home: &str, slug: &str, rel: &str) -> Vec<String> {
    let mut paths = Vec::new();
    if !home.is_empty() && !slug.is_empty() {
        paths.push(format!("{home}/.config/omarchy/themes/{slug}/{rel}"));
    }
    if !slug.is_empty() {
        paths.push(format!("/usr/share/omarchy/themes/{slug}/{rel}"));
    }
    paths
}

fn first_text(root: Option<&Path>, paths: &[String]) -> Result<String> {
    for path in paths {
        let resolved = host_path(root, path);
        if resolved.is_file() {
            return read_capped(&resolved);
        }
    }
    Ok(String::new())
}

fn exists_map(root: Option<&Path>, user: &str, home: &str) -> Result<Value> {
    let mut map = Map::new();
    let mut paths = Vec::new();
    if !user.is_empty() && !user.contains('/') {
        paths.push(format!("/var/lib/AccountsService/icons/{user}"));
    }
    if !home.is_empty() {
        paths.push(format!("{home}/.face.icon"));
        paths.push(format!("{home}/.face"));
    }
    for path in paths {
        map.insert(path.clone(), Value::Bool(host_path(root, &path).is_file()));
    }
    Ok(Value::Object(map))
}

fn host_path(root: Option<&Path>, raw: &str) -> PathBuf {
    let raw = raw.trim();
    if let Some(root) = root {
        return root.join(raw.trim_start_matches('/'));
    }
    if raw.starts_with('/') {
        return PathBuf::from(raw);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));
    home.join(raw)
}

fn read_capped(path: &Path) -> Result<String> {
    if !path.is_file() {
        return Ok(String::new());
    }
    // Four bytes per character is the most a UTF-8 character takes, so this
    // reads enough for TEXT_CAP characters and no more of a huge file.
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take((TEXT_CAP as u64) * 4)
        .read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    if text.chars().count() > TEXT_CAP {
        return Ok(text.chars().take(TEXT_CAP).collect());
    }
    Ok(text.into_owned())
}

/// Where a user-chosen path may point. Live, that is anywhere under $HOME;
/// in a fixture, anywhere under the root. A `..` step is refused outright,
/// and the deepest existing ancestor must resolve (through symlinks) inside
/// the allowed tree, so a link in $HOME cannot reach /etc.
pub fn confine(root: Option<&Path>, raw: &str) -> Result<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() || raw.contains('\0') {
        return Err(Error::bad_request("empty path").with_context("path"));
    }
    let base = match root {
        Some(root) => root.to_path_buf(),
        None => std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|home| home.is_absolute())
            .ok_or_else(|| Error::denied("HOME is not set").with_context(raw))?,
    };
    let given = Path::new(raw);
    if given
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(Error::denied("path may not contain ..").with_context(raw));
    }
    let candidate = match root {
        Some(root) => root.join(raw.trim_start_matches('/')),
        None => {
            if let Some(rest) = raw.strip_prefix("~/") {
                base.join(rest)
            } else if given.is_absolute() {
                given.to_path_buf()
            } else {
                base.join(given)
            }
        }
    };
    let allowed = fs::canonicalize(&base).unwrap_or(base);
    let mut probe = candidate.as_path();
    let existing = loop {
        if probe.exists() {
            break fs::canonicalize(probe)?;
        }
        match probe.parent() {
            Some(parent) => probe = parent,
            None => break PathBuf::new(),
        }
    };
    if !existing.starts_with(&allowed) {
        return Err(Error::denied(match root {
            Some(_) => "path is outside the root",
            None => "path is outside your home folder",
        })
        .with_context(raw));
    }
    Ok(candidate)
}

fn signature(path: &Path) -> String {
    if !path.exists() {
        return "missing".into();
    }
    if path.is_file() {
        return file_sig(path);
    }
    if path.is_dir() {
        let mut lines = Vec::new();
        walk(path, path, 0, &mut lines);
        lines.sort();
        return lines.join("\n");
    }
    "other".into()
}

fn walk(root: &Path, dir: &Path, depth: usize, lines: &mut Vec<String>) {
    if depth > 6 || lines.len() >= WALK_CAP {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if lines.len() >= WALK_CAP {
            return;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if path.is_dir() {
            lines.push(format!("d:{rel}:{}", file_sig(&path)));
            walk(root, &path, depth + 1, lines);
        } else {
            lines.push(format!("f:{rel}:{}", file_sig(&path)));
        }
    }
}

fn file_sig(path: &Path) -> String {
    match path.metadata() {
        Ok(meta) => {
            let modified = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|time| time.as_secs())
                .unwrap_or(0);
            format!("{}:{modified}", meta.len())
        }
        Err(_) => "err".into(),
    }
}

fn check_paths(list: &[String]) -> Result<Vec<String>> {
    for text in list {
        if text.is_empty() || text.contains('\0') {
            return Err(Error::bad_request("paths has an empty path").with_context("paths"));
        }
    }
    Ok(list.to_vec())
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn unit_name_ok(unit: &str) -> bool {
    !unit.is_empty()
        && unit.len() < 240
        && unit.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '.' | '@' | '-' | '\\')
        })
}

fn run_omarchy(args: &[&str], limit: Duration) -> Result<Value> {
    let output = Run::new("omarchy").args(args).timeout(limit).output()?;
    Ok(json!({
        "exit": output.code,
        "stdout": output.stdout_text(),
        "stderr": output.stderr_text(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ratmos-confine-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_fixture_path_stays_under_the_root() {
        let root = scratch("root");
        let ok = confine(Some(&root), "/exports/a.md").unwrap();
        assert_eq!(ok, root.join("exports/a.md"));
        assert!(confine(Some(&root), "../escape").unwrap_err().is_denied());
        assert!(confine(Some(&root), "a/../../b").unwrap_err().is_denied());
        assert!(confine(Some(&root), "  ").is_err());
    }

    #[test]
    fn a_symlink_cannot_lead_out_of_the_root() {
        let root = scratch("link");
        let outside = scratch("outside");
        std::os::unix::fs::symlink(&outside, root.join("door")).unwrap();
        let err = confine(Some(&root), "door/secret.txt").unwrap_err();
        assert!(err.is_denied(), "{err}");
    }

    #[test]
    fn a_long_file_is_cut_at_the_cap() {
        let dir = scratch("cap");
        let file = dir.join("big.txt");
        fs::write(&file, "é".repeat(TEXT_CAP + 10)).unwrap();
        assert_eq!(read_capped(&file).unwrap().chars().count(), TEXT_CAP);
    }
}
