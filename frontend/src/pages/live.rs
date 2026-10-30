//! The Home and Monitor hub pages' live rows (`monitor.rs` has what they say, `stats.rs` what they
//! draw, `processes.rs` what each process row says and sends): Home's quick links, the hero line,
//! the activity and thermal sparklines, the GPUs, and this user's processes; the Monitor hub's
//! sampling chips, meters, core and memory bars, page cards, and hottest tasks. A process's End
//! and Force quit ask first, then signal it; the Monitor hub's do too, where the QML row's
//! buttons send nothing (its `onActed` handles only the copies).

use std::rc::Rc;

use odk::prelude::*;
use serde_json::Value;

use super::catalog::{Asked, CatalogPage};
use super::charts::{self, Spark};
use crate::feed::INTERVALS;
use crate::monitor::{self, Captions, Home};
use crate::pages::row;
use crate::processes::{self, Filter};
use crate::settings::Known;
use crate::stats;

const HOME_QUERY: &str = "home-query";
const HOME_SORT: &str = "home-sort";
const MONITOR_QUERY: &str = "monitor-query";

pub(super) fn draft(page: &CatalogPage, key: &str) -> String {
    page.drafts.get(key).cloned().unwrap_or_default()
}

pub(super) fn put(
    page: &mut CatalogPage,
    key: &str,
    value: impl Into<String>,
    cx: &mut Context<CatalogPage>,
) {
    page.drafts.insert(key.into(), value.into());
    cx.notify();
}

/// The live rows, and a number that moves with each sample, for a chart's transition.
pub(super) fn history<'a>(page: &CatalogPage, cx: &'a App) -> (&'a [Value], u64) {
    let live = page.live.read(cx);
    let at = live
        .latest()
        .and_then(|l| l.get("at"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    (&live.history, at as u64)
}

pub(super) fn spark(values: Vec<f64>, hot: bool) -> Spark {
    Spark {
        values,
        second: Vec::new(),
        hot,
        fill: true,
    }
}

/// A caption over its chart, and a bar under it when there is one.
pub(super) fn shown(
    caption: String,
    chart: impl IntoElement,
    bar: Option<(String, &'static str, f64)>,
) -> impl IntoElement {
    v_flex()
        .w_full()
        .gap(Space::Xs)
        .child(Text::new(caption))
        .child(chart)
        .children(bar.map(|(id, name, v)| {
            Progress::new(SharedString::from(id), name)
                .value(v)
                .max(100.0)
        }))
}

/// Whether a row of the page's own shows.
pub fn shows(_: &str) -> bool {
    true
}

/// Home's quick links: the starred rows, else the pages most people open first.
pub fn quick(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let starred: Vec<(String, String, String)> = row::favorite_items(page.stars.settings.read(cx))
        .into_iter()
        .filter(|f| !f.hub.is_empty() && !f.label.is_empty())
        .take(5)
        .map(|f| {
            let title = if f.hub_title.is_empty() {
                crate::hubs::title(&f.hub).to_owned()
            } else {
                f.hub_title
            };
            (f.hub, f.label, title)
        })
        .collect();
    let links = if starred.is_empty() {
        ["appearance", "display", "network", "sound", "system"]
            .iter()
            .map(|h| {
                (
                    (*h).to_owned(),
                    crate::hubs::title(h).to_owned(),
                    String::new(),
                )
            })
            .filter(|(_, title, _)| !title.is_empty())
            .collect()
    } else {
        starred
    };
    links
        .into_iter()
        .enumerate()
        .map(|(i, (hub, label, says))| {
            let open = page.open.clone();
            let go = Button::new(SharedString::from(format!("quick-{i}")))
                .label("Open…")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| open(&hub, cx));
            row::setting(label, says, "", go, v_flex()).into_any_element()
        })
        .collect()
}

/// Home's hero line, and a placeholder while no sample has come.
pub fn hero(page: &CatalogPage, cx: &App) -> impl IntoElement + use<> {
    let (h, _) = history(page, cx);
    let line = Home {
        latest: h.last(),
        rows: h.len(),
    }
    .hero();
    v_flex()
        .gap(Space::Xs)
        .child(Heading::new(line).level(3))
        .children(h.is_empty().then(Skeleton::line))
}

/// A Home row's control: its caption and sparkline (`cpu`, `mem`, `net`, `temp`).
pub fn control(which: &str, id: String, page: &CatalogPage, cx: &App) -> impl IntoElement + use<> {
    let (h, generation) = history(page, cx);
    let home = Home {
        latest: h.last(),
        rows: h.len(),
    };
    let (name, caption, key): (&'static str, String, &str) = match which {
        "mem" => ("Memory in use", home.mem(), "mem"),
        "net" => ("Received per second", home.net(), "rxBps"),
        "temp" => ("CPU package temperature", home.temp(), "cpuTemp"),
        _ => ("CPU busy", home.cpu(), "cpu"),
    };
    let chart = charts::spark(
        id.clone(),
        name,
        caption.clone(),
        spark(stats::series(h, key), false),
        generation,
    );
    let mem = h
        .last()
        .and_then(|l| stats::finite(l.get("mem")))
        .filter(|_| which == "mem");
    shown(
        caption,
        chart,
        mem.map(|m| (format!("{id}-bar"), "Memory in use", m)),
    )
}

/// Home's GPUs: each with its temperature over the last minute.
pub fn gpus(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let (h, generation) = history(page, cx);
    let home = Home {
        latest: h.last(),
        rows: h.len(),
    };
    let inventory = match page.inventories.read(cx).known("hardware") {
        Known::Value(doc) => crate::hardware::normalize(Some(doc)),
        _ => crate::hardware::normalize(None),
    };
    stats::gpu_rows(inventory.get("gpus"), h.last())
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let caption = home.gpu(g);
            let chart = charts::spark(
                format!("gpu-{i}"),
                "GPU temperature",
                caption.clone(),
                spark(stats::gpu_series(h, g), false),
                generation,
            );
            row::stacked(
                monitor::gpu_name(g),
                monitor::gpu_says(g),
                "/sys/class/drm",
                shown(caption, chart, None),
                v_flex(),
            )
            .into_any_element()
        })
        .collect()
}

