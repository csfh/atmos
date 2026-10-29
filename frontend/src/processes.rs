//! `services/Processes.js` and `ProcessRow.qml`: the processes Home and Monitor list, filtered,
//! sorted (or as a tree), and capped, what each row says, and the signals and copies its actions
//! send. `tests/processes_parity.rs` holds them to the JavaScript.

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::js;

/// `ROW_CAP`.
pub const ROW_CAP: usize = 80;

/// `sortChips()`.
pub const SORTS: &[(&str, &str)] = &[
    ("cpu", "CPU"),
    ("memory", "Memory"),
    ("pid", "Pid"),
    ("name", "Name"),
    ("threads", "Threads"),
    ("io", "I/O"),
];

/// `scopeChips()`.
pub const SCOPES: &[(&str, &str)] = &[("mine", "Mine"), ("all", "All"), ("kernel", "Kernel")];

/// `stateChips()`.
pub const STATES: &[(&str, &str)] = &[
    ("all", "All states"),
    ("running", "Running"),
    ("sleeping", "Sleeping"),
    ("idle", "Idle"),
    ("disk", "Disk sleep"),
    ("stopped", "Stopped"),
    ("zombie", "Zombie"),
];

/// `minCpuChips()`, `minRssChips()` and `capChips()`: each id, label and value.
pub const MIN_CPU: &[(&str, &str, f64)] = &[
    ("0", "Any CPU", 0.0),
    ("1", "≥1%", 1.0),
    ("5", "≥5%", 5.0),
    ("25", "≥25%", 25.0),
];
pub const MIN_RSS: &[(&str, &str, f64)] = &[
    ("0", "Any RSS", 0.0),
    ("10240", "≥10 MB", 10240.0),
    ("102400", "≥100 MB", 102_400.0),
    ("1048576", "≥1 GB", 1_048_576.0),
];
pub const CAPS: &[(&str, &str, f64)] = &[
    ("40", "40", 40.0),
    ("80", "80", 80.0),
    ("200", "200", 200.0),
];

/// `Number(v)` as a field keeps it: none for one that is not finite, or below 0 when `non_neg`.
fn field(v: Option<&Value>, non_neg: bool) -> Option<f64> {
    let n = js::number(v);
    (n.is_finite() && (!non_neg || n >= 0.0)).then_some(n)
}

fn num(n: Option<f64>) -> Value {
    n.map_or(Value::Null, js::json_number)
}

fn get(row: &Value, k: &str) -> Option<f64> {
    row.get(k).and_then(Value::as_f64)
}

/// `normalize(row)`.
pub fn normalize(row: &Value) -> Option<Value> {
    if !(row.is_object() || row.is_array()) {
        return None;
    }
    let pid = js::number(row.get("pid"));
    if !pid.is_finite() || pid <= 0.0 {
        return None;
    }
    let comm = row
        .get("comm")
        .filter(|v| js::truthy(Some(v)))
        .map_or_else(String::new, js::string)
        .trim()
        .to_owned();
    if comm.is_empty() {
        return None;
    }
    let text = |k: &str| {
        row.get(k)
            .filter(|v| js::truthy(Some(v)))
            .map_or_else(String::new, js::string)
    };
    let state: String = text("state").trim().chars().take(1).collect();
    let mine = match row.get("mine") {
        Some(Value::Bool(b)) => Value::Bool(*b),
        _ => Value::Null,
    };
    Some(json!({
        "pid": js::json_number(pid), "ppid": js::json_number(field(row.get("ppid"), true).unwrap_or(0.0)),
        "comm": comm, "cmdline": text("cmdline"), "uid": num(field(row.get("uid"), false)),
        "rssKb": num(field(row.get("rssKb"), true)), "cpu": num(field(row.get("cpu"), true)), "state": state,
        "threads": num(field(row.get("threads"), true)), "nice": num(field(row.get("nice"), false)),
        "kthread": row.get("kthread") == Some(&Value::Bool(true)), "mine": mine,
        "readBps": num(field(row.get("readBps"), true)), "writeBps": num(field(row.get("writeBps"), true)),
        "depth": js::json_number(field(row.get("depth"), true).unwrap_or(0.0)),
    }))
}

fn haystack(row: &Value) -> String {
    format!(
        "{} {} {} {}",
        js::string(&row["comm"]),
        js::string(&row["cmdline"]),
        js::string(&row["pid"]),
        js::string(&row["state"])
    )
    .to_lowercase()
}

