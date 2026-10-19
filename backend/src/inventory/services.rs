//! User services, failed system units, and the Atmos allowlist.

use super::{capture, require};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

// Keep in sync with services/Systemd.js allowlist().
const ALLOWLIST: &[(&str, &str)] = &[
    ("pipewire.service", "user"),
    ("pipewire-pulse.service", "user"),
    ("wireplumber.service", "user"),
    ("xdg-desktop-portal.service", "user"),
    ("xdg-desktop-portal-hyprland.service", "user"),
    ("bluetooth.service", "system"),
    ("NetworkManager.service", "system"),
    ("cups.service", "system"),
    ("docker.service", "system"),
    ("tailscaled.service", "system"),
    ("fstrim.timer", "system"),
];
const ROW_CAP: usize = 80;

struct Row {
    unit: String,
    scope: &'static str,
    load: String,
    active: String,
    sub: String,
    description: String,
    unit_file_state: String,
}

/// What the snapshot embeds: a tool that fails leaves its part empty.
pub fn collect() -> Value {
    gather(&|program, args| Ok(capture(program, args))).unwrap_or(Value::Null)
}

/// What the services page shows: the unit lists come from `systemctl`, and a
/// `systemctl` that fails is an error, not a machine with no services.
pub fn try_collect() -> Result<Value, String> {
    gather(&require)
}

/// How `gather` gets a tool's output: leniently or strictly.
type Fetch<'a> = &'a dyn Fn(&str, &[&str]) -> Result<String, String>;

fn gather(fetch: Fetch) -> Result<Value, String> {
    let user_units = parse_units(
        &fetch(
            "systemctl",
            &[
                "--user",
                "--no-legend",
                "--no-pager",
                "list-units",
                "--type=service",
                "--all",
            ],
        )?,
        "user",
    );
    let sys_failed = parse_units(
        &fetch(
            "systemctl",
            &[
                "--no-legend",
                "--no-pager",
                "list-units",
                "--type=service",
                "--state=failed",
            ],
        )?,
        "system",
    );
    let mut args = vec!["--no-legend", "--no-pager", "list-units", "--all"];
    args.extend(
        ALLOWLIST
            .iter()
            .filter(|(_, s)| *s == "system")
            .map(|(u, _)| *u),
    );
    let extra = parse_units(&fetch("systemctl", &args)?, "system");
    let user_files = parse_files(&fetch(
        "systemctl",
        &["--user", "--no-legend", "--no-pager", "list-unit-files"],
    )?);
    let sys_files = parse_files(&fetch(
        "systemctl",
        &["--no-legend", "--no-pager", "list-unit-files"],
    )?);
    Ok(build(
        sys_failed,
        extra,
        user_units,
        &user_files,
        &sys_files,
    ))
}

fn build(
    sys_failed: Vec<Row>,
    extra: Vec<Row>,
    user_units: Vec<Row>,
    user_files: &HashMap<String, String>,
    sys_files: &HashMap<String, String>,
) -> Value {
    let mut seen = HashSet::new();
    let mut rows: Vec<Row> = Vec::new();
    for mut row in sys_failed.into_iter().chain(extra).chain(user_units) {
        if !seen.insert((row.scope, row.unit.clone())) {
            continue;
        }
        let files = if row.scope == "user" {
            user_files
        } else {
            sys_files
        };
        row.unit_file_state = files.get(&row.unit).cloned().unwrap_or_default();
        rows.push(row);
    }
    rows.sort_by_key(rank);
    Value::Array(
        rows.into_iter()
            .take(ROW_CAP)
            .map(|r| {
                json!({
                    "unit": r.unit,
                    "scope": r.scope,
                    "load": r.load,
                    "active": r.active,
                    "sub": r.sub,
                    "description": r.description,
                    "unitFileState": r.unit_file_state,
                })
            })
            .collect(),
    )
}

fn rank(row: &Row) -> (u8, u8, String) {
    let failed = row.active == "failed" || row.sub == "failed";
    let allowed = ALLOWLIST
        .iter()
        .any(|(u, s)| *u == row.unit && *s == row.scope);
    (u8::from(!failed), u8::from(!allowed), row.unit.clone())
}

fn parse_units(text: &str, scope: &'static str) -> Vec<Row> {
    text.lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            let get = |i: usize| cols.get(i).map(|c| c.to_string()).unwrap_or_default();
            (!cols.is_empty()).then(|| Row {
                unit: get(0),
                scope,
                load: get(1),
                active: get(2),
                sub: get(3),
                description: cols.iter().skip(4).copied().collect::<Vec<_>>().join(" "),
                unit_file_state: String::new(),
            })
        })
        .collect()
}

fn parse_files(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            (cols.len() >= 2).then(|| (cols[0].to_string(), cols[1].to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_units_rank_first_then_the_allowlist_then_name() {
        let user = parse_units("zeta.service loaded active running Zeta\npipewire.service loaded active running PipeWire\n", "user");
        let failed = parse_units("bad.service loaded failed failed Bad thing\n", "system");
        let files = HashMap::from([("zeta.service".to_string(), "enabled".to_string())]);
        let got = build(failed, Vec::new(), user, &files, &HashMap::new());
        let units: Vec<_> = got
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["unit"].as_str().unwrap())
            .collect();
        assert_eq!(units, ["bad.service", "pipewire.service", "zeta.service"]);
        assert_eq!(got[0]["description"], "Bad thing");
        assert_eq!(got[2]["unitFileState"], "enabled");
    }
}
