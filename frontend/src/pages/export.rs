//! The Export page's rows (`export.rs` has what they hold, say and send; `omafile` what a file
//! holds): the sections to write, where, and Export; the file to read, Review, what the file
//! and its plan say; Apply, which asks first (and again for the commands a file installs), its
//! progress, and Put it back. A file is written, opened and read by `atmosd` (`host.write`,
//! `host.open`, `host.read`); the window never touches it.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::export::{self, State};
use crate::jobs::Jobs;
use crate::omafile::{self, lines, plan};
use crate::pages::row;

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

/// The page as it opens.
pub fn state() -> State {
    State::new(&home())
}

fn applying_job(page: &CatalogPage, cx: &App) -> bool {
    matches!(
        page.jobs.read(cx).running(),
        Some("settings-import" | "settings-undo")
    )
}

/// `working`: a file is being read or written, or an apply runs or waits.
fn working(page: &CatalogPage, cx: &App) -> bool {
    page.omafile.io.is_some() || applying_job(page, cx) || page.omafile.pending_apply
}

fn snapshot(page: &CatalogPage, cx: &App) -> Value {
    export::snapshot(page.stars.settings.read(cx))
}

fn options(snap: &Value) -> plan::Options {
    plan::Options {
        hardware: omafile::text_or_empty(snap.get("dmiProduct")),
        workspace_gesture_unmanaged: snap.get("hyprWorkspaceGestureUnmanaged")
            == Some(&Value::Bool(true)),
    }
}

/// A file chosen to write to (`exportFileDialog`), or to read (`importFileDialog`).
pub fn chose(page: &mut CatalogPage, save: bool, path: String) {
    let s = &mut page.omafile;
    if save {
        s.export_path = export::with_md(&path, &home());
        s.forget_export();
    } else {
        s.import_path = path;
        s.forget_plan();
    }
}

/// What the page holds, for what a test asks.
pub fn held_state(page: &CatalogPage) -> &State {
    &page.omafile
}

/// An apply or an undo that ended, or that the sudo prompt kept from running.
pub fn took(page: &mut CatalogPage, jobs: &Jobs) {
    let s = &mut page.omafile;
    if !s.pending_apply {
        return;
    }
    if jobs.ended > s.seen.0 {
        s.seen.0 = jobs.ended;
        if let Some(("settings-import" | "settings-undo", failed)) = jobs.last {
            s.finished(!failed, &jobs.last_out, &jobs.last_err);
        }
    }
    if jobs.cancelled.0 > s.seen.1 {
        s.seen.1 = jobs.cancelled.0;
        if let Some("settings-import" | "settings-undo") = jobs.cancelled.1 {
            s.refused();
        }
    }
}

/// Asks `atmosd` to write, open or read a file, and takes in its answer.
fn io(
    page: &mut CatalogPage,
    kind: &'static str,
    body: Value,
    path: String,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if page.omafile.io.is_some() {
        return;
    }
    page.omafile.io = Some(kind);
    let reply = page.stars.client.request(&body);
    cx.spawn_in(window, async move |this, cx| {
        let outcome = reply.await;
        let _ = this.update(cx, |p, cx| {
            let snap = snapshot(p, cx);
            let opts = options(&snap);
            match &outcome {
                Ok(v) => p.omafile.answered(Ok(v), &path, &snap, &opts),
                Err(f) => {
                    let why = export::error_text(&f.message, f.context.as_deref());
                    p.omafile.answered(Err(&why), &path, &snap, &opts)
                }
            }
            cx.notify();
        });
    })
    .detach();
    cx.notify();
}

fn export_now(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let snap = snapshot(page, cx);
    let now = export::iso(crate::settings::now_ms());
    let Some(text) = page.omafile.export_text(&snap, &now) else {
        return;
    };
    page.omafile.forget_export();
    let path = export::real_path(&page.omafile.export_path, &home());
    let body = crate::protocol::host_write(&path, &text);
    io(page, "write", body, path, window, cx);
}

fn run(page: &mut CatalogPage, undo: bool, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let seen = {
        let j = page.jobs.read(cx);
        (j.ended, j.cancelled.0)
    };
    let job = if undo {
        page.omafile.undo(seen)
    } else {
        page.omafile.apply(seen)
    };
    if let Some(job) = job {
        page.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
    }
    cx.notify();
}

fn asked(title: &str, message: String, action: &'static str, go: super::catalog::Then) -> Asked {
    Asked {
        title: title.into(),
        message,
        action,
        go,
    }
}

