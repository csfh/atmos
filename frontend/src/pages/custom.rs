//! The rows a page module draws itself (`catalog/custom.rs`): their description, when it is
//! built from what the window holds, and their control. Writes go through the page's `call` and
//! `press`, so a setter runs as any other row's does, and a confirm is the page's one dialog.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::catalog::Field;
use crate::catalog::Shown;
use crate::catalog::custom::Custom;
use crate::catalog::setter::Btn;
use crate::pages::{
    agents, appearance, capture, environment, hardware, row, rules, software, startup, workspaces,
};
use crate::settings::Known;
use crate::{kernel, notifications};

/// The row's description, when it is built from what the window holds; `None` keeps the
/// catalog's.
pub fn description(c: Custom, page: &CatalogPage, cx: &App) -> Option<String> {
    if !c.speaks() {
        return None;
    }
    let settings = page.stars.settings.read(cx);
    match c {
        Custom::RunningKernel => match page.inventories.read(cx).known("diagnostics") {
            Known::Value(doc) => {
                let summary = kernel::kernel_summary(Some(doc));
                Some(if summary.is_empty() {
                    kernel::UNREAD.to_owned()
                } else {
                    summary
                })
            }
            Known::Unknown(_) => Some(kernel::UNREAD.to_owned()),
            Known::Waiting => None,
        },
        Custom::Swappiness => Field::Key("tweaks").read(settings).value().map(|t| {
            if kernel::swappiness(Some(t)) {
                kernel::SWAPPINESS_ON
            } else {
                kernel::SWAPPINESS_OFF
            }
            .to_owned()
        }),
        Custom::Detected("path") => environment::shows("path", page, cx),
        Custom::AddVar => Some(environment::add_description(page)),
        Custom::AddScratch => Some(workspaces::scratch_says(page)),
        Custom::ExtraThemes => Some(appearance::extras_says(page, cx)),
        Custom::AddTheme => Some(appearance::add_says(page, cx)),
        Custom::NightSchedule => Some(appearance::schedule_says(page, cx).to_owned()),
        Custom::UseSchedule => Some(appearance::use_schedule_says(page, cx).to_owned()),
        Custom::SaveFolder(key) => Some(capture::folder_says(page, key, cx)),
        Custom::Hw(which) => hardware::says(which, page, cx),
        Custom::Speed(which) => crate::pages::speedtest::says(which, page, cx),
        Custom::Hist(which) => crate::pages::history::says(which, page, cx),
        Custom::Disk(which) => crate::pages::disks::says(which, page, cx),
        Custom::Kb(which) => crate::pages::keyboard::says(which, page, cx),
        Custom::Bind(which) => crate::pages::bindings::says(which, page, cx),
        Custom::Meter(which) => crate::pages::meters::says(which, page, cx),
        Custom::Net(which) => crate::pages::network::says(which, page, cx),
        Custom::Wifi(which) => crate::pages::wifi::says(which, page, cx),
        Custom::Bt(which) => crate::pages::bluetooth::says(which, page, cx),
        Custom::Export(which) => crate::pages::export::says(which, page, cx),
        Custom::Acct(which) => crate::pages::accounts::says(which, page, cx),
        Custom::Logo(_) => Some(
            crate::system::screensaver_says(
                Field::Key("screensaverBranded").read(settings).value() == Some(&Value::Bool(true)),
            )
            .to_owned(),
        ),
        Custom::Display("backlight") => Some(crate::pages::display::backlight_says(page, cx)),
        Custom::Display(_) => None,
        Custom::Diag(which) => crate::pages::diagnostics::says(which, page, cx),
        Custom::Bar(which) => crate::pages::bar::says(which, page, cx),
        Custom::Sys(which) => crate::pages::system::says(which, page, cx),
        _ => None,
    }
}

/// Whether the row's control takes the row's width, under its words (`stretchControl`).
pub fn stacked(c: Custom) -> bool {
    if let Custom::Bar(which) = c {
        return crate::pages::bar::stacked(which);
    }
    if let Custom::Sys(which) = c {
        return crate::pages::system::stacked(which);
    }
    if let Custom::Export(which) = c {
        return crate::pages::export::stacked(which);
    }
    matches!(
        c,
        Custom::PathPrepend
            | Custom::Detected("path")
            | Custom::BarCount
            | Custom::Themes
            | Custom::Kb("layouts" | "variants")
            | Custom::Bind("filter")
            | Custom::Net("servers")
            | Custom::Acct("fullname")
            | Custom::Wifi("qr")
            | Custom::Live(_)
            | Custom::Meter("busy" | "load" | "swap" | "bandwidth" | "temp")
    )
}

