//! A window on GPUI's test platform, for the page tests: nothing opens on the desktop. A scripted
//! daemon answers what the window asks and pushes what the test says, so a test says what the
//! backend said and sees what the window drew and what it sent back.

use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use atmos_ui::app::Atmos;
use atmos_ui::client::Client;
use odk::gpui::{
    AppContext as _, Bounds, Entity, Modifiers, Pixels, TestAppContext, VisualTestContext, px, size,
};
use odk::ui::test_support::open_view;
use serde_json::{Value, json};

use super::{Auto, Reply, script};

/// The theme and foundation tokens are the process's, so the windows are tested one at a time.
static SERIAL: Mutex<()> = Mutex::new(());

pub struct Win<'a> {
    _serial: MutexGuard<'static, ()>,
    pub atmos: Entity<Atmos>,
    pub daemon: Auto,
    /// The servers the window starts after the first goes away, in order.
    pub later: Vec<Auto>,
    /// When the window opened, on the clock `sampledAt` and a write's stamp are on.
    epoch: u64,
    pub cx: &'a mut VisualTestContext,
}

/// What a daemon says to an ordinary request: the version, a held set, and a stored setting whose
/// read-back says nothing.
pub fn ordinary(request: &Value) -> Reply {
    match request["op"].as_str() {
        Some("version") => Reply::Ok(json!("0.4.0")),
        Some("sub.set") => Reply::Ok(json!({"holding": []})),
        Some("settings.set") => Reply::Ok(json!({
            "domain": request["domain"], "value": request["value"], "snapshot": {}})),
        _ => Reply::Ok(Value::Null),
    }
}

/// A window on `page`, its daemon answering by `answer`.
pub fn open<'a>(
    app: &'a mut TestAppContext,
    page: &str,
    answer: impl Fn(&Value) -> Reply + Send + Sync + 'static,
) -> Win<'a> {
    open_with(app, page, 1, answer)
}

/// A window on `page` that can start `servers` servers, each answering by `answer`.
pub fn open_with<'a>(
    app: &'a mut TestAppContext,
    page: &str,
    servers: usize,
    answer: impl Fn(&Value) -> Reply + Send + Sync + 'static,
) -> Win<'a> {
    let serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (transport, scripted) = script(servers);
    let answer = Arc::new(answer);
    let mut daemons: Vec<Auto> = scripted
        .into_iter()
        .map(|server| {
            let answer = answer.clone();
            server.auto(move |request| answer(request))
        })
        .collect();
    let daemon = daemons.remove(0);
    let (client, notices) = Client::start(transport);
    let page = page.to_owned();
    let (atmos, cx) = open_view(app, move |window, cx| {
        cx.new(|cx| Atmos::new(client, notices, &page, window, cx))
    });
    cx.simulate_resize(size(px(1400.0), px(1400.0)));
    cx.run_until_parked();
    Win {
        _serial: serial,
        atmos,
        daemon,
        later: daemons,
        epoch: atmos_ui::settings::now_ms(),
        cx,
    }
}

impl Win<'_> {
    /// Looks, and lets what other threads said reach the window, until `done` holds.
    pub fn until(&mut self, what: &str, done: impl Fn(&mut Self) -> bool) {
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
    pub fn settle(&mut self) {
        thread::sleep(Duration::from_millis(100));
        self.cx.run_until_parked();
    }

    pub fn bounds(&mut self, part: &str) -> Option<Bounds<Pixels>> {
        self.cx
            .debug_bounds(Box::leak(part.to_owned().into_boxed_str()))
    }

    pub fn drawn(&mut self, part: &str) -> bool {
        self.bounds(part).is_some()
    }

    pub fn click(&mut self, part: &str) {
        let at = self
            .bounds(part)
            .unwrap_or_else(|| panic!("{part} is not drawn"))
            .center();
        self.cx.simulate_click(at, Modifiers::default());
        self.cx.run_until_parked();
    }

    /// The daemon pushes a sample of a settings topic taken `at` milliseconds after the window
    /// opened, with nothing failed unless it says. A write is stamped when it is sent, so a
    /// sample from here began before any write the test has made, and the window does not take
    /// it for the keys a write named (`Settings::adopt`); `sample_now` is one that began after.
    pub fn sample(&mut self, topic: &str, at: u64, keys: Value) {
        let at = self.epoch + at;
        self.sample_at(topic, at, keys);
    }

    /// A sample that began now, after every write the window has sent so far.
    pub fn sample_now(&mut self, topic: &str, keys: Value) {
        let at = atmos_ui::settings::now_ms() + 1;
        self.sample_at(topic, at, keys);
    }

    fn sample_at(&mut self, topic: &str, at: u64, keys: Value) {
        let mut result = keys;
        if result.get("_failed").is_none() {
            result["_failed"] = json!({});
        }
        self.daemon.say(json!({
            "event": "sample", "kind": topic, "sampledAt": at, "tookMs": 1, "result": result,
        }));
    }

    /// The daemon says it could not collect a topic.
    pub fn sample_failed(&mut self, topic: &str, why: &str) {
        self.daemon.say(json!({
            "event": "sample", "kind": topic, "stale": true, "error": why,
        }));
    }

    pub fn a_toast_is_shown(&mut self) -> bool {
        (1..64).any(|n| self.drawn(&format!("odk-toast/card-{n}")))
    }

    /// What the window sent besides what every window sends when it opens: the version, what it
    /// holds, and the Sentinel page's limits.
    pub fn sent_by_pages(&self) -> Vec<Value> {
        self.daemon
            .requests()
            .into_iter()
            .filter(|r| {
                let sentinel = r["op"] == "settings.get"
                    && r["domain"]
                        .as_str()
                        .is_some_and(|d| d.starts_with("sentinel"));
                !(r["op"] == "version" || r["op"] == "sub.set" || sentinel)
            })
            .collect()
    }
}
