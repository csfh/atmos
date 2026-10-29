//! The Monitor pages' rows and lists (`meters.rs` has what they say and list): each row's value
//! and sparkline, the cores, the CPU histogram, the memory bar, the disks, interfaces, GPUs and
//! sensors with their filters, and each page's processes. A group whose chart is not drawn yet
//! keeps the catalog's placeholder.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use super::charts::{self, Spark};
use super::live::{draft, history, process_row, put, shown, spark};
use crate::meters::{self, Read};
use crate::pages::row;
use crate::settings::Known;
use crate::stats;

fn read(h: &[Value]) -> Read<'_> {
    Read {
        history: h,
        waiting: h.is_empty(),
    }
}

fn chart(
    id: String,
    name: &'static str,
    caption: String,
    s: Option<meters::Series>,
    generation: u64,
) -> impl IntoElement {
    let chart = s.map(|(values, second, hot)| {
        let fill = second.is_empty();
        charts::spark(
            id,
            name,
            caption.clone(),
            Spark {
                values,
                second,
                hot,
                fill,
            },
            generation,
        )
    });
    v_flex()
        .w_full()
        .gap(Space::Xs)
        .child(Text::new(caption))
        .children(chart)
}

/// A row's value and sparkline, by row.
pub fn control(which: &str, id: String, page: &CatalogPage, cx: &App) -> impl IntoElement + use<> {
    let (h, generation) = history(page, cx);
    let (caption, series) = read(h).meter(which);
    chart(id, "The last minute", caption, series, generation)
}

pub(super) fn latest_list(h: &[Value], key: &str) -> Vec<Value> {
    h.last()
        .and_then(|l| l.get(key))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn gpus(page: &CatalogPage, h: &[Value], cx: &App) -> Vec<Value> {
    let inventory = match page.inventories.read(cx).known("hardware") {
        Known::Value(doc) => crate::hardware::normalize(Some(doc)),
        _ => crate::hardware::normalize(None),
    };
    stats::gpu_rows(inventory.get("gpus"), h.last())
}

fn listed(which: &str, page: &CatalogPage, cx: &App) -> usize {
    let (h, _) = history(page, cx);
    let procs = |p: &str| {
        let latest = h.last();
        let uid = latest.and_then(|l| stats::finite(l.get("uid")));
        crate::processes::list(
            &latest_list(h, "processes"),
            &crate::processes::Filter {
                uid,
                ..meters::procs(p)
            },
        )
        .len()
    };
    match which {
        "nohot" => procs("cpu"),
        "nolarge" => procs("memory"),
        "noio" => procs("io"),
        "nodisks" => meters::rates(
            &latest_list(h, "disks"),
            &draft(page, "disk-filter"),
            "io",
            [("read", "readBps"), ("write", "writeBps")],
        )
        .len(),
        "noifaces" => meters::rates(
            &latest_list(h, "ifaces"),
            &draft(page, "iface-filter"),
            "io",
            [("rx", "rxBps"), ("tx", "txBps")],
        )
        .len(),
        "nogpus" => gpus(page, h, cx).len(),
        "nosensors" => meters::sensors(
            &latest_list(h, "sensors"),
            &draft(page, "sensor-filter"),
            &kind(page),
        )
        .len(),
        _ => 1,
    }
}

/// Whether a row shows: an empty one while its list is.
pub fn shows(which: &str, page: &CatalogPage, cx: &App) -> bool {
    !which.starts_with("no") || listed(which, page, cx) == 0
}

/// An empty row's words, for the ones that say whether a sample has come.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    matches!(which, "nodisks" | "noifaces" | "nosensors").then(|| {
        let (h, _) = history(page, cx);
        if h.is_empty() {
            "Waiting for the first sample."
        } else {
            "Nothing matches that filter."
        }
        .into()
    })
}

fn kind(page: &CatalogPage) -> String {
    match draft(page, "sensor-kind") {
        k if k.is_empty() => "all".into(),
        k => k,
    }
}