/// Whether a group whose rows are all hidden is drawn (`catalog::SHOWN`); one it does not name
/// always is.
pub fn group_shows(page: &CatalogPage, title: &str, cx: &App) -> bool {
    let id = page.page().id;
    match crate::catalog::shown(id, title) {
        None => true,
        Some(Shown::When(w)) => w.holds(page.stars.settings.read(cx)),
        Some(Shown::Page(name)) => match (id, name) {
            ("display", _) => crate::pages::display::shows(page, cx),
            ("disks", _) => crate::pages::disks::shows(name, page, cx),
            ("hardware", _) => crate::pages::hardware::memory_shows(page, cx),
            ("applications/startup", _) => crate::pages::startup::failures_show(page, cx),
            (_, "disk") => crate::pages::diagnostics::shown("root", page, cx),
            _ => crate::pages::diagnostics::shown("ram", page, cx),
        },
    }
}

/// Whether the row shows, for one that shows by what its page read (`visible:`).
pub fn shown(c: Custom, page: &CatalogPage, cx: &App) -> bool {
    match c {
        Custom::McpNone => agents::none_installed(page),
        Custom::NoFailures => startup::none_failed(page, cx),
        Custom::NoRules => rules::none(page, cx),
        Custom::RuleCenter | Custom::InList => false,
        Custom::Themes => appearance::has_themes(page, cx),
        Custom::NoExtraThemes => appearance::no_extras(page, cx) == Some(true),
        Custom::ExtraThemes => appearance::no_extras(page, cx) == Some(false),
        Custom::CapturePick("size") => capture::flag(page, "recWebcam"),
        Custom::ResizeWebcam => capture::overlay(page, cx),
        Custom::Hw(which) => hardware::shown(which, page, cx),
        Custom::DockerDb => software::has_docker(page, cx),
        Custom::AppNoAutostart => crate::pages::launchers::no_autostart(page, cx),
        Custom::NoLaunchers(kind) => crate::pages::launchers::none(kind, page, cx),
        Custom::Hist(which) => crate::pages::history::shown(which, page, cx),
        Custom::Disk(which) => crate::pages::disks::shown(which, page, cx),
        Custom::Display(which) => crate::pages::display::shown(which, page, cx),
        Custom::Kb(_) | Custom::Logo(_) => true,
        Custom::Bind(which) => crate::pages::bindings::shown(which, page, cx),
        Custom::Net(which) => crate::pages::network::shown(which, page, cx),
        Custom::Wifi(which) => crate::pages::wifi::shown(which, page, cx),
        Custom::Bt(which) => crate::pages::bluetooth::shown(which, page, cx),
        Custom::Export(which) => crate::pages::export::shown(which, page, cx),
        Custom::Acct(which) => crate::pages::accounts::shown(which, page, cx),
        Custom::Live(which) => crate::pages::live::shows(which),
        Custom::Meter(which) => crate::pages::meters::shows(which, page, cx),
        Custom::Diag(which) => crate::pages::diagnostics::shown(which, page, cx),
        Custom::Bar(which) => crate::pages::bar::shown(which, page, cx),
        Custom::Sys(which) => crate::pages::system::shown(which, page, cx),
        Custom::Speed(which) => crate::pages::speedtest::shown(which, page),
        _ => true,
    }
}

/// The row's caption (`caption:`), if it has one.
pub fn caption(c: Custom, page: &CatalogPage) -> Option<String> {
    match c {
        Custom::McpInstall => Some(agents::install_caption(page)),
        _ => None,
    }
}

const MINUTES_KEY: &str = "reminder-minutes";
const MESSAGE_KEY: &str = "reminder-message";

/// Sends the new reminder the form holds (`setReminder(String(minutes), message)`).
fn remind(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let minutes = page
        .drafts
        .get(MINUTES_KEY)
        .cloned()
        .unwrap_or_else(|| notifications::MINUTES.0.to_string());
    let message = page.drafts.get(MESSAGE_KEY).cloned().unwrap_or_default();
    let Some(body) = notifications::reminder_write(&minutes, &message) else {
        return;
    };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |settings, cx| {
        settings.send(
            &client,
            "reminder",
            &body,
            "The reminder was not set",
            window,
            cx,
        );
    });
}