/// `stateGroup(state)`.
pub fn state_group(state: &str) -> &'static str {
    match state {
        "R" => "running",
        "S" => "sleeping",
        "D" => "disk",
        "Z" => "zombie",
        "T" | "t" => "stopped",
        "I" => "idle",
        _ => "other",
    }
}

/// `stateLabel(state)`.
pub fn state_label(state: &str) -> String {
    match state_group(state) {
        "running" => "R".into(),
        "sleeping" => "S".into(),
        "disk" => "D".into(),
        "zombie" => "Z".into(),
        "stopped" => "T".into(),
        "idle" => "I".into(),
        _ if state.is_empty() => "?".into(),
        _ => state.into(),
    }
}

fn in_scope(row: &Value, scope: &str, uid: Option<f64>) -> bool {
    let kthread = row["kthread"] == Value::Bool(true);
    if scope == "kernel" {
        return kthread;
    }
    if kthread {
        return false;
    }
    if scope == "all" {
        return true;
    }
    match &row["mine"] {
        Value::Bool(b) => *b,
        _ => match (uid.filter(|u| u.is_finite()), get(row, "uid")) {
            (Some(u), Some(r)) => r == u,
            _ => true,
        },
    }
}

enum Key {
    Num(f64),
    Text(String),
}

fn sort_key(row: &Value, sort: &str) -> Key {
    let or = |k: &str| get(row, k).unwrap_or(-1.0);
    match sort {
        "memory" => Key::Num(or("rssKb")),
        "pid" => Key::Num(or("pid")),
        "name" => Key::Text(js::string(&row["comm"]).to_lowercase()),
        "threads" => Key::Num(or("threads")),
        "read" => Key::Num(or("readBps")),
        "write" => Key::Num(or("writeBps")),
        "io" => Key::Num(get(row, "readBps").unwrap_or(0.0) + get(row, "writeBps").unwrap_or(0.0)),
        _ => Key::Num(or("cpu")),
    }
}

