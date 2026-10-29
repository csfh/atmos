//! The Accounts page's rows, lists and dialogs (`accounts.rs` has what they say and send): this
//! login's face, full name and password, each user with Password… and Remove…, each group with
//! Manage… and Remove…, and the dialogs that add a user, add a group, set a password and manage
//! a group's members. Every change runs as root, behind the sudo prompt. The face's preview is
//! the image `atmosd` reads (`pages/radios.rs`).

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use super::files::choose;
use crate::accounts::{self, AVATAR, CURRENT, FULL_NAME, GROUPS, NewUser, USERS};
use crate::jobs::{Job, RootWrite};
use crate::pages::row;

const DIALOG: &str = "acct-dialog";
const NAME: &str = "acct-name";
const FULL: &str = "acct-full";
const PASSWORD: &str = "acct-password";
const CONFIRM: &str = "acct-confirm";
const ADMIN: &str = "acct-admin";
const ERROR: &str = "acct-error";
const WHO: &str = "acct-who";
const FULL_DRAFT: &str = "acct-full-name";
const FULL_SEEN: &str = "acct-full-seen";

fn value(page: &CatalogPage, field: crate::catalog::Field, cx: &App) -> Option<Value> {
    field.read(page.stars.settings.read(cx)).value().cloned()
}

fn text(page: &CatalogPage, field: crate::catalog::Field, cx: &App) -> String {
    value(page, field, cx)
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, |v| crate::js::string(&v))
}

fn list(page: &CatalogPage, field: crate::catalog::Field, cx: &App) -> Vec<Value> {
    value(page, field, cx)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

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

fn run(
    page: &mut CatalogPage,
    job: Option<Job>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if let Some(job) = job {
        page.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
    }
}

fn as_root(
    page: &mut CatalogPage,
    write: Option<RootWrite>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if let Some(write) = write {
        page.jobs
            .update(cx, |jobs, cx| jobs.write_as_root(write, window, cx));
    }
}

/// Which dialog is open: `user`, `group`, `password`, or `manage`.
pub fn open_dialog(page: &CatalogPage) -> String {
    draft(page, DIALOG)
}

fn open(page: &mut CatalogPage, dialog: &str, who: &str, cx: &mut Context<CatalogPage>) {
    for key in [NAME, FULL, PASSWORD, CONFIRM, ADMIN, ERROR] {
        page.drafts.remove(key);
    }
    page.drafts.insert(WHO.into(), who.into());
    put(page, DIALOG, dialog, cx);
}

fn close(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    put(page, DIALOG, "", cx);
}

fn ask(
    page: &mut CatalogPage,
    title: &str,
    message: String,
    action: &'static str,
    job: Option<Job>,
    cx: &mut Context<CatalogPage>,
) {
    page.ask(
        Asked {
            title: title.into(),
            message,
            action,
            go: Rc::new(move |p, window, cx| run(p, job.clone(), window, cx)),
        },
        cx,
    );
}

fn button(id: String, label: &'static str, variant: ButtonVariant, disabled: bool) -> Button {
    Button::new(SharedString::from(id))
        .label(label)
        .variant(variant)
        .disabled(disabled)
}

/// This login's face: its file, Choose…, and Clear… while one is set.
pub fn face(page: &CatalogPage, this: &Entity<CatalogPage>, cx: &App) -> impl IntoElement + use<> {
    let (user, path) = (text(page, CURRENT, cx), text(page, AVATAR, cx));
    let label = if user.is_empty() {
        "Face".to_owned()
    } else {
        user.clone()
    };
    let says = accounts::face_says(&path);
    let (chosen, cleared) = (this.clone(), this.clone());
    let controls = h_flex()
        .gap(Space::Sm)
        .children(crate::pages::radios::face(page))
        .child(
            button(
                "acct-face".into(),
                "Choose…",
                ButtonVariant::Primary,
                busy(page, cx) || user.is_empty(),
            )
            .on_click(move |_, window, cx| {
                let chooser = FileChooser::open("Choose a face")
                    .filter("Images", ["*.png", "*.jpg", "*.jpeg"])
                    .start_in(StartFolder::Pictures);
                choose(
                    chooser,
                    &chosen,
                    window,
                    cx,
                    Box::new(|p, path, window, cx| {
                        let user = text(p, CURRENT, cx);
                        as_root(p, accounts::avatar_write(&path, &user), window, cx)
                    }),
                )
            }),
        )
        .children((!path.is_empty()).then(|| {
            button(
                "acct-face-clear".into(),
                "Clear…",
                ButtonVariant::Destructive,
                busy(page, cx),
            )
            .on_click(move |_, _, cx| {
                cleared.update(cx, |p, cx| {
                    p.ask(
                        Asked {
                            title: "Clear face".into(),
                            message: "Remove the face icon for this login.".into(),
                            action: "Clear",
                            go: Rc::new(|p, window, cx| {
                                let user = text(p, CURRENT, cx);
                                as_root(p, accounts::avatar_clear(&user), window, cx)
                            }),
                        },
                        cx,
                    )
                })
            })
        }));
    let star = page.stars.star(page.page().id, &label, &says, cx);
    row::setting(label, says, "~/.face.icon", controls, star)
}

/// The full name as typed, while the one set has not changed since.
fn full_draft(page: &CatalogPage, cx: &App) -> String {
    let now = value(page, FULL_NAME, cx).unwrap_or(Value::Null);
    match page.data.get(FULL_SEEN) {
        Some(seen) if *seen == now => draft(page, FULL_DRAFT),
        _ => text(page, FULL_NAME, cx),
    }
}

fn set_full_name(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let typed = full_draft(page, cx);
    if !accounts::is_full_name(&typed) {
        return;
    }
    let body = accounts::full_name_write(&typed, &text(page, FULL_NAME, cx));
    if let Some(body) = body {
        let client = page.stars.client.clone();
        page.stars.settings.update(cx, |s, cx| {
            s.send(
                &client,
                "fullName",
                &body,
                "The full name was not changed",
                window,
                cx,
            )
        });
    }
}

/// What a row of the page's own says.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    (which == "fullname").then(|| accounts::full_name_says(&full_draft(page, cx)).to_owned())
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    match which {
        "nologins" => value(page, USERS, cx).is_some() && list(page, USERS, cx).is_empty(),
        "nogroups" => value(page, GROUPS, cx).is_some() && list(page, GROUPS, cx).is_empty(),
        _ => true,
    }
}

