//! Flock plus atomic rename for the platform files a backend reads and writes.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::domain::{Place, PlaceKind, Ty};

extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

const LOCK_EX: i32 = 2;
const LOCK_UN: i32 = 8;

pub fn read_place(root: &Path, place: &Place, key: &str, ty: Ty) -> Result<Value, String> {
    let path = root.join(&place.rel);
    with_lock(&path, || match &place.kind {
        PlaceKind::Map | PlaceKind::Sentinel { .. } => {
            let map = load_object(&path, place)?;
            Ok(map.get(key).cloned().unwrap_or(Value::Null))
        }
        PlaceKind::Line { prefix } => read_line(&path, prefix, ty),
        PlaceKind::Items => read_items(&path),
        PlaceKind::Whole => read_whole(&path),
    })
}

pub fn write_place(root: &Path, place: &Place, key: &str, ty: Ty, value: &Value) -> Result<(), String> {
    if !ty.accepts(value) {
        return Err(format!("{key} expects {}", ty.name()));
    }
    if matches!(place.kind, PlaceKind::Line { .. }) {
        if let Some(text) = value.as_str() {
            if text.contains('\n') || text.contains('\r') {
                return Err(format!("{key} cannot contain a newline"));
            }
        }
    }
    let path = root.join(&place.rel);
    with_lock(&path, || match &place.kind {
        PlaceKind::Map | PlaceKind::Sentinel { .. } => {
            let mut map = load_object(&path, place)?;
            map.insert(key.to_string(), value.clone());
            store_object(&path, place, &map)
        }
        PlaceKind::Line { prefix } => write_line(&path, prefix, value),
        PlaceKind::Items => {
            let body = serde_json::to_string(&serde_json::json!({ "items": value }))
                .map_err(|err| err.to_string())?;
            atomic_write(&path, format!("{body}\n").as_bytes())
        }
        PlaceKind::Whole => {
            let body = serde_json::to_string(value).map_err(|err| err.to_string())?;
            atomic_write(&path, format!("{body}\n").as_bytes())
        }
    })
}

fn load_object(path: &Path, place: &Place) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match &place.kind {
        PlaceKind::Map => parse_object(&text, path),
        PlaceKind::Sentinel { begin, end, mark } => read_sentinel_map(&text, begin, end, mark),
        _ => Err("not an object place".into()),
    }
}

fn store_object(path: &Path, place: &Place, map: &Map<String, Value>) -> Result<(), String> {
    match &place.kind {
        PlaceKind::Map => {
            let body = serde_json::to_string(&Value::Object(map.clone())).map_err(|err| err.to_string())?;
            atomic_write(path, format!("{body}\n").as_bytes())
        }
        PlaceKind::Sentinel { begin, end, mark } => {
            let existing = if path.exists() {
                fs::read_to_string(path).map_err(|err| err.to_string())?
            } else {
                String::new()
            };
            let next = write_sentinel(&existing, begin, end, mark, map)?;
            atomic_write(path, next.as_bytes())
        }
        _ => Err("not an object place".into()),
    }
}

fn parse_object(text: &str, path: &Path) -> Result<Map<String, Value>, String> {
    match serde_json::from_str(text).map_err(|err| err.to_string())? {
        Value::Object(map) => Ok(map),
        _ => Err(format!("{} is not a JSON object", path.display())),
    }
}

fn read_sentinel_map(text: &str, begin: &str, end: &str, mark: &str) -> Result<Map<String, Value>, String> {
    let lines = split_lines(text);
    let Some(start) = lines.iter().position(|line| line.trim() == begin) else {
        return Ok(Map::new());
    };
    let end_at = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| line.trim() == end)
        .map(|(index, _)| index)
        .ok_or_else(|| format!("missing {end}"))?;
    let mut map = Map::new();
    for line in &lines[start + 1..end_at] {
        let Some(rest) = line.trim().strip_prefix(mark) else {
            continue;
        };
        match serde_json::from_str(rest).map_err(|err| err.to_string())? {
            Value::Object(parsed) => map = parsed,
            _ => return Err("sentinel JSON is not an object".into()),
        }
    }
    Ok(map)
}

