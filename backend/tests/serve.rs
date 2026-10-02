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
    assert_eq!(reply["result"], "0.1.0");
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
