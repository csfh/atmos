//! The window on GPUI's test platform: nothing is opened on the desktop. A scripted server stands
//! in for `atmosd serve`, so the test says what the backend said and sees what the window drew.

mod common;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use atmos_ui::app::Atmos;
use atmos_ui::client::Client;
use atmos_ui::hubs;
use common::{Server, script};
use odk::gpui::{
    AppContext as _, Bounds, Entity, KeyUpEvent, Keystroke, Modifiers, Pixels, TestAppContext,
    VisualTestContext,
};
use odk::ui::test_support::open_view;
use serde_json::json;

/// The theme and foundation tokens are the process's, so the windows are tested one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

struct Window<'a> {
    _serial: std::sync::MutexGuard<'static, ()>,
    atmos: Entity<Atmos>,
    server: Server,
    cx: &'a mut VisualTestContext,
}

fn open<'a>(app: &'a mut TestAppContext, page: &str) -> Window<'a> {
    let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (transport, mut servers) = script(1);
    let server = servers.remove(0);
    let (client, notices) = Client::start(transport);
    let page = page.to_owned();
    let (atmos, cx) = open_view(app, move |window, cx| {
        cx.new(|cx| Atmos::new(client, notices, &page, window, cx))
    });
    Window {
        _serial: serial,
        atmos,
        server,
        cx,
    }
}

impl Window<'_> {
    fn page(&mut self) -> String {
        self.atmos.read_with(self.cx, |a, _| a.page().to_owned())
    }

    /// Where a part was drawn. GPUI keys the lookup by a `'static` name, so the few names a test
    /// asks for are leaked.
    fn bounds(&mut self, part: &str) -> Option<Bounds<Pixels>> {
        self.cx
            .debug_bounds(Box::leak(part.to_owned().into_boxed_str()))
    }

    fn drawn(&mut self, part: &str) -> bool {
        self.bounds(part).is_some()
    }

    fn press(&mut self, key: &str) {
        let keystroke = Keystroke::parse(key).unwrap();
        self.cx
            .update(|window, cx| window.dispatch_keystroke(keystroke.clone(), cx));
        self.cx.simulate_event(KeyUpEvent { keystroke });
        self.cx.run_until_parked();
    }

    /// Lets what the server said reach the window: it comes from another thread, so the test
    /// looks until `done` holds instead of for a fixed time.
    fn until(&mut self, what: &str, done: impl Fn(&mut Self) -> bool) {
        for _ in 0..500 {
            self.cx.run_until_parked();
            if done(self) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("the window never showed {what}");
    }
}

#[test]
fn it_opens_on_the_hub_it_is_given_and_on_the_first_when_it_is_given_none() {
    let mut app = TestAppContext::single();
    assert_eq!(open(&mut app, "monitor").page(), "monitor");
    let mut app = TestAppContext::single();
    assert_eq!(open(&mut app, "").page(), "home");
    let mut app = TestAppContext::single();
    assert_eq!(open(&mut app, "no-such-hub").page(), "home");
}

#[test]
fn the_sidebar_lists_every_hub() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home");
    for i in 0..hubs::HUBS.len() {
        assert!(
            w.drawn(&format!("nav/item-{i}")),
            "hub {i} is in the sidebar"
        );
    }
    assert!(
        !w.drawn(&format!("nav/item-{}", hubs::HUBS.len())),
        "and no more"
    );
}

#[test]
fn a_click_on_a_hub_shows_it() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home");
    let monitor = hubs::HUBS.iter().position(|h| h.id == "monitor").unwrap();
    let at = w.bounds(&format!("nav/item-{monitor}")).unwrap().center();
    w.cx.simulate_click(at, Modifiers::default());
    w.cx.run_until_parked();
    assert_eq!(w.page(), "monitor");
}

#[test]
fn the_keyboard_walks_the_sidebar_and_enter_opens_a_hub() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home");
    w.press("tab");
    w.press("down");
    assert_eq!(w.page(), "home", "moving focus never opens a page");
    w.press("enter");
    assert_eq!(w.page(), hubs::HUBS[1].id);
}

#[test]
fn a_missing_daemon_replaces_the_page_and_its_return_brings_the_page_back() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home");
    assert!(
        !w.drawn("backend-unavailable/alert"),
        "the backend is not said to be gone"
    );

    w.server.say(json!({"event": "backend", "state": "down",
        "why": "no daemon (refused)", "attempt": 1, "retryMs": 1000}));
    w.until("the unavailable notice", |w| {
        w.drawn("backend-unavailable/alert")
    });
    assert!(w.drawn("nav/item-0"), "the sidebar stays");

    w.server.say(json!({"event": "backend", "state": "up"}));
    w.until("the page again", |w| !w.drawn("backend-unavailable/alert"));
}

#[test]
fn a_stopped_server_is_the_same_notice() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home");
    let Window { server, .. } = &mut w;
    // The server's end closing is how a stopped `atmosd serve` looks.
    drop(std::mem::replace(server, script(1).1.remove(0)));
    w.until("the unavailable notice", |w| {
        w.drawn("backend-unavailable/alert")
    });
}
