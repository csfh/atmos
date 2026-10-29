//! The Wi-Fi page's logic (`pages/network/WifiPage.qml`, `NetworkPrefs.js`, `RichUi.js`'s QR
//! parser): the QR code the `wifi.qr` job prints, each saved connection's words, and the writes
//! its controls and the static IPv4 dialog send (`fetchWifiQr`, `copyWifiPassword`,
//! `setConnectionMetered`, `setConnectionPriority`, `setConnectionMac`, `setConnectionIpv4`).
//! Nearby networks, which the QML window reads from Quickshell's scanner, are `radios.rs`'s.
//! `tests/catalog_wifi.rs` runs these under Node.

use serde_json::{Map, Value, json};

use crate::jobs::Job;
use crate::protocol;

fn iface_ok(iface: &str) -> bool {
    !iface.is_empty()
        && iface
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

/// `fetchWifiQr()`: the job, with the interface when it is one.
pub fn qr_job(iface: &str) -> Job {
    let args = if iface_ok(iface) {
        json!({ "iface": iface })
    } else {
        json!({})
    };
    Job::new("wifi.qr", args, "wifi-qr", "The Wi-Fi QR code")
}

/// `copyWifiPassword()`.
pub fn password_copy(iface: &str) -> Option<Value> {
    iface_ok(iface).then(|| protocol::write("wifi.password-copy", &json!({ "iface": iface })))
}

/// A QR code: the network it is for, and its modules row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qr {
    pub ssid: String,
    pub rows: Vec<Vec<bool>>,
}

/// `parseQrOutput(raw)`: the code, or why there is none.
pub fn parse_qr(raw: &str) -> Result<Qr, &'static str> {
    let text = raw.replace('\r', "");
    let mut lines = text.split('\n').peekable();
    let mut ssid = String::new();
    if let Some(first) = lines.peek()
        && first.starts_with("meta\t")
    {
        ssid = first.split('\t').skip(3).collect::<Vec<_>>().join("\t");
        lines.next();
    }
    let rows: Vec<Vec<bool>> = lines
        .filter(|l| !l.is_empty() && l.chars().all(|c| c == '0' || c == '1'))
        .map(|l| l.chars().map(|c| c == '1').collect())
        .collect();
    let Some(size) = rows.first().map(Vec::len) else {
        return Err("Could not read a Wi-Fi QR matrix");
    };
    if rows.iter().any(|r| r.len() != size) {
        return Err("QR matrix rows are uneven");
    }
    Ok(Qr { ssid, rows })
}

/// `applyWifiQr`'s error for a job that failed: what it printed, or a reason.
pub fn qr_failed(err: &str) -> String {
    match err.trim() {
        "" => "Could not build a QR code".into(),
        e => e.into(),
    }
}

/// The QR code row's words.
pub fn qr_says(ssid: &str) -> String {
    if ssid.is_empty() {
        "A scannable code for the network you are on, so someone nearby can join.".into()
    } else {
        format!("A scannable code for {ssid}.")
    }
}

/// `sanitizeUuid`.
fn uuid(raw: &str) -> String {
    let t = raw.trim();
    let groups: Vec<&str> = t.split('-').collect();
    let ok = groups.len() == 5
        && groups
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(g, n)| g.len() == n && g.chars().all(|c| c.is_ascii_hexdigit()));
    if ok { t.into() } else { String::new() }
}

/// `opFor("metered" | "priority" | "mac", { uuid, value })`.
pub fn connection_write(what: &str, raw_uuid: &str, value: &Value) -> Option<Value> {
    let id = uuid(raw_uuid);
    if id.is_empty() {
        return None;
    }
    let text = if crate::js::truthy(Some(value)) {
        crate::js::string(value)
    } else {
        String::new()
    };
    let (name, v) = match what {
        "metered" if matches!(text.as_str(), "yes" | "no" | "unknown") => {
            ("wifi.metered", json!(text))
        }
        "mac"
            if matches!(
                text.as_str(),
                "default" | "random" | "stable" | "permanent" | "preserve"
            ) =>
        {
            ("wifi.mac", json!(text))
        }
        "priority" => {
            let n = crate::js::round(crate::js::number(Some(value)));
            let n = if n.is_finite() {
                n.clamp(-999.0, 999.0)
            } else {
                0.0
            };
            ("wifi.priority", crate::js::json_number(n))
        }
        _ => return None,
    };
    Some(protocol::write(name, &json!({ "uuid": id, "value": v })))
}

/// `sanitizeIpv4`.
fn ipv4(raw: &str) -> String {
    let t = raw.trim();
    let parts: Vec<&str> = t.split('.').collect();
    let ok = parts.len() == 4
        && parts.iter().all(|p| {
            (1..=3).contains(&p.len())
                && p.chars().all(|c| c.is_ascii_digit())
                && p.parse::<u32>().is_ok_and(|n| n <= 255)
        });
    if ok { t.into() } else { String::new() }
}

/// What the static IPv4 dialog holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Static {
    pub address: String,
    pub prefix: String,
    pub gateway: String,
    pub dns: String,
}

/// `opFor("ipv4", spec)`: DHCP when `manual` is none, else the static address; none when the
/// address or prefix is not one.
pub fn ipv4_write(raw_uuid: &str, manual: Option<&Static>) -> Option<Value> {
    let id = uuid(raw_uuid);
    if id.is_empty() {
        return None;
    }
    let Some(s) = manual else {
        return Some(protocol::write("wifi.ipv4-auto", &json!({ "uuid": id })));
    };
    let address = ipv4(&s.address);
    let p = crate::js::round(crate::js::number(Some(&Value::from(s.prefix.as_str()))));
    let prefix = if p.is_finite() && (1.0..=32.0).contains(&p) {
        p
    } else {
        0.0
    };
    if address.is_empty() || prefix == 0.0 {
        return None;
    }
    let mut args = Map::new();
    args.insert("uuid".into(), json!(id));
    args.insert("address".into(), json!(address));
    args.insert("prefix".into(), crate::js::json_number(prefix));
    let gateway = ipv4(&s.gateway);
    if !gateway.is_empty() {
        args.insert("gateway".into(), json!(gateway));
    }
    let dns: Vec<String> = s
        .dns
        .trim()
        .split([' ', ','])
        .filter(|d| !d.is_empty())
        .map(ipv4)
        .filter(|d| !d.is_empty())
        .collect();
    if !dns.is_empty() {
        args.insert("dns".into(), json!(dns.join(",")));
    }
    Some(protocol::write("wifi.ipv4-manual", &Value::Object(args)))
}

/// The static IPv4 dialog's words: what is wrong, or what it asks.
pub fn static_says(error: &str) -> &str {
    if error.is_empty() {
        "Address and prefix are required. Leave gateway or DNS blank to skip them."
    } else {
        error
    }
}

/// A saved connection's words.
pub fn saved_says(row: &Value) -> &'static str {
    if crate::js::truthy(row.get("active")) {
        "Active now."
    } else {
        "Saved."
    }
}
