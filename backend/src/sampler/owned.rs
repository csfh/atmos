//! The daemon's own sampler (`ratmos daemon --with-sampler`, docs/adr/0014).
//!
//! Sampling stays in a process of its own, at the lowest priorities, behind the governor
//! (docs/adr/0002): the daemon answers windows that are waiting for a page and must not be slowed
//! by it. With the flag the daemon starts that process itself and keeps one alive, which a Bundle
//! needs, since `oma` starts one program and the units of an install that start `ratmos sampler`
//! are not there. Without the flag nothing here runs.
//!
//! The child is `ratmos sampler --owned`: it ends with this daemon (see `worker::run`), so a
//! daemon that leaves, or is replaced, takes it along and the next one starts its own.

use std::io;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How soon after starting one the daemon starts another. A sampler that cannot run at all
/// (no `ratmos` to start, a runtime directory that is gone) is tried again this often.
const RESPAWN: Duration = Duration::from_secs(10);

type Spawn = Box<dyn FnMut() -> io::Result<Child> + Send>;

pub(super) struct OwnedSampler {
    spawn: Spawn,
    child: Option<Child>,
    started: Option<Instant>,
}

impl OwnedSampler {
    /// Starts `ratmos sampler --owned` with `nice` and, when there is one, `ionice` in front of
    /// it. Both exec, so the child is the sampler itself and its parent is this daemon.
    pub(super) fn production() -> Self {
        Self::with(Box::new(|| {
            let exe = std::env::current_exe()?;
            let mut command = Command::new("nice");
            command.args(["-n", "19"]);
            if crate::tasks::present("ionice") {
                command.args(["ionice", "-c", "3"]);
            }
            command
                .arg(exe)
                .args(["sampler", "--owned"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
        }))
    }

    pub(super) fn with(spawn: Spawn) -> Self {
        Self {
            spawn,
            child: None,
            started: None,
        }
    }

    /// Called every beat of the scheduled lane: starts a sampler when none is connected and none
    /// of ours is alive, and not more often than [`RESPAWN`].
    pub(super) fn keep(&mut self, connected: bool, now: Instant) {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    eprintln!("ratmos daemon: its sampler ended ({status})");
                    self.child = None;
                }
                Ok(None) => {}
                Err(err) => eprintln!("ratmos daemon: cannot look at its sampler: {err}"),
            }
        }
        if connected || self.child.is_some() {
            return;
        }
        if self
            .started
            .is_some_and(|at| now.duration_since(at) < RESPAWN)
        {
            return;
        }
        self.started = Some(now);
        match (self.spawn)() {
            Ok(child) => {
                eprintln!("ratmos daemon: started its sampler (pid {})", child.id());
                self.child = Some(child);
            }
            Err(err) => eprintln!("ratmos daemon: cannot start a sampler: {err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A keeper whose sampler is `program` with `args`, and how many it has started.
    fn keeper(
        program: &'static str,
        args: &'static [&'static str],
    ) -> (OwnedSampler, Arc<AtomicUsize>) {
        let starts = Arc::new(AtomicUsize::new(0));
        let counted = starts.clone();
        let keeper = OwnedSampler::with(Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            Command::new(program)
                .args(args)
                .stdin(Stdio::null())
                .spawn()
        }));
        (keeper, starts)
    }

    #[test]
    fn a_sampler_that_is_connected_is_left_alone() {
        let (mut keeper, starts) = keeper("sleep", &["30"]);
        keeper.keep(true, Instant::now());
        assert_eq!(starts.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn one_is_started_when_none_is_connected_and_not_again_while_it_lives() {
        let (mut keeper, starts) = keeper("sleep", &["30"]);
        let t0 = Instant::now();
        keeper.keep(false, t0);
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        // It has not connected yet, and is alive: nothing more, however long it takes.
        keeper.keep(false, t0 + Duration::from_secs(60));
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        keeper.child.as_mut().unwrap().kill().unwrap();
    }

    #[test]
    fn one_that_ended_is_replaced_but_not_faster_than_the_respawn_interval() {
        let (mut keeper, starts) = keeper("true", &[]);
        let t0 = Instant::now();
        keeper.keep(false, t0);
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        keeper.child.as_mut().unwrap().wait().unwrap();
        keeper.keep(false, t0 + Duration::from_secs(1));
        assert_eq!(starts.load(Ordering::SeqCst), 1, "too soon");
        keeper.keep(false, t0 + RESPAWN);
        assert_eq!(starts.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_sampler_that_cannot_start_is_tried_again_later_not_in_a_loop() {
        let starts = Arc::new(AtomicUsize::new(0));
        let counted = starts.clone();
        let mut keeper = OwnedSampler::with(Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            Err(io::Error::new(io::ErrorKind::NotFound, "no ratmos"))
        }));
        let t0 = Instant::now();
        for beat in 0..5 {
            keeper.keep(false, t0 + Duration::from_secs(beat));
        }
        assert_eq!(starts.load(Ordering::SeqCst), 1);
        keeper.keep(false, t0 + RESPAWN);
        assert_eq!(starts.load(Ordering::SeqCst), 2);
    }
}
