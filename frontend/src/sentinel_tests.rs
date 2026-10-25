//! The unit tests of `sentinel.rs`, in a file of their own to keep both under the line budget.

use super::*;
use crate::live::{Latest, Process, Socket};

fn proc(pid: u64, cpu: Option<f64>, rss: Option<f64>) -> Process {
    Process {
        pid,
        comm: format!("p{pid}"),
        cmdline: format!("/bin/p{pid} --x"),
        uid: Some(1000.0),
        rss_kb: rss,
        ticks: Some(0.0),
        start_ticks: Some(0.0),
        cpu,
        kthread: false,
        mine: Some(true),
    }
}

fn sample(at: f64, processes: Vec<Process>, sockets: Vec<Socket>) -> Latest {
    Latest {
        at,
        processes,
        mem_total: Some(1_000_000.0),
        uptime_sec: Some(1_000_000.0),
        clk_tck: 100.0,
        tcp_sockets: sockets,
    }
}

fn listen(port: f64, pid: Option<f64>) -> Socket {
    Socket {
        state: "LISTEN".into(),
        addr: Some("0.0.0.0".into()),
        port: Some(port),
        uid: Some(1000.0),
        pid,
    }
}

#[test]
fn a_limit_outside_its_range_or_no_number_falls_back_or_clamps() {
    let c = |l, v: Value| clamp_threshold(l, Some(&v));
    assert_eq!(c(Limit::CpuPercent, json!(200)), 100);
    assert_eq!(c(Limit::CpuPercent, json!(10)), 50);
    assert_eq!(c(Limit::CpuPercent, json!(72.5)), 73, "a half goes up");
    assert_eq!(c(Limit::CpuPercent, json!("70")), 70);
    assert_eq!(c(Limit::CpuPercent, json!(null)), 80);
    assert_eq!(c(Limit::CpuPercent, json!("")), 80);
    assert_eq!(c(Limit::CpuPercent, json!("abc")), 80);
    assert_eq!(clamp_threshold(Limit::MemoryPercent, None), 20);
    assert_eq!(c(Limit::PortAgeDays, json!(0)), 1);
    let t = Thresholds::from_stored(Some(
        &json!({"sentinelCpuPercent": 60, "sentinelPortAgeDays": 999}),
    ));
    assert_eq!(
        (t.cpu_percent, t.memory_percent, t.port_age_days),
        (60, 20, 90)
    );
    assert_eq!(Thresholds::from_stored(None), Thresholds::default());
}

#[test]
fn sizes_and_ages_read_as_the_javascript_writes_them() {
    assert_eq!(human_kb(1000.0), "1000 kB");
    assert_eq!(
        human_kb(1280.0),
        "1.3 MB",
        "an exact tie goes up, as toFixed does"
    );
    assert_eq!(human_kb(1536.0), "1.5 MB");
    assert_eq!(human_kb(1024.0 * 150.0), "150 MB");
    assert_eq!(human_kb(1024.0 * 1024.0 * 1.25), "1.3 GB");
    assert_eq!(human_kb(-1.0), "");
    assert_eq!(human_kb(f64::NAN), "");
    assert_eq!(human_age(89.0), "89 s");
    assert_eq!(human_age(89.5), "90 s", "a half goes up");
    assert_eq!(human_age(90.0), "2 min");
    assert_eq!(human_age(5400.0), "2 h");
    assert_eq!(human_age(DAY * 2.0), "2 d");
    assert_eq!(human_age(-1.0), "");
    // 0.15 is a hair under its decimal, so it rounds down, where multiplying says a tie.
    assert_eq!(to_fixed1(0.15), "0.1");
    assert_eq!(to_fixed1(0.25), "0.3");
    assert_eq!(to_fixed1(0.35), "0.3");
}

#[test]
fn cpu_is_flagged_only_after_three_samples_in_a_row_over_the_limit() {
    let th = Thresholds::default();
    let mut t = Tracked::default();
    for (i, at) in [1000.0, 3000.0, 5000.0].into_iter().enumerate() {
        let l = sample(at, vec![proc(10, Some(95.0), Some(1.0))], vec![]);
        t = t.track(Some(&l), &th);
        let found = rows(Some(&l), &t, &th);
        assert_eq!(found.len(), usize::from(i == 2), "after {} samples", i + 1);
    }
    let l = sample(5000.0, vec![proc(10, Some(95.0), Some(1.0))], vec![]);
    let f = &rows(Some(&l), &t, &th)[0];
    assert_eq!(f.id, "cpu:10");
    assert_eq!(f.detail, "95% CPU for 4 s");
    assert!((f.ratio - 95.0 / 80.0).abs() < 1e-12);
    // Under the limit once forgets it, and the same sample twice changes nothing.
    let again = t.track(Some(&l), &th);
    assert_eq!(again, t);
    let calm = sample(7000.0, vec![proc(10, Some(5.0), Some(1.0))], vec![]);
    assert_eq!(t.track(Some(&calm), &th).over(10), None);
    assert_eq!(t.track(None, &th), Tracked::default());
}

