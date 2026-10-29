//! The Network page's rows (`network.rs` has what they say and send): the connection and its
//! address, gateway and resolvers, the Wi-Fi switch, WireGuard's import, the hotspot, the DNS
//! provider and custom servers, and LocalSend's and Taildrop's files. A file or folder comes from
//! ODK's file chooser (`pages/files.rs`).

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use super::files::choose;
use crate::catalog::Field;
use crate::network;
use crate::pages::controls::select;
use crate::pages::row;

const WG_ERROR: &str = "net-wg-error";
const SSID: &str = "net-ssid";
const PASSWORD: &str = "net-password";
const HOTSPOT_ERROR: &str = "net-hotspot-error";
const DNS: &str = "net-dns";
const MACHINE: &str = "net-machine";

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

fn on(page: &CatalogPage, key: &'static str, cx: &App) -> bool {
    value(page, key, cx) == Some(Value::Bool(true))
}

fn servers(page: &CatalogPage, cx: &App) -> Vec<String> {
    value(page, "netDnsServers", cx)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(crate::js::string)
        .collect()
}

fn draft(page: &CatalogPage, key: &str) -> Option<String> {
    page.drafts.get(key).cloned()
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
    let Some(body) = body else {
        return;
    };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |s, cx| {
        s.send(
            &client,
            key,
            &body,
            "The network setting was not changed",
            window,
            cx,
        )
    });
}

fn run(
    page: &mut CatalogPage,
    job: Option<crate::jobs::Job>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if let Some(job) = job {
        page.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
    }
}

fn busy(page: &CatalogPage, cx: &App) -> bool {
    page.jobs.read(cx).job_busy()
}

fn wifi_says(page: &CatalogPage, cx: &App) -> String {
    network::wifi_says(
        on(page, "wifiHw", cx),
        on(page, "wifiRadio", cx),
        &text(page, "netKind", cx),
        &text(page, "netSsid", cx),
    )
}

/// The Connection row: what the machine reaches the internet through, and Refresh.
pub fn link(page: &CatalogPage, cx: &App) -> impl IntoElement + use<> {
    let t = |k| text(page, k, cx);
    let (label, says) = network::link(
        &t("netKind"),
        &t("netSsid"),
        &t("netIface"),
        &t("netSpeed"),
        &t("netSignal"),
    );
    let client = page.stars.client.clone();
    let refresh = Button::new("net-refresh")
        .label("Refresh")
        .variant(ButtonVariant::Outline)
        .on_click(move |_, _, _| {
            drop(client.request(&crate::protocol::snapshot_invalidate(
                crate::settings::ALL_TOPICS,
            )))
        });
    let star = page.stars.star(page.page().id, &label, &says, cx);
    row::setting(label, says, "omarchy network status", refresh, star)
}

/// What a row of the page's own says.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    Some(match which {
        "address" => text(page, "netIp", cx),
        "gateway" => text(page, "netGateway", cx),
        "dns" => servers(page, cx).join(", "),
        "wifi" => wifi_says(page, cx),
        "wireguard" => draft(page, WG_ERROR)
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| "Import a .conf from the tunnel you already have.".into()),
        "hotspot" => draft(page, HOTSPOT_ERROR)
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| "Share this machine's connection. Password needs 8 to 63 characters.".into()),
        "servers" => match network::dns(&draft(page, DNS).unwrap_or_default()).error {
            e if e.is_empty() => {
                "Type one or more IPv4 or IPv6 resolvers, separated by spaces. That switches the provider to Custom.".into()
            }
            e => e,
        },
        _ => return None,
    })
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let localsend = || {
        value(page, "extras", cx).and_then(|e| e.get("localsend").cloned())
            == Some(Value::Bool(true))
    };
    match which {
        "address" => !text(page, "netIp", cx).is_empty(),
        "gateway" => !text(page, "netGateway", cx).is_empty(),
        "dns" => !servers(page, cx).is_empty(),
        "hotspot" => on(page, "wifiHw", cx),
        "lsfile" | "lsfolder" => localsend(),
        "taildrop" => on(page, "tailscaleInstalled", cx),
        _ => true,
    }
}

fn copy(id: String, copied: String, page: &CatalogPage) -> impl IntoElement + use<> {
    let client = page.stars.client.clone();
    Button::new(SharedString::from(id))
        .label("Copy")
        .variant(ButtonVariant::Outline)
        .disabled(copied.is_empty())
        .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx))
}