/// A filter field and its chips: `(draft key, placeholder)`, `(chip draft key, chips, current)`.
fn filters(
    (key, placeholder): (&'static str, &'static str),
    (chip, chips, now): (
        &'static str,
        &'static [(&'static str, &'static str)],
        String,
    ),
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let typed = this.clone();
    let field = Input::new(SharedString::from(format!("{key}-field")), placeholder)
        .placeholder(placeholder)
        .value(draft(page, key))
        .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, key, t, cx)));
    let buttons = chips.iter().map(|(id, label)| {
        let this = this.clone();
        Button::new(SharedString::from(format!("{chip}-{id}")))
            .label(*label)
            .variant(if now == *id {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            })
            .on_click(move |_, _, cx| this.update(cx, |p, cx| put(p, chip, *id, cx)))
    });
    v_flex()
        .gap(Space::Sm)
        .child(field)
        .child(h_flex().gap(Space::Sm).children(buttons))
}

/// A rate list's filter draft, sort draft, list key, sort keys, and chips.
type Rates = (
    &'static str,
    &'static str,
    &'static str,
    [(&'static str, &'static str); 2],
    &'static [(&'static str, &'static str)],
);

fn sorted(page: &CatalogPage, key: &str, or: &str) -> String {
    match draft(page, key) {
        s if s.is_empty() => or.into(),
        s => s,
    }
}

