//! Kernel and firmware counters: IRQs, buddy pages, cgroups, RAPL, slabs, sensors.

use super::{is_digits, is_int_text, line_of, milli_to_c, sorted_names, Sys};
use serde_json::{json, Map, Value};
use std::path::Path;

pub fn irq_table(text: &str) -> Value {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() < 2 {
        return Value::Null;
    }
    let cpus = lines[0]
        .split_whitespace()
        .filter(|p| p.starts_with("CPU"))
        .count();
    if cpus == 0 {
        return Value::Null;
    }
    let mut rows = Vec::new();
    for line in &lines[1..] {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < cpus + 1 {
            continue;
        }
        let values: Option<Vec<i64>> = cols[1..=cpus]
            .iter()
            .map(|p| is_digits(p).then(|| p.parse().ok()).flatten())
            .collect();
        let Some(values) = values else {
            continue;
        };
        rows.push(json!({"id": cols[0].trim_end_matches(':'), "values": values}));
        if rows.len() >= 48 {
            break;
        }
    }
    if rows.is_empty() {
        Value::Null
    } else {
        json!({"cpus": cpus, "rows": rows})
    }
}

pub fn buddy(sys: &Sys) -> Value {
    let text = sys.text("proc/buddyinfo");
    let mut totals: Vec<i64> = Vec::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        let Some(zone_at) = cols.iter().position(|c| *c == "zone") else {
            continue;
        };
        let orders: Vec<i64> = cols
            .iter()
            .skip(zone_at + 2)
            .filter(|p| is_digits(p))
            .filter_map(|p| p.parse().ok())
            .collect();
        if orders.len() < 8 {
            continue;
        }
        if totals.is_empty() {
            totals = vec![0; orders.len()];
        }
        for (i, total) in totals.iter_mut().enumerate() {
            if let Some(n) = orders.get(i) {
                *total += n;
            }
        }
    }
    if totals.is_empty() {
        return Value::Null;
    }
    totals.truncate(11);
    json!(totals)
}

fn cgroup_node(path: &Path, rel: &str, depth: usize) -> Option<Value> {
    if depth > 4 {
        return None;
    }
    let raw = line_of(&path.join("memory.current"));
    let current = is_digits(&raw).then(|| raw.parse::<i64>().ok()).flatten();
    let mut children = Vec::new();
    if depth < 4 {
        for name in sorted_names(path) {
            let child = path.join(&name);
            if !child.is_dir() || name.starts_with('.') {
                continue;
            }
            let child_rel = format!("{rel}/{name}");
            if let Some(node) = cgroup_node(&child, child_rel.trim_start_matches('/'), depth + 1) {
                children.push(node);
            }
            if children.len() >= 24 {
                break;
            }
        }
    }
    if current.is_none() && children.is_empty() {
        return None;
    }
    let id = if rel.is_empty() { "/" } else { rel };
    let mut node = Map::new();
    node.insert("id".into(), json!(id));
    node.insert("path".into(), json!(id));
    if let Some(value) = current {
        node.insert("value".into(), json!(value));
    }
    if !children.is_empty() {
        node.insert("children".into(), Value::Array(children));
    }
    Some(Value::Object(node))
}

pub fn cgroups(sys: &Sys) -> Value {
    let base = sys.path("sys/fs/cgroup");
    if !base.is_dir() {
        return Value::Null;
    }
    cgroup_node(&base, "", 0).unwrap_or(Value::Null)
}

pub fn rapl(sys: &Sys) -> Vec<Value> {
    let mut out = Vec::new();
    for name in sys.names("sys/class/powercap") {
        let energy = sys.line(&format!("sys/class/powercap/{name}/energy_uj"));
        if !is_digits(&energy) {
            continue;
        }
        let Ok(uj) = energy.parse::<i64>() else {
            continue;
        };
        let label = sys.line(&format!("sys/class/powercap/{name}/name"));
        let label = if label.is_empty() {
            name.clone()
        } else {
            label
        };
        out.push(json!({"id": name, "name": label, "uj": uj}));
        if out.len() >= 12 {
            break;
        }
    }
    out
}

