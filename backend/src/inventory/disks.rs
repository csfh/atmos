//! Disks, LUKS devices, swap and snapper snapshots from lsblk, df and snapper. No sudo.

use super::{capture, number, text};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

pub fn collect() -> Value {
    let raw = capture(
        "lsblk",
        &[
            "-J",
            "-b",
            "-o",
            "NAME,PATH,SIZE,TYPE,FSTYPE,LABEL,UUID,MOUNTPOINT,MODEL,TRAN,ROTA,HOTPLUG,PKNAME,RM",
        ],
    );
    let tree: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    let df = parse_df(&capture("df", &["-B1", "-P"]));
    let mut disks = Vec::new();
    let mut swaps = Vec::new();
    let mut luks = Vec::new();
    for node in tree["blockdevices"].as_array().into_iter().flatten() {
        let name = text(&node["name"]);
        let path = text(&node["path"]);
        collect_luks(node, &mut luks);
        let is_disk = text(&node["type"]) == "disk";
        if is_disk
            && (text(&node["fstype"]) == "swap"
                || name.starts_with("zram")
                || name.starts_with("loop"))
        {
            let label = text(&node["label"]);
            swaps.push(json!({
                "path": path,
                "name": name,
                "size": number(&node["size"]),
                "label": if label.is_empty() { name.clone() } else { label },
            }));
            continue;
        }
        if !is_disk || !path.starts_with("/dev/") {
            continue;
        }
        let info = capture("omarchy", &["drive", "info", &path]);
        let info = info.trim();
        disks.push(json!({
            "path": path,
            "name": name,
            "model": text(&node["model"]).trim(),
            "tran": text(&node["tran"]),
            "size": number(&node["size"]),
            "info": if info.is_empty() { path.as_str() } else { info },
            "mounts": mounts_for_disk(node, &df),
        }));
    }
    json!({
        "disks": disks,
        "luksDevices": luks,
        "swapDevices": swaps,
        "snapshots": snapshots(&capture("snapper", &["--jsonout", "list"])),
    })
}

fn device_paths(node: &Value, acc: &mut HashSet<String>) {
    let path = text(&node["path"]);
    if !path.is_empty() {
        acc.insert(path);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        device_paths(child, acc);
    }
}

fn collect_luks(node: &Value, acc: &mut Vec<String>) {
    if node["fstype"].as_str() == Some("crypto_LUKS") {
        let path = text(&node["path"]);
        if !path.is_empty() {
            acc.push(path);
        }
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect_luks(child, acc);
    }
}

struct Mount {
    target: String,
    source: String,
    size: i64,
    used: i64,
    avail: i64,
}

fn parse_df(raw: &str) -> Vec<Mount> {
    let mut rows = Vec::new();
    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }
        let source = parts[0].split('[').next().unwrap_or("");
        let target = parts[5];
        if !target.starts_with('/')
            || source.starts_with("tmpfs")
            || source.starts_with("devtmpfs")
            || source == "efivarfs"
        {
            continue;
        }
        let (Ok(size), Ok(used), Ok(avail)) = (
            parts[1].parse::<i64>(),
            parts[2].parse::<i64>(),
            parts[3].parse::<i64>(),
        ) else {
            continue;
        };
        rows.push(Mount {
            target: target.into(),
            source: source.into(),
            size,
            used,
            avail,
        });
    }
    rows
}

fn mount_score(target: &str) -> usize {
    match target {
        "/" => 0,
        "/boot" => 1,
        "/home" => 2,
        t if t == "/mnt" || t.starts_with("/mnt/") => 3,
        t => 10 + t.chars().count(),
    }
}

fn base(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn mounts_for_disk(node: &Value, df: &[Mount]) -> Vec<Value> {
    let mut paths = HashSet::new();
    device_paths(node, &mut paths);
    let names: HashSet<String> = paths.iter().map(|p| base(p).to_string()).collect();
    let mut by_source: BTreeMap<&str, &Mount> = BTreeMap::new();
    for row in df {
        if !paths.contains(&row.source) && !names.contains(base(&row.source)) {
            continue;
        }
        match by_source.get(row.source.as_str()) {
            Some(cur) if mount_score(&row.target) >= mount_score(&cur.target) => {}
            _ => {
                by_source.insert(&row.source, row);
            }
        }
    }
    let mut rows: Vec<&Mount> = by_source.into_values().collect();
    rows.sort_by(|a, b| a.target.cmp(&b.target));
    rows.into_iter()
        .map(|m| {
            json!({
                "target": m.target,
                "source": m.source,
                "size": m.size,
                "used": m.used,
                "avail": m.avail,
            })
        })
        .collect()
}

fn snapshots(raw: &str) -> Vec<Value> {
    let Ok(Value::Object(configs)) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    let mut out: Vec<(i64, String, Value)> = Vec::new();
    for (config, entries) in &configs {
        for entry in entries.as_array().into_iter().flatten() {
            let Some(map) = entry.as_object() else {
                continue;
            };
            let number = map
                .get("number")
                .or_else(|| map.get("id"))
                .unwrap_or(&Value::Null);
            let id = match number {
                Value::Number(n) => n.as_i64(),
                Value::String(s) => s.trim().parse().ok(),
                _ => None,
            };
            let Some(id) = id.filter(|id| *id >= 1) else {
                continue;
            };
            let field = |key: &str| map.get(key).map(text).unwrap_or_default();
            out.push((
                id,
                config.clone(),
                json!({
                    "config": config,
                    "id": id,
                    "date": field("date"),
                    "description": field("description"),
                    "type": field("type"),
                }),
            ));
        }
    }
    out.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    out.into_iter().take(40).map(|(_, _, v)| v).collect()
}
