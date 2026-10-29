//! `processes.rs` is `services/Processes.js` and `ProcessRow.qml`'s words: the same rows listed,
//! filtered, sorted and treed for every option, the same row texts and actions, and the same
//! signal writes. Without `node` the test says so and stops.

mod common;

use atmos_ui::processes::{self, Filter};
use common::qml;
use serde_json::{Value, json};

fn rows() -> Value {
    json!([
        {"pid": 1, "ppid": 0, "comm": "systemd", "cmdline": "/sbin/init", "uid": 0, "rssKb": 12000, "cpu": 0.2, "state": "S", "threads": 1, "mine": false},
        {"pid": 200, "ppid": 1, "comm": "firefox", "cmdline": "/usr/lib/firefox", "uid": 1000, "rssKb": 900000, "cpu": 31.5, "state": "R", "threads": 90, "mine": true, "readBps": 2048, "writeBps": 0},
        {"pid": 201, "ppid": 200, "comm": "Web Content", "uid": 1000, "rssKb": 300000, "cpu": 12, "state": "S", "threads": 30, "mine": true, "readBps": null, "writeBps": 5000},
        {"pid": 202, "ppid": 200, "comm": "web content", "uid": 1000, "rssKb": "250000", "cpu": 12, "state": "S", "mine": true},
        {"pid": 300, "ppid": 1, "comm": "bash", "uid": 1000, "rssKb": 4000, "cpu": null, "state": "S", "threads": 1},
        {"pid": 301, "ppid": 300, "comm": "htop", "uid": 1000, "rssKb": -5, "cpu": 2.5, "state": "R"},
        {"pid": 2, "ppid": 0, "comm": "kthreadd", "kthread": true, "state": "S", "cpu": 0},
        {"pid": 40, "ppid": 2, "comm": "kworker/0:1", "kthread": true, "state": "I", "cpu": 1.5},
        {"pid": 41, "ppid": 2, "comm": "kworker/0:2", "kthread": true, "state": "D", "cpu": 7},
        {"pid": 500, "ppid": 999, "comm": "orphan", "uid": 1001, "rssKb": 100, "cpu": 50, "state": "Z"},
        {"pid": 501, "comm": "stopped", "uid": "1000", "state": "T", "nice": -5, "depth": 7},
        {"pid": 502, "comm": "  ", "uid": 1000},
        {"pid": 0, "comm": "x"},
        null,
    ])
}

#[allow(clippy::type_complexity)]
const OPTIONS: &[(
    &str,
    &str,
    &str,
    &str,
    Option<f64>,
    f64,
    f64,
    usize,
    bool,
    bool,
)] = &[
    (
        "",
        "cpu",
        "mine",
        "all",
        Some(1000.0),
        0.0,
        0.0,
        80,
        false,
        false,
    ),
    ("", "cpu", "all", "all", None, 0.0, 0.0, 80, false, false),
    ("", "memory", "all", "all", None, 0.0, 0.0, 3, false, false),
    ("", "pid", "all", "all", None, 0.0, 0.0, 80, true, false),
    ("", "name", "all", "all", None, 0.0, 0.0, 80, false, false),
    (
        "", "threads", "all", "sleeping", None, 0.0, 0.0, 80, false, false,
    ),
    ("", "io", "all", "all", None, 0.0, 0.0, 80, false, false),
    ("", "cpu", "kernel", "all", None, 1.0, 0.0, 80, false, false),
    ("WEB", "cpu", "all", "all", None, 0.0, 0.0, 80, false, false),
    (
        "", "cpu", "all", "all", None, 0.0, 100000.0, 80, false, false,
    ),
    ("", "cpu", "all", "all", None, 0.0, 0.0, 80, false, true),
    ("", "pid", "all", "all", None, 0.0, 0.0, 80, true, true),
    (
        "",
        "cpu",
        "mine",
        "all",
        Some(1001.0),
        0.0,
        0.0,
        80,
        false,
        false,
    ),
    ("1", "cpu", "mine", "all", None, 0.0, 0.0, 80, false, false),
];

