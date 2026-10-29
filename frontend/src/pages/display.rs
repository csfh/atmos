//! The Displays page's rows (`display.rs` has what they say and send): each monitor as a group
//! of its own with its mode, rule and brightness, the layouts, and the laptop's touchpad and
//! keyboard backlight. A rule, the focused monitor's scale, and turning the touchpad off open the
//! guard's bar (`Settings::send_guarded`), as `Omarchy.qml` hands them `dispatchSetting`.

use odk::prelude::*;
use serde_json::{Value, json};

use super::catalog::CatalogPage;
use crate::catalog::Field;
use crate::catalog::control::range;
use crate::display::{self, MONITORS, RULES};
use crate::pages::controls::{number, select};
use crate::pages::row;

fn value(page: &CatalogPage, field: Field, cx: &App) -> Option<Value> {
    field.read(page.stars.settings.read(cx)).value().cloned()
}

fn list(page: &CatalogPage, field: Field, cx: &App) -> Vec<Value> {
    value(page, field, cx)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

fn on(page: &CatalogPage, name: &'static str, cx: &App) -> bool {
    value(page, Field::Key(name), cx) == Some(Value::Bool(true))
}

/// Whether the Displays group is drawn while it has no row: Hyprland reported no output.
pub fn shows(page: &CatalogPage, cx: &App) -> bool {
    list(page, MONITORS, cx).is_empty()
}

fn send(
    page: &mut CatalogPage,
    key: &'static str,
    body: Value,
    guard: Option<(&'static str, Value)>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let client = page.stars.client.clone();
    let refused = "The display setting was not changed";
    page.stars.settings.update(cx, |s, cx| match guard {
        Some(guard) => s.send_guarded(&client, key, &body, refused, guard, window, cx),
        None => s.send(&client, key, &body, refused, window, cx),
    });
}

/// `writeMonitorRules(items)`, guarded by the rules shown now.
fn write_rules(
    page: &mut CatalogPage,
    items: Vec<Value>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let current = value(page, RULES, cx);
    let (body, revert) = display::rules_write(current.as_ref(), items);
    send(
        page,
        "monitorRules",
        body,
        Some(("monitorRules", revert)),
        window,
        cx,
    );
}

/// `patchMonitorRule(output, patch)`.
fn patch(
    page: &mut CatalogPage,
    output: &str,
    patch: Value,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let (rules, monitors) = (list(page, RULES, cx), list(page, MONITORS, cx));
    if let Some(next) = crate::monitors::patch_rules(&rules, &monitors, output, &patch) {
        write_rules(page, next, window, cx);
    }
}

/// The Scale row's choice: the focused output through `setMonitorScale`, another by its rule.
fn scale(
    page: &mut CatalogPage,
    m: &Value,
    v: &str,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if v == display::scale_value(m) {
        return;
    }
    let monitors = list(page, MONITORS, cx);
    if m.get("focused") == Some(&Value::Bool(true)) {
        if let Some((body, revert)) = display::scale_write(v, &monitors) {
            let guard = revert.map(|r| ("monitorScale", r));
            send(page, "monitorScale", body, guard, window, cx);
        }
        return;
    }
    let rules = display::scale_rules(m, &list(page, RULES, cx), v);
    write_rules(page, rules, window, cx);
}

/// A choice of a monitor's row, which hands what is picked to `act` with the monitor.
#[allow(clippy::too_many_arguments)]
fn pick(
    id: String,
    label: &'static str,
    options: &[(String, String)],
    now: String,
    disabled: bool,
    m: &Value,
    this: &Entity<CatalogPage>,
    act: fn(&mut CatalogPage, &Value, &str, &mut Window, &mut Context<CatalogPage>),
) -> impl IntoElement + use<> {
    let (this, m) = (this.clone(), m.clone());
    select(
        id,
        label,
        options,
        &Value::from(now),
        disabled,
        move |v, window, cx| {
            let v = v.to_owned();
            let m = m.clone();
            this.update(cx, |p, cx| act(p, &m, &v, window, cx))
        },
    )
    .into_any_element()
}

fn fixed(options: &[(&str, &str)]) -> Vec<(String, String)> {
    options
        .iter()
        .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
        .collect()
}

fn name(m: &Value) -> String {
    m.get("name")
        .filter(|v| crate::js::truthy(Some(v)))
        .map_or_else(String::new, crate::js::string)
}

/// Each monitor, as a group of its own.
pub fn monitors(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let (monitors, rules) = (list(page, MONITORS, cx), list(page, RULES, cx));
    let hub = page.page().id;
    monitors
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let named = !name(m).is_empty();
            let id = |part: &str| format!("monitor-{i}-{part}");
            let star = |label: &str, says: &str| page.stars.star(hub, label, says, cx);
            let mut rows = Vec::new();

            let resolution = display::current_resolution(m);
            let says = display::resolution_says(m);
            let copied = display::copy_text(m);
            let client = page.stars.client.clone();
            let control = h_flex()
                .gap(Space::Sm)
                .child(pick(id("resolution"), "Resolution", &display::resolutions(m), resolution.clone(), !named, m, this, |p, m, v, window, cx| {
                    if v != display::current_resolution(m) {
                        let refresh = display::current_refresh(m, v);
                        if let Some(patch_) = display::mode_patch(v, &refresh) {
                            patch(p, &name(m), patch_, window, cx);
                        }
                    }
                }))
                .child(
                    Button::new(SharedString::from(id("copy")))
                        .label("Copy")
                        .variant(ButtonVariant::Outline)
                        .disabled(copied.is_empty())
                        .on_click(move |_, window, cx| row::copy(&client, &copied, window, cx)),
                );
            rows.push(row::setting("Resolution", says.clone(), "~/.config/hypr/monitors.lua", control, star("Resolution", &says)).into_any_element());

            let says = "How many frames this panel draws per second. Only the rates the resolution above supports are listed.";
            let rates = display::refresh_rates(m, &resolution);
            let control = pick(id("refresh"), "Refresh rate", &rates, display::current_refresh(m, &resolution), !named, m, this, |p, m, v, window, cx| {
                let resolution = display::current_resolution(m);
                if v != display::current_refresh(m, &resolution)
                    && let Some(patch_) = display::mode_patch(&resolution, v)
                {
                    patch(p, &name(m), patch_, window, cx);
                }
            });
            rows.push(row::setting("Refresh rate", says, "~/.config/hypr/monitors.lua", control, star("Refresh rate", says)).into_any_element());

            if named {
                let says = "How this panel is turned.";
                let control = pick(id("rotation"), "Rotation", &fixed(display::TRANSFORMS), display::rule_transform(m, &rules), false, m, this, |p, m, v, window, cx| {
                    let t = crate::js::round(v.parse().unwrap_or(f64::NAN));
                    let t = if t.is_nan() { 0.0 } else { t };
                    patch(p, &name(m), json!({ "transform": crate::js::json_number(t) }), window, cx);
                });
                rows.push(row::setting("Rotation", says, "hl.monitor transform", control, star("Rotation", says)).into_any_element());

                let says = display::disable_says(m, &monitors, &rules);
                let off = display::rule_disabled(m, &rules);
                let (this_, m_) = (this.clone(), m.clone());
                let control = Switch::new(SharedString::from(id("disable")), "Disable this display")
                    .checked(off)
                    .disabled(!display::can_disable(m, &monitors, &rules))
                    .on_change(move |_, window, cx| {
                        let m = m_.clone();
                        this_.update(cx, |p, cx| {
                            let (monitors, rules) = (list(p, MONITORS, cx), list(p, RULES, cx));
                            let off = display::rule_disabled(&m, &rules);
                            if off || display::can_disable(&m, &monitors, &rules) {
                                patch(p, &name(&m), json!({ "disabled": !off }), window, cx);
                            }
                        })
                    });
                rows.push(row::setting("Disable this display", says, "hl.monitor disabled", control, star("Disable this display", says)).into_any_element());

                if display::vrr_available(m) {
                    let says = "VRR when this output supports it. Fullscreen-only is the safer game setting.";
                    let control = pick(id("vrr"), "Variable refresh", &fixed(display::VRRS), display::rule_vrr(m, &rules), false, m, this, |p, m, v, window, cx| {
                        let n = crate::js::round(v.parse().unwrap_or(f64::NAN));
                        let n = if n.is_nan() { 0.0 } else { n };
                        patch(p, &name(m), json!({ "vrr": crate::js::json_number(n) }), window, cx);
                    });
                    rows.push(row::setting("Variable refresh", says, "hl.monitor vrr", control, star("Variable refresh", says)).into_any_element());
                }

                let says = "10-bit when the panel and cable can do it.";
                let control = pick(id("bitdepth"), "Bit depth", &fixed(display::DEPTHS), display::rule_bitdepth(m, &rules).to_owned(), false, m, this, |p, m, v, window, cx| {
                    patch(p, &name(m), json!({ "bitdepth": if v == "10" { 10 } else { 8 } }), window, cx);
                });
                rows.push(row::setting("Bit depth", says, "hl.monitor bitdepth", control, star("Bit depth", says)).into_any_element());

                let says = display::color_says(display::hdr_available(m, &rules));
                let control = pick(id("color"), "Color", &fixed(display::COLORS), display::rule_cm(m, &rules), false, m, this, |p, m, v, window, cx| {
                    patch(p, &name(m), json!({ "cm": v }), window, cx);
                });
                rows.push(row::setting("Color", says, "hl.monitor cm", control, star("Color", says)).into_any_element());

                let says = "How large the interface looks on this monitor. The focused output uses omarchy hyprland monitor scaling. Other outputs write a monitor rule.";
                let control = pick(id("scale"), "Scale", &display::scale_options(m), display::scale_value(m), false, m, this, scale);
                rows.push(row::setting("Scale", says, "omarchy hyprland monitor scaling", control, star("Scale", says)).into_any_element());
            }

            if m.get("brightnessAvailable") == Some(&Value::Bool(true)) {
                let says = "How bright this panel is. Works on the built-in display and on some external monitors.";
                let now = m.get("brightness").filter(|v| crate::js::truthy(Some(v))).cloned().unwrap_or(json!(1));
                let (this_, m_) = (this.clone(), m.clone());
                let control = number(id("brightness"), "Brightness", range(1.0, 100.0, 1.0, 0), &now, false, move |v, window, cx| {
                    let next = crate::js::number(Some(&v));
                    if m_.get("brightness").and_then(Value::as_f64) == Some(next) {
                        return;
                    }
                    let Some(body) = display::brightness_write(&name(&m_), next) else {
                        return;
                    };
                    this_.update(cx, |p, cx| send(p, "monitors", body, None, window, cx));
                });
                rows.push(row::stacked("Brightness", says, "omarchy brightness display", control, star("Brightness", says)).into_any_element());
            }

            row::group(
                display::title(m),
                "Mode, scale, rotation, and disable write a monitor rule in ~/.config/hypr/monitors.lua. Brightness works on the built-in panel and on some external monitors.",
                rows,
            )
            .into_any_element()
        })
        .collect()
}

