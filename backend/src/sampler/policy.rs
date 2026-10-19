//! How often each display kind is sampled and how old a sample may be when read.

use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    /// Time between scheduled samples.
    pub interval: Duration,
    /// A sample older than this is a miss for a reader.
    pub max_age: Duration,
    /// Delay before the first sample after the daemon starts with an empty cache.
    pub first: Duration,
    /// How long a reader waits on the daemon before it collects the kind itself.
    /// A starved or wedged daemon must cost a window seconds, never a stalled page.
    pub wait: Duration,
}

const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

/// `live` is cumulative counters that the window turns into rates, so a reader
/// may only be served a moment old and never the same sample twice (the window
/// drops a repeat by its `sampledAt`, but a fallback reader has no such stamp). The other kinds change slowly,
/// and a reader tolerates a few missed samples before it asks for a fresh one.
pub fn of(kind: &str) -> Policy {
    let (interval, max_age, first) = match kind {
        "live" => (secs(30), Duration::from_secs(1), secs(0)),
        "services" => (secs(300), secs(900), secs(2)),
        "disks" => (secs(300), secs(900), secs(4)),
        "software" => (secs(900), secs(2700), secs(6)),
        "diagnostics" => (secs(900), secs(2700), secs(10)),
        "hardware" => (secs(3600), secs(10800), secs(15)),
        _ => (secs(900), secs(2700), secs(30)),
    };
    // A window asks for `live` at 0.5 to 5 seconds; a reader gives up on it sooner than on the rest.
    let wait = if kind == "live" { secs(2) } else { secs(20) };
    Policy {
        interval,
        max_age,
        first,
        wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::KINDS;

    #[test]
    fn every_kind_is_sampled_slower_than_it_goes_stale() {
        for kind in KINDS {
            let policy = of(kind);
            assert!(policy.interval >= secs(30), "{kind} would be sampled hot");
            assert!(policy.max_age >= Duration::from_secs(1), "{kind}");
            assert!(policy.first <= secs(30), "{kind}");
        }
    }

    #[test]
    fn no_reader_waits_on_the_daemon_longer_than_a_page_tolerates() {
        assert!(of("live").wait <= secs(2));
        for kind in KINDS {
            assert!(of(kind).wait <= secs(20), "{kind}");
        }
    }

    #[test]
    fn live_is_never_served_older_than_a_poll() {
        assert!(of("live").max_age < secs(2));
    }

    #[test]
    fn slow_kinds_survive_missed_samples() {
        for kind in KINDS.iter().filter(|k| **k != "live") {
            assert!(of(kind).max_age >= of(kind).interval * 2, "{kind}");
        }
    }
}
