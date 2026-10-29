//! The Diagnostics page's rows (`diagnostics.rs` has what they say): the settings that could not
//! be read, the report's Copy and Ask, and the state of each part, from the diagnostics
//! inventory. Save report writes to a file the person picks, and `odk::prelude` has no file
//! chooser, so it stays Not yet.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::catalog::Field;
use crate::diagnostics::{self as diag, status};
use crate::pages::row;
use crate::settings::{self, Known};

const ASKED: &str = "diag-asked";
const SAVED: &str = "diag-saved";

fn raw(page: &CatalogPage, cx: &App) -> Option<Value> {
    match page.inventories.read(cx).known("diagnostics") {
        Known::Value(doc) => Some(doc.clone()),
        _ => None,
    }
}

fn d(page: &CatalogPage, cx: &App) -> Value {
    diag::normalize(raw(page, cx).as_ref())
}

fn key(page: &CatalogPage, name: &'static str, cx: &App) -> String {
    Field::Key(name)
        .read(page.stars.settings.read(cx))
        .value()
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

fn building(page: &CatalogPage, cx: &App) -> bool {
    let jobs = page.jobs.read(cx);
    jobs.job_busy() && jobs.running() == Some("diag-report")
}

fn info<C: IntoElement>(
    page: &CatalogPage,
    label: String,
    says: String,
    hint: &'static str,
    control: C,
    cx: &App,
) -> impl IntoElement + use<C> {
    let star = page.stars.star(page.page().id, &label, &says, cx);
    row::setting(label, says, hint, h_flex().child(control), star)
}

/// A usage bar: used of total, and what is free.
fn usage(
    id: &str,
    label: &str,
    used: f64,
    total: f64,
    free: f64,
    health: Option<&'static str>,
) -> impl IntoElement + use<> {
    let fmt = crate::hardware::format_bytes;
    v_flex()
        .gap(Space::Xs)
        .children(health.map(Text::new))
        .child(
            Progress::new(SharedString::from(id.to_owned()), label.to_owned())
                .value(used)
                .max(total)
                .value_text(format!(
                    "{} used of {}, {} free",
                    fmt(used),
                    fmt(total),
                    fmt(free)
                )),
        )
}

/// The rows a group's list draws, by group (`stale`, `omarchy`, `atmos`, `hyprland`, `failed`,
/// `errors`).
pub fn rows(which: &str, page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let d = d(page, cx);
    let mut out = Vec::new();
    match which {
        "stale" => {
            let failed = page.stars.settings.read(cx).failed_keys();
            if !failed.is_empty() {
                let (label, says) = diag::stale(&failed);
                out.push(info(page, label, says, "", v_flex(), cx).into_any_element());
            }
        }
        "omarchy" => {
            let v = key(page, "omarchyVersion", cx);
            let label = if v.is_empty() {
                "Omarchy".into()
            } else {
                format!("Omarchy {v}")
            };
            let health = if v.is_empty() {
                "Needs attention"
            } else {
                "OK"
            };
            out.push(
                info(
                    page,
                    label,
                    diag::omarchy_summary(&d),
                    "omarchy version",
                    Text::new(health),
                    cx,
                )
                .into_any_element(),
            );
        }
        "atmos" => {
            let (version, revision) = (
                key(page, "atmosVersion", cx),
                key(page, "atmosRevision", cx),
            );
            let packaged = Field::Key("atmosPackaged")
                .read(page.stars.settings.read(cx))
                .value()
                == Some(&Value::Bool(true));
            let label = if packaged && !version.is_empty() {
                version
            } else if !revision.is_empty() {
                revision
            } else {
                "Atmos".into()
            };
            let health = Text::new(status(diag::atmos_ok(&d)));
            out.push(
                info(
                    page,
                    label,
                    diag::atmos_summary(&d),
                    "~/.config/hypr/hyprland.lua",
                    health,
                    cx,
                )
                .into_any_element(),
            );
        }
        "hyprland" => {
            let v = d["hyprland"]["version"].as_str().unwrap_or_default();
            let label = if v.is_empty() {
                "Hyprland".into()
            } else {
                format!("Hyprland {v}")
            };
            let health = Text::new(status(diag::hypr_ok(&d)));
            out.push(
                info(
                    page,
                    label,
                    diag::hypr_summary(&d),
                    "hyprctl configerrors",
                    health,
                    cx,
                )
                .into_any_element(),
            );
            for e in d["hyprland"]["configErrors"]
                .as_array()
                .into_iter()
                .flatten()
            {
                let said = crate::js::string(e);
                out.push(
                    info(
                        page,
                        "Config error".into(),
                        said,
                        "hyprctl configerrors",
                        v_flex(),
                        cx,
                    )
                    .into_any_element(),
                );
            }
        }
        "failed" => {
            for u in d["failedUnits"].as_array().into_iter().flatten() {
                let s = |k: &str| {
                    u.get(k)
                        .filter(|v| crate::js::truthy(Some(v)))
                        .map(crate::js::string)
                };
                let label = s("unit").unwrap_or_else(|| "unit".into());
                let scope = s("scope").map_or_else(String::new, |s| format!("{s}. "));
                let says = format!(
                    "{scope}{}",
                    s("description").unwrap_or_else(|| "Failed.".into())
                );
                out.push(
                    info(
                        page,
                        label,
                        says,
                        "systemctl --failed",
                        Text::new("Failed"),
                        cx,
                    )
                    .into_any_element(),
                );
            }
        }
        _ => {
            for e in d["recentErrors"].as_array().into_iter().flatten() {
                let said = crate::js::string(e);
                out.push(
                    info(
                        page,
                        "Journal".into(),
                        said,
                        "journalctl -b -p 3",
                        v_flex(),
                        cx,
                    )
                    .into_any_element(),
                );
            }
        }
    }
    out
}

/// What a row of the page's own says, by part.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let d = d(page, cx);
    Some(match which {
        "copy" if building(page, cx) => "Building the report…".into(),
        "copy" => match (page.data.contains_key(ASKED), page.jobs.read(cx).last) {
            (true, Some(("diag-report", true))) => "Could not build the report.".into(),
            (true, Some(("diag-report", false))) => {
                match page.data.get(SAVED).and_then(Value::as_str) {
                    Some(path) => format!("Wrote {path}."),
                    None => "Copied the report.".into(),
                }
            }
            _ => {
                "A text dump of Omarchy, Atmos, Hyprland, systemd, disk, memory, and recent errors."
                    .into()
            }
        },
        "save" => match page.data.get(SAVED).and_then(Value::as_str) {
            Some(path) => format!("Last file: {path}."),
            None => "Write the same report to a text file.".into(),
        },
        "root" if d["disk"]["percent"].as_f64().unwrap_or(0.0) >= 90.0 => {
            "This filesystem is almost full.".into()
        }
        "root" | "ram" | "swap" => String::new(),
        "kernel" => match diag::kernel_summary(&d) {
            k if k.is_empty() => "Kernel version was not readable.".into(),
            k => k,
        },
        "driver" => diag::gpu_summary(&d),
        "portals" => diag::service_map_summary(&d["portals"]),
        "pipewire" => diag::service_map_summary(&d["pipewire"]),
        "connectivity" => diag::network_summary(&d),
        "packages" => diag::pacman_summary(&d),
        _ => return None,
    })
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let d = d(page, cx);
    let n = |p: &str, k: &str| d[p][k].as_f64().unwrap_or(0.0);
    match which {
        "listed" => false,
        "nofailed" => d["failedUnits"].as_array().is_none_or(Vec::is_empty),
        "noerrors" => d["recentErrors"].as_array().is_none_or(Vec::is_empty),
        "root" => n("disk", "total") > 0.0,
        "ram" => n("memory", "total") > 0.0,
        "swap" => n("memory", "swapTotal") > 0.0,
        _ => true,
    }
}

