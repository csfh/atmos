//! The Tweaks page on GPUI's test platform: nothing opens on the desktop. A scripted daemon answers
//! what the window asks and pushes the settings topics, so the test says what the backend said and
//! sees what the window drew and what it sent back.

mod common;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use atmos_ui::app::Atmos;
use atmos_ui::client::Client;
use atmos_ui::hubs;
use atmos_ui::pages::tweaks::TweaksPage;
use atmos_ui::tweaks::Reading;
use common::{Auto, Reply, script};
use odk::gpui::{
    AppContext as _, Bounds, Entity, Modifiers, Pixels, TestAppContext, VisualTestContext, px, size,
};
use odk::ui::test_support::open_view;
use serde_json::{Value, json};

/// The theme and foundation tokens are the process's, so the windows are tested one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

struct Win<'a> {
    _serial: std::sync::MutexGuard<'static, ()>,
    atmos: Entity<Atmos>,
    daemon: Auto,
    cx: &'a mut VisualTestContext,
}

/// What a daemon says to an ordinary request: the version, a held set, and a stored setting whose
/// read-back says nothing.
fn ordinary(request: &Value) -> Reply {
    match request["op"].as_str() {
        Some("version") => Reply::Ok(json!("0.4.0")),
        Some("sub.set") => Reply::Ok(json!({"holding": []})),
        Some("settings.set") => Reply::Ok(json!({
            "domain": request["domain"], "value": request["value"], "snapshot": {}})),
        _ => Reply::Ok(Value::Null),
    }
}

fn open<'a>(
    app: &'a mut TestAppContext,
    page: &str,
    answer: impl Fn(&Value) -> Reply + Send + 'static,
) -> Win<'a> {
    let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (transport, mut servers) = script(1);
    let daemon = servers.remove(0).auto(answer);
    let (client, notices) = Client::start(transport);
    let page = page.to_owned();
    let (atmos, cx) = open_view(app, move |_, cx| {
        cx.new(|cx| Atmos::new(client, notices, &page, cx))
    });
    cx.simulate_resize(size(px(1400.0), px(1400.0)));
    cx.run_until_parked();
    Win {
        _serial: serial,
        atmos,
        daemon,
        cx,
    }
}

impl Win<'_> {
    fn tweaks(&mut self) -> Entity<TweaksPage> {
        self.atmos.read_with(self.cx, |a, _| a.tweaks().clone())
    }

    fn reading(&mut self, id: &str) -> Reading {
        let page = self.tweaks();
        page.read_with(self.cx, |p, _| p.reading(id).expect("a tweak"))
    }

    /// Looks, and lets what other threads said reach the window, until `done` holds.
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

    /// Long enough for anything the window would send to reach the daemon.
    fn settle(&mut self) {
        thread::sleep(Duration::from_millis(100));
        self.cx.run_until_parked();
    }

    fn bounds(&mut self, part: &str) -> Option<Bounds<Pixels>> {
        self.cx
            .debug_bounds(Box::leak(part.to_owned().into_boxed_str()))
    }

    fn drawn(&mut self, part: &str) -> bool {
        self.bounds(part).is_some()
    }

    fn click(&mut self, part: &str) {
        let at = self
            .bounds(part)
            .unwrap_or_else(|| panic!("{part} is not drawn"))
            .center();
        self.cx.simulate_click(at, Modifiers::default());
        self.cx.run_until_parked();
    }

    /// The daemon pushes a sample of a settings topic, with nothing failed.
    fn sample(&mut self, topic: &str, at: u64, keys: Value) {
        let mut result = keys;
        if result.get("_failed").is_none() {
            result["_failed"] = json!({});
        }
        self.daemon.say(json!({
            "event": "sample", "kind": topic, "sampledAt": at, "tookMs": 1, "result": result,
        }));
    }

    /// Both topics, as a machine with flat acceleration off, natural scrolling on, middle-click
    /// paste on and nothing said of the two tweaks Omarchy turns on.
    fn usual(&mut self) {
        self.sample(
            "settings.look",
            10,
            json!({"hyprInput": {"accelProfile": "adaptive", "naturalScroll": true},
                "hyprInputManaged": true}),
        );
        self.sample(
            "settings.system",
            10,
            json!({"tweaks": {"middlePaste": false}}),
        );
        self.until("both topics taken in", |w| {
            matches!(w.reading("accelFlat"), Reading::On(_))
                && matches!(w.reading("middlePaste"), Reading::On(_))
        });
    }

    fn unread(&mut self) -> Vec<&'static str> {
        let page = self.tweaks();
        page.read_with(self.cx, |p, _| p.unread())
    }

    fn a_toast_is_shown(&mut self) -> bool {
        (1..64).any(|n| self.drawn(&format!("odk-toast/card-{n}")))
    }
}

