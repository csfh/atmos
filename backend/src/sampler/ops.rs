//! The daemon's command-bus ops: `command`, `barrier` and `lease`.
//!
//! They arrive on the one-shot socket like `sample`. `command` and `barrier`
//! answer when they are done; `lease` answers once the lane is the caller's
//! and then holds the connection, so a holder that dies releases the lane by
//! closing its end.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use super::daemon::Daemon;
use crate::command;
use crate::request::Request;

/// The longest a command may take, queue time included, before its caller is
/// told it did not finish (the command itself is still ordered and runs).
const COMMAND_WAIT: Duration = Duration::from_secs(150);

/// The longest a read may be held back, whatever the caller asked for.
const BARRIER_CAP: Duration = Duration::from_secs(10);

/// How many reads run a child at once. A page that asks once per item (a grid
/// of themes) would otherwise start a process for every one of them together.
const READ_SLOTS: usize = 4;

/// A counting semaphore for the children of reads.
#[derive(Default)]
pub(super) struct Slots {
    taken: std::sync::Mutex<usize>,
    freed: std::sync::Condvar,
}

pub(super) struct Slot<'a>(&'a Slots);

impl Slots {
    fn acquire(&self) -> Slot<'_> {
        let mut taken = self.taken.lock().unwrap_or_else(|err| err.into_inner());
        while *taken >= READ_SLOTS {
            taken = self
                .freed
                .wait(taken)
                .unwrap_or_else(|err| err.into_inner());
        }
        *taken += 1;
        Slot(self)
    }
}

impl Drop for Slot<'_> {
    fn drop(&mut self) {
        let mut taken = self.0.taken.lock().unwrap_or_else(|err| err.into_inner());
        *taken -= 1;
        self.0.freed.notify_one();
    }
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

