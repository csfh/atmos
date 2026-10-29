//! The Network page's rows (`pages/network.rs`): the custom DNS field is `RichUi.js`'s, the
//! connection's row and the Wi-Fi and Bluetooth rows' words are the QML page's own functions, run
//! under Node, and the lines the page module follows are pinned. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::network;
use common::qml;
use serde_json::{Value, json};

const TYPED: &[&str] = &[
    "",
    " ",
    "1.1.1.1",
    "1.1.1.1 ",
    "1.1.1.1,8.8.8.8",
    "1.1.1.1;;8.8.8.8|9.9.9.9",
    "1.1.1",
    "1.1.1.1 8.8",
    "999.1.1.1",
    "12345",
    "1.2.3.4.5",
    "19216801",
    "1.2.3.",
    "1..2",
    "abc",
    "-1.1.1.1",
    "2606:4700::1111",
    "2606:4700:4700::1111 2001:4860:4860::8888",
    "fe80::",
    "::",
    "2001:db8:::1",
    "2001:DB8::1",
    "1:2:3:4:5:6:7:8",
    "1:2:3:4:5:6:7:8:9",
    "2001:db8",
    "2001:db8:",
    "x1.1.1.1y",
    "1.1.1.1\t8.8.8.8\n",
    "  9.9.9.9",
    "256.256.256.256",
    "001.002.003.004",
    "1.1.1.1 2001:db8::1 bad",
];

#[test]
fn the_dns_field_is_rich_ui_s() {
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const R = load("services/RichUi.js");
process.stdout.write(JSON.stringify({typed}.map((t) => {{
  const s = R.dnsInputStatus(t);
  const p = R.parseDnsServers(t);
  return [s.formatted, s.servers, s.ok, s.error, p.ok ? p.servers : p.error,
    R.isIpv4(t), R.isIpv6(t), R.isPartialIpv4(t), R.isPartialIpv6(t)];
}})));
"#,
        typed = json!(TYPED)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<Value> = TYPED
        .iter()
        .map(|t| {
            let d = network::dns(t);
            let parsed = match network::parse_dns(t) {
                Ok(servers) => json!(servers),
                Err(why) => json!(why),
            };
            json!([
                d.formatted,
                d.servers,
                d.ok,
                d.error,
                parsed,
                network::is_ipv4(t),
                network::is_ipv6(t),
                network::is_partial_ipv4(t),
                network::is_partial_ipv6(t),
            ])
        })
        .collect();
    for (i, (ours, theirs)) in ours.iter().zip(theirs.as_array().unwrap()).enumerate() {
        assert_eq!(ours, theirs, "{:?}", TYPED[i]);
    }
}

