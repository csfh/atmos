//! The radios' typed writes: Wi-Fi's band and connections, and a Bluetooth device and the
//! adapter's discoverable switch. Apart from `writes.rs` so the registry stays within its file
//! budget; they are built and run as every other write is.

use serde_json::{json, Value};

use super::args::*;
use super::writes::{equals, row_gone, row_has, Built};
use crate::error::Result;

/// Check the arguments of radio write `name` and build it.
pub(super) fn build(name: &str, args: &Value) -> Result<Built> {
    Ok(match name {
        "wifi.band" => {
            let band = one_of(args, "band", &["auto", "2.4", "5", "6"])?;
            Built::new(
                "network",
                "wifiBandSelected",
                s(&["omarchy", "network", "band", band]),
                &["wifiBandSelected"],
            )
            .settle(equals("wifiBandSelected", json!(band)))
            .slow()
        }
        "wifi.connection" => {
            let action = one_of(args, "action", &["up", "down", "delete"])?;
            let id = uuid(args)?;
            let built = Built::new(
                "network",
                &format!("wifi:{id}"),
                task("set-wifi-connection", &[action, id])
                    .into_iter()
                    .collect(),
                &["wifiConnections", "wifiConnected", "netKind", "netSsid"],
            );
            let id = id.to_string();
            let built = built.slow();
            match action {
                "up" => built.settle(row_has(
                    "wifiConnections",
                    "uuid",
                    id,
                    "active",
                    json!(true),
                )),
                "down" => built.settle(row_has(
                    "wifiConnections",
                    "uuid",
                    id,
                    "active",
                    json!(false),
                )),
                _ => built.settle(row_gone("wifiConnections", "uuid", id)),
            }
        }
        "bluetooth.device" => {
            let action = one_of(args, "action", &["connect", "disconnect", "forget"])?;
            let addr = address(args)?;
            let built = Built::new(
                "bluetooth",
                &format!("bluetooth:{addr}"),
                s(&["omarchy", "bluetooth", "device", action, addr]),
                &["bluetoothDevices"],
            );
            let id = addr.to_string();
            let built = built.slow();
            match action {
                "connect" => built.settle(row_has(
                    "bluetoothDevices",
                    "address",
                    id,
                    "connected",
                    json!(true),
                )),
                "disconnect" => built.settle(row_has(
                    "bluetoothDevices",
                    "address",
                    id,
                    "connected",
                    json!(false),
                )),
                _ => built.settle(row_gone("bluetoothDevices", "address", id)),
            }
        }
        "bluetooth.discoverable" => {
            let on = flag(args, "on")?;
            Built::new(
                "bluetooth",
                "bluetoothDiscoverable",
                s(&[
                    "bluetoothctl",
                    "discoverable",
                    if on { "on" } else { "off" },
                ]),
                &["bluetoothDiscoverable"],
            )
            .settle(equals("bluetoothDiscoverable", json!(on)))
        }
        other => unreachable!("{other} is not a radio write"),
    })
}
