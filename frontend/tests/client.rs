//! The client against a scripted server (a socket pair, so what it does on a restart is exact) and
//! against the real `ratmos serve`, in a private HOME and runtime directory with autostart off so
//! it cannot reach the daemon of the session it runs in.

mod common;

use std::path::{Path, PathBuf};

use atmos_ui::client::{Client, Link, Notice, ServeProcess, locate};
use atmos_ui::protocol::{self, Code, Daemon, Event};
use common::{block_on, next, script};
use serde_json::json;

#[test]
fn a_reply_finds_the_request_it_answers_whatever_the_order() {
    let (transport, mut servers) = script(1);
    let mut server = servers.remove(0);
    let (client, notices) = Client::start(transport);
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert_eq!(next(&notices), Notice::Link(Link::Up));

    let first = client.request(&protocol::version());
    let second = client.request(&protocol::platform());
    let (a, b) = (server.read(), server.read());
    assert_eq!(
        (a["op"].as_str(), b["op"].as_str()),
        (Some("version"), Some("platform"))
    );
    // Reply to the second first, as the server may.
    server.say(json!({"id": b["id"], "ok": true, "result": "two"}));
    server.say(json!({"id": a["id"], "ok": true, "result": "one"}));
    assert_eq!(block_on(first), Ok(json!("one")));
    assert_eq!(block_on(second), Ok(json!("two")));
}

#[test]
fn events_arrive_on_the_channel_and_a_failure_keeps_its_code() {
    let (transport, mut servers) = script(1);
    let mut server = servers.remove(0);
    let (client, notices) = Client::start(transport);
    next(&notices);
    next(&notices);

    server.say(json!({"event": "backend", "state": "down", "why": "no daemon", "attempt": 1, "retryMs": 1000}));
    assert_eq!(
        next(&notices),
        Notice::Event(Event::Backend(Daemon::Down {
            why: "no daemon".into(),
            attempt: 1,
            retry_ms: 1000
        }))
    );

    let reply = client.request(&json!({"op": "nope"}));
    let sent = server.read();
    server.say(json!({"id": sent["id"], "ok": false, "result": null,
        "error": {"code": "bad_request", "message": "unknown op", "context": null}}));
    assert_eq!(block_on(reply).unwrap_err().code, Code::BadRequest);
}

#[test]
fn a_stopped_server_fails_what_waits_and_comes_back_with_what_the_window_holds() {
    let (transport, mut servers) = script(2);
    let mut second = servers.pop().unwrap();
    let mut first = servers.pop().unwrap();
    let (client, notices) = Client::start(transport);
    next(&notices);
    next(&notices);

    let held = client.hold(&[("live", 2000)]);
    let sent = first.read();
    assert_eq!(sent["op"], "sub.set");
    first.say(json!({"id": sent["id"], "ok": true, "result": {"holding": ["live"]}}));
    assert_eq!(block_on(held), Ok(json!({"holding": ["live"]})));

    let waiting = client.request(&protocol::version());
    first.read();
    drop(first);

    let failure = block_on(waiting).unwrap_err();
    assert_eq!(failure.code, Code::Unavailable, "{failure:?}");
    assert_eq!(
        next(&notices),
        Notice::Link(Link::Down {
            failures: 1,
            retry_ms: 1000
        })
    );
    // Down means nothing is sent: a request is refused, not queued.
    assert_eq!(
        block_on(client.request(&protocol::version()))
            .unwrap_err()
            .code,
        Code::Unavailable
    );
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert_eq!(next(&notices), Notice::Link(Link::Up));
    let again = second.read();
    assert_eq!(again["op"], "sub.set");
    assert_eq!(again["topics"]["live"]["intervalMs"], 2000);
}

#[test]
fn a_server_that_cannot_start_is_unavailable_and_tried_again() {
    let (transport, _) = script(0);
    let (client, notices) = Client::start(transport);
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert_eq!(
        next(&notices),
        Notice::Link(Link::Down {
            failures: 1,
            retry_ms: 1000
        })
    );
    assert_eq!(
        block_on(client.request(&protocol::version()))
            .unwrap_err()
            .code,
        Code::Unavailable
    );
    // It looks again, finds nothing, and backs off.
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert_eq!(
        next(&notices),
        Notice::Link(Link::Down {
            failures: 2,
            retry_ms: 2000
        })
    );
}

