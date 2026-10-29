//! Input's Hyprland layouts rows (`keyboard.rs` has what they say and send): the layout list and
//! the variant list, each typed and Set, and Alt+Alt. What is typed is kept until the input block
//! changes, as the QML page resets its drafts then; each write opens the guard's bar.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::keyboard::{self, Draft, GESTURE_UNMANAGED, INPUT_KEY};

const LAYOUTS: &str = "kb-layouts";
const VARIANTS: &str = "kb-variants";
/// The input block the drafts were typed against.
const SEEN: &str = "kb-seen";

fn raw(page: &CatalogPage, cx: &App) -> Option<Value> {
    INPUT_KEY
        .read(page.stars.settings.read(cx))
        .value()
        .cloned()
}

fn unmanaged(page: &CatalogPage, cx: &App) -> bool {
    GESTURE_UNMANAGED.read(page.stars.settings.read(cx)).value() == Some(&Value::Bool(true))
}

/// The window's `hyprInput`, and the drafts as the page reads them: what was typed while the
/// block has not changed since, else what is set.
fn state(page: &CatalogPage, cx: &App) -> (Value, Draft, String, String) {
    let raw = raw(page, cx);
    let input = keyboard::input(raw.as_ref());
    let fresh = page.data.get(SEEN) == Some(raw.as_ref().unwrap_or(&Value::Null));
    let typed = |key: &str, field: &str| match page.drafts.get(key) {
        Some(text) if fresh => text.clone(),
        _ => crate::js::string(&input[field]),
    };
    let (layouts, variants) = (
        typed(LAYOUTS, "kbLayoutOverride"),
        typed(VARIANTS, "kbVariantOverride"),
    );
    let draft = Draft::of(&input, &layouts, &variants);
    (input, draft, layouts, variants)
}

/// What a row of the page's own says, when it follows the drafts.
pub fn says(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let (_, draft, _, _) = state(page, cx);
    match which {
        "layouts" => Some(draft.layouts_says().to_owned()),
        "variants" => Some(draft.variants_says().to_owned()),
        _ => None,
    }
}

fn send(
    page: &mut CatalogPage,
    write: Option<(Value, Value)>,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let Some((body, revert)) = write else {
        return;
    };
    let client = page.stars.client.clone();
    page.stars.settings.update(cx, |s, cx| {
        let refused = "The keyboard layouts were not changed";
        s.send_guarded(
            &client,
            "hyprInput",
            &body,
            refused,
            ("kbOverride", revert),
            window,
            cx,
        )
    });
}

/// `applyKbOverride()`: the drafts, while they are valid.
fn apply(page: &mut CatalogPage, window: &mut Window, cx: &mut Context<CatalogPage>) {
    let (input, draft, _, _) = state(page, cx);
    if !draft.valid() {
        return;
    }
    let toggle = input["kbGroupToggle"] == Value::Bool(true);
    let write = keyboard::set(
        &input,
        (&draft.layouts, &draft.variants, toggle),
        unmanaged(page, cx),
    );
    send(page, write, window, cx);
}

fn keep(page: &mut CatalogPage, key: &str, text: &str, cx: &mut Context<CatalogPage>) {
    let (layouts, variants) = {
        let (_, _, l, v) = state(page, cx);
        (l, v)
    };
    // The other list keeps what it shows, now typed against this block too.
    page.drafts.insert(LAYOUTS.into(), layouts);
    page.drafts.insert(VARIANTS.into(), variants);
    page.drafts.insert(key.into(), text.to_owned());
    page.data.insert(SEEN, raw(page, cx).unwrap_or(Value::Null));
    cx.notify();
}

/// A list's field and Set (`layouts` or `variants`).
fn list(
    which: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let (_, draft, layouts, variants) = state(page, cx);
    let (key, label, placeholder, shown, ok) = if which == "layouts" {
        (
            LAYOUTS,
            "Hyprland layouts",
            "us,dk",
            layouts,
            draft.layouts_ok,
        )
    } else {
        (
            VARIANTS,
            "Variants",
            ",nodeadkeys",
            variants,
            draft.variants_ok,
        )
    };
    let (typed, submitted, set) = (this.clone(), this.clone(), this.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-field")), label)
                .placeholder(placeholder)
                .value(shown)
                .invalid(!ok)
                .on_change(move |text, _, cx| {
                    let text = text.to_owned();
                    typed.update(cx, |p, cx| keep(p, key, &text, cx))
                })
                .on_submit(move |text, window, cx| {
                    let text = text.to_owned();
                    submitted.update(cx, |p, cx| {
                        keep(p, key, &text, cx);
                        apply(p, window, cx)
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-set")))
                .label("Set")
                .variant(ButtonVariant::Primary)
                .disabled(!(draft.valid() && draft.dirty))
                .on_click(move |_, window, cx| set.update(cx, |p, cx| apply(p, window, cx))),
        )
}

/// Alt+Alt layout switch.
fn toggle(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let (input, _, _, _) = state(page, cx);
    let this = this.clone();
    Switch::new(SharedString::from(id), "Alt+Alt layout switch")
        .checked(input["kbGroupToggle"] == Value::Bool(true))
        .on_change(move |_, window, cx| {
            this.update(cx, |p, cx| {
                let (input, draft, _, _) = state(p, cx);
                let write = draft.toggle(&input, unmanaged(p, cx));
                send(p, write, window, cx)
            })
        })
}

/// The control of a row of the page's own (`layouts`, `variants`, `toggle`).
pub fn control(
    which: &'static str,
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    match which {
        "toggle" => toggle(id, page, this, cx).into_any_element(),
        _ => list(which, id, page, this, cx).into_any_element(),
    }
}