/// Apply's two confirms: the plan, then the commands a file installs, if it does.
fn ask_apply(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    let Some(plan) = page.omafile.plan.clone() else {
        return;
    };
    let message = lines::apply_confirm(&plan);
    page.ask(
        asked(
            "Apply this Omafile?",
            message,
            "Apply",
            Rc::new(move |p, window, cx| {
                if lines::has_commands(&plan) {
                    let commands = lines::command_confirm(&plan);
                    let go: super::catalog::Then =
                        Rc::new(|p, window, cx| run(p, false, window, cx));
                    p.ask(
                        asked("These commands will run", commands, "Apply commands", go),
                        cx,
                    );
                } else {
                    run(p, false, window, cx);
                }
            }),
        ),
        cx,
    );
}

fn button(id: String, label: &'static str, variant: ButtonVariant, disabled: bool) -> Button {
    Button::new(SharedString::from(id))
        .label(label)
        .variant(variant)
        .disabled(disabled)
}

fn chooser(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    save: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let filters = |c: FileChooser| c.filter("Omafile", ["*.md"]).filter("All files", ["*"]);
    let s = &page.omafile;
    let chooser = if save {
        let folder = export::folder(&s.export_path, &home());
        filters(FileChooser::save("Write an Omafile"))
            .name(omafile::file_name())
            .start_in(StartFolder::Path(folder.into()))
    } else {
        let folder = export::folder(&s.import_path, &home());
        filters(FileChooser::open("Open an Omafile")).start_in(StartFolder::Path(folder.into()))
    };
    crate::pages::files::choose(
        chooser,
        this,
        window,
        cx,
        Box::new(move |p, path, _, cx| {
            chose(p, save, path);
            cx.notify();
        }),
    );
}

/// The control of a row of the page's own.
pub fn control(
    which: &str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let busy = working(page, cx);
    let s = &page.omafile;
    type Act = fn(&mut CatalogPage, &mut Window, &mut Context<CatalogPage>);
    let on = |b: Button, f: Act| {
        let this = this.clone();
        b.on_click(move |_, window, cx| this.update(cx, |p, cx| f(p, window, cx)))
    };
    match which {
        "sections" => h_flex()
            .gap(Space::Sm)
            .child(on(
                button(format!("{id}-all"), "All", ButtonVariant::Outline, busy),
                |p, _, cx| {
                    p.omafile.set_all(true);
                    cx.notify()
                },
            ))
            .child(on(
                button(format!("{id}-none"), "None", ButtonVariant::Outline, busy),
                |p, _, cx| {
                    p.omafile.set_all(false);
                    cx.notify()
                },
            ))
            .into_any_element(),
        "where" => {
            let this = this.clone();
            button(id, "Choose…", ButtonVariant::Outline, busy)
                .on_click(move |_, window, cx| {
                    let page = this.clone();
                    this.update(cx, |p, cx| chooser(p, &page, true, window, cx))
                })
                .into_any_element()
        }
        "write" => h_flex()
            .gap(Space::Sm)
            .child(on(
                button(
                    format!("{id}-export"),
                    "Export",
                    ButtonVariant::Primary,
                    busy || s.chosen_keys().is_empty(),
                ),
                export_now,
            ))
            .child(on(
                button(
                    format!("{id}-open"),
                    "Open file",
                    ButtonVariant::Outline,
                    busy || s.written_path.is_empty(),
                ),
                |p, window, cx| {
                    let path = export::real_path(&p.omafile.written_path, &home());
                    let body = crate::protocol::host_open(&path);
                    io(p, "open", body, path, window, cx)
                },
            ))
            .into_any_element(),
        "file" => {
            let this = this.clone();
            button(id, "Choose…", ButtonVariant::Outline, busy)
                .on_click(move |_, window, cx| {
                    let page = this.clone();
                    this.update(cx, |p, cx| chooser(p, &page, false, window, cx))
                })
                .into_any_element()
        }
        "review" => on(
            button(
                id,
                "Review",
                ButtonVariant::Outline,
                busy || s.import_path.is_empty(),
            ),
            |p, window, cx| {
                p.omafile.forget_plan();
                let path = export::real_path(&p.omafile.import_path, &home());
                let body = crate::protocol::host_read(&[&path]);
                io(p, "read", body, path, window, cx)
            },
        )
        .into_any_element(),
        "summary" => Text::new(
            s.last_doc
                .as_ref()
                .map(|d| lines::file_summary(d, s.plan.as_ref()))
                .unwrap_or_default(),
        )
        .muted()
        .into_any_element(),
        "changes" => Text::new(s.plan.as_ref().map(lines::change_lines).unwrap_or_default())
            .into_any_element(),
        "warnings" => Text::new(
            s.plan
                .as_ref()
                .map(lines::warning_lines)
                .unwrap_or_default(),
        )
        .into_any_element(),
        "blocked" => Alert::new(
            SharedString::from(id),
            s.plan
                .as_ref()
                .map(lines::blocked_lines)
                .unwrap_or_default(),
        )
        .variant(Tone::Danger)
        .into_any_element(),
        "apply" => on(
            button(id, "Apply…", ButtonVariant::Destructive, busy),
            |p, _, cx| ask_apply(p, cx),
        )
        .into_any_element(),
        "applying" => {
            let (now, total, _) = export::step(page.jobs.read(cx).output(), s.step_total);
            Progress::new(SharedString::from(id), "Applying")
                .value(now as f64)
                .max(total.max(1) as f64)
                .value_text(if total > 0 {
                    format!("{now} / {total}")
                } else {
                    String::new()
                })
                .into_any_element()
        }
        "putback" => on(
            button(
                id,
                "Put it back…",
                if s.applied_count > 0 {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                },
                busy || s.last_backup_dir.is_empty(),
            ),
            |p, _, cx| {
                let go: super::catalog::Then = Rc::new(|p, window, cx| run(p, true, window, cx));
                let message = "Puts back the values the last import replaced.".to_owned();
                p.ask(asked("Undo the last import?", message, "Undo", go), cx)
            },
        )
        .into_any_element(),
        _ => v_flex().into_any_element(),
    }
}

