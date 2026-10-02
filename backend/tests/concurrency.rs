mod common;

use std::collections::BTreeMap;
use std::thread;

use common::{cleanup, request, temp_root};
use serde_json::{json, Value};

/// Several windows write different keys of one file at once. Every write must
/// survive: that is the flock + atomic rename promise in AGENTS.md.
#[test]
fn concurrent_writers_to_one_file_lose_nothing() {
    let root = temp_root();
    let listing = request(&root, "plain", &json!({"op": "settings.list"}));
    let mut by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in listing["result"].as_array().unwrap() {
        if item["type"] == "string" {
            by_file
                .entry(item["file"].as_str().unwrap().to_string())
                .or_default()
                .push(item["domain"].as_str().unwrap().to_string());
        }
    }
    let keys = by_file
        .values()
        .find(|keys| keys.len() >= 4)
        .expect("a plain file holding four string settings")
        .clone();

    let handles: Vec<_> = keys
        .iter()
        .cloned()
        .map(|key| {
            let root = root.clone();
            thread::spawn(move || {
                for round in 0..5 {
                    request(
                        &root,
                        "plain",
                        &json!({"op": "settings.set", "domain": key, "value": format!("{key}-{round}")}),
                    );
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }

    for key in &keys {
        let got = request(
            &root,
            "plain",
            &json!({"op": "settings.get", "domain": key}),
        );
        assert_eq!(
            got["result"],
            Value::String(format!("{key}-4")),
            "{key} lost a write"
        );
    }
    cleanup(&root);
}
