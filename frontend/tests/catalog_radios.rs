//! What the Wi-Fi, Bluetooth, Accounts and Background pages draw from `atmosd`'s scans and
//! images (`radios.rs`, `pages/radios.rs`): the nearby networks are `RichUi.wifiRow` rows in
//! `sortWifiRows` order with the QML row's words, Join chooses as the QML page's `connectWifi`
//! does, the nearby devices are `bluetoothLists`' `discovered`, the requests are `Requests.js`'s,
//! and the recorded replies (`tests/golden/cases`) read as they should. In the window, the Wi-Fi
//! page scans and joins, Bluetooth's Scan finds a device to pair, and the face is drawn. What the
//! setters send is `catalog_setters.rs`'s. Without `node` the Node test says so and stops.

mod common;

use atmos_ui::protocol;
use atmos_ui::radios::{self, Join};
use common::Reply;
use common::qml;
use common::window::{open, ordinary};
use odk::gpui::{TestAppContext, px, size};
use serde_json::{Value, json};

fn golden(name: &str) -> Value {
    let text = std::fs::read_to_string(qml::repo().join(format!("tests/golden/cases/{name}.json")))
        .unwrap();
    let case: Value = serde_json::from_str(&text).unwrap();
    case["response"]["result"].clone()
}

fn networks() -> Value {
    json!({"networks": [
        {"ssid": "Cafe", "signal": 80, "security": "", "inUse": false, "known": false},
        {"ssid": "Home", "signal": 40, "security": "WPA2", "inUse": true, "known": true},
        {"ssid": "Corp", "signal": 90, "security": "WPA2 802.1X", "inUse": false, "known": false},
        {"ssid": "Saved", "signal": 40, "security": "WPA1 WPA2", "inUse": false, "known": true},
        {"ssid": "Owe", "signal": 55.6, "security": "OWE", "inUse": false, "known": false},
        {"ssid": "Wpa3", "signal": 55, "security": "WPA3", "inUse": false, "known": false},
        {"ssid": "", "signal": 99, "security": "", "inUse": false, "known": false},
        {"ssid": "Dash", "signal": "x", "security": "--", "inUse": 0, "known": 1},
    ]})
}

fn devices() -> Value {
    json!({"adapter": {"address": "00:1A:7D:DA:71:13", "discoverable": false}, "devices": [
        {"address": "AA:BB:CC:DD:EE:01", "name": "Headphones", "paired": true, "connected": true},
        {"address": "AA:BB:CC:DD:EE:02", "name": "  Mouse  ", "paired": false, "connected": false},
        {"address": "AA:BB:CC:DD:EE:03", "name": "", "paired": false, "connected": false},
        {"address": "AA:BB:CC:DD:EE:04", "name": "Kbd", "paired": false, "connected": true},
        {"address": "", "name": "Ghost"},
        {"address": "AA:BB:CC:DD:EE:05", "deviceName": "Pad", "name": "x", "trusted": true},
    ]})
}

