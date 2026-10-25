//! Live samples from the daemon, as the QML window reads them (`services/LiveStats.js`,
//! `LiveStatsStore.qml`): a raw `live` frame is cleaned, and a process's CPU is worked out from the
//! tick counts of two frames. Only what the Sentinel page reads is here: the processes, the
//! listening sockets, and the memory, uptime and clock the rates and ages are measured in.
//!
//! The clean-up is `LiveStats.js`'s and its quirks are kept, since `tests/sentinel_parity.rs`
//! runs both on the same frames: a number is a finite number or nothing, a count is never
//! negative, `""` is no number but `"  "` is zero, and a process whose name is empty is no process.
//! One narrowing: a pid is a whole number, as the kernel's always is.

use std::collections::HashMap;

use serde_json::Value;

/// One process of a sample. A field that was missing or unusable is `None`, never zero.
#[derive(Debug, Clone, PartialEq)]
pub struct Process {
    pub pid: u64,
    pub comm: String,
    pub cmdline: String,
    pub uid: Option<f64>,
    pub rss_kb: Option<f64>,
    pub ticks: Option<f64>,
    pub start_ticks: Option<f64>,
    /// Percent of one core since the sample before, so it may pass 100; `None` for a process
    /// the sample before did not have, and for a counter that went backwards.
    pub cpu: Option<f64>,
    pub kthread: bool,
    /// Whether the user owns it; `None` when the backend did not say.
    pub mine: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Socket {
    pub state: String,
    pub addr: Option<String>,
    pub port: Option<f64>,
    pub uid: Option<f64>,
    pub pid: Option<f64>,
}

/// A sample taken in: when the daemon began collecting it (milliseconds, the daemon's clock, the
/// only one a rate is measured on), and what Sentinel reads of it.
#[derive(Debug, Clone, PartialEq)]
pub struct Latest {
    pub at: f64,
    pub processes: Vec<Process>,
    pub mem_total: Option<f64>,
    pub uptime_sec: Option<f64>,
    pub clk_tck: f64,
    pub tcp_sockets: Vec<Socket>,
}

/// `Number(v)` of a value that is not `""`, `null` or missing: a finite number or nothing
/// (`finiteNumber`). A boolean is 0 or 1 and a string is read as JavaScript would, as far as
/// decimal numbers go.
pub(crate) fn finite_number(v: Option<&Value>) -> Option<f64> {
    let n = match v? {
        Value::Null => return None,
        Value::Number(n) => n.as_f64()?,
        Value::String(s) if s.is_empty() => return None,
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                0.0
            } else if t.contains(|c: char| c.is_ascii_alphabetic() && c != 'e' && c != 'E') {
                return None;
            } else {
                t.parse::<f64>().ok()?
            }
        }
        Value::Bool(b) => f64::from(u8::from(*b)),
        _ => return None,
    };
    n.is_finite().then_some(n)
}

fn non_neg(v: Option<&Value>) -> Option<f64> {
    finite_number(v).filter(|n| *n >= 0.0)
}

/// `String(v || "")`.
fn text(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) | Some(Value::Bool(false)) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(true)) => "true".into(),
        Some(Value::Number(n)) => match n.as_f64() {
            Some(f) if f == 0.0 || f.is_nan() => String::new(),
            Some(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", f as i64),
            Some(f) => format!("{f}"),
            None => String::new(),
        },
        _ => String::new(),
    }
}

fn parse_process(row: &Value) -> Option<Process> {
    let row = row.as_object()?;
    // `Number(row.pid)` of a missing pid is NaN, which is no pid either.
    let pid = finite_number(row.get("pid"))?;
    if pid <= 0.0 || pid.fract() != 0.0 {
        return None;
    }
    let comm = text(row.get("comm")).trim().to_owned();
    if comm.is_empty() {
        return None;
    }
    Some(Process {
        pid: pid as u64,
        comm,
        cmdline: text(row.get("cmdline")),
        uid: finite_number(row.get("uid")),
        rss_kb: non_neg(row.get("rssKb")),
        ticks: non_neg(row.get("ticks")),
        start_ticks: non_neg(row.get("startTicks")),
        cpu: non_neg(row.get("cpu")),
        kthread: row.get("kthread") == Some(&Value::Bool(true)),
        mine: match row.get("mine") {
            Some(Value::Bool(b)) => Some(*b),
            _ => None,
        },
    })
}

