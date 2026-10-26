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
        let mut child = Command::new(env!("CARGO_BIN_EXE_atmosd"))
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
            .expect("spawn atmosd serve");
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
        let mut child = Command::new(env!("CARGO_BIN_EXE_atmosd"))
            .args(["--backend", "omarchy", "serve"])
            .env("XDG_RUNTIME_DIR", run)
            .env("ATMOS_NO_AUTOSTART", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn atmosd serve");
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
    assert_eq!(reply["result"], "0.4.0");
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
        Command::new(env!("CARGO_BIN_EXE_atmosd"))
            .arg("daemon")
            .env("XDG_RUNTIME_DIR", run)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn atmosd daemon"),
    );
    let socket = run.join("atmos/atmosd.sock");
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
fn a_window_says_when_the_daemon_goes_away_and_samples_again_when_it_returns() {
    let run = short_run_dir("gone");
    let daemon = start_daemon(&run);
    let mut server = Server::start_live(&run);
    server.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 300}}}));
    server.until(|line| line["event"] == "backend" && line["state"] == "up");
    let before = server.until(|line| line["event"] == "sample");
    drop(daemon);
    // Told, with a reason and when it looks again; nothing is sampled here instead.
    let down = server.until(|line| line["event"] == "backend" && line["state"] == "down");
    assert!(
        down["why"].as_str().is_some_and(|why| !why.is_empty()),
        "{down}"
    );
    assert!(down["retryMs"].as_u64().unwrap() >= 1000, "{down}");
    let _daemon = start_daemon(&run);
    server.until(|line| line["event"] == "backend" && line["state"] == "up");
    let after = server.until(|line| line["event"] == "sample" && line.get("error").is_none());
    assert!(after["sampledAt"].as_u64() > before["sampledAt"].as_u64());
    server.finish();
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_window_with_no_daemon_collects_nothing_itself() {
    let run = short_run_dir("nodaemon");
    let mut server = Server::start_live(&run);
    server.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {"intervalMs": 200}}}));
    server.send(&json!({"id": 2, "op": "display.get", "kind": "live"}));
    server.send(&json!({"id": 3, "op": "write", "name": "recording.stop", "args": {}}));
    server.send(&json!({"id": 4, "op": "settings.get", "domain": "theme"}));
    let down = server.until(|line| line["event"] == "backend");
    assert_eq!(down["state"], "down", "{down}");
    let (mut read, mut write, mut setting) = (None, None, None);
    let mut samples = 0;
    for _ in 0..40 {
        let Ok(line) = server.lines.recv_timeout(Duration::from_millis(600)) else {
            break;
        };
        match line["id"].as_u64() {
            Some(2) => read = Some(line),
            Some(3) => write = Some(line),
            Some(4) => setting = Some(line),
            _ if line["event"] == "sample" => samples += 1,
            _ => {}
        }
        if read.is_some() && write.is_some() && setting.is_some() {
            break;
        }
    }
    assert_eq!(read.expect("a reply")["error"]["code"], "unavailable");
    assert_eq!(write.expect("a reply")["error"]["code"], "unavailable");
    assert_eq!(
        setting.expect("a reply")["error"]["code"],
        "unavailable",
        "a setting is not read behind the screen either"
    );
    assert_eq!(samples, 0, "no sample came from this window");
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
    let exe = run.join("atmosd-copy");
    fs::copy(env!("CARGO_BIN_EXE_atmosd"), &exe).unwrap();
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
    let socket = run.join("atmos/atmosd.sock");
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
    let next = run.join("atmosd-next");
    fs::copy(env!("CARGO_BIN_EXE_atmosd"), &next).unwrap();
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