/// The waiting reminders, each with Copy and Show, as the page's `Repeater` draws them.
pub fn reminders(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let settings = page.stars.settings.read(cx);
    let list = notifications::REMINDERS
        .read(settings)
        .value()
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    list.iter()
        .enumerate()
        .map(|(i, item)| {
            let (label, description) =
                (notifications::label(item), notifications::description(item));
            let copy = notifications::copy_text(item);
            let (client, shown) = (page.stars.client.clone(), this.clone());
            let buttons = h_flex()
                .gap(Space::Sm)
                .child(
                    Button::new(SharedString::from(format!("reminder-{i}-copy")))
                        .label("Copy")
                        .variant(ButtonVariant::Outline)
                        .disabled(copy.is_empty())
                        .on_click(move |_, window, cx| row::copy(&client, &copy, window, cx)),
                )
                .child(
                    Button::new(SharedString::from(format!("reminder-{i}-show")))
                        .label("Show")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, window, cx| {
                            shown.update(cx, |p, cx| {
                                let client = p.stars.client.clone();
                                p.stars.settings.update(cx, |settings, cx| {
                                    settings.send(
                                        &client,
                                        "reminder.show",
                                        &notifications::show(),
                                        "The reminders were not shown",
                                        window,
                                        cx,
                                    );
                                });
                            })
                        }),
                );
            let star = page.stars.star(page.page().id, &label, &description, cx);
            row::setting(label, description, notifications::HINT, buttons, star).into_any_element()
        })
        .collect()
}

