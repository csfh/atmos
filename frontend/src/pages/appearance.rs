//! The Appearance page's controls (`appearance.rs` has what they say and send): a button for
//! each theme, the extra themes with Update all and Remove…, Add a theme in its dialog, and the
//! night light schedule. The QML page's theme cards also show four colours of each theme and
//! preview one on hover; `odk::prelude` has no swatch and no theme preview yet, so the buttons
//! carry the names only.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::appearance::{self, DAY, EXTRAS, NIGHT, NIGHT_ON, THEME, THEMES};
use crate::catalog::Field;
use crate::pages::controls::select;
use crate::pages::row;

const PICKED: &str = "theme-extra";
const URL: &str = "theme-url";
const ADDING: &str = "theme-adding";
const DAY_DRAFT: &str = "nightlight-day";
const NIGHT_DRAFT: &str = "nightlight-night";

fn read(page: &CatalogPage, field: Field, cx: &App) -> Option<Value> {
    field.read(page.stars.settings.read(cx)).value().cloned()
}

fn text(page: &CatalogPage, field: Field, cx: &App) -> String {
    read(page, field, cx).map_or_else(String::new, |v| crate::js::string(&v))
}

/// Whether the job `kind` is the one running.
fn running(page: &CatalogPage, kind: &str, cx: &App) -> bool {
    page.jobs.read(cx).running() == Some(kind)
}

/// Each theme, the one in use marked.
pub fn themes(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let current = text(page, THEME, cx);
    let all = appearance::names(read(page, THEMES, cx).as_ref());
    h_flex()
        .flex_wrap()
        .gap(Space::Sm)
        .children(all.into_iter().enumerate().map(|(i, name)| {
            let on = name == current;
            let this = this.clone();
            let chosen = name.clone();
            Button::new(SharedString::from(format!("theme-{i}")))
                .label(name)
                .variant(if on {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                })
                .on_click(move |_, window, cx| {
                    let chosen = Value::from(chosen.clone());
                    this.update(cx, |p, cx| {
                        p.call(
                            "Theme",
                            Some(THEME),
                            appearance::SET_THEME,
                            chosen,
                            window,
                            cx,
                        )
                    })
                })
        }))
}

/// Whether the theme buttons show: there are themes.
pub fn has_themes(page: &CatalogPage, cx: &App) -> bool {
    !appearance::names(read(page, THEMES, cx).as_ref()).is_empty()
}

/// The extra themes, once read: `Some(true)` while there are none.
pub fn no_extras(page: &CatalogPage, cx: &App) -> Option<bool> {
    read(page, EXTRAS, cx).map(|v| appearance::names(Some(&v)).is_empty())
}

pub fn extras_says(page: &CatalogPage, cx: &App) -> String {
    appearance::extras_says(appearance::names(read(page, EXTRAS, cx).as_ref()).len())
}

/// The extra themes: pick one to apply, Update all, and Remove….
pub fn extras(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let extras = appearance::names(read(page, EXTRAS, cx).as_ref());
    let draft = page.drafts.get(PICKED).cloned().unwrap_or_default();
    let picked = appearance::picked(&extras, &draft);
    let options: Vec<(String, String)> = extras.iter().map(|e| (e.clone(), e.clone())).collect();
    let busy = page.jobs.read(cx).job_busy();
    let updating = running(page, "theme-update", cx);
    let (chose, updated, asked) = (this.clone(), this.clone(), this.clone());
    let gone = picked.clone();
    v_flex()
        .gap(Space::Sm)
        .child(select(
            format!("{id}-pick"),
            "Extra theme",
            &options,
            &Value::from(picked.clone()),
            extras.is_empty(),
            move |v, window, cx| {
                let v = v.to_owned();
                chose.update(cx, |p, cx| {
                    p.drafts.insert(PICKED.into(), v.clone());
                    let set = Value::from(v);
                    p.call("Theme", Some(THEME), appearance::SET_THEME, set, window, cx)
                })
            },
        ))
        .child(
            h_flex()
                .gap(Space::Sm)
                .child(
                    Button::new(SharedString::from(format!("{id}-update")))
                        .label(if updating {
                            "Updating…"
                        } else {
                            "Update all"
                        })
                        .variant(ButtonVariant::Outline)
                        .disabled(busy || extras.is_empty())
                        .on_click(move |_, window, cx| {
                            updated.update(cx, |p, cx| {
                                let s = appearance::UPDATE_THEMES;
                                p.call("Update all", None, s, Value::Null, window, cx)
                            })
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("{id}-remove")))
                        .label("Remove…")
                        .variant(ButtonVariant::Destructive)
                        .disabled(picked.is_empty())
                        .on_click(move |_, _, cx| {
                            let gone = gone.clone();
                            asked.update(cx, |p, cx| {
                                let name = Value::from(gone.clone());
                                p.ask(
                                    Asked {
                                        title: "Remove this theme".into(),
                                        message: appearance::remove_message(&gone),
                                        action: "Remove",
                                        go: Rc::new(move |p, window, cx| {
                                            let (s, v) = (appearance::REMOVE_THEME, name.clone());
                                            p.call("The theme", None, s, v, window, cx)
                                        }),
                                    },
                                    cx,
                                )
                            })
                        }),
                ),
        )
}

