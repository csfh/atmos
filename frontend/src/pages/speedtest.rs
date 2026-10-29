//! The Speed test page's rows (`speedtest.rs` has the run and what each row says): opening the
//! page starts a run, as the QML page's does, and leaving it stops listening for the answer.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::pages::row;
use crate::speedtest::{self, NET, Run};

const ID: &str = "network/speedtest";
const RUN: &str = "speedtest";

fn run(page: &CatalogPage) -> Run {
    Run::of(page.data.get(RUN))
}

fn save(page: &mut CatalogPage, run: &Run, cx: &mut Context<CatalogPage>) {
    page.data.insert(RUN, run.to_value());
    cx.notify();
}

fn disconnected(page: &CatalogPage, cx: &App) -> bool {
    NET.read(page.stars.settings.read(cx)).value() == Some(&Value::from("disconnected"))
}

/// Sends the phase the run is in, and gives it five seconds.
fn go(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    let r = run(page);
    let token = r.token;
    let reply = page.stars.client.request(&speedtest::request(&r.phase));
    cx.spawn(async move |this, cx| {
        let outcome = reply.await;
        let _ = this.update(cx, |p, cx| {
            let mut r = run(p);
            if r.token != token {
                return;
            }
            let next = match &outcome {
                Ok(result) => r.answered(Ok(result)),
                Err(f) => r.answered(Err(&f.message)),
            };
            then(p, r, next, cx);
        });
    })
    .detach();
    cx.spawn(async move |this, cx| {
        cx.background_executor().timer(speedtest::PHASE).await;
        let _ = this.update(cx, |p, cx| {
            let mut r = run(p);
            if r.token != token {
                return;
            }
            let next = r.timed_out();
            then(p, r, next, cx);
        });
    })
    .detach();
}

/// Keeps the run, and begins its next phase if it has one.
fn then(
    page: &mut CatalogPage,
    mut r: Run,
    next: Option<&'static str>,
    cx: &mut Context<CatalogPage>,
) {
    if let Some(phase) = next {
        r.begin(phase);
        save(page, &r, cx);
        go(page, cx);
    } else {
        save(page, &r, cx);
    }
}

/// `startSpeedtest`.
fn start(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    let mut r = run(page);
    let started = r.start(disconnected(page, cx));
    save(page, &r, cx);
    if started {
        go(page, cx);
    }
}

/// The page was shown: the speed test starts a run, and any other page stops one that runs.
pub fn loaded(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    if page.page().id == ID {
        start(page, cx);
    } else if page.data.contains_key(RUN) {
        let mut r = run(page);
        r.stop();
        save(page, &r, cx);
    }
}

/// What a row says, by part (`run`, `progress`, `error`, `down`, `up`).
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let (r, off) = (run(page), disconnected(page, cx));
    Some(match which {
        "run" => r.phase_label(off).to_owned(),
        "progress" => r.progress_says().to_owned(),
        "error" => r.error,
        "down" => r.down_says(off),
        "up" => r.up_says(off),
        _ => return None,
    })
}

/// Whether a row shows: Could not run, while a run failed.
pub fn shown(which: &str, page: &CatalogPage) -> bool {
    which != "error" || !run(page).error.is_empty()
}

/// A row's control, by part.
pub fn control(
    which: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let (r, off) = (run(page), disconnected(page, cx));
    let this = this.clone();
    match which {
        "run" => Button::new(SharedString::from(id))
            .label(if r.running { "Cancel" } else { "Run now" })
            .variant(if r.running {
                ButtonVariant::Destructive
            } else {
                ButtonVariant::Primary
            })
            .disabled(!r.running && off)
            .on_click(move |_, _, cx| {
                this.update(cx, |p, cx| {
                    let mut r = run(p);
                    if r.running {
                        r.stop();
                        save(p, &r, cx);
                    } else {
                        start(p, cx);
                    }
                })
            })
            .into_any_element(),
        "progress" => h_flex()
            .gap(Space::Sm)
            .children(
                r.running
                    .then(|| Spinner::new(SharedString::from(format!("{id}-spin")))),
            )
            .child(
                Progress::new(SharedString::from(id), "Progress")
                    .value(r.progress())
                    .value_text(r.progress_text()),
            )
            .into_any_element(),
        "error" => Button::new(SharedString::from(id))
            .label("Retry")
            .variant(ButtonVariant::Outline)
            .disabled(r.running || off)
            .on_click(move |_, _, cx| this.update(cx, start))
            .into_any_element(),
        _ => {
            let (raw, direction) = if which == "down" {
                (r.down, "Download")
            } else {
                (r.up, "Upload")
            };
            let client = page.stars.client.clone();
            let copied = speedtest::copy_text(direction, &raw);
            h_flex()
                .gap(Space::Sm)
                .child(Text::new(speedtest::rate(&raw)))
                .child(
                    Button::new(SharedString::from(format!("{id}-copy")))
                        .label("Copy")
                        .variant(ButtonVariant::Outline)
                        .disabled(raw.is_empty())
                        .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx)),
                )
                .into_any_element()
        }
    }
}
