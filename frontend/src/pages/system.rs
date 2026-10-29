//! The System page's rows Atmos draws itself (`system.rs` has what they say and send): the
//! hostname, the Omarchy and Atmos versions, channels and updates, Hyprland integration,
//! printers, the About logo, and the weather location.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::catalog::Field;
use crate::jobs::Job;
use crate::pages::controls::select;
use crate::pages::row;
use crate::settings::Known;
use crate::system::{self, Branding};

fn key(page: &CatalogPage, name: &'static str, cx: &App) -> Option<Value> {
    Field::Key(name)
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn text(page: &CatalogPage, name: &'static str, cx: &App) -> String {
    system::text(key(page, name, cx).as_ref())
}

fn on(page: &CatalogPage, name: &'static str, cx: &App) -> bool {
    key(page, name, cx) == Some(Value::Bool(true))
}

fn busy(page: &CatalogPage, cx: &App) -> bool {
    page.jobs.read(cx).job_busy()
}

fn running(page: &CatalogPage, cx: &App) -> Option<&'static str> {
    page.jobs.read(cx).running().filter(|_| busy(page, cx))
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
            "The setting was not changed",
            window,
            cx,
        )
    });
}

fn run(page: &mut CatalogPage, job: Job, window: &mut Window, cx: &mut Context<CatalogPage>) {
    page.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
}

fn confirm(
    page: &mut CatalogPage,
    title: &str,
    message: String,
    action: &'static str,
    go: impl Fn(&mut CatalogPage, &mut Window, &mut Context<CatalogPage>) + 'static,
    cx: &mut Context<CatalogPage>,
) {
    page.ask(
        Asked {
            title: title.into(),
            message,
            action,
            go: Rc::new(go),
        },
        cx,
    );
}

/// The draft of a field, the snapshot's value until it is edited.
fn draft(page: &CatalogPage, key: &str, or: String) -> String {
    page.drafts.get(key).cloned().unwrap_or(or)
}

/// The Atmos update check's summary, and whether a newer Atmos waits (from the last check).
fn atmos_check(page: &CatalogPage, cx: &App) -> (String, bool) {
    let jobs = page.jobs.read(cx);
    match jobs.last {
        Some(("atmos-update-check" | "atmos-update", _)) => {
            let (status, summary) = system::parse_check(&jobs.last_out);
            (summary, status == "behind")
        }
        _ => (String::new(), false),
    }
}

fn integrated(page: &CatalogPage, cx: &App) -> bool {
    match page.inventories.read(cx).known("diagnostics") {
        Known::Value(doc) => {
            crate::diagnostics::normalize(Some(doc))["atmos"]["hyprAtmos"] == Value::Bool(true)
        }
        _ => false,
    }
}

/// The version rows, by part (`omarchy`, `atmos`), whose label is the version.
pub fn rows(which: &str, page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let client = page.stars.client.clone();
    let (label, says, hint, copied) = if which == "omarchy" {
        let (l, s, c) = system::omarchy_row(
            &text(page, "omarchyVersion", cx),
            &text(page, "omarchyChannel", cx),
        );
        (l, s, "omarchy version", c)
    } else {
        let (l, s, h, c) = system::atmos_row(
            on(page, "atmosPackaged", cx),
            &text(page, "atmosVersion", cx),
            &text(page, "atmosRevision", cx),
            on(page, "atmosInstalled", cx),
        );
        (l, s.to_owned(), h, c)
    };
    let copy = Button::new(SharedString::from(format!("system-{which}-copy")))
        .label("Copy")
        .variant(ButtonVariant::Outline)
        .disabled(copied.is_empty())
        .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx));
    let star = page.stars.star(page.page().id, &label, &says, cx);
    vec![row::setting(label, says, hint, copy, star).into_any_element()]
}

