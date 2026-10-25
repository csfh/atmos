//! The Rust Sentinel is the QML one until the QML one goes: the same frames through
//! `services/LiveStats.js` and `services/Sentinel.js` under Node, and through `live.rs` and
//! `sentinel.rs`, must give the same rates, rows, lists and counts after every frame.
//!
//! The frames are made to hit where the two languages differ: a number as text, a missing or
//! zero pid, a tick counter that goes down, the same sample twice, a size that is exactly a tie
//! for `toFixed(1)`, a process with no start time. Without `node` the test says so and stops.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use atmos_ui::live::Feed;
use atmos_ui::sentinel::{Counts, Filter, Thresholds, Tracked, list, rows};
use serde_json::{Value, json};

/// What the page does with a frame, in JavaScript: `LiveStatsStore.adoptFrame` (a frame not after
/// the last is left out), `pushSample`, then `track` on the new latest, then `rows`, `list` for
/// each chip and each query, and the chips' counts.
const NODE: &str = r#"
const fs = require("fs");
const root = process.env.SERVICES_DIR;
const LS = require(root + "/LiveStats.js");
const S = require(root + "/Sentinel.js");
const input = JSON.parse(fs.readFileSync(0, "utf8"));
const kinds = ["flagged", "cpu", "memory", "port"];
const clean = (r) => ({
  id: r.id, kind: r.kind, pid: r.pid === undefined ? null : r.pid, comm: r.comm,
  cmdline: r.cmdline, detail: r.detail, ratio: r.ratio, flagged: r.flagged,
  search: r.search === undefined ? null : r.search,
});
const out = input.cases.map((c) => {
  let history = [], lastAt = 0;
  const th = S.thresholds(c.stored);
  let tracked = S.track(null, null, null);
  return c.frames.map((f) => {
    const at = Number(f.at);
    let taken = false;
    if (at > lastAt) {
      const parsed = LS.parse(JSON.stringify(f.result));
      if (parsed) {
        lastAt = at;
        history = LS.pushSample(history, parsed, at);
        taken = true;
      }
    }
    const latest = LS.latest(history);
    if (taken) tracked = S.track(tracked, latest, th);
    const all = S.rows(latest, tracked, th);
    const lists = {};
    kinds.forEach((k) => c.queries.forEach((q) => { lists[k + "|" + q] = S.list(all, k, q).map(clean); }));
    const chips = {};
    S.kindChips(all).forEach((chip) => { chips[chip.id] = chip.count; });
    return {
      taken,
      cpus: latest ? latest.processes.map((p) => [p.pid, p.cpu]) : [],
      tracked: tracked.over,
      th,
      all: all.map(clean),
      lists,
      chips,
    };
  });
});
process.stdout.write(JSON.stringify(out));
"#;

/// The same, in Rust, as JSON of the same shape.
fn rust_side(case: &Value) -> Value {
    let stored = case.get("stored");
    let th = Thresholds::from_stored(stored);
    let queries: Vec<&str> = case["queries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| q.as_str().unwrap())
        .collect();
    let mut feed = Feed::default();
    let mut tracked = Tracked::default();
    let mut steps = Vec::new();
    for f in case["frames"].as_array().unwrap() {
        let at = f["at"].as_f64().unwrap();
        let taken = feed.adopt(at, &f["result"]);
        if taken {
            tracked = tracked.track(feed.latest(), &th);
        }
        let latest = feed.latest();
        let all = rows(latest, &tracked, &th);
        let mut lists = serde_json::Map::new();
        for filter in Filter::ALL {
            for q in &queries {
                let shown: Vec<Value> = list(&all, filter, q).iter().map(|r| r.to_json()).collect();
                lists.insert(format!("{}|{q}", filter.id()), Value::Array(shown));
            }
        }
        let counts = Counts::of(&all);
        let chips: serde_json::Map<String, Value> = Filter::ALL
            .iter()
            .map(|f| (f.id().to_owned(), json!(counts.of_filter(*f))))
            .collect();
        steps.push(json!({
            "taken": taken,
            "cpus": latest.map_or(vec![], |l| l.processes.iter().map(|p| json!([p.pid, p.cpu])).collect()),
            "tracked": tracked.to_json()["over"],
            "th": { "cpuPercent": th.cpu_percent, "memoryPercent": th.memory_percent, "portAgeDays": th.port_age_days },
            "all": all.iter().map(|r| r.to_json()).collect::<Vec<_>>(),
            "lists": lists,
            "chips": chips,
        }));
    }
    Value::Array(steps)
}

/// Every number as an `f64`, since JavaScript has one kind and `serde_json` keeps three.
fn normal(v: Value) -> Value {
    match v {
        Value::Number(n) => json!(n.as_f64()),
        Value::Array(a) => Value::Array(a.into_iter().map(normal).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, normal(v))).collect()),
        other => other,
    }
}

