//! The Services page (`units.rs` has what it says and sends): the summary, the search and the
//! state filters, each unit with its main action and a menu of the rest, and a dialog with a
//! unit's status or logs. A unit of the system's scope is changed as root, behind the sudo
//! prompt, as the QML page's `runCommand(..., { sudo: true })` is.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::jobs::RootWrite;
use crate::pages::row;
use crate::units::{self, UNITS};

const QUERY: &str = "units-query";
const FILTER: &str = "units-filter";
const OUTPUT: &str = "units-output";
const TITLE: &str = "units-output-title";

fn all(page: &CatalogPage, cx: &App) -> Vec<Value> {
    units::all(UNITS.read(page.stars.settings.read(cx)).value())
}

fn draft(page: &CatalogPage, key: &str, or: &str) -> String {
    page.drafts
        .get(key)
        .cloned()
        .unwrap_or_else(|| or.to_owned())
}

fn put(page: &mut CatalogPage, key: &str, value: impl Into<String>, cx: &mut Context<CatalogPage>) {
    page.drafts.insert(key.into(), value.into());
    cx.notify();
}

/// A unit's action: Copy, its status or logs, or a change (`actOn`).
fn act(
    page: &mut CatalogPage,
    row: &Value,
    action: &str,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let unit = crate::js::string(&row["unit"]);
    let scope = crate::js::string(&row["scope"]);
    match action {
        "copy" => row::copy(&page.stars.client.clone(), &unit, window, cx),
        "status" | "logs" => {
            put(page, TITLE, units::output_title(action, &unit), cx);
            put(page, OUTPUT, "Reading…", cx);
            let reply = page
                .stars
                .client
                .request(&crate::protocol::unit_output(action, &scope, &unit));
            let title = units::output_title(action, &unit);
            cx.spawn(async move |this, cx| {
                let outcome = reply.await;
                let said = match &outcome {
                    Ok(result) => units::output_text(Ok(result)),
                    Err(f) => units::output_text(Err(&f.message)),
                };
                let _ = this.update(cx, |p, cx| {
                    // Only for the dialog still asked for.
                    if p.drafts.get(TITLE) == Some(&title) {
                        put(p, OUTPUT, said, cx);
                    }
                });
            })
            .detach();
        }
        _ if crate::js::truthy(row.get("allowed")) => {
            let Some((body, root)) = units::action(action, &unit, &scope) else {
                return;
            };
            let refused = format!("{unit} did not {action}");
            if root {
                let write = RootWrite {
                    key: "systemd",
                    body,
                    refused,
                };
                page.jobs
                    .update(cx, |jobs, cx| jobs.write_as_root(write, window, cx));
            } else {
                let client = page.stars.client.clone();
                page.stars.settings.update(cx, |s, cx| {
                    s.send(&client, "systemd", &body, refused, window, cx)
                });
            }
        }
        _ => {}
    }
}

/// The summary, the search and the filters.
fn head(page: &CatalogPage, this: &Entity<CatalogPage>, cx: &App) -> impl IntoElement + use<> {
    let rows = all(page, cx);
    let filter = draft(page, FILTER, "all");
    let bits = units::summary(&rows).into_iter().map(|(id, said, failed)| {
        if failed {
            let this = this.clone();
            Button::new(SharedString::from(format!("units-{id}")))
                .label(said)
                .variant(ButtonVariant::Destructive)
                .on_click(move |_, _, cx| this.update(cx, |p, cx| put(p, FILTER, "failed", cx)))
                .into_any_element()
        } else {
            Text::new(said).muted().into_any_element()
        }
    });
    let typed = this.clone();
    let chips = units::CHIPS.iter().map(|(id, label)| {
        let this = this.clone();
        Button::new(SharedString::from(format!("units-chip-{id}")))
            .label(*label)
            .variant(if filter == *id {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            })
            .on_click(move |_, _, cx| this.update(cx, |p, cx| put(p, FILTER, *id, cx)))
    });
    v_flex()
        .gap(Space::Sm)
        .child(h_flex().flex_wrap().gap(Space::Md).children(bits))
        .child(
            SearchInput::new("units-search", "Search services")
                .placeholder("Search services…")
                .value(draft(page, QUERY, ""))
                .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, QUERY, t, cx))),
        )
        .child(h_flex().flex_wrap().gap(Space::Sm).children(chips))
}

/// The page: its head, each unit, and what an empty list says.
pub fn rows(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let every = all(page, cx);
    let shown = units::shown(&every, &draft(page, QUERY, ""), &draft(page, FILTER, "all"));
    let mut out = vec![head(page, this, cx).into_any_element()];
    for (i, r) in shown.iter().enumerate() {
        let primary = units::primary(r).map(|(id, label)| {
            let (this, r) = (this.clone(), r.clone());
            Button::new(SharedString::from(format!("unit-{i}-{id}")))
                .label(label)
                .variant(ButtonVariant::Outline)
                .on_click(move |_, window, cx| this.update(cx, |p, cx| act(p, &r, id, window, cx)))
        });
        let (menu_this, menu_row) = (this.clone(), r.clone());
        let more = units::overflow(r);
        let menu = DropdownMenu::new(SharedString::from(format!("unit-{i}-more")), "More").menu(
            move |m| {
                more.iter().fold(m, |m, (id, label)| {
                    let (this, r, id) = (menu_this.clone(), menu_row.clone(), *id);
                    m.item(MenuItem::new(*label).on_select(move |window, cx| {
                        this.update(cx, |p, cx| act(p, &r, id, window, cx))
                    }))
                })
            },
        );
        let control = h_flex().gap(Space::Sm).children(primary).child(menu);
        let label = crate::js::string(&r["unit"]);
        // A unit's row has no star: the QML page keeps it out of the catalog.
        out.push(
            row::setting(
                label,
                units::status_line(r),
                units::hint(r),
                control,
                v_flex(),
            )
            .into_any_element(),
        );
    }
    if shown.is_empty() {
        let says = units::empty_says(!every.is_empty());
        out.push(
            row::setting("No matching services", says, "", v_flex(), v_flex()).into_any_element(),
        );
    }
    out
}

/// The dialog with a unit's status or logs.
pub fn dialog(page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let title = draft(page, TITLE, "");
    let text = draft(page, OUTPUT, "");
    let (closed, done) = (this.clone(), this.clone());
    let close = |p: &mut CatalogPage, cx: &mut Context<CatalogPage>| {
        p.drafts.remove(TITLE);
        cx.notify();
    };
    Dialog::new("units-output", title.clone())
        .open(!title.is_empty())
        .on_close(move |_, cx| closed.update(cx, close))
        .content(move |_, _| scroll_y("units-output-text").child(Text::new(text.clone()).mono()))
        .action(move |_, _| {
            let done = done.clone();
            Button::new("units-output-close")
                .label("Close")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| done.update(cx, close))
        })
}

/// The dialog's title while it is open, for what a test asks.
pub fn showing(page: &CatalogPage) -> Option<String> {
    page.drafts.get(TITLE).cloned()
}
