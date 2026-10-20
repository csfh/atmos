//! The command bus end to end: real daemon, real `ratmos request` children,
//! fake `omarchy` and `hyprctl` on the caller's PATH.

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

struct Scene {
    run: PathBuf,
    fakes: PathBuf,
    log: PathBuf,
}

impl Scene {
    fn new(name: &str) -> Scene {
        let run = PathBuf::from(format!("/tmp/rt-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&run);
        let fakes = run.join("fakes");
        fs::create_dir_all(&fakes).unwrap();
        let log = run.join("omarchy.log");
        let script = |name: &str, body: &str| {
            let path = fakes.join(name);
            fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        };
        // Setting writes a start and an end line, slowly; reading prints a level.
        script(
            "omarchy",
            &format!(
                r#"case "$*" in
  *%) echo "start $*" >> {log}; sleep 0.4; echo "end $*" >> {log} ;;
  *) echo "55" ;;
esac"#,
                log = log.display()
            ),
        );
        script(
            "hyprctl",
            r#"echo '[{"name":"DP-3","description":"x","make":"m","model":"m","serial":"s","width":1920,"height":1080,"refreshRate":60.0,"x":0,"y":0,"scale":1.0,"focused":true,"disabled":false,"dpmsStatus":true,"vrr":false,"transform":0,"mirrorOf":"none","availableModes":["1920x1080@60.00Hz"],"id":0}]'"#,
        );
        Scene { run, fakes, log }
    }

    fn path(&self) -> String {
        format!("{}:/usr/bin:/bin", self.fakes.display())
    }

    fn daemon(&self) -> Daemon {
        let child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .arg("daemon")
            .env("XDG_RUNTIME_DIR", &self.run)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let guard = Daemon(child);
        let socket = self.run.join("atmos/ratmos.sock");
        for _ in 0..100 {
            if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
                return guard;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("the daemon never listened");
    }

    /// `ratmos request` as a window's serve would run it, in the scene's environment.
    fn request(&self, body: &Value) -> Value {
        request_in(&self.run, &self.path(), body)
    }

    fn log(&self) -> Vec<String> {
        fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.run);
    }
}

struct Daemon(Child);

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn request_in(run: &Path, path: &str, body: &Value) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
        .args(["--backend", "omarchy", "request"])
        .env_clear()
        .env("XDG_RUNTIME_DIR", run)
        .env("PATH", path)
        .env("HOME", run)
        .env("ATMOS_NO_AUTOSTART", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|_| panic!("not an envelope: {}", String::from_utf8_lossy(&out.stdout)))
}

fn brightness(percent: u32) -> Value {
    json!({"op": "display.brightness", "monitor": "DP-3", "percent": percent})
}

#[test]
fn brightness_through_the_daemon_runs_the_tool_and_returns_the_machines_monitor_list() {
    let scene = Scene::new("cmd-e2e");
    let _daemon = scene.daemon();
    let reply = scene.request(&brightness(40));
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["percent"], 40);
    assert_eq!(reply["result"]["coalesced"], false);
    let monitors = &reply["result"]["snapshot"]["monitors"];
    assert_eq!(monitors[0]["name"], "DP-3", "{reply}");
    assert_eq!(monitors[0]["brightness"], 55, "read back, not guessed");
    assert_eq!(
        scene.log(),
        [
            "start brightness display --no-osd --monitor DP-3 40%",
            "end brightness display --no-osd --monitor DP-3 40%"
        ]
    );
}

#[test]
fn brightness_without_a_daemon_runs_in_place() {
    let scene = Scene::new("cmd-inline");
    let reply = scene.request(&brightness(25));
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["percent"], 25);
    assert!(reply["result"].get("coalesced").is_none());
    assert_eq!(scene.log().len(), 2);
}

#[test]
fn a_failing_tool_is_an_error_not_a_new_monitor_list() {
    let scene = Scene::new("cmd-fail");
    let omarchy = scene.fakes.join("omarchy");
    fs::write(&omarchy, "#!/bin/sh\necho 'no ddc bus' >&2\nexit 3\n").unwrap();
    let _daemon = scene.daemon();
    let reply = scene.request(&brightness(40));
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(reply["error"]["code"], "command");
    assert!(reply["error"]["message"]
        .as_str()
        .unwrap()
        .contains("no ddc bus"));
    assert!(reply["result"].is_null());
}

#[test]
fn a_bad_monitor_name_is_refused_before_the_tool_runs() {
    let scene = Scene::new("cmd-bad");
    let _daemon = scene.daemon();
    let reply = scene.request(&json!({"op": "display.brightness", "monitor": "a b", "percent": 5}));
    assert_eq!(reply["ok"], false);
    assert_eq!(reply["error"]["code"], "bad_request");
    assert!(scene.log().is_empty());
}

