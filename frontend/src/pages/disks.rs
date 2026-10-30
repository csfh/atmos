//! The Disks page's rows (`disks.rs` has what they say and send): each drive as a group of its
//! own with its mounts and speed test, the encryption dialog, Snapper's snapshots and settings,
//! hibernation, TRIM and swap. Snapper's and TRIM's writes run as root, behind the sudo prompt.

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use crate::catalog::Field;
use crate::disks::{self, Run};
use crate::jobs::{Job, RootWrite};
use crate::pages::row;
use crate::settings::Known;

const RUN: &str = "disk-run";
const LUKS: &str = "luks-device";
const CURRENT: &str = "luks-current";
const NEW: &str = "luks-new";
const AGAIN: &str = "luks-again";
const HINT: &str = "luks-hint";

/// The disks inventory's list `name`.
fn list(page: &CatalogPage, name: &str, cx: &App) -> Vec<Value> {
    match page.inventories.read(cx).known("disks") {
        Known::Value(doc) => doc
            .get(name)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// Whether a group is drawn while it has no row: the inventory lists a volume of its kind
/// (`luks`, `swap`; `catalog::SHOWN`).
pub fn shows(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let name = if which == "luks" {
        "luksDevices"
    } else {
        "swapDevices"
    };
    !list(page, name, cx).is_empty()
}

fn key(page: &CatalogPage, name: &'static str, cx: &App) -> Option<Value> {
    Field::Key(name)
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn on(page: &CatalogPage, name: &'static str, cx: &App) -> bool {
    key(page, name, cx) == Some(Value::Bool(true))
}

fn busy(page: &CatalogPage, cx: &App) -> bool {
    page.jobs.read(cx).job_busy()
}

fn run_job(page: &mut CatalogPage, job: Job, window: &mut Window, cx: &mut Context<CatalogPage>) {
    page.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
}

fn as_root(
    page: &mut CatalogPage,
    key: &'static str,
    body: Option<Value>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let Some(body) = body else {
        return;
    };
    let write = RootWrite {
        key,
        body,
        refused: "The disk setting was not changed".into(),
    };
    page.jobs
        .update(cx, |jobs, cx| jobs.write_as_root(write, window, cx));
}

fn run(page: &CatalogPage) -> Run {
    Run::of(page.data.get(RUN))
}

fn save(page: &mut CatalogPage, r: &Run, cx: &mut Context<CatalogPage>) {
    page.data.insert(RUN, r.to_value());
    cx.notify();
}

/// Run now on the drive whose mount is `dir`, or Cancel the run on it.
fn speedtest(page: &mut CatalogPage, dir: String, cx: &mut Context<CatalogPage>) {
    let mut r = run(page);
    if r.running && r.target == dir {
        r.stop();
        save(page, &r, cx);
        return;
    }
    let Some(body) = r.start(&dir) else {
        return;
    };
    let token = r.token;
    save(page, &r, cx);
    let reply = page.stars.client.request(&body);
    cx.spawn(async move |this, cx| {
        let outcome = reply.await;
        let _ = this.update(cx, |p, cx| {
            let mut r = run(p);
            if r.token != token {
                return;
            }
            match &outcome {
                Ok(result) => r.finish(Ok(result)),
                Err(f) => r.finish(Err(&f.message)),
            }
            save(p, &r, cx);
        });
    })
    .detach();
}

fn copy(id: String, text: String, page: &CatalogPage) -> impl IntoElement + use<> {
    let client = page.stars.client.clone();
    Button::new(SharedString::from(id))
        .label("Copy")
        .variant(ButtonVariant::Outline)
        .disabled(text.is_empty())
        .on_click(move |_, window, cx| row::copy(&client, &text, window, cx))
}

/// Each drive, as a group of its own: the drive, its mounts, and its speed test.
pub fn drives(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let r = run(page);
    list(page, "disks", cx)
        .iter()
        .enumerate()
        .map(|(i, disk)| {
            let mut rows = Vec::new();
            let info = disk.get("info").filter(|v| crate::js::truthy(Some(v))).map_or_else(String::new, crate::js::string);
            let star = page.stars.star(page.page().id, "Drive", &info, cx);
            let control = copy(format!("disk-{i}-copy"), info.clone(), page);
            rows.push(row::setting("Drive", info, "omarchy drive info", control, star).into_any_element());
            for (j, m) in disk.get("mounts").and_then(Value::as_array).into_iter().flatten().enumerate() {
                let n = |k: &str| crate::js::number(m.get(k)).max(0.0);
                let (used, size) = (n("used"), n("size"));
                let label = m.get("target").filter(|v| crate::js::truthy(Some(v))).map_or_else(|| "Mount".to_owned(), crate::js::string);
                let bar = Progress::new(SharedString::from(format!("disk-{i}-mount-{j}")), label.clone())
                    .value(used)
                    .max(size)
                    .value_text(format!(
                        "{} used of {}, {} free",
                        crate::hardware::format_bytes(used),
                        crate::hardware::format_bytes(size),
                        crate::hardware::format_bytes(n("avail"))
                    ));
                let star = page.stars.star(page.page().id, &label, "", cx);
                rows.push(row::stacked(label, "", "df", bar, star).into_any_element());
            }
            let dir = disks::speedtest_dir(disk);
            let says = r.says(&dir);
            let this = this.clone();
            let target = dir.clone();
            let error = r.error(&dir);
            let control = v_flex()
                .gap(Space::Sm)
                .child(
                    Button::new(SharedString::from(format!("disk-{i}-speed")))
                        .label(r.button(&dir))
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            let dir = target.clone();
                            this.update(cx, |p, cx| speedtest(p, dir, cx))
                        }),
                )
                .children(r.rates(&dir).map(Text::new))
                .children((!error.is_empty()).then(|| Alert::new(SharedString::from(format!("disk-{i}-error")), error).variant(Tone::Danger)));
            let star = page.stars.star(page.page().id, "Speed test", &says, cx);
            rows.push(row::stacked("Speed test", says, "omarchy disk speedtest", control, star).into_any_element());
            row::list_group(
                disks::title(disk),
                "Usage bars come from df on each mount. Speed test reads for about eight seconds, then writes for about eight, on this disk.",
                rows,
            )
            .into_any_element()
        })
        .collect()
}

/// The encrypted volumes, each with Change….
pub fn luks(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let busy = busy(page, cx);
    list(page, "luksDevices", cx)
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let device = if crate::js::truthy(Some(d)) { crate::js::string(d) } else { "LUKS".into() };
            let says = "This volume is encrypted with LUKS. Change the passphrase here. You will need the current one.";
            let this = this.clone();
            let chosen = device.clone();
            let change = Button::new(SharedString::from(format!("luks-{i}")))
                .label("Change…")
                .variant(ButtonVariant::Outline)
                .disabled(busy)
                .on_click(move |_, _, cx| {
                    let chosen = chosen.clone();
                    this.update(cx, |p, cx| {
                        for k in [CURRENT, NEW, AGAIN, HINT] {
                            p.drafts.insert(k.into(), String::new());
                        }
                        p.drafts.insert(LUKS.into(), chosen);
                        cx.notify();
                    })
                });
            let star = page.stars.star(page.page().id, &device, says, cx);
            row::setting(device, says, "cryptsetup luksChangeKey", change, star).into_any_element()
        })
        .collect()
}