#[test]
fn a_program_that_is_not_there_is_the_same_unavailable() {
    let (client, notices) =
        Client::start(ServeProcess::new("/definitely/not/ratmos", "plain", &[]));
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert!(matches!(
        next(&notices),
        Notice::Link(Link::Down { failures: 1, .. })
    ));
    assert_eq!(
        block_on(client.request(&protocol::version()))
            .unwrap_err()
            .code,
        Code::Unavailable
    );
}

fn ratmos() -> Option<PathBuf> {
    // `tests/frontend` builds the debug binary and names it in `ATMOS_BACKEND`. Without that, the
    // debug build is the only one tried: a release build or `bin/ratmos` can be a stale copy.
    let debug = Path::new(env!("CARGO_MANIFEST_DIR")).join("../backend/target/debug/ratmos");
    let explicit = std::env::var_os("ATMOS_BACKEND")
        .map(PathBuf::from)
        .unwrap_or(debug);
    let found = locate(Some(explicit), None);
    if found.is_absolute() {
        Some(found)
    } else {
        eprintln!("skipped: no built ratmos (cargo build --manifest-path backend/Cargo.toml)");
        None
    }
}

/// A private HOME and runtime directory, removed afterwards.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("atmos-ui-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    /// A server that can reach nothing of the session it runs in: its own home and runtime
    /// directory, no daemon to start, and no tool on its PATH (`omarchy`, `hyprctl` and the rest
    /// are the machine's, and none of these tests needs one).
    fn serve(&self, bin: &Path, backend: &str, extra: &[&str]) -> ServeProcess {
        let tools = self.0.join("no-tools");
        std::fs::create_dir_all(&tools).unwrap();
        ServeProcess::new(bin, backend, extra)
            .env("PATH", tools.into_os_string())
            .env("HOME", self.0.as_os_str())
            .env("XDG_RUNTIME_DIR", self.0.as_os_str())
            .env("XDG_CONFIG_HOME", self.0.join("config").into_os_string())
            .env("ATMOS_NO_AUTOSTART", "1")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_real_server_answers_version_and_refuses_an_unknown_op() {
    let Some(bin) = ratmos() else { return };
    let scratch = Scratch::new("fixture");
    let root = scratch.0.join("root");
    std::fs::create_dir_all(&root).unwrap();
    let (client, notices) =
        Client::start(scratch.serve(&bin, "plain", &["--root", root.to_str().unwrap()]));
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert_eq!(next(&notices), Notice::Link(Link::Up));

    let version = block_on(client.request(&protocol::version())).unwrap();
    assert!(
        version.as_str().is_some_and(|v| v.contains('.')),
        "{version}"
    );
    let platform = block_on(client.request(&protocol::platform())).unwrap();
    assert_eq!(platform["id"], "plain");
    let unknown = block_on(client.request(&json!({"op": "nope"}))).unwrap_err();
    assert_eq!(unknown.code, Code::BadRequest);
    let none = block_on(client.hold(&[])).unwrap();
    assert_eq!(none["holding"], json!([]));
}

#[test]
fn without_a_daemon_the_real_server_says_so_and_refuses_what_needs_one() {
    let Some(bin) = ratmos() else { return };
    let scratch = Scratch::new("nodaemon");
    // The omarchy backend needs the daemon; the plain one samples for itself and never says so.
    let (client, notices) = Client::start(scratch.serve(&bin, "omarchy", &[]));
    assert_eq!(next(&notices), Notice::Link(Link::Starting));
    assert_eq!(next(&notices), Notice::Link(Link::Up));

    // The server says the daemon is not there: the window shows its unavailable screen.
    let mut down = None;
    while down.is_none() {
        if let Notice::Event(Event::Backend(daemon)) = next(&notices) {
            down = Some(daemon);
        }
    }
    assert!(
        matches!(
            &down,
            Some(Daemon::Down {
                attempt: 1,
                retry_ms: 1000,
                ..
            })
        ),
        "{down:?}"
    );

    // And a job, which only a daemon can run, is refused rather than run in the window.
    let job = protocol::job_start("j1", "update.omarchy-check", &json!({}), "check");
    let refused = block_on(client.request(&job)).unwrap_err();
    assert_eq!(refused.code, Code::Unavailable, "{refused:?}");
}
