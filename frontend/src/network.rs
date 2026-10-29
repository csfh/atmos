//! The Network page's logic (`pages/NetworkPage.qml`, `RichUi.js`'s DNS input, `NetworkPrefs.js`):
//! the connection's row, the Wi-Fi and Bluetooth rows' words, the custom DNS field as it is typed,
//! and the writes its rows send (`setDns`, `setCustomDns`, `setHotspot`, `importWireGuard`,
//! `shareFile`, `shareFolder`, `tailscaleSend`). `tests/catalog_network.rs` runs them under Node.

use serde_json::{Value, json};

use crate::jobs::Job;
use crate::protocol;

/// The Connection row: its label and words (`linkLabel`, `linkDescription`).
pub fn link(kind: &str, ssid: &str, iface: &str, speed: &str, signal: &str) -> (String, String) {
    let says = |head: &str, extra: Option<String>| {
        let mut parts = vec![head.to_owned()];
        if !iface.is_empty() {
            parts.push(iface.to_owned());
        }
        parts.extend(extra);
        format!("{}.", parts.join(" · "))
    };
    match kind {
        "ethernet" => (
            "Ethernet".into(),
            says(
                "Wired default route",
                (!speed.is_empty()).then(|| format!("{speed} Mb/s")),
            ),
        ),
        "wifi" => (
            if ssid.is_empty() {
                "Wi-Fi".into()
            } else {
                ssid.to_owned()
            },
            says(
                "Wi-Fi default route",
                (!signal.is_empty()).then(|| format!("{signal}% signal")),
            ),
        ),
        _ => (
            "Offline".into(),
            "No default route. Refresh after you plug in a cable or join Wi-Fi.".into(),
        ),
    }
}

/// The Wi-Fi row's words (`wifiHubDescription`).
pub fn wifi_says(hw: bool, radio: bool, kind: &str, ssid: &str) -> String {
    if !hw {
        "No Wi-Fi adapter.".into()
    } else if !radio {
        "Scan and join nearby networks.".into()
    } else if kind == "wifi" && !ssid.is_empty() {
        format!("Connected to {ssid}.")
    } else {
        "Scanning for access points.".into()
    }
}

/// The Bluetooth row's words (`bluetoothHubDescription`).
pub fn bluetooth_says(on: bool, paired: usize) -> String {
    match (on, paired) {
        (false, _) => "Pair a headset or mouse.".into(),
        (true, 0) => "No paired devices.".into(),
        (true, 1) => "One paired device.".into(),
        (true, n) => format!("{n} paired devices."),
    }
}

fn digits(s: &str, max: usize) -> bool {
    (1..=max).contains(&s.len()) && s.chars().all(|c| c.is_ascii_digit())
}

fn hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit() || c == ':')
}

/// `isIpv4`.
pub fn is_ipv4(t: &str) -> bool {
    let parts: Vec<&str> = t.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| digits(p, 3) && p.parse::<u32>().is_ok_and(|n| n <= 255))
}

/// `ipv6SplitGroups`: the groups, or none when one is not one to four hex digits.
fn groups(s: &str) -> Option<usize> {
    if s.is_empty() {
        return Some(0);
    }
    let g: Vec<&str> = s.split(':').collect();
    g.iter()
        .all(|x| (1..=4).contains(&x.len()) && x.chars().all(|c| c.is_ascii_hexdigit()))
        .then_some(g.len())
}

/// `isIpv6`.
pub fn is_ipv6(t: &str) -> bool {
    if !t.contains(':') || !hex(t) || t == "::" {
        return false;
    }
    match t.find("::") {
        Some(first) if t[first + 2..].contains("::") => false,
        None => groups(t) == Some(8),
        Some(first) => match (groups(&t[..first]), groups(&t[first + 2..])) {
            (Some(l), Some(r)) => l + r < 8,
            _ => false,
        },
    }
}

/// `isPartialIpv4`: what could still become an address as more is typed.
pub fn is_partial_ipv4(t: &str) -> bool {
    if t.is_empty() {
        return true;
    }
    if !t.chars().all(|c| c.is_ascii_digit() || c == '.') || t.contains("..") {
        return false;
    }
    let parts: Vec<&str> = t.split('.').collect();
    if parts.len() > 4 {
        return false;
    }
    parts.iter().enumerate().all(|(i, p)| {
        if p.is_empty() {
            i == parts.len() - 1
        } else {
            digits(p, 3) && p.parse::<u32>().is_ok_and(|n| n <= 255)
        }
    })
}

