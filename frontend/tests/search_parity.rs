//! Search in the window finds what the QML window's search finds on the ported hubs
//! (docs/adr/0020): the real `atmosd` reads the QML pages as `search.query` does, and every row it
//! finds on a ported hub, and every hub, must be in the window's catalog with the same words. The
//! window may find more: rows whose labels the QML builds at run time are not in the QML catalog.
//! Without a built `atmosd` or `node` the test says so and stops.

mod common;

use atmos_ui::app::PORTED;
use atmos_ui::search::{self, Row};
use serde_json::{Value, json};

fn words(row: &Row) -> Value {
    json!([row.label, row.description, row.hint, row.keywords])
}

#[test]
fn the_window_finds_every_row_the_qml_search_finds_on_a_ported_hub() {
    let Some(hits) = common::qml::hits() else {
        return;
    };
    let ours = search::catalog();
    for hit in hits {
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
