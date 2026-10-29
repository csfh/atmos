//! What the Wi-Fi, Bluetooth, Accounts and Background pages draw from `atmosd`'s scans and
//! images (`radios.rs` has what they say and send). Wi-Fi scans when its page opens with the
//! radio on, when the radio comes on, after a join ends, and on Refresh; each nearby network has
//! Join (asking for a password, or an identity too, when it needs one), Disconnect and Forget….
//! Bluetooth's Scan runs one discovery each time it is turned on, and lists what it found, each
//! with Pair, while it stays on; Discoverable is the adapter's. The face and the wallpaper are
//! read when their page opens and again when their path changes.

use std::rc::Rc;
use std::sync::Arc;

use odk::prelude::*;
use serde_json::{Value, json};

use super::catalog::{Asked, CatalogPage};
use crate::catalog::Field;
use crate::pages::row;
use crate::radios::{self, Join, Sends};

const WIFI_SCAN: &str = "wifi-scan";
const WIFI_SCANNING: &str = "wifi-scanning";
const WIFI_RADIO: &str = "wifi-scan-radio";
const WIFI_ENDED: &str = "wifi-ended";
const ASKING: &str = "wifi-asking";
const IDENTITY: &str = "wifi-identity";
const PASSWORD: &str = "wifi-password";
const ACTION: &str = "wifi-action";
const ERROR: &str = "wifi-error";
const BT_SCAN: &str = "bt-scan";
const BT_ON: &str = "bt-scan-on";
const BT_BUSY: &str = "bt-scan-busy";
const SEEN: &str = "image-seen";

/// How long a Bluetooth scan looks, the backend's own default.
const BT_SECONDS: u32 = 8;

