//! The Environment page's rows and list (`environment.rs` has what they say and send): the
//! detected values, the PATH prepend and its Set, the form that adds a variable, and each
//! variable with Remove. The PATH prepend's draft starts again when the stored prepend changes
//! (`onEnvPathPrependChanged`).

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::environment::{self as env, DETECTED, PREPEND, VARS};
use crate::pages::row;

const PREPEND_DRAFT: &str = "env-prepend";
const KEY_DRAFT: &str = "env-key";
const VALUE_DRAFT: &str = "env-value";
const ERROR: &str = "env-error";
const SEEN: &str = "env-prepend-seen";

fn read(page: &CatalogPage, field: crate::catalog::Field, cx: &App) -> Option<Value> {
    field.read(page.stars.settings.read(cx)).value().cloned()
}

/// The stored prepend changed: its draft starts again from it.
pub fn settings_changed(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    let now = read(page, PREPEND, cx).unwrap_or(Value::Null);
    if page.data.get(SEEN).is_some_and(|seen| *seen != now) {
        page.drafts.remove(PREPEND_DRAFT);
    }
    page.data.insert(SEEN, now);
}

/// Sends `setEnvVars(vars, prepend)`.
fn set_env(
    page: &mut CatalogPage,
    vars: &Value,
    prepend: &Value,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    let client = page.stars.client.clone();
    let bodies = env::set_env(vars, prepend);
    page.stars.settings.update(cx, |settings, cx| {
        for body in bodies {
            let key = if body["domain"] == "envVars" {
                "envVars"
            } else {
                "envPathPrepend"
            };
            settings.send(
                &client,
                key,
                &body,
                "The environment was not changed",
                window,
                cx,
            );
        }
    });
}

/// A detected value, as its row shows it: `None` until the snapshot has it.
pub fn shows(which: &str, page: &CatalogPage, cx: &App) -> Option<String> {
    let raw = read(page, DETECTED, cx)?;
    Some(match which {
        "session" => env::session(Some(&raw)),
        "shell" => env::shell(Some(&raw)),
        "tools" => env::tools(Some(&raw)),
        _ => env::path(Some(&raw)),
    })
}

/// What Add a variable says: what went wrong, or what it takes.
pub fn add_description(page: &CatalogPage) -> String {
    page.data
        .get(ERROR)
        .and_then(Value::as_str)
        .filter(|e| !e.is_empty())
        .unwrap_or(env::ADD)
        .to_owned()
}

/// The PATH prepend's field and Set.
pub fn prepend(
    id: String,
    label: &'static str,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> impl IntoElement + use<> {
    let stored = read(page, PREPEND, cx).map_or_else(String::new, |v| crate::js::string(&v));
    let draft = page
        .drafts
        .get(PREPEND_DRAFT)
        .cloned()
        .unwrap_or_else(|| stored.clone());
    let changed = draft != stored;
    let (typed, submitted, set) = (this.clone(), this.clone(), this.clone());
    let sent = draft.clone();
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-field")), label)
                .placeholder("/opt/bin")
                .value(draft)
                .on_change(move |text, _, cx| {
                    typed.update(cx, |p, cx| {
                        p.drafts.insert(PREPEND_DRAFT.into(), text.to_owned());
                        cx.notify();
                    })
                })
                .on_submit(move |text, window, cx| {
                    submitted.update(cx, |p, cx| {
                        p.drafts.insert(PREPEND_DRAFT.into(), text.to_owned());
                        let vars = read(p, VARS, cx).unwrap_or(Value::Null);
                        set_env(p, &vars, &Value::from(text), window, cx);
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-set")))
                .label("Set")
                .variant(ButtonVariant::Primary)
                .disabled(!changed)
                .on_click(move |_, window, cx| {
                    let sent = sent.clone();
                    set.update(cx, |p, cx| {
                        let vars = read(p, VARS, cx).unwrap_or(Value::Null);
                        set_env(p, &vars, &Value::from(sent), window, cx);
                    })
                }),
        )
}

/// The form that adds a variable (`addVar`).
pub fn add(id: String, page: &CatalogPage, this: &Entity<CatalogPage>) -> impl IntoElement + use<> {
    let key = page.drafts.get(KEY_DRAFT).cloned().unwrap_or_default();
    let value = page.drafts.get(VALUE_DRAFT).cloned().unwrap_or_default();
    let (keyed, valued, added) = (this.clone(), this.clone(), this.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Input::new(SharedString::from(format!("{id}-key")), "Name")
                .placeholder("EDITOR")
                .value(key)
                .on_change(move |text, _, cx| {
                    keyed.update(cx, |p, cx| {
                        p.drafts.insert(KEY_DRAFT.into(), text.to_owned());
                        cx.notify();
                    })
                }),
        )
        .child(
            Input::new(SharedString::from(format!("{id}-value")), "Value")
                .placeholder("nvim")
                .value(value)
                .on_change(move |text, _, cx| {
                    valued.update(cx, |p, cx| {
                        p.drafts.insert(VALUE_DRAFT.into(), text.to_owned());
                        cx.notify();
                    })
                }),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-add")))
                .label("Add")
                .variant(ButtonVariant::Primary)
                .on_click(move |_, window, cx| {
                    added.update(cx, |p, cx| {
                        let key = p.drafts.get(KEY_DRAFT).cloned().unwrap_or_default();
                        let value = p.drafts.get(VALUE_DRAFT).cloned().unwrap_or_default();
                        let vars = read(p, VARS, cx).unwrap_or(Value::Null);
                        match env::add_var(&vars, &key, &value) {
                            Err(why) => {
                                p.data.insert(ERROR, Value::from(why));
                                cx.notify();
                            }
                            Ok(next) => {
                                p.data.insert(ERROR, Value::from(""));
                                let prepend = read(p, PREPEND, cx).unwrap_or(Value::from(""));
                                set_env(p, &Value::Array(next), &prepend, window, cx);
                            }
                        }
                    })
                }),
        )
}

/// Each variable, with Remove.
pub fn vars(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let all = read(page, VARS, cx).unwrap_or(Value::Null);
    all.as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, row)| {
            let (label, value) = (env::var_label(&row), env::var_value(&row));
            let (this, list, key) = (this.clone(), all.clone(), row.get("key").cloned());
            let remove = Button::new(SharedString::from(format!("env-var-{i}")))
                .label("Remove")
                .variant(ButtonVariant::Destructive)
                .on_click(move |_, window, cx| {
                    let next = Value::Array(env::remove_var(&list, key.as_ref()));
                    this.update(cx, |p, cx| {
                        let prepend = read(p, PREPEND, cx).unwrap_or(Value::from(""));
                        set_env(p, &next, &prepend, window, cx);
                    })
                });
            let star = page.stars.star(page.page().id, &label, &value, cx);
            row::setting(label, value, "", remove, star).into_any_element()
        })
        .collect()
}
