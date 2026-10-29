//! The Hooks page's rows (`hooks.rs` has what they say and send): an event's group with Add…,
//! Run now and Open folder, its scripts with Enable, Disable, Edit and Remove…, and Add's dialog,
//! which copies a file from the file chooser or writes a one-line script.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::catalog::Field;
use crate::hooks::{self, Draft};
use crate::pages::row;

const KIND: &str = "hook-kind";
const MODE: &str = "hook-mode";
const FILE: &str = "hook-file";
const NAME: &str = "hook-name";
const COMMAND: &str = "hook-command";
const ERROR: &str = "hook-error";
const OPEN: &str = "hook-open";

fn hooks_list(page: &CatalogPage, cx: &App) -> Option<Value> {
    Field::Key("hooks")
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn text(page: &CatalogPage, name: &'static str, cx: &App) -> String {
    Field::Key(name)
        .read(page.stars.settings.read(cx))
        .value()
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

fn draft(page: &CatalogPage, k: &str) -> String {
    page.drafts.get(k).cloned().unwrap_or_default()
}

fn put(page: &mut CatalogPage, k: &str, v: impl Into<String>, cx: &mut Context<CatalogPage>) {
    page.drafts.insert(k.into(), v.into());
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
            "hooks",
            &body,
            "The hook was not changed",
            window,
            cx,
        )
    });
}

/// `openAdd(type)`.
fn open(page: &mut CatalogPage, kind: &str, cx: &mut Context<CatalogPage>) {
    let kind = if hooks::is_hook_id(kind) {
        kind.to_owned()
    } else {
        draft(page, KIND)
    };
    let kind = if kind.is_empty() {
        "theme-set".to_owned()
    } else {
        kind
    };
    for (k, v) in [
        (KIND, kind.as_str()),
        (MODE, "file"),
        (FILE, ""),
        (NAME, ""),
        (COMMAND, ""),
        (ERROR, ""),
    ] {
        page.drafts.insert(k.into(), v.into());
    }
    put(page, OPEN, "1", cx);
}

fn held(page: &CatalogPage) -> Draft {
    Draft {
        kind: draft(page, KIND),
        mode: draft(page, MODE),
        file: draft(page, FILE),
        name: draft(page, NAME),
        command: draft(page, COMMAND),
    }
}

/// `submitAdd`.
fn submit(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    match hooks::submit(&held(page)) {
        Err(why) => put(page, ERROR, why, cx),
        Ok(body) => {
            put(page, ERROR, "", cx);
            send(page, body, window, cx);
            put(page, OPEN, "", cx);
        }
    }
}

/// Add… of the Install group.
pub fn add_button(id: String, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label("Add…")
        .variant(ButtonVariant::Primary)
        .on_click(move |_, _, cx| this.update(cx, |p, cx| open(p, "", cx)))
}

/// Each event's group: This event, then its scripts.
pub fn groups(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = hooks_list(page, cx);
    let (theme, font) = (text(page, "theme", cx), text(page, "font", cx));
    hooks::display_types(list.as_ref())
        .into_iter()
        .map(|(kind, label)| {
            let mut rows = Vec::new();
            let blurb = hooks::blurb(&kind);
            let says = format!("{blurb} {}", hooks::run_hint(&kind, &theme, &font));
            let hint = format!("omarchy hook {kind}");
            let arg = hooks::run_arg(&kind, &theme, &font);
            let button = |what: &'static str, kind: String| {
                let this = this.clone();
                let arg = arg.clone();
                Button::new(SharedString::from(format!("hook-{kind}-{what}")))
                    .label(match what {
                        "add" => "Add…",
                        "run" => "Run now",
                        _ => "Open folder",
                    })
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, window, cx| {
                        let kind = kind.clone();
                        let arg = arg.clone();
                        this.update(cx, |p, cx| match what {
                            "add" => open(p, &kind, cx),
                            "run" => send(p, hooks::run_write(&kind, &arg), window, cx),
                            _ => send(p, hooks::folder_write(&kind), window, cx),
                        })
                    })
            };
            let control = h_flex()
                .gap(Space::Sm)
                .child(button("add", kind.clone()))
                .child(button("run", kind.clone()))
                .child(button("folder", kind.clone()));
            let star = page.stars.star(page.page().id, "This event", &says, cx);
            rows.push(row::stacked("This event", says, hint, control, star).into_any_element());
            let items = hooks::items_for(list.as_ref(), &kind);
            if items.is_empty() {
                let star = page
                    .stars
                    .star(page.page().id, "Scripts", "No scripts.", cx);
                rows.push(
                    row::setting("Scripts", "No scripts.", "", v_flex(), star).into_any_element(),
                );
            }
            for (i, item) in items.into_iter().enumerate() {
                rows.push(script(&kind, i, &item, page, this, cx).into_any_element());
            }
            row::group(label, blurb, rows).into_any_element()
        })
        .collect()
}

