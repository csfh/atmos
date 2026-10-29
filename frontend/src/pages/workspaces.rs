//! The Workspaces page's controls and lists (`workspaces.rs` has what they say and send): the
//! bar count, the two switches, the login workspace, each numbered workspace's name and monitor,
//! and the scratch workspaces with Add and Remove.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::catalog::Field;
use crate::pages::controls::select;
use crate::pages::row;
use crate::workspaces::{self, COUNT, ITEMS, MONITORS};

const HINT: &str = "~/.config/hypr/atmos.lua";
const SCRATCH_DRAFT: &str = "ws-scratch";
const SCRATCH_ERROR: &str = "ws-scratch-error";

fn read(page: &CatalogPage, field: Field, cx: &App) -> Option<Value> {
    field.read(page.stars.settings.read(cx)).value().cloned()
}

/// Sends a write to `key`, when there is one.
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
        settings.send(
            &client,
            key,
            &body,
            "The workspaces were not changed",
            window,
            cx,
        );
    });
}

/// Writes the workspaces `change` makes of what the snapshot holds.
fn change(
    page: &mut CatalogPage,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
    change: impl FnOnce(Option<&Value>) -> Vec<Value>,
) {
    let list = read(page, ITEMS, cx);
    let body = workspaces::write(list.as_ref(), change(list.as_ref()));
    send(page, "workspaces", Some(body), window, cx);
}

/// Number of workspaces.
pub fn count(
    id: String,
    label: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let Some(shown) = read(page, COUNT, cx) else {
        return row::reading(label).into_any_element();
    };
    let (lo, hi, step) = workspaces::SHOWN;
    let this = this.clone();
    NumberInput::new(SharedString::from(id), label)
        .value(crate::js::number(Some(&shown)))
        .min(lo)
        .max(hi)
        .step(step)
        .on_commit(move |n, window, cx| {
            let shown = shown.clone();
            this.update(cx, |p, cx| {
                let body = workspaces::count_write(Some(&shown), n);
                send(p, "workspaceBarCount", body, window, cx)
            })
        })
        .into_any_element()
}

/// Wrap switching, or Super + mouse wheel.
pub fn switch(
    id: String,
    label: &'static str,
    key: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let Some(on) = read(page, Field::Key(key), cx) else {
        return row::reading(label).into_any_element();
    };
    let on = crate::js::truthy(Some(&on));
    let this = this.clone();
    Switch::new(SharedString::from(id), label)
        .checked(on)
        .on_change(move |next, window, cx| {
            this.update(cx, |p, cx| {
                let body = workspaces::switch_write(key, next);
                send(p, key, Some(body), window, cx)
            })
        })
        .into_any_element()
}

/// Open on login.
pub fn login(
    id: String,
    label: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let Some(list) = read(page, ITEMS, cx) else {
        return row::reading(label).into_any_element();
    };
    let numbered = workspaces::numbered(Some(&list));
    let now = workspaces::default_id(&numbered);
    let this = this.clone();
    let options = workspaces::default_options(&numbered);
    select(
        id,
        label,
        &options,
        &Value::from(now.clone()),
        false,
        move |v, window, cx| {
            if v == now {
                return;
            }
            let want = v.to_owned();
            this.update(cx, |p, cx| {
                change(p, window, cx, |list| {
                    workspaces::with_default(&workspaces::state(list), &want)
                })
            })
        },
    )
    .into_any_element()
}

/// The scratch workspace's description: what is wrong with the name, or what to name it.
pub fn scratch_says(page: &CatalogPage) -> String {
    match page.drafts.get(SCRATCH_ERROR) {
        Some(why) if !why.is_empty() => why.clone(),
        _ => workspaces::SCRATCH_SAYS.to_owned(),
    }
}

/// `addSpecial`.
fn add(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let draft = page.drafts.get(SCRATCH_DRAFT).cloned().unwrap_or_default();
    let list = read(page, ITEMS, cx);
    match workspaces::add_special(list.as_ref(), &draft) {
        Err(why) => {
            page.drafts.insert(SCRATCH_ERROR.into(), why.into());
            cx.notify();
        }
        Ok(items) => {
            page.drafts.remove(SCRATCH_ERROR);
            page.drafts.remove(SCRATCH_DRAFT);
            let body = workspaces::write(list.as_ref(), items);
            send(page, "workspaces", Some(body), window, cx);
        }
    }
}