fn button<G>(
    id: String,
    label: &'static str,
    variant: ButtonVariant,
    disabled: bool,
    this: &Entity<CatalogPage>,
    go: G,
) -> impl IntoElement + use<G>
where
    G: Fn(&mut CatalogPage, &mut Window, &mut Context<CatalogPage>) + 'static,
{
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label(label)
        .variant(variant)
        .disabled(disabled)
        .on_click(move |_, window, cx| this.update(cx, |p, cx| go(p, window, cx)))
}

/// A file or folder from the chooser, handed to `go`.
fn pick(
    id: String,
    label: &'static str,
    chooser: fn() -> FileChooser,
    disabled: bool,
    this: &Entity<CatalogPage>,
    go: fn(&mut CatalogPage, String, &mut Window, &mut Context<CatalogPage>),
) -> impl IntoElement + use<> {
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label(label)
        .variant(ButtonVariant::Outline)
        .disabled(disabled)
        .on_click(move |_, window, cx| choose(chooser(), &this, window, cx, Box::new(go)))
}

/// The control of a row of the page's own.
pub fn control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    match which {
        "address" => copy(id, text(page, "netIp", cx), page).into_any_element(),
        "gateway" => copy(id, text(page, "netGateway", cx), page).into_any_element(),
        "dns" => copy(id, servers(page, cx).join(", "), page).into_any_element(),
        "wifi" => {
            let hw = on(page, "wifiHw", cx);
            let (flip, open) = (this.clone(), page.open.clone());
            h_flex()
                .gap(Space::Sm)
                .child(
                    Switch::new(SharedString::from(format!("{id}-radio")), "Wi-Fi")
                        .checked(hw && on(page, "wifiRadio", cx))
                        .disabled(!hw)
                        .on_change(move |_, window, cx| {
                            flip.update(cx, |p, cx| {
                                let next = !on(p, "wifiRadio", cx);
                                let body = crate::settings::write("wifiRadio", &Value::Bool(next));
                                send(p, "wifiRadio", Some(body), window, cx)
                            })
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("{id}-manage")))
                        .label("Manage…")
                        .variant(ButtonVariant::Outline)
                        .disabled(!hw)
                        .on_click(move |_, _, cx| open("network/wifi", cx)),
                )
                .into_any_element()
        }
        "wireguard" => pick(
            id,
            "Import…",
            || {
                FileChooser::open("Import WireGuard")
                    .filter("WireGuard configs", ["*.conf", "*.nmconnection"])
                    .start_in(StartFolder::Home)
            },
            false,
            this,
            |p, path, window, cx| {
                if network::wireguard_path(&path).is_empty() {
                    put(p, WG_ERROR, "Pick an absolute .conf file.", cx);
                    return;
                }
                put(p, WG_ERROR, "", cx);
                send(
                    p,
                    "netWireguard",
                    network::wireguard_write(&path),
                    window,
                    cx,
                );
            },
        )
        .into_any_element(),
        "hotspot" => {
            let ssid = draft(page, SSID).unwrap_or_else(|| "Atmos".into());
            let (named, typed) = (this.clone(), this.clone());
            v_flex()
                .gap(Space::Sm)
                .child(
                    Input::new(SharedString::from(format!("{id}-ssid")), "SSID")
                        .placeholder("SSID")
                        .value(ssid)
                        .on_change(move |t, _, cx| named.update(cx, |p, cx| put(p, SSID, t, cx))),
                )
                .child(
                    Input::new(SharedString::from(format!("{id}-password")), "Password")
                        .placeholder("Password")
                        .value(draft(page, PASSWORD).unwrap_or_default())
                        .on_change(move |t, _, cx| {
                            typed.update(cx, |p, cx| put(p, PASSWORD, t, cx))
                        }),
                )
                .child(
                    h_flex()
                        .gap(Space::Sm)
                        .child(button(
                            format!("{id}-start"),
                            "Start",
                            ButtonVariant::Primary,
                            false,
                            this,
                            |p, window, cx| {
                                let ssid = draft(p, SSID).unwrap_or_else(|| "Atmos".into());
                                match network::hotspot_write(
                                    true,
                                    &ssid,
                                    &draft(p, PASSWORD).unwrap_or_default(),
                                ) {
                                    None => put(
                                        p,
                                        HOTSPOT_ERROR,
                                        "SSID and an 8–63 character password.",
                                        cx,
                                    ),
                                    body => {
                                        put(p, HOTSPOT_ERROR, "", cx);
                                        send(p, "netHotspot", body, window, cx)
                                    }
                                }
                            },
                        ))
                        .child(button(
                            format!("{id}-stop"),
                            "Stop",
                            ButtonVariant::Outline,
                            false,
                            this,
                            |p, window, cx| {
                                send(
                                    p,
                                    "netHotspot",
                                    network::hotspot_write(false, "", ""),
                                    window,
                                    cx,
                                )
                            },
                        )),
                )
                .into_any_element()
        }
        "provider" => {
            let current = text(page, "dns", cx);
            let options: Vec<(String, String)> = ["Cloudflare", "Google", "DHCP", "Custom"]
                .iter()
                .map(|o| ((*o).to_owned(), (*o).to_owned()))
                .collect();
            let this = this.clone();
            select(
                id,
                "DNS provider",
                &options,
                &Value::from(current.clone()),
                busy(page, cx) || current.is_empty(),
                move |v, window, cx| {
                    let v = v.to_owned();
                    this.update(cx, |p, cx| {
                        let now = text(p, "dns", cx);
                        send(p, "dns", network::dns_write(&v, &now), window, cx)
                    })
                },
            )
            .into_any_element()
        }
        "servers" => {
            let typed = draft(page, DNS).unwrap_or_default();
            let status = network::dns(&typed);
            let disabled = busy(page, cx);
            let (edited, entered) = (this.clone(), this.clone());
            let apply =
                |p: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>| {
                    if let Ok(servers) = network::parse_dns(&draft(p, DNS).unwrap_or_default()) {
                        run(p, network::custom_dns_job(&servers.join(" ")), window, cx);
                    }
                };
            h_flex()
                .gap(Space::Sm)
                .child(
                    Input::new(SharedString::from(format!("{id}-field")), "Custom servers")
                        .placeholder("1.1.1.1 8.8.8.8")
                        .value(status.formatted.clone())
                        .disabled(disabled)
                        .invalid(!status.error.is_empty())
                        .on_change(move |t, _, cx| {
                            let shown = network::dns(t).formatted;
                            edited.update(cx, |p, cx| put(p, DNS, shown, cx))
                        })
                        .on_submit(move |t, window, cx| {
                            let t = t.to_owned();
                            entered.update(cx, |p, cx| {
                                p.drafts.insert(DNS.into(), t);
                                apply(p, window, cx)
                            })
                        }),
                )
                .child(button(
                    format!("{id}-set"),
                    "Set",
                    ButtonVariant::Primary,
                    disabled || !status.ok,
                    this,
                    apply,
                ))
                .into_any_element()
        }
        "lsfile" => pick(
            id,
            "Share file…",
            || FileChooser::open("Share a file").start_in(StartFolder::Home),
            false,
            this,
            |p, path, window, cx| {
                send(
                    p,
                    "netShare",
                    network::share_write(false, &path),
                    window,
                    cx,
                )
            },
        )
        .into_any_element(),
        "lsfolder" => pick(
            id,
            "Share folder…",
            || FileChooser::folder("Share a folder").start_in(StartFolder::Home),
            false,
            this,
            |p, path, window, cx| {
                send(p, "netShare", network::share_write(true, &path), window, cx)
            },
        )
        .into_any_element(),
        "taildrop" => {
            let machine = draft(page, MACHINE).unwrap_or_default();
            let typed = this.clone();
            h_flex()
                .gap(Space::Sm)
                .child(
                    Input::new(SharedString::from(format!("{id}-machine")), "Machine")
                        .placeholder("machine")
                        .value(machine.clone())
                        .on_change(move |t, _, cx| {
                            typed.update(cx, |p, cx| put(p, MACHINE, t, cx))
                        }),
                )
                .child(pick(
                    format!("{id}-send"),
                    "Send file…",
                    || FileChooser::open("Send a file").start_in(StartFolder::Home),
                    machine.is_empty(),
                    this,
                    |p, path, window, cx| {
                        let machine = draft(p, MACHINE).unwrap_or_default();
                        run(p, network::tailscale_send(&machine, &path), window, cx)
                    },
                ))
                .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}
