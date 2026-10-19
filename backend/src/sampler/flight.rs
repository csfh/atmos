//! One collection per kind at a time. A request for a kind that is already
//! being collected waits for that run instead of starting a second one.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use super::cache::Sample;

pub type Outcome = Result<Arc<Sample>, String>;

struct Slot {
    done: Mutex<Option<Outcome>>,
    ready: Condvar,
}

#[derive(Default)]
pub struct Flights {
    slots: Mutex<HashMap<String, Arc<Slot>>>,
    interactive: AtomicUsize,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|err| err.into_inner())
}

impl Flights {
    /// Interactive collections running now. The scheduled lane waits while this is non-zero.
    pub fn interactive_active(&self) -> usize {
        self.interactive.load(Ordering::SeqCst)
    }

    /// Run `work` for `kind`, or wait for the run already in progress.
    ///
    /// A panicking collector is an error for everyone waiting on it, so the
    /// daemon outlives one bad inventory.
    pub fn run(
        &self,
        kind: &str,
        interactive: bool,
        work: impl FnOnce() -> Result<Sample, String>,
    ) -> Outcome {
        // A reader never joins a background run: that run may crawl at idle priority.
        let key = format!("{}:{kind}", if interactive { "fg" } else { "bg" });
        let (slot, owner) = {
            let mut slots = lock(&self.slots);
            match slots.get(&key) {
                Some(slot) => (slot.clone(), false),
                None => {
                    let slot = Arc::new(Slot {
                        done: Mutex::new(None),
                        ready: Condvar::new(),
                    });
                    slots.insert(key.clone(), slot.clone());
                    (slot, true)
                }
            }
        };
        if !owner {
            let mut done = lock(&slot.done);
            while done.is_none() {
                done = slot.ready.wait(done).unwrap_or_else(|err| err.into_inner());
            }
            return done.clone().unwrap_or_else(|| Err("lost".into()));
        }
        if interactive {
            self.interactive.fetch_add(1, Ordering::SeqCst);
        }
        let outcome: Outcome = catch_unwind(AssertUnwindSafe(work))
            .unwrap_or_else(|_| Err(format!("the {kind} collector panicked")))
            .map(Arc::new);
        if interactive {
            self.interactive.fetch_sub(1, Ordering::SeqCst);
        }
        lock(&self.slots).remove(&key);
        *lock(&slot.done) = Some(outcome.clone());
        slot.ready.notify_all();
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    fn sample(n: u64) -> Sample {
        Sample {
            at_ms: n,
            took_ms: 0,
            value: json!(n),
        }
    }

    #[test]
    fn simultaneous_requests_share_one_collection() {
        let flights = Arc::new(Flights::default());
        let runs = Arc::new(AtomicUsize::new(0));
        let gate = Arc::new(AtomicBool::new(false));
        let owner = {
            let (flights, runs, gate) = (flights.clone(), runs.clone(), gate.clone());
            std::thread::spawn(move || {
                flights.run("services", true, || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    while !gate.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Ok(sample(7))
                })
            })
        };
        while runs.load(Ordering::SeqCst) == 0 {
            std::thread::sleep(Duration::from_millis(5));
        }
        let joiner = {
            let (flights, runs) = (flights.clone(), runs.clone());
            std::thread::spawn(move || {
                flights.run("services", true, || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    Ok(sample(8))
                })
            })
        };
        std::thread::sleep(Duration::from_millis(50));
        gate.store(true, Ordering::SeqCst);
        let (a, b) = (
            owner.join().unwrap().unwrap(),
            joiner.join().unwrap().unwrap(),
        );
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        assert_eq!(a.at_ms, 7);
        assert_eq!(b.at_ms, 7);
    }

    #[test]
    fn an_interactive_request_never_joins_a_background_run() {
        let flights = Arc::new(Flights::default());
        let gate = Arc::new(AtomicBool::new(false));
        let started = Arc::new(AtomicBool::new(false));
        let background = {
            let (flights, gate, started) = (flights.clone(), gate.clone(), started.clone());
            std::thread::spawn(move || {
                flights.run("services", false, || {
                    started.store(true, Ordering::SeqCst);
                    while !gate.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Ok(sample(1))
                })
            })
        };
        while !started.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }
        let began = std::time::Instant::now();
        let own = flights.run("services", true, || Ok(sample(2))).unwrap();
        assert_eq!(own.at_ms, 2, "the reader ran its own collection");
        assert!(
            began.elapsed() < Duration::from_millis(500),
            "and did not wait"
        );
        gate.store(true, Ordering::SeqCst);
        assert_eq!(background.join().unwrap().unwrap().at_ms, 1);
    }

    #[test]
    fn different_kinds_run_side_by_side() {
        let flights = Arc::new(Flights::default());
        let other = {
            let flights = flights.clone();
            std::thread::spawn(move || {
                flights.run("disks", true, || {
                    std::thread::sleep(Duration::from_millis(100));
                    Ok(sample(1))
                })
            })
        };
        std::thread::sleep(Duration::from_millis(20));
        let started = std::time::Instant::now();
        flights.run("live", true, || Ok(sample(2))).unwrap();
        assert!(started.elapsed() < Duration::from_millis(80));
        other.join().unwrap().unwrap();
    }

    #[test]
    fn a_finished_collection_does_not_answer_the_next_request() {
        let flights = Flights::default();
        assert_eq!(
            flights.run("live", true, || Ok(sample(1))).unwrap().at_ms,
            1
        );
        assert_eq!(
            flights.run("live", true, || Ok(sample(2))).unwrap().at_ms,
            2
        );
    }

    #[test]
    fn a_panic_is_an_error_and_the_next_run_still_works() {
        let flights = Flights::default();
        let err = flights
            .run("hardware", true, || -> Result<Sample, String> {
                panic!("boom")
            })
            .unwrap_err();
        assert!(err.contains("panicked"), "{err}");
        assert_eq!(flights.interactive_active(), 0);
        assert!(flights.run("hardware", false, || Ok(sample(3))).is_ok());
    }

    #[test]
    fn interactive_runs_are_counted_while_they_run() {
        let flights = Arc::new(Flights::default());
        let seen = Arc::new(AtomicUsize::new(99));
        {
            let (inner, seen) = (flights.clone(), seen.clone());
            flights
                .run("live", true, move || {
                    seen.store(inner.interactive_active(), Ordering::SeqCst);
                    Ok(sample(1))
                })
                .unwrap();
        }
        assert_eq!(seen.load(Ordering::SeqCst), 1);
        assert_eq!(flights.interactive_active(), 0);
    }
}
