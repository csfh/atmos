//! The Software page's rows (`software.rs` has what they say and start): each stack with
//! Install… or Remove…, which ask first, the language toolchains, and the Docker databases.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::catalog::Field;
use crate::jobs::Job;
use crate::pages::controls::select;
use crate::pages::row;
use crate::software::{self, Item};

const DEV: &str = "devLang";
const DB: &str = "dockerDb";

fn busy(page: &CatalogPage, cx: &App) -> bool {
    page.jobs.read(cx).job_busy()
}

/// Asks the confirm, and runs `job` on yes.
fn ask(
    page: &mut CatalogPage,
    remove: bool,
    label: &str,
    wipe: bool,
    job: Job,
    cx: &mut Context<CatalogPage>,
) {
    let (title, message) = software::confirm(remove, label, wipe);
    page.ask(
        Asked {
            title,
            message,
            action: if remove { "Remove" } else { "Install" },
            go: Rc::new(move |p, window, cx| {
                let job = job.clone();
                p.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
            }),
        },
        cx,
    );
}

/// The stacks of a group, each with Install… or Remove….
pub fn items(
    group: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let busy = busy(page, cx);
    software::group_items(group)
        .map(|item: &'static Item| {
            let bag = software::bag(item)
                .read(page.stars.settings.read(cx))
                .value()
                .cloned();
            let installed = software::installed(item, bag.as_ref());
            let (argv, remove) = if installed {
                (item.remove, true)
            } else {
                (item.install, false)
            };
            let this = this.clone();
            let control = argv.map(|argv| {
                Button::new(SharedString::from(format!("software-{}", item.id)))
                    .label(if remove { "Remove…" } else { "Install…" })
                    .variant(if remove {
                        ButtonVariant::Destructive
                    } else {
                        ButtonVariant::Primary
                    })
                    .disabled(busy)
                    .on_click(move |_, _, cx| {
                        let Some(job) = software::software_job(argv, remove) else {
                            return;
                        };
                        this.update(cx, |p, cx| {
                            ask(p, remove, item.label, remove && item.wipe, job, cx)
                        })
                    })
            });
            let says = software::description(item, installed);
            let star = page.stars.star(page.page().id, item.label, says, cx);
            row::setting(
                item.label,
                says,
                item.hint,
                h_flex().children(control),
                star,
            )
            .into_any_element()
        })
        .collect()
}

fn draft(page: &CatalogPage, key: &str, or: &str) -> String {
    software::choice(page.drafts.get(key), or)
}

/// A choice the page holds: its draft's name, where it starts, and its options.
type Choice = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

const LANGS: Choice = (DEV, software::DEV_LANG, software::DEV_ENVS);
const DBS: Choice = (DB, software::DOCKER_DB, software::DOCKER_DBS);

fn pick(
    id: String,
    label: &'static str,
    (key, or, list): Choice,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let options: Vec<(String, String)> = list
        .iter()
        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
        .collect();
    let now = Value::from(draft(page, key, or));
    let this = this.clone();
    select(
        id,
        label,
        &options,
        &now,
        busy(page, cx),
        move |v, _, cx| {
            let v = v.to_owned();
            this.update(cx, |p, cx| {
                p.drafts.insert(key.into(), v);
                cx.notify();
            })
        },
    )
    .into_any_element()
}

/// Language toolchain: a language, Install… and Remove….
pub fn dev_env(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let busy = busy(page, cx);
    let lang = draft(page, DEV, software::DEV_LANG);
    let button = |remove: bool| {
        let (this, lang) = (this.clone(), lang.clone());
        Button::new(SharedString::from(format!(
            "{id}-{}",
            if remove { "remove" } else { "install" }
        )))
        .label(if remove { "Remove…" } else { "Install…" })
        .variant(if remove {
            ButtonVariant::Destructive
        } else {
            ButtonVariant::Primary
        })
        .disabled(busy)
        .on_click(move |_, _, cx| {
            let Some(job) = software::dev_env_job(&lang, remove) else {
                return;
            };
            this.update(cx, |p, cx| ask(p, remove, &lang, false, job, cx))
        })
    };
    h_flex()
        .gap(Space::Sm)
        .child(pick(
            format!("{id}-lang"),
            "Language",
            LANGS,
            page,
            this,
            cx,
        ))
        .child(button(false))
        .child(button(true))
}

/// Whether the Docker database row shows.
pub fn has_docker(page: &CatalogPage, cx: &App) -> bool {
    software::has_docker(
        Field::Key("extras")
            .read(page.stars.settings.read(cx))
            .value(),
    )
}

/// Docker database: a database, and Install.
pub fn docker_db(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let name = draft(page, DB, software::DOCKER_DB);
    let installed = this.clone();
    h_flex()
        .gap(Space::Sm)
        .child(pick(format!("{id}-db"), "Database", DBS, page, this, cx))
        .child(
            Button::new(SharedString::from(format!("{id}-install")))
                .label("Install")
                .variant(ButtonVariant::Primary)
                .disabled(busy(page, cx))
                .on_click(move |_, window, cx| {
                    let Some(job) = software::docker_db_job(&name) else {
                        return;
                    };
                    installed.update(cx, |p, cx| {
                        p.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx))
                    })
                }),
        )
}