#[test]
fn the_scans_read_as_the_qml_window_reads_quickshell() {
    let nets = networks();
    let kinds: Vec<&str> = nets["networks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| radios::wifi_kind(n["security"].as_str().unwrap()))
        .collect();
    assert_eq!(
        kinds,
        [
            "open",
            "psk",
            "enterprise",
            "psk",
            "owe",
            "psk",
            "open",
            "open"
        ]
    );
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("catalog_radios.json");
    let b64 = ["R0lGODlhLXRpbnk=", "AA", "AAE", "/+8=", ""];
    std::fs::write(
        &input,
        json!({"nets": nets, "kinds": kinds, "devices": devices(), "b64": b64}).to_string(),
    )
    .unwrap();
    let script = format!(
        r#"{root}
globalThis.Quickshell = {{ env: () => "" }};
const {{ nets, kinds, devices, b64 }} = JSON.parse(fs.readFileSync({input}, "utf8"));
const R = load("services/RichUi.js"), Q = load("services/Requests.js");
const rows = [];
nets.networks.forEach((n, i) => {{ if (n && n.ssid) rows.push(R.wifiRow(n.ssid, n.signal, kinds[i], n.inUse, n.known)); }});
const sorted = R.sortWifiRows(rows);
const src = fs.readFileSync(process.env.REPO_DIR + "/pages/network/WifiPage.qml", "utf8");
const at = src.indexOf("description: {{", src.indexOf("model: root.wifiRows"));
const body = src.slice(at + "description: {{".length, src.indexOf("\n        }}\n", at));
const says = sorted.map((modelData) => new Function("modelData", body)(modelData));
const choices = sorted.map((row) => {{
  let joined = null;
  const Omarchy = {{ joinWifi: (ssid, pw) => {{ joined = [ssid, pw]; }} }};
  const page = pageRoot("network/WifiPage.qml", Omarchy, {{ actionKind: "", passwordSsid: "", actionSsid: "" }});
  page.actionKind = ""; page.passwordSsid = "";
  page.connectWifi(row);
  return joined ? "now" : (R.isEnterprise(row.securityKind) ? "enterprise" : (page.passwordSsid ? "password" : "none"));
}});
process.stdout.write(JSON.stringify({{
  sorted, says, choices,
  discovered: R.bluetoothLists(devices.devices).discovered,
  requests: [Q.hostImage("avatar"), Q.wifiScan(), Q.bluetoothScan(8)],
  b64: b64.map((t) => Array.from(Buffer.from(t, "base64"))),
}}));
"#,
        root = qml::PAGE_ROOT,
        input = json!(input.to_string_lossy()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let rows = radios::wifi_rows(&nets);
    assert_eq!(json!(rows), theirs["sorted"]);
    let says: Vec<String> = rows.iter().map(radios::wifi_says).collect();
    assert_eq!(json!(says), theirs["says"]);
    let choices: Vec<&str> = rows
        .iter()
        .map(|r| match radios::join(r) {
            Join::Now => "now",
            Join::Enterprise => "enterprise",
            Join::Password => "password",
        })
        .collect();
    assert_eq!(json!(choices), theirs["choices"]);
    assert_eq!(json!(radios::bt_nearby(&devices())), theirs["discovered"]);
    assert_eq!(
        json!([
            protocol::host_image("avatar"),
            protocol::wifi_scan(),
            protocol::bluetooth_scan(8)
        ]),
        theirs["requests"]
    );
    for (i, t) in b64.iter().enumerate() {
        assert_eq!(
            json!(radios::decode_base64(t).unwrap()),
            theirs["b64"][i],
            "{t}"
        );
    }
    assert_eq!(radios::decode_base64("A"), None);
    assert_eq!(radios::decode_base64("A*=="), None);

    // The recorded replies of the backend read as they should.
    let home = radios::wifi_rows(&golden("wifi-scan-omarchy"));
    assert_eq!(home[0]["ssid"], "Home");
    assert_eq!(radios::wifi_says(&home[0]), "Connected. Signal 72%.");
    assert!(
        radios::bt_nearby(&golden("bluetooth-scan-omarchy")).is_empty(),
        "the one device is paired"
    );
    assert_eq!(
        radios::image_bytes(&golden("host-image-avatar")).unwrap(),
        b"GIF89a-tiny"
    );
    assert_eq!(
        radios::image_bytes(&golden("host-image-background-none")),
        None
    );

    // The QML lines the module follows, and the backend's discoverable write.
    let flat = |file: &str| {
        let src = std::fs::read_to_string(qml::repo().join(file)).unwrap();
        src.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(" query: root.query", "")
    };
    let wifi = flat("pages/network/WifiPage.qml");
    for want in [
        "label: modelData && modelData.ssid ? modelData.ssid : \"Network\"",
        "text: modelData && modelData.connected ? \"Disconnect\" : (root.actionSsid === modelData.ssid ? \"Joining…\" : \"Join\") primary: !(modelData && modelData.connected) enabled: root.actionKind === \"\" && !root.wifiJoinBusy && modelData",
        "if (modelData.connected) Omarchy.deactivateWifiSsid(modelData.ssid) else root.connectWifi(modelData)",
        "visible: !!(modelData && modelData.known && !modelData.connected) text: \"Forget…\" danger: true",
        "title: \"Forget network\" message: \"Forget this saved Wi-Fi network? You will need the password again the next time you join.\" confirmText: \"Forget\" onConfirmed: Omarchy.forgetWifiSsid(forgetWifiConfirm.payload)",
        "visible: modelData && RichUi.isEnterprise(modelData.securityKind) placeholder: \"Identity\"",
        "PrefsPassword { id: rowPassword placeholder: \"Password\"",
        "text: \"Connect\" enabled: root.actionKind === \"\" onClicked: root.submitWifiSecrets(modelData, rowPassword.currentText(), identityField.currentText())",
        "text: \"Refresh\" enabled: Omarchy.wifiHw && Omarchy.wifiRadio",
        "available: root.wifiRows.length === 0",
        "if (Omarchy.lastError) root.wifiError = root.actionKind === \"enterprise\" ? \"Enterprise join failed\" : \"Could not join\"",
        "text: \"Dismiss\" enabled: root.wifiError.length > 0 onClicked: root.wifiError = \"\"",
    ] {
        assert!(wifi.contains(want), "{want}");
    }
    let bt = flat("pages/network/BluetoothPage.qml");
    for want in [
        "label: modelData && modelData.name ? modelData.name : \"Device\" description: modelData && modelData.address ? modelData.address : \"\" hint: \"omarchy bluetooth device pair\"",
        "text: \"Pair\" primary: true enabled: modelData && modelData.address onClicked: Omarchy.pairBluetoothDevice(modelData.address)",
        "available: Omarchy.bluetooth && root.scanningBt && root.discoveredBt.length === 0",
        "checked: Omarchy.bluetooth && root.scanningBt enabled: Omarchy.bluetooth onToggled: root.scanningBt = !root.scanningBt",
        "checked: !!(root.adapter && root.adapter.discoverable) enabled: Omarchy.bluetooth && !!root.adapter",
        "var paired = !!(d.paired || d.bonded || d.trusted)",
    ] {
        let hay = if want.starts_with("var paired") {
            flat("services/RichUi.js")
        } else {
            bt.clone()
        };
        assert!(hay.contains(want), "{want}");
    }
    let radio = std::fs::read_to_string(qml::repo().join("backend/src/command/radio.rs")).unwrap();
    let flat_radio = radio.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat_radio.contains("\"bluetooth.discoverable\" => { let on = flag(args, \"on\")?;"),
        "{radio}"
    );
    assert!(flat_radio.contains("&[\"bluetoothDiscoverable\"],"));
    assert_eq!(
        radios::discoverable_write(true),
        json!({"op": "write", "name": "bluetooth.discoverable", "args": {"on": true}})
    );
}

fn network(request: &Value) -> Reply {
    match request["op"].as_str() {
        Some("wifi.scan") => Reply::Ok(networks()),
        Some("bluetooth.scan") => Reply::Ok(devices()),
        Some("host.image") => Reply::Ok(
            json!({"which": request["which"], "image": {"base64": "R0lGODlhLXRpbnk=", "bytes": 11, "type": "image/gif"}}),
        ),
        _ => ordinary(request),
    }
}

/// The Wi-Fi page scans when it opens with the radio on, lists what it found, and joins an open
/// network at once; one that needs a password asks for it first.
#[test]
fn wifi_scans_lists_and_joins() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "network/wifi", network);
    w.cx.simulate_resize(size(px(1400.0), px(9000.0)));
    w.sample(
        "settings.network",
        1,
        json!({"wifiHw": true, "wifiRadio": true, "wifiConnected": true, "wifiIface": "wlan0",
            "wifiConnections": [], "netSsid": "Home"}),
    );
    w.until("the scan", |w| !w.daemon.of("wifi.scan").is_empty());
    // Sorted: Home (in use), Saved, Dash (saved), then Corp, Cafe, Owe, Wpa3 by signal.
    w.until("the networks", |w| w.drawn("net-4-join/button"));
    w.settle();
    w.click("net-4-join/button");
    w.until("the join", |w| !w.daemon.of("job.start").is_empty());
    let start = &w.daemon.of("job.start")[0];
    assert_eq!(
        (&start["name"], &start["args"]),
        (&json!("wifi.join"), &json!({"ssid": "Cafe"}))
    );
    assert!(!w.drawn("net-6-password/input"), "Wpa3 has not asked");
}

