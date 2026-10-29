//! The Bar page's rows Atmos draws itself (`bar.rs` has what they say and send): the tray's
//! hidden and pinned icons, a switch per indicator, the agents' usage refresh, snapshot file and
//! device id, the clock's formats, week start, birth year and life expectancy, and the plugins.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::bar;
use crate::catalog::Field;
use crate::pages::controls::select;
use crate::pages::row;

fn key(page: &CatalogPage, name: &'static str, cx: &App) -> Option<Value> {
    Field::Key(name)
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn on(page: &CatalogPage, name: &'static str, cx: &App) -> bool {
    key(page, name, cx) == Some(Value::Bool(true))
}

fn text(page: &CatalogPage, name: &'static str, cx: &App) -> String {
    key(page, name, cx)
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, |v| crate::js::string(&v))
}

fn count(page: &CatalogPage, name: &'static str, cx: &App) -> usize {
    key(page, name, cx)
        .and_then(|v| v.as_array().map(Vec::len))
        .unwrap_or(0)
}

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
    page.stars.settings.update(cx, |s, cx| {
        s.send(&client, key, &body, "The bar was not changed", window, cx)
    });
}

/// What a row of the page's own says, when it is built.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    Some(match which {
        "hidden" => bar::hidden_says(key(page, "trayHidden", cx).as_ref()),
        "pinned" => bar::pinned_says(key(page, "trayPinned", cx).as_ref()),
        "birth" => bar::birth_says(
            key(page, "clockBirthYear", cx)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        )
        .into(),
        _ => return None,
    })
}

/// Whether a row of the page's own shows.
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    let tray = on(page, "trayPresent", cx);
    match which {
        "hiddenNone" => tray && count(page, "trayHidden", cx) == 0,
        "hidden" => tray && count(page, "trayHidden", cx) > 0,
        "pinnedNone" => tray && count(page, "trayPinned", cx) == 0,
        "pinned" => tray && count(page, "trayPinned", cx) > 0,
        "indicators" => on(page, "indicatorsPresent", cx),
        "refresh" | "syncdir" | "syncfile" | "device" => on(page, "agentsPresent", cx),
        "clock" | "clockAlt" | "week" | "birth" | "life" => on(page, "clockPresent", cx),
        "pluginsNone" => {
            key(page, "plugins", cx).is_some_and(|v| v.as_array().is_none_or(Vec::is_empty))
        }
        _ => true,
    }
}

/// Whether the row's control takes the row's width (`stretchControl`).
pub fn stacked(which: &str) -> bool {
    matches!(which, "indicators" | "refresh")
}

/// A line of text with Set, and Clear while it holds something.
/// A text the agents' sync takes: its placeholder, key, and write.
type Entry = (
    &'static str,
    &'static str,
    fn(&str, Option<&Value>) -> Option<Value>,
);

/// A year the clock takes: its key, its most, and its write.
type Year = (
    &'static str,
    f64,
    fn(&Value, Option<&Value>) -> Option<Value>,
);

