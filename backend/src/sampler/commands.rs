//! The command bus: every write, and every read of a device, is a command on a
//! lane.
//!
//! - Commands on one lane run in order; different lanes run in parallel.
//! - A command may carry a coalescing key. A command queued behind nothing but
//!   others of the same key is replaced by the newest, so a dragged slider
//!   costs one write per settle, and every caller still gets the answer.
//! - A command names the topics it affects. When it finishes the bus runs the
//!   `done` hook (which invalidates them) and only then stops holding readers
//!   back: `barrier` waits until nothing pending touches the topics, so a read
//!   after a write sees the write.
//! - A caller that runs its own process tree (sudo, a terminal, streamed
//!   output) takes a lease on the lane instead. Commands on the lane wait for
//!   it, the topics stay barred, and releasing it (or the holder vanishing)
//!   runs the same hook.

use std::collections::{HashMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use serde_json::Value;

/// The longest a lease is honoured without being renewed by its release. A
/// holder that dies without closing its connection must not bar a lane forever.
pub const LEASE_TTL: Duration = Duration::from_secs(30 * 60);

pub type Done = dyn Fn(&[String]) + Send + Sync;

/// What a command's caller gets back.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    pub value: Value,
    /// A newer command of the same key ran in its place; `value` is that one's.
    pub coalesced: bool,
}

type Job = Box<dyn FnOnce() -> Value + Send>;

enum Work {
    Run(Job),
    /// The holder waits on the receiver; the lane then waits for `release`.
    Lease {
        granted: Sender<u64>,
        ttl: Duration,
    },
}

struct Item {
    key: Option<String>,
    affects: Vec<String>,
    work: Work,
    waiters: Vec<(Sender<Reply>, bool)>,
}

#[derive(Default)]
struct Lane {
    queue: VecDeque<Item>,
    worker: bool,
    leased: Option<Held>,
}

struct Held {
    id: u64,
    affects: Vec<String>,
    until: Instant,
}

#[derive(Default)]
struct State {
    lanes: HashMap<String, Lane>,
    /// Commands and leases that touch a topic and have not finished.
    pending: HashMap<String, usize>,
    next_lease: u64,
}

pub struct Commands {
    state: Mutex<State>,
    changed: Condvar,
    done: Box<Done>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

impl Commands {
    /// `done` runs after a command or lease ends, before readers are let go.
    pub fn new(done: Box<Done>) -> Arc<Commands> {
        Arc::new(Commands {
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
            done,
        })
    }

    /// Queue `run` on `lane`. The receiver yields once, when it has run or when
    /// a newer command of the same `key` has run in its place.
    pub fn submit(
        self: &Arc<Self>,
        lane: &str,
        key: Option<&str>,
        affects: &[&str],
        run: impl FnOnce() -> Value + Send + 'static,
    ) -> Receiver<Reply> {
        let (tx, rx) = channel();
        let affects: Vec<String> = affects.iter().map(|topic| topic.to_string()).collect();
        let mut state = lock(&self.state);
        let entry = state.lanes.entry(lane.to_string()).or_default();
        // Only the tail may be replaced, so a command never jumps one queued
        // before it.
        if let (Some(key), Some(tail)) = (key, entry.queue.back_mut()) {
            if tail.key.as_deref() == Some(key) && matches!(tail.work, Work::Run(_)) {
                tail.work = Work::Run(Box::new(run));
                for waiter in &mut tail.waiters {
                    waiter.1 = true;
                }
                tail.waiters.push((tx, false));
                return rx;
            }
        }
        entry.queue.push_back(Item {
            key: key.map(str::to_string),
            affects: affects.clone(),
            work: Work::Run(Box::new(run)),
            waiters: vec![(tx, false)],
        });
        for topic in &affects {
            *state.pending.entry(topic.clone()).or_default() += 1;
        }
        self.start_worker(&mut state, lane);
        rx
    }

    /// Take `lane` for a job the caller runs itself. Blocks until the lane
    /// reaches this lease, then returns its id; None if `wait` runs out first
    /// (the lease is then withdrawn, and the topics are not barred).
    pub fn lease(self: &Arc<Self>, lane: &str, affects: &[&str], wait: Duration) -> Option<u64> {
        let (granted, rx) = channel();
        let affects: Vec<String> = affects.iter().map(|topic| topic.to_string()).collect();
        {
            let mut state = lock(&self.state);
            state
                .lanes
                .entry(lane.to_string())
                .or_default()
                .queue
                .push_back(Item {
                    key: None,
                    affects: affects.clone(),
                    work: Work::Lease {
                        granted,
                        ttl: LEASE_TTL,
                    },
                    waiters: Vec::new(),
                });
            for topic in &affects {
                *state.pending.entry(topic.clone()).or_default() += 1;
            }
            self.start_worker(&mut state, lane);
        }
        match rx.recv_timeout(wait) {
            Ok(id) => Some(id),
            Err(_) => {
                self.withdraw(lane, &affects);
                None
            }
        }
    }

