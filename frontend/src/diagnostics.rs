//! The Diagnostics page's logic (`pages/system/DiagnosticsPage.qml`, `services/Diagnostics.js`):
//! the diagnostics inventory as `normalize` keeps it, what each row says, and the report the
//! Copy and Ask actions send (`reportText`). `tests/catalog_diagnostics.rs` runs `Diagnostics.js`
//! and the page's expressions under Node against this.

use serde_json::{Map, Value, json};

use crate::jobs::Job;
use crate::js;
use crate::kernel::clean_line;

fn obj(v: Option<&Value>) -> Value {
    v.filter(|v| v.is_object())
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn arr(v: Option<&Value>) -> Vec<Value> {
    v.and_then(Value::as_array).cloned().unwrap_or_default()
}

fn int(v: Option<&Value>, zero_ok: bool) -> Value {
    let n = js::round(js::number(v));
    let ok = n.is_finite() && if zero_ok { n >= 0.0 } else { n > 0.0 };
    js::json_number(if ok { n } else { 0.0 })
}

fn yes(v: &Value, k: &str) -> bool {
    v.get(k) == Some(&Value::Bool(true))
}

fn t(v: &Value, k: &str, max: usize) -> String {
    clean_line(v.get(k), max)
}

fn unit_name(v: Option<&Value>) -> String {
    let s = v
        .filter(|v| !v.is_null())
        .map_or_else(String::new, js::string);
    crate::units::unit_name(&s)
}

const STATES: &[&str] = &[
    "active",
    "inactive",
    "failed",
    "activating",
    "deactivating",
    "unknown",
];

fn state(v: Option<&Value>) -> String {
    let s = v
        .filter(|v| !v.is_null())
        .map_or_else(String::new, js::string);
    let s = s.trim();
    if STATES.contains(&s) {
        s.to_owned()
    } else {
        String::new()
    }
}

fn lines(v: Option<&Value>, keep: usize) -> Vec<Value> {
    arr(v)
        .iter()
        .map(|l| clean_line(Some(l), 240))
        .filter(|l| !l.is_empty())
        .take(keep)
        .map(Value::from)
        .collect()
}

fn services(v: Option<&Value>, names: &[&str]) -> Value {
    let src = obj(v);
    let map: Map<String, Value> = names
        .iter()
        .map(|n| ((*n).to_owned(), Value::from(state(src.get(*n)))))
        .collect();
    Value::Object(map)
}

/// `normalize(raw)`.
pub fn normalize(raw: Option<&Value>) -> Value {
    let src = obj(raw);
    let part = |k: &str| obj(src.get(k));
    let (k, o, a, h, d, m, g, n, p) = (
        part("kernel"),
        part("omarchy"),
        part("atmos"),
        part("hyprland"),
        part("disk"),
        part("memory"),
        part("gpu"),
        part("network"),
        part("pacman"),
    );
    let sentinels = obj(a.get("sentinels"));
    let failed: Vec<Value> = arr(src.get("failedUnits"))
        .iter()
        .filter_map(|row| {
            let row = obj(Some(row));
            let unit = unit_name(row.get("unit"));
            (!unit.is_empty()).then(|| json!({
                "unit": unit,
                "scope": if row.get("scope") == Some(&Value::from("user")) { "user" } else { "system" },
                "sub": t(&row, "sub", 32), "description": t(&row, "description", 160),
            }))
        })
        .take(24)
        .collect();
    let percent = int(d.get("percent"), true)
        .as_f64()
        .unwrap_or(0.0)
        .min(100.0);
    let kind = n
        .get("kind")
        .and_then(Value::as_str)
        .filter(|k| matches!(*k, "wifi" | "ethernet" | "offline"))
        .unwrap_or_default();
    let hostname = match src.get("hostname") {
        Some(v) if js::string(v).contains(['\r', '\n']) => String::new(),
        v => clean_line(v, 253),
    };
    json!({
        "generatedAt": t(&src, "generatedAt", 64),
        "hostname": hostname,
        "kernel": { "sysname": t(&k, "sysname", 32), "release": t(&k, "release", 64), "machine": t(&k, "machine", 32) },
        "omarchy": { "version": t(&o, "version", 64), "channel": t(&o, "channel", 32), "path": t(&o, "path", 240) },
        "atmos": {
            "revision": t(&a, "revision", 80), "installed": yes(&a, "installed"), "packaged": yes(&a, "packaged"),
            "hyprAtmos": yes(&a, "hyprAtmos"), "hyprAtmosLayout": yes(&a, "hyprAtmosLayout"),
            "sentinels": {
                "look": yes(&sentinels, "look"), "input": yes(&sentinels, "input"),
                "autostart": yes(&sentinels, "autostart"), "bindings": yes(&sentinels, "bindings"),
                "windows": yes(&sentinels, "windows"),
            },
        },
        "hyprland": { "version": t(&h, "version", 80), "configErrors": lines(h.get("configErrors"), 20) },
        "failedUnits": failed,
        "disk": {
            "path": "/", "total": int(d.get("total"), false), "used": int(d.get("used"), true),
            "available": int(d.get("available"), true), "percent": js::json_number(percent),
        },
        "memory": {
            "total": int(m.get("total"), false), "used": int(m.get("used"), true),
            "available": int(m.get("available"), true), "swapTotal": int(m.get("swapTotal"), true),
            "swapUsed": int(m.get("swapUsed"), true),
        },
        "gpu": { "driver": t(&g, "driver", 64), "identity": t(&g, "identity", 160) },
        "portals": services(src.get("portals"), &["xdg-desktop-portal", "xdg-desktop-portal-hyprland", "xdg-desktop-portal-gtk"]),
        "pipewire": services(src.get("pipewire"), &["pipewire", "pipewire-pulse", "wireplumber"]),
        "network": { "online": yes(&n, "online"), "kind": kind },
        "pacman": { "syncOk": yes(&p, "syncOk"), "lastSync": t(&p, "lastSync", 64), "dbCount": int(p.get("dbCount"), true) },
        "recentErrors": lines(src.get("recentErrors"), 15),
    })
}

fn s(v: &Value, k: &str) -> String {
    v.get(k)
        .filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

fn num(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

pub fn status(ok: bool) -> &'static str {
    if ok { "OK" } else { "Needs attention" }
}

pub fn hypr_ok(d: &Value) -> bool {
    arr(d["hyprland"].get("configErrors")).is_empty()
}

pub fn services_ok(map: &Value) -> bool {
    let states: Vec<String> = map
        .as_object()
        .into_iter()
        .flatten()
        .map(|(_, v)| state(Some(v)))
        .filter(|s| !s.is_empty())
        .collect();
    !states.is_empty() && states.iter().all(|s| s == "active")
}

pub fn disk_pressure(d: &Value) -> bool {
    num(&d["disk"], "percent") >= 90.0
}

pub fn memory_pressure(d: &Value) -> bool {
    let total = num(&d["memory"], "total");
    total > 0.0 && num(&d["memory"], "used") / total >= 0.95
}

pub fn atmos_ok(d: &Value) -> bool {
    yes(&d["atmos"], "hyprAtmos") || yes(&d["atmos"], "packaged")
}

pub fn kernel_summary(d: &Value) -> String {
    ["sysname", "release", "machine"]
        .iter()
        .map(|k| s(&d["kernel"], k))
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn omarchy_summary(d: &Value) -> String {
    let (v, c) = (s(&d["omarchy"], "version"), s(&d["omarchy"], "channel"));
    match (v.is_empty(), c.is_empty()) {
        (false, false) => format!("Omarchy {v} ({c})"),
        (false, true) => format!("Omarchy {v}"),
        _ => "Omarchy version was not readable.".into(),
    }
}

pub fn atmos_summary(d: &Value) -> String {
    let a = &d["atmos"];
    if !js::truthy(a.get("installed")) {
        return "Atmos is not installed in XDG data.".into();
    }
    let mut bits = Vec::new();
    let rev = s(a, "revision");
    if !rev.is_empty() {
        bits.push(rev);
    }
    bits.push(if js::truthy(a.get("hyprAtmos")) {
        "hypr.atmos is required".into()
    } else if js::truthy(a.get("packaged")) {
        "Hyprland integration is off. Run atmos --setup to turn it on".into()
    } else {
        "hypr.atmos is missing from hyprland.lua".into()
    });
    format!("{}.", bits.join(". "))
}

pub fn hypr_summary(d: &Value) -> String {
    let v = s(&d["hyprland"], "version");
    let ver = if v.is_empty() {
        "Hyprland version was not readable".into()
    } else {
        format!("Hyprland {v}")
    };
    match arr(d["hyprland"].get("configErrors")).len() {
        0 => format!("{ver}. No config errors."),
        1 => format!("{ver}. One config error."),
        n => format!("{ver}. {n} config errors."),
    }
}

pub fn gpu_summary(d: &Value) -> String {
    let (dr, id) = (s(&d["gpu"], "driver"), s(&d["gpu"], "identity"));
    match (dr.is_empty(), id.is_empty()) {
        (false, false) => format!("{id} ({dr})"),
        (false, true) => dr,
        (true, false) => id,
        _ => "GPU driver was not readable.".into(),
    }
}

pub fn network_summary(d: &Value) -> String {
    let n = &d["network"];
    if !js::truthy(n.get("online")) {
        return "No default route.".into();
    }
    match n["kind"].as_str() {
        Some("wifi") => "Online on Wi-Fi.",
        Some("ethernet") => "Online on Ethernet.",
        _ => "Online.",
    }
    .into()
}

pub fn pacman_summary(d: &Value) -> String {
    let p = &d["pacman"];
    if !js::truthy(p.get("syncOk")) {
        return "Package databases were not readable.".into();
    }
    let count = js::string(&p["dbCount"]);
    match s(p, "lastSync") {
        l if l.is_empty() => format!("{count} sync databases."),
        l => format!("{count} sync databases. Last sync {l}."),
    }
}

pub fn service_map_summary(map: &Value) -> String {
    let bits: Vec<String> = map
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(k, v)| {
            Some(state(Some(v)))
                .filter(|s| !s.is_empty())
                .map(|s| format!("{k} {s}"))
        })
        .collect();
    if bits.is_empty() {
        "No units reported.".into()
    } else {
        format!("{}.", bits.join(". "))
    }
}

fn section(title: &str, body: &str) -> String {
    let text = body.trim();
    let text = if text.is_empty() { "(none)" } else { text };
    format!("== {title} ==\n{text}\n")
}

/// `reportText(raw)`.
pub fn report_text(raw: Option<&Value>) -> String {
    let d = normalize(raw);
    let sentinels = ["look", "input", "autostart", "bindings", "windows"]
        .iter()
        .map(|n| {
            format!(
                "{n}={}",
                if js::truthy(d["atmos"]["sentinels"].get(*n)) {
                    "yes"
                } else {
                    "no"
                }
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let failed: Vec<String> = arr(d.get("failedUnits"))
        .iter()
        .filter(|r| js::truthy(r.get("unit")))
        .map(|r| {
            let mut line = format!("{} {}", s(r, "scope"), s(r, "unit"));
            let sub = s(r, "sub");
            if !sub.is_empty() {
                line.push_str(&format!(" {sub}"));
            }
            let desc = s(r, "description");
            if !desc.is_empty() {
                line.push_str(&format!(" - {desc}"));
            }
            line
        })
        .collect();
    let joined = |v: Option<&Value>| arr(v).iter().map(js::string).collect::<Vec<_>>().join("\n");
    let errors = arr(d["hyprland"].get("configErrors"));
    let n = |p: &str, k: &str| js::string(&d[p][k]);
    let disk = if num(&d["disk"], "total") > 0.0 {
        format!(
            "{} {}% used ({} / {} bytes)",
            n("disk", "path"),
            n("disk", "percent"),
            n("disk", "used"),
            n("disk", "total")
        )
    } else {
        String::new()
    };
    let memory = if num(&d["memory"], "total") > 0.0 {
        format!(
            "RAM used {} / {} bytes. Swap used {} / {} bytes.",
            n("memory", "used"),
            n("memory", "total"),
            n("memory", "swapUsed"),
            n("memory", "swapTotal")
        )
    } else {
        String::new()
    };
    let path = s(&d["omarchy"], "path");
    let recent = arr(d.get("recentErrors"));
    let generated = s(&d, "generatedAt");
    let host = s(&d, "hostname");
    let list = vec![
        "Atmos diagnostic report".to_owned(),
        if generated.is_empty() {
            String::new()
        } else {
            format!("Generated: {generated}")
        },
        if host.is_empty() {
            String::new()
        } else {
            format!("Hostname: {host}")
        },
        String::new(),
        section(
            "Omarchy",
            &format!(
                "{}{}",
                omarchy_summary(&d),
                if path.is_empty() {
                    String::new()
                } else {
                    format!("\nPath: {path}")
                }
            ),
        ),
        section(
            "Atmos",
            &format!("{}\nSentinels: {sentinels}", atmos_summary(&d)),
        ),
        section(
            "Hyprland",
            &format!(
                "{}{}",
                hypr_summary(&d),
                if errors.is_empty() {
                    String::new()
                } else {
                    format!("\n{}", joined(d["hyprland"].get("configErrors")))
                }
            ),
        ),
        section(
            "Failed systemd units",
            &if failed.is_empty() {
                "None.".to_owned()
            } else {
                failed.join("\n")
            },
        ),
        section("Disk", &disk),
        section("Memory", &memory),
        section("Kernel", &kernel_summary(&d)),
        section("GPU", &gpu_summary(&d)),
        section("Portals", &service_map_summary(&d["portals"])),
        section("PipeWire", &service_map_summary(&d["pipewire"])),
        section("Network", &network_summary(&d)),
        section("Package database", &pacman_summary(&d)),
        section(
            "Recent errors",
            &if recent.is_empty() {
                "None.".to_owned()
            } else {
                joined(d.get("recentErrors"))
            },
        ),
    ];
    let mut out: Vec<String> = Vec::new();
    for l in list {
        if l.is_empty() && out.last().is_some_and(String::is_empty) {
            continue;
        }
        out.push(l);
    }
    format!("{}\n", out.join("\n").trim_end())
}

/// `copyDiagnosticReport()` or `askAgentAboutDiagnostics()`: none without a report.
pub fn report_job(raw: Option<&Value>, action: &str) -> Option<Job> {
    let header = report_text(raw);
    (!header.is_empty()).then(|| {
        Job::new(
            "diag.report",
            json!({ "action": action, "header": header }),
            "diag-report",
            "The report",
        )
    })
}

/// The file Save suggests (`reportFileName`), stamped on the UTC clock: the window has no time
/// zone of its own to read.
pub fn report_file_name(hostname: &str, now_secs: u64) -> String {
    let host: String = clean_line(Some(&Value::from(hostname)), 32)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || "._-".contains(*c))
        .collect();
    let host = if host.is_empty() {
        "host".to_owned()
    } else {
        host
    };
    let days = (now_secs / 86_400) as i64;
    let secs = now_secs % 86_400;
    // Days to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "atmos-diagnostics-{host}-{year}{month:02}{day:02}-{:02}{:02}.txt",
        secs / 3600,
        secs % 3600 / 60
    )
}

/// Save: the path with `.txt`, and `saveDiagnosticReport(path)`'s job.
pub fn save_job(raw: Option<&Value>, path: &str) -> Option<(String, Job)> {
    if path.is_empty() {
        return None;
    }
    let path = if path.len() < 5 || !path.ends_with(".txt") {
        format!("{path}.txt")
    } else {
        path.to_owned()
    };
    if path.contains('\n') || !path.starts_with('/') {
        return None;
    }
    let header = report_text(raw);
    let args = json!({ "action": "save", "dest": path, "header": header });
    Some((
        path,
        Job::new("diag.report", args, "diag-report", "The report"),
    ))
}

/// What the stale settings row says: its label and description (`failedKeys`, `failedText`).
pub fn stale(failed: &[(String, String)]) -> (String, String) {
    let n = failed.len();
    let label = format!(
        "{n}{}",
        if n == 1 {
            " setting is stale"
        } else {
            " settings are stale"
        }
    );
    let lines: Vec<String> = failed
        .iter()
        .take(12)
        .map(|(k, why)| {
            let mut units = 0;
            let why: String = why
                .chars()
                .take_while(|c| {
                    units += c.len_utf16();
                    units <= 140
                })
                .collect();
            format!("{k}: {why}")
        })
        .collect();
    let more = if n > 12 {
        format!("\nand {} more", n - 12)
    } else {
        String::new()
    };
    let text = format!("{}{more}", lines.join("\n"));
    (label, text)
}
