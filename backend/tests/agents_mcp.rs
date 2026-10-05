mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serde_json::Value;

use common::{cleanup, request, request_error, temp_root};

fn atmos_bin(root: &Path) {
    let path = root.join(".local/bin/atmos");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "#!/bin/bash\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
}

fn manifest(root: &Path) {
    let path = root.join(".local/state/omarchy/agents-mcp.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        r#"{"installed":["grok","claude","cursor-agent","pi"]}"#,
    )
    .unwrap();
}

fn row<'a>(list: &'a Value, id: &str) -> &'a Value {
    list["result"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("missing {id}"))
}

#[test]
fn list_marks_installed_agents_from_the_fixture() {
    let root = temp_root();
    atmos_bin(&root);
    manifest(&root);
    let listed = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.list"}),
    );
    let ids: Vec<&str> = listed["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec![
            "pi",
            "omp",
            "opencode",
            "claude",
            "codex",
            "grok",
            "gemini",
            "openclaw",
            "hermes",
            "copilot",
            "crush",
            "cursor-agent",
            "muse",
        ]
    );
    assert_eq!(row(&listed, "grok")["installed"], true);
    assert_eq!(row(&listed, "grok")["writer"], true);
    assert_eq!(row(&listed, "grok")["state"], "absent");
    assert_eq!(row(&listed, "pi")["installed"], true);
    assert_eq!(row(&listed, "pi")["writer"], false);
    assert_eq!(row(&listed, "codex")["installed"], false);
    let refused = request_error(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "pi", "on": true}).to_string(),
    );
    assert_eq!(refused["error"]["code"], "denied");
    cleanup(&root);
}

#[test]
fn grok_toml_keeps_a_sibling_server_and_a_second_set_is_a_noop() {
    let root = temp_root();
    atmos_bin(&root);
    let path = root.join(".grok/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "# keep me\n[mcp_servers.other]\ncommand = \"other\"\n",
    )
    .unwrap();
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "grok", "on": true}),
    );
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("# keep me"), "{text}");
    assert!(text.contains("other"), "{text}");
    assert!(text.contains(".local/bin/atmos"), "{text}");
    let again = text.clone();
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "grok", "on": true}),
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), again);
    let listed = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.list"}),
    );
    assert_eq!(row(&listed, "grok")["state"], "ours");
    assert_eq!(row(&listed, "grok")["on"], true);
    cleanup(&root);
}

#[test]
fn a_custom_entry_is_refused_until_replace_and_remove_deletes_only_omarchy() {
    let root = temp_root();
    atmos_bin(&root);
    let path = root.join(".grok/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original = "[mcp_servers.other]\ncommand = \"other\"\n\n[mcp_servers.omarchy]\ncommand = \"/usr/bin/custom-mcp\"\nargs = [\"serve\"]\n";
    fs::write(&path, original).unwrap();
    let refused = request_error(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "grok", "on": true}).to_string(),
    );
    assert_eq!(refused["error"]["code"], "denied");
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "grok", "on": true, "replace": true}),
    );
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains(".local/bin/atmos"));
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "grok", "on": false}),
    );
    let text = fs::read_to_string(&path).unwrap();
    assert!(!text.contains("omarchy"), "{text}");
    assert!(text.contains("other"), "{text}");
    cleanup(&root);
}

#[test]
fn cursor_json_keeps_sibling_keys() {
    let root = temp_root();
    atmos_bin(&root);
    let path = root.join(".cursor/mcp.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"mcpServers":{"other":{"command":"other"}},"keep":true}"#,
    )
    .unwrap();
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "cursor-agent", "on": true}),
    );
    let value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(value["keep"], true);
    assert_eq!(value["mcpServers"]["other"]["command"], "other");
    assert!(value["mcpServers"]["omarchy"]["command"]
        .as_str()
        .unwrap()
        .ends_with(".local/bin/atmos"));
    assert_eq!(value["mcpServers"]["omarchy"]["args"][0], "mcp");
    cleanup(&root);
}

#[test]
fn claude_under_a_fixture_logs_argv_and_does_not_spawn() {
    let root = temp_root();
    atmos_bin(&root);
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.set", "agent": "claude", "on": true}),
    );
    let log = fs::read_to_string(root.join("commands.log")).unwrap();
    let line: Value = serde_json::from_str(log.lines().next().unwrap()).unwrap();
    let argv: Vec<&str> = line["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| part.as_str().unwrap())
        .collect();
    assert_eq!(
        argv,
        vec!["claude", "mcp", "add", "--scope", "user", "omarchy", "--", argv[7], "mcp",]
    );
    assert!(argv[7].ends_with(".local/bin/atmos"), "{}", argv[7]);
    let listed = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.list"}),
    );
    assert_eq!(row(&listed, "claude")["state"], "ours");
    cleanup(&root);
}

#[test]
fn check_reads_tool_names_from_the_atmos_command() {
    let root = temp_root();
    let path = root.join(".local/bin/atmos");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        "#!/bin/bash\ncat >/dev/null\nprintf '%s\\n' '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[{\"name\":\"list\"},{\"name\":\"snapshot\"}]}}'\n",
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    let checked = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "agents.mcp.check"}),
    );
    assert_eq!(checked["result"]["tools"][0], "list");
    assert_eq!(checked["result"]["tools"][1], "snapshot");
    cleanup(&root);
}
