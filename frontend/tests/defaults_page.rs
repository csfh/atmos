//! The Defaults page on GPUI's test platform (`common/window.rs`): the topics it holds, a picker
//! that waits for its key, the programs it offers, and the write a choice sends.

mod common;

use atmos_ui::hubs;
use common::Reply;
use common::window::{Win, open, ordinary};
use odk::gpui::{Entity, TestAppContext};
use serde_json::{Value, json};

use atmos_ui::pages::defaults::DefaultsPage;

fn page(w: &mut Win) -> Entity<DefaultsPage> {
    w.atmos.read_with(w.cx, |a, _| a.defaults().clone())
}

fn current(w: &mut Win, id: &str) -> Option<String> {
    let page = page(w);
    page.read_with(w.cx, |p, cx| p.current(id, cx))
}

fn offered(w: &mut Win, id: &str) -> Vec<String> {
    let page = page(w);
    page.read_with(w.cx, |p, cx| p.offered(id, cx))
}

fn unread(w: &mut Win) -> Vec<&'static str> {
    let page = page(w);
    page.read_with(w.cx, |p, cx| p.unread(cx))
}

/// Both topics, as a machine with Chromium the default, Chrome and Zen not installed, Zed the
/// editor, and Evince the only PDF handler.
fn usual(w: &mut Win) {
    w.sample(
        "settings.system",
        10,
        json!({"browser": "chromium", "terminal": "ghostty", "editor": "zeditor",
            "mimePdf": "org.gnome.Evince.desktop", "mimeImage": "", "mimeVideo": "",
            "mimePdfOptions": [{"value": "org.gnome.Evince.desktop", "label": "Evince"}],
            "mimeImageOptions": [], "mimeVideoOptions": []}),
    );
    w.sample(
        "settings.host",
        10,
        json!({"browsers": {"chromium": true, "chrome": false, "zen": false},
            "terminals": {}, "editors": {"zeditor": true}}),
    );
    w.until("both topics taken in", |w| {
        current(w, "browser").is_some() && offered(w, "browser").len() == 5
    });
}

#[test]
fn the_page_holds_its_two_settings_topics_only_while_it_is_shown() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home", ordinary);
    let defaults = hubs::HUBS.iter().position(|h| h.id == "defaults").unwrap();
    w.until("the first hold", |w| !w.daemon.of("sub.set").is_empty());

    w.click(&format!("nav/item-{defaults}"));
    w.until("the topics asked for", |w| {
        w.daemon.of("sub.set").last().is_some_and(|r| {
            r["topics"]
                == json!({"settings.system": {"intervalMs": 0},
                    "settings.host": {"intervalMs": 0}})
        })
    });
    w.click("nav/item-0");
    w.until("the topics let go", |w| {
        w.daemon
            .of("sub.set")
            .last()
            .is_some_and(|r| r["topics"] == json!({}))
    });
}

#[test]
fn a_picker_waits_for_its_key_and_then_offers_what_is_installed() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "defaults", ordinary);
    assert_eq!(current(&mut w, "browser"), None);
    assert!(
        !w.drawn("default-browser/trigger"),
        "no picker with a made-up choice"
    );

    usual(&mut w);
    assert!(w.drawn("default-browser/trigger"));
    assert_eq!(
        current(&mut w, "editor").as_deref(),
        Some("zed"),
        "Zed by the picker's name"
    );
    assert_eq!(
        offered(&mut w, "browser"),
        ["chromium", "brave", "brave-origin", "edge", "firefox"]
    );
    assert!(
        w.drawn("default-video/trigger"),
        "a picker with nothing to offer is drawn"
    );
    w.settle();
    assert!(
        w.sent_by_pages().is_empty(),
        "drawing the page sent {:?}",
        w.sent_by_pages()
    );
}

#[test]
fn a_choice_writes_the_default_and_shows_what_the_backend_read_back() {
    let mut app = TestAppContext::single();
    let mut w = open(
        &mut app,
        "defaults",
        |request: &Value| match request["op"].as_str() {
            Some("settings.set") => Reply::Ok(json!({
            "domain": request["domain"], "value": request["value"],
            "snapshot": {"browser": request["value"]}})),
            _ => ordinary(request),
        },
    );
    usual(&mut w);

    w.click("default-browser/trigger");
    // Firefox is the fifth of the five installed browsers.
    w.click("default-browser/row-4");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    assert_eq!(
        w.daemon.of("settings.set")[0],
        json!({"id": w.daemon.of("settings.set")[0]["id"], "op": "settings.set",
            "domain": "browser", "value": "firefox", "readback": ["browser"]})
    );
    w.until("the read-back shown", |w| {
        current(w, "browser").as_deref() == Some("firefox")
    });

    // Choosing what is shown sends nothing.
    w.click("default-browser/trigger");
    w.click("default-browser/row-4");
    w.settle();
    assert_eq!(w.daemon.of("settings.set").len(), 1);
}

#[test]
fn a_refused_choice_keeps_what_is_shown_and_says_so() {
    let mut app = TestAppContext::single();
    let mut w = open(
        &mut app,
        "defaults",
        |request: &Value| match request["op"].as_str() {
            Some("settings.set") => Reply::Err("failed", "omarchy default browser failed"),
            _ => ordinary(request),
        },
    );
    usual(&mut w);
    w.click("default-browser/trigger");
    w.click("default-browser/row-4");
    w.until("the refusal told", |w| w.a_toast_is_shown());
    assert_eq!(current(&mut w, "browser").as_deref(), Some("chromium"));
}

#[test]
fn a_key_that_could_not_be_read_has_no_picker_and_says_why() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "defaults", ordinary);
    w.sample(
        "settings.system",
        5,
        json!({"_failed": {"browser": "omarchy default browser failed"}}),
    );
    w.sample_failed("settings.host", "the host topic timed out");
    w.until("the failures told", |w| {
        w.drawn("defaults-unread-browser/alert") && w.drawn("defaults-unread-editors/alert")
    });
    assert!(!w.drawn("default-browser/trigger"));
    assert_eq!(
        unread(&mut w),
        ["browser", "browsers", "terminals", "editors"]
    );
}
