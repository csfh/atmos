//! What the radios can find right now: the Wi-Fi networks in range and the Bluetooth devices
//! nearby, for the Wi-Fi and Bluetooth pages, which the QML window read from Quickshell's
//! Networking and Bluetooth modules. Each is a read the window asks for while its page is shown
//! (a scan button, or on opening the page), never sampled on a clock: a rescan and a discovery
//! cost power and air time.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};

use crate::error::{Error, Kind, Result};
use crate::platform::Backend;
use crate::runner::Run;

/// How long a Bluetooth discovery runs when the window does not say, and the longest it may.
pub const DISCOVERY_SECONDS: u64 = 8;
const DISCOVERY_MAX: u64 = 30;

/// Where a fixture root keeps what the radios found, for tests.
fn fixture(root: &Path, name: &str, empty: Value) -> Result<Value> {
    let path = root.join(".local/state/omarchy/radios").join(name);
    if !path.is_file() {
        return Ok(empty);
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn run(program: &str, args: &[&str], secs: u64) -> Result<String> {
    Run::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .timeout(Duration::from_secs(secs))
        .checked()
        .map(|out| out.stdout_text())
        .map_err(|err| Error::new(Kind::Command, format!("{program} failed: {err}")))
}

// ---- Wi-Fi ---------------------------------------------------------------

/// The networks in range after a rescan, one row per name, strongest first, the one in use at
/// the top: its name, signal (0 to 100), security, whether it is in use and whether a saved
/// profile has its name.
pub fn wifi_scan(backend: Backend, root: Option<&Path>) -> Result<Value> {
    if backend == Backend::Plain {
        return Ok(json!({"networks": []}));
    }
    if let Some(root) = root {
        return fixture(root, "wifi.json", json!({"networks": []}));
    }
    let list = run(
        "nmcli",
        &[
            "-t",
            "-f",
            "IN-USE,SSID,SIGNAL,SECURITY",
            "device",
            "wifi",
            "list",
            "--rescan",
            "yes",
        ],
        30,
    )?;
    // A profile is named after its network unless the user renamed it.
    let saved = run(
        "nmcli",
        &["-t", "-f", "NAME,TYPE", "connection", "show"],
        10,
    )
    .unwrap_or_default();
    Ok(json!({"networks": networks(&list, &known(&saved))}))
}

/// The fields of one line of `nmcli -t`: split on `:`, with `\:` and `\\` escaped.
fn fields(line: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        let last = out.last_mut().expect("never empty");
        match c {
            '\\' => last.extend(chars.next()),
            ':' => out.push(String::new()),
            c => last.push(c),
        }
    }
    out
}

/// The names of the saved Wi-Fi profiles.
fn known(saved: &str) -> Vec<String> {
    saved
        .lines()
        .map(fields)
        .filter(|f| f.len() >= 2 && f[1] == "802-11-wireless")
        .map(|f| f[0].clone())
        .collect()
}

fn networks(list: &str, known: &[String]) -> Vec<Value> {
    struct Seen {
        signal: u8,
        security: String,
        in_use: bool,
    }
    let mut by_name: BTreeMap<String, Seen> = BTreeMap::new();
    for f in list.lines().map(fields) {
        if f.len() < 4 || f[1].is_empty() {
            continue; // a hidden network has no name to show
        }
        let signal = f[2].parse::<u8>().unwrap_or(0).min(100);
        let in_use = f[0].trim() == "*";
        let seen = by_name.entry(f[1].clone()).or_insert(Seen {
            signal: 0,
            security: String::new(),
            in_use: false,
        });
        if signal >= seen.signal {
            seen.signal = signal;
            seen.security = f[3].clone();
        }
        seen.in_use |= in_use;
    }
    let mut rows: Vec<(String, Seen)> = by_name.into_iter().collect();
    rows.sort_by(|(an, a), (bn, b)| {
        b.in_use
            .cmp(&a.in_use)
            .then(b.signal.cmp(&a.signal))
            .then(an.cmp(bn))
    });
    rows.into_iter()
        .map(|(ssid, s)| {
            json!({
                "ssid": ssid,
                "signal": s.signal,
                "security": s.security,
                "inUse": s.in_use,
                "known": known.contains(&ssid),
            })
        })
        .collect()
}

// ---- Bluetooth -----------------------------------------------------------

/// The adapter and the devices it knows after a discovery of `seconds` (1 to 30): every device
/// it has seen, paired or not, with whether it is paired and connected, and a connected
/// device's battery when it reports one. Discovery stops when the scan ends. With no adapter,
/// the adapter is null and there are no devices.
pub fn bluetooth_scan(backend: Backend, root: Option<&Path>, seconds: u64) -> Result<Value> {
    let none = json!({"adapter": null, "devices": []});
    if backend == Backend::Plain {
        return Ok(none);
    }
    if let Some(root) = root {
        return fixture(root, "bluetooth.json", none);
    }
    let Ok(show) = run("bluetoothctl", &["show"], 5) else {
        return Ok(none);
    };
    let Some(adapter) = adapter(&show) else {
        return Ok(none);
    };
    let seconds = seconds.clamp(1, DISCOVERY_MAX);
    // A powered-off adapter finds nothing, and says so on stderr; the list below is still true.
    let _ = Run::new("bluetoothctl")
        .args(["--timeout", &seconds.to_string(), "scan", "on"])
        .env("LC_ALL", "C")
        .timeout(Duration::from_secs(seconds + 5))
        .output();
    let all = run("bluetoothctl", &["devices"], 5)?;
    let paired = run("bluetoothctl", &["devices", "Paired"], 5).unwrap_or_default();
    let connected = run("bluetoothctl", &["devices", "Connected"], 5).unwrap_or_default();
    let connected = device_list(&connected);
    let batteries: BTreeMap<String, u8> = connected
        .iter()
        .filter_map(|(address, _)| {
            let info = run("bluetoothctl", &["info", address], 5).ok()?;
            Some((address.clone(), battery(&info)?))
        })
        .collect();
    let adapter = json!(adapter);
    Ok(json!({
        "adapter": adapter,
        "devices": devices(&all, &paired, &connected, &batteries),
    }))
}

/// A controller's address, name and switches, from `bluetoothctl show`.
fn adapter(show: &str) -> Option<BTreeMap<&'static str, Value>> {
    let first = show.lines().next()?;
    let address = first.strip_prefix("Controller ")?.split(' ').next()?;
    let mut a = BTreeMap::new();
    a.insert("address", json!(address));
    for line in show.lines().skip(1) {
        let Some((key, value)) = line.trim().split_once(": ") else {
            continue;
        };
        let yes = value == "yes";
        match key {
            "Alias" => a.insert("name", json!(value)),
            "Powered" => a.insert("powered", json!(yes)),
            "Discoverable" => a.insert("discoverable", json!(yes)),
            "Pairable" => a.insert("pairable", json!(yes)),
            "Discovering" => a.insert("discovering", json!(yes)),
            _ => None,
        };
    }
    Some(a)
}

