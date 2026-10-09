//! Which installed coding agents have the Atmos MCP entry, and the writers
//! that put it there. The agent's own config is the record. Atmos does not
//! keep a second list.
//!
//! Under `--root` nothing is probed on PATH and Claude is not spawned. The
//! fixture says who is installed, and a Claude change is appended to
//! `commands.log`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};
use toml_edit::{value, Array, DocumentMut, Item, Table};

use crate::error::{Error, Kind, Result};
use crate::fsutil::atomic_write;
use crate::host;
use crate::runner::{Run, COMMAND_TIMEOUT};
use crate::store;

const CLAUDE_RECORD: &str = ".local/state/omarchy/claude-mcp.json";
const INSTALL_MANIFEST: &str = ".local/state/omarchy/agents-mcp.json";

#[derive(Clone, Copy)]
struct Spec {
    id: &'static str,
    name: &'static str,
    /// Binary `omarchy-default-agent` looks for under ~/.local/bin.
    binary: &'static str,
    /// `mise where` package. Ignored when `installer` is set.
    package: &'static str,
    installer: Option<&'static str>,
    writer: Writer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Writer {
    None,
    Grok,
    Cursor,
    Claude,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Presence {
    Absent,
    Ours,
    Stale,
    Custom,
}

struct Found {
    presence: Presence,
    enabled: bool,
}

impl Found {
    fn absent() -> Self {
        Found {
            presence: Presence::Absent,
            enabled: false,
        }
    }

    fn on(&self) -> bool {
        self.presence == Presence::Ours && self.enabled
    }
}

const ROSTER: &[Spec] = &[
    spec("pi", "Pi", "pi", "pi", None, Writer::None),
    spec(
        "omp",
        "Oh My Pi",
        "omp",
        "github:can1357/oh-my-pi",
        None,
        Writer::None,
    ),
    spec("opencode", "OpenCode", "opencode", "opencode", None, Writer::None),
    spec("claude", "Claude Code", "claude", "claude", None, Writer::Claude),
    spec("codex", "Codex", "codex", "codex", None, Writer::None),
    spec(
        "grok",
        "Grok",
        "grok",
        "npm:@xai-official/grok",
        None,
        Writer::Grok,
    ),
    spec("gemini", "Gemini", "gemini", "gemini", None, Writer::None),
    spec(
        "openclaw",
        "OpenClaw",
        "openclaw",
        "openclaw",
        Some("omarchy-install-openclaw-cli"),
        Writer::None,
    ),
    spec(
        "hermes",
        "Hermes",
        "hermes",
        "hermes",
        Some("omarchy-install-hermes-cli"),
        Writer::None,
    ),
    spec("copilot", "GitHub Copilot", "copilot", "copilot", None, Writer::None),
    spec("crush", "Crush", "crush", "crush", None, Writer::None),
    spec(
        "cursor-agent",
        "Cursor",
        "cursor-agent",
        "cursor-agent",
        None,
        Writer::Cursor,
    ),
    spec(
        "muse",
        "Muse",
        "muse",
        "http:muse[url=https://api.meta.ai/muse-launcher.sh,bin=muse,version_list_url=https://api.meta.ai/muse-code/channels/muse-stable,version_json_path=.version]",
        None,
        Writer::None,
    ),
];

const fn spec(
    id: &'static str,
    name: &'static str,
    binary: &'static str,
    package: &'static str,
    installer: Option<&'static str>,
    writer: Writer,
) -> Spec {
    Spec {
        id,
        name,
        binary,
        package,
        installer,
        writer,
    }
}

pub fn list(root: Option<&Path>) -> Result<Value> {
    let atmos = find_atmos(root);
    let installed = installed_ids(root)?;
    let mut rows = Vec::with_capacity(ROSTER.len());
    for spec in ROSTER {
        let found = read_entry(root, spec, atmos.as_deref())?;
        let path = entry_path(root, spec)?;
        rows.push(json!({
            "id": spec.id,
            "name": spec.name,
            "installed": installed.iter().any(|id| id == spec.id),
            "writer": spec.writer != Writer::None,
            "path": path.map(|p| p.display().to_string()).unwrap_or_default(),
            "state": presence_name(found.presence),
            "on": found.on(),
        }));
    }
    Ok(Value::Array(rows))
}

pub fn set(root: Option<&Path>, agent: &str, on: bool, replace: bool) -> Result<Value> {
    let spec = find_spec(agent)?;
    if spec.writer == Writer::None {
        return Err(
            Error::denied("Atmos cannot write this agent's MCP config yet").with_context(agent),
        );
    }
    let atmos = require_atmos(root)?;
    match spec.writer {
        Writer::Grok => set_toml(root, ".grok/config.toml", &atmos, on, replace)?,
        Writer::Cursor => set_cursor(root, &atmos, on, replace)?,
        Writer::Claude => set_claude(root, &atmos, on, replace)?,
        Writer::None => {}
    }
    Ok(json!({ "agent": spec.id, "on": on }))
}

pub fn check(root: Option<&Path>) -> Result<Value> {
    let atmos = require_atmos(root)?;
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"atmos","version":"0.1.0"}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n",
    );
    let output = Run::new(&atmos)
        .arg("mcp")
        .input(input.as_bytes().to_vec())
        .timeout(Duration::from_secs(20))
        .output()?;
    let names = tool_names(&output.stdout_text());
    if names.is_empty() {
        let detail = output.stderr_text();
        let message = if detail.trim().is_empty() {
            format!("exited {}", output.code)
        } else {
            detail.trim().to_string()
        };
        return Err(Error::command(message).with_context(atmos.display().to_string()));
    }
    Ok(json!({ "tools": names }))
}