/// A small deterministic generator, so a failure is the same failure next time.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

/// A number the way a frame might carry it: a number, mostly, and now and then as text.
fn maybe_text(rng: &mut Lcg, n: u64) -> Value {
    if rng.chance(6) {
        json!(n.to_string())
    } else {
        json!(n)
    }
}

fn process(rng: &mut Lcg, pid: u64, step: u64) -> Value {
    let mut p = serde_json::Map::new();
    p.insert("pid".into(), maybe_text(rng, pid));
    let comm = rng.pick(&["firefox", "node", " spaced ", "", "  ", "Ünï", "a:b", "hog"]);
    p.insert("comm".into(), json!(comm));
    p.insert(
        "cmdline".into(),
        json!(rng.pick(&["", "/usr/bin/x --flag", "node server.js", "LISTEN me", "p1"])),
    );
    // Ticks mostly grow with each frame, and now and then go back.
    let ticks = if rng.chance(8) {
        rng.below(500)
    } else {
        pid * 10 + step * rng.below(900)
    };
    p.insert(
        "ticks".into(),
        if rng.chance(5) {
            Value::Null
        } else {
            maybe_text(rng, ticks)
        },
    );
    let rss = *rng.pick(&[
        0, 1, 1000, 1280, 1536, 1792, 3840, 99_999, 300_000, 524_288, 1_310_720, 2_000_000,
    ]);
    p.insert(
        "rssKb".into(),
        if rng.chance(10) {
            Value::Null
        } else {
            maybe_text(rng, rss)
        },
    );
    p.insert(
        "startTicks".into(),
        if rng.chance(15) {
            Value::Null
        } else {
            json!(rng.below(80_000_000))
        },
    );
    p.insert("kthread".into(), json!(rng.chance(8)));
    p.insert(
        "mine".into(),
        match rng.below(4) {
            0 => json!(false),
            1 => Value::Null,
            _ => json!(true),
        },
    );
    p.insert("uid".into(), json!(rng.pick(&[0, 1000, 1001])));
    Value::Object(p)
}

fn socket(rng: &mut Lcg, pids: &[u64]) -> Value {
    let mut s = serde_json::Map::new();
    s.insert(
        "state".into(),
        json!(rng.pick(&["LISTEN", "LISTEN", "LISTEN", "ESTABLISHED", ""])),
    );
    s.insert(
        "addr".into(),
        match rng.below(7) {
            0 => json!("0.0.0.0"),
            1 => json!("::"),
            2 => json!("127.0.0.1"),
            3 => json!("::1"),
            4 => json!("192.168.1.5"),
            5 => Value::Null,
            _ => json!(""),
        },
    );
    s.insert(
        "port".into(),
        if rng.chance(5) {
            Value::Null
        } else {
            json!(rng.pick(&[22, 80, 80, 443, 3000, 8080, 65535]))
        },
    );
    s.insert("uid".into(), json!(rng.pick(&[0, 1000, 1001])));
    s.insert(
        "pid".into(),
        match rng.below(5) {
            0 => Value::Null,
            1 => json!(0),
            _ => json!(rng.pick(pids)),
        },
    );
    Value::Object(s)
}

