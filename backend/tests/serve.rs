mod common;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use common::{cleanup, temp_root};
use serde_json::{json, Value};

struct Server {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<Value>,
}

impl Server {
    fn start(root: &std::path::Path, backend: &str) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .args([
                "--backend",
                backend,
                "--root",
                &root.display().to_string(),
                "serve",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ratmos serve");
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if tx.send(value).is_err() {
                        break;
                    }
                }
            }
        });
        Server {
            child,
            stdin,
            lines,
        }
    }

    /// A serve with no fixture root, talking to whatever daemon lives under `run`.
    fn start_live(run: &std::path::Path) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .args(["--backend", "omarchy", "serve"])
            .env("XDG_RUNTIME_DIR", run)
            .env("ATMOS_NO_AUTOSTART", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ratmos serve");
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if tx.send(value).is_err() {
                        break;
                    }
                }
            }
        });
        Server {
            child,
            stdin,
            lines,
        }
    }

    fn send(&mut self, value: &Value) {
        let stdin = self.stdin.as_mut().expect("stdin open");
        writeln!(stdin, "{}", serde_json::to_string(value).unwrap()).unwrap();
        stdin.flush().unwrap();
    }

    fn next(&self) -> Value {
        self.lines
            .recv_timeout(Duration::from_secs(10))
            .expect("a line from serve")
    }

    /// The next line that matches, skipping anything else the server pushed.
    fn until(&self, want: impl Fn(&Value) -> bool) -> Value {
        for _ in 0..50 {
            let value = self.next();
            if want(&value) {
                return value;
            }
        }
        panic!("no matching line");
    }

    fn finish(mut self) -> std::process::ExitStatus {
        drop(self.stdin.take());
        self.child.wait().expect("serve exits")
    }
}

#[test]
fn replies_carry_the_request_id() {
    let root = temp_root();
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": 7, "op": "version"}));
    let reply = server.next();
    assert_eq!(reply["id"], 7);
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["result"], "0.3.0");
    assert!(server.finish().success());
    cleanup(&root);
}

#[test]
fn failures_are_envelopes_with_the_same_id() {
    let root = temp_root();
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": "a", "op": "nope"}));
    let reply = server.next();
    assert_eq!(reply["id"], "a");
    assert_eq!(reply["ok"], false);
    assert_eq!(reply["error"]["code"], "bad_request");
    server.send(&json!({"id": "b", "op": "version"}));
    assert_eq!(
        server.next()["id"],
        "b",
        "one bad request does not stop the server"
    );
    server.finish();
    cleanup(&root);
}

#[test]
fn a_garbled_line_is_answered_and_survived() {
    let root = temp_root();
    let mut server = Server::start(&root, "omarchy");
    let stdin = server.stdin.as_mut().unwrap();
    writeln!(stdin, "{{not json").unwrap();
    stdin.flush().unwrap();
    let reply = server.next();
    assert_eq!(reply["ok"], false);
    server.send(&json!({"id": 1, "op": "version"}));
    assert_eq!(server.next()["id"], 1);
    server.finish();
    cleanup(&root);
}

#[test]
fn settings_round_trip_over_one_process() {
    let root = temp_root();
    let mut server = Server::start(&root, "plain");
    server.send(&json!({"id": 1, "op": "settings.set", "domain": "theme", "value": "tokyo"}));
    assert_eq!(server.next()["result"]["value"], "tokyo");
    server.send(&json!({"id": 2, "op": "settings.get", "domain": "theme"}));
    assert_eq!(server.next()["result"], "tokyo");
    server.finish();
    cleanup(&root);
}

#[test]
fn watching_pushes_a_stamp_only_when_something_changes() {
    let root = temp_root();
    let file = root.join("watched.txt");
    fs::write(&file, "one").unwrap();
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": 1, "op": "watch.set", "paths": ["watched.txt"]}));
    assert_eq!(server.until(|v| v["id"] == 1)["ok"], true);

    let first = server.until(|v| v["event"] == "stamp");
    assert_eq!(first["result"]["items"][0]["text"], "one");

    // Nothing changed, so the next second must stay quiet.
    assert!(
        server
            .lines
            .recv_timeout(Duration::from_millis(1600))
            .is_err(),
        "an unchanged file must not push"
    );

    fs::write(&file, "two plus").unwrap();
    let second = server.until(|v| v["event"] == "stamp");
    assert_eq!(second["result"]["items"][0]["text"], "two plus");
    server.finish();
    cleanup(&root);
}