/// "No monitors reported.", with Refresh and Edit, while Hyprland reports none.
pub fn empty(page: &CatalogPage, cx: &App) -> Vec<impl IntoElement + use<>> {
    if value(page, MONITORS, cx).is_none() || !shows(page, cx) {
        return Vec::new();
    }
    let client = page.stars.client.clone();
    let edit = client.clone();
    let control = h_flex()
        .gap(Space::Sm)
        .child(
            Button::new("monitors-refresh")
                .label("Refresh")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, _| {
                    drop(client.request(&crate::protocol::snapshot_invalidate(
                        crate::settings::ALL_TOPICS,
                    )))
                }),
        )
        .child(
            Button::new("monitors-edit")
                .label("Edit")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, _| {
                    if let Some(body) =
                        display::edit_write(&std::env::var("HOME").unwrap_or_default())
                    {
                        drop(edit.request(&body));
                    }
                }),
        );
    let star = page
        .stars
        .star(page.page().id, "Monitors", "No monitors reported.", cx);
    vec![row::setting(
        "Monitors",
        "No monitors reported.",
        "hyprctl monitors all",
        control,
        star,
    )]
}

/// Apply a layout: Desk, Laptop while there is a built-in panel, Docked while there is an
/// external one (`applyMonitorLayout`).
pub fn layouts(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let button = |name: &'static str, label: &'static str, usable: bool| {
        let this = this.clone();
        Button::new(SharedString::from(format!("{id}-{name}")))
            .label(label)
            .variant(ButtonVariant::Outline)
            .disabled(!usable)
            .on_click(move |_, window, cx| {
                this.update(cx, |p, cx| {
                    let present = (on(p, "internalPresent", cx), on(p, "externalPresent", cx));
                    let (monitors, current) = (list(p, MONITORS, cx), value(p, RULES, cx));
                    if let Some((body, revert)) =
                        crate::monitors::layout_write(name, &monitors, present, current.as_ref())
                    {
                        let guard = Some(("monitorRules", revert));
                        send(p, "monitorRules", body, guard, window, cx);
                    }
                })
            })
    };
    h_flex()
        .gap(Space::Sm)
        .child(button("desk", "Desk", true))
        .child(button("laptop", "Laptop", on(page, "internalPresent", cx)))
        .child(button("docked", "Docked", on(page, "externalPresent", cx)))
}

