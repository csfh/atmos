//! The Capture page's controls (`capture.rs` has what they say and send): the screenshot's mode
//! and place and Capture, the recording's switches, Start or Stop, the webcam's buttons, and the
//! save folders. The choices are the page's own, as the QML page's properties are: they start
//! where its do and are kept while the window lives.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::capture::{self, OVERLAY, RECORDING};
use crate::catalog::Field;
use crate::jobs::Job;
use crate::pages::controls::select;
use crate::pages::row;

const MODE: (&str, &str) = ("shotMode", "smart");
const DEST: (&str, &str) = ("shotDest", "slurp");
const SIZE: (&str, &str) = ("recWebcamSize", "medium");

fn choice(page: &CatalogPage, (key, or): (&str, &str)) -> String {
    page.drafts
        .get(key)
        .cloned()
        .unwrap_or_else(|| or.to_owned())
}

/// A recording switch, as the page holds it.
pub fn flag(page: &CatalogPage, key: &str) -> bool {
    let or = [
        capture::DESKTOP_AUDIO,
        capture::MICROPHONE,
        capture::WEBCAM,
        capture::FULLSCREEN,
    ]
    .iter()
    .find(|(k, _)| *k == key)
    .is_some_and(|(_, on)| *on);
    page.drafts.get(key).map_or(or, |v| v == "1")
}

fn on(page: &CatalogPage, field: Field, cx: &App) -> bool {
    field.read(page.stars.settings.read(cx)).value() == Some(&Value::Bool(true))
}

/// Whether a recording runs.
pub fn recording(page: &CatalogPage, cx: &App) -> bool {
    on(page, RECORDING, cx)
}

/// Whether the webcam overlay is up.
pub fn overlay(page: &CatalogPage, cx: &App) -> bool {
    on(page, OVERLAY, cx)
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
    page.stars.settings.update(cx, |settings, cx| {
        settings.send(&client, key, &body, "Capture did not answer", window, cx);
    });
}

/// Mode, Where it goes, or Webcam size: a choice the page holds.
pub fn pick(
    id: String,
    label: &'static str,
    which: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let (held, list, disabled) = match which {
        "mode" => (MODE, capture::MODES, false),
        "dest" => (DEST, capture::DESTS, false),
        _ => (SIZE, capture::SIZES, recording(page, cx)),
    };
    let options: Vec<(String, String)> = list
        .iter()
        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
        .collect();
    let now = Value::from(choice(page, held));
    let this = this.clone();
    select(id, label, &options, &now, disabled, move |v, _, cx| {
        let v = v.to_owned();
        this.update(cx, |p, cx| {
            p.drafts.insert(held.0.into(), v);
            cx.notify();
        })
    })
    .into_any_element()
}

/// Take screenshot: Capture, or Capture… when a picker opens.
pub fn take(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let (mode, dest) = (choice(page, MODE), choice(page, DEST));
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label(capture::capture_label(&mode))
        .variant(ButtonVariant::Primary)
        .on_click(move |_, window, cx| {
            let Some(args) = capture::screenshot(&mode, &dest) else {
                return;
            };
            this.update(cx, |p, cx| {
                let job = Job::new("capture.screenshot", args, "screenshot", "The screenshot");
                p.jobs
                    .update(cx, |jobs, cx| jobs.run(job.interactive(), window, cx));
            })
        })
}

/// A recording switch, off while a recording runs.
pub fn switch(
    id: String,
    label: &'static str,
    key: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let this = this.clone();
    Switch::new(SharedString::from(id), label)
        .checked(flag(page, key))
        .disabled(recording(page, cx))
        .on_change(move |next, _, cx| {
            this.update(cx, |p, cx| {
                p.drafts
                    .insert(key.into(), if next { "1" } else { "" }.into());
                cx.notify();
            })
        })
}

/// Resize webcam: Smaller, Larger and Reset.
pub fn resize(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let up = overlay(page, cx);
    h_flex()
        .gap(Space::Sm)
        .children(capture::RESIZE.iter().map(|(label, action)| {
            let this = this.clone();
            Button::new(SharedString::from(format!("{id}-{action}")))
                .label(*label)
                .variant(ButtonVariant::Outline)
                .disabled(!up)
                .on_click(move |_, window, cx| {
                    this.update(cx, |p, cx| {
                        send(p, "webcam.resize", capture::resize(action), window, cx)
                    })
                })
        }))
}

fn dir(page: &CatalogPage, key: &'static str, cx: &App) -> String {
    Field::Key(key)
        .read(page.stars.settings.read(cx))
        .value()
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

/// A save folder's description.
pub fn folder_says(page: &CatalogPage, key: &'static str, cx: &App) -> String {
    let which = if key == "picturesDir" {
        "Pictures"
    } else {
        "Videos"
    };
    capture::folder_says(&dir(page, key, cx), which)
}

/// A save folder: Open folder and Copy.
pub fn folder(
    id: String,
    key: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let path = dir(page, key, cx);
    let (opened, client) = (this.clone(), page.stars.client.clone());
    let copied = path.clone();
    h_flex()
        .gap(Space::Sm)
        .child(
            Button::new(SharedString::from(format!("{id}-open")))
                .label("Open folder")
                .variant(ButtonVariant::Outline)
                .disabled(path.is_empty())
                .on_click(move |_, window, cx| {
                    let body = capture::open_dir(&path);
                    opened.update(cx, |p, cx| send(p, "open.path", body, window, cx))
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-copy")))
                .label("Copy")
                .variant(ButtonVariant::Outline)
                .disabled(copied.is_empty())
                .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx)),
        )
}

/// The recording's row: Start, or Stop while a recording runs. Its label follows that, so
/// search does not list it (`search.query` reads the labels a page spells out).
pub fn recording_row(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let running = recording(page, cx);
    let (label, says) = (
        capture::recording_label(running),
        capture::recording_says(running),
    );
    let args = capture::recording(
        flag(page, capture::DESKTOP_AUDIO.0),
        flag(page, capture::MICROPHONE.0),
        flag(page, capture::WEBCAM.0),
        &choice(page, SIZE),
        flag(page, capture::FULLSCREEN.0),
    );
    let this = this.clone();
    let control = if running {
        Button::new("capture-stop")
            .label("Stop")
            .variant(ButtonVariant::Destructive)
            .on_click(move |_, window, cx| {
                this.update(cx, |p, cx| {
                    send(p, "recordingActive", Some(capture::stop()), window, cx)
                })
            })
    } else {
        Button::new("capture-start")
            .label(capture::start_label(flag(page, capture::FULLSCREEN.0)))
            .variant(ButtonVariant::Primary)
            .on_click(move |_, window, cx| {
                let job = Job::new(
                    "capture.recording",
                    args.clone(),
                    "recording",
                    "The recording",
                );
                this.update(cx, |p, cx| {
                    p.jobs
                        .update(cx, |jobs, cx| jobs.run(job.interactive(), window, cx))
                })
            })
    };
    let star = page.stars.star(page.page().id, label, says, cx);
    row::setting(
        label,
        says,
        "omarchy capture screenrecording",
        control,
        star,
    )
}