fn write_sentinel(
    text: &str,
    begin: &str,
    end: &str,
    mark: &str,
    map: &Map<String, Value>,
) -> Result<String, String> {
    let lines = split_lines(text);
    let json = serde_json::to_string(&Value::Object(map.clone())).map_err(|err| err.to_string())?;
    let block = [begin.to_string(), format!("{mark}{json}"), end.to_string()];
    let begin_at = lines.iter().position(|line| line.trim() == begin);
    let (head, tail) = if let Some(start) = begin_at {
        let end_at = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, line)| line.trim() == end)
            .map(|(index, _)| index)
            .ok_or_else(|| format!("missing {end}"))?;
        (lines[..start].to_vec(), lines[end_at + 1..].to_vec())
    } else {
        (lines, Vec::new())
    };
    let mut out = Vec::new();
    out.extend(head);
    out.extend(block);
    out.extend(tail);
    Ok(join_lines(&out))
}

fn read_line(path: &Path, prefix: &str, ty: Ty) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let text = text.trim_end_matches(['\n', '\r']);
    if text.is_empty() && prefix.is_empty() {
        return match ty {
            Ty::String => Ok(Value::String(String::new())),
            _ => Ok(Value::Null),
        };
    }
    let rest = text
        .strip_prefix(prefix)
        .ok_or_else(|| format!("{} does not start with {prefix}", path.display()))?;
    match ty {
        Ty::String => Ok(Value::String(rest.to_string())),
        Ty::Bool => match rest {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(format!("expected true or false in {}", path.display())),
        },
        Ty::Int => {
            let number: i64 = rest
                .parse()
                .map_err(|_| format!("expected an integer in {}", path.display()))?;
            Ok(Value::from(number))
        }
        Ty::Number => {
            let number: serde_json::Number = rest
                .parse()
                .map_err(|_| format!("expected a number in {}", path.display()))?;
            Ok(Value::Number(number))
        }
        Ty::List => Err("line files do not store lists".into()),
    }
}

fn write_line(path: &Path, prefix: &str, value: &Value) -> Result<(), String> {
    let rendered = match value {
        Value::String(text) => text.clone(),
        Value::Bool(true) => "true".into(),
        Value::Bool(false) => "false".into(),
        Value::Number(number) => number.to_string(),
        _ => return Err("line files store a scalar".into()),
    };
    atomic_write(path, format!("{prefix}{rendered}\n").as_bytes())
}

fn read_items(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    match serde_json::from_str(&text).map_err(|err| err.to_string())? {
        Value::Object(map) => Ok(map.get("items").cloned().unwrap_or(Value::Null)),
        _ => Err(format!("{} is not a favorites object", path.display())),
    }
}

fn read_whole(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn split_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn join_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn with_lock<T>(path: &Path, body: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut lock_name = path.as_os_str().to_os_string();
    lock_name.push(".atmos.lock");
    let lock_path = PathBuf::from(lock_name);
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|err| err.to_string())?;
    let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX) };
    if rc != 0 {
        return Err(format!("flock {} failed: {}", lock_path.display(), std::io::Error::last_os_error()));
    }
    let result = body();
    unsafe { flock(file.as_raw_fd(), LOCK_UN) };
    result
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("no parent for {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(format!(".tmp-{}-{nanos}", std::process::id()));
    let tmp = parent.join(tmp_name);
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|err| err.to_string())?;
        file.write_all(bytes).map_err(|err| err.to_string())?;
        file.sync_all().map_err(|err| err.to_string())?;
    }
    fs::rename(&tmp, path).map_err(|err| {
        let _ = fs::remove_file(&tmp);
        err.to_string()
    })
}