fn frame(rng: &mut Lcg, step: u64, pids: &[u64]) -> Value {
    let mut procs: Vec<Value> = Vec::new();
    for pid in pids {
        if rng.chance(85) {
            procs.push(process(rng, *pid, step));
        }
    }
    let mut socks: Vec<Value> = Vec::new();
    for _ in 0..rng.below(8) {
        socks.push(socket(rng, pids));
    }
    json!({
        "clkTck": rng.pick(&[100, 100, 250, 0]),
        "memTotal": rng.pick(&[0, 20_000, 1_000_000, 16_000_000, 4_194_304]),
        "uptimeSec": rng.pick(&[0, 100, 900_000, 100_000_000]),
        "processes": procs,
        "tcpSockets": socks,
    })
}

fn stored(rng: &mut Lcg) -> Value {
    let mut s = serde_json::Map::new();
    for key in [
        "sentinelCpuPercent",
        "sentinelMemoryPercent",
        "sentinelPortAgeDays",
    ] {
        let v = match rng.below(9) {
            0 => continue,
            1 => Value::Null,
            2 => json!(""),
            3 => json!("abc"),
            4 => json!(-5),
            5 => json!(1000),
            6 => json!(55.5),
            7 => json!("10"),
            // What each limit is usually set to, so that a limit is crossed as often as not.
            _ => match key {
                "sentinelCpuPercent" => json!(*rng.pick(&[50, 60, 80, 100])),
                "sentinelMemoryPercent" => json!(*rng.pick(&[5, 6, 10, 20, 50])),
                _ => json!(*rng.pick(&[1, 7, 30])),
            },
        };
        s.insert(key.into(), v);
    }
    Value::Object(s)
}

fn random_case(rng: &mut Lcg) -> Value {
    let pids: Vec<u64> = vec![0, 1, 2, 3, 50, 100, 101, 200, 201, 4_194_304];
    let mut at = 1000 + rng.below(5000);
    let frames: Vec<Value> = (0..6)
        .map(|step| {
            // Mostly forward by about two seconds; now and then the same time, or earlier.
            at = match rng.below(10) {
                0 => at,
                1 => at.saturating_sub(500),
                _ => at + 1000 + rng.below(2500),
            };
            json!({ "at": at, "result": frame(rng, step, &pids) })
        })
        .collect();
    json!({
        "stored": stored(rng),
        "queries": ["", "p1", "port", ":80", "NODE", "listen", " "],
        "frames": frames,
    })
}

/// A process over its limit for four samples in a row, listeners old and young and ownerless:
/// what the page exists for, kept as it is for a reader.
fn steady_case() -> Value {
    let proc = |ticks: u64| {
        json!({"pid": 77, "comm": "hog", "cmdline": "hog --spin", "ticks": ticks,
        "rssKb": 1280, "startTicks": 100, "kthread": false, "mine": true, "uid": 1000})
    };
    let socks = json!([
        {"state": "LISTEN", "addr": "0.0.0.0", "port": 8080, "uid": 1000, "pid": 77},
        {"state": "LISTEN", "addr": "127.0.0.1", "port": 8081, "uid": 1000, "pid": 77},
        {"state": "LISTEN", "addr": "::", "port": 8082, "uid": 1000, "pid": 77},
        {"state": "LISTEN", "addr": "::", "port": 8083, "uid": 1000, "pid": 77},
        {"state": "LISTEN", "addr": "::1", "port": 53, "uid": 0, "pid": null},
    ]);
    let frames: Vec<Value> = (0..5)
        .map(|i| {
            json!({ "at": 2000 * (i + 1), "result": {
                "clkTck": 100, "memTotal": 1_000_000, "uptimeSec": 100_000, "processes": [proc(i * 200)],
                "tcpSockets": socks } })
        })
        .collect();
    json!({
        "stored": {"sentinelCpuPercent": 50, "sentinelMemoryPercent": 5, "sentinelPortAgeDays": 1},
        "queries": ["", "hog", ":8083", "unknown"],
        "frames": frames,
    })
}