/// Touchpad: its switch, guarded when it turns the touchpad off (`setTouchpad`).
pub fn touchpad(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let this = this.clone();
    let enabled = value(page, Field::Key("touchpadEnabled"), cx);
    Switch::new(SharedString::from(id), "Touchpad")
        .checked(enabled == Some(Value::Bool(true)))
        .disabled(!on(page, "touchpadPresent", cx))
        .on_change(move |_, window, cx| {
            this.update(cx, |p, cx| {
                let now = value(p, Field::Key("touchpadEnabled"), cx);
                let next = now != Some(Value::Bool(true));
                if let Some((body, revert)) = display::touchpad_write(next, now.as_ref()) {
                    let guard = revert.map(|r| ("touchpad", r));
                    send(p, "touchpadEnabled", body, guard, window, cx);
                }
            })
        })
}

fn brightness(page: &CatalogPage, cx: &App) -> f64 {
    crate::js::number(value(page, Field::Key("keyboardBrightness"), cx).as_ref())
}

/// The keyboard backlight row's words.
pub fn backlight_says(page: &CatalogPage, cx: &App) -> String {
    display::backlight_says(brightness(page, cx))
}

/// Keyboard backlight: Dim, Brighten, and Turn off while it is on (`adjustKeyboardBacklight`).
pub fn backlight(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let present = on(page, "keyboardBacklightPresent", cx);
    let lit = brightness(page, cx) > 0.0;
    let button = |direction: &'static str, label: &'static str, usable: bool| {
        let this = this.clone();
        Button::new(SharedString::from(format!("{id}-{direction}")))
            .label(label)
            .variant(ButtonVariant::Outline)
            .disabled(!usable)
            .on_click(move |_, window, cx| {
                if let Some(body) = display::backlight_write(direction) {
                    this.update(cx, |p, cx| {
                        send(p, "keyboardBrightness", body, None, window, cx)
                    });
                }
            })
    };
    h_flex()
        .gap(Space::Sm)
        .child(button("down", "Dim", present))
        .child(button("up", "Brighten", present))
        .child(button("off", "Turn off", present && lit))
}

/// Whether a row of the page's own shows (`listed` ones are drawn by each monitor's group).
pub fn shown(which: &str, page: &CatalogPage, cx: &App) -> bool {
    match which {
        "listed" => false,
        "touchpad" => on(page, "touchpadPresent", cx),
        "backlight" => on(page, "keyboardBacklightPresent", cx),
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
    match which {
        "layout" => layouts(id, page, this, cx).into_any_element(),
        "touchpad" => touchpad(id, page, this, cx).into_any_element(),
        "backlight" => backlight(id, page, this, cx).into_any_element(),
        _ => v_flex().into_any_element(),
    }
}
