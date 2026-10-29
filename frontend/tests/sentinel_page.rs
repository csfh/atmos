//! The Sentinel page on GPUI's test platform: nothing opens on the desktop. A scripted daemon
//! answers what the window asks and pushes `live` frames, so the test says what the backend said
//! and sees what the window drew and what it sent back.

mod common;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use atmos_ui::actions::Signal;
use atmos_ui::app::Atmos;
use atmos_ui::client::Client;
use atmos_ui::hubs;
use atmos_ui::pages::sentinel::SentinelPage;
use atmos_ui::sentinel::Filter;
use common::{Auto, Reply, script};
use odk::gpui::{
    AppContext as _, Bounds, Entity, KeyUpEvent, Keystroke, Modifiers, Pixels, TestAppContext,
    VisualTestContext, px, size,
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

/// What a daemon says to an ordinary request: the version, a held set, an unset setting, a
/// settled write.
fn ordinary(request: &Value) -> Reply {
    match request["op"].as_str() {
        Some("version") => Reply::Ok(json!("0.4.0")),
        Some("sub.set") => Reply::Ok(json!({"holding": []})),
        Some("settings.get") => Reply::Ok(Value::Null),
        Some("settings.set") => Reply::Ok(json!({})),
        Some("write") => Reply::Ok(json!({"settled": true, "snapshot": {}})),
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
    let (atmos, cx) = open_view(app, move |window, cx| {
        cx.new(|cx| Atmos::new(client, notices, &page, window, cx))
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
    fn sentinel(&mut self) -> Entity<SentinelPage> {
        self.atmos.read_with(self.cx, |a, _| a.sentinel().clone())
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

    fn press(&mut self, key: &str) {
        let keystroke = Keystroke::parse(key).unwrap();
        self.cx
            .update(|window, cx| window.dispatch_keystroke(keystroke.clone(), cx));
        self.cx.simulate_event(KeyUpEvent { keystroke });
        self.cx.run_until_parked();
    }

    fn taken(&mut self) -> usize {
        let page = self.sentinel();
        page.read_with(self.cx, |p, _| p.taken())
    }

    fn shown_ids(&mut self) -> Vec<String> {
        let page = self.sentinel();
        page.read_with(self.cx, |p, _| {
            p.shown().into_iter().map(|f| f.id).collect()
        })
    }

    /// The daemon pushes a `live` frame taken at `at` milliseconds, and the window takes it in.
    fn frame(&mut self, at: u64, processes: Vec<Value>, sockets: Vec<Value>) {
        let before = self.taken();
        self.frame_unchecked(at, processes, sockets);
        self.until("the frame taken in", move |w| w.taken() > before);
    }

    fn frame_unchecked(&mut self, at: u64, processes: Vec<Value>, sockets: Vec<Value>) {
        self.daemon.say(json!({
            "event": "sample", "kind": "live", "sampledAt": at, "tookMs": 1,
            "result": {
                "clkTck": 100, "memTotal": 1_000_000, "uptimeSec": 1_000_000,
                "processes": processes, "tcpSockets": sockets,
            },
        }));
    }

    fn a_toast_is_shown(&mut self) -> bool {
        (1..64).any(|n| self.drawn(&format!("odk-toast/card-{n}")))
    }
}

/// A process that has used `ticks` ticks of CPU so far: 1000 ticks in a frame two seconds after
/// the one before is 500% of a core.
fn hog(pid: u64, ticks: u64, mine: bool) -> Value {
    json!({"pid": pid, "comm": "hog", "cmdline": "/usr/bin/hog --spin", "ticks": ticks,
        "rssKb": 1000, "startTicks": 100, "kthread": false, "mine": mine, "uid": 1000})
}

/// A frame sequence that has `pid` over the limit from the second frame on.
fn spin(w: &mut Win, pid: u64, mine: bool, frames: u64) {
    for i in 1..=frames {
        w.frame(2000 * i, vec![hog(pid, i * 1000, mine)], vec![]);
    }
}

#[test]
fn the_page_holds_the_live_feed_only_while_it_is_shown() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "appearance", ordinary);
    let at = |id: &str| hubs::HUBS.iter().position(|h| h.id == id).unwrap();
    let (sentinel, appearance) = (at("sentinel"), at("appearance"));
    w.until("the first hold", |w| !w.daemon.of("sub.set").is_empty());
    assert!(
        w.daemon.of("sub.set")[0]["topics"].get("live").is_none(),
        "appearance reads nothing live"
    );

    w.click(&format!("nav/item-{sentinel}"));
    w.until("the live feed asked for", |w| {
        w.daemon
            .of("sub.set")
            .iter()
            .any(|r| r["topics"]["live"]["intervalMs"] == 2000)
    });
    w.click(&format!("nav/item-{appearance}"));
    w.until("the feed let go", |w| {
        w.daemon
            .of("sub.set")
            .last()
            .is_some_and(|r| r["topics"].get("live").is_none())
    });
    assert_eq!(w.daemon.of("sub.set").len(), 3);
}

#[test]
fn a_process_is_a_finding_only_after_three_samples_over_its_limit() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "sentinel", ordinary);
    spin(&mut w, 77, true, 3);
    assert!(
        w.shown_ids().is_empty(),
        "two rates are not three: {:?}",
        w.shown_ids()
    );
    w.frame(8000, vec![hog(77, 4000, true)], vec![]);
    assert_eq!(w.shown_ids(), ["cpu:77"]);
    w.cx.run_until_parked();
    assert!(
        w.drawn("more-cpu:77/button/button"),
        "its menu is drawn too"
    );
    assert!(w.drawn("end-cpu:77/button"), "and with End");
}

#[test]
fn a_frame_that_is_not_after_the_last_is_not_taken() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "sentinel", ordinary);
    w.frame(2000, vec![hog(77, 0, true)], vec![]);
    // The same sample twice, an older one, and one that is no sample: none is taken in.
    w.frame_unchecked(2000, vec![hog(77, 999, true)], vec![]);
    w.frame_unchecked(1000, vec![hog(77, 999, true)], vec![]);
    w.daemon
        .say(json!({"event": "sample", "kind": "live", "stale": true, "error": "slow"}));
    w.frame(4000, vec![hog(77, 1, true)], vec![]);
    assert_eq!(w.taken(), 2, "the two real ones");
}