#[test]
fn watching_chrome_pushes_the_theme_files() {
    let root = temp_root();
    let dir = root.join(".local/state/omarchy/current");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("theme.name"), "tokyo-night\n").unwrap();
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": 1, "op": "watch.set", "chrome": true}));
    let pushed = server.until(|v| v["event"] == "chrome");
    assert_eq!(pushed["result"]["themeName"], "tokyo-night\n");
    server.finish();
    cleanup(&root);
}

#[test]
fn a_slow_request_does_not_block_a_fast_one() {
    let root = temp_root();
    let mut server = Server::start(&root, "omarchy");
    // The first request reads a whole snapshot; the second is trivial. Both
    // must come back, whichever order they finish in.
    server.send(&json!({"id": 1, "op": "settings.snapshot"}));
    server.send(&json!({"id": 2, "op": "version"}));
    let mut seen = vec![server.next()["id"].as_i64().unwrap()];
    seen.push(server.next()["id"].as_i64().unwrap());
    seen.sort();
    assert_eq!(seen, vec![1, 2]);
    server.finish();
    cleanup(&root);
}

#[test]
fn closing_stdin_stops_the_server() {
    let root = temp_root();
    let server = Server::start(&root, "omarchy");
    assert!(server.finish().success());
    cleanup(&root);
}

fn live_fixture(root: &std::path::Path) {
    let dir = root.join(".local/state/omarchy/display");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("live.json"), r#"{"cpuIdle": 1, "cpuTotal": 2}"#).unwrap();
}

#[test]
fn holding_live_pushes_samples_on_the_holders_clock() {
    let root = temp_root();
    live_fixture(&root);
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 300}}}));
    let reply = server.until(|line| line["id"] == 1);
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["result"]["holding"], json!(["live"]));
    let first = server.until(|line| line["event"] == "sample");
    let second = server.until(|line| line["event"] == "sample");
    assert_eq!(first["kind"], "live");
    assert_eq!(first["result"]["cpuTotal"], 2);
    let gap = second["sampledAt"].as_u64().unwrap() - first["sampledAt"].as_u64().unwrap();
    assert!((250..900).contains(&gap), "gap {gap}");
    assert!(server.finish().success());
    cleanup(&root);
}

#[test]
fn releasing_live_stops_the_samples() {
    let root = temp_root();
    live_fixture(&root);
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 250}}}));
    server.until(|line| line["event"] == "sample");
    server.send(&json!({"id": 2, "op": "sub.set", "topics": {}}));
    server.until(|line| line["id"] == 2);
    // Anything already on its way lands first; then it goes quiet.
    thread::sleep(Duration::from_millis(400));
    while server.lines.try_recv().is_ok() {}
    thread::sleep(Duration::from_millis(900));
    assert!(server.lines.try_recv().is_err(), "samples after release");
    server.finish();
    cleanup(&root);
}

#[test]
fn an_unknown_topic_is_refused() {
    let root = temp_root();
    let mut server = Server::start(&root, "omarchy");
    server.send(&json!({"id": 3, "op": "sub.set", "topics": {"nope": {}}}));
    let reply = server.next();
    assert_eq!(reply["id"], 3);
    assert_eq!(reply["ok"], false);
    assert_eq!(reply["error"]["code"], "bad_request");
    server.finish();
    cleanup(&root);
}

