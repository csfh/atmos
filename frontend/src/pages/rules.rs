//! The Window rules page's dialog and list (`rules.rs` has what they say and send): Add a rule
//! in its dialog, as the QML page has it, and each rule with Remove…, which only Atmos's rules
//! can use.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::pages::row;
use crate::rules::{self, Draft, FOCUSED, RULES};

const HINT: &str = "~/.config/hypr/atmos.lua";
const INTRO: &str =
    "Match is the window class Hyprland sees. Regex is fine. Leave size at 0 to skip it.";
const CENTER_SAYS: &str = "A floating window sits in the middle of the screen.";

const OPEN: &str = "rule-open";
const MATCH: &str = "rule-match";
const PLACEMENT: &str = "rule-placement";
const CENTER: &str = "rule-center";
const WIDTH: &str = "rule-width";
const HEIGHT: &str = "rule-height";
const WORKSPACE: &str = "rule-workspace";
const ERROR: &str = "rule-error";

fn list(page: &CatalogPage, cx: &App) -> Option<Value> {
    RULES.read(page.stars.settings.read(cx)).value().cloned()
}

fn focused(page: &CatalogPage, cx: &App) -> String {
    FOCUSED
        .read(page.stars.settings.read(cx))
        .value()
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

fn draft(page: &CatalogPage, key: &str) -> String {
    page.drafts.get(key).cloned().unwrap_or_default()
}

fn number(page: &CatalogPage, key: &str) -> f64 {
    draft(page, key).parse().unwrap_or(0.0)
}

/// What the dialog holds.
fn held(page: &CatalogPage) -> Draft {
    Draft {
        matches: draft(page, MATCH),
        placement: draft(page, PLACEMENT),
        center: draft(page, CENTER) == "1",
        width: number(page, WIDTH),
        height: number(page, HEIGHT),
        workspace: draft(page, WORKSPACE),
    }
}

/// Whether Add's dialog is open, for what a test asks.
pub fn adding(page: &CatalogPage) -> bool {
    draft(page, OPEN) == "1"
}

/// Sends a `windowRules` write, when there is one.
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
            "windowRules",
            &body,
            "The window rules were not changed",
            window,
            cx,
        );
    });
}

fn put(page: &mut CatalogPage, key: &str, value: impl Into<String>, cx: &mut Context<CatalogPage>) {
    page.drafts.insert(key.into(), value.into());
    cx.notify();
}

/// `openAdd`: the dialog, floating and centered, with nothing else set.
fn open(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    for (key, value) in [
        (MATCH, ""),
        (PLACEMENT, "float"),
        (CENTER, "1"),
        (WIDTH, "0"),
        (HEIGHT, "0"),
        (WORKSPACE, ""),
        (ERROR, ""),
        (OPEN, "1"),
    ] {
        page.drafts.insert(key.into(), value.into());
    }
    cx.notify();
}