/// `isPartialIpv6`.
pub fn is_partial_ipv6(t: &str) -> bool {
    if t.is_empty() {
        return true;
    }
    if !hex(t) || t.contains(":::") {
        return false;
    }
    if let Some(first) = t.find("::")
        && t[first + 2..].contains("::")
    {
        return false;
    }
    let mut body = t;
    if body.ends_with(':') && !body.ends_with("::") {
        body = &body[..body.len() - 1];
    }
    if body.is_empty() || body == ":" {
        return false;
    }
    let count = |s: &str| -> i32 {
        if s.is_empty() {
            return 0;
        }
        let g: Vec<&str> = s.split(':').collect();
        let ok = g
            .iter()
            .all(|x| (1..=4).contains(&x.len()) && x.chars().all(|c| c.is_ascii_hexdigit()));
        if ok { g.len() as i32 } else { -1 }
    };
    match body.find("::") {
        None => (1..=8).contains(&count(body)),
        Some(first) => {
            let (l, r) = (count(&body[..first]), count(&body[first + 2..]));
            l >= 0 && r >= 0 && l + r < 8
        }
    }
}

/// `formatIpv4Piece`: the address it starts with, and what is left over.
fn ipv4_piece(token: &str) -> (String, String) {
    let s: String = token
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let (mut octets, mut extra, mut cur): (Vec<String>, String, String) =
        (Vec::new(), String::new(), String::new());
    let commit = |octets: &mut Vec<String>, extra: &mut String, cur: &mut String| {
        if octets.len() >= 4 {
            extra.push_str(cur);
        } else {
            octets.push(cur.clone());
        }
        cur.clear();
    };
    for ch in s.chars() {
        if ch == '.' {
            if octets.len() < 4 {
                commit(&mut octets, &mut extra, &mut cur);
            }
            continue;
        }
        if octets.len() >= 4 {
            extra.push(ch);
            continue;
        }
        let next = format!("{cur}{ch}");
        if next.len() > 3 || next.parse::<u32>().unwrap_or(0) > 255 {
            if cur.is_empty() {
                extra.push(ch);
                continue;
            }
            commit(&mut octets, &mut extra, &mut cur);
            if octets.len() >= 4 {
                extra.push(ch);
                continue;
            }
            cur = ch.to_string();
            continue;
        }
        cur = next;
    }
    if octets.len() < 4 {
        octets.push(cur);
    } else {
        extra.push_str(&cur);
    }
    let mut text = octets.join(".");
    if s.ends_with('.') && octets.len() < 4 && octets.last().is_some_and(|o| !o.is_empty()) {
        text.push('.');
    }
    (text, extra)
}

/// `formatDnsInput`: what the field shows of what was typed.
pub fn format_dns(text: &str) -> String {
    let mut s = String::new();
    let mut run = false;
    for c in text.chars() {
        if matches!(c, ',' | '\t' | ';' | '|' | '\n' | '\r') {
            if !run {
                s.push(' ');
            }
            run = true;
        } else {
            s.push(c);
            run = false;
        }
    }
    let space = |c: char| c.is_whitespace();
    let (leading, trailing) = (s.starts_with(space), s.ends_with(space));
    let s: String = s
        .chars()
        .filter(|c| c.is_whitespace() || c.is_ascii_hexdigit() || *c == '.' || *c == ':')
        .collect();
    let tokens: Vec<&str> = s.split_whitespace().collect();
    if tokens.is_empty() {
        return if leading || trailing {
            " ".into()
        } else {
            String::new()
        };
    }
    let mut out: Vec<String> = Vec::new();
    for tok in tokens {
        if tok.contains(':') {
            out.push(
                tok.chars()
                    .filter(|c| c.is_ascii_hexdigit() || *c == ':')
                    .collect::<String>()
                    .to_lowercase(),
            );
            continue;
        }
        let (text, mut rest) = ipv4_piece(tok);
        out.push(text);
        while !rest.is_empty() {
            let (more, extra) = ipv4_piece(&rest);
            if more.is_empty() {
                break;
            }
            out.push(more);
            if extra == rest {
                break;
            }
            rest = extra;
        }
    }
    let mut result = out.join(" ");
    if trailing {
        result.push(' ');
    }
    result
}

/// `dnsInputStatus(text)`: what the field shows, the servers in it, whether they are all
/// addresses, and what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dns {
    pub formatted: String,
    pub servers: Vec<String>,
    pub ok: bool,
    pub error: String,
}

