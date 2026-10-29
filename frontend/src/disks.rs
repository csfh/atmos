//! The Disks page's logic (`pages/DisksPage.qml`): each drive's title and the mount its speed
//! test runs on, the run and what it says, Snapper's summary and snapshots, hibernation, swap,
//! and the writes and jobs its controls send, as `Omarchy.setSnapperNumberLimit`,
//! `setSnapperTimeline`, `setFstrim`, `restoreSnapshot`, `setupHibernation`, `removeHibernation`
//! and `changeDrivePassword` make them. The Snapper and TRIM writes run as root.
//! `tests/catalog_disks.rs` runs the page's code, and `tests/catalog_setters.rs` the setters,
//! under Node against this.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::jobs::Job;
use crate::{js, protocol};

fn s(v: &Value, name: &str) -> String {
    v.get(name)
        .filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

/// `diskTitle(disk)`: its model, else its name.
pub fn title(disk: &Value) -> String {
    let model = s(disk, "model");
    if !model.is_empty() {
        return model;
    }
    match s(disk, "name") {
        n if n.is_empty() => "disk".into(),
        n => n,
    }
}

/// `speedtestDir(disk)`: the first mount that is not `/boot`.
pub fn speedtest_dir(disk: &Value) -> String {
    disk.get("mounts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|m| s(m, "target"))
        .find(|t| !t.is_empty() && t != "/boot" && !t.starts_with("/boot/"))
        .unwrap_or_default()
}

/// `RichUi.validMountPath`.
pub fn valid_mount(dir: &str) -> bool {
    dir.starts_with('/')
        && !dir.contains("..")
        && dir
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._/-".contains(c))
}

/// One drive's last result.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Result {
    pub name: String,
    pub read: String,
    pub write: String,
    pub error: String,
}

/// The page's speed test (`diskRunning`, `diskTarget`, ...), and each drive's last result.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Run {
    pub running: bool,
    pub token: u64,
    pub name: String,
    pub read: String,
    pub write: String,
    pub error: String,
    pub target: String,
    pub results: BTreeMap<String, Result>,
}

/// `parseDiskSpeedLine`: `disk NAME`, `read N` or `write N`.
pub fn parse_line(line: &str) -> Option<(&'static str, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    match parts[0] {
        "disk" => Some(("disk", parts[1..].join(" "))),
        kind @ ("read" | "write") => {
            let n = crate::speedtest::parse_mbps(parts[1]);
            let kind = if kind == "read" { "read" } else { "write" };
            n.is_finite()
                .then(|| (kind, js::string(&js::json_number(n))))
        }
        _ => None,
    }
}

impl Run {
    /// `startDiskSpeedtest(dir)`: the request, none while a run is going.
    pub fn start(&mut self, dir: &str) -> Option<Value> {
        if self.running {
            return None;
        }
        self.error.clear();
        self.name.clear();
        self.read.clear();
        self.write.clear();
        self.running = true;
        self.target = dir.to_owned();
        if !dir.is_empty() {
            self.results.insert(dir.to_owned(), Result::default());
        }
        self.token += 1;
        let mut body = json!({ "op": "speedtest.disk" });
        if valid_mount(dir) {
            body["dir"] = Value::from(dir);
        }
        Some(body)
    }

    /// `stopDiskSpeedtest`.
    pub fn stop(&mut self) {
        self.token += 1;
        self.running = false;
    }

    fn result(&mut self) -> Option<&mut Result> {
        (!self.target.is_empty()).then(|| self.results.entry(self.target.clone()).or_default())
    }

    /// `finishDiskSpeedtest`: the reply's result, or why there is none.
    pub fn finish(&mut self, reply: std::result::Result<&Value, &str>) {
        if let Ok(r) = reply {
            let text = s(r, "stdout");
            for line in text.split('\n') {
                let Some((kind, value)) = parse_line(line) else {
                    continue;
                };
                match kind {
                    "disk" => self.name = value.clone(),
                    "read" => self.read = value.clone(),
                    _ => self.write = value.clone(),
                }
                if let Some(res) = self.result() {
                    match kind {
                        "disk" => res.name = value,
                        "read" => res.read = value,
                        _ => res.write = value,
                    }
                }
            }
        }
        self.running = false;
        let failed = match reply {
            Ok(r) => js::number(r.get("exit")) != 0.0,
            Err(_) => true,
        };
        let error = if failed {
            let why = match reply {
                Ok(r) if js::truthy(r.get("stderr")) => js::string(&r["stderr"]),
                Ok(_) => String::new(),
                Err(why) => why.to_owned(),
            };
            match why.trim() {
                "" => "Disk speed test failed".to_owned(),
                w => w.to_owned(),
            }
        } else {
            String::new()
        };
        self.error = error.clone();
        if let Some(res) = self.result() {
            res.error = error;
        }
    }