#[test]
fn concurrent_writes_to_one_monitor_never_overlap_and_collapse_to_the_last() {
    let scene = Scene::new("cmd-drag");
    let _daemon = scene.daemon();
    let (run, path) = (scene.run.clone(), scene.path());
    let callers: Vec<_> = [10u32, 20, 30]
        .into_iter()
        .map(|percent| {
            let (run, path) = (run.clone(), path.clone());
            let handle = thread::spawn(move || request_in(&run, &path, &brightness(percent)));
            thread::sleep(Duration::from_millis(150));
            handle
        })
        .collect();
    let replies: Vec<Value> = callers.into_iter().map(|c| c.join().unwrap()).collect();
    assert!(
        replies.iter().all(|reply| reply["ok"] == true),
        "{replies:?}"
    );
    let log = scene.log();
    assert_eq!(
        log,
        [
            "start brightness display --no-osd --monitor DP-3 10%",
            "end brightness display --no-osd --monitor DP-3 10%",
            "start brightness display --no-osd --monitor DP-3 30%",
            "end brightness display --no-osd --monitor DP-3 30%",
        ],
        "the second write was replaced by the third, and nothing interleaved"
    );
    assert_eq!(replies[1]["result"]["coalesced"], true);
    assert_eq!(
        replies[1]["result"]["percent"], 30,
        "told what actually ran"
    );
    assert_eq!(replies[2]["result"]["coalesced"], false);
}

#[test]
fn a_snapshot_read_after_a_write_waits_for_it() {
    let scene = Scene::new("cmd-ryw");
    let _daemon = scene.daemon();
    let (run, path) = (scene.run.clone(), scene.path());
    let writer = {
        let (run, path) = (run.clone(), path.clone());
        thread::spawn(move || request_in(&run, &path, &brightness(70)))
    };
    thread::sleep(Duration::from_millis(150));
    let began = std::time::Instant::now();
    let snapshot = request_in(
        &run,
        &path,
        &json!({"op": "settings.snapshot", "group": "look"}),
    );
    let waited = began.elapsed();
    assert_eq!(snapshot["ok"], true, "{snapshot}");
    assert!(
        scene.log().iter().any(|line| line.starts_with("end ")),
        "the write had finished by the time the read answered"
    );
    assert!(waited >= Duration::from_millis(150), "{waited:?}");
    writer.join().unwrap();
}

#[test]
fn a_panel_that_reads_the_old_level_for_a_while_is_waited_for() {
    let scene = Scene::new("cmd-lag");
    let state = scene.run.join("state");
    fs::create_dir_all(&state).unwrap();
    fs::write(state.join("level"), "73").unwrap();
    // A DDC panel: a read right after a write still reports the old level, twice.
    let script = format!(
        r#"S={state}
case "$*" in
  *%) n=$(echo "$*" | sed 's/.* \([0-9]*\)%$/\1/'); echo "$n" > $S/next; echo 2 > $S/lag ;;
  *) lag=$(cat $S/lag 2>/dev/null || echo 0)
     if [ "$lag" -gt 0 ]; then echo $((lag - 1)) > $S/lag; else [ -f $S/next ] && cp $S/next $S/level; fi
     cat $S/level ;;
esac"#,
        state = state.display()
    );
    let omarchy = scene.fakes.join("omarchy");
    fs::write(&omarchy, format!("#!/bin/sh\n{script}\n")).unwrap();
    let _daemon = scene.daemon();
    let reply = scene.request(&brightness(68));
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(
        reply["result"]["snapshot"]["monitors"][0]["brightness"], 68,
        "the reply shows the level the panel settled on, not the stale read: {reply}"
    );
}

#[test]
fn scale_runs_the_tool_and_returns_the_monitor_list() {
    let scene = Scene::new("cmd-scale");
    let omarchy = scene.fakes.join("omarchy");
    fs::write(
        &omarchy,
        format!("#!/bin/sh\necho \"$*\" >> {}\n", scene.log.display()),
    )
    .unwrap();
    let _daemon = scene.daemon();
    let reply = scene.request(&json!({"op": "display.scale", "scale": "1.5"}));
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["scale"], "1.5");
    assert_eq!(reply["result"]["snapshot"]["monitors"][0]["name"], "DP-3");
    let scaled: Vec<String> = scene
        .log()
        .into_iter()
        .filter(|line| line.contains("scaling"))
        .collect();
    assert_eq!(scaled, ["hyprland monitor scaling 1.5"]);
    let bad = scene.request(&json!({"op": "display.scale", "scale": "x"}));
    assert_eq!(bad["error"]["code"], "bad_request");
}

/// A window's `ratmos serve`, in the scene's environment, talking to its daemon.
struct Window {
    child: Child,
    lines: std::sync::mpsc::Receiver<Value>,
}