/// A script, with its main button, Edit, and Remove….
fn script(
    kind: &str,
    i: usize,
    item: &Value,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let s = |k: &str| {
        item.get(k)
            .filter(|v| crate::js::truthy(Some(v)))
            .map_or_else(String::new, crate::js::string)
    };
    let (name, path) = (s("name"), s("path"));
    let label = if name.is_empty() {
        "hook".to_owned()
    } else {
        name.clone()
    };
    let (sample, flat) = (
        crate::js::truthy(item.get("sample")),
        crate::js::truthy(item.get("flat")),
    );
    let id = format!("hook-{kind}-{i}");
    let press = |what: &'static str, label: &'static str, variant: ButtonVariant| {
        let (this, path) = (this.clone(), path.clone());
        Button::new(SharedString::from(format!("{id}-{what}")))
            .label(label)
            .variant(variant)
            .disabled(path.is_empty())
            .on_click(move |_, window, cx| {
                let path = path.clone();
                this.update(cx, |p, cx| {
                    let body = match what {
                        "enable" => hooks::sample_write(&path, true),
                        "disable" => hooks::sample_write(&path, false),
                        _ => hooks::edit_write(&path),
                    };
                    send(p, body, window, cx)
                })
            })
    };
    let main = match hooks::action(item) {
        "Enable" => press("enable", "Enable", ButtonVariant::Primary),
        "Edit" => press("edit", "Edit", ButtonVariant::Outline),
        _ => press("disable", "Disable", ButtonVariant::Outline),
    };
    let edit = (!sample && !flat).then(|| press("edit-extra", "Edit", ButtonVariant::Outline));
    let remove = (!sample).then(|| {
        let (this, path, name, kind) = (this.clone(), path.clone(), name.clone(), kind.to_owned());
        Button::new(SharedString::from(format!("{id}-remove")))
            .label("Remove…")
            .variant(ButtonVariant::Destructive)
            .disabled(path.is_empty())
            .on_click(move |_, _, cx| {
                let (path, message) = (path.clone(), hooks::remove_message(&name, &kind));
                this.update(cx, |p, cx| {
                    p.ask(
                        Asked {
                            title: "Remove this hook".into(),
                            message,
                            action: "Remove",
                            go: Rc::new(move |p, window, cx| {
                                send(p, hooks::remove_write(&path), window, cx)
                            }),
                        },
                        cx,
                    )
                })
            })
    });
    let says = hooks::describe(item);
    let star = page.stars.star(page.page().id, &label, &says, cx);
    row::setting(
        label,
        says,
        path,
        h_flex()
            .gap(Space::Sm)
            .child(main)
            .children(edit)
            .children(remove),
        star,
    )
}

/// Add's dialog.
pub fn dialog(page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let d = held(page);
    let ready = (d.mode == "file" && !d.file.is_empty())
        || (d.mode == "command" && !d.name.is_empty() && !d.command.is_empty());
    let (closed, cancelled, added, built) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("hook-add", "Add a hook")
        .open(draft(page, OPEN) == "1")
        .description(hooks::blurb(&d.kind))
        .on_close(move |_, cx| closed.update(cx, |p, cx| put(p, OPEN, "", cx)))
        .content(move |_, cx| form(built.read(cx), &built))
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("hook-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, |p, cx| put(p, OPEN, "", cx)))
        })
        .action(move |_, _| {
            let added = added.clone();
            Button::new("hook-install")
                .label("Install")
                .variant(ButtonVariant::Primary)
                .disabled(!ready)
                .on_click(move |_, window, cx| added.update(cx, |p, cx| submit(p, window, cx)))
        })
}

fn form(page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let d = held(page);
    let error = draft(page, ERROR);
    let options: Vec<(String, String)> = hooks::TYPES
        .iter()
        .map(|t| (t.id.to_owned(), t.label.to_owned()))
        .collect();
    let (typed, moded, chosen) = (this.clone(), this.clone(), this.clone());
    let field = |k: &'static str, name: &'static str, placeholder: &'static str| {
        let (typed, entered) = (this.clone(), this.clone());
        Input::new(SharedString::from(k), name)
            .placeholder(placeholder)
            .value(draft(page, k))
            .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, k, t, cx)))
            .on_submit(move |t, window, cx| {
                entered.update(cx, |p, cx| {
                    p.drafts.insert(k.into(), t.to_owned());
                    submit(p, window, cx)
                })
            })
    };
    let file_label = if d.file.is_empty() {
        "Choose…".to_owned()
    } else {
        format!("File: {}", d.file.rsplit('/').next().unwrap_or_default())
    };
    let command = d.mode == "command";
    v_flex()
        .gap(Space::Sm)
        .child(crate::pages::controls::select(
            "hook-type".into(),
            "Event",
            &options,
            &Value::from(d.kind.clone()),
            false,
            move |v, _, cx| {
                let v = v.to_owned();
                typed.update(cx, |p, cx| put(p, KIND, v, cx))
            },
        ))
        .child(
            RadioGroup::new("hook-mode", "How")
                .horizontal(true)
                .item("file", "Existing file")
                .item("command", "One-line command")
                .value(d.mode.clone())
                .on_change(move |v, _, cx| moded.update(cx, |p, cx| put(p, MODE, v, cx))),
        )
        .children((!command).then(|| {
            Button::new("hook-choose")
                .label(file_label)
                .variant(ButtonVariant::Outline)
                .on_click(move |_, window, cx| {
                    let chooser = FileChooser::open("Install a hook").start_in(StartFolder::Home);
                    crate::pages::files::choose(
                        chooser,
                        &chosen,
                        window,
                        cx,
                        Box::new(|p, path, _, cx| put(p, FILE, path, cx)),
                    );
                })
        }))
        .children(command.then(|| field(NAME, "Script name", "notify.sh")))
        .children(command.then(|| {
            field(
                COMMAND,
                "Command",
                "omarchy-notification-send -u low \"Theme\" \"$1\"",
            )
        }))
        .child(Text::new(hooks::dest_text(&d)).muted())
        .children(
            (!error.is_empty()).then(|| Alert::new("hook-error", error).variant(Tone::Danger)),
        )
}
