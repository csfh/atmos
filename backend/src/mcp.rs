//! `atmosd mcp`: Atmos for agents, as newline-delimited JSON-RPC over stdio.
//! Each tool call is one or two backend requests, answered in-process.

use crate::platform::Backend;
use crate::{error_envelope, handle};
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::path::Path;

const SERVER_NAME: &str = "atmos";
const DEFAULT_PROTOCOL: &str = "2024-11-05";
const KNOWN_PROTOCOLS: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18"];

/// Settings that grant access to the machine. Readable, never writable by an agent.
const TRUST: &[&str] = &[
    "sshdEnabled",
    "passwordlessSudo",
    "sudolessDocker",
    "fingerprintConfigured",
    "fido2Configured",
    "directBoot",
];

fn tool(name: &str, description: &str, schema: Value) -> Value {
    json!({"name": name, "description": description, "inputSchema": schema})
}

pub fn tools() -> Vec<Value> {
    let empty = || json!({"type": "object", "properties": {}});
    vec![
        tool(
            "list",
            "List every settings domain, its type, and whether Atmos tools may change it. Trust domains are readable and tagged writable false.",
            empty(),
        ),
        tool(
            "snapshot",
            "Read the current settings. group is look, rest, all, network, disks, accounts, or system. Omit group for all. keys limits the top-level names.",
            json!({"type": "object", "properties": {"group": {"type": "string"}, "keys": {"type": "array", "items": {"type": "string"}}}}),
        ),
        tool(
            "get",
            "Read one settings domain.",
            json!({"type": "object", "properties": {"domain": {"type": "string"}}, "required": ["domain"]}),
        ),
        tool(
            "set",
            "Change one settings domain through the same writers the Atmos window uses. Trust domains (sshdEnabled, passwordlessSudo, sudolessDocker, fingerprintConfigured, fido2Configured, directBoot) are refused. The result includes the previous value.",
            json!({"type": "object", "properties": {"domain": {"type": "string"}, "value": {}}, "required": ["domain", "value"]}),
        ),
        tool("displays", "Read the connected monitors.", empty()),
        tool("sample", "Read one live sample of CPU, memory, network, temperatures, and processes.", empty()),
        tool("diagnostics", "Read the diagnostics inventory.", empty()),
    ]
}

fn rpc_result(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn rpc_error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn refused(message: &str) -> Value {
    json!({"ok": false, "error": {"code": "denied", "message": message}})
}

fn to_result(envelope: &Value, previous: Option<Value>) -> Value {
    let ok = envelope["ok"] == Value::Bool(true);
    let payload = if ok {
        let result = envelope["result"].clone();
        match (previous, result) {
            (Some(prev), Value::Object(map)) => {
                let mut merged = Map::new();
                merged.insert("previous".into(), prev);
                merged.extend(map);
                Value::Object(merged)
            }
            (Some(prev), other) => json!({"previous": prev, "result": other}),
            (None, other) => other,
        }
    } else {
        let error = envelope
            .get("error")
            .cloned()
            .unwrap_or_else(|| json!({"message": "The backend failed"}));
        json!({"error": error})
    };
    let mut out = json!({"content": [{"type": "text", "text": serde_json::to_string(&payload).unwrap_or_default()}]});
    if !ok {
        out["isError"] = Value::Bool(true);
    }
    out
}

fn tag_writable(envelope: Value) -> Value {
    let Some(rows) = envelope["result"]
        .as_array()
        .filter(|_| envelope["ok"] == Value::Bool(true))
    else {
        return envelope;
    };
    let tagged: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut copy = row.clone();
            let domain = row["domain"].as_str().unwrap_or("");
            copy["writable"] = Value::Bool(!TRUST.contains(&domain));
            copy
        })
        .collect();
    json!({"ok": true, "result": tagged})
}