/// The row's control.
pub fn control(
    c: Custom,
    label: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let settings = page.stars.settings.read(cx);
    let busy = page.jobs.read(cx).job_busy();
    let this = this.clone();
    match c {
        Custom::RunningKernel => {
            let summary = match page.inventories.read(cx).known("diagnostics") {
                Known::Value(doc) => kernel::kernel_summary(Some(doc)),
                Known::Waiting => return row::reading(label).into_any_element(),
                Known::Unknown(_) => String::new(),
            };
            let client = page.stars.client.clone();
            Button::new(SharedString::from(id))
                .label("Copy")
                .variant(ButtonVariant::Outline)
                .disabled(summary.is_empty())
                .on_click(move |_, window, cx| row::copy(&client, &summary, window, cx))
                .into_any_element()
        }
        Custom::DirectBoot => {
            let Some(direct) = kernel::DIRECT.read(settings).value().cloned() else {
                return row::reading(label).into_any_element();
            };
            let set = direct == Value::Bool(true);
            let (text, confirm) = if set { kernel::REMOVE } else { kernel::SET_UP };
            let b = Btn {
                label: text,
                setter: kernel::SETUP_DIRECT_BOOT,
                when: kernel::DIRECT_ENABLED,
                confirm: Some(confirm),
                prompt: None,
            };
            Button::new(SharedString::from(id))
                .label(text)
                .variant(if set {
                    ButtonVariant::Destructive
                } else {
                    ButtonVariant::Outline
                })
                .disabled(!b.when.holds_with(settings, busy))
                .on_click(move |_, window, cx| {
                    this.update(cx, |p, cx| p.press(label, b, window, cx))
                })
                .into_any_element()
        }
        Custom::McpInstall => agents::install_buttons(id, page, &this).into_any_element(),
        Custom::Detected("path") => v_flex().into_any_element(),
        Custom::Detected(which) => match environment::shows(which, page, cx) {
            Some(text) => Text::new(text).into_any_element(),
            None => row::reading(label).into_any_element(),
        },
        Custom::PathPrepend => environment::prepend(id, label, page, &this, cx).into_any_element(),
        Custom::AddAutostart => startup::form(id, page, &this).into_any_element(),
        Custom::NoFailures | Custom::NoRules | Custom::RuleCenter => v_flex().into_any_element(),
        Custom::AddRule => rules::add_button(id, &this).into_any_element(),
        Custom::BarCount => workspaces::count(id, label, page, &this, cx).into_any_element(),
        Custom::WorkspaceSwitch(key) => {
            workspaces::switch(id, label, key, page, &this, cx).into_any_element()
        }
        Custom::LoginWorkspace => workspaces::login(id, label, page, &this, cx).into_any_element(),
        Custom::AddScratch => workspaces::add_scratch(id, page, &this).into_any_element(),
        Custom::Themes => appearance::themes(page, &this, cx).into_any_element(),
        Custom::NoExtraThemes => v_flex().into_any_element(),
        Custom::ExtraThemes => appearance::extras(id, page, &this, cx).into_any_element(),
        Custom::AddTheme => appearance::add_button(id, page, &this, cx).into_any_element(),
        Custom::NightSchedule => appearance::schedule(id, page, &this, cx).into_any_element(),
        Custom::UseSchedule => {
            appearance::use_schedule(id, label, page, &this, cx).into_any_element()
        }
        Custom::CapturePick(which) => {
            capture::pick(id, label, which, page, &this, cx).into_any_element()
        }
        Custom::TakeShot => capture::take(id, page, &this).into_any_element(),
        Custom::RecSwitch(key) => {
            capture::switch(id, label, key, page, &this, cx).into_any_element()
        }
        Custom::ResizeWebcam => capture::resize(id, page, &this, cx).into_any_element(),
        Custom::SaveFolder(key) => capture::folder(id, key, page, &this, cx).into_any_element(),
        Custom::Hw(which) => hardware::control(which, id, page, cx).into_any_element(),
        Custom::DevEnv => software::dev_env(id, page, &this, cx).into_any_element(),
        Custom::DockerDb => software::docker_db(id, page, &this, cx).into_any_element(),
        Custom::AppAutostart => {
            crate::pages::launchers::add_autostart(id, page, &this).into_any_element()
        }
        Custom::AppNoAutostart | Custom::NoLaunchers(_) => v_flex().into_any_element(),
        Custom::Hist(which) => {
            crate::pages::history::control(which, id, label, page, &this, cx).into_any_element()
        }
        Custom::Disk(which) => {
            crate::pages::disks::control(which, id, label, page, &this, cx).into_any_element()
        }
        Custom::Display(which) => {
            crate::pages::display::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Kb(which) => {
            crate::pages::keyboard::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Live(which) => crate::pages::live::control(which, id, page, cx).into_any_element(),
        Custom::Meter(which) if which.starts_with("no") => v_flex().into_any_element(),
        Custom::Meter(which) => {
            crate::pages::meters::control(which, id, page, cx).into_any_element()
        }
        Custom::Acct(which) => {
            crate::pages::accounts::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Wifi(which) => {
            crate::pages::wifi::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Bt(which) => match crate::pages::radios::bt_control(which, id, page, &this, cx) {
            Some(switch) => switch.into_any_element(),
            None => v_flex().into_any_element(),
        },
        Custom::Export(which) => {
            crate::pages::export::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Net(which) => {
            crate::pages::network::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Bind(which) => {
            crate::pages::bindings::control(which, id, page, &this).into_any_element()
        }
        Custom::Logo(which) => {
            crate::pages::system::logo(which, id, page, &this, cx).into_any_element()
        }
        Custom::Diag(which) => {
            crate::pages::diagnostics::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::Bar(which) => {
            crate::pages::bar::control(which, id, label, page, &this, cx).into_any_element()
        }
        Custom::Sys(which) => {
            crate::pages::system::control(which, id, label, page, &this, cx).into_any_element()
        }
        Custom::InList => v_flex().into_any_element(),
        Custom::HookAdd => crate::pages::hooks::add_button(id, &this).into_any_element(),
        Custom::Choose(pick) => {
            choose(crate::catalog::picks::pick(pick), label, id, busy, &this).into_any_element()
        }
        Custom::BootReset => {
            let b = Btn {
                label: "Reset…",
                setter: crate::catalog::custom::BOOT_RESET,
                when: crate::catalog::When::All(&[
                    crate::catalog::When::Idle,
                    crate::catalog::When::IsNot(Field::Key("plymouth"), "default"),
                ]),
                confirm: Some(crate::catalog::setter::Confirm {
                    title: "Reset boot screen",
                    message: "Put the stock Omarchy unlock theme and login screen back.",
                    action: "Reset",
                }),
                prompt: None,
            };
            Button::new(SharedString::from(id))
                .label("Reset…")
                .variant(ButtonVariant::Destructive)
                .disabled(!b.when.holds_with(settings, busy))
                .on_click(move |_, window, cx| {
                    this.update(cx, |p, cx| p.press(label, b, window, cx))
                })
                .into_any_element()
        }
        Custom::AddLauncher(kind) => {
            crate::pages::launchers::add_button(kind, id, page, &this, cx).into_any_element()
        }
        Custom::Speed(which) => {
            crate::pages::speedtest::control(which, id, page, &this, cx).into_any_element()
        }
        Custom::AddVar => environment::add(id, page, &this).into_any_element(),
        Custom::McpNone => Text::new("None").into_any_element(),
        Custom::NewReminder => {
            let draft = |key: &str, or: &str| {
                page.drafts
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| or.to_owned())
            };
            let minutes = draft(MINUTES_KEY, &notifications::MINUTES.0.to_string());
            let message = draft(MESSAGE_KEY, "");
            let (lo, hi) = (notifications::MINUTES.1, notifications::MINUTES.2);
            let (counted, typed, submitted, set) = (this.clone(), this.clone(), this.clone(), this);
            h_flex()
                .gap(Space::Sm)
                .child(
                    NumberInput::new(SharedString::from(format!("{id}-minutes")), "Minutes")
                        .value(minutes.parse().unwrap_or(notifications::MINUTES.0))
                        .min(lo)
                        .max(hi)
                        .on_commit(move |n, _, cx| {
                            counted.update(cx, |p, cx| {
                                p.drafts
                                    .insert(MINUTES_KEY.into(), (n.round() as u64).to_string());
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Input::new(SharedString::from(format!("{id}-message")), "Message")
                        .placeholder("Message")
                        .value(message)
                        .on_change(move |text, _, cx| {
                            typed.update(cx, |p, cx| {
                                p.drafts.insert(MESSAGE_KEY.into(), text.to_owned());
                                cx.notify();
                            })
                        })
                        .on_submit(move |text, window, cx| {
                            submitted.update(cx, |p, cx| {
                                p.drafts.insert(MESSAGE_KEY.into(), text.to_owned());
                                remind(p, window, cx);
                            })
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("{id}-set")))
                        .label("Set")
                        .variant(ButtonVariant::Primary)
                        .on_click(move |_, window, cx| {
                            set.update(cx, |p, cx| remind(p, window, cx))
                        }),
                )
                .into_any_element()
        }
        Custom::IdleLock => {
            let field = Field::Key("idleLock");
            let Some(current) = field.read(settings).value().cloned() else {
                return row::reading(label).into_any_element();
            };
            let range = crate::catalog::control::range(0.0, 3600.0, 60.0, 0);
            super::controls::number(
                id,
                label,
                range,
                &current,
                false,
                move |next, window, cx| {
                    this.update(cx, |p, cx| {
                        let s = p.stars.settings.read(cx);
                        let body = crate::system::idle_lock_write(
                            Field::Key("idleScreensaver").read(s).value(),
                            Field::Key("idleLock").read(s).value(),
                            crate::js::number(Some(&next)),
                        );
                        let Some(body) = body else { return };
                        let client = p.stars.client.clone();
                        p.stars.settings.update(cx, |s, cx| {
                            s.send(
                                &client,
                                "idleScreensaver",
                                &body,
                                "The idle timeout was not changed",
                                window,
                                cx,
                            )
                        });
                    })
                },
            )
            .into_any_element()
        }
        Custom::Swappiness => {
            let field = Field::Key("tweaks");
            let Some(tweaks) = field.read(settings).value().cloned() else {
                return row::reading(label).into_any_element();
            };
            let on = kernel::swappiness(Some(&tweaks));
            let (toggled, reset) = (this.clone(), this);
            h_flex()
                .gap(Space::Sm)
                .child(
                    Switch::new(SharedString::from(format!("{id}-switch")), label)
                        .checked(on)
                        .on_change(move |_, window, cx| {
                            toggled.update(cx, |p, cx| {
                                p.call(
                                    label,
                                    Some(field),
                                    kernel::SET_SWAPPINESS,
                                    Value::Bool(!on),
                                    window,
                                    cx,
                                )
                            })
                        }),
                )
                .child(
                    Button::new(SharedString::from(format!("{id}-reset")))
                        .label("Reset")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, window, cx| {
                            reset.update(cx, |p, cx| {
                                p.call(
                                    label,
                                    Some(field),
                                    kernel::SET_SWAPPINESS,
                                    Value::Bool(false),
                                    window,
                                    cx,
                                )
                            })
                        }),
                )
                .into_any_element()
        }
    }
}

/// A button that opens the file chooser and hands the path to the pick's setter.
fn choose(
    pick: &'static crate::catalog::picks::Pick,
    label: &'static str,
    id: String,
    busy: bool,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    use crate::catalog::picks::{Kind, Start};
    let this = this.clone();
    Button::new(SharedString::from(id))
        .label(pick.button)
        .variant(ButtonVariant::Outline)
        .disabled(pick.idle && busy)
        .on_click(move |_, window, cx| {
            let chooser = match pick.kind {
                Kind::Open => FileChooser::open(pick.title),
                Kind::Save => FileChooser::save(pick.title),
                Kind::Folder => FileChooser::folder(pick.title),
            };
            let chooser = pick.filters.iter().fold(chooser, |c, (name, patterns)| {
                c.filter(*name, patterns.iter().copied())
            });
            let chooser = chooser.start_in(match pick.start {
                Start::Home => StartFolder::Home,
                Start::Pictures => StartFolder::Pictures,
            });
            crate::pages::files::choose(
                chooser,
                &this,
                window,
                cx,
                Box::new(move |p, path, window, cx| {
                    p.call(label, None, pick.setter, Value::from(path), window, cx)
                }),
            );
        })
}