fn field(
    id: String,
    label: &'static str,
    (placeholder, name, write): Entry,
    choose: Option<Button>,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let current = key(page, name, cx);
    let shown = text(page, name, cx);
    let draft_key = format!("bar-{name}");
    let draft = page
        .drafts
        .get(&draft_key)
        .cloned()
        .unwrap_or_else(|| shown.clone());
    let usable = on(page, "agentsPresent", cx);
    let (typed, entered, set, cleared) = (this.clone(), this.clone(), this.clone(), this.clone());
    let (k1, k2, c1, c2, c3) = (
        draft_key.clone(),
        draft_key,
        current.clone(),
        current.clone(),
        current,
    );
    let d2 = draft.clone();
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-field")), label)
                .placeholder(placeholder)
                .value(draft)
                .disabled(!usable)
                .on_change(move |t, _, cx| {
                    typed.update(cx, |p, cx| {
                        p.drafts.insert(k1.clone(), t.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |t, window, cx| {
                    let body = write(t, c1.as_ref());
                    entered.update(cx, |p, cx| send(p, name, body, window, cx))
                }),
        )
        .children(choose)
        .child(
            Button::new(SharedString::from(format!("{id}-set")))
                .label("Set")
                .variant(ButtonVariant::Outline)
                .disabled(!usable)
                .on_click(move |_, window, cx| {
                    set.update(cx, |p, cx| {
                        let typed = p.drafts.get(&k2).cloned().unwrap_or_else(|| d2.clone());
                        send(p, name, write(&typed, c2.as_ref()), window, cx)
                    })
                }),
        )
        .children((!shown.is_empty()).then(|| {
            Button::new(SharedString::from(format!("{id}-clear")))
                .label("Clear")
                .variant(ButtonVariant::Outline)
                .disabled(!usable)
                .on_click(move |_, window, cx| {
                    cleared.update(cx, |p, cx| {
                        send(p, name, write("", c3.as_ref()), window, cx)
                    })
                })
        }))
}

/// A number with Clear while it is set (Birth year, Life expectancy).
fn year(
    id: String,
    label: &'static str,
    (name, max, write): Year,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let Some(current) = key(page, name, cx) else {
        return row::reading(label).into_any_element();
    };
    let n = current.as_f64().unwrap_or(0.0);
    let usable = on(page, "clockPresent", cx);
    let (set, cleared) = (this.clone(), this.clone());
    let (c1, c2) = (current.clone(), current);
    h_flex()
        .gap(Space::Sm)
        .child(
            NumberInput::new(SharedString::from(id.clone()), label)
                .value(n)
                .min(0.0)
                .max(max)
                .disabled(!usable)
                .on_commit(move |v, window, cx| {
                    let body = write(&crate::js::json_number(v), Some(&c1));
                    set.update(cx, |p, cx| send(p, name, body, window, cx))
                }),
        )
        .children((n > 0.0).then(|| {
            Button::new(SharedString::from(format!("{id}-clear")))
                .label("Clear")
                .variant(ButtonVariant::Outline)
                .disabled(!usable)
                .on_click(move |_, window, cx| {
                    let body = write(&Value::from(""), Some(&c2));
                    cleared.update(cx, |p, cx| send(p, name, body, window, cx))
                })
        }))
        .into_any_element()
}

fn birth(v: &Value, current: Option<&Value>) -> Option<Value> {
    bar::birth_write(v, current, bar::this_year())
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
        "hidden" | "pinned" => {
            let (name, button) = if which == "hidden" {
                ("trayHidden", "Show all")
            } else {
                ("trayPinned", "Unpin all")
            };
            let current = key(page, name, cx);
            Button::new(SharedString::from(id))
                .label(button)
                .variant(ButtonVariant::Outline)
                .disabled(!on(page, "trayPresent", cx))
                .on_click(move |_, window, cx| {
                    let body = bar::clear_tray(name, current.as_ref());
                    this.update(cx, |p, cx| send(p, name, body, window, cx))
                })
                .into_any_element()
        }
        "indicators" => {
            let items = key(page, "indicatorsItems", cx);
            let usable = on(page, "indicatorsPresent", cx);
            v_flex()
                .gap(Space::Xs)
                .children(bar::INDICATORS.iter().map(|(ind, name)| {
                    let (this, items) = (this.clone(), items.clone());
                    let switch = Switch::new(SharedString::from(format!("{id}-{ind}")), *name)
                        .checked(bar::indicator_shown(items.as_ref(), ind))
                        .disabled(!usable)
                        .on_change(move |_, window, cx| {
                            let body = bar::toggle_indicator(items.as_ref(), ind);
                            this.update(cx, |p, cx| send(p, "indicatorsItems", body, window, cx))
                        });
                    h_flex()
                        .gap(Space::Sm)
                        .child(switch)
                        .child(Text::new(*name))
                }))
                .into_any_element()
        }
        "refresh" => {
            let Some(seconds) = key(page, "agentsRefreshIntervalSec", cx) else {
                return row::reading(label).into_any_element();
            };
            NumberInput::new(SharedString::from(id), label)
                .value(bar::refresh_minutes(Some(&seconds)))
                .min(5.0)
                .max(60.0)
                .step(5.0)
                .disabled(!on(page, "agentsPresent", cx))
                .on_commit(move |m, window, cx| {
                    let body = bar::refresh_write(m, Some(&seconds));
                    this.update(cx, |p, cx| {
                        send(p, "agentsRefreshIntervalSec", body, window, cx)
                    })
                })
                .into_any_element()
        }
        "syncdir" => {
            let usable = on(page, "agentsPresent", cx);
            let current = key(page, "agentsSyncDir", cx);
            let chosen = this.clone();
            let choose = Button::new(SharedString::from(format!("{id}-choose")))
                .label("Choose…")
                .variant(ButtonVariant::Outline)
                .disabled(!usable)
                .on_click(move |_, window, cx| {
                    let current = current.clone();
                    let chooser =
                        FileChooser::folder("Agent usage sync folder").start_in(StartFolder::Home);
                    crate::pages::files::choose(
                        chooser,
                        &chosen,
                        window,
                        cx,
                        Box::new(move |p, path, window, cx| {
                            send(
                                p,
                                "agentsSyncDir",
                                bar::sync_dir_write(&path, current.as_ref()),
                                window,
                                cx,
                            )
                        }),
                    );
                });
            field(
                id,
                label,
                ("~/Sync/agent-usage", "agentsSyncDir", bar::sync_dir_write),
                Some(choose),
                page,
                &this,
                cx,
            )
            .into_any_element()
        }
        "syncfile" => field(
            id,
            label,
            ("laptop.json", "agentsSyncFileName", bar::sync_file_write),
            None,
            page,
            &this,
            cx,
        )
        .into_any_element(),
        "device" => field(
            id,
            label,
            ("laptop", "agentsSyncDeviceId", bar::device_write),
            None,
            page,
            &this,
            cx,
        )
        .into_any_element(),
        "clock" | "clockAlt" | "week" => {
            let name = match which {
                "clock" => "clockFormat",
                "clockAlt" => "clockFormatAlt",
                _ => "clockWeekStart",
            };
            let current = text(page, name, cx);
            let options = if which == "week" {
                bar::week_options(&current)
            } else {
                bar::clock_options(&text(page, "barPosition", cx), &current)
            };
            let shown = key(page, name, cx);
            let now = Value::from(current);
            select(
                id,
                label,
                &options,
                &now,
                !on(page, "clockPresent", cx),
                move |v, window, cx| {
                    let body = if which == "week" {
                        bar::week_write(v, shown.as_ref())
                    } else {
                        (shown.as_ref() != Some(&Value::from(v)))
                            .then(|| crate::settings::write(name, &Value::from(v)))
                    };
                    this.update(cx, |p, cx| send(p, name, body, window, cx))
                },
            )
            .into_any_element()
        }
        "birth" => year(
            id,
            label,
            ("clockBirthYear", 2100.0, birth),
            page,
            &this,
            cx,
        )
        .into_any_element(),
        "life" => year(
            id,
            label,
            ("clockLifeExpectancy", 150.0, bar::life_write),
            page,
            &this,
            cx,
        )
        .into_any_element(),
        "pluginsNone" => {
            let client = page.stars.client.clone();
            Button::new(SharedString::from(id))
                .label("Refresh")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, _| {
                    drop(client.request(&crate::protocol::snapshot_invalidate(
                        crate::settings::ALL_TOPICS,
                    )))
                })
                .into_any_element()
        }
        _ => v_flex().into_any_element(),
    }
}

/// The plugins, each with its switch, or On for one that cannot be turned off.
pub fn plugins(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = key(page, "plugins", cx);
    list.as_ref()
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, p)| {
            let (label, says, can) = bar::plugin_row(p);
            let pid = p
                .get("id")
                .filter(|v| crate::js::truthy(Some(v)))
                .map_or_else(String::new, crate::js::string);
            let enabled = crate::js::truthy(p.get("enabled"));
            let (this, all) = (this.clone(), list.clone());
            let control = if can {
                Switch::new(SharedString::from(format!("plugin-{i}")), label.clone())
                    .checked(enabled)
                    .disabled(pid.is_empty())
                    .on_change(move |_, window, cx| {
                        let body = bar::plugin_write(&pid, !enabled, all.as_ref());
                        this.update(cx, |p, cx| send(p, "plugins", body, window, cx))
                    })
                    .into_any_element()
            } else {
                Text::new("On").into_any_element()
            };
            let star = page.stars.star(page.page().id, &label, says, cx);
            row::setting(label, says, "omarchy plugin enable", control, star).into_any_element()
        })
        .collect()
}
