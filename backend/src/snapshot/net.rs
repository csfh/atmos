//! Network, Wi-Fi, Bluetooth, Tailscale and audio.

use super::sh::{is_ident, is_uint, ok, omarchy, out, out_within, present};
use crate::runner::Run;
use regex_lite::Regex;
use serde_json::{json, Map, Value};
use std::time::Duration;

fn tab_field(text: &str, key: &str) -> String {
    text.lines()
        .find_map(|l| {
            let mut cols = l.split('\t');
            (cols.next() == Some(key)).then(|| cols.next().unwrap_or("").to_string())
        })
        .unwrap_or_default()
}

fn first_line_col(text: &str, col: usize) -> String {
    text.lines()
        .next()
        .and_then(|l| l.split('\t').nth(col))
        .unwrap_or("")
        .to_string()
}

fn valid_ip(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
}

fn nmcli(args: &[&str]) -> String {
    match Run::new("nmcli")
        .args(args)
        .env("LC_ALL", "C")
        .timeout(Duration::from_secs(30))
        .output()
    {
        Ok(o) => o.stdout_text().trim_end_matches('\n').to_string(),
        Err(_) => String::new(),
    }
}

pub fn dns() -> String {
    let dns = omarchy(&["dns"]);
    if ["Cloudflare", "Google", "DHCP", "Custom"].contains(&dns.as_str()) {
        dns
    } else {
        String::new()
    }
}

pub fn bluetooth() -> bool {
    ok("omarchy", &["bluetooth", "power", "is-on"])
}

/// The first non-empty line of an nmcli field list.
fn first_nonempty(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .to_string()
}

pub fn network() -> Map<String, Value> {
    let mut f = Map::new();
    let band_out = omarchy(&["network", "band"]);
    let (mut connected, mut band, mut selected, mut bands) =
        (false, String::new(), "auto".to_string(), json!(["auto"]));
    if !band_out.is_empty() {
        connected = true;
        band = tab_field(&band_out, "band");
        selected = tab_field(&band_out, "selected");
        let avail = tab_field(&band_out, "available");
        if !["auto", "2.4", "5", "6"].contains(&selected.as_str()) {
            selected = "auto".into();
        }
        let ok_band = |b: &str| matches!(b, "2.4" | "5" | "6");
        let mut list = vec!["auto".to_string()];
        list.extend(avail.split(' ').filter(|b| ok_band(b)).map(String::from));
        if selected != "auto" && ok_band(&selected) && !list.contains(&selected) {
            list.push(selected.clone());
        }
        bands = json!(list);
    }
    let mut iface = String::new();
    if connected {
        let status = nmcli(&["-e", "no", "-g", "DEVICE,TYPE,STATE", "device", "status"]);
        iface = status
            .lines()
            .find_map(|l| {
                let c: Vec<&str> = l.split(':').collect();
                (c.len() >= 3 && c[1] == "wifi" && c[2] == "connected").then(|| c[0].to_string())
            })
            .unwrap_or_default();
        if !is_ident(&iface) {
            iface.clear();
        }
    }
    let status = omarchy(&["network", "status"]);
    let mut kind = first_line_col(&status, 0);
    let (mut net_iface, mut ssid, mut signal) = (String::new(), String::new(), String::new());
    match kind.as_str() {
        "ethernet" => net_iface = first_line_col(&status, 1),
        "wifi" => {
            ssid = first_line_col(&status, 1);
            signal = first_line_col(&status, 2);
        }
        _ => kind = "disconnected".into(),
    }
    if !is_ident(&net_iface) {
        net_iface.clear();
    }
    if !is_uint(&signal) {
        signal.clear();
    }
    let (mut ip, mut speed) = (String::new(), String::new());
    let verbose = omarchy(&["network", "status", "--verbose"]);
    if !verbose.is_empty() {
        if net_iface.is_empty() {
            net_iface = tab_field(&verbose, "iface");
        }
        ip = tab_field(&verbose, "ip");
        speed = tab_field(&verbose, "speed");
    }
    if !is_ident(&net_iface) {
        net_iface.clear();
    }
    if !valid_ip(&ip) {
        ip.clear();
    }
    if !is_uint(&speed) {
        speed.clear();
    }
    let (mut gateway, mut dns_servers) = (String::new(), json!([]));
    if present("nmcli") {
        gateway = first_nonempty(&out("nmcli", &["-g", "IP4.GATEWAY", "device", "show"]));
        dns_servers = json!(out("nmcli", &["-g", "IP4.DNS", "device", "show"])
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>());
    }
    let radio = out("nmcli", &["radio", "wifi"]) == "enabled";
    let hw = out("nmcli", &["-t", "-f", "WIFI-HW", "radio"]) == "enabled";
    f.insert("dns".into(), json!(dns()));
    f.insert("bluetooth".into(), json!(bluetooth()));
    f.insert("wifiConnected".into(), json!(connected));
    f.insert("wifiBand".into(), json!(band));
    f.insert("wifiBandSelected".into(), json!(selected));
    f.insert("wifiBands".into(), bands);
    f.insert("wifiIface".into(), json!(iface));
    f.insert("netKind".into(), json!(kind));
    f.insert("netIface".into(), json!(net_iface));
    f.insert("netSsid".into(), json!(ssid));
    f.insert("netSignal".into(), json!(signal));
    f.insert("netIp".into(), json!(ip));
    f.insert("netGateway".into(), json!(gateway));
    f.insert("netDnsServers".into(), dns_servers);
    f.insert("netSpeed".into(), json!(speed));
    f.insert("wifiHw".into(), json!(hw));
    f.insert("wifiRadio".into(), json!(radio));
    f.insert("wifiConnections".into(), wifi_connections());
    f.insert("bluetoothDevices".into(), bluetooth_devices());
    let (installed, running, peers) = tailscale();
    f.insert("tailscaleInstalled".into(), json!(installed));
    f.insert("tailscaleRunning".into(), json!(running));
    f.insert("tailscalePeers".into(), peers);
    f
}

