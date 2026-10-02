//! Flock plus atomic rename for the platform files a backend reads and writes.

use crate::error::{Error, Result};
use crate::fsutil::atomic_write;
use std::fs::{self, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::domain::{Place, PlaceKind, Ty};
use crate::effects;
use crate::patch;

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

const LOCK_SH: i32 = 1;
const LOCK_EX: i32 = 2;
const LOCK_UN: i32 = 8;

pub fn read_place(root: Option<&Path>, place: &Place, key: &str, ty: Ty) -> Result<Value> {
    let path = place_path(root, &place.rel);
    with_read_lock(&path, || match &place.kind {
        PlaceKind::Map => {
            let map = load_object(&path, place)?;
            Ok(map.get(key).cloned().unwrap_or(Value::Null))
        }
        PlaceKind::Shell => {
            let doc = load_value(&path)?;
            Ok(patch::shell_get(&doc, key))
        }
        PlaceKind::Nested { path: keys } => {
            let doc = load_value(&path)?;
            Ok(patch::nested_get(&doc, keys))
        }
        PlaceKind::Lua { begin, end } => {
            let text = read_text(&path)?;
            patch::lua_read(&text, begin, end, key, ty)
        }
        PlaceKind::Line { prefix } => {
            if !path.exists() {
                return Ok(Value::Null);
            }
            let text = fs::read_to_string(&path)?;
            patch::line_read(&text, prefix, ty)
        }
        PlaceKind::Flag => Ok(Value::Bool(path.is_file())),
        PlaceKind::Hypr { kind } => effects::read_hypr(&path, kind, key),
        PlaceKind::Doc => effects::read_doc(root, &path, key),
        PlaceKind::Items => read_items(&path),
        PlaceKind::Whole => read_whole(&path),
    })
}

pub fn write_place(
    root: Option<&Path>,
    place: &Place,
    key: &str,
    ty: Ty,
    value: &Value,
) -> Result<()> {
    if !ty.accepts(value) {
        return Err(format!("{key} expects {}", ty.name()).into());
    }
    if matches!(place.kind, PlaceKind::Line { .. }) {
        if let Some(text) = value.as_str() {
            if text.contains('\n') || text.contains('\r') {
                return Err(format!("{key} cannot contain a newline").into());
            }
        }
    }
    let path = place_path(root, &place.rel);
    if let PlaceKind::Hypr { kind } = &place.kind {
        // hypr-sentinel.py locks this same file. Taking the lock here first deadlocks the child.
        return effects::write_hypr(&path, kind, key, value);
    }
    with_lock(&path, || match &place.kind {
        PlaceKind::Map => {
            let mut map = load_object(&path, place)?;
            map.insert(key.to_string(), value.clone());
            store_object(&path, place, &map)
        }
        PlaceKind::Shell => {
            let mut doc = load_value(&path)?;
            patch::shell_set(&mut doc, key, value);
            write_pretty(&path, &doc)
        }
        PlaceKind::Nested { path: keys } => {
            let mut doc = load_value(&path)?;
            patch::nested_set(&mut doc, keys, value);
            write_pretty(&path, &doc)
        }
        PlaceKind::Lua { begin, end } => {
            let text = read_text(&path)?;
            let next = patch::lua_write(&text, begin, end, key, value)?;
            atomic_write(&path, next.as_bytes())
        }
        PlaceKind::Line { prefix } => {
            let existing = read_text(&path)?;
            let next = patch::line_write(&existing, prefix, value);
            atomic_write(&path, next.as_bytes())
        }
        PlaceKind::Flag => write_flag(&path, &place.rel, value),
        PlaceKind::Hypr { .. } => Err("hypr writes run outside the lock".into()),
        PlaceKind::Doc => effects::write_doc(root, &path, key, value),
        PlaceKind::Items => {
            let body = serde_json::to_string(&serde_json::json!({ "items": value }))?;
            atomic_write(&path, format!("{body}\n").as_bytes())
        }
        PlaceKind::Whole => {
            let body = serde_json::to_string(value)?;
            atomic_write(&path, format!("{body}\n").as_bytes())
        }
    })
}

fn place_path(root: Option<&Path>, rel: &str) -> PathBuf {
    if let Some(root) = root {
        return root.join(rel);
    }
    if rel.starts_with("etc/") || rel.starts_with("var/") || rel.starts_with("sys/") {
        return Path::new("/").join(rel);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));
    home.join(rel)
}

fn read_text(path: &Path) -> Result<String> {
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(path).map_err(Error::from)
}