impl Daemon {
    /// `{"op":"command","request":{...},"backend":"omarchy","env":{...}}`
    pub(super) fn command_op(self: &Arc<Self>, request: &Value) -> Value {
        let body = request.get("request").cloned().unwrap_or(Value::Null);
        let parsed = match Request::parse(body.clone()) {
            Ok(parsed) => parsed,
            Err(err) => return json!({"ok": false, "message": format!("bad command: {err}")}),
        };
        // A write has the lane its entry names; anything else is a read, which
        // has a lane of its own so reads never wait for each other.
        let command = command::plan(&parsed);
        let is_command = command.is_some();
        // A speed test holds its child for seconds on purpose; it must not
        // take a slot from the quick reads.
        let slotted =
            !is_command && !matches!(parsed, Request::SpeedtestDisk(_) | Request::SpeedtestNet(_));
        let plan = command.unwrap_or_else(command::read_plan);
        self.commands_run
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let backend = request
            .get("backend")
            .and_then(Value::as_str)
            .unwrap_or("omarchy")
            .to_string();
        let env: Vec<(String, String)> = super::client::or_own_env(
            request
                .get("env")
                .and_then(Value::as_object)
                .map(|map| {
                    map.iter()
                        .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
        );
        let affects: Vec<&str> = plan.affects.iter().map(String::as_str).collect();
        let run = Arc::clone(&self.run_command);
        let slots = Arc::clone(&self.read_slots);
        let reply = self
            .commands
            .submit(&plan.lane, plan.key.as_deref(), &affects, move || {
                // A write has its lane to keep it in order; a read waits its turn.
                let _slot = slotted.then(|| slots.acquire());
                run(&backend, &body, &env)
            });
        match reply.recv_timeout(COMMAND_WAIT) {
            Ok(reply) => {
                // What the write read back is what the topics now show.
                if is_command && reply.value["ok"].as_bool() == Some(true) {
                    self.apply_readback(&reply.value["result"]);
                }
                json!({"ok": true, "envelope": reply.value, "coalesced": reply.coalesced})
            }
            Err(_) => json!({"ok": false, "message": "the command did not finish in time"}),
        }
    }

    /// `{"op":"barrier","topics":[...],"waitMs":3000}`: answers when nothing
    /// pending touches the topics, or when the wait runs out.
    pub(super) fn barrier_op(&self, request: &Value) -> Value {
        let topics = strings(request.get("topics"));
        let wait = Duration::from_millis(
            request
                .get("waitMs")
                .and_then(Value::as_u64)
                .unwrap_or(3000),
        )
        .min(BARRIER_CAP);
        json!({"ok": true, "clear": self.commands.barrier(&topics, wait)})
    }
}

/// `{"op":"lease","lane":"job:123","affects":["services"],"waitMs":2000}`.
/// The reply is `{"ok":true,"lease":N}` once the lane is the caller's; then
/// the connection stays open, and closing it releases the lane and
/// invalidates what it affects.
pub(super) fn hold_lease(daemon: &Arc<Daemon>, mut stream: UnixStream, request: &Value) {
    let lane = request.get("lane").and_then(Value::as_str).unwrap_or("");
    let affects = strings(request.get("affects"));
    let wait = Duration::from_millis(
        request
            .get("waitMs")
            .and_then(Value::as_u64)
            .unwrap_or(2000),
    )
    .min(BARRIER_CAP);
    if lane.is_empty() {
        let _ = writeln!(
            stream,
            "{}",
            json!({"ok": false, "message": "lane is required"})
        );
        return;
    }
    let named: Vec<&str> = affects.iter().map(String::as_str).collect();
    let Some(id) = daemon.commands.lease(lane, &named, wait) else {
        let _ = writeln!(
            stream,
            "{}",
            json!({"ok": false, "message": "the lane is busy"})
        );
        return;
    };
    if writeln!(stream, "{}", json!({"ok": true, "lease": id})).is_ok() {
        let _ = stream.set_read_timeout(None);
        let mut sink = [0u8; 64];
        // The holder says nothing; the end of the stream is the release.
        while matches!(stream.read(&mut sink), Ok(n) if n > 0) {}
    }
    daemon.commands.release(id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::cache;
    use crate::sampler::client::{self, Command};
    use crate::sampler::daemon::tests::serve_commands;
    use std::sync::Mutex;
    use std::time::Instant;

    fn brightness(percent: u32) -> Value {
        json!({"op": "display.brightness", "monitor": "DP-3", "percent": percent})
    }

    /// A runner that records each percent it is asked to apply, slowly.
    fn slow_recorder(
        delay: Duration,
    ) -> (
        Box<crate::sampler::daemon::RunCommand>,
        Arc<Mutex<Vec<u64>>>,
    ) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let record = log.clone();
        let run = Box::new(move |_: &str, body: &Value, _: &[(String, String)]| {
            std::thread::sleep(delay);
            let percent = body["percent"].as_u64().unwrap_or(0);
            record.lock().unwrap().push(percent);
            json!({"ok": true, "result": {"percent": percent}})
        });
        (run, log)
    }

    #[test]
    fn a_command_comes_back_with_the_answer_of_the_command_that_ran() {
        let (run, log) = slow_recorder(Duration::ZERO);
        let (dir, _daemon) = serve_commands("cmd-basic", run);
        match client::command_in(&dir, &brightness(40), "omarchy") {
            Command::Done {
                envelope,
                coalesced,
            } => {
                assert_eq!(envelope["result"]["percent"], 40);
                assert!(!coalesced);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(*log.lock().unwrap(), [40]);
    }

    #[test]
    fn the_callers_environment_reaches_the_runner() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = seen.clone();
        let (dir, _daemon) = serve_commands(
            "cmd-env",
            Box::new(move |backend, _, env| {
                record
                    .lock()
                    .unwrap()
                    .push((backend.to_string(), env.len()));
                json!({"ok": true, "result": {}})
            }),
        );
        client::command_in(&dir, &brightness(1), "plain");
        let seen = seen.lock().unwrap();
        assert_eq!(seen[0].0, "plain");
        assert!(seen[0].1 > 0, "the environment came along");
    }

    #[test]
    fn a_dragged_slider_across_windows_runs_the_first_and_the_last() {
        let (run, log) = slow_recorder(Duration::from_millis(300));
        let (dir, _daemon) = serve_commands("cmd-drag", run);
        let callers: Vec<_> = (1..=6u32)
            .map(|n| {
                let dir = dir.clone();
                let handle = std::thread::spawn(move || {
                    client::command_in(&dir, &brightness(n * 10), "omarchy")
                });
                std::thread::sleep(Duration::from_millis(30));
                handle
            })
            .collect();
        let results: Vec<Command> = callers.into_iter().map(|c| c.join().unwrap()).collect();
        let ran = log.lock().unwrap().clone();
        // How many ran depends on how fast the callers arrived; the shape does not.
        assert_eq!(ran.first(), Some(&10), "the one in flight ran first");
        assert_eq!(ran.last(), Some(&60), "the newest ran last");
        assert!(ran.len() < 6, "some were collapsed: {ran:?}");
        assert!(
            ran.windows(2).all(|pair| pair[0] < pair[1]),
            "in order: {ran:?}"
        );
        let mut coalesced = 0;
        for result in &results {
            match result {
                Command::Done {
                    envelope,
                    coalesced: c,
                } => {
                    coalesced += usize::from(*c);
                    let percent = envelope["result"]["percent"].as_u64().unwrap();
                    assert!(ran.contains(&percent));
                }
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(coalesced, 6 - ran.len(), "every replaced caller is told so");
    }

    #[test]
    fn two_monitors_do_not_wait_for_each_other() {
        let (run, _log) = slow_recorder(Duration::from_millis(300));
        let (dir, _daemon) = serve_commands("cmd-lanes", run);
        let began = Instant::now();
        let callers: Vec<_> = ["DP-3", "eDP-1"]
            .into_iter()
            .map(|monitor| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    let body =
                        json!({"op": "display.brightness", "monitor": monitor, "percent": 50});
                    client::command_in(&dir, &body, "omarchy")
                })
            })
            .collect();
        for caller in callers {
            caller.join().unwrap();
        }
        assert!(
            began.elapsed() < Duration::from_millis(550),
            "{:?}",
            began.elapsed()
        );
    }

    #[test]
    fn a_read_after_a_write_waits_for_it() {
        let (run, _log) = slow_recorder(Duration::from_millis(400));
        let (dir, _daemon) = serve_commands("cmd-barrier", run);
        let writer = {
            let dir = dir.clone();
            std::thread::spawn(move || client::command_in(&dir, &brightness(30), "omarchy"))
        };
        std::thread::sleep(Duration::from_millis(100));
        let began = Instant::now();
        client::barrier_in(&dir, &["monitors".to_string()], Duration::from_secs(3));
        assert!(
            began.elapsed() >= Duration::from_millis(200),
            "{:?}",
            began.elapsed()
        );
        assert!(began.elapsed() < Duration::from_secs(2));
        writer.join().unwrap();
        let again = Instant::now();
        client::barrier_in(&dir, &["monitors".to_string()], Duration::from_secs(3));
        assert!(
            again.elapsed() < Duration::from_millis(100),
            "nothing pending"
        );
    }

    #[test]
    fn a_barrier_gives_up_after_its_wait() {
        let (run, _log) = slow_recorder(Duration::from_secs(2));
        let (dir, _daemon) = serve_commands("cmd-giveup", run);
        let dir2 = dir.clone();
        std::thread::spawn(move || client::command_in(&dir2, &brightness(30), "omarchy"));
        std::thread::sleep(Duration::from_millis(100));
        let began = Instant::now();
        client::barrier_in(&dir, &["monitors".to_string()], Duration::from_millis(300));
        assert!(
            began.elapsed() < Duration::from_millis(900),
            "{:?}",
            began.elapsed()
        );
    }

    #[test]
    fn a_read_is_run_by_the_daemon_on_a_lane_of_its_own_and_a_nonsense_request_is_refused() {
        let (run, log) = slow_recorder(Duration::from_millis(300));
        let (dir, daemon) = serve_commands("cmd-reads", run);
        // Two reads at once do not wait for each other.
        let began = Instant::now();
        let reads: Vec<_> = (0..2)
            .map(|_| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    client::command_in(&dir, &json!({"op": "version"}), "omarchy")
                })
            })
            .collect();
        for read in reads {
            assert!(matches!(read.join().unwrap(), Command::Done { .. }));
        }
        assert!(
            began.elapsed() < Duration::from_millis(550),
            "{:?}",
            began.elapsed()
        );
        assert_eq!(log.lock().unwrap().len(), 2);
        assert_eq!(daemon.status()["commandsRun"], 2);
        // Refused before anything was queued.
        assert!(matches!(
            client::command_in(&dir, &json!({"op": "nope"}), "omarchy"),
            Command::Unavailable
        ));
        assert_eq!(log.lock().unwrap().len(), 2);
    }

    #[test]
    fn a_burst_of_reads_never_runs_more_children_than_there_are_slots() {
        let (now, most) = (
            Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        );
        let (a, b) = (now.clone(), most.clone());
        let (dir, _daemon) = serve_commands(
            "cmd-burst",
            Box::new(move |_, _, _| {
                let at_once = a.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                b.fetch_max(at_once, std::sync::atomic::Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(20));
                a.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                json!({"ok": true, "result": {}})
            }),
        );
        let askers: Vec<_> = (0..24)
            .map(|_| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    client::command_in(&dir, &json!({"op": "version"}), "omarchy")
                })
            })
            .collect();
        for asker in askers {
            assert!(matches!(asker.join().unwrap(), Command::Done { .. }));
        }
        let most = most.load(std::sync::atomic::Ordering::SeqCst);
        assert!((2..=READ_SLOTS).contains(&most), "{most} children at once");
    }

    #[test]
    fn with_no_daemon_a_command_is_unavailable_and_nothing_waits() {
        let dir = cache::tests::scratch("cmd-nodaemon");
        assert!(matches!(
            client::command_in(&dir, &brightness(1), "omarchy"),
            Command::Unavailable
        ));
        let began = Instant::now();
        client::barrier_in(&dir, &["monitors".into()], Duration::from_secs(3));
        assert!(began.elapsed() < Duration::from_millis(200));
        assert!(client::lease_in(&dir, "job:1", &["services"], Duration::from_secs(1)).is_none());
    }

    #[test]
    fn a_leased_job_bars_its_kinds_and_drops_them_when_it_ends() {
        let (run, _log) = slow_recorder(Duration::ZERO);
        let (dir, _daemon) = serve_commands("cmd-lease", run);
        cache::write(
            &dir,
            "services",
            &cache::Sample {
                at_ms: cache::now_ms(),
                took_ms: 1,
                value: json!({"a": 1}),
            },
        )
        .unwrap();
        let lease = client::lease_in(&dir, "job:42", &["services"], Duration::from_secs(1))
            .expect("an idle lane");
        let began = Instant::now();
        client::barrier_in(&dir, &["services".to_string()], Duration::from_millis(300));
        assert!(
            began.elapsed() >= Duration::from_millis(250),
            "{:?}",
            began.elapsed()
        );
        std::thread::sleep(Duration::from_millis(5));
        drop(lease);
        let released = Instant::now();
        client::barrier_in(&dir, &["services".to_string()], Duration::from_secs(3));
        assert!(
            released.elapsed() < Duration::from_secs(1),
            "{:?}",
            released.elapsed()
        );
        assert!(
            cache::is_dirty(&dir, "services"),
            "the job's kinds were dropped"
        );
    }

    #[test]
    fn a_holder_that_dies_releases_its_lane() {
        let (run, _log) = slow_recorder(Duration::ZERO);
        let (dir, _daemon) = serve_commands("cmd-die", run);
        let first = client::lease_in(&dir, "lane", &["a"], Duration::from_secs(1)).unwrap();
        assert!(client::lease_in(&dir, "lane", &["b"], Duration::from_millis(200)).is_none());
        drop(first);
        assert!(client::lease_in(&dir, "lane", &["c"], Duration::from_secs(2)).is_some());
    }
}