#[test]
fn the_page_holds_its_two_settings_topics_only_while_it_is_shown() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "home", ordinary);
    let tweaks = hubs::HUBS.iter().position(|h| h.id == "tweaks").unwrap();
    w.until("the first hold", |w| !w.daemon.of("sub.set").is_empty());
    assert_eq!(w.daemon.of("sub.set")[0]["topics"], json!({}));

    w.click(&format!("nav/item-{tweaks}"));
    w.until("the topics asked for", |w| {
        w.daemon.of("sub.set").last().is_some_and(|r| {
            r["topics"]
                == json!({"settings.look": {"intervalMs": 0},
                    "settings.system": {"intervalMs": 0}})
        })
    });
    w.click("nav/item-0");
    w.until("the topics let go", |w| {
        w.daemon
            .of("sub.set")
            .last()
            .is_some_and(|r| r["topics"] == json!({}))
    });
    assert_eq!(w.daemon.of("sub.set").len(), 3);
}

#[test]
fn nothing_is_sent_until_a_toggle_or_a_reset_is_pressed() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "tweaks", ordinary);
    w.settle();
    w.usual();
    w.settle();
    // The Sentinel page reads its limits when the window opens; the Tweaks page reads nothing.
    let sent: Vec<Value> = w
        .daemon
        .requests()
        .into_iter()
        .filter(|r| {
            let sentinel = r["op"] == "settings.get"
                && r["domain"]
                    .as_str()
                    .is_some_and(|d| d.starts_with("sentinel"));
            !(r["op"] == "version" || r["op"] == "sub.set" || sentinel)
        })
        .collect();
    assert!(sent.is_empty(), "drawing the page sent {sent:?}");
}

#[test]
fn a_row_shows_what_the_snapshot_says_and_no_toggle_before_it_says() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "tweaks", ordinary);
    assert_eq!(w.reading("accelFlat"), Reading::Waiting);
    assert!(
        !w.drawn("tweak-accelFlat/track"),
        "no toggle with a made-up state"
    );
    assert!(
        w.drawn("reset-accelFlat/button"),
        "Reset is there all along"
    );

    w.usual();
    assert_eq!(
        w.reading("accelFlat"),
        Reading::On(false),
        "adaptive is not flat"
    );
    assert_eq!(w.reading("naturalScroll"), Reading::On(true));
    assert_eq!(w.reading("middlePaste"), Reading::On(false));
    assert_eq!(
        w.reading("electronWayland"),
        Reading::On(true),
        "on until the machine says false"
    );
    assert_eq!(w.reading("forceZeroScaling"), Reading::On(true));
    assert!(w.drawn("tweak-accelFlat/track"));

    // An older sample of a topic is not taken; a newer one is.
    w.sample(
        "settings.system",
        9,
        json!({"tweaks": {"electronWayland": false}}),
    );
    w.settle();
    assert_eq!(w.reading("electronWayland"), Reading::On(true));
    w.sample(
        "settings.system",
        11,
        json!({"tweaks": {"electronWayland": false}}),
    );
    w.until("the newer sample", |w| {
        w.reading("electronWayland") == Reading::On(false)
    });
}

#[test]
fn a_toggle_writes_the_opposite_of_what_is_shown_with_its_read_back() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "tweaks", ordinary);
    w.usual();

    w.click("tweak-electronWayland/track");
    w.until("the write", |w| !w.daemon.of("settings.set").is_empty());
    assert_eq!(
        w.daemon.of("settings.set")[0],
        json!({"op": "settings.set", "domain": "tweaks.electronWayland", "value": false,
            "readback": ["tweaks"], "id": w.daemon.of("settings.set")[0]["id"]})
    );

    w.click("tweak-accelFlat/track");
    w.until("the second write", |w| {
        w.daemon.of("settings.set").len() == 2
    });
    let sent = &w.daemon.of("settings.set")[1];
    assert_eq!(sent["domain"], "hyprInput.accelProfile");
    assert_eq!(sent["value"], "flat");
    assert_eq!(sent["readback"], json!(["hyprInput", "hyprInputManaged"]));

    w.click("tweak-naturalScroll/track");
    w.until("the third write", |w| {
        w.daemon.of("settings.set").len() == 3
    });
    assert_eq!(w.daemon.of("settings.set")[2]["value"], false);
}

