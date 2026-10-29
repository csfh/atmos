//! The History page (`history.rs` has the log): Preview and what it holds, with Apply and
//! Discard, and every change this window made, newest first.

use odk::prelude::*;

use super::catalog::CatalogPage;
use crate::history::{self, Log};
use crate::jobs::RootWrite;
use crate::pages::row;

fn log(page: &CatalogPage, cx: &App) -> Log {
    page.stars.settings.read(cx).log.clone()
}

/// Apply: everything held is sent, oldest first, a root write behind the sudo prompt.
fn apply(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let held = page.stars.settings.update(cx, |s, _| s.log.take_held());
    let client = page.stars.client.clone();
    for h in held {
        if h.root {
            let write = RootWrite {
                key: h.key,
                body: h.body,
                refused: h.refused,
            };
            page.jobs
                .update(cx, |jobs, cx| jobs.run_as_root(write, window, cx));
        } else {
            page.stars.settings.update(cx, |s, cx| {
                s.send_now(&client, h.key, &h.body, h.refused, false, window, cx)
            });
        }
    }
    page.stars.settings.update(cx, |_, cx| cx.notify());
}

fn discard(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    page.stars.settings.update(cx, |s, cx| {
        s.log.held.clear();
        cx.notify();
    });
}

/// What a row of the page's own says (`preview`, `held`), when it is built from the log.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let log = log(page, cx);
    match which {
        "preview" => Some(history::preview_says(log.preview).to_owned()),
        "held" => Some(history::held_says(log.held.len())),
        _ => None,
    }
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let log = log(page, cx);
    match which {
        "held" => !log.held.is_empty(),
        "none" => log.changes.is_empty(),
        "you" => !log.changes.is_empty(),
        _ => true,
    }
}

/// A row of the page's own: its control.
pub fn control(
    which: &'static str,
    id: String,
    label: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let log = log(page, cx);
    let this = this.clone();
    match which {
        "preview" => Switch::new(SharedString::from(id), label)
            .checked(log.preview)
            .on_change(move |on, _, cx| {
                this.update(cx, |p, cx| {
                    p.stars.settings.update(cx, |s, cx| {
                        s.log.preview = on;
                        if !on {
                            s.log.held.clear();
                        }
                        cx.notify();
                    })
                })
            })
            .into_any_element(),
        "held" => {
            let discarded = this.clone();
            h_flex()
                .gap(Space::Sm)
                .child(
                    Button::new(SharedString::from(format!("{id}-apply")))
                        .label("Apply")
                        .variant(ButtonVariant::Primary)
                        .on_click(move |_, window, cx| {
                            this.update(cx, |p, cx| apply(p, window, cx))
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("{id}-discard")))
                        .label("Discard")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| discarded.update(cx, discard)),
                )
                .into_any_element()
        }
        "you" => Text::new(log.by_you().to_string()).into_any_element(),
        _ => v_flex().into_any_element(),
    }
}

/// What Preview holds, each marked held.
pub fn held(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    log(page, cx)
        .held
        .iter()
        .map(|h| {
            let label = history::row_label(&h.change, "");
            let star = page.stars.star(page.page().id, &label, &h.change.text, cx);
            row::setting(label, h.change.text.clone(), "", Text::new("held"), star)
                .into_any_element()
        })
        .collect()
}

/// Each change, newest first, with how long ago.
pub fn changes(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let now = crate::settings::now_ms();
    log(page, cx)
        .changes
        .iter()
        .map(|c| {
            let label = history::row_label(c, "change");
            let star = page.stars.star(page.page().id, &label, &c.text, cx);
            let ago = history::relative_time(c.at, now);
            row::setting(label, c.text.clone(), "", Text::new(ago), star).into_any_element()
        })
        .collect()
}
