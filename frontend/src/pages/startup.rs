//! The Startup page's form and lists (`startup.rs` has what they say and send): Add a command,
//! each line with its switch and Remove…, and the units that failed this boot, from the services
//! inventory.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::pages::row;
use crate::settings::Known;
use crate::startup::{self, AUTOSTART};

const COMMAND_DRAFT: &str = "startup-command";
const DELAY_DRAFT: &str = "startup-delay";
const HINT: &str = "~/.config/hypr/autostart.lua";

fn autostart(page: &CatalogPage, cx: &App) -> Option<Value> {
    AUTOSTART
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

/// Sends an `autostart` write, when there is one.
fn send(
    page: &mut CatalogPage,
    body: Option<Value>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let Some(body) = body else {
        return;
    };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |settings, cx| {
        settings.send(
            &client,
            "autostart",
            &body,
            "Startup was not changed",
            window,
            cx,
        );
    });
}

/// `addAutostart(root.autostartDraft, root.delayDraft)`.
fn add(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let command = page.drafts.get(COMMAND_DRAFT).cloned().unwrap_or_default();
    let delay: f64 = page
        .drafts
        .get(DELAY_DRAFT)
        .and_then(|d| d.parse().ok())
        .unwrap_or(0.0);
    let list = autostart(page, cx);
    let body = startup::add(list.as_ref(), &command, &crate::js::json_number(delay));
    send(page, body, window, cx);
}

/// A command, a delay, and Add.
pub fn form(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let command = page.drafts.get(COMMAND_DRAFT).cloned().unwrap_or_default();
    let delay: f64 = page
        .drafts
        .get(DELAY_DRAFT)
        .and_then(|d| d.parse().ok())
        .unwrap_or(0.0);
    let (typed, submitted, counted, added) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-command")), "Command")
                .placeholder("hyprsunset")
                .value(command)
                .on_change(move |text, _, cx| {
                    typed.update(cx, |p, cx| {
                        p.drafts.insert(COMMAND_DRAFT.into(), text.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |text, window, cx| {
                    submitted.update(cx, |p, cx| {
                        p.drafts.insert(COMMAND_DRAFT.into(), text.to_owned());
                        add(p, window, cx);
                    })
                }),
        )
        .child(
            NumberInput::new(SharedString::from(format!("{id}-delay")), "Delay")
                .value(delay)
                .min(0.0)
                .max(startup::MAX_DELAY)
                .on_commit(move |n, _, cx| {
                    counted.update(cx, |p, cx| {
                        p.drafts
                            .insert(DELAY_DRAFT.into(), crate::js::round(n).to_string());
                        cx.notify();
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-add")))
                .label("Add")
                .variant(ButtonVariant::Primary)
                .on_click(move |_, window, cx| added.update(cx, |p, cx| add(p, window, cx))),
        )
}

/// Each line, with its switch and Remove…, which only Atmos's lines can use.
pub fn lines(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = autostart(page, cx);
    startup::rows(list.as_ref())
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let command = crate::js::string(&r["command"]);
            let managed = crate::js::truthy(r.get("managed"));
            let on = startup::enabled(&r);
            let (toggled, asked) = (this.clone(), this.clone());
            let (c1, c2) = (command.clone(), command.clone());
            let control = h_flex()
                .gap(Space::Sm)
                .child(
                    Switch::new(SharedString::from(format!("startup-{i}")), command.clone())
                        .checked(on)
                        .disabled(!managed)
                        .on_change(move |_, window, cx| {
                            let command = c1.clone();
                            toggled.update(cx, |p, cx| {
                                let list = autostart(p, cx);
                                let body = startup::set_enabled(list.as_ref(), &command, !on);
                                send(p, body, window, cx);
                            })
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("startup-{i}-remove")))
                        .label("Remove…")
                        .variant(ButtonVariant::Destructive)
                        .disabled(!managed)
                        .on_click(move |_, _, cx| {
                            let command = c2.clone();
                            asked.update(cx, |p, cx| {
                                let gone = command.clone();
                                p.ask(
                                    Asked {
                                        title: "Remove startup command".into(),
                                        message: startup::remove_message(&command),
                                        action: "Remove",
                                        go: Rc::new(move |p, window, cx| {
                                            let list = autostart(p, cx);
                                            let body = startup::remove(list.as_ref(), &gone);
                                            send(p, body, window, cx);
                                        }),
                                    },
                                    cx,
                                )
                            })
                        }),
                );
            let description = startup::description(&r);
            let star = page.stars.star(page.page().id, &command, &description, cx);
            row::setting(command, description, HINT, control, star).into_any_element()
        })
        .collect()
}

fn failed(page: &CatalogPage, cx: &App) -> Vec<Value> {
    let units = match page.inventories.read(cx).known("services") {
        Known::Value(doc) => doc.get("items").cloned(),
        _ => None,
    };
    startup::failed(units.as_ref())
}

/// Whether the Failures group is drawn while it has no row: a unit failed (`catalog::SHOWN`).
pub fn failures_show(page: &CatalogPage, cx: &App) -> bool {
    !failed(page, cx).is_empty()
}

/// The units that failed this boot.
pub fn failures(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    failed(page, cx)
        .into_iter()
        .map(|u| {
            let label = u
                .get("unit")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(|| "unit".to_owned(), crate::js::string);
            let description = u
                .get("description")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(|| "Failed this boot.".to_owned(), crate::js::string);
            let star = page.stars.star(page.page().id, &label, &description, cx);
            row::setting(label, description, "", Text::new(""), star).into_any_element()
        })
        .collect()
}

/// Whether "No failed units reported." shows: the inventory has been read and none failed.
pub fn none_failed(page: &CatalogPage, cx: &App) -> bool {
    match page.inventories.read(cx).known("services") {
        Known::Value(doc) => startup::failed(doc.get("items")).is_empty(),
        _ => false,
    }
}