/// The control of a row of the page's own.
pub fn control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let this = this.clone();
    let idle = !busy(page, cx);
    match which {
        "fullname" => {
            let typed = full_draft(page, cx);
            let parsed = typed.trim().to_owned();
            let valid = accounts::is_full_name(&typed);
            let can_set = valid && parsed != text(page, FULL_NAME, cx);
            let (edited, entered) = (this.clone(), this.clone());
            h_flex()
                .gap(Space::Sm)
                .child(
                    Input::new(SharedString::from(format!("{id}-field")), "Full name")
                        .placeholder("Your name")
                        .value(typed.clone())
                        .invalid(!typed.is_empty() && !valid)
                        .on_change(move |t, _, cx| {
                            let t = t.to_owned();
                            edited.update(cx, |p, cx| {
                                let seen = value(p, FULL_NAME, cx).unwrap_or(Value::Null);
                                p.data.insert(FULL_SEEN, seen);
                                put(p, FULL_DRAFT, t, cx)
                            })
                        })
                        .on_submit(move |t, window, cx| {
                            let t = t.to_owned();
                            entered.update(cx, |p, cx| {
                                let seen = value(p, FULL_NAME, cx).unwrap_or(Value::Null);
                                p.data.insert(FULL_SEEN, seen);
                                p.drafts.insert(FULL_DRAFT.into(), t);
                                set_full_name(p, window, cx)
                            })
                        }),
                )
                .child(
                    button(format!("{id}-set"), "Set", ButtonVariant::Primary, !can_set).on_click(
                        move |_, window, cx| this.update(cx, |p, cx| set_full_name(p, window, cx)),
                    ),
                )
                .into_any_element()
        }
        "password" => {
            let user = text(page, CURRENT, cx);
            button(
                id,
                "Change…",
                ButtonVariant::Outline,
                !idle || user.is_empty(),
            )
            .on_click(move |_, _, cx| {
                let user = user.clone();
                this.update(cx, |p, cx| open(p, "password", &user, cx))
            })
            .into_any_element()
        }
        "adduser" | "addgroup" => {
            let dialog = if which == "adduser" { "user" } else { "group" };
            button(id, "Add…", ButtonVariant::Primary, !idle)
                .on_click(move |_, _, cx| this.update(cx, |p, cx| open(p, dialog, "", cx)))
                .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}

/// Each user, with Password… and Remove…, which this session's login has not.
pub fn users(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let idle = !busy(page, cx);
    list(page, USERS, cx)
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let name = u
                .get("name")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(|| "user".to_owned(), crate::js::string);
            let named = crate::js::truthy(u.get("name"));
            let current = crate::js::truthy(u.get("current"));
            let (pw, gone, who, rm) = (this.clone(), this.clone(), name.clone(), name.clone());
            let controls = h_flex()
                .gap(Space::Sm)
                .child(
                    button(
                        format!("user-{i}-password"),
                        "Password…",
                        ButtonVariant::Outline,
                        !idle || !named,
                    )
                    .on_click(move |_, _, cx| pw.update(cx, |p, cx| open(p, "password", &who, cx))),
                )
                .children((!current).then(|| {
                    button(
                        format!("user-{i}-remove"),
                        "Remove…",
                        ButtonVariant::Destructive,
                        !idle || !named,
                    )
                    .on_click(move |_, _, cx| {
                        gone.update(cx, |p, cx| {
                            let me = text(p, CURRENT, cx);
                            let message = format!("Remove {rm} and their home directory.");
                            ask(
                                p,
                                "Remove user",
                                message,
                                "Remove",
                                accounts::remove_user(&rm, &me),
                                cx,
                            )
                        })
                    })
                }));
            let says = accounts::user_says(u);
            let hint = u
                .get("home")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(|| "/etc/passwd".to_owned(), crate::js::string);
            let star = page.stars.star(page.page().id, &name, &says, cx);
            row::setting(name, says, hint, controls, star).into_any_element()
        })
        .collect()
}

