//! The setter samples of the pages whose rows are forms and dialogs: Displays, Input's layouts,
//! Keybindings, Network and Accounts (`Custom::samples` in `samples.rs`).

use serde_json::json;

use super::sample_pages::Sample;
use crate::software;

/// What the Displays page's rows call, on a laptop panel and an external monitor with a rule.
pub(super) fn display() -> Vec<Sample> {
    use crate::display;
    let monitors = json!([
        {"name": "eDP-1", "description": "BOE 0x0BCA", "width": 2880, "height": 1920,
         "refreshRate": 120.0, "scale": 2, "focused": true, "enabled": true, "transform": 0,
         "vrr": false, "availableModes": ["2880x1920@120.00Hz", "2880x1920@60.00Hz", "1920x1200@60.00Hz"],
         "brightnessAvailable": true, "brightness": 40, "internal": true},
        {"name": "DP-1", "description": "Dell Inc. DELL U2723QE", "width": 3840, "height": 2160,
         "refreshRate": 59.997, "scale": 1.5, "focused": false, "enabled": true, "transform": 0,
         "vrr": 0, "availableModes": ["3840x2160@60.00Hz", "2560x1440@59.95Hz"]},
    ]);
    let rules = json!([
        {"output": "DP-1", "mode": "3840x2160@60", "position": "auto", "scale": 1.5,
         "transform": 0, "disabled": false, "vrr": 0, "bitdepth": 10, "cm": ""},
    ]);
    let doc = |internal: bool, external: bool| {
        json!({"monitors": monitors, "monitorRules": rules,
               "internalPresent": internal, "externalPresent": external})
    };
    let (live, current) = (
        monitors.as_array().unwrap().clone(),
        rules.as_array().unwrap().clone(),
    );
    let mut out: Vec<Sample> = Vec::new();
    for (output, patch) in [
        ("DP-1", json!({"transform": 1})),
        ("eDP-1", json!({"disabled": true})),
        ("eDP-1", json!({"mode": "1920x1200@60", "disabled": false})),
        ("HDMI-A-1", json!({"cm": "hdr"})),
        ("bad name", json!({"vrr": 1})),
    ] {
        let body = crate::monitors::patch_rules(&current, &live, output, &patch)
            .map(|next| display::rules_write(Some(&rules), next).0);
        out.push((
            "patchMonitorRule",
            doc(true, true),
            json!([output, patch]),
            body,
        ));
    }
    let items = display::scale_rules(&live[1], &current, "2");
    out.push((
        "writeMonitorRules",
        doc(true, true),
        json!([items.clone()]),
        Some(display::rules_write(Some(&rules), items).0),
    ));
    for scale in ["1.25", "2", "0", "x", ""] {
        let body = display::scale_write(scale, &live).map(|(b, _)| b);
        out.push(("setMonitorScale", doc(true, true), json!([scale]), body));
    }
    for (name, internal, external) in [
        ("desk", true, true),
        ("laptop", true, true),
        ("laptop", false, true),
        ("docked", true, true),
        ("docked", true, false),
        ("other", true, true),
    ] {
        let body = crate::monitors::layout_write(name, &live, (internal, external), Some(&rules))
            .map(|(b, _)| b);
        out.push((
            "applyMonitorLayout",
            doc(internal, external),
            json!([name]),
            body,
        ));
    }
    for (name, percent) in [
        ("eDP-1", 40.0),
        ("DP-1", 100.4),
        ("bad name", 5.0),
        ("eDP-1", 101.0),
        ("eDP-1", -1.0),
    ] {
        let body = display::brightness_write(name, percent);
        out.push((
            "setDisplayBrightness",
            json!({}),
            json!([name, percent]),
            body,
        ));
    }
    for (now, on) in [(true, false), (false, true), (true, true)] {
        let current = json!(now);
        let body = display::touchpad_write(on, Some(&current)).map(|(b, _)| b);
        out.push((
            "setTouchpad",
            json!({"touchpadEnabled": now}),
            json!([on]),
            body,
        ));
    }
    for direction in ["up", "down", "off", "restore", "x"] {
        let body = display::backlight_write(direction);
        out.push((
            "adjustKeyboardBacklight",
            json!({}),
            json!([direction]),
            body,
        ));
    }
    out
}