/// What a row of the page's own says.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let s = &page.omafile;
    let or = |text: &str, empty: &str| {
        if text.is_empty() {
            empty.to_owned()
        } else {
            text.to_owned()
        }
    };
    Some(match which {
        "sections" => s.sections_says(),
        "where" => s.export_path.clone(),
        "write" => or(&s.export_status, "Writes the sections selected above."),
        "file" => or(&s.import_path, "No file chosen yet."),
        "review" => or(
            &s.import_status,
            "Reads the file and compares it against this machine. Changes nothing.",
        ),
        "changes" => s
            .plan
            .as_ref()
            .map(|p| crate::js::string(&p["summary"]))
            .unwrap_or_default(),
        "apply" => match &s.plan {
            Some(p) => lines::forecast(p),
            None => "Runs exactly the changes listed above.".into(),
        },
        "applying" => export::applying_says(
            &export::step(page.jobs.read(cx).output(), s.step_total),
            page.jobs.read(cx).prompt().is_some(),
        ),
        "putback" => export::put_back_says(s.applied_count),
        _ => return None,
    })
}

/// Whether a row of the page's own shows (`available:`).
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let s = &page.omafile;
    let listed = |k: &str| {
        s.plan
            .as_ref()
            .and_then(|p| p[k].as_array())
            .is_some_and(|l| !l.is_empty())
    };
    match which {
        "summary" => s
            .last_doc
            .as_ref()
            .is_some_and(|d| !lines::file_summary(d, s.plan.as_ref()).is_empty()),
        "changes" | "apply" => s.has_changes(),
        "warnings" => listed("warnings"),
        "blocked" => listed("blocked"),
        "applying" => applying_job(page, cx) || s.pending_apply,
        "putback" => !s.last_backup_dir.is_empty(),
        _ => true,
    }
}

/// Whether a row's control takes the row's width (`stretchControl`).
pub fn stacked(which: &str) -> bool {
    matches!(
        which,
        "summary" | "changes" | "warnings" | "blocked" | "applying"
    )
}

/// A row per section offered, each with its switch and its count.
pub fn sections(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let busy = working(page, cx);
    omafile::selectable_sections()
        .into_iter()
        .map(|o| {
            let this = this.clone();
            let id = o.id;
            let switch = Switch::new(SharedString::from(format!("section-{id}")), o.title)
                .checked(page.omafile.chosen(id))
                .disabled(busy)
                .on_change(move |on, _, cx| {
                    this.update(cx, |p, cx| {
                        p.omafile.set_section(id, on);
                        cx.notify()
                    })
                });
            let star = page.stars.star(page.page().id, o.title, o.note, cx);
            row::captioned(
                o.title,
                o.note,
                format!("{} settings", o.count),
                "",
                switch,
                star,
            )
            .into_any_element()
        })
        .collect()
}