/// A Monitor page's list, by name.
pub fn list(
    which: &str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let (h, generation) = history(page, cx);
    let r = read(h);
    let latest = h.last();
    let mut out = Vec::new();
    let procs = |p: &'static str, prefix: &'static str, out: &mut Vec<_>| {
        let uid = latest.and_then(|l| stats::finite(l.get("uid")));
        let rows = crate::processes::list(
            &latest_list(h, "processes"),
            &crate::processes::Filter {
                uid,
                ..meters::procs(p)
            },
        );
        out.extend(
            rows.iter()
                .enumerate()
                .map(|(i, p)| process_row(i, prefix, p, this).into_any_element()),
        );
    };
    match which {
        "corerows" => {
            let cores = stats::core_percents(latest);
            out.push(
                charts::core_bars(
                    "cpu-core-bars".into(),
                    format!("{} cores", cores.len()),
                    cores,
                    generation,
                )
                .into_any_element(),
            );
            for (i, c) in latest_list(h, "cpus").iter().enumerate() {
                let id = c
                    .get("id")
                    .filter(|v| !v.is_null())
                    .cloned()
                    .unwrap_or(Value::from(i));
                let (pct, hz) = (
                    stats::format_percent(stats::finite(c.get("cpu"))),
                    stats::format_mhz(stats::finite(c.get("freqMhz"))),
                );
                let caption = match (pct.is_empty(), hz.is_empty()) {
                    (false, false) => format!("{pct}  ·  {hz}"),
                    (false, true) => pct.clone(),
                    (true, false) => hz,
                    _ => "unknown".into(),
                };
                let governor = c
                    .get("governor")
                    .filter(|g| crate::js::truthy(Some(g)))
                    .map_or_else(|| "governor unknown".into(), crate::js::string);
                let hot = crate::monitor::hot(stats::finite(c.get("cpu")));
                let spark = charts::spark(
                    format!("core-{i}"),
                    "Core busy",
                    caption.clone(),
                    spark(stats::core_series(h, &id), hot),
                    generation,
                );
                out.push(
                    row::stacked(
                        format!("CPU {}", crate::js::string(&id)),
                        governor,
                        "/sys/devices/system/cpu",
                        shown(caption, spark, None),
                        v_flex(),
                    )
                    .into_any_element(),
                );
            }
        }
        "table" => out.extend(
            table(page, this, cx)
                .into_iter()
                .map(|r| r.into_any_element()),
        ),
        "parallel" => out.push(parallel(page, cx).into_any_element()),
        "cpuhist" | "stream" | "calendar" | "waterfall" | "bees" | "radar" => {
            out.push(super::plots::plot(which, h, generation).into_any_element())
        }
        "horizon" | "treemap" | "sankey" | "sunburst" | "icicle" => {
            out.push(super::memory_plots::plot(which, page, this, h, generation).into_any_element())
        }
        "heatmap" | "ridgeline" | "violin" | "rose" => {
            out.push(super::cpu_plots::plot(which, h, generation).into_any_element())
        }
        "cpuhot" => procs("cpu", "cpuhot", &mut out),
        "fat" => procs("memory", "fat", &mut out),
        "io" => procs("io", "io", &mut out),
        "memmix" => {
            out.push(
                charts::memory(
                    "memory-mix".into(),
                    stats::format_percent(r_at(latest, "mem")),
                    stats::mem_parts(latest),
                    generation,
                )
                .into_any_element(),
            );
            out.push(
                charts::spark(
                    "memory-spark".into(),
                    "Memory in use",
                    stats::format_percent(r_at(latest, "mem")),
                    spark(
                        stats::series(h, "mem"),
                        crate::monitor::hot(r_at(latest, "mem")),
                    ),
                    generation,
                )
                .into_any_element(),
            );
        }
        "disks" | "ifaces" => {
            let disks = which == "disks";
            let (filter, chip, list_key, keys, chips): Rates = if disks {
                (
                    "disk-filter",
                    "disk-sort",
                    "disks",
                    [("read", "readBps"), ("write", "writeBps")],
                    &[
                        ("io", "I/O"),
                        ("read", "Read"),
                        ("write", "Write"),
                        ("name", "Name"),
                    ],
                )
            } else {
                (
                    "iface-filter",
                    "iface-sort",
                    "ifaces",
                    [("rx", "rxBps"), ("tx", "txBps")],
                    &[
                        ("io", "Total"),
                        ("rx", "Receive"),
                        ("tx", "Transmit"),
                        ("name", "Name"),
                    ],
                )
            };
            let sort = sorted(page, chip, "io");
            let placeholder = if disks {
                "Filter disks…"
            } else {
                "Filter interfaces…"
            };
            out.push(
                filters(
                    (filter, placeholder),
                    (chip, chips, sort.clone()),
                    page,
                    this,
                )
                .into_any_element(),
            );
            for (i, d) in
                meters::rates(&latest_list(h, list_key), &draft(page, filter), &sort, keys)
                    .iter()
                    .enumerate()
            {
                let name = d
                    .get("name")
                    .filter(|n| crate::js::truthy(Some(n)))
                    .map_or_else(
                        || if disks { "disk".into() } else { "iface".into() },
                        crate::js::string,
                    );
                let line = r.pair_line(d, keys[0].1, keys[1].1);
                let (a, b) = if disks {
                    (
                        stats::disk_series(h, &name, "readBps"),
                        stats::disk_series(h, &name, "writeBps"),
                    )
                } else {
                    (
                        stats::iface_series(h, &name, "rxBps"),
                        stats::iface_series(h, &name, "txBps"),
                    )
                };
                let spark = charts::spark(
                    format!("{list_key}-{i}"),
                    "Rates",
                    line.clone(),
                    Spark {
                        values: a,
                        second: b,
                        hot: false,
                        fill: false,
                    },
                    generation,
                );
                let hint = if disks {
                    "/proc/diskstats"
                } else {
                    "/proc/net/dev"
                };
                out.push(
                    row::stacked(name, line.clone(), hint, shown(line, spark, None), v_flex())
                        .into_any_element(),
                );
            }
        }
        "sgpus" => {
            for (i, g) in gpus(page, h, cx).iter().enumerate() {
                let line = r.gpu_line(g);
                let spark = charts::spark(
                    format!("sensor-gpu-{i}"),
                    "GPU busy and temperature",
                    line.clone(),
                    Spark {
                        values: stats::gpu_busy_series(h, g),
                        second: stats::gpu_series(h, g),
                        hot: false,
                        fill: false,
                    },
                    generation,
                );
                out.push(
                    row::stacked(
                        crate::monitor::gpu_name(g),
                        r.gpu_says(g),
                        "/sys/class/drm",
                        shown(line, spark, None),
                        v_flex(),
                    )
                    .into_any_element(),
                );
            }
        }
        _ => {
            let kind = kind(page);
            out.push(
                filters(
                    ("sensor-filter", "Filter sensors…"),
                    (
                        "sensor-kind",
                        &[("all", "All"), ("temp", "Temp"), ("fan", "Fan")],
                        kind.clone(),
                    ),
                    page,
                    this,
                )
                .into_any_element(),
            );
            for (i, s) in meters::sensors(
                &latest_list(h, "sensors"),
                &draft(page, "sensor-filter"),
                &kind,
            )
            .iter()
            .enumerate()
            {
                let line = r.sensor_line(s);
                let id = crate::js::string(s.get("id").unwrap_or(&Value::Null));
                let hot = s.get("kind").and_then(Value::as_str) == Some("temp")
                    && stats::finite(s.get("value")).is_some_and(|v| v >= 90.0);
                let spark = charts::spark(
                    format!("sensor-{i}"),
                    "Sensor",
                    line.clone(),
                    spark(stats::sensor_series(h, &id), hot),
                    generation,
                );
                let label = s
                    .get("label")
                    .filter(|l| crate::js::truthy(Some(l)))
                    .map_or_else(|| "sensor".into(), crate::js::string);
                let chip = s
                    .get("chip")
                    .filter(|c| crate::js::truthy(Some(c)))
                    .map_or_else(String::new, crate::js::string);
                out.push(
                    row::stacked(
                        label,
                        chip,
                        "/sys/class/hwmon",
                        shown(line, spark, None),
                        v_flex(),
                    )
                    .into_any_element(),
                );
            }
        }
    }
    out
}

