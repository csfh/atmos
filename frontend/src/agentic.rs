//! The Agentic and MCP pages' logic (`pages/AgenticPage.qml`, `pages/agentic/McpPage.qml`): the
//! agents the backend lists (`agents.mcp.list`), what each page says of them, and what choosing
//! one or turning its MCP entry on sends. `tests/catalog_custom.rs` runs the pages' own
//! functions under Node against this (`common::qml::PAGE_ROOT`).

use serde_json::Value;

use crate::js;

fn text(row: &Value, name: &str) -> String {
    row.get(name)
        .filter(|v| js::truthy(Some(v)))
        .map_or_else(String::new, js::string)
}

fn is(row: &Value, name: &str) -> bool {
    js::truthy(row.get(name))
}

/// An agent's name: `modelData.name || modelData.id`.
pub fn name(row: &Value) -> String {
    let name = text(row, "name");
    if name.is_empty() {
        text(row, "id")
    } else {
        name
    }
}

/// Agentic: `installedCount`.
pub fn installed_count(agents: &[Value]) -> usize {
    agents.iter().filter(|a| is(a, "installed")).count()
}

/// Agentic: `knownAgent`, whether the default is one the list has.
pub fn known_agent(agents: &[Value], agent: &str) -> bool {
    agent.is_empty()
        || agents
            .iter()
            .any(|a| a.get("id").and_then(Value::as_str) == Some(agent))
}

/// Agentic: what the Agents group says.
pub fn agents_detail(agents: &[Value], error: &str) -> String {
    if !error.is_empty() {
        return error.to_owned();
    }
    if agents.is_empty() {
        return "Choosing one writes it as the Omarchy default. If it is missing, Omarchy offers to install it.".to_owned();
    }
    format!(
        "{} of {} installed. Choosing one writes it as the Omarchy default.",
        installed_count(agents),
        agents.len()
    )
}

/// Agentic: `rowDescription`.
pub fn agent_description(row: &Value, agent: &str) -> &'static str {
    if row.get("id").and_then(Value::as_str) == Some(agent) {
        "Omarchy opens this agent."
    } else if is(row, "installed") {
        "Installed on this machine."
    } else {
        "Omarchy can install this agent and open it."
    }
}

/// Agentic: the button of an agent that is not the default.
pub fn agent_button(row: &Value) -> &'static str {
    if is(row, "installed") {
        "Use"
    } else {
        "Install"
    }
}

/// MCP: the agents that are installed.
pub fn installed(agents: &[Value]) -> Vec<Value> {
    agents
        .iter()
        .filter(|a| is(a, "installed"))
        .cloned()
        .collect()
}

/// MCP: whether Atmos can write the agent's entry, which is absent or its own.
pub fn writable(row: &Value) -> bool {
    row.get("writer") == Some(&Value::Bool(true))
        && matches!(
            row.get("state").and_then(Value::as_str),
            Some("absent" | "ours")
        )
}

/// MCP: `installable`, the installed agents Install for every installed agent would write.
pub fn installable(agents: &[Value]) -> Vec<String> {
    installed(agents)
        .iter()
        .filter(|a| {
            is(a, "writer")
                && matches!(
                    a.get("state").and_then(Value::as_str),
                    Some("absent" | "ours")
                )
        })
        .map(|a| js::string(&a["id"]))
        .collect()
}

/// MCP: what the Installed agents group says (`missingNames`).
pub fn mcp_detail(agents: &[Value]) -> String {
    let missing: Vec<String> = agents
        .iter()
        .filter(|a| !is(a, "installed"))
        .map(|a| {
            let n = text(a, "name");
            if n.is_empty() { text(a, "id") } else { n }
        })
        .collect();
    if missing.is_empty() {
        "Every agent Omarchy knows is installed.".to_owned()
    } else {
        format!("Not installed: {}.", missing.join(", "))
    }
}

/// MCP: `rowDescription`.
pub fn mcp_description(row: &Value) -> &'static str {
    if !is(row, "writer") {
        return "Atmos cannot write this agent's MCP config yet.";
    }
    match row.get("state").and_then(Value::as_str) {
        Some("custom") => "This agent already has an Omarchy entry with a different command.",
        Some("stale") => "The Omarchy entry points at an old command.",
        _ => "Reads this machine and changes settings through Atmos.",
    }
}

/// MCP: an agent's hint, its config path or `claude mcp`.
pub fn mcp_hint(row: &Value) -> String {
    let path = text(row, "path");
    if !path.is_empty() {
        path
    } else if row.get("id").and_then(Value::as_str) == Some("claude") {
        "claude mcp".to_owned()
    } else {
        String::new()
    }
}

/// MCP: the caption of an agent whose entry is not Atmos's.
pub fn mcp_caption(row: &Value) -> &'static str {
    match row.get("state").and_then(Value::as_str) {
        Some("custom") => "Custom command",
        Some("stale") => "Out of date",
        _ => "",
    }
}

/// MCP: whether an agent's Replace shows.
pub fn replaceable(row: &Value) -> bool {
    row.get("writer") == Some(&Value::Bool(true))
        && matches!(
            row.get("state").and_then(Value::as_str),
            Some("custom" | "stale")
        )
}

/// MCP: what Check says of the tools the server has.
pub fn tools_text(result: &Value) -> String {
    let n = result
        .get("tools")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    format!("{n}{}", if n == 1 { " tool" } else { " tools" })
}