/// `compareRows(a, b, sort, reverse)`.
fn compare(a: &Value, b: &Value, sort: &str, reverse: bool) -> std::cmp::Ordering {
    let cmp = match (sort_key(a, sort), sort_key(b, sort)) {
        (Key::Text(x), Key::Text(y)) => x.cmp(&y),
        (Key::Num(x), Key::Num(y)) if sort == "pid" => {
            x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Key::Num(x), Key::Num(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
        _ => std::cmp::Ordering::Equal,
    };
    let cmp = if reverse { cmp.reverse() } else { cmp };
    cmp.then_with(|| {
        get(a, "pid")
            .partial_cmp(&get(b, "pid"))
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// `treeOrder(rows, sort, reverse)`: each root, then its children under it, each level sorted.
fn tree(rows: Vec<Value>, sort: &str, reverse: bool) -> Vec<Value> {
    let pids: HashMap<String, usize> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| (js::string(&r["pid"]), i))
        .collect();
    let mut children: HashMap<String, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        let ppid = js::string(&r["ppid"]);
        if js::truthy(r.get("ppid")) && pids.contains_key(&ppid) {
            children.entry(ppid).or_default().push(i);
        } else {
            roots.push(i);
        }
    }
    let order =
        |list: &mut Vec<usize>| list.sort_by(|a, b| compare(&rows[*a], &rows[*b], sort, reverse));
    order(&mut roots);
    let mut out = Vec::new();
    let mut stack: Vec<(usize, usize)> = roots.into_iter().rev().map(|i| (i, 0)).collect();
    while let Some((i, depth)) = stack.pop() {
        let mut row = rows[i].clone();
        row["depth"] = json!(depth);
        let mut kids = children
            .get(&js::string(&rows[i]["pid"]))
            .cloned()
            .unwrap_or_default();
        order(&mut kids);
        stack.extend(kids.into_iter().rev().map(|k| (k, depth + 1)));
        out.push(row);
    }
    out
}

/// What `list` is asked for (`opts`).
#[derive(Debug, Clone, PartialEq)]
pub struct Filter<'a> {
    pub query: &'a str,
    pub sort: &'a str,
    pub scope: &'a str,
    pub state: &'a str,
    pub uid: Option<f64>,
    pub min_cpu: f64,
    pub min_rss_kb: f64,
    pub cap: usize,
    pub reverse: bool,
    pub tree: bool,
}

impl Default for Filter<'_> {
    fn default() -> Self {
        Filter {
            query: "",
            sort: "cpu",
            scope: "mine",
            state: "all",
            uid: None,
            min_cpu: 0.0,
            min_rss_kb: 0.0,
            cap: ROW_CAP,
            reverse: false,
            tree: false,
        }
    }
}

/// `list(rows, query, sort, opts)`.
pub fn list(rows: &[Value], f: &Filter) -> Vec<Value> {
    let q = f.query.trim().to_lowercase();
    let mut out: Vec<Value> = rows
        .iter()
        .filter_map(normalize)
        .filter(|r| in_scope(r, f.scope, f.uid))
        .filter(|r| f.state == "all" || state_group(&js::string(&r["state"])) == f.state)
        .filter(|r| f.min_cpu <= 0.0 || get(r, "cpu").is_some_and(|c| c >= f.min_cpu))
        .filter(|r| f.min_rss_kb <= 0.0 || get(r, "rssKb").is_some_and(|c| c >= f.min_rss_kb))
        .filter(|r| q.is_empty() || haystack(r).contains(&q))
        .collect();
    if f.tree {
        out = tree(out, f.sort, f.reverse);
    } else {
        out.sort_by(|a, b| compare(a, b, f.sort, f.reverse));
    }
    out.truncate(if f.cap == 0 { ROW_CAP } else { f.cap });
    out
}

/// `canSignal(row)`: not a kernel thread, nor another user's.
pub fn can_signal(row: &Value) -> bool {
    row["kthread"] != Value::Bool(true) && row["mine"] != Value::Bool(false)
}

/// `overflowActions(row)`: each action's id and label.
pub fn overflow(row: &Value) -> Vec<(&'static str, &'static str)> {
    let all = [
        ("kill", "Force quit"),
        ("copy", "Copy pid"),
        ("copyCmd", "Copy command"),
    ];
    all.into_iter()
        .filter(|(id, _)| *id != "kill" || can_signal(row))
        .collect()
}

/// `ProcessRow.statusText`: the pid, state, CPU, memory, threads and I/O.
pub fn status(row: &Value) -> String {
    let mut bits = vec![
        js::string(&row["pid"]),
        state_label(&js::string(&row["state"])),
    ];
    bits.push(crate::stats::format_percent(get(row, "cpu")));
    if let Some(kb) = get(row, "rssKb") {
        bits.push(crate::hardware::format_bytes(kb * 1024.0));
    }
    if let Some(t) = get(row, "threads") {
        bits.push(format!("{} thr", js::string(&js::json_number(t))));
    }
    let (r, w) = (
        crate::stats::format_bps(get(row, "readBps")),
        crate::stats::format_bps(get(row, "writeBps")),
    );
    if !r.is_empty() || !w.is_empty() {
        let or = |s: String| if s.is_empty() { "—".to_owned() } else { s };
        bits.push(format!("↓ {}  ↑ {}", or(r), or(w)));
    }
    bits.retain(|b| !b.is_empty());
    bits.join("   ")
}

/// `parsePid` and `signalName`: the typed `process.signal` write, or none.
pub fn signal_write(pid: &Value, signal: &str) -> Option<Value> {
    let s = js::string(pid);
    let s = s.trim();
    let ok = s.chars().next().is_some_and(|c| ('1'..='9').contains(&c))
        && s.chars().all(|c| c.is_ascii_digit());
    let pid: f64 = if ok { s.parse().unwrap_or(0.0) } else { 0.0 };
    let sig = match signal.trim().to_uppercase().as_str() {
        "TERM" | "SIGTERM" | "15" => "TERM",
        "KILL" | "SIGKILL" | "9" => "KILL",
        _ => "",
    };
    (pid >= 2.0 && pid.is_finite() && !sig.is_empty()).then(|| {
        crate::protocol::write(
            "process.signal",
            &json!({ "pid": js::json_number(pid), "signal": sig }),
        )
    })
}

/// The confirm a signal asks: its title, message and action.
pub fn signal_confirm(row: &Value, kill: bool) -> (&'static str, String, &'static str) {
    let (comm, pid) = (js::string(&row["comm"]), js::string(&row["pid"]));
    if kill {
        (
            "Force quit this process",
            format!("Send SIGKILL to {comm} ({pid}). It will not get a chance to exit cleanly."),
            "Force quit",
        )
    } else {
        (
            "End this process",
            format!("Send SIGTERM to {comm} ({pid})."),
            "End",
        )
    }
}
