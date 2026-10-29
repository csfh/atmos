//! The Wi-Fi page's rows, list and dialog (`wifi.rs` has what they say and send): the QR code of
//! the network in use with Copy password, each saved connection with its metered, priority, MAC
//! and IPv4 controls, and the static IPv4 dialog. Nearby networks are `pages/radios.rs`'s, from
//! `atmosd`'s scan. The QR code is drawn as tall as ODK lets a chart be (`catalog::BLOCKED`).

use odk::prelude::*;
use serde_json::{Value, json};

use super::catalog::CatalogPage;
use crate::catalog::Field;
use crate::jobs::Jobs;
use crate::pages::controls::select;
use crate::pages::row;
use crate::wifi::{self, Static};

const QR: &str = "wifi-qr";
const STATIC: &str = "wifi-static";
const ERROR: &str = "wifi-static-error";
const FIELDS: [(&str, &str); 4] = [
    ("wifi-addr", "10.0.0.8"),
    ("wifi-prefix", "24"),
    ("wifi-gateway", "10.0.0.1"),
    ("wifi-dns", "1.1.1.1"),
];

fn value(page: &CatalogPage, key: &'static str, cx: &App) -> Option<Value> {
    Field::Key(key)
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn text(page: &CatalogPage, key: &'static str, cx: &App) -> String {
    value(page, key, cx)
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, |v| crate::js::string(&v))
}

fn draft(page: &CatalogPage, key: &str) -> String {
    page.drafts.get(key).cloned().unwrap_or_default()
}

fn put(page: &mut CatalogPage, key: &str, value: impl Into<String>, cx: &mut Context<CatalogPage>) {
    page.drafts.insert(key.into(), value.into());
    cx.notify();
}

fn send(
    page: &mut CatalogPage,
    key: &'static str,
    body: Option<Value>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let Some(body) = body else { return };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |s, cx| {
        s.send(
            &client,
            key,
            &body,
            "The Wi-Fi setting was not changed",
            window,
            cx,
        )
    });
}

/// What the QR job left, kept until it runs again (`applyWifiQr`): the code, or why there is
/// none. The window's jobs say when it starts and ends.
pub fn took(page: &mut CatalogPage, jobs: &Jobs) {
    if jobs.running() == Some("wifi-qr") {
        page.data.remove(QR);
        return;
    }
    if let Some(("wifi-qr", failed)) = jobs.last {
        let qr = if failed {
            json!({ "error": wifi::qr_failed(&jobs.last_err) })
        } else {
            match wifi::parse_qr(&jobs.last_out) {
                Ok(q) => json!({ "ssid": q.ssid, "rows": q.rows }),
                Err(why) => json!({ "error": why }),
            }
        };
        page.data.insert(QR, qr);
    }
}

fn qr(page: &CatalogPage) -> (String, String, Vec<Vec<bool>>) {
    let q = page.data.get(QR).cloned().unwrap_or(Value::Null);
    let rows = serde_json::from_value(q["rows"].clone()).unwrap_or_default();
    (
        crate::js::string(&q["ssid"]).replace("null", ""),
        crate::js::string(&q["error"]).replace("null", ""),
        rows,
    )
}

/// What a row of the page's own says: the QR code's network, or (for the rows Quickshell's
/// scanner draws in the QML window, which stay hidden here) what the QML row says.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let on = |k| value(page, k, cx) == Some(Value::Bool(true));
    Some(match which {
        "qr" => wifi::qr_says(&qr(page).0),
        "nearby" if !on("wifiHw") => "No Wi-Fi adapter.".into(),
        "nearby" if on("wifiRadio") => "No networks nearby.".into(),
        "nearby" => "No networks. Turn the radio on above.".into(),
        "joinerror" => crate::pages::radios::wifi_error(page),
        _ => String::new(),
    })
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    if let Some(shows) = crate::pages::radios::wifi_shown(which, page, cx) {
        return shows;
    }
    match which {
        "qr" => value(page, "wifiConnected", cx) == Some(Value::Bool(true)),
        "nosaved" => {
            value(page, "wifiConnections", cx).and_then(|v| v.as_array().map(Vec::is_empty))
                == Some(true)
        }
        _ => false,
    }
}