/// What a row of the page's own says, by part.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    Some(match which {
        "hostname" => {
            let d = draft(page, "sys-hostname", text(page, "hostname", cx));
            system::hostname_says(d.is_empty() || !system::parse_hostname(&d).is_empty()).into()
        }
        "oupdates" => system::updates_says(
            running(page, cx) == Some("omarchy-update"),
            on(page, "updateAvailable", cx),
            &text(page, "updateSummary", cx),
        ),
        "aupdates" => {
            let (summary, available) = atmos_check(page, cx);
            system::atmos_updates_says(running(page, cx), &summary, available)
        }
        "integration" => system::integration(integrated(page, cx)).0.into(),
        "printers" => system::printers_says(on(page, "cupsActive", cx)).into(),
        "about" => system::about_says(on(page, "aboutBranded", cx)).into(),
        "location" => {
            let auto = on(page, "weatherAuto", cx);
            let d = draft(
                page,
                "sys-location",
                if auto {
                    String::new()
                } else {
                    text(page, "weatherLocation", cx)
                },
            );
            system::location_says(d.is_empty() || !system::parse_location(&d).is_empty(), auto)
                .into()
        }
        "coords" => {
            let d = draft(page, "sys-coords", text(page, "weatherCoords", cx));
            system::coords_says(d.is_empty() || !system::parse_coords(&d).is_empty()).into()
        }
        _ => return None,
    })
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    match which {
        "ochannel" | "aupdates" => !on(page, "atmosPackaged", cx),
        "integration" => on(page, "atmosPackaged", cx),
        "coords" => !on(page, "weatherAuto", cx) && !text(page, "weatherLocation", cx).is_empty(),
        _ => true,
    }
}

/// Whether the row's control takes the row's width.
pub fn stacked(which: &str) -> bool {
    matches!(which, "hostname" | "location" | "coords")
}

/// What Set does with the text.
type Set = Rc<dyn Fn(&mut CatalogPage, &str, &mut Window, &mut Context<CatalogPage>)>;

