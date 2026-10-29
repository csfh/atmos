//! The Keybindings page's rows, lists and dialog (`bindings.rs` has what they say and send): Add
//! a binding in its dialog, each override with Remove…, which only Atmos's can use, and what
//! Hyprland has bound, filtered, each with Copy and Override…. Record shortcut listens for the
//! next key press (ODK's listening `Button`) and writes its chord, or the chord is typed.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::bindings::{self, BINDINGS, Draft, KEYBINDINGS, LIMIT};
use crate::pages::row;

const HINT: &str = "~/.config/hypr/bindings.lua";
const PRINT: &str = "omarchy menu keybindings --print";
const UNBIND_SAYS: &str = "The default chord is removed, with no replacement.";

const OPEN: &str = "bind-open";
const KEYS: &str = "bind-keys";
/// Whether Record shortcut listens.
const LISTEN: &str = "bind-listen";
const LABEL: &str = "bind-label";
const COMMAND: &str = "bind-command";
const UNBIND: &str = "bind-unbind";
const ERROR: &str = "bind-error";
const FILTER: &str = "bind-filter";
const SHOW_ALL: &str = "bind-all";

fn value(page: &CatalogPage, field: crate::catalog::Field, cx: &App) -> Option<Value> {
    field.read(page.stars.settings.read(cx)).value().cloned()
}

fn draft(page: &CatalogPage, key: &str) -> String {
    page.drafts.get(key).cloned().unwrap_or_default()
}

fn held(page: &CatalogPage) -> Draft {
    Draft {
        keys: draft(page, KEYS),
        label: draft(page, LABEL),
        command: draft(page, COMMAND),
        unbind: draft(page, UNBIND) == "1",
    }
}

/// Opens Add's dialog with the chord given, as the page's Add does.
pub fn open_add(page: &mut CatalogPage, keys: &str, cx: &mut Context<CatalogPage>) {
    open(page, keys, cx)
}

/// The chord Add's dialog holds, and whether Record listens, for what a test asks.
pub fn chord(page: &CatalogPage) -> (String, bool) {
    (draft(page, KEYS), draft(page, LISTEN) == "1")
}

/// Whether Add's dialog is open, for what a test asks.
pub fn adding(page: &CatalogPage) -> bool {
    draft(page, OPEN) == "1"
}

fn put(page: &mut CatalogPage, key: &str, value: impl Into<String>, cx: &mut Context<CatalogPage>) {
    page.drafts.insert(key.into(), value.into());
    cx.notify();
}

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
    page.stars.settings.update(cx, |s, cx| {
        s.send(
            &client,
            "bindings",
            &body,
            "The keybindings were not changed",
            window,
            cx,
        )
    });
}

/// `openAdd(keys)`: the dialog, with the chord given and nothing else.
fn open(page: &mut CatalogPage, keys: &str, cx: &mut Context<CatalogPage>) {
    let chord = bindings::sanitize_keys(keys);
    for (key, value) in [
        (KEYS, chord.as_str()),
        (LABEL, ""),
        (COMMAND, ""),
        (UNBIND, ""),
        (ERROR, ""),
        (LISTEN, ""),
        (OPEN, "1"),
    ] {
        page.drafts.insert(key.into(), value.into());
    }
    cx.notify();
}

/// `submitAdd()`: the binding, or what is wrong with it.
fn submit(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let catalog = value(page, KEYBINDINGS, cx);
    match held(page).submit(catalog.as_ref()) {
        Err(why) => put(page, ERROR, why, cx),
        Ok(add) => {
            put(page, ERROR, "", cx);
            put(page, OPEN, "", cx);
            let current = value(page, BINDINGS, cx);
            let (keys, label, command, unbind) = &add;
            let body = bindings::add(
                current.as_ref(),
                catalog.as_ref(),
                (keys, label, command, *unbind),
            );
            send(page, body, window, cx);
        }
    }
}

/// Add…, which opens the dialog.
pub fn add_button(id: String, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label("Add…")
        .variant(ButtonVariant::Primary)
        .on_click(move |_, _, cx| this.update(cx, |p, cx| open(p, "", cx)))
}