pub fn slabs(sys: &Sys) -> Vec<Value> {
    let mut rows: Vec<(i64, Value)> = Vec::new();
    for line in sys.text_limit("proc/slabinfo", 256_000).lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("slabinfo") {
            continue;
        }
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 3 || !is_digits(cols[1]) || !is_digits(cols[2]) {
            continue;
        }
        let (Ok(active), Ok(num)) = (cols[1].parse::<i64>(), cols[2].parse::<i64>()) else {
            continue;
        };
        rows.push((
            active,
            json!({"name": cols[0], "active": active, "num": num}),
        ));
    }
    rows.sort_by_key(|row| std::cmp::Reverse(row.0));
    rows.into_iter().take(40).map(|(_, v)| v).collect()
}

const CPU_HWMON_NAMES: &[&str] = &["coretemp", "k10temp", "k8temp", "zenpower"];
const CPU_TZ_TYPES: &[&str] = &[
    "x86_pkg_temp",
    "cpu-thermal",
    "cpu_thermal",
    "k10temp",
    "soc-thermal",
    "tjmax",
    "pkg-temp",
];

pub fn cpu_temp(sys: &Sys) -> Option<i64> {
    for hw in sys.names("sys/class/hwmon") {
        if !hw.starts_with("hwmon") {
            continue;
        }
        let base = format!("sys/class/hwmon/{hw}");
        if !CPU_HWMON_NAMES.contains(&sys.line(&format!("{base}/name")).as_str()) {
            continue;
        }
        if let Some(temp) = milli_to_c(&sys.line(&format!("{base}/temp1_input"))) {
            return Some(temp);
        }
    }
    for zone in sys.names("sys/class/thermal") {
        if !zone.starts_with("thermal_zone") {
            continue;
        }
        let base = format!("sys/class/thermal/{zone}");
        let kind = sys.line(&format!("{base}/type")).trim().to_lowercase();
        if !CPU_TZ_TYPES.contains(&kind.as_str()) && !kind.starts_with("cpu") {
            continue;
        }
        if let Some(temp) = milli_to_c(&sys.line(&format!("{base}/temp"))) {
            return Some(temp);
        }
    }
    None
}

fn hwmon_label(sys: &Sys, base: &str, prefix: &str, idx: &str) -> String {
    let label = sys.line(&format!("{base}/{prefix}{idx}_label"));
    if label.is_empty() {
        format!("{prefix}{idx}")
    } else {
        label
    }
}

pub fn sensors(sys: &Sys) -> Vec<Value> {
    let mut out = Vec::new();
    for hw in sys.names("sys/class/hwmon") {
        if !hw.starts_with("hwmon") {
            continue;
        }
        let base = format!("sys/class/hwmon/{hw}");
        let chip = sys.line(&format!("{base}/name"));
        let chip = if chip.is_empty() { hw.clone() } else { chip };
        for name in sys.names(&base) {
            if name.starts_with("temp") && name.ends_with("_input") {
                let idx = &name[4..name.len() - 6];
                let Some(temp) = milli_to_c(&sys.line(&format!("{base}/{name}"))) else {
                    continue;
                };
                out.push(json!({
                    "id": format!("{chip}:{name}"),
                    "chip": chip,
                    "label": hwmon_label(sys, &base, "temp", idx),
                    "kind": "temp",
                    "value": temp,
                }));
            } else if name.starts_with("fan") && name.ends_with("_input") {
                let idx = &name[3..name.len() - 6];
                let raw = sys.line(&format!("{base}/{name}"));
                if !is_int_text(&raw) {
                    continue;
                }
                let rpm: i64 = raw.parse().unwrap_or(0);
                if rpm <= 0 {
                    continue;
                }
                out.push(json!({
                    "id": format!("{chip}:{name}"),
                    "chip": chip,
                    "label": hwmon_label(sys, &base, "fan", idx),
                    "kind": "fan",
                    "value": rpm,
                }));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn irq_tables_keep_numeric_rows_only() {
        let text = "           CPU0       CPU1\n  0:         10          0   IO-APIC   timer\nNMI:          1          2   Non-maskable\nERR:          0\n";
        let table = irq_table(text);
        assert_eq!(table["cpus"], 2);
        assert_eq!(table["rows"][0]["id"], "0");
        assert_eq!(table["rows"][1]["values"], json!([1, 2]));
        assert_eq!(table["rows"].as_array().unwrap().len(), 2);
        assert_eq!(irq_table("CPU0\n"), Value::Null);
    }
}