fn send(
    page: &mut CatalogPage,
    body: Option<Value>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let Some(body) = body else { return };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |s, cx| {
        s.send(
            &client,
            "process",
            &body,
            "The signal was not sent",
            window,
            cx,
        )
    });
}

/// `actOn(row, action)`: End and Force quit ask first; the copies copy.
fn act(
    page: &mut CatalogPage,
    r: &Value,
    action: &str,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    match action {
        "copy" | "copyCmd" => {
            let text = if action == "copy" {
                crate::js::string(&r["pid"])
            } else {
                r.get("cmdline")
                    .filter(|c| crate::js::truthy(Some(c)))
                    .map_or_else(|| crate::js::string(&r["comm"]), crate::js::string)
            };
            row::copy(&page.stars.client, &text, window, cx);
        }
        "term" | "kill" => {
            let kill = action == "kill";
            let (title, message, verb) = processes::signal_confirm(r, kill);
            let pid = r["pid"].clone();
            page.ask(
                Asked {
                    title: title.into(),
                    message,
                    action: verb,
                    go: Rc::new(move |p, window, cx| {
                        send(
                            p,
                            processes::signal_write(&pid, if kill { "KILL" } else { "TERM" }),
                            window,
                            cx,
                        )
                    }),
                },
                cx,
            );
        }
        _ => {}
    }
}