/// A line of text, Set, and the row's other button.
#[allow(clippy::too_many_arguments)]
fn line(
    id: String,
    label: &'static str,
    (draft_key, placeholder): (&'static str, &'static str),
    shown: String,
    ok: fn(&str) -> bool,
    extra: Option<Button>,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    set: Set,
) -> impl IntoElement + use<> {
    let typed_now = draft(page, draft_key, shown);
    let valid = ok(&typed_now);
    let (typed, entered, pressed) = (this.clone(), this.clone(), this.clone());
    let (s1, s2) = (set.clone(), set);
    let d = typed_now.clone();
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-field")), label)
                .placeholder(placeholder)
                .value(typed_now.clone())
                .invalid(!typed_now.is_empty() && !valid)
                .on_change(move |t, _, cx| {
                    typed.update(cx, |p, cx| {
                        p.drafts.insert(draft_key.into(), t.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |t, window, cx| {
                    let t = t.to_owned();
                    entered.update(cx, |p, cx| {
                        p.drafts.insert(draft_key.into(), t.clone());
                        if ok(&t) {
                            s1(p, &t, window, cx)
                        }
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-set")))
                .label("Set")
                .variant(ButtonVariant::Primary)
                .disabled(!valid)
                .on_click(move |_, window, cx| {
                    let d = d.clone();
                    pressed.update(cx, |p, cx| s2(p, &d, window, cx))
                }),
        )
        .children(extra)
}

fn button(
    id: String,
    label: &'static str,
    variant: ButtonVariant,
    disabled: bool,
    this: &Entity<CatalogPage>,
    go: impl Fn(&mut CatalogPage, &mut Window, &mut Context<CatalogPage>) + 'static,
) -> Button {
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label(label)
        .variant(variant)
        .disabled(disabled)
        .on_click(move |_, window, cx| this.update(cx, |p, cx| go(p, window, cx)))
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
    let busy = busy(page, cx);
    match which {
        "hostname" => {
            let current = text(page, "hostname", cx);
            let set: Set = Rc::new(move |p, t, window, cx| {
                let body = system::hostname_write(t, &current);
                send(p, "hostname", body, window, cx)
            });
            let ok = |t: &str| !system::parse_hostname(t).is_empty();
            line(
                id,
                label,
                ("sys-hostname", "hostname"),
                text(page, "hostname", cx),
                ok,
                None,
                page,
                this,
                set,
            )
            .into_any_element()
        }
        "ochannel" | "achannel" => {
            let omarchy = which == "ochannel";
            let current = text(
                page,
                if omarchy {
                    "omarchyChannel"
                } else {
                    "atmosChannel"
                },
                cx,
            );
            let options: Vec<(String, String)> = if omarchy {
                system::CHANNELS
                    .iter()
                    .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
                    .collect()
            } else {
                vec![("stable".into(), "Stable".into())]
            };
            let this = this.clone();
            let now = Value::from(current.clone());
            let disabled = busy || (omarchy && current.is_empty());
            select(id, label, &options, &now, disabled, move |v, window, cx| {
                if v == current {
                    return;
                }
                let (v, current) = (v.to_owned(), current.clone());
                this.update(cx, |p, cx| {
                    if !omarchy {
                        let body = system::atmos_channel_write(&v, &current);
                        return send(p, "atmosChannel", body, window, cx);
                    }
                    let message = system::channel_message(&v);
                    confirm(
                        p,
                        "Switch channel",
                        message,
                        "Switch",
                        move |p, window, cx| {
                            if let Some(job) = system::channel_job(&v, &current) {
                                run(p, job, window, cx)
                            }
                        },
                        cx,
                    )
                })
            })
            .into_any_element()
        }
        "oupdates" | "aupdates" => {
            let omarchy = which == "oupdates";
            let (_, available) = atmos_check(page, cx);
            let check = button(
                format!("{id}-check"),
                "Check",
                ButtonVariant::Outline,
                busy,
                this,
                move |p, window, cx| {
                    run(
                        p,
                        if omarchy {
                            system::omarchy_check()
                        } else {
                            system::atmos_check()
                        },
                        window,
                        cx,
                    )
                },
            );
            let update = button(
                format!("{id}-update"),
                "Update…",
                ButtonVariant::Primary,
                busy || (!omarchy && !available),
                this,
                move |p, _, cx| {
                    let (title, message) = if omarchy {
                        (
                            "Update Omarchy",
                            "Download and install Omarchy and system package updates. This can take a while and may ask for a password.",
                        )
                    } else {
                        (
                            "Update Atmos",
                            "Fetch the stable branch and replace the installed Atmos files under ~/.local/share/atmos. If the window does not reload, quit Atmos and open it again.",
                        )
                    };
                    confirm(
                        p,
                        title,
                        message.into(),
                        "Update",
                        move |p, window, cx| {
                            run(
                                p,
                                if omarchy {
                                    system::omarchy_update()
                                } else {
                                    system::atmos_update()
                                },
                                window,
                                cx,
                            )
                        },
                        cx,
                    )
                },
            );
            h_flex()
                .gap(Space::Sm)
                .child(check)
                .child(update)
                .into_any_element()
        }
        "integration" => Text::new(system::integration(integrated(page, cx)).1).into_any_element(),
        "layout" => {
            let settings = page.stars.settings.read(cx);
            let options =
                crate::catalog::control::Options::Listed(Field::Key("keyboardLayouts"), &[])
                    .list(settings);
            let current = text(page, "keyboardLayout", cx);
            let now = Value::from(current.clone());
            let this = this.clone();
            select(
                id,
                label,
                &options,
                &now,
                options.is_empty(),
                move |v, window, cx| {
                    if v == current {
                        return;
                    }
                    let Some((body, revert)) = system::layout_write(v, &current) else {
                        return;
                    };
                    this.update(cx, |p, cx| {
                        let client = p.stars.client.clone();
                        p.stars.settings.update(cx, |s, cx| {
                            let guard = ("keyboardLayout", revert);
                            s.send_guarded(
                                &client,
                                "keyboardLayout",
                                &body,
                                "The layout was not changed",
                                guard,
                                window,
                                cx,
                            )
                        })
                    })
                },
            )
            .into_any_element()
        }
        "printers" => {
            let setup = on(page, "printerSetup", cx);
            let open = button(
                format!("{id}-setup"),
                "Set up…",
                ButtonVariant::Primary,
                false,
                this,
                move |p, window, cx| {
                    send(
                        p,
                        "printers",
                        Some(system::printers_write(setup)),
                        window,
                        cx,
                    )
                },
            );
            let cups = button(
                format!("{id}-cups"),
                "Open CUPS",
                ButtonVariant::Outline,
                false,
                this,
                |p, window, cx| send(p, "printers", Some(system::cups_write()), window, cx),
            );
            h_flex()
                .gap(Space::Sm)
                .child(open)
                .child(cups)
                .into_any_element()
        }
        "about" => logo("about", id, page, this, cx).into_any_element(),
        "location" => {
            let auto = on(page, "weatherAuto", cx);
            let current = text(page, "weatherLocation", cx);
            let shown = if auto { String::new() } else { current.clone() };
            let set: Set = Rc::new(move |p, t, window, cx| {
                let body = system::location_write(t, &current, auto);
                send(p, "weatherLocation", body, window, cx)
            });
            let auto_button = button(
                format!("{id}-auto"),
                "Auto",
                ButtonVariant::Outline,
                auto,
                this,
                move |p, window, cx| {
                    send(p, "weatherLocation", system::auto_write(auto), window, cx)
                },
            );
            let ok = |t: &str| !system::parse_location(t).is_empty();
            line(
                id,
                label,
                ("sys-location", "City name"),
                shown,
                ok,
                Some(auto_button),
                page,
                this,
                set,
            )
            .into_any_element()
        }
        "coords" => {
            let (auto, location, current) = (
                on(page, "weatherAuto", cx),
                text(page, "weatherLocation", cx),
                text(page, "weatherCoords", cx),
            );
            let shown = current.clone();
            let set: Set = Rc::new(move |p, t, window, cx| {
                let body = system::coords_write(t, &location, auto, &current);
                send(p, "weatherCoords", body, window, cx)
            });
            let ok = |t: &str| !system::parse_coords(t).is_empty();
            line(
                id,
                label,
                ("sys-coords", "lat,lon"),
                shown,
                ok,
                None,
                page,
                this,
                set,
            )
            .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}

/// A logo's Choose…, Edit, and Reset while it is custom (`about`, or Idle's `screensaver`).
pub fn logo(
    which: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let branded = on(
        page,
        if which == "about" {
            "aboutBranded"
        } else {
            "screensaverBranded"
        },
        cx,
    );
    let act = |action: &'static str| {
        move |p: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>| {
            match system::branding(which, action, branded) {
                Some(Branding::Job(job)) => run(p, job, window, cx),
                Some(Branding::Write(body)) => send(
                    p,
                    if which == "about" {
                        "aboutBranding"
                    } else {
                        "screensaverBranding"
                    },
                    Some(body),
                    window,
                    cx,
                ),
                None => {}
            }
        }
    };
    h_flex()
        .gap(Space::Sm)
        .child(button(
            format!("{id}-image"),
            "Choose…",
            ButtonVariant::Outline,
            false,
            this,
            act("image"),
        ))
        .child(button(
            format!("{id}-text"),
            "Edit",
            ButtonVariant::Outline,
            false,
            this,
            act("text"),
        ))
        .children(branded.then(|| {
            button(
                format!("{id}-reset"),
                "Reset",
                ButtonVariant::Destructive,
                false,
                this,
                act("reset"),
            )
        }))
}
