//! The Drivers page on GPUI's test platform (`common/window.rs`), and with it the jobs every page
//! runs: what it holds, the GPUs it lists, a confirmation before a job, the sudo prompt a job
//! that needs root waits behind, a wrong password, and a job's failure.

mod common;

use std::sync::{Arc, Mutex};

use atmos_ui::pages::drivers::{Ask, DriversPage};
use common::Reply;
use common::window::{Win, open, ordinary};
use odk::gpui::{Entity, TestAppContext};
use serde_json::{Value, json};

fn page(w: &mut Win) -> Entity<DriversPage> {
    w.atmos.read_with(w.cx, |a, _| a.drivers().clone())
}

fn gpus(w: &mut Win) -> Vec<String> {
    let page = page(w);
    page.read_with(w.cx, |p, cx| p.gpus(cx))
}

fn asking(w: &mut Win) -> Option<Ask> {
    let page = page(w);
    page.read_with(w.cx, |p, _| p.asking())
}

fn prompting(w: &mut Win) -> bool {
    let jobs = w.atmos.read_with(w.cx, |a, _| a.jobs().clone());
    jobs.read_with(w.cx, |j, _| j.prompt().is_some())
}

fn busy(w: &mut Win) -> bool {
    let jobs = w.atmos.read_with(w.cx, |a, _| a.jobs().clone());
    jobs.read_with(w.cx, |j, _| j.busy())
}

fn inventory(w: &mut Win, kind: &str, at: u64, result: Value) {
    w.daemon.say(json!({
        "event": "sample", "kind": kind, "sampledAt": at, "tookMs": 1, "result": result,
    }));
}

/// A machine with one NVIDIA GPU that can switch, sudo mode as `sudo` says.
fn usual(w: &mut Win, sudo: bool) {
    inventory(
        w,
        "hardware",
        5,
        json!({"gpus": [{"name": "RTX 4070", "vendor": "NVIDIA", "pciId": "10de:2786",
            "driver": "nvidia"}], "platform": "omarchy"}),
    );
    w.sample(
        "settings.host",
        5,
        json!({"hwNvidia": true, "hwNvidiaGsp": true, "hwNvidiaWithoutGsp": false,
            "hwVulkan": true, "hybridGpuAvailable": true, "hybridGpuMode": "Hybrid",
            "passwordlessSudo": sudo, "favorites": []}),
    );
    w.until("the page read", |w| {
        !gpus(w).is_empty() && w.drawn("drivers-switch/button")
    });
}

/// The job events the daemon sends for the job a `job.start` names: started, then its end.
fn end(w: &Win, start: &Value, code: i64, summary: Value) {
    let cid = start["cid"].clone();
    w.daemon
        .say(json!({"event": "job", "kind": "started", "cid": cid, "job": 41}));
    w.daemon.say(
        json!({"event": "job", "kind": "stream", "cid": cid, "job": 41,
        "seq": 0, "fd": "out", "line": "working"}),
    );
    w.daemon.say(
        json!({"event": "job", "kind": "exit", "cid": cid, "job": 41,
        "code": code, "summary": summary}),
    );
}

fn starts(w: &Win) -> Vec<Value> {
    w.daemon.of("job.start")
}

#[test]
fn the_page_holds_the_hardware_inventory_and_its_keys_while_it_is_shown() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "drivers", ordinary);
    w.until("the topics asked for", |w| {
        w.daemon.of("sub.set").last().is_some_and(|r| {
            r["topics"]
                == json!({"settings.host": {"intervalMs": 0}, "hardware": {"intervalMs": 0}})
        })
    });
    usual(&mut w, true);
    assert_eq!(gpus(&mut w), ["NVIDIA RTX 4070. Driver nvidia. 10de:2786."]);
    w.settle();
    assert!(
        w.sent_by_pages().is_empty(),
        "drawing the page sent {:?}",
        w.sent_by_pages()
    );
}

