//! The sampling bus: who wants which topic pushed, and how fast.
//!
//! A session (see `session`) attaches one `Subscriber` with a bounded outbox.
//! The daemon publishes every new sample of a kind; each subscriber takes it
//! only once its own interval has passed, measured by `sampledAt`, so a window
//! that asked for two seconds is never fed one. A subscriber whose outbox is
//! full or closed is dropped, so a stalled window cannot hold up publishing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{json, Value};

use super::cache::Sample;

/// Frames a window may fall behind before it is dropped.
const OUTBOX: usize = 64;

/// A subscriber takes a sample that arrives this close to its interval, so
/// jitter in the pump does not cost it a whole extra period.
const SLACK: f64 = 0.75;

/// Kinds the daemon samples itself, on the subscribers' clock, while any
/// subscriber holds them. Everything else is pushed when it changes.
pub fn is_pumped(kind: &str) -> bool {
    kind == "live"
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

#[derive(Clone, Copy, Debug)]
struct Want {
    interval_ms: u64,
    last_at: Option<u64>,
}

pub struct Subscriber {
    id: u64,
    outbox: SyncSender<Value>,
    wants: Mutex<HashMap<String, Want>>,
}

impl Subscriber {
    /// Queue one frame. False means the outbox is full or closed.
    pub fn send(&self, frame: Value) -> bool {
        match self.outbox.try_send(frame) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => false,
        }
    }

    /// Replace what this subscriber wants. A kept kind keeps its throttle.
    pub fn set(&self, topics: HashMap<String, u64>) {
        let mut wants = lock(&self.wants);
        let mut next = HashMap::new();
        for (kind, interval_ms) in topics {
            let last_at = wants.get(&kind).and_then(|want| want.last_at);
            next.insert(
                kind,
                Want {
                    interval_ms,
                    last_at,
                },
            );
        }
        *wants = next;
    }

    /// Send a sample the daemon already holds, counting it for the throttle.
    pub fn admit_and_send(&self, kind: &str, sample: &Sample) {
        if self.admit(kind, sample.at_ms) {
            self.send(event(kind, sample));
        }
    }

    fn interval_of(&self, kind: &str) -> Option<u64> {
        lock(&self.wants).get(kind).map(|want| want.interval_ms)
    }

    /// Take `sample` if this subscriber wants the kind and is due for one.
    fn admit(&self, kind: &str, at_ms: u64) -> bool {
        let mut wants = lock(&self.wants);
        let Some(want) = wants.get_mut(kind) else {
            return false;
        };
        if let Some(last) = want.last_at {
            if at_ms <= last {
                return false;
            }
            if ((at_ms - last) as f64) < want.interval_ms as f64 * SLACK {
                return false;
            }
        }
        want.last_at = Some(at_ms);
        true
    }
}

#[derive(Default)]
pub struct Bus {
    subscribers: Mutex<Vec<Arc<Subscriber>>>,
    next: AtomicU64,
    changed: Condvar,
    generation: Mutex<u64>,
}

pub fn event(kind: &str, sample: &Sample) -> Value {
    json!({
        "t": "event",
        "topic": kind,
        "sampledAt": sample.at_ms,
        "tookMs": sample.took_ms,
        "value": sample.value,
    })
}

impl Bus {
    pub fn attach(&self) -> (Arc<Subscriber>, Receiver<Value>) {
        let (outbox, inbox) = sync_channel(OUTBOX);
        let subscriber = Arc::new(Subscriber {
            id: self.next.fetch_add(1, Ordering::SeqCst),
            outbox,
            wants: Mutex::new(HashMap::new()),
        });
        lock(&self.subscribers).push(subscriber.clone());
        self.poke();
        (subscriber, inbox)
    }

    /// Sessions attached now.
    pub fn count(&self) -> usize {
        lock(&self.subscribers).len()
    }

    pub fn detach(&self, subscriber: &Subscriber) {
        lock(&self.subscribers).retain(|other| other.id != subscriber.id);
        self.poke();
    }

    /// Wake whoever waits in `wait`: the wants changed.
    pub fn poke(&self) {
        *lock(&self.generation) += 1;
        self.changed.notify_all();
    }

    /// Sleep until the wants change or `timeout` passes.
    pub fn wait(&self, timeout: Duration) {
        let guard = lock(&self.generation);
        let seen = *guard;
        let _ = self
            .changed
            .wait_timeout_while(guard, timeout, |now| *now == seen);
    }

    /// The shortest interval anyone wants `kind` at.
    pub fn interval(&self, kind: &str) -> Option<Duration> {
        lock(&self.subscribers)
            .iter()
            .filter_map(|subscriber| subscriber.interval_of(kind))
            .min()
            .map(Duration::from_millis)
    }

    pub fn publish(&self, kind: &str, sample: &Sample) {
        let frame = event(kind, sample);
        self.deliver(|subscriber| subscriber.admit(kind, sample.at_ms).then(|| frame.clone()));
    }