fn parse_sockets(raw: Option<&Value>) -> Vec<Socket> {
    let Some(Value::Array(src)) = raw else {
        return Vec::new();
    };
    src.iter()
        .filter_map(|row| {
            let row = row.as_object()?;
            // `!src[i].state` is false for any text that is not empty; `0` and `false` are empty.
            let state = text(row.get("state"));
            if state.is_empty() {
                return None;
            }
            Some(Socket {
                state,
                addr: match row.get("addr") {
                    None | Some(Value::Null) => None,
                    other => Some(match other {
                        Some(Value::String(s)) => s.clone(),
                        Some(v) => v.to_string(),
                        None => unreachable!(),
                    }),
                },
                port: non_neg(row.get("port")),
                uid: finite_number(row.get("uid")),
                pid: non_neg(row.get("pid")),
            })
        })
        .collect()
}

/// `processCpu`: percent of one core from two tick counts over `dt_ms`, or nothing when either
/// count is missing, no time passed, or the counter went backwards.
fn process_cpu(prev: Option<f64>, next: Option<f64>, dt_ms: f64, clk_tck: f64) -> Option<f64> {
    let (prev, next) = (prev?, next?);
    // Both are finite here, so "not above zero" is "at or below it".
    if dt_ms <= 0.0 || clk_tck <= 0.0 {
        return None;
    }
    let d = next - prev;
    if d < 0.0 {
        return None;
    }
    Some((100.0 * d) / (clk_tck * (dt_ms / 1000.0)))
}

/// The samples taken in so far, as far as a rate needs them: the newest, and the clock of it.
#[derive(Debug, Default)]
pub struct Feed {
    latest: Option<Latest>,
    /// The clock of the newest sample taken in; a frame that is not after it is not taken in.
    last_at: f64,
}

impl Feed {
    pub fn latest(&self) -> Option<&Latest> {
        self.latest.as_ref()
    }