/// The QR code row: Show QR, Copy password, and the code once built.
pub fn control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    if let Some(control) = crate::pages::radios::wifi_control(which, id.clone(), page, this, cx) {
        return control.into_any_element();
    }
    if which != "qr" {
        return v_flex().into_any_element();
    }
    let jobs = page.jobs.read(cx);
    let building = jobs.job_busy() && jobs.running() == Some("wifi-qr");
    let connected = value(page, "wifiConnected", cx) == Some(Value::Bool(true));
    let iface = text(page, "wifiIface", cx);
    let (_, error, rows) = qr(page);
    let (shows, copies) = (this.clone(), this.clone());
    let buttons = h_flex()
        .gap(Space::Sm)
        .child(
            Button::new(SharedString::from(format!("{id}-show")))
                .label(if building { "Building…" } else { "Show QR" })
                .variant(ButtonVariant::Outline)
                .disabled(building || !connected)
                .on_click(move |_, window, cx| {
                    shows.update(cx, |p, cx| {
                        let job = wifi::qr_job(&text(p, "wifiIface", cx));
                        p.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx))
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-copy")))
                .label("Copy password")
                .variant(ButtonVariant::Outline)
                .disabled(iface.is_empty())
                .on_click(move |_, window, cx| {
                    copies.update(cx, |p, cx| {
                        let body = wifi::password_copy(&text(p, "wifiIface", cx));
                        send(p, "wifiPassword", body, window, cx)
                    })
                }),
        );
    let size = rows.len();
    // As tall as ODK lets a chart ask to be, with the code's quiet zone of four modules round it
    // and each module a whole number of pixels, so a phone reads it.
    let code = (size > 0).then(|| {
        Chart::new(SharedString::from(format!("{id}-code")), "Wi-Fi QR code")
            .description(wifi::qr_says(&qr(page).0))
            .height(ChartHeight::Tall)
            .paint(move |scene, _| {
                let side = scene.width().min(scene.height());
                let cell = (side / (size + 8) as f32).floor().max(1.0);
                scene.fill_rect(
                    0.0,
                    0.0,
                    cell * (size + 8) as f32,
                    cell * (size + 8) as f32,
                    Ink::Canvas,
                );
                for (y, row) in rows.iter().enumerate() {
                    for (x, on) in row.iter().enumerate() {
                        if *on {
                            let (mx, my) = ((x + 4) as f32 * cell, (y + 4) as f32 * cell);
                            scene.fill_rect(mx, my, cell, cell, Ink::Text);
                        }
                    }
                }
            })
    });
    v_flex()
        .gap(Space::Sm)
        .child(buttons)
        .children((!error.is_empty()).then(|| Text::new(error)))
        .children(code)
        .into_any_element()
}

/// Each saved connection, with its metered, priority, MAC and IPv4 controls.
pub fn saved(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = value(page, "wifiConnections", cx)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    list.iter()
        .enumerate()
        .map(|(i, c)| {
            let uuid = c
                .get("uuid")
                .filter(|u| crate::js::truthy(Some(u)))
                .map_or_else(String::new, crate::js::string);
            let name = c
                .get("name")
                .filter(|n| crate::js::truthy(Some(n)))
                .map_or_else(|| "Connection".to_owned(), crate::js::string);
            let pick =
                |what: &'static str, label: &'static str, options: &[(&str, &str)], now: String| {
                    let (this, uuid) = (this.clone(), uuid.clone());
                    let options: Vec<(String, String)> = options
                        .iter()
                        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
                        .collect();
                    select(
                        format!("saved-{i}-{what}"),
                        label,
                        &options,
                        &Value::from(now),
                        false,
                        move |v, window, cx| {
                            let body = wifi::connection_write(what, &uuid, &Value::from(v));
                            this.update(cx, |p, cx| send(p, "wifiConnections", body, window, cx))
                        },
                    )
                    .into_any_element()
                };
            let metered = c
                .get("metered")
                .filter(|m| crate::js::truthy(Some(m)))
                .map_or_else(|| "unknown".to_owned(), crate::js::string);
            let mac = c
                .get("mac")
                .filter(|m| crate::js::truthy(Some(m)))
                .map_or_else(|| "default".to_owned(), crate::js::string);
            let priority = c
                .get("priority")
                .filter(|p| !p.is_null())
                .map_or_else(|| "0".to_owned(), crate::js::string);
            let (set, dhcp, opened) = (this.clone(), this.clone(), this.clone());
            let (u1, u2, u3) = (uuid.clone(), uuid.clone(), uuid.clone());
            let controls = v_flex()
                .gap(Space::Sm)
                .child(pick(
                    "metered",
                    "Metered",
                    &[
                        ("unknown", "Metered: auto"),
                        ("yes", "Metered"),
                        ("no", "Not metered"),
                    ],
                    metered,
                ))
                .child(
                    Input::new(
                        SharedString::from(format!("saved-{i}-priority")),
                        "Priority",
                    )
                    .placeholder("Priority")
                    .value(priority)
                    .on_submit(move |t, window, cx| {
                        let body = wifi::connection_write("priority", &u1, &Value::from(t));
                        set.update(cx, |p, cx| send(p, "wifiConnections", body, window, cx))
                    }),
                )
                .child(pick(
                    "mac",
                    "MAC address",
                    &[
                        ("default", "MAC default"),
                        ("random", "Random MAC"),
                        ("stable", "Stable random"),
                        ("permanent", "Permanent"),
                        ("preserve", "Preserve"),
                    ],
                    mac,
                ))
                .child(
                    h_flex()
                        .gap(Space::Sm)
                        .child(
                            Button::new(SharedString::from(format!("saved-{i}-dhcp")))
                                .label("DHCP")
                                .variant(ButtonVariant::Outline)
                                .on_click(move |_, window, cx| {
                                    let body = wifi::ipv4_write(&u2, None);
                                    dhcp.update(cx, |p, cx| {
                                        send(p, "wifiConnections", body, window, cx)
                                    })
                                }),
                        )
                        .child(
                            Button::new(SharedString::from(format!("saved-{i}-static")))
                                .label("Static…")
                                .variant(ButtonVariant::Outline)
                                .on_click(move |_, _, cx| {
                                    let uuid = u3.clone();
                                    opened.update(cx, |p, cx| {
                                        for (key, _) in FIELDS {
                                            p.drafts.remove(key);
                                        }
                                        p.drafts.insert(ERROR.into(), String::new());
                                        put(p, STATIC, uuid, cx)
                                    })
                                }),
                        ),
                );
            let says = wifi::saved_says(c);
            let hint = if uuid.is_empty() {
                "nmcli".to_owned()
            } else {
                uuid.clone()
            };
            let star = page.stars.star(page.page().id, &name, says, cx);
            row::setting(name, says, hint, controls, star).into_any_element()
        })
        .collect()
}

/// Whether the static IPv4 dialog is open, for what a test asks.
pub fn editing(page: &CatalogPage) -> bool {
    !draft(page, STATIC).is_empty()
}

fn apply(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let s = Static {
        address: draft(page, FIELDS[0].0),
        prefix: draft(page, FIELDS[1].0),
        gateway: draft(page, FIELDS[2].0),
        dns: draft(page, FIELDS[3].0),
    };
    match wifi::ipv4_write(&draft(page, STATIC), Some(&s)) {
        None => put(page, ERROR, "Check the address and prefix.", cx),
        body => {
            put(page, ERROR, "", cx);
            put(page, STATIC, "", cx);
            send(page, "wifiConnections", body, window, cx);
        }
    }
}

/// The static IPv4 dialog.
pub fn dialog(page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let (built, closed, cancelled, applied) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("wifi-static", "Static IPv4")
        .open(editing(page))
        .on_close(move |_, cx| closed.update(cx, |p, cx| put(p, STATIC, "", cx)))
        .content(move |_, cx| {
            let page = built.read(cx);
            let fields = FIELDS.iter().map(|(key, placeholder)| {
                let typed = built.clone();
                let key = *key;
                Input::new(key, *placeholder)
                    .placeholder(*placeholder)
                    .value(draft(page, key))
                    .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, key, t, cx)))
            });
            v_flex()
                .gap(Space::Md)
                .child(Text::new(wifi::static_says(&draft(page, ERROR)).to_owned()).muted())
                .children(fields)
        })
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("wifi-static-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, |p, cx| put(p, STATIC, "", cx)))
        })
        .action(move |_, _| {
            let applied = applied.clone();
            Button::new("wifi-static-apply")
                .label("Apply")
                .variant(ButtonVariant::Primary)
                .on_click(move |_, window, cx| applied.update(cx, |p, cx| apply(p, window, cx)))
        })
}
