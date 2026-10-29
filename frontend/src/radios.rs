//! What the Wi-Fi and Bluetooth pages read from `atmosd`'s scans (`wifi.scan`, `bluetooth.scan`,
//! docs/protocol.md), where the QML window reads Quickshell's: the nearby networks as
//! `RichUi.wifiRow` rows in `sortWifiRows` order, the nearby devices as `bluetoothLists`'
//! `discovered`, what each row says, and what Join, Disconnect, Forget and Pair send (`joinWifi`,
//! `connectEnterpriseWifi`, `deactivateWifiSsid`, `forgetWifiSsid`, `pairBluetoothDevice`).
//! Also the images `host.image` sends for the Accounts and Background pages.
//! `tests/catalog_radios.rs` holds these to the JavaScript and to the recorded replies.

use serde_json::{Value, json};

use crate::jobs::Job;
use crate::js;
use crate::protocol;

/// The kind of security a scanned network has, as the QML page's `securityKind` names
/// Quickshell's: nmcli's `SECURITY` column, which is empty for an open network.
pub fn wifi_kind(security: &str) -> &'static str {
    let s = security.trim();
    if s.is_empty() || s == "--" {
        "open"
    } else if s.contains("802.1X") {
        "enterprise"
    } else if s.contains("OWE") && !s.contains("WPA") {
        "owe"
    } else {
        "psk"
    }
}