/// Add a scratch workspace: its name, and Add.
pub fn add_scratch(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let draft = page.drafts.get(SCRATCH_DRAFT).cloned().unwrap_or_default();
    let (typed, submitted, added) = (this.clone(), this.clone(), this.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(
                SharedString::from(format!("{id}-name")),
                "Scratch workspace",
            )
            .placeholder("notes")
            .value(draft)
            .on_change(move |text, _, cx| {
                typed.update(cx, |p, cx| {
                    p.drafts.insert(SCRATCH_DRAFT.into(), text.to_owned());
                    cx.notify();
                })
            })
            .on_submit(move |text, window, cx| {
                submitted.update(cx, |p, cx| {
                    p.drafts.insert(SCRATCH_DRAFT.into(), text.to_owned());
                    add(p, window, cx);
                })
            }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-add")))
                .label("Add")
                .variant(ButtonVariant::Primary)
                .on_click(move |_, window, cx| added.update(cx, |p, cx| add(p, window, cx))),
        )
}

/// Each numbered workspace, with its bar name and Set, and its monitor.
pub fn names(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = read(page, ITEMS, cx);
    let monitors = read(page, MONITORS, cx);
    workspaces::numbered(list.as_ref())
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let label = workspaces::label(&r, "Workspace");
            let id = r.get("id").cloned().unwrap_or(Value::Null);
            let key = format!("ws-name-{}", crate::js::string(&id));
            let named = r
                .get("name")
                .filter(|n| crate::js::truthy(Some(n)))
                .map_or_else(String::new, crate::js::string);
            let draft = page.drafts.get(&key).cloned().unwrap_or(named);
            let monitor = r.get("monitor").filter(|m| crate::js::truthy(Some(m)));
            let pinned = monitor.cloned().unwrap_or(Value::from(""));
            let options = workspaces::monitor_options(monitors.as_ref(), monitor);
            let was = pinned.clone();
            let (typed, entered, set, picked) =
                (this.clone(), this.clone(), this.clone(), this.clone());
            let (k1, k2) = (key.clone(), key.clone());
            let (id1, id2, id3) = (id.clone(), id.clone(), id);
            let control = h_flex()
                .gap(Space::Sm)
                .child(
                    Input::new(SharedString::from(format!("ws-{i}-name")), "Bar name")
                        .placeholder("Bar name")
                        .value(draft.clone())
                        .on_change(move |text, _, cx| {
                            typed.update(cx, |p, cx| {
                                p.drafts.insert(k1.clone(), text.to_owned());
                                cx.notify();
                            })
                        })
                        .on_submit(move |text, window, cx| {
                            let (id, name) = (id1.clone(), Value::from(text));
                            entered.update(cx, |p, cx| {
                                change(p, window, cx, |l| workspaces::patch(l, &id, "name", name))
                            })
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("ws-{i}-set")))
                        .label("Set")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, window, cx| {
                            let id = id2.clone();
                            set.update(cx, |p, cx| {
                                let name = p.drafts.get(&k2).cloned().unwrap_or(draft.clone());
                                let name = Value::from(name);
                                change(p, window, cx, |l| workspaces::patch(l, &id, "name", name))
                            })
                        }),
                )
                .child(select(
                    format!("ws-{i}-monitor"),
                    "Monitor",
                    &options,
                    &pinned,
                    false,
                    move |v, window, cx| {
                        if was == *v {
                            return;
                        }
                        let (id, monitor) = (id3.clone(), Value::from(v));
                        picked.update(cx, |p, cx| {
                            change(p, window, cx, |l| {
                                workspaces::patch(l, &id, "monitor", monitor)
                            })
                        })
                    },
                ));
            let description = workspaces::description(&r);
            let star = page.stars.star(page.page().id, &label, &description, cx);
            row::setting(label, description, HINT, control, star).into_any_element()
        })
        .collect()
}

/// Each scratch workspace, with Remove.
pub fn scratch(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    const SAYS: &str = "A scratch workspace. Remove drops the Atmos rule.";
    let list = read(page, ITEMS, cx);
    workspaces::specials(list.as_ref())
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let label = workspaces::label(&r, "special");
            let id = r.get("id").cloned().unwrap_or(Value::Null);
            let this = this.clone();
            let remove = Button::new(SharedString::from(format!("scratch-{i}-remove")))
                .label("Remove")
                .variant(ButtonVariant::Destructive)
                .on_click(move |_, window, cx| {
                    let id = id.clone();
                    this.update(cx, |p, cx| {
                        change(p, window, cx, |l| workspaces::remove(l, &id))
                    })
                });
            let star = page.stars.star(page.page().id, &label, SAYS, cx);
            row::setting(label, SAYS, "", remove, star).into_any_element()
        })
        .collect()
}
