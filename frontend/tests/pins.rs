//! What the window copies from the app it replaces is pinned to its source, so a change on one
//! side fails here instead of showing up as a missing page or a `bad_request`.

use atmos_ui::{hubs, protocol};
use std::fs;
use std::path::PathBuf;

fn repo(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn every_op_is_one_the_backend_answers() {
    let request = repo("backend/src/request.rs");
    let serve = repo("backend/src/serve.rs");
    for op in protocol::OPS {
        let variant = format!("#[serde(rename = \"{op}\")]");
        // `serve` answers some ops itself: as a comparison or as an arm of its `match`.
        let served = [format!("Some(\"{op}\")"), format!("\"{op}\" =>")];
        assert!(
            request.contains(&variant) || served.iter().any(|s| serve.contains(s)),
            "`{op}` is neither a Request variant nor an op serve answers"
        );
    }
}

#[test]
fn the_hubs_are_the_launchers_in_the_same_order() {
    let launch = repo("backend/src/launch.rs");
    let list = launch
        .split("const HUBS: &[&str] = &[")
        .nth(1)
        .and_then(|rest| rest.split("];").next())
        .expect("launch.rs has a HUBS list");
    let launcher: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
    let ours: Vec<&str> = hubs::HUBS.iter().map(|h| h.id).collect();
    assert_eq!(ours, launcher);
}

/// The value of `key: "..."` on a line, if it has that key.
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.trim()
        .strip_prefix(key)?
        .strip_prefix(": \"")?
        .split('"')
        .next()
}

#[test]
fn titles_and_groups_are_those_of_hubs_js() {
    let source = repo("services/Hubs.js");
    // A hub's own fields sit at six spaces; its children's are deeper.
    let own = |line: &&str| line.starts_with("      ") && !line.starts_with("       ");
    let mut found: Vec<(String, String, String)> = Vec::new();
    for line in source.lines().filter(own) {
        if let Some(id) = field(line, "id") {
            found.push((id.into(), String::new(), String::new()));
        } else if let (Some(title), Some(last)) = (field(line, "title"), found.last_mut()) {
            last.1 = title.into();
        } else if let (Some(group), Some(last)) = (field(line, "navGroup"), found.last_mut()) {
            last.2 = group.into();
        }
    }
    let ours: Vec<_> = hubs::HUBS
        .iter()
        .map(|h| (h.id.to_string(), h.title.to_string(), h.group.to_string()))
        .collect();
    assert_eq!(ours, found);
}

#[test]
fn group_headings_are_those_of_layout_js() {
    let source = repo("services/Layout.js");
    let table = source
        .split("var NAV_GROUP_LABELS = {")
        .nth(1)
        .and_then(|rest| rest.split("};").next())
        .expect("Layout.js has NAV_GROUP_LABELS");
    let mut theirs: Vec<(String, String)> = vec![("home".into(), String::new())];
    for line in table.lines() {
        let line = line.trim().trim_end_matches(',');
        if let Some((key, label)) = line.split_once(": \"") {
            theirs.push((key.into(), label.trim_end_matches('"').into()));
        }
    }
    let ours: Vec<_> = hubs::GROUPS
        .iter()
        .map(|(id, label)| (id.to_string(), label.to_string()))
        .collect();
    assert_eq!(ours, theirs);
}