/// `Device <address> <name>` lines: each address and its name, the address when it has none.
fn device_list(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("Device ")?;
            let (address, name) = rest.split_once(' ').unwrap_or((rest, ""));
            if address.len() != 17 || address.split(':').count() != 6 {
                return None;
            }
            let name = if name.is_empty() { address } else { name };
            Some((address.to_string(), name.to_string()))
        })
        .collect()
}

/// `Battery Percentage: 0x55 (85)` in `bluetoothctl info`.
fn battery(info: &str) -> Option<u8> {
    let line = info
        .lines()
        .find_map(|l| l.trim().strip_prefix("Battery Percentage:"))?;
    let inner = line.split_once('(')?.1.split_once(')')?.0;
    inner.trim().parse::<u8>().ok().map(|n| n.min(100))
}

fn devices(
    all: &str,
    paired: &str,
    connected: &[(String, String)],
    batteries: &BTreeMap<String, u8>,
) -> Vec<Value> {
    let paired: Vec<String> = device_list(paired).into_iter().map(|d| d.0).collect();
    let mut rows: Vec<(String, String, bool, bool)> = device_list(all)
        .into_iter()
        .map(|(address, name)| {
            let on = connected.iter().any(|(a, _)| *a == address);
            let is_paired = paired.contains(&address);
            (address, name, is_paired, on)
        })
        .collect();
    rows.sort_by(|a, b| b.3.cmp(&a.3).then(b.2.cmp(&a.2)).then(a.1.cmp(&b.1)));
    rows.into_iter()
        .map(|(address, name, paired, connected)| {
            json!({
                "address": address,
                "name": name,
                "paired": paired,
                "connected": connected,
                "battery": batteries.get(&address),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nmcli_fields_keep_escaped_colons() {
        assert_eq!(
            fields(r"*:Cafe\: Wi-Fi:80:WPA2"),
            ["*", "Cafe: Wi-Fi", "80", "WPA2"]
        );
        assert_eq!(fields(r" :a\\b::"), [" ", r"a\b", "", ""]);
    }

    #[test]
    fn networks_are_one_row_per_name_the_one_in_use_first() {
        let list = "\
 :Home:60:WPA2
*:Home:72:WPA2
 :Cafe\\: Free:85:
 ::90:WPA2
 :Office:40:WPA2 802.1X
";
        let rows = networks(list, &["Office".into()]);
        let names: Vec<&str> = rows.iter().map(|r| r["ssid"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            ["Home", "Cafe: Free", "Office"],
            "hidden one left out"
        );
        assert_eq!(rows[0]["signal"], 72, "the strongest of its access points");
        assert_eq!(rows[0]["inUse"], true);
        assert_eq!(rows[1]["security"], "", "an open network");
        assert_eq!(rows[2]["known"], true);
        assert_eq!(rows[0]["known"], false);
    }

    #[test]
    fn saved_wifi_profiles_are_known() {
        let saved = "Home:802-11-wireless\nWired connection 1:802-3-ethernet\nCafe\\: Free:802-11-wireless\n";
        assert_eq!(known(saved), ["Home", "Cafe: Free"]);
    }

    #[test]
    fn the_adapter_is_read_from_show() {
        let show = "Controller 00:1A:7D:DA:71:13 (public)\n\tName: host\n\tAlias: Laptop\n\tPowered: yes\n\tDiscoverable: no\n\tPairable: yes\n\tDiscovering: no\n";
        let a = adapter(show).unwrap();
        assert_eq!(a["address"], "00:1A:7D:DA:71:13");
        assert_eq!(a["name"], "Laptop");
        assert_eq!(
            (&a["powered"], &a["discoverable"]),
            (&json!(true), &json!(false))
        );
        assert!(adapter("No default controller available\n").is_none());
    }

    #[test]
    fn devices_say_paired_connected_and_battery() {
        let all = "Device AA:BB:CC:DD:EE:01 Headphones\nDevice AA:BB:CC:DD:EE:02 Phone\nDevice AA:BB:CC:DD:EE:03\nnot a device\n";
        let paired = "Device AA:BB:CC:DD:EE:01 Headphones\nDevice AA:BB:CC:DD:EE:02 Phone\n";
        let connected = device_list("Device AA:BB:CC:DD:EE:01 Headphones\n");
        let batteries = BTreeMap::from([("AA:BB:CC:DD:EE:01".to_string(), 85)]);
        let rows = devices(all, paired, &connected, &batteries);
        let names: Vec<&str> = rows.iter().map(|r| r["name"].as_str().unwrap()).collect();
        assert_eq!(names, ["Headphones", "Phone", "AA:BB:CC:DD:EE:03"]);
        assert_eq!(
            (&rows[0]["connected"], &rows[0]["battery"]),
            (&json!(true), &json!(85))
        );
        assert_eq!(
            (&rows[1]["paired"], &rows[1]["battery"]),
            (&json!(true), &Value::Null)
        );
        assert_eq!(
            rows[2]["paired"], false,
            "found by the discovery, not paired"
        );
    }

    #[test]
    fn the_battery_is_read_from_info() {
        assert_eq!(
            battery("\tPaired: yes\n\tBattery Percentage: 0x55 (85)\n"),
            Some(85)
        );
        assert_eq!(battery("\tPaired: yes\n"), None);
    }
}