/// `RichUi.wifiRow(...)` of each network the scan found, in `sortWifiRows` order: the one in
/// use, then the saved ones, then by signal.
pub fn wifi_rows(scan: &Value) -> Vec<Value> {
    let mut rows: Vec<Value> = scan["networks"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|n| js::truthy(n.get("ssid")))
        .map(|n| {
            let signal = js::number(n.get("signal"));
            let signal = if signal.is_finite() { signal } else { 0.0 };
            json!({
                "ssid": js::string(&n["ssid"]),
                "signal": js::json_number(js::round(signal)),
                "securityKind": wifi_kind(n["security"].as_str().unwrap_or_default()),
                "connected": js::truthy(n.get("inUse")),
                "known": js::truthy(n.get("known")),
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        let flag = |r: &Value, k: &str| r[k] == Value::Bool(true);
        flag(b, "connected")
            .cmp(&flag(a, "connected"))
            .then(flag(b, "known").cmp(&flag(a, "known")))
            .then(
                b["signal"]
                    .as_f64()
                    .unwrap_or(0.0)
                    .partial_cmp(&a["signal"].as_f64().unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    rows
}

/// A nearby network's words: connected or saved, its signal, and open or enterprise.
pub fn wifi_says(row: &Value) -> String {
    let mut bits = Vec::new();
    if row["connected"] == Value::Bool(true) {
        bits.push("Connected".to_owned());
    } else if row["known"] == Value::Bool(true) {
        bits.push("Saved".to_owned());
    }
    bits.push(format!("Signal {}%", js::string(&row["signal"])));
    match row["securityKind"].as_str() {
        Some("open") => bits.push("Open".into()),
        Some("enterprise") => bits.push("Enterprise".into()),
        _ => {}
    }
    format!("{}.", bits.join(". "))
}

/// What Join does with a row (`connectWifi`): ask for an identity and a password, ask for a
/// password, or join at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Enterprise,
    Password,
    Now,
}

/// `connectWifi(row)`'s choice.
pub fn join(row: &Value) -> Join {
    match row["securityKind"].as_str().unwrap_or_default() {
        "enterprise" => Join::Enterprise,
        "open" | "owe" => Join::Now,
        _ if row["known"] == Value::Bool(true) => Join::Now,
        _ => Join::Password,
    }
}

/// What a setter sends: a typed write, or a job.
#[derive(Debug, Clone, PartialEq)]
pub enum Sends {
    Write(Value),
    Job(Job),
}

fn bad(s: &str) -> bool {
    s.contains(['\r', '\n', '\0'])
}

/// `wifiUuidForSsid(ssid)`: the saved connection of that name.
fn uuid_for(ssid: &str, connections: Option<&Value>) -> String {
    connections
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|c| js::truthy(Some(c)) && crate::omafile::text_or_empty(c.get("name")) == ssid)
        .map_or_else(String::new, |c| {
            crate::omafile::text_or_empty(c.get("uuid"))
        })
}

fn uuid_ok(uuid: &str) -> bool {
    uuid.len() == 36 && uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn ssid_ok(ssid: &str) -> bool {
    !ssid.is_empty() && ssid.encode_utf16().count() <= 64 && !bad(ssid)
}

/// `wifi.connection` with `action` for a saved connection (`activateWifiConnection`, ...).
fn connection(action: &str, uuid: &str) -> Option<Sends> {
    uuid_ok(uuid).then(|| {
        Sends::Write(protocol::write(
            "wifi.connection",
            &json!({ "action": action, "uuid": uuid }),
        ))
    })
}

/// `joinWifi(ssid, password)`: a saved network comes up by its connection, any other is joined.
pub fn join_wifi(ssid: &str, password: &str, connections: Option<&Value>) -> Option<Sends> {
    if !ssid_ok(ssid) || ssid.starts_with('-') || bad(password) {
        return None;
    }
    if password.encode_utf16().count() > 256 {
        return None;
    }
    let uuid = uuid_for(ssid, connections);
    if !uuid.is_empty() && password.is_empty() {
        return connection("up", &uuid);
    }
    let args = if password.is_empty() {
        json!({ "ssid": ssid })
    } else {
        json!({ "ssid": ssid, "password": password })
    };
    Some(Sends::Job(Job::new(
        "wifi.join",
        args,
        "wifi-join",
        "Joining the network",
    )))
}

/// `connectEnterpriseWifi(ssid, identity, password)`.
pub fn join_enterprise(ssid: &str, identity: &str, password: &str) -> Option<Job> {
    let len = |s: &str| s.encode_utf16().count();
    let ok = !ssid.is_empty()
        && !identity.is_empty()
        && !password.is_empty()
        && len(ssid) <= 64
        && len(identity) <= 256
        && len(password) <= 256
        && ![ssid, identity, password].iter().any(|s| bad(s));
    ok.then(|| {
        Job::new(
            "wifi.enterprise",
            json!({ "ssid": ssid, "identity": identity, "password": password }),
            "wifi-enterprise",
            "Joining the network",
        )
    })
}

/// `deactivateWifiSsid(ssid)` and `forgetWifiSsid(ssid)`: by the saved connection when there is
/// one, else by name.
pub fn by_ssid(action: &str, ssid: &str, connections: Option<&Value>) -> Option<Sends> {
    if !ssid_ok(ssid) {
        return None;
    }
    let uuid = uuid_for(ssid, connections);
    if !uuid.is_empty() {
        return connection(if action == "down" { "down" } else { "delete" }, &uuid);
    }
    let action = if action == "down" {
        "down-ssid"
    } else {
        "delete-ssid"
    };
    Some(Sends::Write(protocol::write(
        "wifi.ssid",
        &json!({ "action": action, "ssid": ssid }),
    )))
}

/// The nearby devices a Bluetooth scan found (`bluetoothLists(devices).discovered`): neither
/// connected nor paired, each named, or called by its address.
pub fn bt_nearby(scan: &Value) -> Vec<Value> {
    scan["devices"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|d| js::truthy(Some(d)) && js::truthy(d.get("address")))
        .filter_map(|d| {
            let named = ["deviceName", "name"]
                .iter()
                .find_map(|k| d.get(*k).filter(|v| js::truthy(Some(v))))
                .map_or_else(String::new, |v| {
                    crate::omafile::trim(&js::string(v)).to_owned()
                });
            let name = if named.is_empty() {
                js::string(&d["address"])
            } else {
                named
            };
            let paired = ["paired", "bonded", "trusted"]
                .iter()
                .any(|k| js::truthy(d.get(*k)));
            let connected = js::truthy(d.get("connected"));
            (!name.is_empty() && !connected && !paired).then(|| {
                json!({ "address": js::string(&d["address"]), "name": name, "connected": false,
                    "paired": false })
            })
        })
        .collect()
}

/// `pairBluetoothDevice(address)`.
pub fn pair_write(address: &str) -> Option<Value> {
    crate::bluetooth::address_ok(address)
        .then(|| protocol::write("bluetooth.pair", &json!({ "address": address })))
}

/// The adapter's discoverable switch, which the QML window flips on Quickshell's adapter and
/// this one through `atmosd` (`bluetooth.discoverable`, read back as `bluetoothDiscoverable`).
pub fn discoverable_write(on: bool) -> Value {
    protocol::write("bluetooth.discoverable", &json!({ "on": on }))
}

/// The bytes `host.image` sent, none when there is no such file.
pub fn image_bytes(reply: &Value) -> Option<Vec<u8>> {
    decode_base64(reply["image"]["base64"].as_str()?)
}

/// Standard base64, padding optional; none for text that is not.
pub fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let value = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let body = text.trim().trim_end_matches('=');
    if body.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(body.len() * 3 / 4);
    for chunk in body.as_bytes().chunks(4) {
        let mut n = 0u32;
        for (i, c) in chunk.iter().enumerate() {
            n |= value(*c)? << (18 - 6 * i);
        }
        let bytes = n.to_be_bytes();
        out.extend_from_slice(&bytes[1..chunk.len()]);
    }
    Some(out)
}
