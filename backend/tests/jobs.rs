//! Jobs end to end: a real daemon, real `ratmos serve` windows, fake tools on
//! the window's PATH. The runner is the direct one: a test never reaches the
//! real user manager.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

struct Scene {
    run: PathBuf,
    fakes: PathBuf,
}

impl Scene {
    fn new(name: &str) -> Scene {
        let run = PathBuf::from(format!("/tmp/rt-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&run);
        let fakes = run.join("fakes");
        fs::create_dir_all(&fakes).unwrap();
        Scene { run, fakes }
    }

    fn tool(&self, name: &str, body: &str) {
        let path = self.fakes.join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn env(&self, command: &mut Command) {
        command
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", self.fakes.display()))
            .env("HOME", &self.run)
            .env("XDG_RUNTIME_DIR", &self.run)
            .env("ATMOS_NO_AUTOSTART", "1");
    }

    fn daemon(&self, exe: &Path) -> Child {
        let mut command = Command::new(exe);
        command
            .arg("daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        self.env(&mut command);
        let mut child = command.spawn().unwrap();
        let socket = self.run.join("atmos/ratmos.sock");
        for _ in 0..100 {
            if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
                return child;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        panic!("the daemon never listened");
    }

    fn window(&self) -> Window {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ratmos"));
        command
            .args(["--backend", "omarchy", "serve"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        self.env(&mut command);
        let mut child = command.spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    let _ = tx.send(value);
                }
            }
        });
        let window = Window {
            child,
            stdin,
            lines,
        };
        // The session is opened when the window starts.
        for _ in 0..100 {
            if status(&self.run)["sessions"].as_u64().unwrap_or(0) >= 1 {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        window
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.run);
    }
}

struct Window {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<Value>,
}

impl Window {
    fn send(&mut self, value: Value) {
        writeln!(self.stdin, "{value}").unwrap();
    }

    /// Job events for `cid` until one of `kind`.
    fn until(&self, cid: &str, kind: &str) -> Vec<Value> {
        let mut seen = Vec::new();
        loop {
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(15))
                .expect("a line from serve");
            if line["event"] == "job" && line["cid"] == cid {
                let done = line["kind"] == kind;
                seen.push(line);
                if done {
                    return seen;
                }
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn status(run: &Path) -> Value {
    let stream = std::os::unix::net::UnixStream::connect(run.join("atmos/ratmos.sock")).unwrap();
    writeln!(&stream, r#"{{"op":"status"}}"#).unwrap();
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn lines_of(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .filter(|e| e["kind"] == "stream")
        .map(|e| e["line"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_window_starts_a_job_by_name_and_follows_it() {
    let scene = Scene::new("jobs-follow");
    scene.tool("omarchy", "echo \"args: $*\"; echo oops >&2; exit 3");
    let mut daemon = scene.daemon(Path::new(env!("CARGO_BIN_EXE_ratmos")));
    let mut window = scene.window();
    window.send(
        json!({"id": 1, "op": "job.start", "cid": "j1", "label": "theme-install",
        "name": "theme.install", "args": {"url": "https://example.org/t.git"}}),
    );
    let events = window.until("j1", "exit");
    assert_eq!(events[0]["kind"], "started");
    // Output and error are two pipes read at once: each keeps its own order,
    // and nothing says which arrives first.
    let mut lines = lines_of(&events);
    lines.sort();
    assert_eq!(
        lines,
        ["args: theme install https://example.org/t.git", "oops"]
    );
    let end = events.last().unwrap();
    assert_eq!(end["code"], 3);
    assert_eq!(end["summary"]["message"], "oops");
    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn an_unknown_job_and_a_bad_argument_end_at_once_without_running_anything() {
    let scene = Scene::new("jobs-refuse");
    scene.tool("omarchy", "echo ran >> /tmp/never-ran-jobs-refuse");
    let mut daemon = scene.daemon(Path::new(env!("CARGO_BIN_EXE_ratmos")));
    let mut window = scene.window();
    window.send(json!({"id": 1, "op": "job.start", "cid": "a", "name": "no.such.job", "args": {}}));
    let first = window.until("a", "exit");
    assert!(first.last().unwrap()["summary"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown job"));
    window.send(json!({"id": 2, "op": "job.start", "cid": "b", "name": "theme.install", "args": {"url": "--help"}}));
    let second = window.until("b", "exit");
    assert_eq!(second.last().unwrap()["code"], 2);
    assert!(!Path::new("/tmp/never-ran-jobs-refuse").exists());
    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn a_job_outlives_its_window_and_a_new_window_picks_it_up() {
    let scene = Scene::new("jobs-reopen");
    scene.tool("omarchy", "echo first; sleep 1.5; echo second");
    let mut daemon = scene.daemon(Path::new(env!("CARGO_BIN_EXE_ratmos")));
    let job = {
        let mut window = scene.window();
        window.send(
            json!({"id": 1, "op": "job.start", "cid": "j", "label": "theme-update",
            "name": "theme.update", "args": {}}),
        );
        let started = window.until("j", "started");
        let job = started[0]["job"].as_u64().unwrap();
        loop {
            let line = window.lines.recv_timeout(Duration::from_secs(10)).unwrap();
            if line["line"] == "first" {
                break;
            }
        }
        job
        // The window is dropped here.
    };
    assert_eq!(
        status(&scene.run)["jobsRunning"],
        1,
        "the job is still running with nobody listening"
    );
    let mut again = scene.window();
    again.send(json!({"id": 2, "op": "job.list"}));
    let listed = loop {
        let line = again.lines.recv_timeout(Duration::from_secs(10)).unwrap();
        if line["id"] == 2 {
            break line;
        }
    };
    assert_eq!(
        listed["result"]["jobs"][0]["label"], "theme-update",
        "{listed}"
    );
    again.send(json!({"id": 3, "op": "job.attach", "job": job, "from": 0}));
    let events = again.until("j", "exit");
    assert_eq!(
        lines_of(&events),
        ["first", "second"],
        "everything it printed, from the start"
    );
    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn a_running_job_is_cancelled_by_a_request() {
    let scene = Scene::new("jobs-cancel");
    scene.tool("omarchy", "echo waiting; sleep 30");
    let mut daemon = scene.daemon(Path::new(env!("CARGO_BIN_EXE_ratmos")));
    let mut window = scene.window();
    window.send(
        json!({"id": 1, "op": "job.start", "cid": "w", "name": "tailscale.receive", "args": {}}),
    );
    let started = window.until("w", "started");
    let job = started[0]["job"].as_u64().unwrap();
    loop {
        if window.lines.recv_timeout(Duration::from_secs(10)).unwrap()["line"] == "waiting" {
            break;
        }
    }
    window.send(json!({"id": 2, "op": "job.cancel", "job": job}));
    let began = Instant::now();
    let end = window.until("w", "exit");
    assert!(began.elapsed() < Duration::from_secs(5));
    assert_ne!(end.last().unwrap()["code"], 0);
    let _ = daemon.kill();
    let _ = daemon.wait();
}

#[test]
fn with_no_daemon_a_job_cannot_start_and_the_window_is_told() {
    let scene = Scene::new("jobs-nodaemon");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ratmos"));
    command
        .args(["--backend", "omarchy", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    scene.env(&mut command);
    let mut child = command.spawn().unwrap();
    writeln!(
        child.stdin.as_mut().unwrap(),
        "{}",
        json!({"id": 1, "op": "job.start", "cid": "x", "name": "theme.update", "args": {}})
    )
    .unwrap();
    // The window is told the daemon is down, and its job is refused.
    let reply = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .find(|line| line["id"] == 1)
        .expect("a reply");
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(reply["error"]["code"], "unavailable", "{reply}");
    let _ = child.kill();
    let _ = child.wait();
}

/// A daemon run from its own copy of the binary, so the test can replace it.
fn copy_of_ratmos(scene: &Scene) -> PathBuf {
    let exe = scene.run.join("ratmos-copy");
    fs::copy(env!("CARGO_BIN_EXE_ratmos"), &exe).unwrap();
    exe
}

#[test]
fn a_daemon_does_not_replace_itself_while_a_job_runs() {
    let scene = Scene::new("jobs-defer");
    scene.tool("omarchy", "echo going; sleep 8; echo done");
    let exe = copy_of_ratmos(&scene);
    // A binary just written can still be open for writing elsewhere: exec says
    // "text file busy" until it is not.
    let mut daemon = None;
    for _ in 0..40 {
        let mut command = Command::new(&exe);
        command
            .arg("daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        scene.env(&mut command);
        match command.spawn() {
            Ok(child) => {
                daemon = Some(child);
                break;
            }
            Err(err) if err.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                thread::sleep(Duration::from_millis(50))
            }
            Err(err) => panic!("{err}"),
        }
    }
    let mut daemon = daemon.expect("the copy started");
    let socket = scene.run.join("atmos/ratmos.sock");
    for _ in 0..100 {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let mut window = scene.window();
    window
        .send(json!({"id": 1, "op": "job.start", "cid": "j", "name": "theme.update", "args": {}}));
    window.until("j", "started");
    // A rebuild: a new file renamed over the old one.
    let next = scene.run.join("ratmos-next");
    fs::copy(env!("CARGO_BIN_EXE_ratmos"), &next).unwrap();
    fs::rename(&next, &exe).unwrap();
    thread::sleep(Duration::from_secs(6));
    assert!(
        daemon.try_wait().unwrap().is_none(),
        "the daemon left while a job was running"
    );
    window.until("j", "exit");
    let began = Instant::now();
    loop {
        if daemon.try_wait().unwrap().is_some() {
            break;
        }
        assert!(
            began.elapsed() < Duration::from_secs(12),
            "the daemon never left after the job"
        );
        thread::sleep(Duration::from_millis(200));
    }
}