/// Each group, with Manage… and Remove…, which only a removable group has.
pub fn groups(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let idle = !busy(page, cx);
    list(page, GROUPS, cx)
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let name = g
                .get("name")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(|| "group".to_owned(), crate::js::string);
            let named = crate::js::truthy(g.get("name"));
            let removable = accounts::removable_gid(
                g.get("gid"),
                &crate::js::string(g.get("name").unwrap_or(&Value::Null)),
            );
            let (manage, gone, who, rm) = (this.clone(), this.clone(), name.clone(), name.clone());
            let controls = h_flex()
                .gap(Space::Sm)
                .child(
                    button(
                        format!("group-{i}-manage"),
                        "Manage…",
                        ButtonVariant::Outline,
                        !named,
                    )
                    .on_click(move |_, _, cx| {
                        manage.update(cx, |p, cx| open(p, "manage", &who, cx))
                    }),
                )
                .children(removable.then(|| {
                    button(
                        format!("group-{i}-remove"),
                        "Remove…",
                        ButtonVariant::Destructive,
                        !idle,
                    )
                    .on_click(move |_, _, cx| {
                        gone.update(cx, |p, cx| {
                            let message = format!("Remove the {rm} group. Logins stay.");
                            ask(
                                p,
                                "Remove group",
                                message,
                                "Remove",
                                accounts::remove_group(&rm),
                                cx,
                            )
                        })
                    })
                }));
            let says = accounts::group_says(g);
            let star = page.stars.star(page.page().id, &name, &says, cx);
            row::setting(name, says, "/etc/group", controls, star).into_any_element()
        })
        .collect()
}

fn submit(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let outcome = match open_dialog(page).as_str() {
        "user" => NewUser {
            name: draft(page, NAME),
            full: draft(page, FULL),
            password: draft(page, PASSWORD),
            confirm: draft(page, CONFIRM),
            admin: draft(page, ADMIN) == "1",
        }
        .submit(&list(page, USERS, cx)),
        "group" => accounts::submit_group(&draft(page, NAME), &list(page, GROUPS, cx)),
        "password" => accounts::submit_password(
            &draft(page, WHO),
            &draft(page, PASSWORD),
            &draft(page, CONFIRM),
        )
        .map_err(String::from),
        _ => return,
    };
    match outcome {
        Err(why) => put(page, ERROR, why, cx),
        Ok(job) => {
            close(page, cx);
            run(page, job, window, cx);
        }
    }
}

fn field(
    id: &'static str,
    placeholder: &'static str,
    key: &'static str,
    masked: bool,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let (typed, entered) = (this.clone(), this.clone());
    Input::new(id, placeholder)
        .placeholder(placeholder)
        .masked(masked)
        .value(draft(page, key))
        .disabled(busy(page, cx))
        .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, key, t, cx)))
        .on_submit(move |t, window, cx| {
            entered.update(cx, |p, cx| {
                p.drafts.insert(key.into(), t.to_owned());
                submit(p, window, cx)
            })
        })
}

fn members(page: &CatalogPage, this: &Entity<CatalogPage>, cx: &App) -> impl IntoElement + use<> {
    let group = draft(page, WHO);
    let (users, groups) = (list(page, USERS, cx), list(page, GROUPS, cx));
    let idle = !busy(page, cx);
    let rows: Vec<_> = users
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let name = u
                .get("name")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(String::new, crate::js::string);
            let shown = if name.is_empty() {
                "user".to_owned()
            } else {
                name.clone()
            };
            let locked = accounts::member_locked(&groups, &group, u);
            let inside = accounts::has_member(accounts::group(&groups, &group), &name);
            let (this, group) = (this.clone(), group.clone());
            let switch = Switch::new(SharedString::from(format!("member-{i}")), shown.clone())
                .checked(inside)
                .disabled(!idle || name.is_empty() || locked)
                .on_change(move |_, window, cx| {
                    let (group, name) = (group.clone(), name.clone());
                    this.update(cx, |p, cx| {
                        let groups = list(p, GROUPS, cx);
                        let on = !accounts::has_member(accounts::group(&groups, &group), &name);
                        let me = text(p, CURRENT, cx);
                        as_root(
                            p,
                            accounts::member_write(&group, &name, on, &me),
                            window,
                            cx,
                        )
                    })
                });
            let star = page.stars.star(page.page().id, &shown, "", cx);
            row::setting(
                shown,
                accounts::member_says(&groups, &draft(page, WHO), u),
                "",
                switch,
                star,
            )
        })
        .collect();
    v_flex()
        .gap(Space::Sm)
        .child(Text::new(accounts::manage_says(&group)).muted())
        .children(
            users
                .is_empty()
                .then(|| Text::new("No human logins to add.").muted()),
        )
        .children(rows)
}