fn catalog(page: &CatalogPage, cx: &App) -> (Vec<Value>, usize) {
    let all = value(page, KEYBINDINGS, cx);
    let total = all.as_ref().and_then(Value::as_array).map_or(0, Vec::len);
    (bindings::catalog(all.as_ref(), &draft(page, FILTER)), total)
}

/// Whether a row of the page's own shows, once what it reads has been read.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let read = |f| value(page, f, cx).is_some();
    let (rows, total) = catalog(page, cx);
    match which {
        "add" => true,
        "none" => {
            read(BINDINGS) && bindings::overrides(value(page, BINDINGS, cx).as_ref()).is_empty()
        }
        "empty" => read(KEYBINDINGS) && total == 0,
        "filter" => total > 0,
        "nomatch" => total > 0 && rows.is_empty(),
        _ => false,
    }
}

/// The Filter row's words.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let (rows, total) = catalog(page, cx);
    (which == "filter").then(|| bindings::filter_says(rows.len(), total))
}

/// The control of a row of the page's own.
pub fn control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    match which {
        "add" => add_button(id, this).into_any_element(),
        "filter" => {
            let this = this.clone();
            Input::new(SharedString::from(id), "Filter")
                .placeholder("SUPER + Q or Close window")
                .value(draft(page, FILTER))
                .on_change(move |text, _, cx| {
                    let text = text.to_owned();
                    this.update(cx, |p, cx| {
                        p.drafts.remove(SHOW_ALL);
                        put(p, FILTER, text, cx)
                    })
                })
                .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}

/// Each override, with Remove…, which only Atmos's can use.
pub fn overrides(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    bindings::overrides(value(page, BINDINGS, cx).as_ref())
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let keys = crate::js::string(&r["keys"]);
            let asked = this.clone();
            let gone = keys.clone();
            let remove = Button::new(SharedString::from(format!("binding-{i}-remove")))
                .label("Remove…")
                .variant(ButtonVariant::Destructive)
                .disabled(r.get("managed") != Some(&Value::Bool(true)))
                .on_click(move |_, _, cx| {
                    let gone = gone.clone();
                    asked.update(cx, |p, cx| {
                        p.ask(
                            Asked {
                                title: "Remove binding".into(),
                                message: bindings::remove_message(&gone),
                                action: "Remove",
                                go: Rc::new(move |p, window, cx| {
                                    let current = value(p, BINDINGS, cx);
                                    send(p, bindings::remove(current.as_ref(), &gone), window, cx);
                                }),
                            },
                            cx,
                        )
                    })
                });
            let says = bindings::describe(&r);
            let star = page.stars.star(page.page().id, &keys, &says, cx);
            row::setting(keys, says, HINT, remove, star).into_any_element()
        })
        .collect()
}

/// What Hyprland has bound, filtered and drawn to the limit, each with Copy and Override…, then
/// More bindings while some are not drawn.
pub fn bound(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let (rows, _) = catalog(page, cx);
    let limit = if draft(page, SHOW_ALL) == "1" {
        rows.len()
    } else {
        LIMIT
    };
    let mut out: Vec<_> = rows
        .iter()
        .take(limit)
        .enumerate()
        .map(|(i, r)| {
            let keys = crate::js::string(&r["keys"]);
            let client = page.stars.client.clone();
            let copied = bindings::copy_text(r);
            let (chord, this) = (keys.clone(), this.clone());
            let control = h_flex()
                .gap(Space::Sm)
                .child(
                    Button::new(SharedString::from(format!("bound-{i}-copy")))
                        .label("Copy")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx)),
                )
                .child(
                    Button::new(SharedString::from(format!("bound-{i}-override")))
                        .label("Override…")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| this.update(cx, |p, cx| open(p, &chord, cx))),
                );
            let says = bindings::catalog_says(r);
            let star = page.stars.star(page.page().id, &keys, &says, cx);
            row::setting(keys, says, PRINT, control, star).into_any_element()
        })
        .collect();
    if rows.len() > limit {
        let this = this.clone();
        let says = bindings::more_says(limit, rows.len());
        let all = Button::new("bound-all")
            .label("Show all")
            .variant(ButtonVariant::Outline)
            .on_click(move |_, _, cx| this.update(cx, |p, cx| put(p, SHOW_ALL, "1", cx)));
        let star = page.stars.star(page.page().id, "More bindings", &says, cx);
        out.push(row::setting("More bindings", says, PRINT, all, star).into_any_element());
    }
    out
}

