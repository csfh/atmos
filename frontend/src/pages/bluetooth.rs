//! The Bluetooth page's paired devices (`bluetooth.rs` has what they say and send): each with
//! Connect or Disconnect, Trust, and Forget…, which asks first. The discoverable switch, the
//! scan and the nearby devices are `pages/radios.rs`'s, from `atmosd`'s scan.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::bluetooth;
use crate::catalog::Field;
use crate::pages::row;

fn devices(page: &CatalogPage, cx: &App) -> Option<Vec<Value>> {
    Field::Key("bluetoothDevices")
        .read(page.stars.settings.read(cx))
        .value()
        .and_then(|v| v.as_array().cloned())
}

/// Whether a row of the page's own shows: "No paired devices." while there are none read.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    if let Some(shows) = crate::pages::radios::bt_shown(which, page, cx) {
        return shows;
    }
    which != "nopaired" || devices(page, cx).is_some_and(|d| d.is_empty())
}

/// What a row of the page's own says: Scan's words (`pages/radios.rs`).
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    crate::pages::radios::bt_says(which, page, cx)
}

fn send(
    page: &mut CatalogPage,
    body: Option<Value>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let Some(body) = body else { return };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |s, cx| {
        s.send(
            &client,
            "bluetoothDevices",
            &body,
            "The device was not changed",
            window,
            cx,
        )
    });
}

/// Each paired device, with its actions.
pub fn paired(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    devices(page, cx)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let address = d.get("address").filter(|a| crate::js::truthy(Some(a))).map_or_else(String::new, crate::js::string);
            let name = d.get("name").filter(|n| crate::js::truthy(Some(n))).map_or_else(|| "Bluetooth device".into(), crate::js::string);
            let connected = crate::js::truthy(d.get("connected"));
            let act = |id: &str, label: &'static str, action: &'static str, variant: ButtonVariant| {
                let (this, address) = (this.clone(), address.clone());
                Button::new(SharedString::from(format!("bt-{i}-{id}")))
                    .label(label)
                    .variant(variant)
                    .disabled(address.is_empty())
                    .on_click(move |_, window, cx| {
                        let body = bluetooth::device_write(action, &address);
                        this.update(cx, |p, cx| send(p, body, window, cx))
                    })
            };
            let (asked, gone) = (this.clone(), address.clone());
            let forget = Button::new(SharedString::from(format!("bt-{i}-forget")))
                .label("Forget…")
                .variant(ButtonVariant::Destructive)
                .disabled(address.is_empty())
                .on_click(move |_, _, cx| {
                    let gone = gone.clone();
                    asked.update(cx, |p, cx| {
                        p.ask(
                            Asked {
                                title: "Forget device".into(),
                                message: "Forget this pairing? You will need to pair the device again the next time you want it.".into(),
                                action: "Forget",
                                go: Rc::new(move |p, window, cx| send(p, bluetooth::device_write("forget", &gone), window, cx)),
                            },
                            cx,
                        )
                    })
                });
            let controls = h_flex()
                .gap(Space::Sm)
                .child(if connected {
                    act("connect", "Disconnect", "disconnect", ButtonVariant::Outline)
                } else {
                    act("connect", "Connect", "connect", ButtonVariant::Primary)
                })
                .child(act("trust", "Trust", "trust", ButtonVariant::Outline))
                .child(forget);
            let says = bluetooth::says(d);
            let star = page.stars.star(page.page().id, &name, &says, cx);
            row::setting(name, says, "omarchy bluetooth device", controls, star).into_any_element()
        })
        .collect()
}