#[test]
fn a_kernel_thread_and_pid_one_are_never_findings() {
    let th = Thresholds::default();
    let mut kthread = proc(50, Some(99.0), Some(900_000.0));
    kthread.kthread = true;
    let init = proc(1, Some(99.0), Some(900_000.0));
    let l = sample(1000.0, vec![kthread, init], vec![]);
    let t = Tracked::default().track(Some(&l), &th);
    assert!(rows(Some(&l), &t, &th).is_empty());
}

#[test]
fn memory_is_flagged_by_its_share_of_memory() {
    let th = Thresholds::default();
    let l = sample(
        1000.0,
        vec![proc(9, None, Some(300_000.0)), proc(8, None, Some(100.0))],
        vec![],
    );
    let found = rows(Some(&l), &Tracked::default(), &th);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "memory:9");
    assert_eq!(found[0].detail, "293 MB, 30% of memory");
    assert!((found[0].ratio - 1.5).abs() < 1e-12);
}

#[test]
fn a_port_is_listed_once_per_process_and_flagged_when_its_process_is_old() {
    let th = Thresholds::default();
    let mut old = proc(20, None, Some(1.0));
    old.start_ticks = Some(100.0);
    let mut young = proc(21, None, Some(1.0));
    young.start_ticks = Some(100_000_000.0 - 100.0);
    let mut l = sample(
        1000.0,
        vec![old, young],
        vec![
            listen(80.0, Some(20.0)),
            listen(80.0, Some(20.0)),
            listen(81.0, Some(21.0)),
            listen(53.0, None),
            Socket {
                state: "ESTABLISHED".into(),
                ..listen(9.0, Some(20.0))
            },
        ],
    );
    l.uptime_sec = Some(100_000_000.0 / 100.0);
    let found = rows(Some(&l), &Tracked::default(), &th);
    let ids: Vec<_> = found.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(ids, ["port:80:20", "port:81:21", "port:53:u1000"]);
    assert!(found[0].flagged && !found[1].flagged && !found[2].flagged);
    assert_eq!(found[0].detail, ":80 · all interfaces · process up 12 d");
    assert_eq!(
        found[2].detail,
        ":53 · all interfaces · owner unknown (uid 1000)"
    );
    assert_eq!(found[2].comm, ":53");
}

#[test]
fn several_old_ports_of_one_process_are_one_finding() {
    let th = Thresholds::default();
    let mut old = proc(20, None, Some(1.0));
    old.start_ticks = Some(100.0);
    let ports: Vec<_> = [80.0, 81.0, 82.0, 83.0, 84.0]
        .iter()
        .map(|p| listen(*p, Some(20.0)))
        .collect();
    let l = sample(1000.0, vec![old], ports);
    let all = rows(Some(&l), &Tracked::default(), &th);
    assert_eq!(all.len(), 5);
    let flagged = list(&all, Filter::Flagged, "");
    assert_eq!(flagged.len(), 1);
    assert_eq!(flagged[0].id, "ports:20");
    assert_eq!(
        flagged[0].detail,
        "process up 12 d · :80 all interfaces, :81 all interfaces, :82 all interfaces, +2 more"
    );
    assert_eq!(flagged[0].search.as_deref(), Some(":80 :81 :82 :83 :84"));
    assert_eq!(Counts::of(&all).flagged, 1);
    assert_eq!(Counts::of(&all).port, 5);
    assert_eq!(list(&all, Filter::Port, "").len(), 5);
    assert_eq!(
        list(&all, Filter::Flagged, ":83").len(),
        1,
        "found by a port it hides"
    );
}

#[test]
fn the_list_is_searched_ordered_and_capped() {
    let th = Thresholds {
        cpu_percent: 50,
        ..Thresholds::default()
    };
    let procs: Vec<_> = (2..=70)
        .map(|n| proc(n, Some(60.0 + n as f64), Some(1.0)))
        .collect();
    let l0 = sample(1000.0, procs.clone(), vec![]);
    let mut t = Tracked::default();
    let mut l = l0;
    for step in 0..3 {
        l.at = 1000.0 + 2000.0 * step as f64;
        t = t.track(Some(&l), &th);
    }
    let all = rows(Some(&l), &t, &th);
    let shown = list(&all, Filter::Cpu, "");
    assert_eq!(shown.len(), ROW_CAP, "capped");
    assert_eq!(shown[0].id, "cpu:70", "the most over first");
    assert_eq!(list(&all, Filter::Cpu, "P33").len(), 1);
    assert_eq!(list(&all, Filter::Memory, "").len(), 0);
}
