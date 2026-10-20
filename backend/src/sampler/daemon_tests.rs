use super::*;
use crate::sampler::cache::tests::scratch;
use crate::sampler::worker::serve_jobs;
use std::sync::atomic::AtomicUsize;

/// A pressure dial a test can turn.
pub struct Dial(pub AtomicBool);
impl Probe for Arc<Dial> {
    fn pressure(&self) -> Option<f64> {
        Some(if self.0.load(Ordering::SeqCst) {
            50.0
        } else {
            0.0
        })
    }
    fn load_per_core(&self) -> Option<f64> {
        None
    }
}

pub fn test_config(dial: &Arc<Dial>) -> Config {
    Config {
        floor: Duration::ZERO,
        limits: Limits {
            duty: 1000.0,
            backoff_min: Duration::from_millis(100),
            backoff_max: Duration::from_millis(200),
            ..Limits::default()
        },
        probe: Box::new(dial.clone()),
        job_timeout: Duration::from_secs(5),
        job_runner: crate::sampler::jobs::Runner::Direct,
        run_command: Box::new(|_, body, _| json!({"ok": true, "result": {"ran": body}})),
    }
}

/// A daemon over a real socket. Its foreground collector counts calls and
/// says it ran in the daemon.
pub fn serve(name: &str, delay: Duration) -> (PathBuf, Arc<Daemon>, Arc<AtomicUsize>) {
    let (dir, daemon, calls, _) = serve_with_dial(name, delay);
    (dir, daemon, calls)
}

pub fn serve_with_dial(
    name: &str,
    delay: Duration,
) -> (PathBuf, Arc<Daemon>, Arc<AtomicUsize>, Arc<Dial>) {
    let dir = scratch(name);
    let (listener, lock) = bind(&dir).unwrap();
    std::mem::forget(lock);
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = calls.clone();
    let dial = Arc::new(Dial(AtomicBool::new(false)));
    let daemon = Daemon::new(
        dir.clone(),
        Box::new(move |kind| {
            let n = counted.fetch_add(1, Ordering::SeqCst) + 1;
            std::thread::sleep(delay);
            Ok(json!({"kind": kind, "n": n, "from": "daemon"}))
        }),
        test_config(&dial),
    );
    let lane = daemon.clone();
    std::thread::spawn(move || lane.accept(listener));
    let pump = daemon.clone();
    std::thread::spawn(move || pump.pump(&AtomicBool::new(false)));
    let watcher = daemon.clone();
    std::thread::spawn(move || watcher.watches.run(&AtomicBool::new(false)));
    (dir, daemon, calls, dial)
}

/// A daemon whose commands run `run` instead of a child process.
pub fn serve_commands(name: &str, run: Box<RunCommand>) -> (PathBuf, Arc<Daemon>) {
    let dir = scratch(name);
    let (listener, lock) = bind(&dir).unwrap();
    std::mem::forget(lock);
    let dial = Arc::new(Dial(AtomicBool::new(false)));
    let mut config = test_config(&dial);
    config.run_command = run;
    let daemon = Daemon::new(
        dir.clone(),
        Box::new(|kind| Ok(json!({"kind": kind}))),
        config,
    );
    let lane = daemon.clone();
    std::thread::spawn(move || lane.accept(listener));
    (dir, daemon)
}