#[test]
fn ending_a_process_asks_first_and_then_sends_exactly_term() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "sentinel", ordinary);
    spin(&mut w, 77, true, 4);
    w.click("end-cpu:77/button");
    let page = w.sentinel();
    assert!(page.read_with(w.cx, |p, _| p.asking()), "asked, not done");
    assert!(w.drawn("sentinel-confirm/panel"));
    assert!(
        w.daemon.of("process.signal").is_empty(),
        "nothing is sent before the yes"
    );

    w.click("sentinel-confirm-go/button");
    w.until("the signal sent", |w| {
        !w.daemon.of("process.signal").is_empty()
    });
    let sent = w.daemon.of("process.signal");
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["args"], json!({"pid": 77, "signal": "TERM"}));
    assert!(
        !page.read_with(w.cx, |p, _| p.asking()),
        "and the dialog is done"
    );
}

#[test]
fn cancelling_sends_nothing_and_a_force_quit_sends_kill() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "sentinel", ordinary);
    spin(&mut w, 77, true, 4);
    w.click("end-cpu:77/button");
    w.click("sentinel-cancel/button");
    let page = w.sentinel();
    assert!(!page.read_with(w.cx, |p, _| p.asking()));
    thread::sleep(Duration::from_millis(100));
    assert!(w.daemon.of("process.signal").is_empty(), "cancelled");

    page.update(w.cx, |p, cx| p.ask(77, "hog", Signal::Kill, cx));
    w.cx.run_until_parked();
    w.click("sentinel-confirm-go/button");
    w.until("the kill sent", |w| {
        !w.daemon.of("process.signal").is_empty()
    });
    assert_eq!(
        w.daemon.of("process.signal")[0]["args"],
        json!({"pid": 77, "signal": "KILL"})
    );
}

#[test]
fn a_process_that_is_not_the_users_is_listed_but_has_no_end() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "sentinel", ordinary);
    spin(&mut w, 77, false, 4);
    assert_eq!(w.shown_ids(), ["cpu:77"]);
    w.cx.run_until_parked();
    assert_eq!(w.shown_ids(), ["cpu:77"], "listed all the same");
    assert!(
        !w.drawn("end-cpu:77/button"),
        "the daemon would refuse it, so it is not offered"
    );
}

#[test]
fn a_signal_the_daemon_refuses_is_told() {
    let mut app = TestAppContext::single();
    let answer = |r: &Value| {
        if r["name"] == "process.signal" {
            Reply::Err("failed", "not your process")
        } else {
            ordinary(r)
        }
    };
    let mut w = open(&mut app, "sentinel", answer);
    spin(&mut w, 77, true, 4);
    assert!(!w.a_toast_is_shown());
    w.click("end-cpu:77/button");
    w.click("sentinel-confirm-go/button");
    w.until("the refusal told", |w| w.a_toast_is_shown());
}