    /// A lease that gave up waiting: drop it from the queue, or release it if
    /// it was granted in the meantime.
    fn withdraw(self: &Arc<Self>, lane: &str, affects: &[String]) {
        let mut state = lock(&self.state);
        let mut found = false;
        if let Some(entry) = state.lanes.get_mut(lane) {
            if let Some(at) = entry
                .queue
                .iter()
                .position(|item| matches!(item.work, Work::Lease { .. }) && item.affects == affects)
            {
                entry.queue.remove(at);
                found = true;
            }
        }
        if found {
            self.settle(&mut state, affects);
        } else if let Some(id) = state
            .lanes
            .get(lane)
            .and_then(|entry| entry.leased.as_ref())
            .filter(|held| held.affects == affects)
            .map(|held| held.id)
        {
            drop(state);
            self.release(id);
        }
    }

    /// End a lease. Unknown or already ended ids are ignored.
    pub fn release(self: &Arc<Self>, id: u64) {
        let mut state = lock(&self.state);
        let affects = state
            .lanes
            .values_mut()
            .find(|lane| lane.leased.as_ref().is_some_and(|held| held.id == id))
            .and_then(|lane| lane.leased.take())
            .map(|held| held.affects);
        let Some(affects) = affects else { return };
        drop(state);
        (self.done)(&affects);
        let mut state = lock(&self.state);
        self.settle(&mut state, &affects);
        self.changed.notify_all();
    }

    /// Run the invalidation hook for `topics` now. A job that is not on the
    /// barrier (it may run for minutes) says what it changed this way when it
    /// ends.
    pub fn invalidate(&self, topics: &[String]) {
        (self.done)(topics);
    }

    /// Wait until nothing pending touches any of `topics`. False on timeout.
    pub fn barrier(&self, topics: &[String], timeout: Duration) -> bool {
        let state = lock(&self.state);
        let clear = |state: &State| {
            topics
                .iter()
                .all(|topic| state.pending.get(topic).copied().unwrap_or(0) == 0)
        };
        let (state, _) = self
            .changed
            .wait_timeout_while(state, timeout, |state| !clear(state))
            .unwrap_or_else(|err| err.into_inner());
        clear(&state)
    }

    /// Commands and leases not yet finished.
    #[cfg(test)]
    pub fn pending(&self) -> usize {
        lock(&self.state).pending.values().sum()
    }

