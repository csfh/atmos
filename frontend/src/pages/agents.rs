//! The Agentic and MCP pages' lists and actions (`agentic.rs` has what they say): the agents
//! the backend lists, read when either page is shown and again when the default agent changes or
//! an MCP entry is written. The QML MCP page also reads the list again when an agent's config
//! file changes (`Backend.watch`); this window reads it when the page is shown.

use odk::prelude::*;
use serde_json::Value;

use super::catalog::CatalogPage;
use crate::agentic;
use crate::catalog::Field;
use crate::pages::row;
use crate::protocol;
use crate::settings;

/// The pages that read the agents.
pub fn reads_agents(page: &str) -> bool {
    matches!(page, "agentic" | "agentic/mcp")
}

const AGENTS: &str = "agents";
const ERROR: &str = "agents-error";
const CHECK: &str = "mcp-check";
const SEEN: &str = "agent-seen";

/// The agents as last read, empty until they are.
fn agents(page: &CatalogPage) -> Vec<Value> {
    page.data
        .get(AGENTS)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn text(page: &CatalogPage, key: &str) -> String {
    page.data
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// The default agent, "" until read.
fn agent(page: &CatalogPage, cx: &App) -> String {
    Field::Key("agent")
        .read(page.stars.settings.read(cx))
        .value()
        .map_or_else(String::new, crate::js::string)
}

/// Reads the agents again (`reload`); `then` is what the last write said went wrong, which the
/// page says once the list is read (`apply`, `installAll`).
pub fn load(page: &mut CatalogPage, then: Option<String>, cx: &mut Context<CatalogPage>) {
    if !reads_agents(page.page().id) {
        return;
    }
    let reply = page.stars.client.request(&protocol::agents_mcp_list());
    cx.spawn(async move |this, cx| {
        let outcome = reply.await;
        let _ = this.update(cx, |page, cx| {
            match outcome {
                Ok(list) => {
                    page.data.insert(AGENTS, list);
                    page.data
                        .insert(ERROR, Value::from(then.unwrap_or_default()));
                }
                Err(failure) if then.is_none() => {
                    let why = or(failure.message, "Could not read agents");
                    page.data.insert(ERROR, Value::from(why));
                }
                Err(_) => {}
            }
            cx.notify();
        });
    })
    .detach();
}

/// The failure's words, or `fallback` when it has none (`failureText`).
fn or(message: String, fallback: &str) -> String {
    if message.is_empty() {
        fallback.to_owned()
    } else {
        message
    }
}

/// The default agent changed: Agentic reads the list again (`onAgentChanged`).
pub fn settings_changed(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    if page.page().id != "agentic" {
        return;
    }
    let now = Value::from(agent(page, cx));
    if page.data.get(SEEN).is_some_and(|seen| *seen != now) {
        load(page, None, cx);
    }
    page.data.insert(SEEN, now);
}

/// What a list's group says, built from the list.
pub fn detail(list: crate::catalog::List, page: &CatalogPage) -> Option<String> {
    match list {
        crate::catalog::List::Agents => {
            Some(agentic::agents_detail(&agents(page), &text(page, ERROR)))
        }
        crate::catalog::List::McpAgents => Some(agentic::mcp_detail(&agents(page))),
        _ => None,
    }
}

/// Agentic: makes `id` the default (`useAgent`, `Omarchy.set("agent", id)`).
fn use_agent(
    page: &mut CatalogPage,
    id: String,
    window: &mut Window,
    cx: &mut Context<CatalogPage>,
) {
    if id.is_empty() || id == agent(page, cx) {
        return;
    }
    let client = page.stars.client.clone();
    let body = settings::write("agent", &Value::from(id));
    page.stars.settings.update(cx, |settings, cx| {
        settings.send(
            &client,
            "agent",
            &body,
            "The agent was not changed",
            window,
            cx,
        );
    });
}

/// MCP: turns an agent's entry on or off (`apply`), then reads the list again.
fn apply(
    page: &mut CatalogPage,
    id: String,
    on: bool,
    replace: bool,
    cx: &mut Context<CatalogPage>,
) {
    let reply = page
        .stars
        .client
        .request(&protocol::agents_mcp_set(&id, on, replace));
    cx.spawn(async move |this, cx| {
        let why = reply
            .await
            .err()
            .map_or_else(String::new, |f| or(f.message, "Could not update"));
        let _ = this.update(cx, |page, cx| load(page, Some(why), cx));
    })
    .detach();
}

/// MCP: Install for every installed agent (`installAll`): each in turn, then the list again, with
/// the first failure.
fn install_all(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    let todo = agentic::installable(&agents(page));
    let client = page.stars.client.clone();
    cx.spawn(async move |this, cx| {
        let mut failed = String::new();
        for id in todo {
            if let Err(f) = client
                .request(&protocol::agents_mcp_set(&id, true, false))
                .await
                && failed.is_empty()
            {
                failed = or(f.message, "Could not update");
            }
        }
        let _ = this.update(cx, |page, cx| load(page, Some(failed), cx));
    })
    .detach();
}

/// MCP: Check (`runCheck`).
fn run_check(page: &mut CatalogPage, cx: &mut Context<CatalogPage>) {
    page.data.insert(CHECK, Value::from("Checking…"));
    cx.notify();
    let reply = page.stars.client.request(&protocol::agents_mcp_check());
    cx.spawn(async move |this, cx| {
        let said = match reply.await {
            Ok(result) => agentic::tools_text(&result),
            Err(f) => or(f.message, "Check failed"),
        };
        let _ = this.update(cx, |page, cx| {
            page.data.insert(CHECK, Value::from(said));
            cx.notify();
        });
    })
    .detach();
}

/// MCP: the Install row's caption, what went wrong or what Check said.
pub fn install_caption(page: &CatalogPage) -> String {
    let error = text(page, ERROR);
    if error.is_empty() {
        text(page, CHECK)
    } else {
        error
    }
}

/// MCP: the Install row's buttons.
pub fn install_buttons(
    id: String,
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
) -> impl IntoElement + use<> {
    let none = agentic::installable(&agents(page)).is_empty();
    let (all, check) = (this.clone(), this.clone());
    h_flex()
        .gap(Space::Sm)
        .child(
            Button::new(SharedString::from(format!("{id}-all")))
                .label("Install for every installed agent")
                .variant(ButtonVariant::Outline)
                .disabled(none)
                .on_click(move |_, _, cx| all.update(cx, install_all)),
        )
        .child(
            Button::new(SharedString::from(format!("{id}-check")))
                .label("Check")
                .variant(ButtonVariant::Outline)
                .on_click(move |_, _, cx| check.update(cx, run_check)),
        )
}

/// MCP: whether "No agents" shows: none installed, and nothing went wrong.
pub fn none_installed(page: &CatalogPage) -> bool {
    agentic::installed(&agents(page)).is_empty() && text(page, ERROR).is_empty()
}

/// Agentic: the agents, each with Use or Install, and the default when the list lacks it.
pub fn agent_rows(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    let list = agents(page);
    let default = agent(page, cx);
    let mut rows = Vec::new();
    if !default.is_empty() && !agentic::known_agent(&list, &default) {
        let star = page
            .stars
            .star(page.page().id, &default, "Omarchy opens this agent.", cx);
        rows.push(
            row::setting(
                default.clone(),
                "Omarchy opens this agent.",
                "",
                Text::new("Default"),
                star,
            )
            .into_any_element(),
        );
    }
    for (i, a) in list.iter().enumerate() {
        let (name, id) = (agentic::name(a), crate::js::string(&a["id"]));
        let description = agentic::agent_description(a, &default);
        let control = if id == default {
            Text::new("Default").into_any_element()
        } else {
            let this = this.clone();
            let chosen = id.clone();
            Button::new(SharedString::from(format!("agent-{i}")))
                .label(agentic::agent_button(a))
                .variant(ButtonVariant::Outline)
                .on_click(move |_, window, cx| {
                    let chosen = chosen.clone();
                    this.update(cx, |p, cx| use_agent(p, chosen, window, cx))
                })
                .into_any_element()
        };
        let star = page.stars.star(page.page().id, &name, description, cx);
        rows.push(row::setting(name, description, "", control, star).into_any_element());
    }
    rows
}

/// MCP: the installed agents, each with its entry's switch and Replace.
pub fn mcp_rows(
    page: &CatalogPage,
    this: &Entity<CatalogPage>,
    cx: &App,
) -> Vec<impl IntoElement + use<>> {
    agentic::installed(&agents(page))
        .into_iter()
        .enumerate()
        .map(|(i, a)| {
            let (name, id) = (agentic::name(&a), crate::js::string(&a["id"]));
            let on = a.get("on") == Some(&Value::Bool(true));
            let (toggled, replaced) = (this.clone(), this.clone());
            let (id_a, id_b) = (id.clone(), id);
            let control = h_flex()
                .gap(Space::Sm)
                .child(
                    Switch::new(SharedString::from(format!("mcp-{i}")), name.clone())
                        .checked(on)
                        .disabled(!agentic::writable(&a))
                        .on_change(move |next, _, cx| {
                            let id = id_a.clone();
                            toggled.update(cx, |p, cx| apply(p, id, next, false, cx))
                        }),
                )
                .children(agentic::replaceable(&a).then(|| {
                    Button::new(SharedString::from(format!("mcp-{i}-replace")))
                        .label("Replace")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            let id = id_b.clone();
                            replaced.update(cx, |p, cx| apply(p, id, true, true, cx))
                        })
                }));
            let description = agentic::mcp_description(&a);
            let star = page.stars.star(page.page().id, &name, description, cx);
            row::captioned(
                name,
                description,
                agentic::mcp_caption(&a),
                agentic::mcp_hint(&a),
                control,
                star,
            )
            .into_any_element()
        })
        .collect()
}
