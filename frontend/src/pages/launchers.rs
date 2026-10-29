//! The Applications page's rows (`launchers.rs` has what they say and send): the launch-on-start
//! commands with Add and Remove…, Add… for each kind of launcher in its dialog, and the launchers
//! you added, each with Remove…. Removing a desktop launcher runs as root, behind the sudo prompt.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::jobs::RootWrite;
use crate::launchers::{self, Draft};
use crate::pages::controls::select;
use crate::pages::row;
use crate::startup::{self, AUTOSTART};

const KIND: &str = "app-kind";
const NAME: &str = "app-name";
const COMMAND: &str = "app-command";
const URL: &str = "app-url";
const STYLE: &str = "app-style";
const ICON: &str = "app-icon";
const ERROR: &str = "app-error";
const COMMAND_DRAFT: &str = "app-autostart";

fn draft(page: &CatalogPage, key: &str) -> String {
    page.drafts.get(key).cloned().unwrap_or_default()
}

fn put(page: &mut CatalogPage, key: &str, value: impl Into<String>, cx: &mut Context<CatalogPage>) {
    page.drafts.insert(key.into(), value.into());
    cx.notify();
}

fn busy(page: &CatalogPage, cx: &App) -> bool {
    page.jobs.read(cx).job_busy()
}

fn read(page: &CatalogPage, kind: &str, cx: &App) -> Option<Value> {
    launchers::list(kind)
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

/// Add…, which opens the dialog for `kind` (`openAdd`).
pub fn add_button(
    kind: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label("Add…")
        .variant(ButtonVariant::Primary)
        .disabled(busy(page, cx))
        .on_click(move |_, _, cx| {
            this.update(cx, |p, cx| {
                for (key, value) in [
                    (NAME, ""),
                    (COMMAND, ""),
                    (URL, ""),
                    (ERROR, ""),
                    (STYLE, "tile"),
                ] {
                    p.drafts.insert(key.into(), value.into());
                }
                p.drafts
                    .insert(ICON.into(), launchers::icon_default(kind).into());
                put(p, KIND, kind, cx);
            })
        })
}

/// Whether the dialog is open, for what a test asks.
pub fn adding(page: &CatalogPage) -> Option<String> {
    Some(draft(page, KIND)).filter(|k| !k.is_empty())
}

fn close(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    put(page, KIND, "", cx);
}

/// `submitAdd`.
fn submit(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let d = Draft {
        kind: draft(page, KIND),
        name: draft(page, NAME),
        command: draft(page, COMMAND),
        url: draft(page, URL),
        style: draft(page, STYLE),
        icon: draft(page, ICON),
    };
    match launchers::submit(&d) {
        Err(why) => put(page, ERROR, why, cx),
        Ok(job) => {
            put(page, ERROR, "", cx);
            page.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
            close(page, cx);
        }
    }
}

/// Add's dialog: a name, the command or the site, the window style of a terminal app, an icon,
/// and what is wrong.
pub fn dialog(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let kind = draft(page, KIND);
    let busy = busy(page, cx);
    let running = page.jobs.read(cx).running();
    let making = busy && running.is_some_and(|k| launchers::MAKING.contains(&k));
    let (closed, cancelled, added, built) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("app-add", launchers::add_title(&kind))
        .open(!kind.is_empty())
        .description(launchers::add_blurb(&kind, making))
        .on_close(move |_, cx| closed.update(cx, close))
        .content(move |_, cx| form(built.read(cx), &built, busy))
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("app-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, close))
        })
        .action(move |_, _| {
            let added = added.clone();
            Button::new("app-go")
                .label("Add")
                .variant(ButtonVariant::Primary)
                .disabled(busy)
                .on_click(move |_, window, cx| added.update(cx, |p, cx| submit(p, window, cx)))
        })
}

fn field(
    key: &'static str,
    label: &'static str,
    placeholder: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    busy: bool,
) -> impl IntoElement + use<> {
    let (typed, entered) = (this.clone(), this.clone());
    Input::new(SharedString::from(key), label)
        .placeholder(placeholder)
        .value(draft(page, key))
        .disabled(busy)
        .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, key, t, cx)))
        .on_submit(move |t, window, cx| {
            entered.update(cx, |p, cx| {
                p.drafts.insert(key.into(), t.to_owned());
                submit(p, window, cx)
            })
        })
}

fn form(page: &CatalogPage, this: &Entity<CatalogPage>, busy: bool) -> impl IntoElement + use<> {
    let kind = draft(page, KIND);
    let error = draft(page, ERROR);
    let styled = this.clone();
    let style = select(
        "app-style".into(),
        "Window",
        &launchers::STYLES
            .iter()
            .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
            .collect::<Vec<_>>(),
        &Value::from(draft(page, STYLE)),
        busy,
        move |v, _, cx| {
            let v = v.to_owned();
            styled.update(cx, |p, cx| put(p, STYLE, v, cx))
        },
    )
    .into_any_element();
    let command = if kind == "tui" {
        "lazydocker"
    } else {
        "Command"
    };
    let icon = if kind == "web" {
        "Icon URL or name (optional)"
    } else {
        "Icon URL or name"
    };
    v_flex()
        .gap(Space::Sm)
        .child(field(NAME, "Name", "Name", page, this, busy))
        .children((kind != "web").then(|| field(COMMAND, "Command", command, page, this, busy)))
        .children(
            (kind == "web").then(|| field(URL, "Site", "https://example.com", page, this, busy)),
        )
        .children((kind == "tui").then_some(style))
        .child(field(ICON, "Icon", icon, page, this, busy))
        .children((!error.is_empty()).then(|| Alert::new("app-error", error).variant(Tone::Danger)))
}