fn r_at(latest: Option<&Value>, k: &str) -> Option<f64> {
    latest.and_then(|l| stats::finite(l.get(k)))
}

/// The Processes page's filter, as its chips and search hold it.
fn table_filter<'a>(
    page: &CatalogPage,
    query: &'a str,
    sort: &'a str,
    scope: &'a str,
    state: &'a str,
    uid: Option<f64>,
) -> crate::processes::Filter<'a> {
    let num = |k: &str, or: f64| draft(page, k).parse().unwrap_or(or);
    crate::processes::Filter {
        query,
        sort,
        scope,
        state,
        uid,
        min_cpu: num("ps-mincpu", 0.0),
        min_rss_kb: num("ps-minrss", 0.0),
        cap: num("ps-cap", 80.0) as usize,
        reverse: draft(page, "ps-reverse") == "1",
        tree: draft(page, "ps-tree") == "1",
    }
}

/// The Processes page's rows as its filter lists them.
fn table_rows(page: &CatalogPage, h: &[Value]) -> Vec<Value> {
    let (query, sort, scope, state) = (
        draft(page, "ps-query"),
        sorted(page, "ps-sort", "cpu"),
        sorted(page, "ps-scope", "mine"),
        sorted(page, "ps-state", "all"),
    );
    let uid = h.last().and_then(|l| stats::finite(l.get("uid")));
    crate::processes::list(
        &latest_list(h, "processes"),
        &table_filter(page, &query, &sort, &scope, &state, uid),
    )
}