/// A row of the page's own: its control.
pub fn control(
    which: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let d = d(page, cx);
    let n = |p: &str, k: &str| d[p][k].as_f64().unwrap_or(0.0);
    let busy = building(page, cx);
    let this = this.clone();
    match which {
        "copy" | "ask" => {
            let action = if which == "copy" { "copy" } else { "agent" };
            let label = match which {
                "copy" if busy => "Building…",
                "copy" => "Copy",
                _ => "Ask…",
            };
            Button::new(SharedString::from(id))
                .label(label)
                .variant(if which == "copy" {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                })
                .disabled(busy)
                .on_click(move |_, window, cx| {
                    this.update(cx, |p, cx| {
                        let Some(job) = diag::report_job(raw(p, cx).as_ref(), action) else {
                            return;
                        };
                        p.data.insert(ASKED, Value::Bool(true));
                        p.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
                    })
                })
                .into_any_element()
        }
        "save" => {
            let host = key(page, "hostname", cx);
            let saved = page
                .data
                .get(SAVED)
                .and_then(Value::as_str)
                .map(str::to_owned);
            Button::new(SharedString::from(id))
                .label("Save…")
                .variant(ButtonVariant::Outline)
                .disabled(busy)
                .on_click(move |_, window, cx| {
                    let now = crate::settings::now_ms() / 1000;
                    // The last report's folder, else Pictures (`folderUrl`).
                    let start = match saved
                        .as_deref()
                        .and_then(|p| p.rsplit_once('/'))
                        .filter(|(dir, _)| !dir.is_empty())
                    {
                        Some((dir, _)) => StartFolder::Path(dir.into()),
                        None => StartFolder::Pictures,
                    };
                    let chooser = FileChooser::save("Save diagnostic report")
                        .name(diag::report_file_name(&host, now))
                        .filter("Text files", ["*.txt"])
                        .filter("All files", ["*"])
                        .start_in(start);
                    crate::pages::files::choose(
                        chooser,
                        &this,
                        window,
                        cx,
                        Box::new(|p, path, window, cx| {
                            let Some((path, job)) = diag::save_job(raw(p, cx).as_ref(), &path)
                            else {
                                return;
                            };
                            p.data.insert(SAVED, Value::from(path));
                            p.data.insert(ASKED, Value::Bool(true));
                            p.jobs.update(cx, |jobs, cx| jobs.run(job, window, cx));
                        }),
                    );
                })
                .into_any_element()
        }
        "refresh" => {
            let client = page.stars.client.clone();
            Button::new(SharedString::from(id))
                .label("Refresh")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, _| {
                    drop(
                        client.request(&crate::protocol::snapshot_invalidate(settings::ALL_TOPICS)),
                    )
                })
                .into_any_element()
        }
        "nofailed" | "noerrors" => Text::new("OK").into_any_element(),
        "root" => usage(
            &id,
            "Root",
            n("disk", "used"),
            n("disk", "total"),
            n("disk", "available"),
            Some(status(!diag::disk_pressure(&d))),
        )
        .into_any_element(),
        "ram" => usage(
            &id,
            "RAM",
            n("memory", "used"),
            n("memory", "total"),
            n("memory", "available"),
            Some(status(!diag::memory_pressure(&d))),
        )
        .into_any_element(),
        "swap" => {
            let (used, total) = (n("memory", "swapUsed"), n("memory", "swapTotal"));
            usage(&id, "Swap", used, total, (total - used).max(0.0), None).into_any_element()
        }
        "kernel" | "driver" => {
            let text = if which == "kernel" {
                diag::kernel_summary(&d)
            } else {
                diag::gpu_summary(&d)
            };
            let client = page.stars.client.clone();
            Button::new(SharedString::from(id))
                .label("Copy")
                .variant(ButtonVariant::Outline)
                .disabled(text.is_empty())
                .on_click(move |_, window, cx| row::copy(&client, &text, window, cx))
                .into_any_element()
        }
        "portals" => Text::new(status(diag::services_ok(&d["portals"]))).into_any_element(),
        "pipewire" => Text::new(status(diag::services_ok(&d["pipewire"]))).into_any_element(),
        "connectivity" => {
            Text::new(status(d["network"]["online"] == Value::Bool(true))).into_any_element()
        }
        "packages" => {
            Text::new(status(d["pacman"]["syncOk"] == Value::Bool(true))).into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}
