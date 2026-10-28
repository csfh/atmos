//! Search in the window finds what the QML window's search finds on the ported hubs
//! (docs/adr/0020): the real `atmosd` reads the QML pages as `search.query` does, and every row it
//! finds on a ported hub, and every hub, must be in the window's catalog with the same words. The
//! window may find more: rows whose labels the QML builds at run time are not in the QML catalog.
//! Without a built `atmosd` or `node` the test says so and stops.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use atmos_ui::app::PORTED;
use atmos_ui::client::locate;
use atmos_ui::search::{self, Row};
use serde_json::{Value, json};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn atmosd() -> Option<PathBuf> {
    let debug = repo().join("backend/target/debug/atmosd");
    let explicit = std::env::var_os("ATMOS_BACKEND").map_or(debug, PathBuf::from);
    let found = locate(Some(explicit), None);
    found.is_absolute().then_some(found)
}

/// The `search.query` body the QML window sends for an empty query: every row.
fn body() -> Option<Value> {
    const NODE: &str = r#"
const root = process.env.REPO_DIR;
const R = require(root + "/tests/harness.js").load("services/Requests.js");
const H = require(root + "/services/Hubs.js");
process.stdout.write(JSON.stringify(R.searchQuery("", H.searchContext())));
"#;
    let out = Command::new("node")
        .args(["-e", NODE])
        .env("REPO_DIR", repo())
        .output()
        .ok()?;
    assert!(out.status.success(), "Requests.js failed under node");
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

fn qml_catalog(atmosd: &Path, body: &Value) -> Vec<Value> {
    let scratch = std::env::temp_dir().join(format!("atmos-ui-search-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut child = Command::new(atmosd)
        .arg("request")
        .env("ATMOS_ROOT", repo())
        .env("ATMOS_NO_AUTOSTART", "1")
        .env("HOME", &scratch)
        .env("XDG_RUNTIME_DIR", &scratch)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let _ = std::fs::remove_dir_all(&scratch);
    let reply: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(reply["ok"], true, "{reply}");
    reply["result"]["hits"].as_array().unwrap().clone()
}

fn words(row: &Row) -> Value {
    json!([row.label, row.description, row.hint, row.keywords])
}

#[test]
fn the_window_finds_every_row_the_qml_search_finds_on_a_ported_hub() {
    let Some(bin) = atmosd() else {
        eprintln!("skip: no built atmosd (cargo build --manifest-path backend/Cargo.toml)");
        return;
    };
    let Some(body) = body() else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours = search::catalog();
    for hit in qml_catalog(&bin, &body) {
        let hub = hit["hub"].as_str().unwrap();
        let id = hit["id"].as_str().unwrap();
        let label = hit["label"].as_str().unwrap();
        if id == hub {
            // The hub itself, whichever hubs are ported.
            let row = ours.iter().find(|r| r.hub == hub && r.label == label);
            let row = row.unwrap_or_else(|| panic!("hub {hub} is not in the catalog"));
            assert_eq!(
                json!([row.label, row.description, row.keywords]),
                json!([hit["label"], hit["description"], hit["keywords"]]),
                "hub {hub}"
            );
            continue;
        }
        let root = hub.split('/').next().unwrap();
        if !PORTED.contains(&root) {
            continue;
        }
        if id.contains("-PrefsGroup-") {
            // `search.rs` takes a group's description, hint and keywords from the first row inside
            // it; its own words are its title and detail.
            assert!(
                ours.iter().any(|r| r.hub == hub
                    && r.label == label
                    && r.detail == hit["detail"].as_str().unwrap()),
                "the group {label:?} on {hub}: {hit}"
            );
            continue;
        }
        let want = json!([
            hit["label"],
            hit["description"],
            hit["hint"],
            hit["keywords"]
        ]);
        assert!(
            ours.iter().any(|r| r.hub == hub && words(r) == want),
            "the row {label:?} on {hub} with the QML's words: {want}"
        );
    }
}