fn find_spec(agent: &str) -> Result<&'static Spec> {
    ROSTER
        .iter()
        .find(|spec| spec.id == agent)
        .ok_or_else(|| Error::bad_request(format!("unknown agent {agent}")).with_context(agent))
}

fn presence_name(presence: Presence) -> &'static str {
    match presence {
        Presence::Absent => "absent",
        Presence::Ours => "ours",
        Presence::Stale => "stale",
        Presence::Custom => "custom",
    }
}

fn installed_ids(root: Option<&Path>) -> Result<Vec<String>> {
    if let Some(root) = root {
        let path = root.join(INSTALL_MANIFEST);
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let text = fs::read_to_string(&path).map_err(|err| Error::io(err, &path))?;
        let value: Value = serde_json::from_str(&text)?;
        let ids = value
            .get("installed")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        return Ok(ids);
    }
    let mut ids = Vec::new();
    for spec in ROSTER {
        if installed_live(spec) {
            ids.push(spec.id.to_string());
        }
    }
    Ok(ids)
}

fn installed_live(spec: &Spec) -> bool {
    if user_binary(spec.binary) {
        return true;
    }
    if let Some(installer) = spec.installer {
        return Run::new(installer)
            .arg("--check")
            .timeout(Duration::from_secs(10))
            .output()
            .map(|out| out.success())
            .unwrap_or(false);
    }
    Run::new("mise")
        .args(["where", spec.package])
        .timeout(Duration::from_secs(15))
        .output()
        .map(|out| out.success())
        .unwrap_or(false)
}

/// A real binary in ~/.local/bin. The mise stub that only runs `mise use -g`
/// does not count, matching `omarchy-default-agent`.
fn user_binary(name: &str) -> bool {
    let Some(home) = std::env::var_os("HOME") else {
        return false;
    };
    let path = PathBuf::from(home).join(".local/bin").join(name);
    if !is_exec(&path) {
        return false;
    }
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(_) => return false,
    };
    if meta.file_type().is_symlink() {
        return true;
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    !text.lines().any(|line| line.starts_with("mise use -g"))
}