fn daemon_status(run: &std::path::Path) -> Value {
    let stream = std::os::unix::net::UnixStream::connect(run.join("atmos/atmosd.sock")).unwrap();
    writeln!(&stream, r#"{{"op":"status"}}"#).unwrap();
    let mut line = String::new();
    BufRead::read_line(&mut BufReader::new(&stream), &mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn a_window_that_has_a_daemon_is_told_about_its_files_by_the_daemon() {
    let run = short_run_dir("stamps");
    let _daemon = start_daemon(&run);
    let file = run.join("watched.txt");
    fs::write(&file, "one").unwrap();
    let mut server = Server::start_live(&run);
    server.send(&json!({"id": 1, "op": "watch.set", "paths": [file.display().to_string()]}));
    let first = server.until(|line| line["event"] == "stamp");
    assert_eq!(first["result"]["items"][0]["text"], "one");
    // The daemon holds the watch, one for the one window.
    for _ in 0..100 {
        if daemon_status(&run)["watching"] == 1 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(daemon_status(&run)["watching"], 1);
    thread::sleep(Duration::from_millis(1100));
    fs::write(&file, "three").unwrap();
    let changed = server
        .until(|line| line["event"] == "stamp" && line["result"]["items"][0]["text"] == "three");
    assert_eq!(
        changed["result"]["items"][0]["path"],
        file.display().to_string()
    );
    server.finish();
    for _ in 0..100 {
        if daemon_status(&run)["watching"] == 0 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        daemon_status(&run)["watching"],
        0,
        "the watch ended with the window"
    );
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_window_whose_daemon_returns_hears_about_its_files_again() {
    let run = short_run_dir("stampsgone");
    let daemon = start_daemon(&run);
    let file = run.join("watched.txt");
    fs::write(&file, "one").unwrap();
    let mut server = Server::start_live(&run);
    server.send(&json!({"id": 1, "op": "watch.set", "paths": [file.display().to_string()]}));
    server.until(|line| line["event"] == "stamp");
    drop(daemon);
    server.until(|line| line["event"] == "backend" && line["state"] == "down");
    fs::write(&file, "three").unwrap();
    let _daemon = start_daemon(&run);
    // The window's watch is put to the new daemon, which reports what it finds.
    let changed = server
        .until(|line| line["event"] == "stamp" && line["result"]["items"][0]["text"] == "three");
    assert_eq!(changed["event"], "stamp");
    server.finish();
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_request_sent_as_the_window_opens_waits_for_the_session() {
    let run = short_run_dir("early");
    let _daemon = start_daemon(&run);
    let mut server = Server::start_live(&run);
    // Before the window has had time to open its session.
    server.send(&json!({"id": 1, "op": "display.get", "kind": "live"}));
    let reply = server.until(|line| line["id"] == 1);
    assert_eq!(reply["ok"], true, "{reply}");
    server.finish();
    let _ = fs::remove_dir_all(&run);
}

/// A window with its own environment: a scratch HOME and a PATH of fake tools,
/// which is what the daemon collects a settings topic with (docs/adr/0011).
struct Window {
    server: Server,
    dir: std::path::PathBuf,
}

fn fake(dir: &std::path::Path, name: &str, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("bin").join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn window_with_tools(run: &std::path::Path, name: &str) -> Window {
    let dir = std::path::PathBuf::from(format!("/tmp/rt-win-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bin")).unwrap();
    // `theme current` answers with what the test last wrote to the `theme` file.
    fake(
        &dir,
        "omarchy",
        &format!(
            r#"if [ "$*" = "theme current" ]; then
  [ -f {d}/broken ] && {{ echo 'no state' >&2; exit 3; }}
  cat {d}/theme; exit 0
fi
case "$*" in *list|"theme extras") printf 'a\nb\n' ;; *) echo ok ;; esac"#,
            d = dir.display()
        ),
    );
    fs::write(dir.join("theme"), "tokyo-night").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_atmosd"))
        .args(["--backend", "omarchy", "serve"])
        .env_clear()
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", dir.join("bin").display()),
        )
        .env("HOME", &dir)
        .env("XDG_RUNTIME_DIR", run)
        .env("ATMOS_NO_AUTOSTART", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn atmosd serve");
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
    Window {
        server: Server {
            child,
            stdin,
            lines,
        },
        dir,
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn topic_event(server: &Server, topic: &str) -> Value {
    server.until(|line| {
        line["event"] == "sample" && line["kind"] == topic && line.get("result").is_some()
    })
}

/// A daemon whose home is `home`, so the files it watches are the ones a window there reads.
fn start_daemon_in(run: &std::path::Path, home: &std::path::Path) -> Daemon {
    let daemon = Daemon(
        Command::new(env!("CARGO_BIN_EXE_atmosd"))
            .arg("daemon")
            .env("XDG_RUNTIME_DIR", run)
            .env("HOME", home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn atmosd daemon"),
    );
    let socket = run.join("atmos/atmosd.sock");
    for _ in 0..100 {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            return daemon;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("the daemon never listened");
}

/// The window's `omarchy theme current` answers with Omarchy's own state file, which the daemon
/// watches for `settings.look`.
fn theme_from_state(window: &Window, theme: &str) {
    let state = window.dir.join(".local/state/omarchy/current");
    fs::create_dir_all(&state).unwrap();
    fs::write(state.join("theme.name"), theme).unwrap();
    fake(
        &window.dir,
        "omarchy",
        r#"if [ "$*" = "theme current" ]; then cat "$HOME/.local/state/omarchy/current/theme.name"; exit 0; fi
case "$*" in *list|"theme extras") printf 'a\nb\n' ;; *) echo ok ;; esac"#,
    );
}

fn set_theme(window: &Window, theme: &str) {
    fs::write(
        window.dir.join(".local/state/omarchy/current/theme.name"),
        theme,
    )
    .unwrap();
}

/// The first settings.look sample a hold is sent, with the theme it shows. A cached sample from
/// before the change would be the first, so the first is the one that counts (docs/adr/0015).
fn hold_look(window: &mut Window, id: u64) -> Value {
    window
        .server
        .send(&json!({"id": id, "op": "sub.set", "topics": {"settings.look": {}}}));
    topic_event(&window.server, "settings.look")
}

fn release(window: &mut Window, id: u64) {
    window
        .server
        .send(&json!({"id": id, "op": "sub.set", "topics": {}}));
    window.server.until(|line| line["id"] == id);
}

#[test]
fn a_file_changed_while_no_window_held_its_topic_is_never_sent_from_the_cache() {
    let run = short_run_dir("gap");
    // The home `window_with_tools` makes, so the daemon watches the files the window's tools read.
    let home = std::path::PathBuf::from(format!("/tmp/rt-win-gap-{}", std::process::id()));
    fs::create_dir_all(&home).unwrap();
    let daemon = start_daemon_in(&run, &home);
    let mut window = window_with_tools(&run, "gap");
    assert_eq!(window.dir, home);
    theme_from_state(&window, "tokyo-night");
    assert_eq!(hold_look(&mut window, 1)["result"]["theme"], "tokyo-night");

    // Nobody holds the topic while the theme changes, so the daemon's watcher is idle.
    release(&mut window, 2);
    thread::sleep(Duration::from_millis(1500));
    set_theme(&window, "gruvbox-material");
    thread::sleep(Duration::from_millis(1500));
    let first = hold_look(&mut window, 3);
    assert_eq!(
        first["result"]["theme"], "gruvbox-material",
        "the cached topic was sent: {first}"
    );

    // The daemon restarts with that sample on disk, and the theme changes while none runs.
    release(&mut window, 4);
    drop(daemon);
    window
        .server
        .until(|line| line["event"] == "backend" && line["state"] == "down");
    set_theme(&window, "kanagawa");
    let _daemon = start_daemon_in(&run, &home);
    window
        .server
        .until(|line| line["event"] == "backend" && line["state"] == "up");
    let first = hold_look(&mut window, 5);
    assert_eq!(
        first["result"]["theme"], "kanagawa",
        "the earlier daemon's sample was sent: {first}"
    );
    drop(window);
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_window_that_holds_a_settings_topic_is_pushed_it_and_the_next_one_when_it_changes() {
    let run = short_run_dir("topic");
    let _daemon = start_daemon(&run);
    let mut window = window_with_tools(&run, "topic");
    window
        .server
        .send(&json!({"id": 1, "op": "sub.set", "topics": {"settings.look": {}}}));
    let first = topic_event(&window.server, "settings.look");
    assert_eq!(first["result"]["theme"], "tokyo-night", "{first}");
    // Other tools of the topic may fail on a machine with no session; the theme did not.
    assert!(first["result"]["_failed"].get("theme").is_none(), "{first}");
    // Something changed the theme; the window asks for the topic again.
    fs::write(window.dir.join("theme"), "gruvbox").unwrap();
    window
        .server
        .send(&json!({"id": 2, "op": "snapshot.invalidate", "topics": ["settings.look"]}));
    let next = window
        .server
        .until(|line| line["event"] == "sample" && line["result"]["theme"] == "gruvbox");
    assert!(next["sampledAt"].as_u64() > first["sampledAt"].as_u64());
    drop(window);
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_tool_that_fails_leaves_a_key_out_of_the_pushed_topic_and_says_why() {
    let run = short_run_dir("topicfail");
    let _daemon = start_daemon(&run);
    let mut window = window_with_tools(&run, "topicfail");
    fs::write(window.dir.join("broken"), "").unwrap();
    window
        .server
        .send(&json!({"id": 1, "op": "sub.set", "topics": {"settings.look": {}}}));
    let event = topic_event(&window.server, "settings.look");
    assert!(
        event["result"].get("theme").is_none(),
        "no value was made up: {event}"
    );
    let why = event["result"]["_failed"]["theme"]
        .as_str()
        .expect("a reason");
    assert!(why.contains("theme current"), "{why}");
    assert!(
        event["result"].get("monitors").is_some()
            || event["result"]["_failed"].get("monitors").is_some()
    );
    drop(window);
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_setting_written_by_a_window_comes_back_in_the_topic_that_holds_it() {
    let run = short_run_dir("topicset");
    let _daemon = start_daemon(&run);
    let mut window = window_with_tools(&run, "topicset");
    window
        .server
        .send(&json!({"id": 1, "op": "sub.set", "topics": {"settings.look": {}}}));
    let first = topic_event(&window.server, "settings.look");
    assert!(
        first["result"].get("hyprLook").is_none() || first["result"]["hyprLook"]["gapsIn"] != 4
    );
    window
        .server
        .send(&json!({"id": 2, "op": "settings.set", "domain": "hyprLook.gapsIn", "value": 4}));
    let reply = window.server.until(|line| line["id"] == 2);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(
        reply["result"]["coalesced"], false,
        "it went through a lane of the daemon"
    );
    // The daemon invalidated the topic the setting lives in and pushed it again.
    let after = window
        .server
        .until(|line| line["event"] == "sample" && line["result"]["hyprLook"]["gapsIn"] == 4);
    assert!(after["sampledAt"].as_u64() > first["sampledAt"].as_u64());
    drop(window);
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn opening_a_file_from_a_window_replies_at_once_whatever_the_opener_does() {
    let run = short_run_dir("opener");
    let _daemon = start_daemon(&run);
    let mut window = window_with_tools(&run, "opener");
    // An opener that stays up as long as the app it starts.
    fake(&window.dir, "xdg-open", "sleep 30");
    fs::write(window.dir.join("note.txt"), "x").unwrap();
    let began = std::time::Instant::now();
    window.server.send(&json!({
        "id": 1, "op": "host.open",
        "path": window.dir.join("note.txt").display().to_string(),
    }));
    let reply = window.server.until(|line| line["id"] == 1);
    assert_eq!(reply["ok"], true, "{reply}");
    assert!(
        began.elapsed() < Duration::from_secs(5),
        "{:?}",
        began.elapsed()
    );
    drop(window);
    let _ = fs::remove_dir_all(&run);
}

fn commands_run(run: &std::path::Path) -> u64 {
    daemon_status(run)["commandsRun"].as_u64().unwrap()
}

/// Every op a window can send is answered by the daemon, except the three that
/// only ask it for something it already holds (docs/adr/0012): `serve` reads no
/// file, runs no tool and collects nothing itself.
#[test]
fn every_op_a_window_sends_is_answered_by_the_daemon() {
    let run = short_run_dir("bridge");
    let _daemon = start_daemon(&run);
    let mut window = window_with_tools(&run, "bridge");
    fake(&window.dir, "systemctl", "echo active");
    fake(&window.dir, "journalctl", "echo log");
    let note = window.dir.join("note.txt");
    fs::write(&note, "x").unwrap();
    let path = note.display().to_string();
    let bodies = vec![
        json!({"op": "version"}),
        json!({"op": "platform"}),
        json!({"op": "settings.list"}),
        json!({"op": "settings.get", "domain": "theme"}),
        json!({"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 3}),
        json!({"op": "settings.snapshot", "group": "look", "keys": ["theme"]}),
        json!({"op": "snapshot.keys", "keys": ["hostname"]}),
        json!({"op": "snapshot.topic", "topic": "settings.accounts"}),
        json!({"op": "write", "name": "audio.output.mute-toggle", "args": {}}),
        json!({"op": "display.scale", "scale": "1.5"}),
        json!({"op": "display.brightness", "monitor": "DP-3", "percent": 20}),
        json!({"op": "search.query", "query": "bright"}),
        json!({"op": "host.chrome"}),
        json!({"op": "host.themePack", "name": "x", "home": window.dir.display().to_string()}),
        json!({"op": "host.accounts", "user": "u", "home": window.dir.display().to_string()}),
        json!({"op": "host.stamp", "paths": [path]}),
        json!({"op": "host.read", "paths": [path]}),
        json!({"op": "host.write", "path": window.dir.join("out.txt").display().to_string(), "text": "t"}),
        json!({"op": "host.open", "path": path}),
        json!({"op": "unit.output", "kind": "status", "scope": "user", "unit": "x.service"}),
        json!({"op": "speedtest.disk"}),
        json!({"op": "speedtest.net", "phase": "down"}),
        json!({"op": "agents.mcp.list"}),
        json!({"op": "agents.mcp.check"}),
        json!({"op": "agents.mcp.set", "agent": "nobody", "on": false}),
    ];
    fake(&window.dir, "xdg-open", "true");
    // Nothing is left out: every op `request.rs` accepts is in one list or the other.
    let native = ["display.get", "display.snapshot", "snapshot.invalidate"];
    let accepted: Vec<&str> = include_str!("../src/request.rs")
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("#[serde(rename = \"")?
                .strip_suffix("\")]")
        })
        .collect();
    for op in &accepted {
        assert!(
            native.contains(op)
                || bodies.iter().any(|body| body["op"] == *op)
                || *op == "display.scale"
                || *op == "display.brightness",
            "{op} is not tested over a window's serve"
        );
    }
    for (n, body) in bodies.iter().enumerate() {
        let before = commands_run(&run);
        let id = 100 + n as u64;
        let mut asked = body.clone();
        asked["id"] = json!(id);
        window.server.send(&asked);
        let reply = window.server.until(|line| line["id"] == id);
        assert!(reply.get("ok").is_some(), "{body}: {reply}");
        assert_eq!(
            commands_run(&run),
            before + 1,
            "the daemon ran {body} (reply {reply})"
        );
    }
    // These three only ask the daemon for what it holds.
    for body in [
        json!({"op": "display.get", "kind": "services"}),
        json!({"op": "display.snapshot"}),
        json!({"op": "snapshot.invalidate", "topics": ["settings.look"]}),
    ] {
        let before = commands_run(&run);
        let mut asked = body.clone();
        asked["id"] = json!(900);
        window.server.send(&asked);
        window.server.until(|line| line["id"] == 900);
        assert_eq!(commands_run(&run), before, "serve answered {body} itself");
    }
    drop(window);
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn a_window_refuses_a_daemon_too_old_to_run_its_reads() {
    use std::os::unix::net::UnixListener;
    let run = short_run_dir("oldproto");
    fs::create_dir_all(run.join("atmos")).unwrap();
    // A daemon from before reads were its job: it says `ready` and nothing about a protocol.
    let listener = UnixListener::bind(run.join("atmos/atmosd.sock")).unwrap();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            thread::spawn(move || {
                let mut line = String::new();
                let _ = BufReader::new(&stream).read_line(&mut line);
                let mut out = &stream;
                let _ = writeln!(out, r#"{{"ok":true,"t":"ready","version":"0.0.1"}}"#);
                thread::sleep(Duration::from_secs(5));
            });
        }
    });
    let mut server = Server::start_live(&run);
    server.send(&json!({"id": 1, "op": "sub.set", "topics": {"live": {}}}));
    let down = server.until(|line| line["event"] == "backend" && line["state"] == "down");
    assert!(
        down["why"]
            .as_str()
            .unwrap()
            .contains("older than this window"),
        "{down}"
    );
    let _ = fs::remove_dir_all(&run);
}

/// The processes whose parent is `pid`, as `(pid, command line)`, from `/proc`.
fn children_of(pid: u32) -> Vec<(u32, String)> {
    let mut found = Vec::new();
    for entry in fs::read_dir("/proc").unwrap().flatten() {
        let Ok(child) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Ok(stat) = fs::read_to_string(format!("/proc/{child}/stat")) else {
            continue;
        };
        // `pid (comm) state ppid ...`, where comm may hold spaces and parentheses.
        let Some((_, after)) = stat.rsplit_once(')') else {
            continue;
        };
        if after
            .split_whitespace()
            .nth(1)
            .and_then(|p| p.parse::<u32>().ok())
            == Some(pid)
        {
            let cmd = fs::read_to_string(format!("/proc/{child}/cmdline"))
                .unwrap_or_default()
                .replace('\0', " ");
            found.push((child, cmd));
        }
    }
    found
}

/// Whether `pid` is still a process that runs: gone, or only waiting to be reaped.
fn alive(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|stat| {
            stat.rsplit_once(')')
                .and_then(|(_, after)| after.split_whitespace().next().map(|s| s != "Z"))
        })
        .unwrap_or(false)
}

/// A daemon with `flags`, and `path` as its `PATH` when given.
fn start_daemon_with(
    run: &std::path::Path,
    flags: &[&str],
    path: Option<&std::path::Path>,
) -> Daemon {
    let mut command = Command::new(env!("CARGO_BIN_EXE_atmosd"));
    command
        .arg("daemon")
        .args(flags)
        .env("XDG_RUNTIME_DIR", run);
    if let Some(path) = path {
        command.env("PATH", path);
    }
    let daemon = Daemon(
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn atmosd daemon"),
    );
    let socket = run.join("atmos/atmosd.sock");
    for _ in 0..100 {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            return daemon;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("the daemon never listened");
}

/// With `--with-sampler` the daemon starts a sampler of its own, a child of it at the lowest
/// priorities, and the sampler ends when the daemon does: it must not wait for the next one for
/// ever, or a replaced daemon would start a second beside the old (docs/adr/0014).
#[test]
fn a_daemon_with_a_sampler_of_its_own_starts_it_and_it_leaves_with_the_daemon() {
    let run = short_run_dir("owned");
    // Only `nice` is on its PATH: the sampler starts, and what it would collect (`lsblk`,
    // `systemctl`) is not there to run, so it costs nothing while the other tests run beside it.
    let tools = run.join("tools");
    fs::create_dir_all(&tools).unwrap();
    let nice = ["/usr/bin/nice", "/bin/nice"]
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
        .expect("nice");
    std::os::unix::fs::symlink(nice, tools.join("nice")).unwrap();
    let mut daemon = start_daemon_with(&run, &["--with-sampler"], Some(&tools));

    let mut sampler = None;
    for _ in 0..150 {
        let status = daemon_status(&run);
        if status["sampler"]["connected"] == true {
            sampler = status["sampler"]["pid"].as_u64().map(|p| p as u32);
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    let sampler = sampler.expect("a sampler connected to the daemon");
    let children = children_of(daemon.0.id());
    let own = children
        .iter()
        .find(|(pid, _)| *pid == sampler)
        .expect("the sampler is the daemon's child");
    assert!(own.1.contains("sampler --owned"), "{own:?}");
    assert_eq!(
        children.len(),
        1,
        "one sampler and nothing else: {children:?}"
    );

    // The daemon is killed without a chance to say goodbye.
    daemon.0.kill().unwrap();
    daemon.0.wait().unwrap();
    let until = std::time::Instant::now() + Duration::from_secs(10);
    while alive(sampler) && std::time::Instant::now() < until {
        thread::sleep(Duration::from_millis(100));
    }
    assert!(!alive(sampler), "the sampler outlived its daemon");
    let _ = fs::remove_dir_all(&run);
}

/// Without the flag nothing is started: an install's units start the sampler, or a hand does,
/// and a daemon that started one as well would sample twice.
#[test]
fn a_daemon_without_the_flag_starts_no_sampler() {
    let run = short_run_dir("unowned");
    let daemon = start_daemon_with(&run, &[], None);
    // Longer than the grace the keeper would take to start one.
    thread::sleep(Duration::from_millis(2500));
    assert_eq!(daemon_status(&run)["sampler"]["connected"], false);
    assert!(children_of(daemon.0.id()).is_empty());
    let _ = fs::remove_dir_all(&run);
}

#[test]
fn the_daemon_and_the_sampler_take_only_their_own_flags() {
    let run = short_run_dir("flags");
    for args in [["daemon", "--nope"], ["sampler", "--nope"]] {
        let out = Command::new(env!("CARGO_BIN_EXE_atmosd"))
            .args(args)
            .env("XDG_RUNTIME_DIR", &run)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("takes only"),
            "{args:?}"
        );
    }
    let _ = fs::remove_dir_all(&run);
}

/// What a write reads back is what the topic shows next, and a value Atmos stored once never
/// covers what the machine reads now (docs/adr/0016).
#[test]
fn a_setting_written_by_a_window_reads_back_what_its_topic_then_shows() {
    let run = short_run_dir("agree");
    let home = std::path::PathBuf::from(format!("/tmp/rt-win-agree-{}", std::process::id()));
    fs::create_dir_all(&home).unwrap();
    let _daemon = start_daemon_in(&run, &home);
    let mut window = window_with_tools(&run, "agree");
    // Atmos once turned middle-click paste off; the GTK file it wrote has since gone, so the
    // machine pastes again.
    fs::create_dir_all(window.dir.join(".config/omarchy")).unwrap();
    fs::write(
        window.dir.join(".config/omarchy/tweaks.json"),
        r#"{"tweaks.middlePaste": true}"#,
    )
    .unwrap();
    window
        .server
        .send(&json!({"id": 1, "op": "sub.set", "topics": {"settings.system": {}}}));
    let first = topic_event(&window.server, "settings.system");
    assert_eq!(
        first["result"]["tweaks"]["middlePaste"], false,
        "the GTK file, not tweaks.json: {}",
        first["result"]["tweaks"]
    );

    window.server.send(&json!({"id": 2, "op": "settings.set",
        "domain": "tweaks.electronWayland", "value": false, "readback": ["tweaks"]}));
    let reply = window.server.until(|line| line["id"] == 2);
    assert_eq!(reply["ok"], true, "{reply}");
    let read_back = reply["result"]["snapshot"]["tweaks"].clone();
    assert_eq!(read_back["electronWayland"], false, "{reply}");
    let next = window.server.until(|line| {
        line["event"] == "sample"
            && line["kind"] == "settings.system"
            && line["sampledAt"].as_u64() > first["sampledAt"].as_u64()
    });
    assert_eq!(
        next["result"]["tweaks"], read_back,
        "the reply and the next sample disagree"
    );
    drop(window);
    let _ = fs::remove_dir_all(&run);
}