/// `submitAdd`: the rule, or what is wrong with it.
fn submit(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    match rules::submit(&held(page)) {
        Err(why) => put(page, ERROR, why, cx),
        Ok((m, placement, center, width, height, workspace)) => {
            put(page, ERROR, "", cx);
            put(page, OPEN, "", cx);
            let current = list(page, cx);
            let (w, h) = (
                crate::js::json_number(width),
                crate::js::json_number(height),
            );
            let body = rules::add(current.as_ref(), &m, &placement, center, &w, &h, &workspace);
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
        .on_click(move |_, _, cx| this.update(cx, open))
}

/// Whether "No personal window rules." shows: the rules have been read and there are none.
pub fn none(page: &CatalogPage, cx: &App) -> bool {
    RULES
        .read(page.stars.settings.read(cx))
        .value()
        .is_some_and(|v| rules::rows(Some(v)).is_empty())
}

/// Add's form: the match and Use focused, the placement, Center, the size, the workspace, and
/// what is wrong.
fn form(page: &CatalogPage, this: &Entity<CatalogPage>, cx: &App) -> impl IntoElement + use<> {
    let d = held(page);
    let error = draft(page, ERROR);
    let focused = focused(page, cx);
    let star = page.stars.star(page.page().id, "Center", CENTER_SAYS, cx);
    let (lo, hi, step) = rules::SIZE;
    let (typed, entered, used, placed) = (this.clone(), this.clone(), this.clone(), this.clone());
    let (centered, wide, tall) = (this.clone(), this.clone(), this.clone());
    let (spaced, sent) = (this.clone(), this.clone());
    let placements = rules::PLACEMENTS.iter().fold(
        RadioGroup::new("rule-placement", "Placement").horizontal(true),
        |g, (value, label)| g.item(*value, *label),
    );
    v_flex()
        .gap(Space::Md)
        .child(
            h_flex()
                .gap(Space::Sm)
                .child(
                    Input::new("rule-match", "Match")
                        .placeholder("firefox")
                        .value(d.matches.clone())
                        .on_change(move |text, _, cx| {
                            typed.update(cx, |p, cx| put(p, MATCH, text, cx))
                        })
                        .on_submit(move |text, window, cx| {
                            entered.update(cx, |p, cx| {
                                p.drafts.insert(MATCH.into(), text.to_owned());
                                submit(p, window, cx)
                            })
                        }),
                )
                .child(
                    Button::new("rule-focused")
                        .label("Use focused")
                        .variant(ButtonVariant::Outline)
                        .disabled(focused.is_empty())
                        .on_click(move |_, _, cx| {
                            let class = focused.clone();
                            used.update(cx, |p, cx| put(p, MATCH, class, cx))
                        }),
                ),
        )
        .child(
            placements
                .value(d.placement.clone())
                .on_change(move |value, _, cx| {
                    placed.update(cx, |p, cx| put(p, PLACEMENT, value, cx))
                }),
        )
        .child(row::setting(
            "Center",
            CENTER_SAYS,
            "",
            Switch::new("rule-center", "Center")
                .checked(d.center)
                .on_change(move |on, _, cx| {
                    centered.update(cx, |p, cx| put(p, CENTER, if on { "1" } else { "" }, cx))
                }),
            star,
        ))
        .child(
            h_flex()
                .gap(Space::Sm)
                .child(
                    NumberInput::new("rule-width", "Width")
                        .value(d.width)
                        .min(lo)
                        .max(hi)
                        .step(step)
                        .on_commit(move |n, _, cx| {
                            wide.update(cx, |p, cx| put(p, WIDTH, n.round().to_string(), cx))
                        }),
                )
                .child(
                    NumberInput::new("rule-height", "Height")
                        .value(d.height)
                        .min(lo)
                        .max(hi)
                        .step(step)
                        .on_commit(move |n, _, cx| {
                            tall.update(cx, |p, cx| put(p, HEIGHT, n.round().to_string(), cx))
                        }),
                ),
        )
        .child(
            Input::new("rule-workspace", "Workspace")
                .placeholder("Workspace (optional)")
                .value(d.workspace)
                .on_change(move |text, _, cx| {
                    spaced.update(cx, |p, cx| put(p, WORKSPACE, text, cx))
                })
                .on_submit(move |text, window, cx| {
                    sent.update(cx, |p, cx| {
                        p.drafts.insert(WORKSPACE.into(), text.to_owned());
                        submit(p, window, cx)
                    })
                }),
        )
        .children(
            (!error.is_empty()).then(|| Alert::new("rule-error", error).variant(Tone::Danger)),
        )
}

/// Add's dialog, built from the page each frame it is shown.
pub fn dialog(page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let can_add = !draft(page, MATCH).is_empty();
    let (built, closed, cancelled, added) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("rule-add", "Add a window rule")
        .open(adding(page))
        .description(INTRO)
        .on_close(move |_, cx| closed.update(cx, |p, cx| put(p, OPEN, "", cx)))
        .content(move |_, cx| form(built.read(cx), &built, cx))
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("rule-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, |p, cx| put(p, OPEN, "", cx)))
        })
        .action(move |_, _| {
            let added = added.clone();
            Button::new("rule-go")
                .label("Add")
                .variant(ButtonVariant::Primary)
                .disabled(!can_add)
                .on_click(move |_, window, cx| added.update(cx, |p, cx| submit(p, window, cx)))
        })
}

/// Each rule, with Remove…, which only Atmos's rules can use.
pub fn rows(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let current = list(page, cx);
    rules::rows(current.as_ref())
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let label = rules::label(&r);
            let m = crate::js::string(&r["match"]);
            let managed = crate::js::truthy(r.get("managed"));
            let asked = this.clone();
            let remove = Button::new(SharedString::from(format!("rule-{i}-remove")))
                .label("Remove…")
                .variant(ButtonVariant::Destructive)
                .disabled(!managed)
                .on_click(move |_, _, cx| {
                    let gone = m.clone();
                    asked.update(cx, |p, cx| {
                        p.ask(
                            Asked {
                                title: "Remove window rule".into(),
                                message: rules::remove_message(&gone),
                                action: "Remove",
                                go: Rc::new(move |p, window, cx| {
                                    let current = list(p, cx);
                                    let body = rules::remove(current.as_ref(), &gone);
                                    send(p, body, window, cx);
                                }),
                            },
                            cx,
                        )
                    })
                });
            let description = rules::description(&r);
            let star = page.stars.star(page.page().id, &label, &description, cx);
            row::setting(label, description, HINT, remove, star).into_any_element()
        })
        .collect()
}
