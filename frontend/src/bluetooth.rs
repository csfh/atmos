//! The Bluetooth page's paired devices (`pages/network/BluetoothPage.qml`): what each says, and
//! the writes Connect, Disconnect, Trust and Forget send (`connectBluetoothDevice`, ...). The
//! nearby devices, Scan and Discoverable, which the QML window reads from Quickshell's
//! Bluetooth module, are `radios.rs`'s. `tests/catalog_bluetooth.rs` holds them to the QML.

use serde_json::{Value, json};

use crate::protocol;

/// Whether `a` is a device address (`^([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}$`).
pub fn address_ok(a: &str) -> bool {
    let parts: Vec<&str> = a.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// A device's write: `connect`, `disconnect`, `forget`, or `trust`.
pub fn device_write(action: &str, address: &str) -> Option<Value> {
    if !address_ok(address) {
        return None;
    }
    Some(match action {
        "trust" => protocol::write("bluetooth.trust", &json!({ "address": address })),
        "connect" | "disconnect" | "forget" => protocol::write(
            "bluetooth.device",
            &json!({ "action": action, "address": address }),
        ),
        _ => return None,
    })
}

/// A paired device's words: whether it is connected, its battery, and what Trust does.
pub fn says(row: &Value) -> String {
    let state = if crate::js::truthy(row.get("connected")) {
        "Connected. "
    } else {
        "Paired. "
    };
    let battery = match row.get("battery") {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) if s.is_empty() => String::new(),
        Some(b) => format!("Battery {}. ", crate::js::string(b)),
    };
    format!("{state}{battery}Trust keeps it auto-connecting.")
}