/// What Add a theme says: while the clone runs, and before.
pub fn add_says(page: &CatalogPage, cx: &App) -> String {
    if running(page, "theme-install", cx) {
        appearance::INSTALLING
    } else {
        appearance::ADD_SAYS
    }
    .to_owned()
}

/// Add…, which opens Add's dialog (`openAddTheme`).
pub fn add_button(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label("Add…")
        .variant(ButtonVariant::Primary)
        .disabled(page.jobs.read(cx).job_busy())
        .on_click(move |_, _, cx| {
            this.update(cx, |p, cx| {
                p.drafts.insert(URL.into(), String::new());
                p.drafts.insert(ADDING.into(), "1".into());
                cx.notify();
            })
        })
}

/// Whether Add's dialog is open, for what a test asks.
pub fn adding(page: &CatalogPage) -> bool {
    page.drafts.get(ADDING).is_some_and(|a| a == "1")
}

fn close(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    page.drafts.remove(ADDING);
    cx.notify();
}

/// `submitAddTheme`: installs a URL that parses, while no job runs.
fn submit(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let url = appearance::parse_git_url(page.drafts.get(URL).map_or("", String::as_str));
    if url.is_empty() || page.jobs.read(cx).job_busy() {
        return;
    }
    let (s, v) = (appearance::INSTALL_THEME, Value::from(url));
    page.call("The theme", None, s, v, window, cx);
    close(page, cx);
}