/// What Input's Hyprland layouts rows call, with the swipe in Atmos's block and outside it.
pub(super) fn keyboard() -> Vec<Sample> {
    use crate::keyboard;
    let raw = json!({"sensitivity": 0.3, "accelProfile": "flat", "kbLayoutOverride": "us",
        "kbVariantOverride": "", "kbGroupToggle": false, "workspaceGesture": true, "kbLayout": "us"});
    let mut out: Vec<Sample> = Vec::new();
    for unmanaged in [false, true] {
        let doc = json!({"hyprInput": raw, "hyprWorkspaceGestureUnmanaged": unmanaged});
        let input = keyboard::input(Some(&raw));
        for (layouts, variants, toggle) in [
            ("us,dk", ",nodeadkeys", false),
            ("US , DK", "", true),
            ("us", "", false),
            ("us", "", true),
            ("bad layout!", "", false),
            ("us,dk", "x", false),
            ("", "", false),
            ("us,dk,de", ",,", true),
        ] {
            let body =
                keyboard::set(&input, (layouts, variants, toggle), unmanaged).map(|(b, _)| b);
            out.push((
                "setHyprKbOverride",
                doc.clone(),
                json!([layouts, variants, toggle]),
                body,
            ));
        }
        // A revert writes what was set, unchecked (`dispatchRevert`).
        for (layouts, variants, toggle) in [
            ("us", "", false),
            ("", "", true),
            ("fr,us", "azerty,", false),
        ] {
            let (body, _) = keyboard::write(&input, (layouts, variants, toggle), unmanaged);
            out.push((
                "writeHyprKbOverride",
                doc.clone(),
                json!([layouts, variants, toggle]),
                Some(body),
            ));
        }
    }
    out
}

/// What Keybindings' Add and Remove call, with an override of Atmos's, one of the user's, and
/// two chords Hyprland has bound.
pub(super) fn bindings() -> Vec<Sample> {
    use crate::bindings;
    let mine = json!([
        {"keys": "SUPER + F", "label": "Files", "command": "nautilus", "unbind": false, "managed": true},
        {"keys": "SUPER + U", "command": "", "unbind": true, "managed": true},
        {"keys": "SUPER + B", "command": "firefox", "managed": false},
    ]);
    let bound = json!([{"keys": "SUPER + Q", "action": "Close window"}, {"keys": "SUPER + RETURN", "action": "Terminal"}]);
    let doc = json!({"bindings": mine, "keybindings": bound});
    let mut out: Vec<Sample> = Vec::new();
    let long = "SUPER + ".repeat(9);
    for (keys, label, command, unbind) in [
        ("SUPER + T", "Term", "alacritty", false),
        ("SUPER + Q", "", "kitty", false),
        ("  SUPER   +  F ", "", "", true),
        ("SUPER + F", "  Files  ", "thunar", false),
        ("", "", "x", false),
        ("SUPER + X", "", "", false),
        (long.as_str(), "", "x", false),
        ("SUPER + Y", "", "a\nb", false),
    ] {
        let body = bindings::add(Some(&mine), Some(&bound), (keys, label, command, unbind));
        out.push((
            "addBinding",
            doc.clone(),
            json!([keys, label, command, unbind]),
            body,
        ));
    }
    for keys in ["SUPER + F", "SUPER + B", "nope"] {
        out.push((
            "removeBinding",
            doc.clone(),
            json!([keys]),
            bindings::remove(Some(&mine), keys),
        ));
    }
    out
}

/// What the Network page's own rows call.
pub(super) fn network() -> Vec<Sample> {
    use crate::network;
    let job = |j: Option<crate::jobs::Job>| j.map(|j| software::job_body(&j));
    let mut out: Vec<Sample> = Vec::new();
    for (name, now) in [
        ("Google", "Cloudflare"),
        ("Cloudflare", "Cloudflare"),
        ("Custom", "DHCP"),
        ("DHCP", "Google"),
        ("x", ""),
    ] {
        out.push((
            "setDns",
            json!({"dns": now}),
            json!([name]),
            network::dns_write(name, now),
        ));
    }
    for servers in ["1.1.1.1 8.8.8.8", "  9.9.9.9  ", "", "   "] {
        out.push((
            "setCustomDns",
            json!({}),
            json!([servers]),
            job(network::custom_dns_job(servers)),
        ));
    }
    for (on, ssid, password) in [
        (true, "Atmos", "secret123"),
        (true, " Cafe ", "short"),
        (true, "", "longenough"),
        (true, &"s".repeat(33), "longenough"),
        (true, "Net", &"p".repeat(64)),
        (true, "a\nb", "longenough"),
        (false, "", ""),
    ] {
        out.push((
            "setHotspot",
            json!({}),
            json!([on, ssid, password]),
            network::hotspot_write(on, ssid, password),
        ));
    }
    for path in [
        "/home/u/wg0.conf",
        " /etc/x.nmconnection ",
        "wg0.conf",
        "/home/u/../x.conf",
        "/home/u/x.txt",
        "",
    ] {
        out.push((
            "importWireGuard",
            json!({}),
            json!([path]),
            network::wireguard_write(path),
        ));
    }
    for path in ["/home/u/a.txt", "a.txt", "/home/../x", ""] {
        out.push((
            "shareFile",
            json!({}),
            json!([path]),
            network::share_write(false, path),
        ));
        out.push((
            "shareFolder",
            json!({}),
            json!([path]),
            network::share_write(true, path),
        ));
    }
    for (machine, path) in [
        ("laptop", "/home/u/a.txt"),
        (" pi-4 ", ""),
        ("bad name", "/x"),
        ("laptop", "rel"),
        ("", "/x"),
    ] {
        out.push((
            "tailscaleSend",
            json!({}),
            json!([machine, path]),
            job(network::tailscale_send(machine, path)),
        ));
    }
    out
}