/// Connect a sampler process stand-in and wait until the daemon sees it.
pub fn attach_worker(
    daemon: &Arc<Daemon>,
    dir: &Path,
    collect: impl Fn(&str) -> std::result::Result<Value, String> + Send + 'static,
) {
    let stream = UnixStream::connect(cache::socket(dir)).unwrap();
    std::thread::spawn(move || {
        let _ = serve_jobs(stream, &collect);
    });
    let deadline = Instant::now() + Duration::from_secs(3);
    while !daemon.workers.available() {
        assert!(Instant::now() < deadline, "the worker never attached");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn worker_doc(kind: &str) -> std::result::Result<Value, String> {
    Ok(json!({"kind": kind, "from": "worker"}))
}

fn run_schedule(daemon: &Arc<Daemon>) -> (Arc<AtomicBool>, std::thread::JoinHandle<()>) {
    let stop = Arc::new(AtomicBool::new(false));
    let (lane, flag) = (daemon.clone(), stop.clone());
    (stop, std::thread::spawn(move || lane.schedule(&flag)))
}

fn finish(stop: Arc<AtomicBool>, handle: std::thread::JoinHandle<()>) {
    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();
}

fn wait_until(what: &str, secs: u64, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    while !done() {
        assert!(Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn a_passed_socket_is_recognised_only_when_it_is_for_this_process() {
    assert_eq!(listen_fds(Some("42"), Some("1"), 42), Some(3));
    assert_eq!(listen_fds(Some("42"), Some("2"), 42), Some(3));
    assert_eq!(
        listen_fds(Some("41"), Some("1"), 42),
        None,
        "another process"
    );
    assert_eq!(
        listen_fds(Some("42"), Some("0"), 42),
        None,
        "no descriptors"
    );
    assert_eq!(listen_fds(None, Some("1"), 42), None);
    assert_eq!(listen_fds(Some("42"), None, 42), None);
    assert_eq!(listen_fds(Some("x"), Some("1"), 42), None);
}

#[test]
fn a_second_daemon_cannot_start() {
    let dir = scratch("single");
    let first = bind(&dir).unwrap();
    let err = bind(&dir).expect_err("second bind should fail");
    assert!(err.to_string().contains("already running"), "{err}");
    drop(first);
    // A child another test forks shares the lock until it execs, so the release can lag.
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match bind(&dir) {
            Ok(_) => break,
            Err(err) if Instant::now() < deadline => {
                assert!(err.to_string().contains("already running"), "{err}");
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(err) => panic!("the lock goes with the first daemon: {err}"),
        }
    }
}

#[test]
fn the_socket_is_private() {
    let dir = scratch("mode");
    let _held = bind(&dir).unwrap();
    let mode = std::fs::metadata(cache::socket(&dir))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn an_interactive_sample_runs_in_the_daemon_and_is_cached() {
    let (dir, daemon, _) = serve("caches", Duration::ZERO);
    let sample = daemon.sample("services", true).unwrap();
    assert_eq!(sample.value["from"], "daemon");
    assert_eq!(cache::read(&dir, "services").unwrap().value, sample.value);
    assert_eq!(daemon.topics.get("services").seq, 1);
}

#[test]
fn an_unknown_kind_is_refused() {
    let (_, daemon, calls) = serve("unknown", Duration::ZERO);
    assert!(daemon.sample("../x", true).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn with_no_sampler_nothing_is_sampled_in_the_background() {
    let (dir, daemon, calls) = serve("noworker", Duration::ZERO);
    let (stop, handle) = run_schedule(&daemon);
    std::thread::sleep(Duration::from_millis(1500));
    finish(stop, handle);
    assert!(
        cache::read(&dir, "live").is_none(),
        "live was due at once and still not sampled"
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "the daemon did no background work itself"
    );
    assert!(
        daemon.sample("live", false).is_err(),
        "a background sample needs a sampler"
    );
}

#[test]
fn with_a_sampler_the_schedule_collects_in_the_sampler_and_stops() {
    let (dir, daemon, calls) = serve("worker", Duration::ZERO);
    attach_worker(&daemon, &dir, worker_doc);
    let (stop, handle) = run_schedule(&daemon);
    wait_until("live was not sampled", 5, || {
        cache::read(&dir, "live").is_some()
    });
    finish(stop, handle);
    assert_eq!(cache::read(&dir, "live").unwrap().value["from"], "worker");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "the daemon's own collector was not used"
    );
}

#[test]
fn an_interactive_request_is_answered_in_the_daemon_even_with_a_sampler_attached() {
    let (dir, daemon, _) = serve("fg", Duration::ZERO);
    attach_worker(&daemon, &dir, worker_doc);
    assert_eq!(
        daemon.sample("disks", true).unwrap().value["from"],
        "daemon"
    );
}

#[test]
fn a_slow_background_run_never_holds_up_an_interactive_one() {
    let (dir, daemon, _) = serve("yield", Duration::ZERO);
    attach_worker(&daemon, &dir, |kind| {
        std::thread::sleep(Duration::from_millis(1500));
        worker_doc(kind)
    });
    let (stop, handle) = run_schedule(&daemon);
    // Live is due at once, so the sampler is busy with it.
    std::thread::sleep(Duration::from_millis(300));
    let began = Instant::now();
    let sample = daemon.sample("live", true).unwrap();
    assert!(
        began.elapsed() < Duration::from_millis(600),
        "{:?}",
        began.elapsed()
    );
    assert_eq!(
        sample.value["from"], "daemon",
        "the reader did not join the sampler's run"
    );
    stop.store(true, Ordering::SeqCst);
    handle.join().unwrap();
    assert!(cache::read(&dir, "live").is_some());
}

#[test]
fn a_failed_background_run_marks_the_topic_stale_and_keeps_the_last_good_sample() {
    let (dir, daemon, _) = serve("stale", Duration::ZERO);
    let first = daemon.sample("services", true).unwrap();
    attach_worker(&daemon, &dir, |kind| Err(format!("{kind} hung")));
    let err = daemon.sample("services", false).unwrap_err();
    assert_eq!(err, "services hung");
    let topic = daemon.topics.get("services");
    assert!(topic.stale);
    assert_eq!(topic.error.as_deref(), Some("services hung"));
    assert_eq!(topic.seq, 1, "the failure is not a collection");
    assert_eq!(
        cache::read(&dir, "services").unwrap().value,
        first.value,
        "the good sample is still what readers get"
    );
}

#[test]
fn a_sampler_that_dies_mid_run_is_a_failure_and_the_daemon_carries_on() {
    let (dir, daemon, _) = serve("sampler-dies", Duration::ZERO);
    let stream = UnixStream::connect(cache::socket(&dir)).unwrap();
    let cut = stream.try_clone().unwrap();
    std::thread::spawn(move || {
        let _ = serve_jobs(stream, &|kind| {
            std::thread::sleep(Duration::from_millis(500));
            worker_doc(kind)
        });
    });
    wait_until("the worker attached", 3, || daemon.workers.available());
    let asker = {
        let daemon = daemon.clone();
        std::thread::spawn(move || daemon.sample("disks", false))
    };
    std::thread::sleep(Duration::from_millis(100));
    cut.shutdown(std::net::Shutdown::Both).unwrap();
    assert!(asker.join().unwrap().unwrap_err().contains("went away"));
    assert!(daemon.topics.get("disks").stale);
    assert!(
        daemon.sample("disks", true).is_ok(),
        "readers are still served"
    );
    wait_until("nobody is current", 3, || !daemon.workers.available());
    attach_worker(&daemon, &dir, worker_doc);
    assert!(
        daemon.sample("disks", false).is_ok(),
        "a new sampler takes over"
    );
    assert!(!daemon.topics.get("disks").stale, "and the topic heals");
}

#[test]
fn pressure_holds_background_sampling_and_calm_releases_it() {
    let (dir, daemon, _, dial) = serve_with_dial("pressure", Duration::ZERO);
    attach_worker(&daemon, &dir, worker_doc);
    dial.0.store(true, Ordering::SeqCst);
    let (stop, handle) = run_schedule(&daemon);
    std::thread::sleep(Duration::from_millis(1500));
    assert!(
        cache::read(&dir, "live").is_none(),
        "a busy machine is not sampled"
    );
    dial.0.store(false, Ordering::SeqCst);
    wait_until("live was not sampled once it calmed", 5, || {
        cache::read(&dir, "live").is_some()
    });
    finish(stop, handle);
}

#[test]
fn a_dropped_kind_is_resampled_in_the_background_without_a_reader() {
    let (dir, daemon, _) = serve("redo", Duration::ZERO);
    attach_worker(&daemon, &dir, worker_doc);
    // Everything is cached and not yet due, so only the drop can wake a kind.
    for kind in crate::display::KINDS {
        daemon.sample(kind, true).unwrap();
    }
    let (stop, handle) = run_schedule(&daemon);
    std::thread::sleep(Duration::from_millis(50));
    cache::drop_kinds(&dir, &["services"]);
    wait_until("services was not resampled", 9, || {
        cache::read(&dir, "services").is_some_and(|s| s.value["from"] == "worker")
    });
    finish(stop, handle);
}

#[test]
fn the_sampler_watch_speaks_once_when_it_is_missing_and_once_when_it_returns() {
    let mut watch = SamplerWatch::default();
    let start = Instant::now();
    watch.look(false, start);
    assert!(
        !watch.told_off,
        "a sampler normally takes a few seconds to connect"
    );
    watch.look(false, start + SamplerWatch::GRACE);
    assert!(watch.told_off);
    watch.look(false, start + SamplerWatch::GRACE * 2);
    assert!(watch.told_off, "and it does not repeat itself");
    watch.look(true, start + SamplerWatch::GRACE * 3);
    assert!(
        !watch.told_off && watch.absent_since.is_none(),
        "the next absence starts over"
    );
}

#[test]
fn status_reports_the_sampler_and_every_topic() {
    let (dir, daemon, _) = serve("status", Duration::ZERO);
    let before = daemon.answer(&json!({"op": "status"}));
    assert_eq!(before["sampler"]["connected"], false);
    attach_worker(&daemon, &dir, worker_doc);
    daemon.sample("live", true).unwrap();
    let after = daemon.answer(&json!({"op": "status"}));
    assert_eq!(after["sampler"]["connected"], true);
    assert_eq!(after["topics"]["live"]["seq"], 1);
    assert_eq!(after["topics"]["hardware"]["seq"], 0);
    assert_eq!(after["version"], crate::VERSION);
}

#[test]
fn a_restarted_daemon_knows_what_the_cache_already_holds() {
    let (dir, daemon, _) = serve("seed", Duration::ZERO);
    daemon.sample("software", true).unwrap();
    let again = Daemon::new(
        dir.clone(),
        Box::new(|_| Err("unused".into())),
        test_config(&Arc::new(Dial(AtomicBool::new(false)))),
    );
    let topic = again.topics.get("software");
    assert_eq!(topic.seq, 0);
    assert!(topic.sampled_at.is_some());
}