#[test]
fn with_sudo_mode_on_a_confirmed_switch_runs_the_job_once() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "drivers", ordinary);
    usual(&mut w, true);

    w.click("drivers-switch/button");
    assert_eq!(asking(&mut w), Some(Ask::Switch));
    w.click("drivers-confirm-cancel/button");
    w.settle();
    assert_eq!(asking(&mut w), None);
    assert!(starts(&w).is_empty(), "cancelled");

    w.click("drivers-switch/button");
    w.click("drivers-confirm-go/button");
    w.until("the job", |w| !starts(w).is_empty());
    let start = starts(&w)[0].clone();
    assert_eq!(start["name"], "hybrid-gpu.toggle");
    assert_eq!(start["label"], "hybrid-gpu");
    assert_eq!(start["args"], json!({}));
    assert!(busy(&mut w));
    end(
        &w,
        &start,
        0,
        json!({"ok": true, "code": 0, "message": "", "warnings": []}),
    );
    w.until("the job over", |w| !busy(w));
    assert!(!w.a_toast_is_shown());
}

#[test]
fn a_job_that_needs_root_waits_for_the_password_then_for_sudo_mode() {
    let mut app = TestAppContext::single();
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let kept = seen.clone();
    let mut w = open(&mut app, "drivers", move |request: &Value| {
        kept.lock().unwrap().push(request.clone());
        match request["op"].as_str() {
            Some("job.start") => Reply::Ok(json!({"sent": true})),
            Some("snapshot.keys") => {
                Reply::Ok(json!({"values": {"passwordlessSudo": true}, "settled": true}))
            }
            _ => ordinary(request),
        }
    });
    usual(&mut w, false);

    w.click("drivers-firmware/button");
    w.click("drivers-confirm-go/button");
    w.until("the prompt", |w| prompting(w));
    assert!(starts(&w).is_empty(), "nothing runs before sudo mode");

    // An empty password is refused here and sends nothing.
    w.click("sudo-continue/button");
    w.settle();
    assert!(starts(&w).is_empty());

    w.click("sudo-password/box");
    w.cx.simulate_input("hunter2");
    w.click("sudo-continue/button");
    w.until("sudo mode asked for", |w| !starts(w).is_empty());
    let enable = starts(&w)[0].clone();
    assert_eq!(enable["name"], "sudo.enable");
    assert_eq!(
        enable["args"],
        json!({"minutes": 15, "password": "hunter2"})
    );

    // A wrong password says so and leaves the job waiting.
    end(
        &w,
        &enable,
        1,
        json!({"ok": false, "code": 1, "message": "", "warnings": []}),
    );
    w.until("the refusal said", |w| w.drawn("sudo-error/alert"));
    assert!(prompting(&mut w));
    assert_eq!(starts(&w).len(), 1);

    w.click("sudo-password/box");
    w.cx.simulate_input("right");
    w.click("sudo-continue/button");
    w.until("sudo mode asked for again", |w| starts(w).len() == 2);
    let again = starts(&w)[1].clone();
    assert_eq!(again["args"]["password"], "right");
    end(
        &w,
        &again,
        0,
        json!({"ok": true, "code": 0, "message": "", "warnings": []}),
    );

    // Then the machine is asked, and only then does the job run.
    w.until("the job", |w| starts(w).len() == 3);
    let keys: Vec<_> = seen
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r["op"] == "snapshot.keys")
        .cloned()
        .collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0]["keys"], json!(["passwordlessSudo"]));
    assert_eq!(starts(&w)[2]["name"], "update.firmware");
    assert!(!prompting(&mut w));
}

#[test]
fn a_failed_job_says_why_and_the_next_one_can_run() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "drivers", ordinary);
    usual(&mut w, true);
    w.click("drivers-firmware/button");
    w.click("drivers-confirm-go/button");
    w.until("the job", |w| !starts(w).is_empty());
    let start = starts(&w)[0].clone();
    end(
        &w,
        &start,
        1,
        json!({"ok": false, "code": 1, "message": "fwupd: no devices", "warnings": []}),
    );
    w.until("the failure told", |w| w.a_toast_is_shown());
    assert!(!busy(&mut w));
    w.click("drivers-switch/button");
    w.click("drivers-confirm-go/button");
    w.until("the next job", |w| starts(w).len() == 2);
}

#[test]
fn refresh_reads_every_topic_and_inventory_again() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "drivers", ordinary);
    usual(&mut w, true);
    w.click("drivers-refresh/button");
    w.until("the refresh", |w| {
        !w.daemon.of("snapshot.invalidate").is_empty()
    });
    assert_eq!(
        w.daemon.of("snapshot.invalidate")[0]["topics"],
        json!([
            "settings.look",
            "settings.network",
            "settings.accounts",
            "settings.system",
            "settings.host",
            "hardware",
            "disks",
            "services",
            "software",
            "diagnostics"
        ])
    );
}