pub fn dns(text: &str) -> Dns {
    let formatted = format_dns(text);
    let trimmed = formatted.trim().to_owned();
    let mut d = Dns {
        formatted: formatted.clone(),
        servers: Vec::new(),
        ok: false,
        error: String::new(),
    };
    if trimmed.is_empty() {
        return d;
    }
    let trailing = formatted.ends_with(char::is_whitespace);
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if token.starts_with('-') {
            d.error = "DNS servers cannot start with -".into();
            return d;
        }
        if is_ipv4(token) || is_ipv6(token) {
            d.servers.push((*token).to_owned());
            continue;
        }
        let last = i == tokens.len() - 1 && !trailing;
        if !(last && (is_partial_ipv4(token) || is_partial_ipv6(token))) {
            d.error = format!("Not an IPv4 or IPv6 address: {token}");
        }
        return d;
    }
    d.ok = true;
    d
}

/// `applyCustomDns(value)`: the servers, or what is wrong (`parseDnsServers`).
pub fn parse_dns(text: &str) -> Result<Vec<String>, String> {
    let d = dns(text);
    if d.ok {
        Ok(d.servers)
    } else if d.error.is_empty() {
        Err("Enter one or more DNS servers".into())
    } else {
        Err(d.error)
    }
}

/// `setDns(name)`.
pub fn dns_write(name: &str, current: &str) -> Option<Value> {
    (matches!(name, "Cloudflare" | "Google" | "DHCP") && name != current)
        .then(|| crate::settings::write("dns", &Value::from(name)))
}

/// `setCustomDns(servers)`.
pub fn custom_dns_job(servers: &str) -> Option<Job> {
    let servers = servers.trim();
    (!servers.is_empty()).then(|| {
        Job::new(
            "dns.custom",
            json!({ "servers": servers }),
            "dns-custom",
            "The DNS change",
        )
    })
}

fn one_line(s: &str) -> bool {
    !s.contains(['\n', '\r'])
}

/// `setHotspot(on, ssid, password)`: Stop, or Start with an SSID of at most 32 and a password
/// of 8 to 63.
pub fn hotspot_write(on: bool, ssid: &str, password: &str) -> Option<Value> {
    if !on {
        return Some(protocol::write("wifi.hotspot-off", &json!({})));
    }
    let name = ssid.trim();
    let units = |s: &str| s.encode_utf16().count();
    let ok = one_line(ssid)
        && !name.is_empty()
        && units(name) <= 32
        && one_line(password)
        && (8..=63).contains(&units(password));
    ok.then(|| {
        protocol::write(
            "wifi.hotspot-on",
            &json!({ "ssid": name, "password": password }),
        )
    })
}

/// `sanitizeWgPath`: an absolute `.conf` or `.nmconnection`, or "".
pub fn wireguard_path(raw: &str) -> String {
    let t = raw.trim();
    let ok = t.starts_with('/')
        && !t.contains('\n')
        && !t.contains("..")
        && (t.ends_with(".conf") || t.ends_with(".nmconnection"));
    if ok { t.to_owned() } else { String::new() }
}

/// `importWireGuard(path)`.
pub fn wireguard_write(path: &str) -> Option<Value> {
    let path = wireguard_path(path);
    (!path.is_empty()).then(|| protocol::write("wifi.wireguard-import", &json!({ "path": path })))
}

fn absolute(path: &str) -> bool {
    path.starts_with('/') && !path.contains("..")
}

/// `shareFile(path)` or `shareFolder(path)`.
pub fn share_write(folder: bool, path: &str) -> Option<Value> {
    absolute(path).then(|| {
        protocol::write(
            if folder { "share.folder" } else { "share.file" },
            &json!({ "path": path }),
        )
    })
}

/// `tailscaleSend(machine, path)`.
pub fn tailscale_send(machine: &str, path: &str) -> Option<Job> {
    let machine = machine.trim();
    let ok = !machine.is_empty()
        && machine
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
    if !ok || (!path.is_empty() && !absolute(path)) {
        return None;
    }
    let args = if path.is_empty() {
        json!({ "machine": machine })
    } else {
        json!({ "machine": machine, "path": path })
    };
    Some(
        Job::new(
            "tailscale.send",
            args,
            "tailscale-send",
            "The Taildrop send",
        )
        .interactive(),
    )
}