fn call_tool(backend: Backend, root: Option<&Path>, name: &str, args: &Value) -> Value {
    let run = |body: Value| -> Value {
        handle(backend, root, &body).unwrap_or_else(|err| error_envelope(backend, &err))
    };
    let text = |key: &str| {
        args.get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    match name {
        "list" => to_result(&tag_writable(run(json!({"op": "settings.list"}))), None),
        "snapshot" => {
            let mut body = json!({"op": "settings.snapshot", "group": text("group").unwrap_or_else(|| "all".into())});
            if let Some(keys) = args.get("keys").and_then(Value::as_array) {
                body["keys"] = Value::Array(
                    keys.iter()
                        .map(|k| {
                            Value::String(k.as_str().map_or_else(|| k.to_string(), String::from))
                        })
                        .collect(),
                );
            }
            to_result(&run(body), None)
        }
        "get" => match text("domain") {
            Some(domain) => to_result(&run(json!({"op": "settings.get", "domain": domain})), None),
            None => to_result(&refused("domain is required"), None),
        },
        "set" => {
            let Some(domain) = text("domain") else {
                return to_result(&refused("domain is required"), None);
            };
            let Some(value) = args.get("value") else {
                return to_result(&refused("value is required"), None);
            };
            if TRUST.contains(&domain.as_str()) {
                return to_result(&refused(&format!("{domain} is refused")), None);
            }
            let got = run(json!({"op": "settings.get", "domain": domain}));
            let previous = if got["ok"] == Value::Bool(true) {
                got["result"].clone()
            } else {
                Value::Null
            };
            to_result(
                &run(json!({"op": "settings.set", "domain": domain, "value": value})),
                Some(previous),
            )
        }
        "displays" => to_result(&run(json!({"op": "display.snapshot"})), None),
        "sample" => to_result(&run(json!({"op": "display.get", "kind": "live"})), None),
        "diagnostics" => to_result(
            &run(json!({"op": "display.get", "kind": "diagnostics"})),
            None,
        ),
        other => to_result(&refused(&format!("{other} is not a tool")), None),
    }
}

/// One JSON-RPC line in, at most one response out.
pub fn handle_line(backend: Backend, root: Option<&Path>, line: &str) -> Option<Value> {
    let text = line.trim();
    if text.is_empty() {
        return None;
    }
    let Ok(Value::Object(msg)) = serde_json::from_str::<Value>(text) else {
        return Some(rpc_error(&Value::Null, -32700, "Parse error"));
    };
    let id = msg.get("id");
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    if method.is_empty() {
        return Some(rpc_error(
            id.unwrap_or(&Value::Null),
            -32600,
            "Invalid request",
        ));
    }
    let id = id?;
    if method.starts_with("notifications/") {
        return None;
    }
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => {
            let requested = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or("");
            let version = if KNOWN_PROTOCOLS.contains(&requested) {
                requested
            } else {
                DEFAULT_PROTOCOL
            };
            rpc_result(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": SERVER_NAME, "version": crate::VERSION},
                }),
            )
        }
        "ping" => rpc_result(id, json!({})),
        "tools/list" => rpc_result(id, json!({"tools": tools()})),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params
                .get("arguments")
                .filter(|a| a.is_object())
                .cloned()
                .unwrap_or_else(|| json!({}));
            rpc_result(id, call_tool(backend, root, name, &args))
        }
        _ => rpc_error(id, -32601, "Method not found"),
    })
}

pub fn serve(backend: Backend, root: Option<&Path>, input: impl BufRead, mut out: impl Write) {
    for line in input.lines() {
        let Ok(line) = line else {
            break;
        };
        if let Some(response) = handle_line(backend, root, &line) {
            let _ = writeln!(out, "{response}");
            let _ = out.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(line: &str) -> Value {
        handle_line(Backend::Plain, None, line).expect("a response")
    }

    #[test]
    fn initialize_negotiates_a_known_protocol() {
        let r = ask(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
        );
        assert_eq!(r["result"]["protocolVersion"], "2025-03-26");
        let r = ask(
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"1999"}}"#,
        );
        assert_eq!(r["result"]["protocolVersion"], DEFAULT_PROTOCOL);
    }

    #[test]
    fn notifications_and_blank_lines_get_no_reply() {
        assert!(handle_line(
            Backend::Plain,
            None,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
        )
        .is_none());
        assert!(handle_line(Backend::Plain, None, "  ").is_none());
        assert_eq!(ask("not json")["error"]["code"], -32700);
        assert_eq!(ask(r#"{"id":3,"method":"nope"}"#)["error"]["code"], -32601);
    }

    #[test]
    fn trust_domains_are_refused_and_tools_are_listed() {
        let r = ask(
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"set","arguments":{"domain":"sshdEnabled","value":true}}}"#,
        );
        assert_eq!(r["result"]["isError"], true);
        assert!(r["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("sshdEnabled is refused"));
        let list = ask(r#"{"jsonrpc":"2.0","id":5,"method":"tools/list"}"#);
        assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 7);
    }
}