/// Whether a kind's empty row shows: its list has been read and is empty.
pub fn none(kind: &str, page: &CatalogPage, cx: &App) -> bool {
    read(page, kind, cx).is_some_and(|v| v.as_array().is_none_or(Vec::is_empty))
}

/// The launchers of a kind, each with Remove….
pub fn rows(
    kind: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = read(page, kind, cx);
    list.as_ref()
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, r)| {
            let (label, says, hint, usable) = launchers::row(kind, r);
            let id = r
                .get("id")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(String::new, crate::js::string);
            let name = r
                .get("name")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(String::new, crate::js::string);
            let this = this.clone();
            let remove = Button::new(SharedString::from(format!("app-{kind}-{i}-remove")))
                .label("Remove…")
                .variant(ButtonVariant::Destructive)
                .disabled(!usable)
                .on_click(move |_, _, cx| {
                    let (title, message) = launchers::remove_confirm(kind, &name);
                    let (id, name) = (id.clone(), name.clone());
                    this.update(cx, |p, cx| {
                        p.ask(
                            Asked {
                                title,
                                message,
                                action: "Remove",
                                go: Rc::new(move |p, window, cx| {
                                    let Some((body, root)) = launchers::remove(kind, &id, &name)
                                    else {
                                        return;
                                    };
                                    let refused = format!("{name} was not removed");
                                    if root {
                                        let write = RootWrite {
                                            key: "launchers",
                                            body,
                                            refused,
                                        };
                                        p.jobs.update(cx, |jobs, cx| {
                                            jobs.write_as_root(write, window, cx)
                                        });
                                    } else {
                                        let client = p.stars.client.clone();
                                        p.stars.settings.update(cx, |s, cx| {
                                            s.send(&client, "launchers", &body, refused, window, cx)
                                        });
                                    }
                                }),
                            },
                            cx,
                        )
                    })
                });
            let star = page.stars.star(page.page().id, &label, &says, cx);
            row::setting(label, says, hint, remove, star).into_any_element()
        })
        .collect()
}

fn autostart(page: &CatalogPage, cx: &App) -> Option<Value> {
    AUTOSTART
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn send_autostart(
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
            "autostart",
            &body,
            "Startup was not changed",
            window,
            cx,
        )
    });
}

/// `Omarchy.addAutostart(command)`.
fn add_command(
    page: &mut CatalogPage,
    command: &str,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let list = autostart(page, cx);
    let body = startup::add(list.as_ref(), command, &Value::Null);
    send_autostart(page, body, window, cx);
}

/// Add a command: the command, and Add.
pub fn add_autostart(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let typed_now = draft(page, COMMAND_DRAFT);
    let (typed, entered, added) = (this.clone(), this.clone(), this.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-command")), "Command")
                .placeholder("hyprsunset")
                .value(typed_now.clone())
                .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, COMMAND_DRAFT, t, cx)))
                .on_submit(move |t, window, cx| {
                    entered.update(cx, |p, cx| {
                        p.drafts.insert(COMMAND_DRAFT.into(), t.to_owned());
                        add_command(p, t, window, cx)
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-add")))
                .label("Add")
                .variant(ButtonVariant::Primary)
                .disabled(typed_now.is_empty())
                .on_click(move |_, window, cx| {
                    added.update(cx, |p, cx| {
                        let command = draft(p, COMMAND_DRAFT);
                        add_command(p, &command, window, cx)
                    })
                }),
        )
}

/// Whether "No launch-on-start commands." shows.
pub fn no_autostart(page: &CatalogPage, cx: &App) -> bool {
    autostart(page, cx).is_some_and(|v| v.as_array().is_none_or(Vec::is_empty))
}

/// The launch-on-start commands, each with Remove… while Atmos wrote it.
pub fn autostart_rows(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = autostart(page, cx);
    list.as_ref()
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, r)| {
            let command = r
                .get("command")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(|| "command".to_owned(), crate::js::string);
            let managed = crate::js::truthy(r.get("managed"));
            let says = if managed {
                "Atmos wrote this line."
            } else {
                "This line is outside the Atmos block, so Remove stays off."
            };
            let this = this.clone();
            let gone = command.clone();
            let remove = managed.then(|| {
                Button::new(SharedString::from(format!("app-autostart-{i}-remove")))
                    .label("Remove…")
                    .variant(ButtonVariant::Destructive)
                    .on_click(move |_, _, cx| {
                        let gone = gone.clone();
                        this.update(cx, |p, cx| {
                            p.ask(
                                Asked {
                                    title: "Remove autostart".into(),
                                    message: format!("Stop launching {gone} at login?"),
                                    action: "Remove",
                                    go: Rc::new(move |p, window, cx| {
                                        let list = autostart(p, cx);
                                        let body = startup::remove(list.as_ref(), &gone);
                                        send_autostart(p, body, window, cx);
                                    }),
                                },
                                cx,
                            )
                        })
                    })
            });
            let star = page.stars.star(page.page().id, &command, says, cx);
            row::setting(
                command,
                says,
                "~/.config/hypr/autostart.lua",
                h_flex().children(remove),
                star,
            )
            .into_any_element()
        })
        .collect()
}
