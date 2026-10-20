use super::*;
use crate::sampler::bus::Bus;
use std::time::Duration;

fn env() -> Vec<(String, String)> {
    vec![("PATH".into(), "/usr/bin:/bin".into())]
}

fn spec(script: &str, lane: &str, keep_stdin: bool) -> Spec {
    Spec {
        argv: vec!["sh".into(), "-c".into(), script.into()],
        lane: lane.into(),
        affects: Vec::new(),
        keep_stdin,
        stdin: None,
    }
}

fn collect(inbox: &std::sync::mpsc::Receiver<Value>, until: &str) -> Vec<Value> {
    let mut seen = Vec::new();
    loop {
        let frame = inbox
            .recv_timeout(Duration::from_secs(10))
            .expect("a frame");
        let done = frame["t"] == until;
        seen.push(frame);
        if done {
            return seen;
        }
    }
}

/// Read frames until the job prints `text`.
fn wait_for_line(inbox: &std::sync::mpsc::Receiver<Value>, text: &str) {
    loop {
        let frame = inbox
            .recv_timeout(Duration::from_secs(10))
            .expect("a frame");
        if frame["t"] == "stream" && frame["line"] == text {
            return;
        }
    }
}

fn setup() -> (Arc<Commands>, Arc<Jobs>, Bus) {
    (
        Commands::new(Box::new(|_| {})),
        Arc::new(Jobs::default()),
        Bus::default(),
    )
}

#[test]
fn a_job_streams_its_lines_and_ends_with_a_summary() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    jobs.start(
        &commands,
        &owner,
        "c1".into(),
        "t".into(),
        "test",
        spec("echo one; echo two; echo oops >&2; exit 3", "t", false),
        env(),
        None,
    );
    let frames = collect(&inbox, "job.exit");
    assert_eq!(frames[0]["t"], "job.started");
    assert_eq!(frames[0]["cid"], "c1");
    let lines: Vec<(&str, &str)> = frames
        .iter()
        .filter(|f| f["t"] == "stream")
        .map(|f| (f["fd"].as_str().unwrap(), f["line"].as_str().unwrap()))
        .collect();
    assert!(lines.contains(&("out", "one")) && lines.contains(&("out", "two")));
    assert!(lines.contains(&("err", "oops")));
    let end = frames.last().unwrap();
    assert_eq!(end["code"], 3);
    assert_eq!(end["summary"]["ok"], false);
    assert_eq!(end["summary"]["message"], "oops");
    assert_eq!(jobs.running(), 0);
}

#[test]
fn the_callers_environment_and_only_it_reaches_the_job() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    let env = vec![
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ("ATMOS_PROBE".to_string(), "from the window".to_string()),
    ];
    jobs.start(
        &commands,
        &owner,
        "c".into(),
        "t".into(),
        "test",
        spec("echo \"$ATMOS_PROBE|$HOME\"", "env", false),
        env,
        None,
    );
    let frames = collect(&inbox, "job.exit");
    let line = frames.iter().find(|f| f["t"] == "stream").unwrap();
    assert_eq!(
        line["line"], "from the window|",
        "the daemon's own HOME is not passed on"
    );
}

#[test]
fn input_reaches_a_job_that_keeps_reading() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    let job = jobs.start(
        &commands,
        &owner,
        "c".into(),
        "t".into(),
        "test",
        spec(
            "read first; echo got-$first; read second; echo got-$second",
            "in",
            true,
        ),
        env(),
        Some("a\n".into()),
    );
    let mut frames = Vec::new();
    while frames.iter().all(|f: &Value| f["line"] != "got-a") {
        frames.push(inbox.recv_timeout(Duration::from_secs(10)).unwrap());
    }
    assert!(jobs.input(job.id, "b\n", true));
    let rest = collect(&inbox, "job.exit");
    assert!(rest.iter().any(|f| f["line"] == "got-b"));
    assert!(
        !jobs.input(job.id, "late\n", false),
        "stdin is closed once the job is over"
    );
}

#[test]
fn a_job_that_reads_to_the_end_gets_its_end() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    jobs.start(
        &commands,
        &owner,
        "c".into(),
        "t".into(),
        "test",
        spec("cat", "eof", false),
        env(),
        Some("whole text\n".into()),
    );
    let frames = collect(&inbox, "job.exit");
    assert!(frames.iter().any(|f| f["line"] == "whole text"));
}

#[test]
fn cancel_stops_the_job_and_what_it_started() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    let job = jobs.start(
        &commands,
        &owner,
        "c".into(),
        "t".into(),
        "test",
        spec("sleep 30 & echo started; wait", "cancel", false),
        env(),
        None,
    );
    wait_for_line(&inbox, "started");
    assert!(jobs.cancel(job.id));
    let end = collect(&inbox, "job.exit");
    assert_ne!(end.last().unwrap()["code"], 0);
    assert_eq!(jobs.running(), 0);
}