/// Bluetooth's Scan finds the devices nearby, each with Pair, and Pair sends its write.
#[test]
fn bluetooth_scan_finds_a_device_to_pair() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "bluetooth", network);
    w.cx.simulate_resize(size(px(1400.0), px(4000.0)));
    w.sample(
        "settings.network",
        1,
        json!({"bluetooth": true, "bluetoothDevices": [], "bluetoothDiscoverable": false}),
    );
    let scan = "ctl-bluetooth/scan/track";
    w.until("the switch", |w| w.drawn(scan));
    w.settle();
    w.click(scan);
    w.until("the scan", |w| !w.daemon.of("bluetooth.scan").is_empty());
    w.until("the devices", |w| w.drawn("bt-near-0-pair/button"));
    // Mouse, and the one with no name: not the paired, the connected or the trusted ones.
    assert!(w.drawn("bt-near-1-pair/button") && !w.drawn("bt-near-2-pair/button"));
    w.settle();
    w.click("bt-near-0-pair/button");
    w.until("the pair", |w| !w.daemon.of("bluetooth.pair").is_empty());
    assert_eq!(
        w.daemon.of("bluetooth.pair")[0]["args"],
        json!({"address": "AA:BB:CC:DD:EE:02"})
    );
}

/// Accounts reads this login's face from `atmosd` and draws it.
#[test]
fn accounts_draws_the_face_it_reads() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "accounts", network);
    w.cx.simulate_resize(size(px(1400.0), px(3000.0)));
    w.sample(
        "settings.accounts",
        1,
        json!({"currentUser": "me", "avatarPath": "/home/me/.face", "fullName": "Me", "users": [], "groups": []}),
    );
    w.until("the read", |w| !w.daemon.of("host.image").is_empty());
    assert_eq!(w.daemon.of("host.image")[0]["which"], "avatar");
    w.until("the face", |w| w.drawn("acct-face-image/image"));
}