    /// A failed collection: whoever holds the kind is told, with the last good
    /// value left where it is.
    pub fn fail(&self, kind: &str, message: &str) {
        let frame = json!({"t": "event", "topic": kind, "stale": true, "error": message});
        self.deliver(|subscriber| subscriber.interval_of(kind).map(|_| frame.clone()));
    }

    fn deliver(&self, frame_for: impl Fn(&Subscriber) -> Option<Value>) {
        let mut dropped = Vec::new();
        for subscriber in lock(&self.subscribers).iter() {
            if let Some(frame) = frame_for(subscriber) {
                if !subscriber.send(frame) {
                    dropped.push(subscriber.id);
                }
            }
        }
        if !dropped.is_empty() {
            lock(&self.subscribers).retain(|subscriber| !dropped.contains(&subscriber.id));
            self.poke();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(at_ms: u64) -> Sample {
        Sample {
            at_ms,
            took_ms: 3,
            value: json!({"at": at_ms}),
        }
    }

    fn wants(pairs: &[(&str, u64)]) -> HashMap<String, u64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn a_subscriber_gets_only_what_it_asked_for() {
        let bus = Bus::default();
        let (subscriber, inbox) = bus.attach();
        subscriber.set(wants(&[("live", 1000)]));
        bus.publish("services", &sample(1));
        bus.publish("live", &sample(1));
        let frame = inbox.try_recv().unwrap();
        assert_eq!(frame["topic"], "live");
        assert_eq!(frame["sampledAt"], 1);
        assert!(inbox.try_recv().is_err());
    }

    #[test]
    fn each_subscriber_is_throttled_to_its_own_interval() {
        let bus = Bus::default();
        let (fast, fast_inbox) = bus.attach();
        let (slow, slow_inbox) = bus.attach();
        fast.set(wants(&[("live", 1000)]));
        slow.set(wants(&[("live", 2000)]));
        for second in 0..6u64 {
            bus.publish("live", &sample(10_000 + second * 1000 + second % 2 * 10));
        }
        assert_eq!(fast_inbox.try_iter().count(), 6);
        assert_eq!(slow_inbox.try_iter().count(), 3);
        assert_eq!(bus.interval("live"), Some(Duration::from_millis(1000)));
    }

    #[test]
    fn the_same_sample_is_never_delivered_twice() {
        let bus = Bus::default();
        let (subscriber, inbox) = bus.attach();
        subscriber.set(wants(&[("live", 0)]));
        bus.publish("live", &sample(5));
        bus.publish("live", &sample(5));
        bus.publish("live", &sample(4));
        assert_eq!(inbox.try_iter().count(), 1);
    }

    #[test]
    fn changing_wants_keeps_the_throttle_of_a_kept_kind() {
        let bus = Bus::default();
        let (subscriber, inbox) = bus.attach();
        subscriber.set(wants(&[("live", 1000)]));
        bus.publish("live", &sample(1000));
        subscriber.set(wants(&[("live", 1000), ("disks", 0)]));
        bus.publish("live", &sample(1100));
        bus.publish("disks", &sample(1100));
        let topics: Vec<String> = inbox
            .try_iter()
            .map(|frame| frame["topic"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(topics, ["live", "disks"]);
    }

    #[test]
    fn a_stalled_subscriber_is_dropped_and_the_others_carry_on() {
        let bus = Bus::default();
        let (stalled, _unread) = bus.attach();
        let (healthy, inbox) = bus.attach();
        stalled.set(wants(&[("disks", 0)]));
        healthy.set(wants(&[("disks", 0)]));
        for n in 1..=(OUTBOX as u64 + 5) {
            bus.publish("disks", &sample(n));
            inbox.try_iter().for_each(drop);
        }
        assert_eq!(lock(&bus.subscribers).len(), 1);
        bus.publish("disks", &sample(1_000));
        assert_eq!(inbox.try_iter().count(), 1);
    }

    #[test]
    fn a_closed_subscriber_is_dropped() {
        let bus = Bus::default();
        let (subscriber, inbox) = bus.attach();
        subscriber.set(wants(&[("disks", 0)]));
        drop(inbox);
        bus.publish("disks", &sample(1));
        assert!(lock(&bus.subscribers).is_empty());
    }

    #[test]
    fn a_failure_reaches_holders_and_says_stale() {
        let bus = Bus::default();
        let (subscriber, inbox) = bus.attach();
        subscriber.set(wants(&[("live", 1000)]));
        bus.fail("live", "no /proc");
        let frame = inbox.try_recv().unwrap();
        assert_eq!(frame["stale"], true);
        assert_eq!(frame["error"], "no /proc");
        bus.fail("disks", "ignored");
        assert!(inbox.try_recv().is_err());
    }

    #[test]
    fn a_change_wakes_a_waiter() {
        let bus = Arc::new(Bus::default());
        let waiter = bus.clone();
        let handle = std::thread::spawn(move || {
            let began = std::time::Instant::now();
            waiter.wait(Duration::from_secs(10));
            began.elapsed()
        });
        std::thread::sleep(Duration::from_millis(100));
        bus.poke();
        assert!(handle.join().unwrap() < Duration::from_secs(5));
    }
}