fn draft(page: &CatalogPage, k: &str) -> String {
    page.drafts.get(k).cloned().unwrap_or_default()
}

fn close_luks(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    for k in [LUKS, CURRENT, NEW, AGAIN] {
        page.drafts.remove(k);
    }
    cx.notify();
}

/// Change: the passphrases checked, then the job.
fn change(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let (device, cur, new, again) = (
        draft(page, LUKS),
        draft(page, CURRENT),
        draft(page, NEW),
        draft(page, AGAIN),
    );
    match disks::change_key(&device, &cur, &new, &again) {
        Err(why) => {
            page.drafts.insert(HINT.into(), why.into());
            cx.notify();
        }
        Ok(job) => {
            page.drafts.insert(HINT.into(), String::new());
            if let Some(job) = job {
                run_job(page, job, window, cx);
            }
            close_luks(page, cx);
        }
    }
}

/// The encryption dialog.
pub fn luks_dialog(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let device = draft(page, LUKS);
    let hint = draft(page, HINT);
    let busy = busy(page, cx);
    let (closed, cancelled, changed, typed) =
        (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("luks", "Change encryption password")
        .open(!device.is_empty())
        .description(format!(
            "New passphrase for {device}. You will need the current one."
        ))
        .on_close(move |_, cx| closed.update(cx, close_luks))
        .content(move |_, _| {
            let field = |k: &'static str, name: &'static str| {
                let typed = typed.clone();
                PasswordInput::new(SharedString::from(k), name)
                    .placeholder(name)
                    .on_change(move |t, _, cx| {
                        typed.update(cx, |p, _| {
                            p.drafts.insert(k.into(), t.to_owned());
                        })
                    })
            };
            v_flex()
                .gap(Space::Sm)
                .child(field(CURRENT, "Current passphrase"))
                .child(field(NEW, "New passphrase"))
                .child(field(AGAIN, "Confirm new passphrase"))
                .children(
                    (!hint.is_empty())
                        .then(|| Alert::new("luks-hint", hint.clone()).variant(Tone::Danger)),
                )
        })
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("luks-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| cancelled.update(cx, close_luks))
        })
        .action(move |_, _| {
            let changed = changed.clone();
            Button::new("luks-change")
                .label("Change")
                .variant(ButtonVariant::Primary)
                .disabled(busy)
                .on_click(move |_, window, cx| changed.update(cx, |p, cx| change(p, window, cx)))
        })
}