/// Record shortcut: pressed, it listens; the next key press is the chord, unless no binding can
/// take that key, and Escape or focus leaving stops it (`captureKey`).
fn record(listening: bool, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let (started, heard) = (this.clone(), this.clone());
    Button::new("bind-record")
        .label(if listening {
            "Listening…"
        } else {
            "Record shortcut"
        })
        .variant(if listening {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Outline
        })
        .listening(listening)
        .on_click(move |_, _, cx| started.update(cx, |p, cx| put(p, LISTEN, "1", cx)))
        .on_chord(move |chord, _, cx| {
            let keys =
                chord.map(|c| bindings::recorded(&c.key, c.platform, c.control, c.alt, c.shift));
            heard.update(cx, |p, cx| match keys {
                // A key no binding can take: still listening.
                Some(k) if k.is_empty() => {}
                Some(k) => {
                    p.drafts.insert(KEYS.into(), k);
                    put(p, LISTEN, "", cx)
                }
                None => put(p, LISTEN, "", cx),
            })
        })
}

fn form(page: &CatalogPage, this: &Entity<CatalogPage>, cx: &App) -> impl IntoElement + use<> {
    let d = held(page);
    let catalog = value(page, KEYBINDINGS, cx);
    let error = draft(page, ERROR);
    let listening = draft(page, LISTEN) == "1";
    let field = |id: &'static str,
                 label: &'static str,
                 placeholder: &'static str,
                 key: &'static str,
                 shown: String| {
        let (typed, entered) = (this.clone(), this.clone());
        Input::new(id, label)
            .placeholder(placeholder)
            .value(shown)
            .on_change(move |text, _, cx| typed.update(cx, |p, cx| put(p, key, text, cx)))
            .on_submit(move |text, window, cx| {
                entered.update(cx, |p, cx| {
                    p.drafts.insert(key.into(), text.to_owned());
                    submit(p, window, cx)
                })
            })
    };
    let unbinding = this.clone();
    let star = page
        .stars
        .star(page.page().id, "Unbind only", UNBIND_SAYS, cx);
    v_flex()
        .gap(Space::Md)
        .child(Text::new(d.says(catalog.as_ref(), listening)).muted())
        .child(field("bind-keys", "Chord", "SUPER + F", KEYS, d.keys.clone()).disabled(listening))
        .child(record(listening, this))
        .child(Text::new(d.line(catalog.as_ref())).muted())
        .child(field(
            "bind-label",
            "Name",
            "Name (optional)",
            LABEL,
            d.label.clone(),
        ))
        .children((!d.unbind).then(|| {
            field(
                "bind-command",
                "Command",
                "nautilus",
                COMMAND,
                d.command.clone(),
            )
        }))
        .child(row::setting(
            "Unbind only",
            UNBIND_SAYS,
            "",
            Switch::new("bind-unbind", "Unbind only")
                .checked(d.unbind)
                .on_change(move |on, _, cx| {
                    unbinding.update(cx, |p, cx| put(p, UNBIND, if on { "1" } else { "" }, cx))
                }),
            star,
        ))
        .children(
            (!error.is_empty()).then(|| Alert::new("bind-error", error).variant(Tone::Danger)),
        )
}

/// Add's dialog, built from the page each frame it is shown.
pub fn dialog(page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let ready = held(page).ready();
    let (built, closed, cancelled, added) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("bind-add", "Add a binding")
        .open(adding(page))
        .on_close(move |_, cx| closed.update(cx, |p, cx| put(p, OPEN, "", cx)))
        .content(move |_, cx| form(built.read(cx), &built, cx))
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("bind-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, |p, cx| put(p, OPEN, "", cx)))
        })
        .action(move |_, _| {
            let added = added.clone();
            Button::new("bind-go")
                .label("Add")
                .variant(ButtonVariant::Primary)
                .disabled(!ready)
                .on_click(move |_, window, cx| added.update(cx, |p, cx| submit(p, window, cx)))
        })
}