/// A row of chips that set `key` to their id; the one it holds is primary.
fn chips(
    key: &'static str,
    items: Vec<(&'static str, String)>,
    now: &str,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    h_flex()
        .gap(Space::Sm)
        .children(items.into_iter().map(|(id, label)| {
            let this = this.clone();
            Button::new(SharedString::from(format!("{key}-{id}")))
                .label(label)
                .variant(if now == id {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                })
                .on_click(move |_, _, cx| this.update(cx, |p, cx| put(p, key, id, cx)))
        }))
}

/// The Processes page: the count, search, every filter's chips, each process, and "No matching
/// processes" while none is listed.
pub fn table(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    use crate::processes::{CAPS, MIN_CPU, MIN_RSS, SCOPES, SORTS, STATES};
    let (h, _) = history(page, cx);
    let rows = table_rows(page, h);
    let paused = if page.live.read(cx).paused {
        "  ·  paused"
    } else {
        ""
    };
    let count = format!(
        "{} shown  ·  {} sampled{paused}",
        rows.len(),
        latest_list(h, "processes").len()
    );
    let pairs = |l: &'static [(&'static str, &'static str)]| {
        l.iter()
            .map(|(i, t)| (*i, (*t).to_owned()))
            .collect::<Vec<_>>()
    };
    let values = |l: &'static [(&'static str, &'static str, f64)], tail: &str| {
        l.iter()
            .map(|(i, t, _)| (*i, format!("{t}{tail}")))
            .collect::<Vec<_>>()
    };
    let flip = |key: &'static str, on: &'static str, off: &'static str| {
        let this = this.clone();
        let set = draft(page, key) == "1";
        Button::new(SharedString::from(key.to_owned()))
            .label(if set { on } else { off })
            .variant(if set && key == "ps-tree" {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            })
            .on_click(move |_, _, cx| {
                this.update(cx, |p, cx| put(p, key, if set { "" } else { "1" }, cx))
            })
    };
    let typed = this.clone();
    let search = Input::new("ps-search", "Search processes")
        .placeholder("Search name, command, pid, state…")
        .value(draft(page, "ps-query"))
        .on_change(move |t, _, cx| typed.update(cx, |p, cx| put(p, "ps-query", t, cx)));
    let mut out = vec![
        v_flex()
            .gap(Space::Sm)
            .child(Text::new(count).muted())
            .child(search)
            .child(chips(
                "ps-scope",
                pairs(SCOPES),
                &sorted(page, "ps-scope", "mine"),
                this,
            ))
            .child(chips(
                "ps-state",
                pairs(STATES),
                &sorted(page, "ps-state", "all"),
                this,
            ))
            .child(
                h_flex()
                    .gap(Space::Sm)
                    .child(chips(
                        "ps-sort",
                        pairs(SORTS),
                        &sorted(page, "ps-sort", "cpu"),
                        this,
                    ))
                    .child(flip("ps-reverse", "Low first", "High first"))
                    .child(flip("ps-tree", "Tree", "Flat")),
            )
            .child(chips(
                "ps-mincpu",
                values(MIN_CPU, ""),
                &sorted(page, "ps-mincpu", "0"),
                this,
            ))
            .child(chips(
                "ps-minrss",
                values(MIN_RSS, ""),
                &sorted(page, "ps-minrss", "0"),
                this,
            ))
            .child(chips(
                "ps-cap",
                values(CAPS, " rows"),
                &sorted(page, "ps-cap", "80"),
                this,
            ))
            .into_any_element(),
    ];
    out.extend(
        rows.iter()
            .enumerate()
            .map(|(i, r)| process_row(i, "ps", r, this).into_any_element()),
    );
    if rows.is_empty() {
        let says = if h.is_empty() {
            "Waiting for the first sample."
        } else {
            "Nothing matches that search or filter."
        };
        out.push(
            row::setting("No matching processes", says, "", v_flex(), v_flex()).into_any_element(),
        );
    }
    out
}

/// The Processes page's parallel coordinates, of the processes it lists.
pub fn parallel(page: &CatalogPage, cx: &App) -> impl IntoElement + use<> {
    let (h, generation) = history(page, cx);
    let rows = crate::layouts::process_parallel(&table_rows(page, h));
    let chart = Chart::new("process-parallel", "Process parallel coordinates")
        .description("Process parallel coordinates")
        .transition(generation)
        .paint(move |scene, _| {
            let (w, hh) = (scene.width() as f64, scene.height() as f64);
            for (label, x, top, align) in
                crate::layouts::parallel_rails(crate::layouts::PARALLEL_AXES, w, hh)
            {
                scene.stroke(
                    &ChartPath::new()
                        .move_to(x as f32, top as f32)
                        .line_to(x as f32, hh as f32),
                    Ink::Axis,
                    StrokeWidth::Thin,
                );
                let anchor = match align {
                    "left" => TextAnchor::Start,
                    "right" => TextAnchor::End,
                    _ => TextAnchor::Middle,
                };
                scene.text(
                    x as f32,
                    0.0,
                    label,
                    Ink::TextMuted,
                    anchor,
                    TextBaseline::Top,
                    false,
                );
            }
            for (i, (_, pts)) in
                crate::layouts::parallel_lines(&rows, crate::layouts::PARALLEL_AXES.len(), w, hh)
                    .iter()
                    .enumerate()
            {
                if let Some(((x, y), rest)) = pts.split_first() {
                    let path = rest.iter().fold(
                        ChartPath::new().move_to(*x as f32, *y as f32),
                        |p, (x, y)| p.line_to(*x as f32, *y as f32),
                    );
                    scene.stroke(&path, Ink::Series(i), StrokeWidth::Thin);
                }
            }
        });
    v_flex().w_full().h(Space::Xl).child(chart)
}