#[test]
fn jobs_on_one_lane_run_in_order_and_other_lanes_do_not_wait() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    for (cid, lane) in [("a", "shared"), ("b", "shared"), ("c", "other")] {
        jobs.start(
            &commands,
            &owner,
            cid.into(),
            "t".into(),
            "test",
            spec(
                &format!("echo {cid}-start; sleep 0.3; echo {cid}-end"),
                lane,
                false,
            ),
            env(),
            None,
        );
    }
    let mut order = Vec::new();
    let mut ended = 0;
    while ended < 3 {
        let frame = inbox.recv_timeout(Duration::from_secs(10)).unwrap();
        if frame["t"] == "stream" {
            order.push(frame["line"].as_str().unwrap().to_string());
        }
        ended += usize::from(frame["t"] == "job.exit");
    }
    let at = |text: &str| order.iter().position(|l| l == text).unwrap();
    assert!(
        at("a-end") < at("b-start"),
        "one lane, one at a time: {order:?}"
    );
    assert!(
        at("c-start") < at("a-end"),
        "another lane does not wait: {order:?}"
    );
}

#[test]
fn a_job_outlives_its_window_and_a_new_window_takes_the_rest() {
    let (commands, jobs, bus) = setup();
    let (first, first_inbox) = bus.attach();
    let job = jobs.start(
        &commands,
        &first,
        "c".into(),
        "t".into(),
        "test",
        spec("echo early; sleep 0.5; echo late", "away", false),
        env(),
        None,
    );
    wait_for_line(&first_inbox, "early");
    drop(first_inbox);
    jobs.forget_owner(&first);
    bus.detach(&first);
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(jobs.running(), 0, "it finished with nobody listening");
    let (second, second_inbox) = bus.attach();
    assert!(jobs.attach(job.id, 0, &second));
    let frames = collect(&second_inbox, "job.exit");
    let lines: Vec<&str> = frames
        .iter()
        .filter(|f| f["t"] == "stream")
        .map(|f| f["line"].as_str().unwrap())
        .collect();
    assert_eq!(
        lines,
        ["early", "late"],
        "everything it printed is still there"
    );
    let listed = jobs.list();
    assert_eq!(listed[0]["state"], "done");
    assert!(!jobs.attach(9999, 0, &second));
}

#[test]
fn units_of_another_daemons_jobs_are_found_in_a_listing() {
    let listing = "atmos-job-100-3.service loaded active running Atmos job tailscale.receive\n\
                   atmos-job-200-1.service loaded active running Atmos job update.omarchy\n\
                   atmos-job-notapid.service loaded active running x\n\
                   other.service loaded active running y\n";
    let found = parse_orphans(listing, 200);
    assert_eq!(
        found,
        [(
            "atmos-job-100-3.service".to_string(),
            "Atmos job tailscale.receive".to_string()
        )],
        "this daemon's own jobs and strangers are not orphans"
    );
    assert!(parse_orphans("", 1).is_empty());
}

#[test]
fn a_command_that_cannot_start_is_a_failed_job_not_a_hang() {
    let (commands, jobs, bus) = setup();
    let (owner, inbox) = bus.attach();
    jobs.start(
        &commands,
        &owner,
        "c".into(),
        "t".into(),
        "test",
        Spec {
            argv: vec!["/nonexistent/tool".into()],
            lane: "x".into(),
            affects: Vec::new(),
            keep_stdin: false,
            stdin: None,
        },
        env(),
        None,
    );
    let end = collect(&inbox, "job.exit");
    let last = end.last().unwrap();
    assert_eq!(last["code"], 127);
    assert!(last["summary"]["message"]
        .as_str()
        .unwrap()
        .contains("/nonexistent/tool"));
}

#[test]
fn a_waited_unit_leaves_what_the_tool_started_running() {
    let argv = ["omarchy".to_string(), "capture".to_string()];
    let waited = unit_argv("atmos-job-1-1", "Atmos job x", &argv, &[], true);
    assert!(waited.contains(&"--property=KillMode=process".to_string()));
    assert!(waited.contains(&"--wait".to_string()));
    let launched = unit_argv("atmos-launch-1", "Atmos launch x", &argv, &[], false);
    assert!(!launched.iter().any(|a| a.starts_with("--property")));
}

#[test]
fn a_running_job_does_not_hold_reads_and_drops_what_it_changed_before_it_ends() {
    use std::sync::Mutex as StdMutex;
    let dropped: Arc<StdMutex<Vec<String>>> = Arc::default();
    let seen = Arc::clone(&dropped);
    let commands = Commands::new(Box::new(move |topics| {
        seen.lock().unwrap().extend(topics.iter().cloned());
    }));
    let (jobs, bus) = (Arc::new(Jobs::default()), Bus::default());
    let (owner, inbox) = bus.attach();
    let mut changes = spec("echo up; read line", "", true);
    changes.affects = vec!["disks".into()];
    let job = jobs.start(
        &commands,
        &owner,
        "c1".into(),
        "install".into(),
        "x",
        changes,
        env(),
        None,
    );
    wait_for_line(&inbox, "up");
    // A job may run for minutes: a read of what it changes is not made to wait.
    assert!(commands.barrier(&["disks".to_string()], Duration::from_millis(50)));
    assert!(dropped.lock().unwrap().is_empty());
    jobs.input(job.id, "go\n", true);
    let end = collect(&inbox, "job.exit");
    assert_eq!(end.last().unwrap()["code"], 0);
    // The caches were told before the window was.
    assert_eq!(*dropped.lock().unwrap(), ["disks"]);
}
