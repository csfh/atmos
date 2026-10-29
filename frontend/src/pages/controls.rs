//! The controls a catalog page draws that are more than one ODK component: a slider of the QML
//! page as a number field, a choice, a line of text and its Set button, and the two dialogs a
//! button can ask first, a confirm and a prompt for one line of text.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::catalog::Field;
use crate::catalog::control::Range;
use crate::catalog::setter::{Btn, Confirm, Sends};
use crate::js;

/// A slider of the QML page as a number field: what is typed is rounded as the slider rounds it,
/// and written when it is committed.
pub fn number(
    id: String,
    label: &'static str,
    range: Range,
    current: &Value,
    disabled: bool,
    commit: impl Fn(Value, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    NumberInput::new(SharedString::from(id), label)
        .value(range.shown(current))
        .min(range.min)
        .max(range.max)
        .step(range.step)
        .disabled(disabled)
        .on_commit(move |typed, window, cx| commit(js::json_number(range.round(typed)), window, cx))
}

/// A choice of `options`, with `current` chosen; it cannot be used while there is none.
pub fn select(
    id: String,
    label: &'static str,
    options: &[(String, String)],
    current: &Value,
    disabled: bool,
    commit: impl Fn(&str, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    options
        .iter()
        .fold(
            Select::new(SharedString::from(id), label),
            |select, (value, text)| select.item(value.clone(), text.clone()),
        )
        .value(js::string(current))
        .disabled(disabled || options.is_empty())
        .on_change(commit)
}

/// A line of text and its Set button: what is typed is kept until it is written, and Enter or
/// Set writes it.
#[allow(clippy::too_many_arguments)]
pub fn text(
    id: String,
    label: &'static str,
    field: Field,
    placeholder: &'static str,
    current: &Value,
    draft: Option<String>,
    disabled: bool,
    this: Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let shown = draft.clone().unwrap_or_else(|| js::string(current));
    let (typed, submitted, set) = (this.clone(), this.clone(), this);
    let (key, key2) = (id.clone(), id.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-field")), label)
                .placeholder(placeholder)
                .value(shown.clone())
                .disabled(disabled)
                .on_change(move |text, _, cx| {
                    let key = key.clone();
                    typed.update(cx, |p, cx| {
                        p.drafts.insert(key, text.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |text, window, cx| {
                    let (key, value) = (key2.clone(), Value::from(text));
                    submitted.update(cx, |p, cx| {
                        p.drafts.remove(&key);
                        p.put(label, field, value, window, cx)
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-set")))
                .label("Set")
                .variant(ButtonVariant::Outline)
                .disabled(disabled)
                .on_click(move |_, window, cx| {
                    let (key, value) = (id.clone(), Value::from(shown.clone()));
                    set.update(cx, |p, cx| {
                        p.drafts.remove(&key);
                        p.put(label, field, value, window, cx)
                    })
                }),
        )
}

/// The confirm a button asks before it calls its setter (`PrefsConfirm`).
pub fn confirm(
    asking: Option<Confirm>,
    asked: Option<(String, String, &'static str)>,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let open = asking.is_some() || asked.is_some();
    let (title, message, action) = match (asking, asked) {
        (Some(c), _) => (c.title.to_owned(), c.message.to_owned(), c.action),
        (None, Some(texts)) => texts,
        (None, None) => (String::new(), String::new(), ""),
    };
    let destructive = Confirm {
        title: "",
        message: "",
        action,
    }
    .destructive();
    let (closed, cancelled, confirmed) = (this.clone(), this.clone(), this.clone());
    Dialog::new("catalog-confirm", title)
        .open(open)
        .description(message)
        .on_close(move |window, cx| closed.update(cx, |p, cx| p.answer(false, window, cx)))
        .action(move |_, _| {
            let cancelled = cancelled.clone();
            Button::new("catalog-confirm-cancel")
                .label("Cancel")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, window, cx| {
                    cancelled.update(cx, |p, cx| p.answer(false, window, cx))
                })
        })
        .action(move |_, _| {
            let confirmed = confirmed.clone();
            Button::new("catalog-confirm-go")
                .label(action)
                .variant(if destructive {
                    ButtonVariant::Destructive
                } else {
                    ButtonVariant::Primary
                })
                .on_click(move |_, window, cx| {
                    confirmed.update(cx, |p, cx| p.answer(true, window, cx))
                })
        })
}

/// The line of text a button asks for before it calls its setter (`PrefsDialog` with a field).
pub fn prompt(
    prompting: Option<(Btn, &'static str, String)>,
    running: Option<&'static str>,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let b = prompting.as_ref().map(|(b, _, _)| *b);
    let p = b.and_then(|b| b.prompt);
    let draft = prompting.map(|(_, _, d)| d).unwrap_or_default();
    // While the job the setter starts runs (`Omarchy.jobKind === "..." && Omarchy.jobBusy`).
    let busy = match b.map(|b| b.setter.sends) {
        Some(Sends::Job(t)) => running == Some(t.kind),
        _ => false,
    };
    let valid = p.is_some_and(|p| !p.parse.parse(&draft).is_empty());
    let (title, placeholder, action) =
        p.map_or(("", "", ""), |p| (p.title, p.placeholder, p.action));
    let say = match p {
        Some(p) if busy => p.busy,
        Some(p) if !draft.is_empty() && !valid => p.invalid,
        Some(p) => p.intro,
        None => "",
    };
    let (closed, typed, submitted, went) = (this.clone(), this.clone(), this.clone(), this.clone());
    Dialog::new("catalog-prompt", title)
        .open(p.is_some())
        .description(say)
        .on_close(move |window, cx| {
            closed.update(cx, |page, cx| page.answer_prompt(false, window, cx))
        })
        .content(move |_, _| {
            let (typed, submitted) = (typed.clone(), submitted.clone());
            Input::new("catalog-prompt-field", title)
                .placeholder(placeholder)
                .value(draft.clone())
                .invalid(!draft.is_empty() && !valid)
                .disabled(busy)
                .on_change(move |text, _, cx| {
                    typed.update(cx, |page, cx| page.type_prompt(text, cx))
                })
                .on_submit(move |_, window, cx| {
                    submitted.update(cx, |page, cx| page.answer_prompt(true, window, cx))
                })
        })
        .action(move |_, _| {
            let went = went.clone();
            Button::new("catalog-prompt-go")
                .label(action)
                .variant(ButtonVariant::Primary)
                .disabled(busy || !valid)
                .on_click(move |_, window, cx| {
                    went.update(cx, |page, cx| page.answer_prompt(true, window, cx))
                })
        })
}