    fn settle(&self, state: &mut State, affects: &[String]) {
        for topic in affects {
            if let Some(count) = state.pending.get_mut(topic) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    state.pending.remove(topic);
                }
            }
        }
        self.changed.notify_all();
    }

    fn start_worker(self: &Arc<Self>, state: &mut State, lane: &str) {
        let entry = state.lanes.entry(lane.to_string()).or_default();
        if entry.worker {
            return;
        }
        entry.worker = true;
        let (bus, lane) = (Arc::clone(self), lane.to_string());
        std::thread::spawn(move || bus.work(&lane));
    }

    /// One thread per busy lane: take items in order until the lane is idle.
    fn work(self: Arc<Self>, lane: &str) {
        loop {
            let item = {
                let mut state = lock(&self.state);
                let Some(entry) = state.lanes.get_mut(lane) else {
                    return;
                };
                match entry.queue.pop_front() {
                    Some(item) => item,
                    None => {
                        entry.worker = false;
                        if entry.leased.is_none() {
                            state.lanes.remove(lane);
                        }
                        return;
                    }
                }
            };
            match item.work {
                Work::Run(job) => {
                    let value = catch_unwind(AssertUnwindSafe(job)).unwrap_or_else(
                        |_| serde_json::json!({"ok": false, "message": "the command panicked"}),
                    );
                    (self.done)(&item.affects);
                    for (waiter, coalesced) in item.waiters {
                        let _ = waiter.send(Reply {
                            value: value.clone(),
                            coalesced,
                        });
                    }
                    let mut state = lock(&self.state);
                    self.settle(&mut state, &item.affects);
                }
                Work::Lease { granted, ttl } => {
                    let id = {
                        let mut state = lock(&self.state);
                        state.next_lease += 1;
                        let id = state.next_lease;
                        if let Some(entry) = state.lanes.get_mut(lane) {
                            entry.leased = Some(Held {
                                id,
                                affects: item.affects.clone(),
                                until: Instant::now() + ttl,
                            });
                        }
                        id
                    };
                    if granted.send(id).is_err() {
                        // The caller gave up; end it as if released.
                        self.release(id);
                        continue;
                    }
                    self.hold(lane, id);
                }
            }
        }
    }

    /// The lane stands still while a lease is held, until it is released or
    /// its time runs out.
    fn hold(self: &Arc<Self>, lane: &str, id: u64) {
        let mut state = lock(&self.state);
        loop {
            let until = match state
                .lanes
                .get(lane)
                .and_then(|entry| entry.leased.as_ref())
            {
                Some(held) if held.id == id => held.until,
                _ => return,
            };
            let now = Instant::now();
            if now >= until {
                drop(state);
                self.release(id);
                return;
            }
            let (next, _) = self
                .changed
                .wait_timeout(state, until - now)
                .unwrap_or_else(|err| err.into_inner());
            state = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    type Seen = Arc<Mutex<Vec<Vec<String>>>>;

    fn bus() -> (Arc<Commands>, Seen) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = seen.clone();
        let bus = Commands::new(Box::new(move |affects| {
            lock(&record).push(affects.to_vec());
        }));
        (bus, seen)
    }

    fn wait(rx: &Receiver<Reply>) -> Reply {
        rx.recv_timeout(Duration::from_secs(5)).expect("a reply")
    }

    #[test]
    fn a_lane_runs_in_order() {
        let (bus, _) = bus();
        let log = Arc::new(Mutex::new(Vec::new()));
        let replies: Vec<_> = (0..5)
            .map(|n| {
                let log = log.clone();
                bus.submit("monitor:DP-3", None, &[], move || {
                    std::thread::sleep(Duration::from_millis(10));
                    lock(&log).push(n);
                    json!(n)
                })
            })
            .collect();
        for (n, rx) in replies.iter().enumerate() {
            assert_eq!(wait(rx).value, json!(n));
        }
        assert_eq!(*lock(&log), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn different_lanes_run_side_by_side() {
        let (bus, _) = bus();
        let began = Instant::now();
        let replies: Vec<_> = (0..4)
            .map(|n| {
                bus.submit(&format!("lane{n}"), None, &[], || {
                    std::thread::sleep(Duration::from_millis(300));
                    json!(1)
                })
            })
            .collect();
        for rx in &replies {
            wait(rx);
        }
        assert!(
            began.elapsed() < Duration::from_millis(900),
            "{:?}",
            began.elapsed()
        );
    }

    #[test]
    fn a_dragged_slider_costs_one_write_after_the_one_in_flight() {
        let (bus, _) = bus();
        let runs = Arc::new(AtomicUsize::new(0));
        let last = Arc::new(Mutex::new(0));
        let go = Arc::new(Mutex::new(()));
        let held = go.lock().unwrap();
        let first = {
            let (runs, go, last) = (runs.clone(), go.clone(), last.clone());
            bus.submit("monitor:DP-3", Some("brightness"), &[], move || {
                let _wait = lock(&go);
                runs.fetch_add(1, Ordering::SeqCst);
                *lock(&last) = 0;
                json!(0)
            })
        };
        // The first is running (blocked); the next four queue and collapse.
        std::thread::sleep(Duration::from_millis(100));
        let rest: Vec<_> = (1..=4)
            .map(|n| {
                let (runs, last) = (runs.clone(), last.clone());
                bus.submit("monitor:DP-3", Some("brightness"), &[], move || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    *lock(&last) = n;
                    json!(n)
                })
            })
            .collect();
        drop(held);
        assert_eq!(wait(&first).value, json!(0));
        let replies: Vec<Reply> = rest.iter().map(wait).collect();
        assert_eq!(
            runs.load(Ordering::SeqCst),
            2,
            "the running one and the newest"
        );
        assert!(replies.iter().all(|reply| reply.value == json!(4)));
        assert_eq!(
            replies.iter().filter(|reply| reply.coalesced).count(),
            3,
            "the three replaced callers are told so"
        );
        assert!(!replies[3].coalesced);
        assert_eq!(*lock(&last), 4);
    }

    #[test]
    fn a_command_never_jumps_one_queued_before_it() {
        let (bus, _) = bus();
        let order = Arc::new(Mutex::new(Vec::new()));
        let go = Arc::new(Mutex::new(()));
        let held = go.lock().unwrap();
        let mut replies = Vec::new();
        for (name, key) in [
            ("block", None),
            ("a1", Some("k")),
            ("other", None),
            ("a2", Some("k")),
        ] {
            let (order, go) = (order.clone(), go.clone());
            replies.push(bus.submit("lane", key, &[], move || {
                if name == "block" {
                    let _wait = lock(&go);
                }
                lock(&order).push(name);
                json!(name)
            }));
            std::thread::sleep(Duration::from_millis(30));
        }
        drop(held);
        for rx in &replies {
            wait(rx);
        }
        assert_eq!(*lock(&order), ["block", "a1", "other", "a2"]);
    }

    #[test]
    fn a_read_after_a_write_waits_for_it_and_sees_the_invalidation() {
        let (bus, seen) = bus();
        let rx = bus.submit("monitor:DP-3", None, &["monitors"], || {
            std::thread::sleep(Duration::from_millis(300));
            json!({"ok": true})
        });
        assert!(!bus.barrier(&["monitors".into()], Duration::from_millis(50)));
        assert!(bus.barrier(&["monitors".into()], Duration::from_secs(5)));
        assert_eq!(*lock(&seen), [vec!["monitors".to_string()]]);
        assert_eq!(wait(&rx).value, json!({"ok": true}));
        assert_eq!(bus.pending(), 0);
    }

    #[test]
    fn a_barrier_on_an_unrelated_topic_does_not_wait() {
        let (bus, _) = bus();
        let rx = bus.submit("lane", None, &["monitors"], || {
            std::thread::sleep(Duration::from_millis(300));
            json!(1)
        });
        let began = Instant::now();
        assert!(bus.barrier(&["services".into()], Duration::from_secs(5)));
        assert!(began.elapsed() < Duration::from_millis(100));
        wait(&rx);
    }

    #[test]
    fn a_panicking_command_is_an_error_reply_and_the_lane_survives() {
        let (bus, _) = bus();
        let bad = bus.submit("lane", None, &["t"], || panic!("boom"));
        assert_eq!(wait(&bad).value["ok"], false);
        let good = bus.submit("lane", None, &["t"], || json!("fine"));
        assert_eq!(wait(&good).value, json!("fine"));
        assert!(bus.barrier(&["t".into()], Duration::from_secs(1)));
    }

    #[test]
    fn a_lease_holds_the_lane_and_the_topics_until_released() {
        let (bus, seen) = bus();
        let id = bus
            .lease("unit:sshd", &["services"], Duration::from_secs(1))
            .expect("the idle lane grants at once");
        let ran = Arc::new(AtomicUsize::new(0));
        let counted = ran.clone();
        let rx = bus.submit("unit:sshd", None, &[], move || {
            counted.fetch_add(1, Ordering::SeqCst);
            json!(1)
        });
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(ran.load(Ordering::SeqCst), 0, "a leased lane stands still");
        assert!(!bus.barrier(&["services".into()], Duration::from_millis(50)));
        bus.release(id);
        assert!(bus.barrier(&["services".into()], Duration::from_secs(5)));
        assert_eq!(wait(&rx).value, json!(1));
        assert_eq!(lock(&seen)[0], ["services"]);
    }

    #[test]
    fn a_second_lease_waits_its_turn_and_gives_up_cleanly() {
        let (bus, _) = bus();
        let first = bus
            .lease("lane", &["a"], Duration::from_secs(1))
            .expect("first");
        assert!(bus
            .lease("lane", &["b"], Duration::from_millis(100))
            .is_none());
        assert!(
            bus.barrier(&["b".into()], Duration::from_millis(100)),
            "a withdrawn lease bars nothing"
        );
        bus.release(first);
        let again = bus.lease("lane", &["c"], Duration::from_secs(1));
        assert!(again.is_some());
        bus.release(again.unwrap());
        assert_eq!(bus.pending(), 0);
    }

    #[test]
    fn releasing_twice_or_an_unknown_lease_is_harmless() {
        let (bus, seen) = bus();
        let id = bus
            .lease("lane", &["a"], Duration::from_secs(1))
            .expect("lease");
        bus.release(id);
        bus.release(id);
        bus.release(9999);
        assert_eq!(lock(&seen).len(), 1, "the hook runs once");
    }

    #[test]
    fn an_idle_lane_leaves_no_state_behind() {
        let (bus, _) = bus();
        let rx = bus.submit("lane", None, &["a"], || json!(1));
        wait(&rx);
        for _ in 0..50 {
            if lock(&bus.state).lanes.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(lock(&bus.state).lanes.is_empty());
        assert_eq!(bus.pending(), 0);
    }
}