fn find_atmos(root: Option<&Path>) -> Option<PathBuf> {
    if let Ok(path) = host::confine(root, ".local/bin/atmos") {
        if is_exec(&path) {
            return Some(path);
        }
    }
    if root.is_some() {
        return None;
    }
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join("atmos");
        if is_exec(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn require_atmos(root: Option<&Path>) -> Result<PathBuf> {
    find_atmos(root).ok_or_else(|| {
        Error::new(Kind::NotFound, "Atmos is not installed").with_context(".local/bin/atmos")
    })
}

fn is_exec(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    meta.is_file() && meta.permissions().mode() & 0o111 != 0
}

fn entry_path(root: Option<&Path>, spec: &Spec) -> Result<Option<PathBuf>> {
    let rel = match spec.writer {
        Writer::Grok => ".grok/config.toml",
        Writer::Cursor => ".cursor/mcp.json",
        Writer::Claude => {
            return Ok(None);
        }
        Writer::None => return Ok(None),
    };
    Ok(Some(host::confine(root, rel)?))
}

fn read_entry(root: Option<&Path>, spec: &Spec, atmos: Option<&Path>) -> Result<Found> {
    match spec.writer {
        Writer::None => Ok(Found::absent()),
        Writer::Grok => read_toml(&host::confine(root, ".grok/config.toml")?, atmos),
        Writer::Cursor => read_cursor(&host::confine(root, ".cursor/mcp.json")?, atmos),
        Writer::Claude => read_claude(root, atmos),
    }
}

fn guard(found: &Found, on: bool, replace: bool, path: &Path) -> Result<Guard> {
    if on && found.on() {
        return Ok(Guard::Skip);
    }
    if !on && found.presence == Presence::Absent {
        return Ok(Guard::Skip);
    }
    let blocked = matches!(found.presence, Presence::Stale | Presence::Custom);
    if blocked && !replace {
        return Err(Error::denied("Replace the existing Omarchy entry first")
            .with_context(path.display().to_string()));
    }
    Ok(Guard::Write)
}

enum Guard {
    Skip,
    Write,
}

fn set_toml(root: Option<&Path>, rel: &str, atmos: &Path, on: bool, replace: bool) -> Result<()> {
    let path = host::confine(root, rel)?;
    store::with_lock(&path, || {
        let found = read_toml(&path, Some(atmos))?;
        if matches!(guard(&found, on, replace, &path)?, Guard::Skip) {
            return Ok(());
        }
        let mut doc = load_toml(&path)?;
        if on {
            let servers = doc
                .as_table_mut()
                .entry("mcp_servers")
                .or_insert(Item::Table(Table::new()));
            let servers = servers.as_table_mut().ok_or_else(|| {
                Error::bad_request("mcp_servers is not a table")
                    .with_context(path.display().to_string())
            })?;
            let agent = servers
                .entry("omarchy")
                .or_insert(Item::Table(Table::new()));
            let agent = agent.as_table_mut().ok_or_else(|| {
                Error::bad_request("omarchy is not a table")
                    .with_context(path.display().to_string())
            })?;
            agent["command"] = value(atmos.display().to_string());
            let mut args = Array::new();
            args.push("mcp");
            agent["args"] = Item::Value(toml_edit::Value::Array(args));
            agent["enabled"] = value(true);
        } else if let Some(servers) = doc
            .as_table_mut()
            .get_mut("mcp_servers")
            .and_then(Item::as_table_mut)
        {
            servers.remove("omarchy");
        }
        atomic_write(&path, doc.to_string().as_bytes())
    })
}

fn load_toml(path: &Path) -> Result<DocumentMut> {
    if !path.is_file() {
        return Ok(DocumentMut::new());
    }
    let text = fs::read_to_string(path).map_err(|err| Error::io(err, path))?;
    if text.trim().is_empty() {
        return Ok(DocumentMut::new());
    }
    text.parse::<DocumentMut>()
        .map_err(|err| Error::bad_request(err.to_string()).with_context(path.display().to_string()))
}

fn read_toml(path: &Path, atmos: Option<&Path>) -> Result<Found> {
    if !path.is_file() {
        return Ok(Found::absent());
    }
    let doc = load_toml(path)?;
    let Some(server) = doc.get("mcp_servers").and_then(|item| item.get("omarchy")) else {
        return Ok(Found::absent());
    };
    let command = server.get("command").and_then(Item::as_str).unwrap_or("");
    let args = string_list(server.get("args"));
    let enabled = server
        .get("enabled")
        .and_then(Item::as_bool)
        .unwrap_or(true);
    Ok(Found {
        presence: classify(command, &args, atmos),
        enabled,
    })
}

fn string_list(item: Option<&Item>) -> Vec<String> {
    item.and_then(Item::as_array)
        .map(|list| {
            list.iter()
                .filter_map(toml_edit::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn set_cursor(root: Option<&Path>, atmos: &Path, on: bool, replace: bool) -> Result<()> {
    let path = host::confine(root, ".cursor/mcp.json")?;
    store::with_lock(&path, || {
        let found = read_cursor(&path, Some(atmos))?;
        if matches!(guard(&found, on, replace, &path)?, Guard::Skip) {
            return Ok(());
        }
        let mut doc = load_json_object(&path)?;
        let servers = doc
            .as_object_mut()
            .ok_or_else(|| Error::bad_request("cursor config is not an object"))?
            .entry("mcpServers")
            .or_insert_with(|| json!({}));
        let servers = servers.as_object_mut().ok_or_else(|| {
            Error::bad_request("mcpServers is not an object")
                .with_context(path.display().to_string())
        })?;
        if on {
            servers.insert(
                "omarchy".into(),
                json!({
                    "command": atmos.display().to_string(),
                    "args": ["mcp"],
                }),
            );
        } else {
            servers.remove("omarchy");
        }
        let mut text = serde_json::to_string_pretty(&doc)?;
        text.push('\n');
        atomic_write(&path, text.as_bytes())
    })
}

fn load_json_object(path: &Path) -> Result<Value> {
    if !path.is_file() {
        return Ok(json!({}));
    }
    let text = fs::read_to_string(path).map_err(|err| Error::io(err, path))?;
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    let value: Value = serde_json::from_str(&text)?;
    if !value.is_object() {
        return Err(
            Error::bad_request("config is not an object").with_context(path.display().to_string())
        );
    }
    Ok(value)
}

fn read_cursor(path: &Path, atmos: Option<&Path>) -> Result<Found> {
    if !path.is_file() {
        return Ok(Found::absent());
    }
    let doc = load_json_object(path)?;
    let Some(server) = doc.get("mcpServers").and_then(|item| item.get("omarchy")) else {
        return Ok(Found::absent());
    };
    let command = server.get("command").and_then(Value::as_str).unwrap_or("");
    let args: Vec<String> = server
        .get("args")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(Found {
        presence: classify(command, &args, atmos),
        enabled: true,
    })
}

fn read_claude(root: Option<&Path>, atmos: Option<&Path>) -> Result<Found> {
    if let Some(root) = root {
        let path = host::confine(Some(root), CLAUDE_RECORD)?;
        if !path.is_file() {
            return Ok(Found::absent());
        }
        let text = fs::read_to_string(&path).map_err(|err| Error::io(err, &path))?;
        let value: Value = serde_json::from_str(&text)?;
        let command = value.get("command").and_then(Value::as_str).unwrap_or("");
        let args: Vec<String> = value
            .get("args")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if command.is_empty() {
            return Ok(Found::absent());
        }
        return Ok(Found {
            presence: classify(command, &args, atmos),
            enabled: true,
        });
    }
    let output = Run::new("claude")
        .args(["mcp", "get", "omarchy"])
        .timeout(Duration::from_secs(20))
        .output()?;
    let text = output.stdout_text();
    if text.contains("No MCP server named") {
        return Ok(Found::absent());
    }
    if let Some(found) = parse_claude_get(&text, atmos) {
        return Ok(found);
    }
    if output.success() {
        return Ok(Found::absent());
    }
    let detail = output.stderr_text();
    Err(Error::command(if detail.trim().is_empty() {
        format!("exited {}", output.code)
    } else {
        detail.trim().to_string()
    })
    .with_context("claude"))
}

fn parse_claude_get(text: &str, atmos: Option<&Path>) -> Option<Found> {
    let mut command = None;
    let mut args: Option<Vec<String>> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Command:") {
            command = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("Args:") {
            args = Some(
                rest.split_whitespace()
                    .filter(|part| !part.is_empty())
                    .map(str::to_string)
                    .collect(),
            );
        }
    }
    let command = command?;
    Some(Found {
        presence: classify(&command, &args.unwrap_or_default(), atmos),
        enabled: true,
    })
}

fn set_claude(root: Option<&Path>, atmos: &Path, on: bool, replace: bool) -> Result<()> {
    let record = if root.is_some() {
        Some(host::confine(root, CLAUDE_RECORD)?)
    } else {
        None
    };
    let found = read_claude(root, Some(atmos))?;
    let context = record
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "claude".into());
    if matches!(
        guard(&found, on, replace, Path::new(context.as_str()),)?,
        Guard::Skip
    ) {
        return Ok(());
    }
    let argv = if on {
        vec![
            "claude".into(),
            "mcp".into(),
            "add".into(),
            "--scope".into(),
            "user".into(),
            "omarchy".into(),
            "--".into(),
            atmos.display().to_string(),
            "mcp".into(),
        ]
    } else {
        vec![
            "claude".into(),
            "mcp".into(),
            "remove".into(),
            "--scope".into(),
            "user".into(),
            "omarchy".into(),
        ]
    };
    if let Some(root) = root {
        log_argv(root, &argv, on)?;
        let path = record.expect("fixture record");
        store::with_lock(&path, || {
            if on {
                let text = serde_json::to_string(&json!({
                    "command": atmos.display().to_string(),
                    "args": ["mcp"],
                }))?;
                atomic_write(&path, text.as_bytes())
            } else if path.exists() {
                fs::remove_file(&path).map_err(|err| Error::io(err, &path))
            } else {
                Ok(())
            }
        })?;
        return Ok(());
    }
    let mut run = Run::new("claude");
    for arg in argv.into_iter().skip(1) {
        run = run.arg(arg);
    }
    run.timeout(COMMAND_TIMEOUT).checked().map(|_| ())
}

fn log_argv(root: &Path, argv: &[String], on: bool) -> Result<()> {
    let line = serde_json::json!({
        "domain": "agents.mcp.claude",
        "argv": argv,
        "value": on,
    });
    let text = serde_json::to_string(&line)?;
    let path = root.join("commands.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| Error::io(err, &path))?;
    writeln!(file, "{text}").map_err(|err| Error::io(err, &path))
}

fn classify(command: &str, args: &[String], atmos: Option<&Path>) -> Presence {
    let args_ok = args.len() == 1 && args[0] == "mcp";
    if let Some(atmos) = atmos {
        if Path::new(command) == atmos && args_ok {
            return Presence::Ours;
        }
    }
    if points_at_atmos(command) {
        return Presence::Stale;
    }
    Presence::Custom
}

fn points_at_atmos(command: &str) -> bool {
    let name = Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command);
    name == "atmos" || name == "ratmos"
}

fn tool_names(stdout: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in stdout.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        let Some(tools) = value.pointer("/result/tools").and_then(Value::as_array) else {
            continue;
        };
        names = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .map(str::to_string)
            .collect();
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Kind;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn roster_ids_are_unique_and_findable() {
        let mut seen = std::collections::HashSet::new();
        for spec in ROSTER {
            assert!(seen.insert(spec.id), "duplicate agent {}", spec.id);
            assert_eq!(find_spec(spec.id).unwrap().id, spec.id);
        }
    }

    #[test]
    fn an_unknown_agent_is_a_bad_request() {
        for id in ["", "nope", "Claude"] {
            let err = find_spec(id).err().expect("unknown agent");
            assert_eq!(err.kind, Kind::BadRequest, "{id:?}");
        }
    }

    #[test]
    fn presence_names_are_the_wire_words() {
        assert_eq!(presence_name(Presence::Absent), "absent");
        assert_eq!(presence_name(Presence::Ours), "ours");
        assert_eq!(presence_name(Presence::Stale), "stale");
        assert_eq!(presence_name(Presence::Custom), "custom");
    }

    #[test]
    fn only_atmos_or_ratmos_by_file_name_point_at_atmos() {
        for command in ["atmos", "ratmos", "/usr/bin/atmos", "/opt/x/ratmos"] {
            assert!(points_at_atmos(command), "{command}");
        }
        for command in [
            "",
            "atmos-old",
            "/usr/bin/atmosphere",
            "node",
            "/atmos/node",
        ] {
            assert!(!points_at_atmos(command), "{command}");
        }
    }

    #[test]
    fn classify_separates_ours_stale_and_custom() {
        let ours = Path::new("/home/u/.local/bin/atmos");
        let mcp = args(&["mcp"]);
        assert!(matches!(
            classify("/home/u/.local/bin/atmos", &mcp, Some(ours)),
            Presence::Ours
        ));
        // Same binary, wrong arguments: the entry is out of date.
        assert!(matches!(
            classify("/home/u/.local/bin/atmos", &args(&["serve"]), Some(ours)),
            Presence::Stale
        ));
        assert!(matches!(
            classify("/home/u/.local/bin/atmos", &args(&["mcp", "x"]), Some(ours)),
            Presence::Stale
        ));
        // An atmos installed somewhere else is stale, not ours.
        assert!(matches!(
            classify("/opt/atmos", &mcp, Some(ours)),
            Presence::Stale
        ));
        assert!(matches!(
            classify("/usr/bin/other", &mcp, Some(ours)),
            Presence::Custom
        ));
        // With no known install, nothing can be ours.
        assert!(matches!(
            classify("/home/u/.local/bin/atmos", &mcp, None),
            Presence::Stale
        ));
    }

    #[test]
    fn claude_get_output_is_read_into_a_presence() {
        let ours = Path::new("/bin/atmos");
        let text = "omarchy:\n  Scope: User\n  Command: /bin/atmos\n  Args: mcp\n";
        let found = parse_claude_get(text, Some(ours)).expect("found");
        assert!(matches!(found.presence, Presence::Ours));
        assert!(found.enabled);
        let other = parse_claude_get("Command: node\nArgs: server.js --flag\n", Some(ours));
        assert!(matches!(other.expect("found").presence, Presence::Custom));
        // No Args line is an empty argument list.
        let bare = parse_claude_get("Command: /bin/atmos\n", Some(ours)).expect("found");
        assert!(matches!(bare.presence, Presence::Stale));
    }

    #[test]
    fn claude_get_output_without_a_command_is_not_an_entry() {
        assert!(parse_claude_get("", None).is_none());
        assert!(parse_claude_get("No MCP server found with name: omarchy\n", None).is_none());
        assert!(parse_claude_get("Args: mcp\n", None).is_none());
    }

    #[test]
    fn tool_names_come_from_the_last_tools_reply() {
        let out = concat!(
            "{\"id\":1,\"result\":{\"protocolVersion\":\"x\"}}\n",
            "not json at all\n",
            "{\"id\":2,\"result\":{\"tools\":[{\"name\":\"a\"},{\"name\":\"b\"},{\"nope\":1}]}}\n",
        );
        assert_eq!(tool_names(out), vec!["a", "b"]);
        let two = format!("{out}{{\"id\":3,\"result\":{{\"tools\":[{{\"name\":\"c\"}}]}}}}\n");
        assert_eq!(tool_names(&two), vec!["c"]);
    }

    #[test]
    fn tool_names_are_empty_for_output_without_tools() {
        assert!(tool_names("").is_empty());
        assert!(tool_names("garbage\n\n").is_empty());
        assert!(tool_names("{\"result\":{}}\n").is_empty());
        assert!(tool_names("{\"result\":{\"tools\":\"x\"}}\n").is_empty());
    }

    #[test]
    fn string_list_keeps_strings_and_drops_the_rest() {
        let doc: toml_edit::DocumentMut = "a = [\"x\", 1, \"y\"]\nb = \"s\"\n".parse().unwrap();
        assert_eq!(string_list(doc.get("a")), vec!["x", "y"]);
        assert!(string_list(doc.get("b")).is_empty());
        assert!(string_list(doc.get("missing")).is_empty());
        assert!(string_list(None).is_empty());
    }
}