    fn here(&self, dir: &str) -> bool {
        self.running && self.target == dir
    }

    fn last(&self, dir: &str) -> Option<&Result> {
        if dir.is_empty() {
            None
        } else {
            self.results.get(dir)
        }
    }

    /// The Speed test row's description, for the drive whose mount is `dir`.
    pub fn says(&self, dir: &str) -> String {
        if self.here(dir) {
            return if self.name.is_empty() {
                "Reading, then writing, for about eight seconds each.".into()
            } else {
                format!("Testing {}.", self.name)
            };
        }
        match self.last(dir) {
            Some(l) if !l.read.is_empty() || !l.write.is_empty() => {
                let mut bits = Vec::new();
                if !l.name.is_empty() {
                    bits.push(l.name.clone());
                }
                bits.push("last run".into());
                format!("{}.", bits.join(" · "))
            }
            _ => "Read for about eight seconds, then write for about eight, on this disk.".into(),
        }
    }

    /// Run now or Cancel.
    pub fn button(&self, dir: &str) -> &'static str {
        if self.here(dir) { "Cancel" } else { "Run now" }
    }

    /// The rates: of the run while it goes on this drive, else of its last run.
    pub fn rates(&self, dir: &str) -> Option<String> {
        let dash = |v: &str| {
            if v.is_empty() {
                "—".to_owned()
            } else {
                v.to_owned()
            }
        };
        if self.here(dir) {
            return Some(format!(
                "Read {} MB/s   Write {} MB/s",
                dash(&self.read),
                dash(&self.write)
            ));
        }
        let l = self
            .last(dir)
            .filter(|l| !l.read.is_empty() || !l.write.is_empty())?;
        Some(format!(
            "Read {} MB/s   Write {} MB/s",
            dash(&l.read),
            dash(&l.write)
        ))
    }

    /// What went wrong on this drive.
    pub fn error(&self, dir: &str) -> String {
        if self.target == dir && !self.error.is_empty() {
            return self.error.clone();
        }
        self.last(dir).map_or_else(String::new, |l| l.error.clone())
    }
}

/// `snapperSummary()`.
pub fn snapper_summary(configs: Option<&Value>) -> String {
    let list = configs.and_then(Value::as_array).filter(|l| !l.is_empty());
    let Some(list) = list else {
        return "Snapper is installed, but no configs were readable.".into();
    };
    let parts: Vec<String> = list
        .iter()
        .filter_map(|c| {
            let (name, sub) = (s(c, "name"), s(c, "subvolume"));
            (!name.is_empty()).then(|| {
                if sub.is_empty() {
                    name.clone()
                } else {
                    format!("{name} on {sub}")
                }
            })
        })
        .collect();
    format!("Configs: {}.", parts.join(", "))
}

/// `snapshotLabel(item)`.
pub fn snapshot_label(item: &Value) -> String {
    if !js::truthy(Some(item)) {
        return "Snapshot".into();
    }
    let id = item
        .get("id")
        .map_or_else(|| "undefined".to_owned(), js::string);
    let mut bits = vec![format!("#{id}")];
    for (name, wrap) in [("date", false), ("description", false), ("config", true)] {
        let v = s(item, name);
        if !v.is_empty() {
            bits.push(if wrap { format!("({v})") } else { v });
        }
    }
    bits.join("  ")
}

/// The Hibernation row's description, by what runs and what the machine says.
pub fn hibernation_says(running: Option<&str>, configured: bool, supported: bool) -> &'static str {
    match running {
        Some("hibernation-remove") => "Removing hibernation…",
        Some("hibernation-setup") => "Setting up hibernation…",
        _ if configured => "Set up. Remove deletes the swap file and the boot resume settings.",
        _ if supported => {
            "This machine can hibernate. Set up writes a swap file and the boot resume settings."
        }
        _ => "Not available on this machine. Set up stays disabled.",
    }
}