/// A runtime dir short enough for a socket path, one per test: the tests of this
/// binary share a pid and run side by side.
fn short_run_dir(name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(format!("/tmp/rt-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

struct Daemon(Child);

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_daemon(run: &std::path::Path) -> Daemon {
    let daemon = Daemon(
        Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .arg("daemon")
            .env("XDG_RUNTIME_DIR", run)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ratmos daemon"),
    );
    let socket = run.join("atmos/ratmos.sock");
    for _ in 0..100 {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            return daemon;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("the daemon never listened");
}

#[test]
fn two_windows_are_fed_by_one_daemon_each_at_its_own_rate() {
    let run = short_run_dir("two");
    let _daemon = start_daemon(&run);
    let mut fast = Server::start_live(&run);
    let mut slow = Server::start_live(&run);
    fast.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 400}}}));
    slow.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 1200}}}));
    let first = fast.until(|line| line["event"] == "sample");
    assert!(first["result"]["cpuTotal"].is_number(), "{first}");
    assert!(first["sampledAt"].as_u64().unwrap() > 1_000_000_000_000);
    let started = std::time::Instant::now();
    let (mut fast_n, mut slow_n) = (1, 0);
    while started.elapsed() < Duration::from_millis(3000) {
        while let Ok(line) = fast.lines.try_recv() {
            if line["event"] == "sample" {
                fast_n += 1;
            }
        }
        while let Ok(line) = slow.lines.try_recv() {
            if line["event"] == "sample" {
                slow_n += 1;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(fast_n >= 5, "fast {fast_n}");
    assert!((1..=4).contains(&slow_n), "slow {slow_n}");
    fast.finish();
    slow.finish();
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_window_keeps_sampling_when_the_daemon_goes_away() {
    let run = short_run_dir("gone");
    let daemon = start_daemon(&run);
    let mut server = Server::start_live(&run);
    server.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 300}}}));
    let before = server.until(|line| line["event"] == "sample");
    drop(daemon);
    let mut after = before.clone();
    for _ in 0..40 {
        let line = server.until(|line| line["event"] == "sample");
        if line["sampledAt"] != before["sampledAt"] && line.get("error").is_none() {
            after = line;
            if after["sampledAt"].as_u64() > before["sampledAt"].as_u64() {
                break;
            }
        }
    }
    assert!(after["sampledAt"].as_u64() > before["sampledAt"].as_u64());
    server.finish();
    let _ = fs::remove_dir_all(&run);
}

fn wait_exit(child: &mut Child, within: Duration) -> Option<std::process::ExitStatus> {
    let until = std::time::Instant::now() + within;
    while std::time::Instant::now() < until {
        if let Some(status) = child.try_wait().unwrap() {
            return Some(status);
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

/// A daemon run from its own copy of the binary, so the test can replace it.
fn start_copy(run: &std::path::Path) -> (Daemon, std::path::PathBuf) {
    let exe = run.join("ratmos-copy");
    fs::copy(env!("CARGO_BIN_EXE_ratmos"), &exe).unwrap();
    // A binary just written can still be open for writing in a process another
    // test forked at that moment, and exec says "text file busy" until it is not.
    let mut tries = 0;
    let child = loop {
        match Command::new(&exe)
            .arg("daemon")
            .env("XDG_RUNTIME_DIR", run)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => break child,
            Err(err) if err.kind() == std::io::ErrorKind::ExecutableFileBusy && tries < 40 => {
                tries += 1;
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => panic!("spawn the copy: {err}"),
        }
    };
    let daemon = Daemon(child);
    let socket = run.join("atmos/ratmos.sock");
    for _ in 0..100 {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            return (daemon, exe);
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("the copy never listened");
}

#[test]
fn a_daemon_leaves_when_its_binary_is_replaced() {
    let run = short_run_dir("swap");
    let (mut daemon, exe) = start_copy(&run);
    assert!(wait_exit(&mut daemon.0, Duration::from_millis(1500)).is_none());
    // A rebuild writes a new file and renames it over the old one.
    let next = run.join("ratmos-next");
    fs::copy(env!("CARGO_BIN_EXE_ratmos"), &next).unwrap();
    fs::rename(&next, &exe).unwrap();
    let status = wait_exit(&mut daemon.0, Duration::from_secs(9));
    assert!(status.is_some_and(|s| s.success()), "{status:?}");
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_daemon_leaves_when_its_runtime_dir_is_gone() {
    let run = short_run_dir("lockgone");
    let (mut daemon, _exe) = start_copy(&run);
    fs::remove_file(run.join("atmos/daemon.lock")).unwrap();
    let status = wait_exit(&mut daemon.0, Duration::from_secs(9));
    assert!(status.is_some_and(|s| s.success()), "{status:?}");
    let _ = fs::remove_dir_all(&run);
}