/// What the Accounts page's own rows and dialogs call, as `me`.
pub(super) fn accounts() -> Vec<Sample> {
    use crate::accounts;
    let job = |j: Option<crate::jobs::Job>| j.map(|j| software::job_body(&j));
    let doc = json!({"currentUser": "me", "fullName": "Me Myself"});
    let mut out: Vec<Sample> = Vec::new();
    for (name, full, password, wheel) in [
        ("ana", "Ana B", "pw", true),
        (" bo ", "", "pw", false),
        ("Root", "", "pw", false),
        ("root", "", "pw", false),
        ("cy", "a,b", "pw", false),
        ("dee", "", "", false),
        ("eve", "", "a\nb", false),
        ("fi-", "", "pw", false),
    ] {
        let body = job(accounts::add_user(name, full, password, wheel));
        out.push((
            "addAccountUser",
            doc.clone(),
            json!([name, full, password, wheel]),
            body,
        ));
    }
    for (name, password) in [("me", "new"), ("me", ""), ("x y", "pw"), ("ana", "a\nb")] {
        out.push((
            "setAccountPassword",
            doc.clone(),
            json!([name, password]),
            job(accounts::set_password(name, password)),
        ));
    }
    for name in ["devs", " media ", "root", "Bad", ""] {
        out.push((
            "addAccountGroup",
            doc.clone(),
            json!([name]),
            job(accounts::add_group(name)),
        ));
        out.push((
            "removeAccountGroup",
            doc.clone(),
            json!([name]),
            job(accounts::remove_group(name)),
        ));
    }
    for name in ["wheel", "docker"] {
        out.push((
            "removeAccountGroup",
            doc.clone(),
            json!([name]),
            job(accounts::remove_group(name)),
        ));
    }
    for name in ["ana", "me", "nobody", ""] {
        out.push((
            "removeAccountUser",
            doc.clone(),
            json!([name]),
            job(accounts::remove_user(name, "me")),
        ));
    }
    for (group, name, on) in [
        ("wheel", "ana", true),
        ("wheel", "me", false),
        ("docker", "me", false),
        ("bad!", "ana", true),
        ("video", "x y", true),
    ] {
        let body = accounts::member_write(group, name, on, "me").map(|w| w.body);
        out.push((
            "setGroupMember",
            doc.clone(),
            json!([group, name, on]),
            body,
        ));
    }
    for name in ["New Name", " Me Myself ", "a:b", "-x", ""] {
        out.push((
            "setFullName",
            doc.clone(),
            json!([name]),
            accounts::full_name_write(name, "Me Myself"),
        ));
    }
    for path in ["/home/me/face.png", "face.png", "/home/../x.png", ""] {
        let body = accounts::avatar_write(path, "me").map(|w| w.body);
        out.push(("setAvatarPath", doc.clone(), json!([path]), body));
    }
    out.push((
        "clearAvatar",
        doc.clone(),
        json!([]),
        accounts::avatar_clear("me").map(|w| w.body),
    ));
    out.push((
        "clearAvatar",
        json!({"currentUser": ""}),
        json!([]),
        accounts::avatar_clear("").map(|w| w.body),
    ));
    out
}