#[test]
fn the_rust_sentinel_gives_what_the_qml_one_gives_on_the_same_frames() {
    let services = Path::new(env!("CARGO_MANIFEST_DIR")).join("../services");
    let mut rng = Lcg(0x5EED_0001);
    let mut cases = vec![steady_case()];
    cases.extend((0..120).map(|_| random_case(&mut rng)));
    let input = json!({ "cases": cases });

    let mut node = match Command::new("node")
        .args(["-e", NODE])
        .env("SERVICES_DIR", &services)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!("skipped: no node to run the QML logic ({err})");
            return;
        }
    };
    node.stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = node.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let js: Vec<Value> = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(js.len(), cases.len());

    // What the frames reached, so that a generator that stops reaching the edges fails here.
    let mut seen = std::collections::BTreeMap::<&str, usize>::new();
    let mut compared = 0;
    for (n, (case, theirs)) in cases.iter().zip(js).enumerate() {
        let ours = rust_side(case);
        let (ours, theirs) = (ours.as_array().unwrap(), theirs.as_array().unwrap());
        for (step, (a, b)) in ours.iter().zip(theirs).enumerate() {
            for key in ["taken", "cpus", "tracked", "th", "all", "chips", "lists"] {
                assert_eq!(
                    normal(a[key].clone()),
                    normal(b[key].clone()),
                    "case {n} frame {step}: {key}\nstored {}\nframes {}",
                    case["stored"],
                    case["frames"]
                );
            }
            compared += 1;
            *seen
                .entry(if b["taken"] == true {
                    "taken"
                } else {
                    "left out"
                })
                .or_default() += 1;
            // A process with several old ports is one row only in what is listed.
            for shown in b["lists"].as_object().unwrap().values() {
                for row in shown.as_array().unwrap() {
                    if row["id"].as_str().unwrap().starts_with("ports:") {
                        *seen.entry("collapsed ports").or_default() += 1;
                    }
                }
            }
            for row in b["all"].as_array().unwrap() {
                let (id, detail) = (row["id"].as_str().unwrap(), row["detail"].as_str().unwrap());
                let flagged = row["flagged"] == true;
                let note = |k: &'static str, seen: &mut std::collections::BTreeMap<&str, usize>| {
                    *seen.entry(k).or_default() += 1
                };
                match id.split(':').next().unwrap() {
                    "cpu" => note("cpu finding", &mut seen),
                    "memory" => note("memory finding", &mut seen),
                    "port" if flagged => note("old port", &mut seen),
                    "port" => note("young or unaged port", &mut seen),
                    _ => {}
                }
                if detail.contains("owner unknown") {
                    note("ownerless port", &mut seen);
                }
                if detail.ends_with(" MB, 6% of memory")
                    || detail.contains(".3 MB")
                    || detail.contains(".3 GB")
                {
                    note("a size that is an exact tie", &mut seen);
                }
            }
        }
    }
    eprintln!("{compared} frames compared; reached {seen:?}");
    assert!(compared > 500, "{compared} frames compared");
    for (what, least) in [
        ("taken", 400),
        ("left out", 20),
        ("cpu finding", 10),
        ("memory finding", 40),
        ("old port", 100),
        ("young or unaged port", 100),
        ("collapsed ports", 5),
        ("ownerless port", 50),
        ("a size that is an exact tie", 3),
    ] {
        assert!(
            seen.get(what).copied().unwrap_or(0) >= least,
            "the frames reached {what} only {} times, of at least {least}: {seen:?}",
            seen.get(what).copied().unwrap_or(0)
        );
    }
}