    /// Takes in a `live` frame, `result` taken at `sampled_at` (what `LiveStatsStore.adoptFrame`
    /// and `LiveStats.pushSample` do together). The same sample twice would read as a rate of
    /// zero, and an older one as time running backwards, so a frame that is not after the one
    /// before is left out, as is one that is no sample. True when it was taken in.
    pub fn adopt(&mut self, sampled_at: f64, result: &Value) -> bool {
        if sampled_at <= self.last_at {
            return false;
        }
        let Some(data) = result.as_object() else {
            return false;
        };
        let procs = match data.get("processes") {
            Some(Value::Array(list)) => list.as_slice(),
            _ => &[],
        };
        let parsed: Vec<Process> = procs.iter().filter_map(parse_process).collect();
        let clk_tck = non_neg(data.get("clkTck"))
            .filter(|c| *c != 0.0)
            .unwrap_or(100.0);
        let at = sampled_at;
        let prev = self.latest.as_ref();
        let dt = prev.map_or(0.0, |p| at - p.at);
        let before: HashMap<u64, &Process> = prev
            .map(|p| p.processes.iter().map(|row| (row.pid, row)).collect())
            .unwrap_or_default();
        let processes = parsed
            .into_iter()
            .map(|mut row| {
                let last = before.get(&row.pid);
                row.cpu = process_cpu(last.and_then(|l| l.ticks), row.ticks, dt, clk_tck);
                row
            })
            .collect();
        self.latest = Some(Latest {
            at,
            processes,
            mem_total: non_neg(data.get("memTotal")),
            uptime_sec: non_neg(data.get("uptimeSec")),
            clk_tck,
            tcp_sockets: parse_sockets(data.get("tcpSockets")),
        });
        self.last_at = at;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn frame(rows: &[Value]) -> Value {
        json!({ "clkTck": 100, "memTotal": 1000, "uptimeSec": 50, "processes": rows })
    }

    fn row(pid: u64, ticks: u64) -> Value {
        json!({ "pid": pid, "comm": format!("p{pid}"), "cmdline": "", "rssKb": 1, "ticks": ticks,
                "startTicks": 1, "kthread": false, "mine": true })
    }

    #[test]
    fn the_first_sample_has_no_rate_and_the_second_measures_it_on_the_daemons_clock() {
        let mut feed = Feed::default();
        assert!(feed.adopt(1000.0, &frame(&[row(7, 100)])));
        assert_eq!(feed.latest().unwrap().processes[0].cpu, None);
        // 150 ticks at 100 per second over 2 seconds: 75% of one core.
        assert!(feed.adopt(3000.0, &frame(&[row(7, 250)])));
        assert_eq!(feed.latest().unwrap().processes[0].cpu, Some(75.0));
        // It may pass 100: more than one core.
        assert!(feed.adopt(4000.0, &frame(&[row(7, 450)])));
        assert_eq!(feed.latest().unwrap().processes[0].cpu, Some(200.0));
    }

    #[test]
    fn a_frame_that_is_not_after_the_last_is_left_out() {
        let mut feed = Feed::default();
        assert!(feed.adopt(2000.0, &frame(&[row(7, 100)])));
        assert!(
            !feed.adopt(2000.0, &frame(&[row(7, 999)])),
            "the same sample twice"
        );
        assert!(!feed.adopt(1000.0, &frame(&[row(7, 999)])), "an older one");
        assert!(!feed.adopt(3000.0, &json!("not a sample")));
        assert_eq!(feed.latest().unwrap().at, 2000.0);
    }

    #[test]
    fn a_counter_that_went_backwards_or_a_new_process_has_no_rate() {
        let mut feed = Feed::default();
        feed.adopt(1000.0, &frame(&[row(7, 500)]));
        feed.adopt(2000.0, &frame(&[row(7, 100), row(8, 100)]));
        let l = feed.latest().unwrap();
        assert_eq!(l.processes[0].cpu, None, "ticks went down");
        assert_eq!(l.processes[1].cpu, None, "no sample before for it");
    }

    #[test]
    fn what_is_no_process_is_left_out() {
        let mut feed = Feed::default();
        let rows = [
            json!({"pid": 0, "comm": "zero"}),
            json!({"pid": 5, "comm": "  "}),
            json!({"pid": -3, "comm": "neg"}),
            json!({"pid": 1.5, "comm": "frac"}),
            json!({"comm": "nopid"}),
            json!("text"),
            json!({"pid": "9", "comm": " ok ", "rssKb": "12", "ticks": -1}),
        ];
        feed.adopt(1000.0, &frame(&rows));
        let l = feed.latest().unwrap();
        assert_eq!(l.processes.len(), 1);
        let p = &l.processes[0];
        assert_eq!(
            (p.pid, p.comm.as_str(), p.rss_kb, p.ticks),
            (9, "ok", Some(12.0), None)
        );
    }

    #[test]
    fn numbers_read_as_javascript_reads_them() {
        let n = |v: Value| finite_number(Some(&v));
        assert_eq!(n(json!("")), None);
        assert_eq!(n(json!("  ")), Some(0.0));
        assert_eq!(n(json!(" 12 ")), Some(12.0));
        assert_eq!(n(json!("1e3")), Some(1000.0));
        assert_eq!(n(json!("abc")), None);
        assert_eq!(n(json!("Infinity")), None);
        assert_eq!(n(json!(true)), Some(1.0));
        assert_eq!(n(json!(null)), None);
        assert_eq!(non_neg(Some(&json!(-1))), None);
    }

    #[test]
    fn a_clock_of_nothing_is_a_hundred_ticks() {
        let mut feed = Feed::default();
        let mut f = frame(&[row(7, 0)]);
        f["clkTck"] = json!(0);
        feed.adopt(1000.0, &f);
        assert_eq!(feed.latest().unwrap().clk_tck, 100.0);
    }

    #[test]
    fn sockets_keep_what_the_backend_says_and_drop_the_stateless() {
        let mut feed = Feed::default();
        let mut f = frame(&[]);
        f["tcpSockets"] = json!([
            {"state": "LISTEN", "addr": "0.0.0.0", "port": 80, "uid": 0, "pid": null},
            {"state": "", "port": 1},
            {"addr": "x"},
            {"state": "ESTABLISHED", "addr": null, "port": "443", "pid": 12},
        ]);
        feed.adopt(1000.0, &f);
        let s = &feed.latest().unwrap().tcp_sockets;
        assert_eq!(s.len(), 2);
        assert_eq!(
            (s[0].port, s[0].pid, s[0].addr.as_deref()),
            (Some(80.0), None, Some("0.0.0.0"))
        );
        assert_eq!(
            (s[1].port, s[1].pid, s[1].addr.as_deref()),
            (Some(443.0), Some(12.0), None)
        );
    }
}