/// What the Wi-Fi page's own rows and dialog call.
pub(super) fn wifi() -> Vec<Sample> {
    use crate::wifi::{self, Static};
    let uuid = "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9";
    let doc = |iface: &str| json!({"wifiIface": iface});
    let mut out: Vec<Sample> = Vec::new();
    for iface in ["wlan0", "bad iface", ""] {
        out.push((
            "fetchWifiQr",
            doc(iface),
            json!([]),
            Some(software::job_body(&wifi::qr_job(iface))),
        ));
        out.push((
            "copyWifiPassword",
            doc(iface),
            json!([]),
            wifi::password_copy(iface),
        ));
    }
    for (what, setter, values) in [
        (
            "metered",
            "setConnectionMetered",
            vec![json!("yes"), json!("no"), json!("unknown"), json!("maybe")],
        ),
        (
            "priority",
            "setConnectionPriority",
            vec![json!("12"), json!("-5000"), json!("x"), json!(3.6)],
        ),
        (
            "mac",
            "setConnectionMac",
            vec![json!("random"), json!("stable"), json!("odd")],
        ),
    ] {
        for v in values {
            for u in [uuid, " 0A1B2C3D-4E5F-6071-8293-A4B5C6D7E8F9 ", "nope"] {
                out.push((
                    setter,
                    json!({}),
                    json!([u, v]),
                    wifi::connection_write(what, u, &v),
                ));
            }
        }
    }
    let spec = |a: &str, p: &str, g: &str, d: &str| Static {
        address: a.into(),
        prefix: p.into(),
        gateway: g.into(),
        dns: d.into(),
    };
    for s in [
        spec("10.0.0.8", "24", "10.0.0.1", "1.1.1.1 8.8.8.8"),
        spec(" 192.168.1.9 ", "16", "", "9.9.9.9,bad, 1.0.0.1"),
        spec("10.0.0.8", "33", "", ""),
        spec("300.0.0.1", "24", "", ""),
        spec("10.0.0.8", "8", "x", ""),
    ] {
        let args = json!({"method": "manual", "address": s.address, "prefix": s.prefix, "gateway": s.gateway, "dns": s.dns});
        out.push((
            "setConnectionIpv4",
            json!({}),
            json!([uuid, args]),
            wifi::ipv4_write(uuid, Some(&s)),
        ));
    }
    out.push((
        "setConnectionIpv4",
        json!({}),
        json!([uuid, {"method": "auto"}]),
        wifi::ipv4_write(uuid, None),
    ));
    out.push((
        "setConnectionIpv4",
        json!({}),
        json!(["x", {"method": "auto"}]),
        wifi::ipv4_write("x", None),
    ));
    out
}

/// What a paired device's Connect, Disconnect, Trust and Forget send, with addresses the setters
/// take and refuse.
pub(super) fn bluetooth() -> Vec<Sample> {
    let mut out: Vec<Sample> = Vec::new();
    for address in [
        "AA:bb:0C:1d:2E:3f",
        "AA:BB:CC:DD:EE",
        " AA:BB:CC:DD:EE:FF",
        "",
        "zz:zz:zz:zz:zz:zz",
    ] {
        for (action, setter) in [
            ("connect", "connectBluetoothDevice"),
            ("disconnect", "disconnectBluetoothDevice"),
            ("forget", "forgetBluetoothDevice"),
            ("trust", "trustBluetoothDevice"),
        ] {
            out.push((
                setter,
                json!({}),
                json!([address]),
                crate::bluetooth::device_write(action, address),
            ));
        }
    }
    out
}

/// What a nearby network's Join, Disconnect and Forget… and a nearby device's Pair send
/// (`pages/radios.rs`), by a saved connection's uuid when there is one, with names and
/// passwords the setters take and refuse.
pub(super) fn radios() -> Vec<Sample> {
    use crate::radios::{self, Sends};
    let uuid = "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9";
    let saved = json!({"wifiConnections": [{"name": "Home", "uuid": uuid}, {"name": "Bad", "uuid": "nope"}]});
    let body = |s: Option<Sends>| match s {
        Some(Sends::Write(b)) => Some(b),
        Some(Sends::Job(j)) => Some(software::job_body(&j)),
        None => None,
    };
    let mut out: Vec<Sample> = Vec::new();
    for ssid in [
        "Home",
        "Cafe",
        "Bad",
        "-flag",
        "",
        "line\nbreak",
        "x".repeat(65).as_str(),
    ] {
        for password in ["", "hunter2", "bad\npass"] {
            let s = radios::join_wifi(ssid, password, saved.get("wifiConnections"));
            out.push(("joinWifi", saved.clone(), json!([ssid, password]), body(s)));
        }
        for (setter, action) in [("deactivateWifiSsid", "down"), ("forgetWifiSsid", "delete")] {
            let s = radios::by_ssid(action, ssid, saved.get("wifiConnections"));
            out.push((setter, saved.clone(), json!([ssid]), body(s)));
        }
    }
    for (ssid, identity, password) in [
        ("Corp", "me@corp", "pw"),
        ("Corp", "", "pw"),
        ("Corp", "me", ""),
        ("", "me", "pw"),
        ("Corp", "me\n", "pw"),
    ] {
        let s = radios::join_enterprise(ssid, identity, password).map(Sends::Job);
        out.push((
            "connectEnterpriseWifi",
            json!({}),
            json!([ssid, identity, password]),
            body(s),
        ));
    }
    for address in ["AA:BB:CC:DD:EE:FF", "aa:bb:cc:dd:ee", ""] {
        out.push((
            "pairBluetoothDevice",
            json!({}),
            json!([address]),
            radios::pair_write(address),
        ));
    }
    out
}