/// The snapshots, each with Roll back…, while Snapper is here.
pub fn snapshots(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    if !on(page, "snapperPresent", cx) {
        return Vec::new();
    }
    let busy = busy(page, cx);
    list(page, "snapshots", cx)
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let label = disks::snapshot_label(item);
            let says = "Make this snapshot the new Snapper default, then reboot into it.";
            let config = item.get("config").filter(|v| crate::js::truthy(Some(v))).map_or_else(|| "root".to_owned(), crate::js::string);
            let id = item.get("id").map_or_else(String::new, crate::js::string);
            let usable = crate::js::truthy(item.get("id"));
            let this = this.clone();
            let back = Button::new(SharedString::from(format!("snapshot-{i}")))
                .label("Roll back…")
                .variant(ButtonVariant::Destructive)
                .disabled(busy || !usable)
                .on_click(move |_, _, cx| {
                    let (config, id) = (config.clone(), id.clone());
                    this.update(cx, |p, cx| {
                        p.ask(
                            Asked {
                                title: "Roll back".into(),
                                message: "Snapper will make this snapshot the new default. Reboot after it finishes so the machine boots into it.".into(),
                                action: "Roll back",
                                go: Rc::new(move |p, window, cx| {
                                    if let Some(job) = disks::rollback(&config, &id) {
                                        run_job(p, job, window, cx);
                                    }
                                }),
                            },
                            cx,
                        )
                    })
                });
            let star = page.stars.star(page.page().id, &label, says, cx);
            row::setting(label, says, "snapper rollback", back, star).into_any_element()
        })
        .collect()
}

/// The swap devices, each with Copy.
pub fn swap(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    list(page, "swapDevices", cx)
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let (label, says) = disks::swap(item);
            let control = copy(format!("swap-{i}-copy"), disks::swap_copy(item), page);
            let star = page.stars.star(page.page().id, &label, &says, cx);
            row::setting(label, says, "lsblk", control, star).into_any_element()
        })
        .collect()
}

/// What a row of the page's own says, by part.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    Some(match which {
        "snapper" => disks::snapper_summary(key(page, "snapperConfigs", cx).as_ref()),
        "hibernation" => {
            let running = page.jobs.read(cx).running().filter(|_| busy(page, cx));
            let configured = on(page, "hibernationConfigured", cx);
            disks::hibernation_says(running, configured, on(page, "hibernationSupported", cx))
                .into()
        }
        _ => return None,
    })
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    match which {
        "listed" => false,
        "snapper" | "limit" | "timeline" => on(page, "snapperPresent", cx),
        "nosnapshots" => on(page, "snapperPresent", cx) && list(page, "snapshots", cx).is_empty(),
        _ => true,
    }
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
    let this = this.clone();
    match which {
        "snapper" => copy(
            id,
            disks::snapper_summary(key(page, "snapperConfigs", cx).as_ref()),
            page,
        )
        .into_any_element(),
        "hibernation" => {
            let configured = on(page, "hibernationConfigured", cx);
            let supported = on(page, "hibernationSupported", cx);
            Button::new(SharedString::from(id))
                .label(if configured { "Remove…" } else { "Set up…" })
                .variant(if configured { ButtonVariant::Destructive } else { ButtonVariant::Primary })
                .disabled(busy(page, cx) || !(configured || supported))
                .on_click(move |_, _, cx| {
                    let (title, message, action) = if configured {
                        ("Remove hibernation", "Remove the hibernation swap file and the boot resume settings that go with it.", "Remove")
                    } else {
                        ("Set up hibernation", "Write a RAM-sized swap file on the boot drive and set up resume. This uses sudo.", "Set up")
                    };
                    this.update(cx, |p, cx| {
                        p.ask(
                            Asked {
                                title: title.into(),
                                message: message.into(),
                                action,
                                go: Rc::new(move |p, window, cx| run_job(p, disks::hibernation(configured), window, cx)),
                            },
                            cx,
                        )
                    })
                })
                .into_any_element()
        }
        "limit" => {
            let Some(limit) = key(page, "snapperNumberLimit", cx) else {
                return row::reading(label).into_any_element();
            };
            NumberInput::new(SharedString::from(id), label)
                .value(crate::js::number(Some(&limit)))
                .min(1.0)
                .max(20.0)
                .step(1.0)
                .disabled(!on(page, "snapperPresent", cx))
                .on_commit(move |n, window, cx| {
                    let limit = limit.clone();
                    this.update(cx, |p, cx| {
                        as_root(
                            p,
                            "snapperNumberLimit",
                            disks::number_limit(n, Some(&limit)),
                            window,
                            cx,
                        )
                    })
                })
                .into_any_element()
        }
        "timeline" | "fstrim" => {
            let (field, name) = if which == "timeline" {
                ("snapperTimeline", "snapper.timeline")
            } else {
                ("fstrimEnabled", "fstrim")
            };
            let Some(now) = key(page, field, cx) else {
                return row::reading(label).into_any_element();
            };
            let checked = now == Value::Bool(true);
            Switch::new(SharedString::from(id), label)
                .checked(checked)
                .disabled(which == "timeline" && !on(page, "snapperPresent", cx))
                .on_change(move |next, window, cx| {
                    let now = now.clone();
                    this.update(cx, |p, cx| {
                        as_root(p, field, disks::flag(name, next, Some(&now)), window, cx)
                    })
                })
                .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}
