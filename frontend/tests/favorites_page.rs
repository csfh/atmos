//! Stars and the Favorites page on GPUI's test platform (`common/window.rs`): a star waits for the
//! list, writes the whole list with the row in it, and lights when the backend reads it back; the
//! Favorites page groups the list by hub and Open… shows the hub.

mod common;

use common::Reply;
use common::window::{Win, open, ordinary};
use odk::gpui::TestAppContext;
use serde_json::{Value, json};

fn groups(w: &mut Win) -> Vec<(String, Vec<String>)> {
    let page = w.atmos.read_with(w.cx, |a, _| a.favorites().clone());
    page.read_with(w.cx, |p, cx| p.groups(cx))
}

/// A daemon that keeps the list a write gives it and reads it back.
fn keeps_the_list(request: &Value) -> Reply {
    if request["op"] == "write" && request["name"] == "favorites.write" {
        let doc: Value = serde_json::from_str(request["args"]["json"].as_str().unwrap()).unwrap();
        return Reply::Ok(json!({"snapshot": {"favorites": doc["items"]}}));
    }
    ordinary(request)
}

const BROWSER: &str = "star-defaults/browser/button";

#[test]
fn a_star_waits_for_the_list_then_writes_it_with_the_row_and_lights_on_the_read_back() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "defaults", keeps_the_list);
    w.sample("settings.system", 1, json!({"browser": "chromium"}));
    w.until("the picker", |w| w.drawn("default-browser/trigger"));
    w.click(BROWSER);
    w.settle();
    assert!(
        w.daemon.of("favorites.write").is_empty(),
        "no list, no write"
    );

    w.sample(
        "settings.host",
        1,
        json!({"favorites": [{"hub": "tweaks", "label": "Natural scrolling"}]}),
    );
    w.settle();
    w.click(BROWSER);
    w.until("the write", |w| !w.daemon.of("favorites.write").is_empty());
    let sent = &w.daemon.of("favorites.write")[0];
    let doc: Value = serde_json::from_str(sent["args"]["json"].as_str().unwrap()).unwrap();
    let ids: Vec<_> = doc["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].clone())
        .collect();
    assert_eq!(
        ids,
        [json!("tweaks/natural-scrolling"), json!("defaults/browser")]
    );
    assert_eq!(doc["items"][1]["hubTitle"], "Defaults");

    // The read-back is what the page shows: the next press takes the row out again.
    w.until("the list read back", |w| {
        w.daemon.of("favorites.write").len() == 1 && {
            let page = w.atmos.read_with(w.cx, |a, _| a.favorites().clone());
            page.read_with(w.cx, |p, cx| p.groups(cx)).len() == 2
        }
    });
    w.click(BROWSER);
    w.until("the second write", |w| {
        w.daemon.of("favorites.write").len() == 2
    });
    let sent = &w.daemon.of("favorites.write")[1];
    let doc: Value = serde_json::from_str(sent["args"]["json"].as_str().unwrap()).unwrap();
    assert_eq!(doc["items"].as_array().unwrap().len(), 1);
}

#[test]
fn the_favorites_page_groups_by_hub_and_open_shows_the_page_of_the_row() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "favorites", ordinary);
    w.sample(
        "settings.host",
        1,
        json!({"favorites": [
            {"hub": "defaults", "hubTitle": "Defaults", "label": "Browser", "description": "Links"},
            {"hub": "nothing", "label": "Gone"},
            {"hub": "system/kernel", "hubTitle": "Kernel", "label": "Direct boot"},
            {"hub": "defaults", "hubTitle": "Defaults", "label": "Editor"},
        ]}),
    );
    w.until("the groups", |w| !groups(w).is_empty());
    assert_eq!(
        groups(&mut w),
        [
            (
                "Defaults".to_owned(),
                vec!["Browser".to_owned(), "Editor".to_owned()]
            ),
            ("Kernel".to_owned(), vec!["Direct boot".to_owned()]),
        ]
    );
    w.click("open-system/kernel/direct-boot/button");
    w.until("the page", |w| {
        w.atmos.read_with(w.cx, |a, _| a.page() == "system/kernel")
    });
}

#[test]
fn an_empty_list_says_how_to_star_and_an_unread_one_says_why() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "favorites", ordinary);
    w.sample(
        "settings.host",
        1,
        json!({"_failed": {"favorites": "favorites.json is not a list"}}),
    );
    w.until("the failure told", |w| w.drawn("favorites-unread/alert"));
    w.sample("settings.host", 2, json!({"favorites": []}));
    w.until("the list read", |w| {
        let page = w.atmos.read_with(w.cx, |a, _| a.favorites().clone());
        w.cx.read(|cx| page.read(cx).groups(cx).is_empty())
    });
}