fn load_value(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(Value::Object(Map::new()));
    }
    match serde_json::from_str::<Value>(&text)? {
        value if value.is_object() => Ok(value),
        _ => Err(format!("{} is not a JSON object", path.display()).into()),
    }
}

fn write_pretty(path: &Path, value: &Value) -> Result<()> {
    let body = serde_json::to_string_pretty(value)?;
    atomic_write(path, format!("{body}\n").as_bytes())
}

fn write_flag(path: &Path, rel: &str, value: &Value) -> Result<()> {
    let on = value
        .as_bool()
        .ok_or_else(|| format!("{rel} expects a bool"))?;
    if !on {
        if path.exists() {
            fs::remove_file(path)?;
        }
        return Ok(());
    }
    let body = flag_text(rel);
    atomic_write(path, body.as_bytes())
}

fn flag_text(rel: &str) -> String {
    let name = Path::new(rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("flag.lua");
    let mut candidates = Vec::new();
    if let Some(root) = std::env::var_os("OMARCHY_PATH") {
        candidates.push(PathBuf::from(root).join("default/hypr/toggles").join(name));
    }
    candidates.push(PathBuf::from("/usr/share/omarchy/default/hypr/toggles").join(name));
    for candidate in candidates {
        if let Ok(text) = fs::read_to_string(&candidate) {
            if !text.trim().is_empty() {
                return if text.ends_with('\n') {
                    text
                } else {
                    format!("{text}\n")
                };
            }
        }
    }
    match name {
        "window-no-gaps.lua" => "\
-- Remove all window gaps and borders.
hl.config({
  general = {
    gaps_out = 0,
    gaps_in = 0,
    border_size = 0,
  },
  decoration = {
    rounding = 0,
  },
})
"
        .into(),
        "single-window-aspect-ratio.lua" => "\
-- Avoid overly wide single-window layouts on wide screens.
hl.config({
  layout = {
    single_window_aspect_ratio = { 1, 1 },
  },
})
"
        .into(),
        _ => format!("-- atmos flag {name}\nhl.config({{}})\n"),
    }
}

fn load_object(path: &Path, place: &Place) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match &place.kind {
        PlaceKind::Map => parse_object(&text, path),
        _ => Err("not an object place".into()),
    }
}

fn store_object(path: &Path, place: &Place, map: &Map<String, Value>) -> Result<()> {
    match &place.kind {
        PlaceKind::Map => {
            let body = serde_json::to_string(&Value::Object(map.clone()))?;
            atomic_write(path, format!("{body}\n").as_bytes())
        }
        _ => Err("not an object place".into()),
    }
}

fn parse_object(text: &str, path: &Path) -> Result<Map<String, Value>> {
    match serde_json::from_str(text)? {
        Value::Object(map) => Ok(map),
        _ => Err(format!("{} is not a JSON object", path.display()).into()),
    }
}

fn read_items(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path)?;
    match serde_json::from_str(&text)? {
        Value::Object(map) => Ok(map.get("items").cloned().unwrap_or(Value::Null)),
        _ => Err(format!("{} is not a favorites object", path.display()).into()),
    }
}

fn read_whole(path: &Path) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(Error::from)
}

/// Holds an flock until it drops, so every exit path releases it.
struct Locked(fs::File);

impl Drop for Locked {
    fn drop(&mut self) {
        unsafe { flock(self.0.as_raw_fd(), LOCK_UN) };
    }
}

fn lock_path_for(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".atmos.lock");
    PathBuf::from(name)
}

fn acquire(file: fs::File, mode: i32, lock_path: &Path) -> Result<Locked> {
    let rc = unsafe { flock(file.as_raw_fd(), mode) };
    if rc != 0 {
        return Err(Error::io(std::io::Error::last_os_error(), lock_path));
    }
    Ok(Locked(file))
}

/// Writers serialise on a sidecar lock next to the file.
fn with_lock<T>(path: &Path, body: impl FnOnce() -> Result<T>) -> Result<T> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let lock_path = lock_path_for(path);
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|err| Error::io(err, &lock_path))?;
    let _held = acquire(file, LOCK_EX, &lock_path)?;
    body()
}

/// Readers never create anything. Writers replace files by rename, so a
/// reader sees the old file or the new one; it only waits on the sidecar
/// lock when a writer has made one, so a read cannot litter /etc or ~/.config.
fn with_read_lock<T>(path: &Path, body: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock_path = lock_path_for(path);
    let held = match OpenOptions::new().read(true).open(&lock_path) {
        Ok(file) => acquire(file, LOCK_SH, &lock_path).ok(),
        Err(_) => None,
    };
    let result = body();
    drop(held);
    result
}
