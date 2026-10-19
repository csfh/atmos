//! Decides whether a background collection may start.
//!
//! Three things hold it back: the machine is under CPU pressure, the sampler
//! has already used its share of one core over the recent window, or both. It
//! is pure bookkeeping over an injected probe and clock, so a test can drive
//! pressure and CPU use directly.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// `some avg10` of `/proc/pressure/cpu`, in percent, above which sampling waits.
    pub pressure_max: f64,
    /// Load average per core, used only when the machine has no pressure file.
    pub load_per_core_max: f64,
    /// The sampler's CPU, children included, as a fraction of one core over `window`.
    pub duty: f64,
    pub window: Duration,
    /// Pressure waits start here and double up to the cap, and reset when it clears.
    pub backoff_min: Duration,
    pub backoff_max: Duration,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            pressure_max: 4.0,
            load_per_core_max: 0.7,
            duty: 0.01,
            window: Duration::from_secs(300),
            backoff_min: Duration::from_secs(5),
            backoff_max: Duration::from_secs(300),
        }
    }
}

/// What the machine looks like right now.
pub trait Probe {
    /// CPU pressure `some avg10`, percent. `None` when the kernel has no PSI.
    fn pressure(&self) -> Option<f64>;
    fn load_per_core(&self) -> Option<f64>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verdict {
    Go,
    Wait { wait: Duration, why: &'static str },
}

pub struct Governor {
    limits: Limits,
    spent: VecDeque<(Instant, f64)>,
    strikes: u32,
}

impl Governor {
    pub fn new(limits: Limits) -> Governor {
        Governor {
            limits,
            spent: VecDeque::new(),
            strikes: 0,
        }
    }

    /// CPU seconds one finished background collection cost, children included.
    pub fn spent(&mut self, now: Instant, cpu_secs: f64) {
        self.spent.push_back((now, cpu_secs.max(0.0)));
    }

    pub fn verdict(&mut self, now: Instant, probe: &dyn Probe) -> Verdict {
        while let Some(&(at, _)) = self.spent.front() {
            if now.saturating_duration_since(at) >= self.limits.window {
                self.spent.pop_front();
            } else {
                break;
            }
        }
        let used: f64 = self.spent.iter().map(|(_, secs)| secs).sum();
        let allowed = self.limits.duty * self.limits.window.as_secs_f64();
        if used >= allowed {
            // Wait until the oldest charge leaves the window.
            let wait = self
                .spent
                .front()
                .map_or(self.limits.backoff_min, |(at, _)| {
                    (*at + self.limits.window).saturating_duration_since(now)
                });
            return Verdict::Wait {
                wait: wait.max(Duration::from_secs(1)),
                why: "the sampler has used its share of a core",
            };
        }
        let busy = match probe.pressure() {
            Some(pressure) => pressure > self.limits.pressure_max,
            None => probe
                .load_per_core()
                .is_some_and(|load| load > self.limits.load_per_core_max),
        };
        if busy {
            let doubled = self
                .limits
                .backoff_min
                .saturating_mul(1u32 << self.strikes.min(16));
            self.strikes = self.strikes.saturating_add(1);
            return Verdict::Wait {
                wait: doubled.min(self.limits.backoff_max),
                why: "the machine is busy",
            };
        }
        self.strikes = 0;
        Verdict::Go
    }
}

/// The real machine, read from `/proc`.
pub struct ProcProbe;

impl Probe for ProcProbe {
    fn pressure(&self) -> Option<f64> {
        parse_pressure(&std::fs::read_to_string("/proc/pressure/cpu").ok()?)
    }

    fn load_per_core(&self) -> Option<f64> {
        let load = parse_load(&std::fs::read_to_string("/proc/loadavg").ok()?)?;
        let cores = std::thread::available_parallelism().ok()?.get() as f64;
        Some(load / cores)
    }
}

/// `some avg10=1.23 avg60=...` from `/proc/pressure/cpu`.
pub fn parse_pressure(text: &str) -> Option<f64> {
    let line = text.lines().find(|line| line.starts_with("some "))?;
    line.split_whitespace()
        .find_map(|word| word.strip_prefix("avg10="))?
        .parse()
        .ok()
}

/// The one-minute load average from `/proc/loadavg`.
pub fn parse_load(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

/// CPU seconds a process and the children it has reaped have used, from `/proc/PID/stat`.
pub fn process_cpu_secs(pid: u32) -> Option<f64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_stat_cpu(&text, crate::inventory::live::clk_tck() as f64)
}

/// utime, stime, cutime and cstime, in ticks, as seconds. The name in field two
/// may hold spaces and parentheses, so the fields are counted from its last `)`.
pub fn parse_stat_cpu(text: &str, ticks_per_sec: f64) -> Option<f64> {
    let rest = &text[text.rfind(')')? + 1..];
    let fields: Vec<&str> = rest.split_whitespace().collect();
    // After the name: state is field 3, so utime (14) is index 11.
    let ticks: f64 = (11..15)
        .map(|i| fields.get(i)?.parse::<f64>().ok())
        .sum::<Option<f64>>()?;
    Some(ticks / ticks_per_sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        pressure: Option<f64>,
        load: Option<f64>,
    }
    impl Probe for Fake {
        fn pressure(&self) -> Option<f64> {
            self.pressure
        }
        fn load_per_core(&self) -> Option<f64> {
            self.load
        }
    }
    const QUIET: Fake = Fake {
        pressure: Some(0.0),
        load: Some(0.1),
    };

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn a_quiet_machine_with_budget_left_may_sample() {
        let mut governor = Governor::new(Limits::default());
        assert_eq!(governor.verdict(Instant::now(), &QUIET), Verdict::Go);
    }

    #[test]
    fn pressure_holds_sampling_back_and_the_wait_doubles_to_a_cap() {
        let mut governor = Governor::new(Limits::default());
        let busy = Fake {
            pressure: Some(30.0),
            load: None,
        };
        let now = Instant::now();
        let waits: Vec<Duration> = (0..9)
            .map(|_| match governor.verdict(now, &busy) {
                Verdict::Wait { wait, .. } => wait,
                Verdict::Go => panic!("a busy machine must wait"),
            })
            .collect();
        assert_eq!(waits[0], secs(5));
        assert_eq!(waits[1], secs(10));
        assert_eq!(waits[2], secs(20));
        assert_eq!(
            *waits.last().unwrap(),
            secs(300),
            "the backoff stops at its cap"
        );
    }

    #[test]
    fn calm_resets_the_backoff() {
        let mut governor = Governor::new(Limits::default());
        let busy = Fake {
            pressure: Some(30.0),
            load: None,
        };
        let now = Instant::now();
        governor.verdict(now, &busy);
        governor.verdict(now, &busy);
        assert_eq!(governor.verdict(now, &QUIET), Verdict::Go);
        assert!(matches!(
            governor.verdict(now, &busy),
            Verdict::Wait { wait, .. } if wait == secs(5)
        ));
    }

    #[test]
    fn without_a_pressure_file_load_per_core_decides() {
        let mut governor = Governor::new(Limits::default());
        let loaded = Fake {
            pressure: None,
            load: Some(2.0),
        };
        assert!(matches!(
            governor.verdict(Instant::now(), &loaded),
            Verdict::Wait {
                why: "the machine is busy",
                ..
            }
        ));
        let idle = Fake {
            pressure: None,
            load: Some(0.1),
        };
        assert_eq!(governor.verdict(Instant::now(), &idle), Verdict::Go);
        let unknown = Fake {
            pressure: None,
            load: None,
        };
        assert_eq!(
            governor.verdict(Instant::now(), &unknown),
            Verdict::Go,
            "no signal at all is not a reason to stop"
        );
    }

    #[test]
    fn an_overspent_budget_waits_until_the_oldest_charge_leaves_the_window() {
        let limits = Limits::default();
        let allowed = limits.duty * limits.window.as_secs_f64();
        let mut governor = Governor::new(limits);
        let start = Instant::now();
        governor.spent(start, allowed + 0.5);
        match governor.verdict(start + secs(100), &QUIET) {
            Verdict::Wait { wait, why } => {
                assert_eq!(why, "the sampler has used its share of a core");
                assert!(wait > secs(190) && wait <= secs(200), "{wait:?}");
            }
            Verdict::Go => panic!("an overspent budget must wait"),
        }
        assert_eq!(
            governor.verdict(start + limits.window + secs(1), &QUIET),
            Verdict::Go,
            "the charge ages out"
        );
    }

    #[test]
    fn small_charges_add_up() {
        let limits = Limits::default();
        let allowed = limits.duty * limits.window.as_secs_f64();
        let mut governor = Governor::new(limits);
        let now = Instant::now();
        for _ in 0..4 {
            governor.spent(now, allowed / 4.0);
        }
        assert!(matches!(
            governor.verdict(now, &QUIET),
            Verdict::Wait { .. }
        ));
    }

    #[test]
    fn pressure_files_and_stat_lines_parse() {
        let psi = "some avg10=1.50 avg60=0.40 avg300=0.10 total=123\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        assert_eq!(parse_pressure(psi), Some(1.5));
        assert_eq!(parse_pressure("full avg10=9.00\n"), None);
        assert_eq!(parse_pressure(""), None);
        assert_eq!(parse_load("0.98 1.02 2.03 2/2477 424401\n"), Some(0.98));
        assert_eq!(parse_load(""), None);
    }

    #[test]
    fn stat_cpu_counts_user_system_and_children_and_survives_odd_names() {
        // utime=100 stime=50 cutime=30 cstime=20 -> 200 ticks.
        let stat =
            "42 (ratmos sampler) (x) S 1 42 42 0 -1 4194560 100 0 0 0 100 50 30 20 20 0 1 0 5 0 0";
        assert_eq!(parse_stat_cpu(stat, 100.0), Some(2.0));
        assert_eq!(parse_stat_cpu("garbage", 100.0), None);
    }

    #[test]
    fn the_real_probe_and_own_stat_are_readable_here() {
        assert!(process_cpu_secs(std::process::id()).is_some());
        // Either answer is valid on a given kernel; neither may panic.
        let _ = ProcProbe.pressure();
        let _ = ProcProbe.load_per_core();
    }
}
