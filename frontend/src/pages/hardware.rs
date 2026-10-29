//! The Hardware page's rows (`hardware.rs` has what they say): what the `hardware` inventory
//! names of each part, each with Copy. A part's row whose label is the part's own name is drawn
//! by its group's list (`List::Hardware`), as search does not list it.

use odk::prelude::*;
use serde_json::{Value, json};

use super::catalog::CatalogPage;
use crate::hardware::{self as hw, field as s, first, has_text};
use crate::pages::row;
use crate::settings::{self, Known};

/// The inventory as `normalize` keeps it; empty until it is read.
fn hardware(page: &CatalogPage, cx: &App) -> Value {
    match page.inventories.read(cx).known("hardware") {
        Known::Value(doc) => hw::normalize(Some(doc)),
        _ => hw::normalize(None),
    }
}

/// Whether the Memory group is drawn while it has no row: memory was read (`catalog::SHOWN`).
pub fn memory_shows(page: &CatalogPage, cx: &App) -> bool {
    let m = &hardware(page, cx)["memory"];
    m["total"].as_f64().unwrap_or(0.0) > 0.0
        || m["modules"].as_array().is_some_and(|a| !a.is_empty())
}

/// Copy, of `text`, off when there is none.
fn copy(
    id: String,
    label: &'static str,
    text: String,
    page: &CatalogPage,
) -> impl IntoElement + use<> {
    let client = page.stars.client.clone();
    Button::new(SharedString::from(id))
        .label(label)
        .variant(ButtonVariant::Outline)
        .disabled(text.is_empty())
        .on_click(move |_, window, cx| row::copy(&client, &text, window, cx))
}

/// What Copy copies of a part's own row.
fn copied(section: &str, h: &Value) -> String {
    let (m, b) = (&h["machine"], &h["bios"]);
    match section {
        "machine" => first(&[hw::machine_summary(m), s(m, "name")], ""),
        "identity" => first(&[s(m, "serial"), s(m, "sku")], ""),
        "board" => hw::board_summary(&h["board"]),
        "chipset" => hw::chipset_summary(&h["chipset"]),
        "bios" => first(&[hw::bios_summary(b), s(b, "version")], ""),
        "tpm" => hw::tpm_summary(&h["tpm"]),
        "cpu" => s(&h["cpu"], "model"),
        "virt" => hw::virt_summary(&h["virtualization"]),
        _ => String::new(),
    }
}

/// What Copy copies of a part in a list, and what the button says.
fn part_copy(section: &str, r: &Value) -> (&'static str, String) {
    match section {
        "modules" => ("Copy", hw::module_summary(r)),
        "npus" => ("Copy", first(&[hw::npu_summary(r), s(r, "name")], "")),
        "nics" if !s(r, "mac").is_empty() => ("Copy MAC", s(r, "mac")),
        "nics" => ("Copy", first(&[s(r, "iface"), s(r, "name")], "")),
        "audio" | "usb" => ("Copy", s(r, "name")),
        "batteries" => ("Copy", hw::battery_summary(r)),
        _ => ("Copy", hw::thermal_summary(r)),
    }
}

/// The rows a group's list draws, by part.
pub fn rows(section: &'static str, page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let h = hardware(page, cx);
    let hint = match section {
        "machine" => "/sys/class/dmi/id",
        "board" => "/sys/class/dmi/id/board_name",
        "chipset" | "npus" => "lspci",
        "cpu" | "virt" => "lscpu",
        "modules" => "dmidecode -t memory",
        "nics" => "/sys/class/net",
        "audio" => "/proc/asound/cards",
        "usb" => "/sys/bus/usb/devices",
        "batteries" => "/sys/class/power_supply",
        _ => "/sys/class/thermal",
    };
    let star = |label: &str, says: &str| page.stars.star(page.page().id, label, says, cx);
    if let Some((label, says)) = hw::row(section, &h) {
        let control = copy(
            format!("hw-{section}-copy"),
            "Copy",
            copied(section, &h),
            page,
        );
        let star = star(&label, &says);
        return vec![row::setting(label, says, hint, control, star).into_any_element()];
    }
    hw::parts(section, &h)
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let (label, says) = hw::part(section, r);
            let (button, text) = part_copy(section, r);
            let id = format!("hw-{section}-{i}");
            let copy = copy(format!("{id}-copy"), button, text, page);
            let control = if section == "batteries" {
                battery(id, r, page, cx, copy).into_any_element()
            } else {
                copy.into_any_element()
            };
            let star = star(&label, &says);
            row::setting(label, says, hint, control, star).into_any_element()
        })
        .collect()
}

/// A battery's charge, Show battery, and Copy.
fn battery(
    id: String,
    r: &Value,
    page: &CatalogPage,
    cx: &App,
    copy: impl IntoElement,
) -> impl IntoElement {
    let capacity = r.get("capacity").and_then(Value::as_f64).unwrap_or(0.0);
    let present = crate::catalog::Field::Key("batteryPresent")
        .read(page.stars.settings.read(cx))
        .value()
        == Some(&Value::Bool(true));
    let (client, shown) = (page.stars.client.clone(), page.stars.settings.clone());
    v_flex()
        .gap(Space::Sm)
        .children((capacity > 0.0).then(|| {
            Progress::new(SharedString::from(format!("{id}-charge")), "Charge").value(capacity)
        }))
        .child(
            h_flex()
                .gap(Space::Sm)
                .child(
                    Button::new(SharedString::from(format!("{id}-show")))
                        .label("Show battery")
                        .variant(ButtonVariant::Outline)
                        .disabled(!present)
                        .on_click(move |_, window, cx| {
                            let body = crate::protocol::write(
                                "notification",
                                &json!({ "kind": "battery" }),
                            );
                            shown.update(cx, |settings, cx| {
                                let refused = "The battery was not shown";
                                settings.send(&client, "notification", &body, refused, window, cx)
                            })
                        }),
                )
                .child(copy),
        )
}

/// What a row of the page's own says, by part.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    hw::row(which, &hardware(page, cx)).map(|(_, says)| says)
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let h = hardware(page, cx);
    match which {
        "refresh" => true,
        "installed" => h["memory"]["total"].as_f64().is_some_and(|t| t > 0.0),
        _ => hw::row(which, &h).is_some(),
    }
}

/// A row of the page's own: its control.
pub fn control(
    which: &'static str,
    id: String,
    page: &CatalogPage,
    cx: &App,
) -> impl IntoElement + use<> {
    let h = hardware(page, cx);
    match which {
        "identity" => {
            let label = if has_text(&h["machine"], &["serial"]) {
                "Copy serial"
            } else {
                "Copy SKU"
            };
            copy(id, label, copied(which, &h), page).into_any_element()
        }
        "refresh" => {
            let client = page.stars.client.clone();
            Button::new(SharedString::from(id))
                .label("Refresh")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, _| {
                    drop(
                        client.request(&crate::protocol::snapshot_invalidate(settings::ALL_TOPICS)),
                    )
                })
                .into_any_element()
        }
        "secureBoot" => {
            let on = crate::js::truthy(h["secureBoot"].get("enabled"));
            Text::new(if on { "On" } else { "Off" }).into_any_element()
        }
        "installed" => {
            let total = h["memory"]["total"].as_f64().unwrap_or(0.0);
            Text::new(hw::format_bytes(total)).into_any_element()
        }
        _ => copy(id, "Copy", copied(which, &h), page).into_any_element(),
    }
}