/// Add a theme's dialog: a git URL, and Install.
pub fn dialog(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let draft = page.drafts.get(URL).cloned().unwrap_or_default();
    let valid = !appearance::parse_git_url(&draft).is_empty();
    let busy = page.jobs.read(cx).job_busy();
    let says = if running(page, "theme-install", cx) && busy {
        appearance::INSTALLING
    } else if !draft.is_empty() && !valid {
        appearance::BAD_URL
    } else {
        appearance::ADD_INTRO
    };
    let (closed, typed, entered, cancelled, went) = (
        this.clone(),
        this.clone(),
        this.clone(),
        this.clone(),
        this.clone(),
    );
    Dialog::new("theme-add", "Add a theme")
        .open(adding(page))
        .description(says)
        .on_close(move |_, cx| closed.update(cx, close))
        .content(move |_, _| {
            let (typed, entered) = (typed.clone(), entered.clone());
            Input::new("theme-url", "Git URL")
                .placeholder("https://github.com/org/omarchy-theme.git")
                .value(draft.clone())
                .invalid(!draft.is_empty() && !valid)
                .disabled(busy)
                .on_change(move |t, _, cx| {
                    typed.update(cx, |p, cx| {
                        p.drafts.insert(URL.into(), t.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |t, window, cx| {
                    entered.update(cx, |p, cx| {
                        p.drafts.insert(URL.into(), t.to_owned());
                        submit(p, window, cx)
                    })
                })
        })
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("theme-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, close))
        })
        .action(move |_, _| {
            let went = went.clone();
            Button::new("theme-install")
                .label("Install")
                .variant(ButtonVariant::Primary)
                .disabled(busy || !valid)
                .on_click(move |_, window, cx| went.update(cx, |p, cx| submit(p, window, cx)))
        })
}

/// The two times as typed, each the snapshot's until it is edited, and whether both parse.
fn times(page: &CatalogPage, cx: &App) -> (String, String, bool) {
    let draft = |key: &str, field| {
        page.drafts
            .get(key)
            .cloned()
            .unwrap_or_else(|| text(page, field, cx))
    };
    let (day, night) = (draft(DAY_DRAFT, DAY), draft(NIGHT_DRAFT, NIGHT));
    let valid =
        !appearance::parse_time(&day).is_empty() && !appearance::parse_time(&night).is_empty();
    (day, night, valid)
}

pub fn schedule_says(page: &CatalogPage, cx: &App) -> &'static str {
    appearance::schedule_says(times(page, cx).2)
}

pub fn use_schedule_says(page: &CatalogPage, cx: &App) -> &'static str {
    appearance::use_schedule_says(times(page, cx).2)
}

/// Writes each time that is not the one shown.
fn apply_times(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let (day, night, valid) = times(page, cx);
    if !valid {
        return;
    }
    let client = page.stars.client.clone();
    for (key, field, typed) in [
        ("nightlightDay", DAY, day),
        ("nightlightNight", NIGHT, night),
    ] {
        let next = appearance::parse_time(&typed);
        if next == text(page, field, cx) {
            continue;
        }
        let body = appearance::schedule_write(key, Value::from(next));
        page.stars.settings.update(cx, |settings, cx| {
            settings.send(
                &client,
                key,
                &body,
                "The schedule was not changed",
                window,
                cx,
            );
        });
    }
}

/// The day and night times, and Set.
pub fn schedule(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let (day, night, valid) = times(page, cx);
    let field =
        |key: &'static str, name: &'static str, typed: String, placeholder: &'static str| {
            let bad = !typed.is_empty() && appearance::parse_time(&typed).is_empty();
            let (changed, entered) = (this.clone(), this.clone());
            Input::new(SharedString::from(format!("{id}-{name}")), name)
                .placeholder(placeholder)
                .value(typed)
                .invalid(bad)
                .on_change(move |t, _, cx| {
                    changed.update(cx, |p, cx| {
                        p.drafts.insert(key.into(), t.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |t, window, cx| {
                    entered.update(cx, |p, cx| {
                        p.drafts.insert(key.into(), t.to_owned());
                        apply_times(p, window, cx)
                    })
                })
        };
    let set = this.clone();
    h_flex()
        .gap(Space::Sm)
        .child(field(DAY_DRAFT, "Day", day, "07:00"))
        .child(field(NIGHT_DRAFT, "Night", night, "20:00"))
        .child(
            Button::new(SharedString::from(format!("{id}-set")))
                .label("Set")
                .variant(ButtonVariant::Outline)
                .disabled(!valid)
                .on_click(move |_, window, cx| set.update(cx, |p, cx| apply_times(p, window, cx))),
        )
}

/// Use schedule.
pub fn use_schedule(
    id: String,
    label: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let Some(on) = read(page, NIGHT_ON, cx) else {
        return row::reading(label).into_any_element();
    };
    let on = on == Value::Bool(true);
    let valid = times(page, cx).2;
    let this = this.clone();
    Switch::new(SharedString::from(id), label)
        .checked(on)
        .disabled(!valid)
        .on_change(move |next, window, cx| {
            this.update(cx, |p, cx| {
                let client = p.stars.client.clone();
                let body = appearance::schedule_write("nightlightNightOn", Value::Bool(next));
                p.stars.settings.update(cx, |settings, cx| {
                    let refused = "The schedule was not changed";
                    settings.send(&client, "nightlightNightOn", &body, refused, window, cx);
                });
            })
        })
        .into_any_element()
}

/// Background's current wallpaper: its preview (`pages/radios.rs`), its file, its path, and
/// Copy path.
pub fn wallpaper(page: &CatalogPage, cx: &App) -> impl IntoElement + use<> {
    let path = crate::catalog::Field::Key("background")
        .read(page.stars.settings.read(cx))
        .value()
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string);
    let (label, says) = crate::appearance::wallpaper_row(&path);
    let client = page.stars.client.clone();
    let copied = path.clone();
    let copy = Button::new("wallpaper-copy")
        .label("Copy path")
        .variant(ButtonVariant::Outline)
        .disabled(path.is_empty())
        .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx));
    let star = page.stars.star(page.page().id, &label, &says, cx);
    let controls = h_flex()
        .gap(Space::Sm)
        .children(crate::pages::radios::wallpaper(page))
        .child(copy);
    row::setting(label, says, "omarchy theme bg current", controls, star)
}