#[test]
fn the_toggle_shows_what_the_backend_read_back_and_not_the_click() {
    let mut app = TestAppContext::single();
    // The machine takes middle-click paste, and keeps natural scrolling on whatever is written.
    let answer = |r: &Value| match r["domain"].as_str() {
        // It answers a moment later, as a machine does.
        Some("tweaks.middlePaste") => {
            thread::sleep(Duration::from_millis(300));
            Reply::Ok(json!({"domain": r["domain"], "value": true,
                "snapshot": {"tweaks": {"middlePaste": true}}}))
        }
        Some("hyprInput.naturalScroll") => Reply::Ok(json!({"domain": r["domain"], "value": true,
            "snapshot": {"hyprInput": {"accelProfile": "adaptive", "naturalScroll": true},
                "hyprInputManaged": true}})),
        _ => ordinary(r),
    };
    let mut w = open(&mut app, "tweaks", answer);
    w.usual();

    w.click("tweak-middlePaste/track");
    assert_eq!(
        w.reading("middlePaste"),
        Reading::On(false),
        "not before the answer"
    );
    w.until("the read-back shown", |w| {
        w.reading("middlePaste") == Reading::On(true)
    });

    w.click("tweak-naturalScroll/track");
    w.until("the write answered", |w| {
        w.daemon.of("settings.set").len() == 2
    });
    w.settle();
    assert_eq!(
        w.reading("naturalScroll"),
        Reading::On(true),
        "the machine kept it on, so it is shown on"
    );
}

#[test]
fn reset_writes_the_default_once() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "tweaks", ordinary);
    w.usual();

    w.click("reset-forceZeroScaling/button");
    w.until("the reset", |w| !w.daemon.of("settings.set").is_empty());
    w.settle();
    let sent = w.daemon.of("settings.set");
    assert_eq!(sent.len(), 1, "one write: {sent:?}");
    assert_eq!(sent[0]["domain"], "tweaks.forceZeroScaling");
    assert_eq!(
        sent[0]["value"], true,
        "back on, as Omarchy sets it, not the off value"
    );
    assert_eq!(sent[0]["readback"], json!(["tweaks"]));

    w.click("reset-accelFlat/button");
    w.until("the second reset", |w| {
        w.daemon.of("settings.set").len() == 2
    });
    assert_eq!(
        w.daemon.of("settings.set")[1]["value"],
        "",
        "no profile of its own: Hyprland's"
    );
    w.click("reset-naturalScroll/button");
    w.until("the third reset", |w| {
        w.daemon.of("settings.set").len() == 3
    });
    assert_eq!(w.daemon.of("settings.set")[2]["value"], false);
}

#[test]
fn a_refused_write_is_told_and_the_toggle_stays() {
    let mut app = TestAppContext::single();
    let answer = |r: &Value| {
        if r["op"] == "settings.set" {
            Reply::Err("failed", "the file is read-only")
        } else {
            ordinary(r)
        }
    };
    let mut w = open(&mut app, "tweaks", answer);
    w.usual();
    assert!(!w.a_toast_is_shown());
    w.click("tweak-middlePaste/track");
    w.until("the refusal told", |w| w.a_toast_is_shown());
    assert_eq!(w.reading("middlePaste"), Reading::On(false));
}

#[test]
fn a_key_that_could_not_be_read_has_no_toggle_and_says_why() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "tweaks", ordinary);
    w.sample(
        "settings.system",
        5,
        json!({"_failed": {"tweaks": "settings.ini is unreadable"}}),
    );
    w.daemon.say(
        json!({"event": "sample", "kind": "settings.look", "stale": true,
        "error": "hyprctl did not answer"}),
    );
    w.until("both failures told", |w| {
        w.drawn("tweaks-unread-tweaks/alert") && w.drawn("tweaks-unread-hyprInput/alert")
    });
    assert_eq!(w.unread(), ["hyprInput", "tweaks"]);
    assert_eq!(
        w.reading("middlePaste"),
        Reading::Unknown("settings.ini is unreadable".into())
    );
    assert_eq!(
        w.reading("accelFlat"),
        Reading::Unknown("hyprctl did not answer".into())
    );
    assert!(
        !w.drawn("tweak-electronWayland/track"),
        "not on by default when it could not be read"
    );

    // A topic read again brings the toggle back and the alert goes.
    w.sample(
        "settings.system",
        6,
        json!({"tweaks": {"electronWayland": false}}),
    );
    w.until("the toggle back", |w| {
        w.drawn("tweak-electronWayland/track")
    });
    // GPUI 0.2.2 keeps a part's bounds after it is gone (`Frame::clear` leaves `debug_bounds`),
    // so what is no longer said is asked of the page.
    assert_eq!(w.unread(), ["hyprInput"], "the other is still said");
}
