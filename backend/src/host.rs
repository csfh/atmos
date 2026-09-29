//! Host reads Quickshell used to do itself. Fixture roots stay off the live machine.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::UNIX_EPOCH;

use serde_json::{json, Map, Value};

const TEXT_CAP: usize = 1_000_000;
const WALK_CAP: usize = 4000;

pub fn chrome(backend: &str, root: Option<&Path>) -> Result<Value, String> {
    if backend == "plain" {
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

pub fn theme_pack(backend: &str, root: Option<&Path>, request: &Value) -> Result<Value, String> {
    if backend == "plain" {
        return Ok(json!({
            "colors": "plain-theme-colors",
            "shell": "plain-theme-shell",
        }));
    }
    let name = request.get("name").and_then(Value::as_str).unwrap_or("");
    let slug = slug(name);
    let home = request.get("home").and_then(Value::as_str).unwrap_or("");
    let part = request.get("part").and_then(Value::as_str).unwrap_or("");
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

pub fn accounts(backend: &str, root: Option<&Path>, request: &Value) -> Result<Value, String> {
    if backend == "plain" {
        return Ok(json!({
            "hostname": "plain-host",
            "passwd": "plain:x:1000:1000:Plain:/home/plain:/bin/sh\n",
            "group": "plain:x:1000:\n",
            "exists": {},
        }));
    }
    let user = request.get("user").and_then(Value::as_str).unwrap_or("");
    let home = request.get("home").and_then(Value::as_str).unwrap_or("");
    Ok(json!({
        "hostname": read_capped(&host_path(root, "/etc/hostname"))?,
        "passwd": read_capped(&host_path(root, "/etc/passwd"))?,
        "group": read_capped(&host_path(root, "/etc/group"))?,
        "exists": exists_map(root, user, home)?,
    }))
}

pub fn stamp(root: Option<&Path>, request: &Value) -> Result<Value, String> {
    let paths = string_list(request, "paths")?;
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

pub fn read_files(root: Option<&Path>, request: &Value) -> Result<Value, String> {
    let paths = string_list(request, "paths")?;
    let mut files = Vec::new();
    for path in paths {
        let resolved = host_path(root, &path);
        files.push(json!({
            "path": path,
            "exists": resolved.is_file(),
            "text": read_capped(&resolved)?,
        }));
    }
    Ok(json!({ "files": files }))
}

pub fn write_file(root: Option<&Path>, request: &Value) -> Result<Value, String> {
    let path = request
        .get("path")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or("missing path")?;
    let text = request.get("text").and_then(Value::as_str).unwrap_or("");
    let resolved = host_path(root, path);
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    atomic_write(&resolved, text.as_bytes())?;
    Ok(json!({ "path": path, "bytes": text.len() }))
}

pub fn open_file(root: Option<&Path>, request: &Value) -> Result<Value, String> {
    let path = request
        .get("path")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or("missing path")?;
    if root.is_some() {
        return Ok(json!({ "opened": path }));
    }
    let status = Command::new("xdg-open")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| err.to_string())?;
    if !status.success() {
        return Err(format!("xdg-open exited {}", status.code().unwrap_or(1)));
    }
    Ok(json!({ "opened": path }))
}

pub fn speed_disk(backend: &str, root: Option<&Path>, request: &Value) -> Result<Value, String> {
    if backend == "plain" {
        return Ok(json!({ "exit": 0, "stdout": "plain-disk\n", "stderr": "" }));
    }
    if root.is_some() {
        let text = read_capped(&host_path(root, ".local/state/omarchy/speed/disk.txt"))?;
        return Ok(json!({ "exit": 0, "stdout": text, "stderr": "" }));
    }
    let dir = request.get("dir").and_then(Value::as_str).unwrap_or("");
    let mut argv = vec!["disk", "speedtest"];
    if !dir.is_empty() {
        argv.push(dir);
    }
    run_omarchy(&argv)
}

pub fn speed_net(backend: &str, root: Option<&Path>, request: &Value) -> Result<Value, String> {
    if backend == "plain" {
        return Ok(json!({ "exit": 0, "stdout": "plain-mbps\n", "stderr": "" }));
    }
    let phase = request
        .get("phase")
        .and_then(Value::as_str)
        .unwrap_or("down");
    if phase != "down" && phase != "up" {
        return Err("speedtest phase must be down or up".into());
    }
    if root.is_some() {
        let rel = format!(".local/state/omarchy/speed/net-{phase}.txt");
        let text = read_capped(&host_path(root, &rel))?;
        return Ok(json!({ "exit": 0, "stdout": text, "stderr": "" }));
    }
    run_omarchy(&["network", "speedtest", phase])
}

pub fn unit_output(backend: &str, root: Option<&Path>, request: &Value) -> Result<Value, String> {
    if backend == "plain" {
        return Ok(json!({ "exit": 0, "text": "plain-unit\n" }));
    }
    let kind = request.get("kind").and_then(Value::as_str).unwrap_or("");
    let scope = request
        .get("scope")
        .and_then(Value::as_str)
        .unwrap_or("system");
    let unit = request.get("unit").and_then(Value::as_str).unwrap_or("");
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
    let output = if kind == "status" {
        let mut cmd = Command::new("systemctl");
        if scope == "user" {
            cmd.arg("--user");
        }
        cmd.args(["--no-pager", "--full", "status", unit])
            .output()
            .map_err(|err| err.to_string())?
    } else {
        let mut cmd = Command::new("journalctl");
        if scope == "user" {
            cmd.arg("--user");
        } else {
            cmd.arg("--system");
        }
        cmd.args(["-u", unit, "-n", "80", "--no-pager"])
            .output()
            .map_err(|err| err.to_string())?
    };
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.trim().is_empty() {
        text = String::from_utf8_lossy(&output.stderr).to_string();
    }
    Ok(json!({
        "exit": output.status.code().unwrap_or(1),
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

fn first_text(root: Option<&Path>, paths: &[String]) -> Result<String, String> {
    for path in paths {
        let resolved = host_path(root, path);
        if resolved.is_file() {
            return read_capped(&resolved);
        }
    }
    Ok(String::new())
}

fn exists_map(root: Option<&Path>, user: &str, home: &str) -> Result<Value, String> {
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

fn read_capped(path: &Path) -> Result<String, String> {
    if !path.is_file() {
        return Ok(String::new());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if text.len() > TEXT_CAP {
        return Ok(text.chars().take(TEXT_CAP).collect());
    }
    Ok(text)
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

fn string_list(request: &Value, key: &str) -> Result<Vec<String>, String> {
    let Some(list) = request.get(key).and_then(Value::as_array) else {
        return Err(format!("missing {key}"));
    };
    let mut out = Vec::new();
    for item in list {
        let Some(text) = item.as_str() else {
            return Err(format!("{key} entries must be strings"));
        };
        if text.is_empty() || text.contains('\0') {
            return Err(format!("{key} has an empty path"));
        }
        out.push(text.to_string());
    }
    Ok(out)
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

fn run_omarchy(args: &[&str]) -> Result<Value, String> {
    let output = Command::new("omarchy")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "exit": output.status.code().unwrap_or(1),
        "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
        "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
    }))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|err| err.to_string())?;
    fs::rename(&tmp, path).map_err(|err| err.to_string())
}