#[test]
fn the_lists_rows_and_signals_are_processes_js() {
    let opts: Vec<Value> = OPTIONS
        .iter()
        .map(|(q, sort, scope, state, uid, min_cpu, min_rss, cap, reverse, tree)| {
            json!({"query": q, "sort": sort, "opts": {"scope": scope, "state": state, "uid": uid, "minCpu": min_cpu, "minRssKb": min_rss, "cap": cap, "reverse": reverse, "tree": tree}})
        })
        .collect();
    let signals = json!([
        [200, "term"],
        ["300", "SIGKILL"],
        [1, "TERM"],
        [0, "KILL"],
        ["2", "9"],
        ["x", "TERM"],
        [400, "HUP"],
        [" 401 ", " 15 "]
    ]);
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const P = load("services/Processes.js");
const L = load("services/LiveStats.js");
const R = load("services/RichUi.js");
const Q = load("services/Requests.js");
const rows = {rows};
// ProcessRow.statusText.
const status = (row) => {{
  const bits = [];
  const pid = row ? String(row.pid) : "", state = row ? P.stateLabel(row.state) : "";
  const cpu = row ? L.formatPercent(row.cpu) : "";
  const rss = row && row.rssKb != null ? R.formatBytes(row.rssKb * 1024) : "";
  const thr = row && row.threads != null ? String(row.threads) + " thr" : "";
  const r = row ? L.formatBps(row.readBps) : "", w = row ? L.formatBps(row.writeBps) : "";
  const io = !r && !w ? "" : "↓ " + (r || "—") + "  ↑ " + (w || "—");
  for (const b of [pid, state, cpu, rss, thr, io]) if (b) bits.push(b);
  return bits.join("   ");
}};
const lists = {opts}.map((o) => P.list(rows, o.query, o.sort, o.opts));
process.stdout.write(JSON.stringify({{
  lists,
  normal: rows.map((r) => P.normalize(r)),
  status: lists[1].map(status),
  labels: ["R", "S", "D", "Z", "T", "t", "I", "X", ""].map((s) => P.stateLabel(s)),
  can: lists[1].map((r) => [P.canSignal(r), P.overflowActions(r).map((a) => a.id)]),
  signals: {signals}.map(([p, s]) => {{ const a = P.signalArgs(p, s); return a ? Q.write("process.signal", a) : null; }}),
  sorts: P.sortChips(),
  chips: [P.scopeChips(), P.stateChips(), P.minCpuChips(), P.minRssChips(), P.capChips()],
}}));
"#,
        rows = rows(),
        opts = json!(opts),
        signals = signals,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let all = rows();
    let all = all.as_array().unwrap();
    let lists: Vec<Vec<Value>> = OPTIONS
        .iter()
        .map(
            |(query, sort, scope, state, uid, min_cpu, min_rss_kb, cap, reverse, tree)| {
                processes::list(
                    all,
                    &Filter {
                        query,
                        sort,
                        scope,
                        state,
                        uid: *uid,
                        min_cpu: *min_cpu,
                        min_rss_kb: *min_rss_kb,
                        cap: *cap,
                        reverse: *reverse,
                        tree: *tree,
                    },
                )
            },
        )
        .collect();
    for (i, (ours, theirs)) in lists
        .iter()
        .zip(theirs["lists"].as_array().unwrap())
        .enumerate()
    {
        assert_eq!(&json!(ours), theirs, "{:?}", OPTIONS[i]);
    }
    let normal: Vec<Value> = all
        .iter()
        .map(|r| processes::normalize(r).unwrap_or(Value::Null))
        .collect();
    assert_eq!(json!(normal), theirs["normal"]);
    let status: Vec<String> = lists[1].iter().map(processes::status).collect();
    assert_eq!(json!(status), theirs["status"]);
    let labels: Vec<String> = ["R", "S", "D", "Z", "T", "t", "I", "X", ""]
        .iter()
        .map(|s| processes::state_label(s))
        .collect();
    assert_eq!(json!(labels), theirs["labels"]);
    let can: Vec<Value> = lists[1]
        .iter()
        .map(|r| {
            json!([
                processes::can_signal(r),
                processes::overflow(r)
                    .iter()
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>()
            ])
        })
        .collect();
    assert_eq!(json!(can), theirs["can"]);
    let sigs: Vec<Value> = signals
        .as_array()
        .unwrap()
        .iter()
        .map(|s| processes::signal_write(&s[0], s[1].as_str().unwrap()).unwrap_or(Value::Null))
        .collect();
    assert_eq!(json!(sigs), theirs["signals"]);
    let sorts: Vec<Value> = processes::SORTS
        .iter()
        .map(|(id, label)| json!({"id": id, "label": label}))
        .collect();
    assert_eq!(json!(sorts), theirs["sorts"]);
    let pairs = |l: &[(&str, &str)]| -> Value {
        l.iter()
            .map(|(id, label)| json!({"id": id, "label": label}))
            .collect()
    };
    let values = |l: &[(&str, &str, f64)]| -> Value {
        l.iter().map(|(id, label, v)| json!({"id": id, "label": label, "value": atmos_ui::js::json_number(*v)})).collect()
    };
    let chips = json!([
        pairs(processes::SCOPES),
        pairs(processes::STATES),
        values(processes::MIN_CPU),
        values(processes::MIN_RSS),
        values(processes::CAPS),
    ]);
    assert_eq!(chips, theirs["chips"]);
}
