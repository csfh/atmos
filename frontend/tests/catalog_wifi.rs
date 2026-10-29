//! The Wi-Fi page (`pages/wifi.rs`): the QR code is `RichUi.parseQrOutput`'s, run under Node, and
//! the lines of the QML page the module follows are pinned. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::wifi;
use common::qml;
use serde_json::{Value, json};

#[test]
fn the_qr_code_is_rich_ui_s() {
    let outputs = [
        "meta\twlan0\tWPA\tHome Net\n0110\n1001\n0110\n1001\n",
        "meta\twlan0\tWPA\tTab\tbed\n01\r\n10\r\n",
        "0101\n101\n",
        "no matrix here\n",
        "",
        "junk\n11\n00\nmore junk\n",
    ];
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const R = load("services/RichUi.js");
process.stdout.write(JSON.stringify({outputs}.map((o) => R.parseQrOutput(o)).map((p) => [p.ok, p.error, p.ssid, p.rows])));
"#,
        outputs = json!(outputs)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<Value> = outputs
        .iter()
        .map(|o| match wifi::parse_qr(o) {
            Ok(q) => {
                let rows: Vec<Vec<u8>> = q
                    .rows
                    .iter()
                    .map(|r| r.iter().map(|c| u8::from(*c)).collect())
                    .collect();
                json!([true, "", q.ssid, rows])
            }
            // A failed parse still says the network it read: compare that from the JavaScript.
            Err(why) => json!([false, why, Value::Null, []]),
        })
        .collect();
    for (o, t) in ours.iter().zip(theirs.as_array().unwrap()) {
        assert_eq!(o[0], t[0]);
        assert_eq!(o[1], t[1]);
        assert_eq!(o[3], t[3]);
        if o[0] == json!(true) {
            assert_eq!(o[2], t[2]);
        }
    }
    let src = std::fs::read_to_string(qml::repo().join("pages/network/WifiPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "text: root.qrLoading ? \"Building…\" : \"Show QR\" enabled: !root.qrLoading && Omarchy.wifiConnected onClicked: root.startQr()",
        "text: \"Copy password\" enabled: Omarchy.wifiIface.length > 0 onClicked: Omarchy.copyWifiPassword()",
        "description: root.qrSsid.length ? (\"A scannable code for \" + root.qrSsid + \".\") : \"A scannable code for the network you are on, so someone nearby can join.\"",
        "description: modelData && modelData.active ? \"Active now.\" : \"Saved.\"",
        "{ value: \"unknown\", label: \"Metered: auto\" }, { value: \"yes\", label: \"Metered\" }, { value: \"no\", label: \"Not metered\" }",
        "if (modelData && modelData.uuid) Omarchy.setConnectionPriority(modelData.uuid, value)",
        "if (modelData && modelData.uuid) Omarchy.setConnectionIpv4(modelData.uuid, { method: \"auto\" })",
        "root.staticError = \"Check the address and prefix.\"",
        "text: root.staticError.length ? root.staticError : \"Address and prefix are required. Leave gateway or DNS blank to skip them.\"",
        "available: Omarchy.wifiConnections.length === 0",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    assert_eq!(wifi::qr_says("Home"), "A scannable code for Home.");
    assert_eq!(wifi::qr_failed("  bad  "), "bad");
    assert_eq!(wifi::qr_failed(""), "Could not build a QR code");
}