/// The connection's row and the Wi-Fi and Bluetooth rows' words are the page's functions.
#[test]
fn the_rows_words_are_the_qml_page_s() {
    let states = json!([
        {"netKind": "ethernet", "netSsid": "", "netIface": "enp3s0", "netSpeed": "1000", "netSignal": "", "wifiHw": true, "wifiRadio": true, "bluetooth": true, "bluetoothDevices": [{}]},
        {"netKind": "ethernet", "netSsid": "", "netIface": "", "netSpeed": "", "netSignal": "", "wifiHw": false, "wifiRadio": false, "bluetooth": false, "bluetoothDevices": []},
        {"netKind": "wifi", "netSsid": "Home", "netIface": "wlan0", "netSpeed": "", "netSignal": "72", "wifiHw": true, "wifiRadio": true, "bluetooth": true, "bluetoothDevices": [{}, {}]},
        {"netKind": "wifi", "netSsid": "", "netIface": "", "netSpeed": "", "netSignal": "", "wifiHw": true, "wifiRadio": false, "bluetooth": true, "bluetoothDevices": []},
        {"netKind": "disconnected", "netSsid": "", "netIface": "", "netSpeed": "", "netSignal": "", "wifiHw": true, "wifiRadio": true, "bluetooth": false, "bluetoothDevices": [{}]},
    ]);
    let script = format!(
        r#"{root}
process.stdout.write(JSON.stringify({states}.map((O) => {{
  const r = pageRoot("NetworkPage.qml", O, {{}});
  return [r.linkLabel(), r.linkDescription(), r.wifiHubDescription(), r.bluetoothHubDescription()];
}})));
"#,
        root = qml::PAGE_ROOT
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<Value> = states
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let t = |k: &str| s[k].as_str().unwrap_or_default().to_owned();
            let b = |k: &str| s[k] == json!(true);
            let (label, says) = network::link(
                &t("netKind"),
                &t("netSsid"),
                &t("netIface"),
                &t("netSpeed"),
                &t("netSignal"),
            );
            json!([
                label,
                says,
                network::wifi_says(b("wifiHw"), b("wifiRadio"), &t("netKind"), &t("netSsid")),
                network::bluetooth_says(
                    b("bluetooth"),
                    s["bluetoothDevices"].as_array().unwrap().len()
                ),
            ])
        })
        .collect();
    assert_eq!(json!(ours), theirs);

    let src = std::fs::read_to_string(qml::repo().join("pages/NetworkPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "available: Omarchy.netIp.length > 0 label: \"Address\" description: Omarchy.netIp",
        "available: Omarchy.netGateway.length > 0 label: \"Gateway\" description: Omarchy.netGateway",
        "available: Omarchy.netDnsServers.length > 0 label: \"DNS\" description: Omarchy.netDnsServers.join(\", \")",
        "checked: Omarchy.wifiHw && Omarchy.wifiRadio enabled: Omarchy.wifiHw onToggled: Omarchy.set(\"wifiRadio\", !Omarchy.wifiRadio)",
        "text: \"Manage…\" enabled: Omarchy.wifiHw onClicked: root.openSubpage(\"wifi\")",
        "title: \"Import WireGuard\" nameFilters: [\"WireGuard configs (*.conf *.nmconnection)\"]",
        "root.wgError = \"Pick an absolute .conf file.\" return } root.wgError = \"\" Omarchy.importWireGuard(path)",
        "description: root.wgError.length ? root.wgError : \"Import a .conf from the tunnel you already have.\"",
        "property string hotspotSsid: \"Atmos\"",
        "description: root.hotspotError.length ? root.hotspotError : \"Share this machine's connection. Password needs 8 to 63 characters.\"",
        "root.hotspotError = \"SSID and an 8–63 character password.\" return } root.hotspotError = \"\" Omarchy.setHotspot(true, root.hotspotSsid, root.hotspotPassword)",
        "text: \"Stop\" onClicked: Omarchy.setHotspot(false)",
        "enabled: !Omarchy.jobBusy && Omarchy.dns.length > 0 onChanged: function(value) { if (value === \"Custom\" || value === Omarchy.dns) return Omarchy.setDns(value) }",
        "{ value: \"Cloudflare\", label: \"Cloudflare\" }, { value: \"Google\", label: \"Google\" }, { value: \"DHCP\", label: \"DHCP\" }, { value: \"Custom\", label: \"Custom\" }",
        "var status = RichUi.dnsInputStatus(value) if (status.formatted !== value) dnsField.replaceText(status.formatted) root.dnsError = status.error root.dnsReady = status.ok",
        "enabled: !Omarchy.jobBusy && root.dnsReady onClicked: root.applyCustomDns(dnsField.currentText())",
        "var parsed = RichUi.parseDnsServers(value) dnsError = parsed.ok ? \"\" : parsed.error dnsReady = parsed.ok if (!parsed.ok) return Omarchy.setCustomDns(parsed.servers.join(\" \"))",
        "title: root.shareFileMode ? \"Share a file\" : \"Send a file\"",
        "if (root.shareFileMode) Omarchy.shareFile(path) else Omarchy.tailscaleSend(root.tailscaleMachine, path)",
        "title: \"Share a folder\" onAccepted: Omarchy.shareFolder(RichUi.pathFromUrl(selectedFolder))",
        "text: \"Send file…\" enabled: Omarchy.tailscaleInstalled && root.tailscaleMachine.length > 0",
    ] {
        assert!(flat.contains(want), "{want}");
    }
}