impl Window {
    fn open(scene: &Scene) -> Window {
        use std::io::BufRead;
        let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .args(["--backend", "omarchy", "serve"])
            .env_clear()
            .env("XDG_RUNTIME_DIR", &scene.run)
            .env("PATH", scene.path())
            .env("HOME", &scene.run)
            .env("ATMOS_NO_AUTOSTART", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, lines) = std::sync::mpsc::channel();
        thread::spawn(move || {
            for line in std::io::BufReader::new(stdout)
                .lines()
                .map_while(Result::ok)
            {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    let _ = tx.send(value);
                }
            }
        });
        Window { child, lines }
    }

    fn send(&mut self, value: &Value) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{value}").unwrap();
    }

    fn reply(&self, id: u64) -> Value {
        loop {
            let line = self
                .lines
                .recv_timeout(Duration::from_secs(10))
                .expect("a reply");
            if line["id"] == id {
                return line;
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

/// The daemon's `status`: its sessions and the `req` frames it has answered.
fn status(run: &Path) -> Value {
    let stream = std::os::unix::net::UnixStream::connect(run.join("atmos/ratmos.sock")).unwrap();
    writeln!(&stream, r#"{{"op":"status"}}"#).unwrap();
    let mut line = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(&stream), &mut line).unwrap();
    serde_json::from_str::<Value>(&line).unwrap()
}

fn request_frames(run: &Path) -> u64 {
    status(run)["requestFrames"].as_u64().unwrap()
}

#[test]
fn a_window_sends_its_commands_and_barriers_over_its_session() {
    let scene = Scene::new("cmd-bridge");
    let _daemon = scene.daemon();
    let mut window = Window::open(&scene);
    // The session is opened when the window starts, whether or not it holds a topic.
    for _ in 0..100 {
        if status(&scene.run)["sessions"] == 1 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        status(&scene.run)["sessions"],
        1,
        "the window opened a session"
    );
    window.send(&json!({"id": 1, "op": "display.brightness", "monitor": "DP-3", "percent": 33}));
    let reply = window.reply(1);
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["percent"], 33);
    assert_eq!(
        reply["result"]["coalesced"], false,
        "it went through the daemon"
    );
    window.send(&json!({"id": 2, "op": "settings.snapshot", "group": "look"}));
    assert_eq!(window.reply(2)["ok"], true);
    // One command and one barrier, both as frames on the session.
    assert!(
        request_frames(&scene.run) >= 2,
        "{}",
        request_frames(&scene.run)
    );
}

#[test]
fn a_window_whose_daemon_dies_refuses_the_command_and_reconnects() {
    let scene = Scene::new("cmd-bridge-gone");
    let daemon = scene.daemon();
    let mut window = Window::open(&scene);
    thread::sleep(Duration::from_millis(500));
    drop(daemon);
    thread::sleep(Duration::from_millis(300));
    window.send(&json!({"id": 1, "op": "display.brightness", "monitor": "DP-3", "percent": 21}));
    let reply = window.reply(1);
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(reply["error"]["code"], "unavailable", "{reply}");
    assert!(
        !scene.log().iter().any(|line| line.ends_with("21%")),
        "nothing ran in the window"
    );
    // The daemon is back; the window finds it again by itself.
    let _daemon = scene.daemon();
    let mut id = 1;
    let reply = loop {
        id += 1;
        window
            .send(&json!({"id": id, "op": "display.brightness", "monitor": "DP-3", "percent": 22}));
        let reply = window.reply(id);
        if reply["ok"] == true || id > 30 {
            break reply;
        }
        thread::sleep(Duration::from_millis(500));
    };
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["coalesced"], false, "through the daemon");
}

#[test]
fn a_file_write_and_a_setting_go_through_the_daemons_lanes() {
    let scene = Scene::new("cmd-files");
    let _daemon = scene.daemon();
    let target = scene.run.join("note.txt");
    let wrote = scene.request(
        &json!({"op": "host.write", "path": target.display().to_string(), "text": "hello"}),
    );
    assert_eq!(wrote["ok"], true, "{wrote}");
    assert_eq!(
        wrote["result"]["coalesced"], false,
        "it went through the daemon"
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "hello");
    let set =
        scene.request(&json!({"op": "settings.set", "domain": "hyprLook.gapsIn", "value": 4}));
    assert_eq!(set["result"]["coalesced"], false, "{set}");
}

#[test]
fn applies_from_two_windows_go_one_at_a_time() {
    let scene = Scene::new("cmd-generic");
    let _daemon = scene.daemon();
    let log = scene.run.join("jobs.log");
    let job = |name: &str| {
        let script = format!(
            "echo start-{name} >> {log}; sleep 0.15; echo end-{name} >> {log}",
            log = log.display()
        );
        Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .args([
                "--backend",
                "omarchy",
                "apply",
                "--",
                "/bin/sh",
                "-c",
                &script,
            ])
            .env("XDG_RUNTIME_DIR", &scene.run)
            .env("ATMOS_NO_AUTOSTART", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    };
    let mut first = job("a");
    let mut second = job("b");
    first.wait().unwrap();
    second.wait().unwrap();
    let lines: Vec<String> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(lines.len(), 4, "{lines:?}");
    assert!(
        lines[0].starts_with("start-")
            && lines[1].starts_with("end-")
            && lines[0][6..] == lines[1][4..],
        "one finished before the other began: {lines:?}"
    );
}