/// A swap device's row: its label, and what it says.
pub fn swap(item: &Value) -> (String, String) {
    let label = match (s(item, "label"), s(item, "name")) {
        (l, _) if !l.is_empty() => l,
        (_, n) if !n.is_empty() => n,
        _ => "swap".into(),
    };
    let size = crate::hardware::format_bytes(js::number(item.get("size")));
    let path = s(item, "path");
    let says = if path.is_empty() {
        size
    } else {
        format!("{path}. {size}.")
    };
    (label, says)
}

/// What Copy copies of a swap device.
pub fn swap_copy(item: &Value) -> String {
    ["path", "name", "label"]
        .iter()
        .map(|n| s(item, n))
        .find(|t| !t.is_empty())
        .unwrap_or_default()
}

/// `setSnapperNumberLimit(n)`: none when it is the limit already, or out of 1 to 50.
pub fn number_limit(n: f64, current: Option<&Value>) -> Option<Value> {
    let n = js::round(n);
    let same = current.and_then(Value::as_f64) == Some(n);
    (n.is_finite() && (1.0..=50.0).contains(&n) && !same)
        .then(|| protocol::write("snapper.number-limit", &json!({ "n": js::json_number(n) })))
}

/// `setSnapperTimeline(on)` and `setFstrim(on)`: none when it is so already.
pub fn flag(name: &str, on: bool, current: Option<&Value>) -> Option<Value> {
    (current != Some(&Value::Bool(on))).then(|| protocol::write(name, &json!({ "on": on })))
}

/// `restoreSnapshot(config, id)`.
pub fn rollback(config: &str, id: &str) -> Option<Job> {
    let word = |t: &str| {
        !t.is_empty()
            && t.chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
    };
    let digits = !id.is_empty() && id.chars().all(|c| c.is_ascii_digit());
    (word(config) && digits).then(|| {
        Job::new(
            "snapshot.rollback",
            json!({ "config": config, "id": id }),
            "snapshot-rollback",
            "The rollback",
        )
        .sudo()
    })
}

/// `setupHibernation()` or `removeHibernation()`.
pub fn hibernation(remove: bool) -> Job {
    let (name, kind) = if remove {
        ("hibernation.remove", "hibernation-remove")
    } else {
        ("hibernation.setup", "hibernation-setup")
    };
    Job::new(name, json!({}), kind, "Hibernation").sudo()
}

/// The encryption dialog's check: what is wrong, else the job (`changeDrivePassword`).
pub fn change_key(
    device: &str,
    current: &str,
    new: &str,
    again: &str,
) -> std::result::Result<Option<Job>, &'static str> {
    if current.is_empty() || new.is_empty() {
        return Err("Passphrases cannot be empty.");
    }
    if new != again {
        return Err("New passphrases do not match.");
    }
    let ok = device.starts_with('/') && !device.contains("..");
    let args = json!({ "device": device, "current": current, "new": new });
    Ok(ok.then(|| Job::new("luks.change-key", args, "luks", "The passphrase change").sudo()))
}

impl Run {
    /// What the page keeps of it.
    pub fn to_value(&self) -> Value {
        let results: serde_json::Map<String, Value> = self
            .results
            .iter()
            .map(|(k, r)| (k.clone(), json!([r.name, r.read, r.write, r.error])))
            .collect();
        json!({
            "running": self.running, "token": self.token, "name": self.name, "read": self.read,
            "write": self.write, "error": self.error, "target": self.target, "results": results,
        })
    }

    pub fn of(v: Option<&Value>) -> Run {
        let Some(v) = v else {
            return Run::default();
        };
        let t = |n: &str| v[n].as_str().unwrap_or_default().to_owned();
        let results = v["results"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(k, r)| {
                let f = |i: usize| r[i].as_str().unwrap_or_default().to_owned();
                (
                    k.clone(),
                    Result {
                        name: f(0),
                        read: f(1),
                        write: f(2),
                        error: f(3),
                    },
                )
            })
            .collect();
        Run {
            running: v["running"] == Value::Bool(true),
            token: v["token"].as_u64().unwrap_or(0),
            name: t("name"),
            read: t("read"),
            write: t("write"),
            error: t("error"),
            target: t("target"),
            results,
        }
    }
}