fn wifi_connections() -> Value {
    if !present("nmcli") {
        return json!([]);
    }
    let uuid = Regex::new(r"^[0-9a-fA-F-]{36}$").expect("regex");
    let raw = nmcli(&["-t", "-f", "UUID,TYPE,STATE,NAME", "connection", "show"]);
    let rows: Vec<Value> = raw
        .lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split(':').collect();
            if cols.len() < 4 || cols[1] != "802-11-wireless" || !uuid.is_match(cols[0]) {
                return None;
            }
            let name = cols[3..].join(":").replace("\\:", ":");
            Some(json!({"uuid": cols[0], "active": cols[2] == "activated", "name": name}))
        })
        .collect();
    json!(rows)
}

fn bluetooth_devices() -> Value {
    if !present("bluetoothctl") {
        return json!([]);
    }
    let paired = out_within("bluetoothctl", &["devices", "Paired"], 2);
    let connected = out_within("bluetoothctl", &["devices", "Connected"], 2);
    let line = Regex::new(r"^Device ([0-9A-Fa-f:]{17})(?: (.*))?$").expect("regex");
    let parse = |text: &str| -> Vec<(String, String)> {
        text.lines()
            .filter(|l| l.starts_with("Device "))
            .filter_map(|l| {
                let m = line.captures(l)?;
                let address = m[1].to_string();
                let name = m.get(2).map_or("", |n| n.as_str());
                Some((
                    address.clone(),
                    if name.is_empty() {
                        address
                    } else {
                        name.to_string()
                    },
                ))
            })
            .collect()
    };
    let on: Vec<String> = parse(&connected).into_iter().map(|d| d.0).collect();
    json!(parse(&paired)
        .into_iter()
        .map(|(address, name)| json!({"address": address, "name": name, "connected": on.contains(&address)}))
        .collect::<Vec<_>>())
}