pub(super) fn process_row(
    i: usize,
    prefix: &'static str,
    r: &Value,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let end = processes::can_signal(r).then(|| {
        let (this, r) = (this.clone(), r.clone());
        Button::new(SharedString::from(format!("{prefix}-{i}-end")))
            .label("End")
            .variant(ButtonVariant::Outline)
            .on_click(move |_, window, cx| this.update(cx, |p, cx| act(p, &r, "term", window, cx)))
    });
    let (menu_this, menu_row, more) = (this.clone(), r.clone(), processes::overflow(r));
    let menu = DropdownMenu::new(SharedString::from(format!("{prefix}-{i}-more")), "More").menu(
        move |m| {
            more.iter().fold(m, |m, (id, label)| {
                let (this, r, id) = (menu_this.clone(), menu_row.clone(), *id);
                m.item(MenuItem::new(*label).on_select(move |window, cx| {
                    this.update(cx, |p, cx| act(p, &r, id, window, cx))
                }))
            })
        },
    );
    let hint = r
        .get("cmdline")
        .filter(|c| crate::js::truthy(Some(c)))
        .map_or_else(String::new, crate::js::string);
    // A process's row has no star: the QML row is no setting.
    row::setting(
        crate::js::string(&r["comm"]),
        processes::status(r),
        hint,
        h_flex().gap(Space::Sm).children(end).child(menu),
        v_flex(),
    )
}

