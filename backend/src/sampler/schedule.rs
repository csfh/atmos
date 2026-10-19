//! When each kind is next sampled. Pure bookkeeping: callers pass the clock.

use std::time::{Duration, Instant};

use super::policy;
use crate::display::KINDS;

/// A kind that was just dropped is resampled this soon, so a burst of
/// changes shares one collection.
pub const DIRTY_DELAY: Duration = Duration::from_secs(5);

/// ...but never sooner than this after the kind's last sample, so a stream of
/// tasks cannot turn slow sampling into constant sampling.
pub const DIRTY_FLOOR: Duration = Duration::from_secs(30);

/// When a dropped kind is next due: soon, but not within `floor` of its last sample.
pub fn dirty_due(now: Instant, last: Option<Instant>, floor: Duration) -> Instant {
    let soon = now + DIRTY_DELAY;
    last.map_or(soon, |last| soon.max(last + floor))
}

pub struct Schedule {
    due: Vec<(&'static str, Instant)>,
}

impl Schedule {
    /// A kind with a sample already cached waits out the rest of its interval.
    /// An empty one is staggered by its `first` offset so start-up is gentle.
    pub fn new(now: Instant, age_of: impl Fn(&str) -> Option<Duration>) -> Schedule {
        let due = KINDS
            .iter()
            .map(|kind| {
                let policy = policy::of(kind);
                let wait = match age_of(kind) {
                    Some(age) => policy.interval.saturating_sub(age),
                    None => policy.first,
                };
                (*kind, now + wait)
            })
            .collect();
        Schedule { due }
    }

    /// The kind that is due first; the earlier entry in `KINDS` wins a tie.
    pub fn next(&self) -> (&'static str, Instant) {
        self.due
            .iter()
            .copied()
            .min_by_key(|(_, due)| *due)
            .unwrap_or(("live", Instant::now()))
    }

    /// `jitter` scales the interval (about 0.9 to 1.1) so kinds drift apart.
    pub fn finished(&mut self, kind: &str, now: Instant, jitter: f64) {
        let interval = policy::of(kind).interval.mul_f64(jitter.clamp(0.5, 1.5));
        if let Some(entry) = self.due.iter_mut().find(|(k, _)| *k == kind) {
            entry.1 = now + interval;
        }
    }

    /// Pull a kind forward, never push it back.
    pub fn bring_forward(&mut self, kind: &str, by: Instant) {
        if let Some(entry) = self.due.iter_mut().find(|(k, _)| *k == kind) {
            if entry.1 > by {
                entry.1 = by;
            }
        }
    }
}

/// About 0.9 to 1.1, from the clock; nothing here needs real randomness.
pub fn jitter() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    0.9 + f64::from(nanos % 200) / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cold_start_samples_live_first_and_hardware_last() {
        let now = Instant::now();
        let mut schedule = Schedule::new(now, |_| None);
        let mut order = Vec::new();
        for _ in KINDS {
            let (kind, due) = schedule.next();
            order.push(kind);
            schedule.finished(kind, due + Duration::from_secs(10_000), 1.0);
        }
        assert_eq!(order.first(), Some(&"live"));
        assert_eq!(order.last(), Some(&"hardware"));
        assert_eq!(order.len(), KINDS.len());
    }

    #[test]
    fn a_cached_kind_waits_out_its_interval() {
        let now = Instant::now();
        let schedule = Schedule::new(now, |kind| {
            (kind == "services").then(|| Duration::from_secs(100))
        });
        let services = policy::of("services").interval;
        let due = schedule
            .due
            .iter()
            .find(|(k, _)| *k == "services")
            .unwrap()
            .1;
        assert_eq!(due, now + services - Duration::from_secs(100));
    }

    #[test]
    fn an_old_cached_kind_is_due_at_once() {
        let now = Instant::now();
        let schedule = Schedule::new(now, |_| Some(Duration::from_secs(100_000)));
        assert_eq!(schedule.next().1, now);
    }

    #[test]
    fn finishing_schedules_the_next_run_one_interval_on() {
        let now = Instant::now();
        let mut schedule = Schedule::new(now, |_| None);
        schedule.finished("live", now, 1.0);
        let due = schedule.due.iter().find(|(k, _)| *k == "live").unwrap().1;
        assert_eq!(due, now + policy::of("live").interval);
    }

    #[test]
    fn a_dropped_kind_waits_out_the_floor_since_its_last_sample() {
        let now = Instant::now();
        assert_eq!(dirty_due(now, None, DIRTY_FLOOR), now + DIRTY_DELAY);
        let just = Some(now - Duration::from_secs(1));
        assert_eq!(
            dirty_due(now, just, DIRTY_FLOOR),
            now - Duration::from_secs(1) + DIRTY_FLOOR
        );
        let old = Some(now - Duration::from_secs(600));
        assert_eq!(dirty_due(now, old, DIRTY_FLOOR), now + DIRTY_DELAY);
        assert_eq!(
            dirty_due(now, just, Duration::ZERO),
            now + DIRTY_DELAY,
            "a zero floor leaves only the delay"
        );
    }

    #[test]
    fn jitter_stays_inside_its_band() {
        for _ in 0..50 {
            let j = jitter();
            assert!((0.9..1.1).contains(&j), "{j}");
        }
    }

    #[test]
    fn bring_forward_never_delays() {
        let now = Instant::now();
        let mut schedule = Schedule::new(now, |_| Some(Duration::ZERO));
        let before = schedule
            .due
            .iter()
            .find(|(k, _)| *k == "services")
            .unwrap()
            .1;
        schedule.bring_forward("services", now + Duration::from_secs(10_000));
        let after = schedule
            .due
            .iter()
            .find(|(k, _)| *k == "services")
            .unwrap()
            .1;
        assert_eq!(before, after);
        schedule.bring_forward("services", now + DIRTY_DELAY);
        let moved = schedule
            .due
            .iter()
            .find(|(k, _)| *k == "services")
            .unwrap()
            .1;
        assert_eq!(moved, now + DIRTY_DELAY);
    }
}