fn value(page: &CatalogPage, key: &'static str, cx: &App) -> Option<Value> {
    Field::Key(key)
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn on(page: &CatalogPage, key: &'static str, cx: &App) -> bool {
    value(page, key, cx) == Some(Value::Bool(true))
}

fn flag(page: &CatalogPage, key: &str) -> bool {
    page.data.get(key) == Some(&Value::Bool(true))
}

fn draft(page: &CatalogPage, key: &str) -> String {
    page.drafts.get(key).cloned().unwrap_or_default()
}

/// Sends `request`, and hands its answer to `then` on the page.
fn ask(
    page: &CatalogPage,
    request: Value,
    then: impl FnOnce(&mut CatalogPage, Result<Value, String>, &mut Context<CatalogPage>) + 'static,
    cx: &mut Context<CatalogPage>,
) {
    let reply = page.stars.client.request(&request);
    cx.spawn(async move |this, cx| {
        let outcome = reply.await.map_err(|f| f.message);
        let _ = this.update(cx, |p, cx| {
            then(p, outcome, cx);
            cx.notify();
        });
    })
    .detach();
}

fn scan_wifi(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    if flag(page, WIFI_SCANNING) {
        return;
    }
    page.data.insert(WIFI_SCANNING, Value::Bool(true));
    ask(
        page,
        crate::protocol::wifi_scan(),
        |p, outcome, _| {
            p.data.insert(WIFI_SCANNING, Value::Bool(false));
            // A scan that failed lists nothing, and is not run again until asked.
            let result = outcome.unwrap_or_else(|why| json!({ "networks": [], "failed": why }));
            p.data.insert(WIFI_SCAN, result);
        },
        cx,
    );
    cx.notify();
}

fn scan_bluetooth(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    page.data.insert(BT_BUSY, Value::Bool(true));
    ask(
        page,
        crate::protocol::bluetooth_scan(BT_SECONDS),
        |p, outcome, _| {
            p.data.insert(BT_BUSY, Value::Bool(false));
            if let Ok(result) = outcome {
                p.data.insert(BT_SCAN, result);
            }
        },
        cx,
    );
}

/// The image `which` of the page shown, read again when its path is not the one it was read for.
fn image(page: &mut CatalogPage, which: &'static str, path: String, cx: &mut Context<CatalogPage>) {
    let seen = format!("{which}\0{path}");
    if page.data.get(SEEN).and_then(Value::as_str) == Some(seen.as_str()) {
        return;
    }
    page.data.insert(SEEN, Value::from(seen));
    if path.is_empty() {
        page.images.remove(which);
        return;
    }
    ask(
        page,
        crate::protocol::host_image(which),
        move |p, outcome, _| match outcome.ok().as_ref().and_then(radios::image_bytes) {
            Some(bytes) => {
                p.images.insert(which, Arc::from(bytes));
            }
            None => {
                p.images.remove(which);
            }
        },
        cx,
    );
}

/// What the page shown reads for itself, when it opens and when the window's settings change.
pub fn want(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    let text = |p: &CatalogPage, k, cx: &App| {
        value(p, k, cx)
            .filter(|v| crate::js::truthy(Some(v)))
            .map_or_else(String::new, |v| crate::js::string(&v))
    };
    match page.page().id {
        "network/wifi" => {
            let ssid = text(page, "netSsid", cx);
            if !ssid.is_empty() && page.drafts.get(ACTION) == Some(&ssid) {
                page.drafts.remove(ACTION);
            }
            let radio = on(page, "wifiHw", cx) && on(page, "wifiRadio", cx);
            let was = flag(page, WIFI_RADIO);
            page.data.insert(WIFI_RADIO, Value::Bool(radio));
            if radio && (!was || !page.data.contains_key(WIFI_SCAN)) {
                scan_wifi(page, cx);
            }
        }
        "accounts" => {
            let path = text(page, "avatarPath", cx);
            image(page, "avatar", path, cx);
        }
        "appearance/background" => {
            let path = text(page, "background", cx);
            image(page, "background", path, cx);
        }
        _ => {}
    }
}

/// A page shown afresh scans again.
pub fn loaded(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    for key in [WIFI_SCAN, WIFI_RADIO, SEEN, BT_SCAN, BT_ON] {
        page.data.remove(key);
    }
    for key in [ACTION, ASKING, PASSWORD, IDENTITY, ERROR] {
        page.drafts.remove(key);
    }
    want(page, cx);
}

/// A join that ended clears what it waited on, says why one failed, and scans again.
pub fn took(
    page: &mut CatalogPage,
    (ended, last): (u64, Option<(&'static str, bool)>),
    cx: &mut Context<CatalogPage>,
) {
    let seen = page
        .data
        .get(WIFI_ENDED)
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if ended <= seen {
        return;
    }
    page.data.insert(WIFI_ENDED, Value::from(ended));
    let Some((kind @ ("wifi-join" | "wifi-enterprise"), failed)) = last else {
        return;
    };
    if failed && page.drafts.contains_key(ACTION) {
        let why = if kind == "wifi-enterprise" {
            "Enterprise join failed"
        } else {
            "Could not join"
        };
        page.drafts.insert(ERROR.into(), why.into());
    }
    page.drafts.remove(ACTION);
    page.drafts.remove(ASKING);
    if page.page().id == "network/wifi" {
        scan_wifi(page, cx);
    }
}

/// Sends what a setter sends. A join that is a job is waited on (`ssid`): the network shows
/// Joining… and the other Joins wait until it ends; a saved network brought up by a write is
/// done when it is the network in use, as the QML page's `onNetSsidChanged` has it.
fn send(
    page: &mut CatalogPage,
    sends: Option<Sends>,
    ssid: Option<String>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if let (Some(Sends::Job(_)), Some(ssid)) = (&sends, ssid) {
        page.drafts.insert(ACTION.into(), ssid);
    }
    match sends {
        Some(Sends::Write(body)) => {
            let client = page.stars.client.clone();
            page.stars.settings.update(cx, |s, cx| {
                s.send(
                    &client,
                    "wifiConnections",
                    &body,
                    "The network was not changed",
                    window,
                    cx,
                )
            });
        }
        Some(Sends::Job(job)) => page.jobs.update(cx, |j, cx| j.run(job, window, cx)),
        None => {}
    }
}

fn connections(page: &CatalogPage, cx: &App) -> Option<Value> {
    value(page, "wifiConnections", cx)
}

/// `submitWifiSecrets(row, secret, identity)`.
fn submit(page: &mut CatalogPage, row: &Value, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let (secret, identity) = (draft(page, PASSWORD), draft(page, IDENTITY));
    if secret.is_empty() {
        return;
    }
    page.drafts.remove(ERROR);
    let ssid = crate::js::string(&row["ssid"]);
    let sends = if radios::join(row) == Join::Enterprise {
        if identity.is_empty() {
            return;
        }
        radios::join_enterprise(&ssid, &identity, &secret).map(Sends::Job)
    } else {
        radios::join_wifi(&ssid, &secret, connections(page, cx).as_ref())
    };
    page.drafts.remove(ASKING);
    page.drafts.remove(PASSWORD);
    send(page, sends, Some(ssid), window, cx);
}

/// `connectWifi(row)`: join, or ask for what it needs first.
fn connect(
    page: &mut CatalogPage,
    row: &Value,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if page.drafts.contains_key(ACTION) {
        return;
    }
    let ssid = crate::js::string(&row["ssid"]);
    if radios::join(row) != Join::Now {
        page.drafts.insert(ASKING.into(), ssid);
        cx.notify();
        return;
    }
    page.drafts.remove(ERROR);
    let sends = radios::join_wifi(&ssid, "", connections(page, cx).as_ref());
    send(page, sends, Some(ssid), window, cx);
}

fn joining(page: &CatalogPage, cx: &App) -> bool {
    matches!(
        page.jobs.read(cx).running(),
        Some("wifi-join" | "wifi-enterprise")
    )
}

fn button(
    id: String,
    label: impl Into<SharedString>,
    variant: ButtonVariant,
    disabled: bool,
) -> Button {
    Button::new(SharedString::from(id))
        .label(label)
        .variant(variant)
        .disabled(disabled)
}

/// The nearby networks, each with its actions.
pub fn networks(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    if !on(page, "wifiRadio", cx) {
        return Vec::new();
    }
    let rows = page
        .data
        .get(WIFI_SCAN)
        .map(radios::wifi_rows)
        .unwrap_or_default();
    let acting = page.drafts.get(ACTION).cloned();
    let asking = draft(page, ASKING);
    let busy = acting.is_some() || joining(page, cx);
    rows.into_iter()
        .enumerate()
        .map(|(i, r)| {
            let ssid = crate::js::string(&r["ssid"]);
            let label = if ssid.is_empty() { "Network".to_owned() } else { ssid.clone() };
            let connected = r["connected"] == Value::Bool(true);
            let known = r["known"] == Value::Bool(true);
            let text = if connected {
                "Disconnect"
            } else if acting.as_deref() == Some(ssid.as_str()) {
                "Joining…"
            } else {
                "Join"
            };
            let variant = if connected { ButtonVariant::Outline } else { ButtonVariant::Primary };
            let (row_for_join, joined) = (r.clone(), this.clone());
            let join = button(format!("net-{i}-join"), text, variant, busy).on_click(move |_, window, cx| {
                let row = row_for_join.clone();
                joined.update(cx, |p, cx| {
                    if row["connected"] == Value::Bool(true) {
                        let s = radios::by_ssid("down", &crate::js::string(&row["ssid"]), connections(p, cx).as_ref());
                        send(p, s, None, window, cx)
                    } else {
                        connect(p, &row, window, cx)
                    }
                })
            });
            let forgets = (known && !connected).then(|| {
                let (forgot, ssid) = (this.clone(), ssid.clone());
                button(format!("net-{i}-forget"), "Forget…", ButtonVariant::Destructive, false).on_click(
                    move |_, _, cx| {
                        let ssid = ssid.clone();
                        forgot.update(cx, |p, cx| {
                            p.ask(
                                Asked {
                                    title: "Forget network".into(),
                                    message: "Forget this saved Wi-Fi network? You will need the password again the next time you join.".into(),
                                    action: "Forget",
                                    go: Rc::new(move |p, window, cx| {
                                        let s = radios::by_ssid("delete", &ssid, connections(p, cx).as_ref());
                                        send(p, s, None, window, cx)
                                    }),
                                },
                                cx,
                            )
                        })
                    },
                )
            });
            let form = (asking == ssid).then(|| secrets(i, &r, page, this));
            let controls = v_flex()
                .gap(Space::Sm)
                .child(h_flex().gap(Space::Sm).child(join).children(forgets))
                .children(form);
            let says = radios::wifi_says(&r);
            let star = page.stars.star(page.page().id, &label, &says, cx);
            row::setting(label, says, "nmcli", controls, star).into_any_element()
        })
        .collect()
}

/// The password (and an enterprise network's identity) a network asks for, and Connect.
fn secrets(
    i: usize,
    row: &Value,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let busy = page.drafts.contains_key(ACTION);
    let typed = |key: &'static str| {
        let this = this.clone();
        move |t: &str, _: &mut Window, cx: &mut App| {
            this.update(cx, |p, cx| {
                p.drafts.insert(key.into(), t.to_owned());
                cx.notify()
            })
        }
    };
    let identity = (radios::join(row) == Join::Enterprise).then(|| {
        Input::new(SharedString::from(format!("net-{i}-identity")), "Identity")
            .placeholder("Identity")
            .value(draft(page, IDENTITY))
            .disabled(busy)
            .on_change(typed(IDENTITY))
    });
    let (entered, pressed, r1, r2) = (this.clone(), this.clone(), row.clone(), row.clone());
    v_flex()
        .gap(Space::Sm)
        .children(identity)
        .child(
            Input::new(SharedString::from(format!("net-{i}-password")), "Password")
                .placeholder("Password")
                .masked(true)
                .value(draft(page, PASSWORD))
                .disabled(busy)
                .on_change(typed(PASSWORD))
                .on_submit(move |t, window, cx| {
                    let row = r1.clone();
                    entered.update(cx, |p, cx| {
                        p.drafts.insert(PASSWORD.into(), t.to_owned());
                        submit(p, &row, window, cx)
                    })
                }),
        )
        .child(
            button(
                format!("net-{i}-connect"),
                "Connect",
                ButtonVariant::Primary,
                busy,
            )
            .on_click(move |_, window, cx| {
                let row = r2.clone();
                pressed.update(cx, |p, cx| submit(p, &row, window, cx))
            }),
        )
}

/// What a Wi-Fi row of the page's own says and shows: no networks nearby, and why a join failed.
pub fn wifi_shown(which: &str, page: &CatalogPage, cx: &App) -> Option<bool> {
    Some(match which {
        "nearby" => {
            !on(page, "wifiRadio", cx)
                || page
                    .data
                    .get(WIFI_SCAN)
                    .map(radios::wifi_rows)
                    .is_none_or(|r| r.is_empty())
        }
        "joinerror" => !draft(page, ERROR).is_empty(),
        _ => return None,
    })
}

/// The error row's words.
pub fn wifi_error(page: &CatalogPage) -> String {
    draft(page, ERROR)
}

/// Refresh (no networks) and Dismiss (a join failed).
pub fn wifi_control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Option<Button> {
    let this = this.clone();
    Some(match which {
        "nearby" => button(
            id,
            "Refresh",
            ButtonVariant::Outline,
            !(on(page, "wifiHw", cx) && on(page, "wifiRadio", cx)) || flag(page, WIFI_SCANNING),
        )
        .on_click(move |_, _, cx| this.update(cx, scan_wifi)),
        "joinerror" => button(
            id,
            "Dismiss",
            ButtonVariant::Outline,
            draft(page, ERROR).is_empty(),
        )
        .on_click(move |_, _, cx| {
            this.update(cx, |p, cx| {
                p.drafts.remove(ERROR);
                cx.notify()
            })
        }),
        _ => return None,
    })
}

/// Whether Scan is on, and whether its discovery still runs.
fn scanning(page: &CatalogPage) -> (bool, bool) {
    (flag(page, BT_ON), flag(page, BT_BUSY))
}

/// What a Bluetooth row of the page's own says.
pub fn bt_says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    (which == "scan").then(|| {
        if !on(page, "bluetooth", cx) {
            "The adapter is off."
        } else if scanning(page).0 {
            "Looking for unpaired devices nearby."
        } else {
            "Unpaired devices nearby appear below while this is on."
        }
        .into()
    })
}

/// Whether a Bluetooth row of the page's own shows: Discoverable while there is an adapter (a
/// scan that found none says there is not), and no devices nearby while Scan found none.
pub fn bt_shown(which: &str, page: &CatalogPage, cx: &App) -> Option<bool> {
    Some(match which {
        "discoverable" => page
            .data
            .get(BT_SCAN)
            .is_none_or(|s| !s["adapter"].is_null()),
        "nonearby" => {
            let (on_, busy) = scanning(page);
            on(page, "bluetooth", cx)
                && on_
                && !busy
                && page
                    .data
                    .get(BT_SCAN)
                    .map(radios::bt_nearby)
                    .is_none_or(|d| d.is_empty())
        }
        _ => return None,
    })
}

/// Discoverable's switch and Scan's.
pub fn bt_control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Option<Switch> {
    let this = this.clone();
    let radio = on(page, "bluetooth", cx);
    Some(match which {
        "discoverable" => Switch::new(SharedString::from(id), "Discoverable")
            .checked(on(page, "bluetoothDiscoverable", cx))
            .disabled(!radio || value(page, "bluetoothDiscoverable", cx).is_none())
            .on_change(move |next, window, cx| {
                this.update(cx, |p, cx| {
                    let client = p.stars.client.clone();
                    let body = radios::discoverable_write(next);
                    p.stars.settings.update(cx, |s, cx| {
                        s.send(
                            &client,
                            "bluetoothDiscoverable",
                            &body,
                            "Discoverable was not changed",
                            window,
                            cx,
                        )
                    })
                })
            }),
        "scan" => Switch::new(SharedString::from(id), "Scan")
            .checked(radio && scanning(page).0)
            .disabled(!radio)
            .on_change(move |next, _, cx| {
                this.update(cx, |p, cx| {
                    p.data.insert(BT_ON, Value::Bool(next));
                    if next {
                        scan_bluetooth(p, cx);
                    }
                    cx.notify()
                })
            }),
        _ => return None,
    })
}

/// The devices Scan found, each with Pair, while it is on.
pub fn devices(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    if !(on(page, "bluetooth", cx) && scanning(page).0) {
        return Vec::new();
    }
    page.data
        .get(BT_SCAN)
        .map(radios::bt_nearby)
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, d)| {
            let address = crate::js::string(&d["address"]);
            let name = crate::js::string(&d["name"]);
            let label = if name.is_empty() {
                "Device".to_owned()
            } else {
                name
            };
            let paired = this.clone();
            let to = address.clone();
            let pair = button(
                format!("bt-near-{i}-pair"),
                "Pair",
                ButtonVariant::Primary,
                address.is_empty(),
            )
            .on_click(move |_, window, cx| {
                let body = radios::pair_write(&to);
                paired.update(cx, |p, cx| {
                    let Some(body) = body else { return };
                    let client = p.stars.client.clone();
                    p.stars.settings.update(cx, |s, cx| {
                        s.send(
                            &client,
                            "bluetoothDevices",
                            &body,
                            "The device was not paired",
                            window,
                            cx,
                        )
                    })
                })
            });
            let star = page.stars.star(page.page().id, &label, &address, cx);
            row::setting(label, address, "omarchy bluetooth device pair", pair, star)
                .into_any_element()
        })
        .collect()
}

/// The face, round, once read.
pub fn face(page: &CatalogPage) -> Option<impl IntoElement + use<>> {
    let bytes = page.images.get("avatar")?.clone();
    Some(Image::avatar("acct-face-image", "Your face").source(bytes))
}

/// The wallpaper, cropped to a thumbnail, once read.
pub fn wallpaper(page: &CatalogPage) -> Option<impl IntoElement + use<>> {
    let bytes = page.images.get("background")?.clone();
    Some(
        v_flex().w(Space::Xl).h(Space::Xl).flex_none().child(
            Image::new("wallpaper-image", "Current background")
                .fit(ImageFit::Cover)
                .source(bytes),
        ),
    )
}