pub fn tailscale() -> (bool, bool, Value) {
    let mut installed = present("tailscale");
    let mut running = false;
    if present("systemctl") && out("systemctl", &["is-active", "tailscaled.service"]) == "active" {
        running = true;
        installed = true;
    }
    let mut peers = json!([]);
    if running && present("tailscale") {
        let raw = out("tailscale", &["status", "--json"]);
        if let Ok(doc) = serde_json::from_str::<Value>(&raw) {
            let rows: Vec<Value> = doc
                .get("Peer")
                .and_then(Value::as_object)
                .map(|m| m.values().collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| {
                    let name = match p.get("HostName") {
                        Some(v) if !v.is_null() && *v != Value::Bool(false) => super::sh::jq_text(v),
                        _ => p.get("DNSName").and_then(Value::as_str).unwrap_or("").split('.').next().unwrap_or("").to_string(),
                    };
                    (!name.is_empty()).then(|| json!({"name": name, "online": p.get("Online") == Some(&Value::Bool(true))}))
                })
                .collect();
            peers = json!(rows);
        }
    }
    (installed, running, peers)
}

fn percent_of(sink: &Value) -> i64 {
    let values: Vec<f64> = sink
        .get("volume")
        .and_then(Value::as_object)
        .map(|v| {
            v.values()
                .map(|c| {
                    c.get("value_percent")
                        .and_then(Value::as_str)
                        .map_or(0.0, |s| {
                            s.trim_end_matches('%').parse::<f64>().unwrap_or(0.0)
                        })
                })
                .collect()
        })
        .unwrap_or_default();
    if values.is_empty() {
        0
    } else {
        (values.iter().sum::<f64>() / values.len() as f64).floor() as i64
    }
}

fn audio_rows(raw: &str, default: &str, skip_monitors: bool) -> Vec<Value> {
    let Ok(Value::Array(devices)) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    devices
        .iter()
        .filter(|d| !(skip_monitors && d.get("name").and_then(Value::as_str).unwrap_or("").ends_with(".monitor")))
        .map(|d| {
            let name = d.get("name").cloned().unwrap_or(Value::Null);
            let id = d
                .pointer("/properties/object.id")
                .map(super::sh::jq_text)
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            json!({
                "id": id,
                "name": name,
                "description": d.get("description").filter(|v| !v.is_null()).cloned().unwrap_or_else(|| name.clone()),
                "default": name.as_str() == Some(default),
                "volume": percent_of(d),
                "muted": d.get("mute") == Some(&Value::Bool(true)),
            })
        })
        .collect()
}

pub fn audio() -> Map<String, Value> {
    let mut f = Map::new();
    let (mut sinks, mut sources) = (Vec::new(), Vec::new());
    if present("pactl") {
        let default_sink = out("pactl", &["get-default-sink"]);
        let default_source = out("pactl", &["get-default-source"]);
        sinks = audio_rows(
            &out("pactl", &["-f", "json", "list", "sinks"]),
            &default_sink,
            false,
        );
        sources = audio_rows(
            &out("pactl", &["-f", "json", "list", "sources"]),
            &default_source,
            true,
        );
    }
    let pick = |rows: &[Value]| -> (i64, bool) {
        let chosen = rows.iter().find(|r| r["default"] == true);
        let volume = chosen
            .or_else(|| rows.first())
            .and_then(|r| r["volume"].as_i64())
            .unwrap_or(0);
        (volume, chosen.is_some_and(|r| r["muted"] == true))
    };
    let (out_volume, out_muted) = pick(&sinks);
    let (in_volume, in_muted) = pick(&sources);
    let tuning = omarchy(&["audio", "tuning", "status"]);
    f.insert("audioSinks".into(), json!(sinks));
    f.insert("audioSources".into(), json!(sources));
    f.insert("audioOutputVolume".into(), json!(out_volume));
    f.insert("audioOutputMuted".into(), json!(out_muted));
    f.insert("audioInputVolume".into(), json!(in_volume));
    f.insert("audioInputMuted".into(), json!(in_muted));
    f.insert(
        "audioTuningMatch".into(),
        json!(ok("omarchy", &["audio", "tuning", "match"])),
    );
    let after = |marker: &str, word: &str| {
        tuning
            .find(marker)
            .is_some_and(|i| tuning[i + marker.len()..].contains(word))
    };
    let on = after("Tuning sink:", "present") || after("Installed:", "yes");
    f.insert("audioTuningOn".into(), json!(on));
    f
}