#[test]
fn the_limits_are_read_and_a_commit_writes_one_integer_and_a_keystroke_writes_nothing() {
    let mut app = TestAppContext::single();
    let answer = |r: &Value| match (r["op"].as_str(), r["domain"].as_str()) {
        (Some("settings.get"), Some("sentinelCpuPercent")) => Reply::Ok(json!(70)),
        (Some("settings.get"), Some("sentinelPortAgeDays")) => Reply::Ok(json!("abc")),
        _ => ordinary(r),
    };
    let mut w = open(&mut app, "sentinel", answer);
    let page = w.sentinel();
    w.until("the limits read", |w| {
        w.drawn("limit-sentinelCpuPercent/box") && w.drawn("limit-sentinelPortAgeDays/box")
    });
    let th = page.read_with(w.cx, |p, _| p.thresholds());
    assert_eq!(
        (th.cpu_percent, th.memory_percent, th.port_age_days),
        (70, 20, 7),
        "text is no limit"
    );

    w.click("limit-sentinelCpuPercent/box");
    w.press("ctrl-a");
    w.cx.simulate_input("9");
    w.cx.simulate_input("0");
    thread::sleep(Duration::from_millis(100));
    w.cx.run_until_parked();
    assert!(
        w.daemon.of("settings.set").is_empty(),
        "typing `90` passes through `9`, and writes none"
    );
    w.press("enter");
    w.until("the limit stored", |w| {
        !w.daemon.of("settings.set").is_empty()
    });
    let sets = w.daemon.of("settings.set");
    assert_eq!(sets.len(), 1, "one write: {sets:?}");
    assert_eq!(sets[0]["domain"], "sentinelCpuPercent");
    assert!(
        sets[0]["value"].is_i64(),
        "an integer domain takes an integer: {}",
        sets[0]["value"]
    );
    assert_eq!(sets[0]["value"], 90);
    assert_eq!(page.read_with(w.cx, |p, _| p.thresholds().cpu_percent), 90);

    // Leaving it as it is writes nothing more.
    w.press("tab");
    thread::sleep(Duration::from_millis(100));
    w.cx.run_until_parked();
    assert_eq!(w.daemon.of("settings.set").len(), 1);
}

#[test]
fn a_limit_the_daemon_refuses_is_told_and_read_again() {
    let mut app = TestAppContext::single();
    let answer = |r: &Value| {
        if r["op"] == "settings.set" {
            Reply::Err("failed", "the file is read-only")
        } else {
            ordinary(r)
        }
    };
    let mut w = open(&mut app, "sentinel", answer);
    w.until("the limits read", |w| {
        w.drawn("limit-sentinelCpuPercent/box")
    });
    let reads = w.daemon.of("settings.get").len();
    w.click("limit-sentinelCpuPercent/box");
    w.press("ctrl-a");
    w.cx.simulate_input("95");
    w.press("enter");
    w.until("the refusal told and the value read again", |w| {
        w.a_toast_is_shown() && w.daemon.of("settings.get").len() > reads
    });
}

#[test]
fn limits_that_cannot_be_read_are_not_shown_as_values() {
    let mut app = TestAppContext::single();
    let answer = |r: &Value| {
        if r["op"] == "settings.get" {
            Reply::Err("io", "shell.json is unreadable")
        } else {
            ordinary(r)
        }
    };
    let mut w = open(&mut app, "sentinel", answer);
    w.until("the failure told", |w| {
        w.drawn("sentinel-limits-alert/alert")
    });
    assert!(
        !w.drawn("limit-sentinelCpuPercent/box"),
        "no field with a made-up value"
    );
}

fn listening(port: u64, addr: &str, pid: u64) -> Value {
    json!({"state": "LISTEN", "addr": addr, "port": port, "uid": 1000, "pid": pid})
}

#[test]
fn the_chips_choose_what_is_listed_and_the_search_narrows_it() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "sentinel", ordinary);
    // A process up for 11 days, over the 7-day limit, with two ports; and a young one.
    let old = json!({"pid": 20, "comm": "webd", "cmdline": "webd --serve", "ticks": 0, "rssKb": 1,
        "startTicks": 100, "kthread": false, "mine": true});
    let young = json!({"pid": 21, "comm": "tmpd", "cmdline": "tmpd", "ticks": 0, "rssKb": 1,
        "startTicks": 99_900_000, "kthread": false, "mine": true});
    w.frame(
        2000,
        vec![old, young],
        vec![
            listening(8080, "0.0.0.0", 20),
            listening(8081, "::", 20),
            listening(9000, "127.0.0.1", 21),
        ],
    );
    let page = w.sentinel();
    assert_eq!(
        w.shown_ids(),
        ["ports:20"],
        "flagged: the old ports are one row"
    );
    assert_eq!(page.read_with(w.cx, |p, _| p.filter()), Filter::Flagged);

    w.click("sentinel-kinds/item-3");
    assert_eq!(page.read_with(w.cx, |p, _| p.filter()), Filter::Port);
    assert_eq!(
        w.shown_ids().len(),
        3,
        "every listener: {:?}",
        w.shown_ids()
    );

    // The field is a usable width, not the width of its icon.
    let box_width = w
        .bounds("sentinel-search/box")
        .map(|b| f32::from(b.size.width));
    assert!(
        box_width.is_some_and(|width| width > 200.0),
        "search is {box_width:?} wide"
    );
    w.click("sentinel-search/box");
    w.cx.simulate_input("9000");
    w.cx.run_until_parked();
    assert_eq!(w.shown_ids(), ["port:9000:21"], "searched by port");
}