/// Background reads the current wallpaper from `atmosd` and draws its preview.
#[test]
fn background_draws_the_wallpaper_it_reads() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "appearance/background", network);
    w.cx.simulate_resize(size(px(1400.0), px(3000.0)));
    w.sample(
        "settings.look",
        1,
        json!({"background": "/home/me/.local/state/omarchy/current/background"}),
    );
    w.until("the read", |w| !w.daemon.of("host.image").is_empty());
    assert_eq!(w.daemon.of("host.image")[0]["which"], "background");
    w.until("the preview", |w| w.drawn("wallpaper-image/image"));
}

/// Joining a saved network brings its connection up by a write and waits on nothing, so the other
/// networks' Joins still work.
#[test]
fn a_saved_network_comes_up_by_its_connection_and_leaves_the_others_free() {
    let uuid = "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9";
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "network/wifi", network);
    w.cx.simulate_resize(size(px(1400.0), px(9000.0)));
    w.sample(
        "settings.network",
        1,
        json!({"wifiHw": true, "wifiRadio": true, "wifiConnected": true, "wifiIface": "wlan0",
            "wifiConnections": [{"name": "Saved", "uuid": uuid}], "netSsid": "Home"}),
    );
    // Home (in use), Saved, Dash, then Corp, Cafe, Owe, Wpa3.
    w.until("the networks", |w| w.drawn("net-1-join/button"));
    w.settle();
    w.click("net-1-join/button");
    w.until("the write", |w| !w.daemon.of("wifi.connection").is_empty());
    assert_eq!(
        w.daemon.of("wifi.connection")[0]["args"],
        json!({"action": "up", "uuid": uuid})
    );
    w.settle();
    w.click("net-4-join/button");
    w.until("the next join", |w| !w.daemon.of("job.start").is_empty());
    assert_eq!(w.daemon.of("job.start")[0]["args"], json!({"ssid": "Cafe"}));
}