/// This user's processes, searched, and on Home sorted by a chip; then "No matching processes"
/// while none is listed. The Monitor hub's are its eight busiest.
pub fn procs(
    home: bool,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let (h, _) = history(page, cx);
    let latest = h.last();
    let (key, prefix) = if home {
        (HOME_QUERY, "proc")
    } else {
        (MONITOR_QUERY, "hot")
    };
    let query = draft(page, key);
    let sort = match draft(page, HOME_SORT) {
        s if s.is_empty() || !home => "cpu".to_owned(),
        s => s,
    };
    let uid = latest.and_then(|l| stats::finite(l.get("uid")));
    let filter = Filter {
        query: &query,
        sort: &sort,
        uid,
        cap: if home { processes::ROW_CAP } else { 8 },
        ..Filter::default()
    };
    let rows = processes::list(
        latest.map_or(&[][..], |l| {
            l.get("processes")
                .and_then(Value::as_array)
                .map_or(&[][..], Vec::as_slice)
        }),
        &filter,
    );
    let typed = this.clone();
    let search = Input::new(
        SharedString::from(format!("{prefix}-search")),
        "Search processes",
    )
    .placeholder("Search processes…")
    .value(query.clone())
    .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, key, t, cx)));
    let chips = home.then(|| {
        h_flex()
            .gap(Space::Sm)
            .children(processes::SORTS.iter().map(|(id, label)| {
                let this = this.clone();
                Button::new(SharedString::from(format!("sort-{id}")))
                    .label(*label)
                    .variant(if sort == *id {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_click(move |_, _, cx| this.update(cx, |p, cx| put(p, HOME_SORT, *id, cx)))
            }))
    });
    let mut out = vec![
        v_flex()
            .gap(Space::Sm)
            .child(search)
            .children(chips)
            .into_any_element(),
    ];
    out.extend(
        rows.iter()
            .enumerate()
            .map(|(i, r)| process_row(i, prefix, r, this).into_any_element()),
    );
    if rows.is_empty() {
        let (icon, says) = match latest {
            Some(_) => (Icon::Search, "Nothing matches that search."),
            None => (Icon::Info, "Waiting for the first sample."),
        };
        let empty = Empty::new("No matching processes").icon(icon);
        out.push(empty.description(says).into_any_element());
    }
    out
}

/// The Monitor hub's Sampling: each interval, Pause or Resume, and what was taken.
pub fn sampling(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let live = page.live.read(cx);
    let caption = monitor::sampling(
        live.history.len(),
        &Captions {
            latest: live.latest(),
            waiting: live.history.is_empty(),
        }
        .psi(),
        &live.error,
    );
    let paused = live.paused;
    let chips = INTERVALS.iter().map(|(id, label, ms)| {
        let feed = page.live.clone();
        Button::new(SharedString::from(format!("interval-{id}")))
            .label(*label)
            .variant(if live.interval_ms == *ms {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            })
            .on_click(move |_, _, cx| {
                feed.update(cx, |l, cx| {
                    if l.set_interval(id) {
                        cx.notify();
                    }
                })
            })
    });
    let feed = page.live.clone();
    let pause = Button::new("sampling-pause")
        .label(if paused { "Resume" } else { "Pause" })
        .variant(if paused {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Outline
        })
        .on_click(move |_, _, cx| {
            feed.update(cx, |l, cx| {
                l.paused = !l.paused;
                cx.notify();
            })
        });
    let _ = this;
    vec![
        v_flex()
            .gap(Space::Sm)
            .child(h_flex().gap(Space::Sm).children(chips).child(pause))
            .child(Text::new(caption).muted())
            .into_any_element(),
    ]
}

/// The Monitor hub's meters: processor, memory, traffic, disk, package and load.
pub fn now(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let (h, generation) = history(page, cx);
    let latest = h.last();
    let c = Captions {
        latest,
        waiting: h.is_empty(),
    };
    let at = |k: &str| latest.and_then(|l| stats::finite(l.get(k)));
    let disks = latest
        .and_then(|l| l.get("disks"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let first = disks.first().map(|d| crate::js::string(&d["name"]));
    let (read, write) = match &first {
        Some(name) => (
            stats::disk_series(h, name, "readBps"),
            stats::disk_series(h, name, "writeBps"),
        ),
        None => (stats::series(h, "rxBps"), Vec::new()),
    };
    let meters: Vec<(&'static str, String, String, Spark, Option<f64>)> = vec![
        (
            "Processor",
            c.cpu(),
            c.cores(),
            spark(stats::series(h, "cpu"), monitor::hot(at("cpu"))),
            at("cpu"),
        ),
        (
            "Memory",
            c.mem(),
            c.available(),
            spark(stats::series(h, "mem"), monitor::hot(at("mem"))),
            at("mem"),
        ),
        (
            "Traffic",
            c.net(),
            c.tcp(),
            Spark {
                values: stats::series(h, "rxBps"),
                second: stats::series(h, "txBps"),
                hot: false,
                fill: false,
            },
            None,
        ),
        (
            "Disk I/O",
            c.disk(),
            String::new(),
            Spark {
                values: read,
                second: write,
                hot: false,
                fill: false,
            },
            None,
        ),
        (
            "Package",
            c.temp(),
            String::new(),
            spark(
                stats::series(h, "cpuTemp"),
                at("cpuTemp").is_some_and(|t| t >= 90.0),
            ),
            None,
        ),
        (
            "Load",
            c.load(),
            c.load_cores(),
            spark(stats::series(h, "load1"), false),
            None,
        ),
    ];
    let mut out = Vec::new();
    if h.is_empty() {
        out.push(Skeleton::line().into_any_element());
    }
    for (i, (label, value, caption, s, bar)) in meters.into_iter().enumerate() {
        let chart = charts::spark(format!("meter-{i}"), label, value.clone(), s, generation);
        let control = shown(
            value,
            chart,
            bar.map(|v| (format!("meter-{i}-bar"), label, v)),
        );
        out.push(row::stacked(label, caption, "", control, v_flex()).into_any_element());
    }
    out
}

/// The Monitor hub's Cores: a bar per core, and memory as one bar.
pub fn cores(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    let (h, generation) = history(page, cx);
    let latest = h.last();
    let c = Captions {
        latest,
        waiting: h.is_empty(),
    };
    vec![
        charts::core_bars(
            "core-bars".into(),
            c.cores(),
            stats::core_percents(latest),
            generation,
        )
        .into_any_element(),
        charts::memory(
            "memory-bar".into(),
            c.mem(),
            stats::mem_parts(latest),
            generation,
        )
        .into_any_element(),
    ]
}

/// The Monitor hub's Pages: each Monitor page, with Configure….
pub fn cards(page: &CatalogPage) -> Vec<impl IntoElement + use<>> {
    monitor::PAGES
        .iter()
        .map(|(id, title, says)| {
            let open = page.open.clone();
            let go = Button::new(SharedString::from(format!("monitor-page-{id}")))
                .label("Configure…")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| open(&format!("monitor/{id}"), cx));
            row::setting(*title, *says, "", go, v_flex()).into_any_element()
        })
        .collect()
}
