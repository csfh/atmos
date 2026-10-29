//! The Bluetooth page (`pages/bluetooth.rs`): a paired device's words are the QML row's own
//! expression, run under Node, and the lines of the QML page the module follows are pinned. What
//! each setter sends is `catalog_setters.rs`'s. In the window, each paired device has its actions,
//! Connect sends its typed write, and Forget asks first. Without `node` the words test says so
//! and stops.

mod common;

use atmos_ui::bluetooth;
use common::qml;
use common::window::{open, ordinary};
use odk::gpui::{TestAppContext, px, size};
use serde_json::{Value, json};

fn flat_page() -> String {
    let src = std::fs::read_to_string(qml::repo().join("pages/network/BluetoothPage.qml")).unwrap();
    src.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "")
}

#[test]
fn a_paired_device_says_what_the_qml_row_says() {
    let flat = flat_page();
    let at = flat
        .find("description: (modelData && modelData.connected")
        .unwrap();
    let expr = &flat[at + "description: ".len()..flat[at..].find(" hint:").unwrap() + at];
    let rows = [
        json!({"name": "Pods", "address": "AA:BB:CC:DD:EE:FF", "connected": true, "battery": 80}),
        json!({"connected": false, "battery": "55%"}),
        json!({"connected": 1, "battery": ""}),
        json!({"battery": null}),
        json!({"battery": 0}),
        json!({}),
    ];
    let script = format!(
        "process.stdout.write(JSON.stringify({rows}.map((modelData) => {expr})));",
        rows = json!(rows)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<Value> = rows.iter().map(|r| json!(bluetooth::says(r))).collect();
    assert_eq!(Value::from(ours), theirs);

    for want in [
        "label: modelData && modelData.name ? modelData.name : \"Bluetooth device\"",
        "hint: \"omarchy bluetooth device\"",
        "text: modelData && modelData.connected ? \"Disconnect\" : \"Connect\" primary: !(modelData && modelData.connected) enabled: modelData && modelData.address",
        "if (modelData.connected) Omarchy.disconnectBluetoothDevice(modelData.address) else Omarchy.connectBluetoothDevice(modelData.address)",
        "text: \"Trust\" enabled: modelData && modelData.address onClicked: Omarchy.trustBluetoothDevice(modelData.address)",
        "text: \"Forget…\" danger: true enabled: modelData && modelData.address",
        "title: \"Forget device\" message: \"Forget this pairing? You will need to pair the device again the next time you want it.\" confirmText: \"Forget\"",
        "Omarchy.forgetBluetoothDevice(forgetBtConfirm.payload)",
        "available: root.pairedBt.length === 0",
        "else if (Omarchy.bluetoothDevices && Omarchy.bluetoothDevices.length) pairedBt = Omarchy.bluetoothDevices",
        // What this window does not draw: the adapter, the scan and what it finds are Quickshell's.
        "available: !!root.adapter label: \"Discoverable\"",
        "available: Omarchy.bluetooth && root.scanningBt && root.discoveredBt.length === 0",
    ] {
        assert!(flat.contains(want), "{want}");
    }
}

#[test]
fn each_paired_device_connects_and_forget_asks_first() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "bluetooth", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(3000.0)));
    w.sample(
        "settings.network",
        1,
        json!({"bluetooth": true, "bluetoothDevices": [
            {"name": "Pods", "address": "AA:BB:CC:DD:EE:FF", "connected": false},
            {"name": "Broken", "address": "", "connected": true}]}),
    );
    w.until("the devices", |w| w.drawn("bt-0-connect/button"));
    assert!(w.drawn("bt-1-forget/button"));
    w.settle();
    w.click("bt-1-connect/button");
    w.click("bt-0-connect/button");
    w.until("the write", |w| !w.daemon.of("bluetooth.device").is_empty());
    assert_eq!(
        w.daemon.of("bluetooth.device").len(),
        1,
        "no address, no write"
    );
    assert_eq!(
        w.daemon.of("bluetooth.device")[0]["args"],
        json!({"action": "connect", "address": "AA:BB:CC:DD:EE:FF"})
    );

    w.settle();
    w.click("bt-0-forget/button");
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    assert_eq!(
        catalog.read_with(w.cx, |c, _| c.asked()),
        Some("Forget device".to_owned())
    );
    assert_eq!(
        w.daemon.of("bluetooth.device").len(),
        1,
        "nothing forgotten yet"
    );
}