fn form(page: &CatalogPage, this: &Entity<CatalogPage>, cx: &App) -> impl IntoElement + use<> {
    let error = draft(page, ERROR);
    let which = open_dialog(page);
    let body = v_flex().gap(Space::Md);
    let body = match which.as_str() {
        "user" => {
            let job = page.jobs.read(cx);
            let adding = job.job_busy() && job.running() == Some("account-add");
            let admin = this.clone();
            body.child(
                Text::new(if adding {
                    "Creating the login…"
                } else {
                    "A local login with a home from /etc/skel."
                })
                .muted(),
            )
            .child(field(
                "acct-user-name",
                "username",
                NAME,
                false,
                page,
                this,
                cx,
            ))
            .child(field(
                "acct-user-full",
                "Full name (optional)",
                FULL,
                false,
                page,
                this,
                cx,
            ))
            .child(field(
                "acct-user-password",
                "Password",
                PASSWORD,
                true,
                page,
                this,
                cx,
            ))
            .child(field(
                "acct-user-confirm",
                "Confirm password",
                CONFIRM,
                true,
                page,
                this,
                cx,
            ))
            .child(row::setting(
                "Admin",
                "Puts this login in wheel.",
                "",
                Switch::new("acct-user-admin", "Admin")
                    .checked(draft(page, ADMIN) == "1")
                    .disabled(busy(page, cx))
                    .on_change(move |on, _, cx| {
                        admin.update(cx, |p, cx| put(p, ADMIN, if on { "1" } else { "" }, cx))
                    }),
                page.stars
                    .star(page.page().id, "Admin", "Puts this login in wheel.", cx),
            ))
            .into_any_element()
        }
        "group" => body
            .child(Text::new("A local group. You can add logins to it after it exists.").muted())
            .child(field(
                "acct-group-name",
                "group",
                NAME,
                false,
                page,
                this,
                cx,
            ))
            .into_any_element(),
        "password" => body
            .child(Text::new(format!("New password for {}.", draft(page, WHO))).muted())
            .child(field(
                "acct-password-new",
                "New password",
                PASSWORD,
                true,
                page,
                this,
                cx,
            ))
            .child(field(
                "acct-password-confirm",
                "Confirm password",
                CONFIRM,
                true,
                page,
                this,
                cx,
            ))
            .into_any_element(),
        _ => members(page, this, cx).into_any_element(),
    };
    v_flex().gap(Space::Md).child(body).children(
        (!error.is_empty()).then(|| Alert::new("acct-error", error).variant(Tone::Danger)),
    )
}

/// The open dialog, built from the page each frame it is shown.
pub fn dialog(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let which = open_dialog(page);
    let title = match which.as_str() {
        "user" => "Add a user".to_owned(),
        "group" => "Add a group".to_owned(),
        "password" => "Set password".to_owned(),
        _ => format!("Manage {}", draft(page, WHO)),
    };
    let (built, closed, cancelled, done) = (this.clone(), this.clone(), this.clone(), this.clone());
    let idle = !busy(page, cx);
    let manage = which == "manage";
    let dialog = Dialog::new("acct-dialog", title)
        .open(!which.is_empty())
        .on_close(move |_, cx| closed.update(cx, close))
        .content(move |_, cx| form(built.read(cx), &built, cx));
    if manage {
        return dialog.action(move |_, _| {
            let done = done.clone();
            Button::new("acct-done")
                .label("Done")
                .variant(ButtonVariant::Primary)
                .on_click(move |_, _, cx| done.update(cx, close))
        });
    }
    let go = if which == "password" { "Set" } else { "Add" };
    dialog
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("acct-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, close))
        })
        .action(move |_, _| {
            let done = done.clone();
            Button::new("acct-go")
                .label(go)
                .variant(ButtonVariant::Primary)
                .disabled(!idle)
                .on_click(move |_, window, cx| done.update(cx, |p, cx| submit(p, window, cx)))
        })
}
