//! `services/LiveStats.js`: the live samples the daemon pushes, parsed, kept as the last minute
//! of rows with the rates between them, and read back as series and the texts the Monitor and
//! Home pages show. A row is the JSON object `pushSample` makes, so `tests/stats_parity.rs` holds
//! each function to its JavaScript by comparing values.

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::js;

pub use crate::sample::parse;

/// `SAMPLE_CAP`: rows kept.
pub const SAMPLE_CAP: usize = 60;
/// `PROCESS_HISTORY`: the newest rows that keep their process lists.
const PROCESS_HISTORY: usize = 2;

/// `finiteNumber`.
pub fn finite(v: Option<&Value>) -> Option<f64> {
    match v {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if s.is_empty() => None,
        Some(v) => Some(js::number(Some(v))).filter(|n| n.is_finite()),
    }
}

/// `nonNeg`.
pub fn non_neg(v: Option<&Value>) -> Option<f64> {
    finite(v).filter(|n| *n >= 0.0)
}

/// `parseTemp`: none for a reading at or below 0.
pub fn temp(v: Option<&Value>) -> Option<f64> {
    finite(v).filter(|n| *n > 0.0)
}

pub(crate) fn num(n: Option<f64>) -> Value {
    n.map_or(Value::Null, js::json_number)
}

/// `String(v || "")`.
pub(crate) fn text(v: Option<&Value>) -> String {
    v.filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

pub(crate) fn list(v: Option<&Value>) -> &[Value] {
    v.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

pub(crate) fn is_object(v: &Value) -> bool {
    v.is_object() || v.is_array()
}

pub(crate) fn mine(v: Option<&Value>) -> Value {
    match v {
        Some(Value::Bool(b)) => Value::Bool(*b),
        _ => Value::Null,
    }
}

/// `countersPercent`.
fn counters(prev: (Option<f64>, Option<f64>), next: (Option<f64>, Option<f64>)) -> Option<f64> {
    let (pi, pt, ni, nt) = (prev.0?, prev.1?, next.0?, next.1?);
    let (d_idle, d_total) = (ni - pi, nt - pt);
    if d_total.is_nan() || d_total <= 0.0 || d_idle < 0.0 {
        return None;
    }
    Some((d_total - d_idle).max(0.0) / d_total * 100.0)
}

fn f(row: &Value, k: &str) -> Option<f64> {
    row.get(k).and_then(Value::as_f64)
}

fn pair(row: &Value, idle: &str, total: &str) -> (Option<f64>, Option<f64>) {
    (f(row, idle), f(row, total))
}

fn share(used: Option<f64>, total: Option<f64>) -> Option<f64> {
    let (used, total) = (used?, total?);
    (total > 0.0).then(|| (used / total * 100.0).max(0.0))
}

/// `byteRate`.
fn byte_rate(prev: Option<f64>, next: Option<f64>, dt: f64) -> Option<f64> {
    let d = next? - prev?;
    (dt > 0.0 && d >= 0.0).then(|| d / (dt / 1000.0))
}

fn by<'a>(prev: Option<&'a Value>, list_key: &str, key: &str) -> HashMap<String, &'a Value> {
    let rows = prev.map_or(&[][..], |p| list(p.get(list_key)));
    let mut out = HashMap::new();
    for r in rows {
        if js::truthy(Some(r)) && (key == "id" || js::truthy(r.get(key))) {
            out.insert(js::string(r.get(key).unwrap_or(&Value::Null)), r);
        }
    }
    out
}

fn decorate_processes(prev: Option<&Value>, sample: &Value, dt: f64) -> Value {
    let last = by(prev, "processes", "pid");
    let clk = f(sample, "clkTck").filter(|c| *c > 0.0).unwrap_or(100.0);
    Value::Array(
        list(sample.get("processes"))
            .iter()
            .filter(|r| js::truthy(Some(r)))
            .map(|r| {
                let before = last.get(&js::string(&r["pid"])).copied();
                let cpu = before.and_then(|b| {
                    let d = f(r, "ticks")? - f(b, "ticks")?;
                    (dt > 0.0 && clk > 0.0 && d >= 0.0).then(|| 100.0 * d / (clk * (dt / 1000.0)))
                });
                let rate = |k: &str, out: &str| {
                    let _ = out;
                    num(before.and_then(|b| byte_rate(f(b, k), f(r, k), dt)))
                };
                let mut o = r.as_object().cloned().unwrap_or_default();
                o.insert("cpu".into(), num(cpu));
                o.insert(
                    "kthread".into(),
                    Value::Bool(r.get("kthread") == Some(&Value::Bool(true))),
                );
                o.insert("mine".into(), mine(r.get("mine")));
                o.insert("readBps".into(), rate("readBytes", "readBps"));
                o.insert("writeBps".into(), rate("writeBytes", "writeBps"));
                o.insert("depth".into(), json!(0));
                Value::Object(o)
            })
            .collect(),
    )
}

fn decorate(
    prev: Option<&Value>,
    sample: &Value,
    list_key: &str,
    key: &str,
    set: impl Fn(&mut Map<String, Value>, &Value, Option<&Value>),
) -> Value {
    let last = by(prev, list_key, key);
    Value::Array(
        list(sample.get(list_key))
            .iter()
            .filter(|r| js::truthy(Some(r)))
            .map(|r| {
                let before = last
                    .get(&js::string(r.get(key).unwrap_or(&Value::Null)))
                    .copied();
                let mut o = r.as_object().cloned().unwrap_or_default();
                set(&mut o, r, before);
                Value::Object(o)
            })
            .collect(),
    )
}

/// `netRate`: received and sent per second.
fn net_rate(
    prev: (Option<f64>, Option<f64>),
    next: (Option<f64>, Option<f64>),
    dt: f64,
) -> (Option<f64>, Option<f64>) {
    let rates = || {
        let (d_rx, d_tx) = (next.0? - prev.0?, next.1? - prev.1?);
        (dt > 0.0 && d_rx >= 0.0 && d_tx >= 0.0)
            .then(|| (d_rx / (dt / 1000.0), d_tx / (dt / 1000.0)))
    };
    rates().map_or((None, None), |(r, t)| (Some(r), Some(t)))
}

/// `pushSample(history, sample, now)`: the rows with `sample` added at `now`, and the rates since
/// the row before it.
pub fn push(history: &[Value], sample: &Value, now: f64) -> Vec<Value> {
    let start = (history.len() + 1).saturating_sub(SAMPLE_CAP);
    let mut rows: Vec<Value> = history[start.min(history.len())..].to_vec();
    let prev = rows.last().cloned();
    let prev = prev.as_ref();
    let at = if now.is_finite() { now } else { 0.0 };
    let dt = prev.map_or(0.0, |p| at - f(p, "at").unwrap_or(f64::NAN));
    let mut o = sample.as_object().cloned().unwrap_or_default();
    o.insert("at".into(), js::json_number(at));
    o.insert(
        "cpus".into(),
        decorate(prev, sample, "cpus", "id", |o, r, before| {
            o.insert(
                "cpu".into(),
                num(before
                    .and_then(|b| counters(pair(b, "idle", "total"), pair(r, "idle", "total")))),
            );
        }),
    );
    o.insert(
        "ifaces".into(),
        decorate(prev, sample, "ifaces", "name", |o, r, before| {
            let (rx, tx) = before.map_or((None, None), |b| {
                net_rate(pair(b, "rx", "tx"), pair(r, "rx", "tx"), dt)
            });
            o.insert("rxBps".into(), num(rx));
            o.insert("txBps".into(), num(tx));
        }),
    );
    o.insert(
        "disks".into(),
        decorate(prev, sample, "disks", "name", |o, r, before| {
            let rate = |k: &str| {
                num(before
                    .and_then(|b| byte_rate(f(b, k), f(r, k), dt))
                    .map(|n| n * 512.0))
            };
            o.insert("readBps".into(), rate("readSectors"));
            o.insert("writeBps".into(), rate("writeSectors"));
        }),
    );
    let cpu = prev.and_then(|p| {
        counters(
            pair(p, "cpuIdle", "cpuTotal"),
            pair(sample, "cpuIdle", "cpuTotal"),
        )
    });
    let (rx, tx) = prev.map_or((None, None), |p| {
        net_rate(
            pair(p, "netRx", "netTx"),
            pair(sample, "netRx", "netTx"),
            dt,
        )
    });
    o.insert("cpu".into(), num(cpu));
    o.insert(
        "mem".into(),
        num(share(f(sample, "memUsed"), f(sample, "memTotal"))),
    );
    o.insert(
        "swap".into(),
        num(share(f(sample, "swapUsed"), f(sample, "swapTotal"))),
    );
    o.insert("rxBps".into(), num(rx));
    o.insert("txBps".into(), num(tx));
    o.insert("processes".into(), decorate_processes(prev, sample, dt));
    rows.push(Value::Object(o));
    while rows.len() > SAMPLE_CAP {
        rows.remove(0);
    }
    let keep = rows.len().saturating_sub(PROCESS_HISTORY);
    for row in &mut rows[..keep] {
        if row
            .get("processes")
            .and_then(Value::as_array)
            .is_some_and(|p| !p.is_empty())
        {
            row["processes"] = json!([]);
        }
    }
    rows
}

/// `series(history, key)`: the key's numbers, oldest first, skipping the rows without one.
pub fn series(history: &[Value], key: &str) -> Vec<f64> {
    history.iter().filter_map(|r| finite(r.get(key))).collect()
}

/// `namedSeries`: a value of the item of a row's list whose `match_key` is `needle`.
fn named_series(
    history: &[Value],
    list_key: &str,
    match_key: &str,
    needle: &str,
    value_key: &str,
) -> Vec<f64> {
    let mut hint: Option<usize> = None;
    let mut out = Vec::new();
    for r in history {
        let rows = list(r.get(list_key));
        let is = |x: &Value| js::string(x.get(match_key).unwrap_or(&Value::Null)) == needle;
        let mut row = hint
            .and_then(|h| rows.get(h))
            .filter(|x| js::truthy(Some(x)) && is(x));
        if row.is_none()
            && let Some(j) = rows.iter().position(|x| js::truthy(Some(x)) && is(x))
        {
            hint = Some(j);
            row = Some(&rows[j]);
        }
        if let Some(n) = row.and_then(|x| finite(x.get(value_key))) {
            out.push(n);
        }
    }
    out
}

/// `coreSeries(history, id)`.
pub fn core_series(history: &[Value], id: &Value) -> Vec<f64> {
    named_series(history, "cpus", "id", &js::string(id), "cpu")
}

/// `diskSeries(history, name, key)`.
pub fn disk_series(history: &[Value], name: &str, key: &str) -> Vec<f64> {
    named_series(history, "disks", "name", name, key)
}

/// `gpuMatches(live, key)`: a key is a card or PCI id, or an inventory GPU.
fn gpu_matches(live: &Value, key: &Value) -> bool {
    let s = |v: &Value, k: &str| text(v.get(k));
    if let Some(needle) = key.as_str() {
        return s(live, "card") == needle || s(live, "pciId") == needle.to_lowercase();
    }
    if key.is_null() {
        return false;
    }
    let pci = s(key, "pciId").to_lowercase();
    let card = s(live, "card");
    (!pci.is_empty() && pci == s(live, "pciId"))
        || (!s(key, "card").is_empty() && !card.is_empty() && s(key, "card") == card)
        || (!s(key, "name").is_empty() && !card.is_empty() && s(key, "name") == card)
}

/// `findGpu(sample, key)`.
pub fn find_gpu<'a>(sample: &'a Value, key: &Value) -> Option<&'a Value> {
    list(sample.get("gpus"))
        .iter()
        .find(|g| js::truthy(Some(g)) && gpu_matches(g, key))
}

/// `gpuOwnTemp(sample, key)`.
pub fn gpu_own_temp(sample: &Value, key: &Value) -> Option<f64> {
    find_gpu(sample, key).and_then(|g| temp(g.get("temp")))
}

/// `gpuTempAt(sample, key)`: its own sensor, or the package's for an integrated part.
pub fn gpu_temp_at(sample: &Value, key: &Value) -> Option<f64> {
    let g = find_gpu(sample, key);
    if let Some(own) = g.and_then(|g| temp(g.get("temp"))) {
        return Some(own);
    }
    let integrated = match g {
        Some(g) => g.get("integrated") == Some(&Value::Bool(true)),
        None => {
            key.get("integrated") == Some(&Value::Bool(true)) || text(key.get("driver")) == "i915"
        }
    };
    if integrated {
        temp(sample.get("cpuTemp"))
    } else {
        None
    }
}

/// `gpuSeries(history, key)`.
pub fn gpu_series(history: &[Value], key: &Value) -> Vec<f64> {
    history.iter().filter_map(|r| gpu_temp_at(r, key)).collect()
}

/// `gpuBusyAt(sample, key)`.
pub fn gpu_busy_at(sample: &Value, key: &Value) -> Option<f64> {
    find_gpu(sample, key).and_then(|g| non_neg(g.get("busy")))
}

/// `gpuBusySeries(history, key)`.
pub fn gpu_busy_series(history: &[Value], key: &Value) -> Vec<f64> {
    history.iter().filter_map(|r| gpu_busy_at(r, key)).collect()
}

/// `ifaceSeries(history, name, key)`.
pub fn iface_series(history: &[Value], name: &str, key: &str) -> Vec<f64> {
    named_series(history, "ifaces", "name", name, key)
}

/// `sensorSeries(history, id)`.
pub fn sensor_series(history: &[Value], id: &str) -> Vec<f64> {
    named_series(history, "sensors", "id", id, "value")
}

/// `formatRpm`.
pub fn format_rpm(v: Option<f64>) -> String {
    v.map_or_else(String::new, |v| format!("{} RPM", js::round(v)))
}

/// `gpuRows(hardwareGpus, sample)`: the inventory's GPUs, else the live ones.
pub fn gpu_rows(hardware: Option<&Value>, sample: Option<&Value>) -> Vec<Value> {
    match list(hardware) {
        hw if !hw.is_empty() => hw.to_vec(),
        _ => sample.map_or(Vec::new(), |s| list(s.get("gpus")).to_vec()),
    }
}

/// `toFixed(digits)`, rounding a tie up as JavaScript does.
pub fn to_fixed(v: f64, digits: i32) -> String {
    let scale = 10f64.powi(digits);
    let scaled = v * scale;
    let v = if (scaled - scaled.floor() - 0.5).abs() == 0.0 {
        (scaled.floor() + 1.0) / scale
    } else {
        v
    };
    format!("{:.*}", digits as usize, v)
}

/// `formatPercent`.
pub fn format_percent(v: Option<f64>) -> String {
    let Some(v) = v else { return String::new() };
    let v = v.max(0.0);
    if v < 10.0 {
        format!("{}%", to_fixed(js::round(v * 10.0) / 10.0, 1))
    } else {
        format!("{}%", js::round(v))
    }
}

/// `formatTemp`.
pub fn format_temp(v: Option<f64>) -> String {
    v.filter(|n| *n > 0.0)
        .map_or_else(String::new, |n| format!("{} °C", js::round(n)))
}

/// `formatBps`.
pub fn format_bps(v: Option<f64>) -> String {
    let Some(mut v) = v else { return String::new() };
    v = v.max(0.0);
    let units = ["B/s", "KB/s", "MB/s", "GB/s"];
    let mut i = 0;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", js::round(v), units[i])
    } else {
        format!("{} {}", to_fixed(js::round(v * 10.0) / 10.0, 1), units[i])
    }
}

/// `formatNet(sample)`.
pub fn format_net(sample: Option<&Value>) -> String {
    let Some(s) = sample else {
        return String::new();
    };
    let (rx, tx) = (
        format_bps(finite(s.get("rxBps"))),
        format_bps(finite(s.get("txBps"))),
    );
    if rx.is_empty() || tx.is_empty() {
        String::new()
    } else {
        format!("↓ {rx}  ↑ {tx}")
    }
}

/// `memBytes(kb)`.
pub fn mem_bytes(kb: Option<&Value>) -> Option<f64> {
    non_neg(kb).map(|n| n * 1024.0)
}

/// `formatLoadLine(sample)`.
pub fn format_load_line(sample: Option<&Value>) -> String {
    let Some(s) = sample else {
        return String::new();
    };
    let parts: Vec<Option<String>> = ["load1", "load5", "load15"]
        .iter()
        .map(|k| finite(s.get(*k)).map(|n| to_fixed(n, 2)))
        .collect();
    match parts.as_slice() {
        [Some(a), Some(b), Some(c)] => format!("{a}  {b}  {c}"),
        _ => String::new(),
    }
}

/// `formatMhz`.
pub fn format_mhz(v: Option<f64>) -> String {
    match v {
        None => String::new(),
        Some(v) if v >= 1000.0 => {
            format!("{} GHz", to_fixed(js::round(v / 1000.0 * 100.0) / 100.0, 2))
        }
        Some(v) => format!("{} MHz", js::round(v)),
    }
}

/// `formatPsi`.
pub fn format_psi(v: Option<f64>) -> String {
    match v {
        None => String::new(),
        Some(v) if v < 0.05 => "0%".into(),
        Some(v) if v < 10.0 => format!("{}%", to_fixed(js::round(v * 10.0) / 10.0, 1)),
        Some(v) => format!("{}%", js::round(v)),
    }
}

/// `memParts(sample)`: used, buffers, cached and free, in kB.
pub fn mem_parts(sample: Option<&Value>) -> Vec<(&'static str, &'static str, f64)> {
    let Some(s) = sample else { return Vec::new() };
    let Some(total) = f(s, "memTotal").filter(|t| *t > 0.0) else {
        return Vec::new();
    };
    let or0 = |k: &str| f(s, k).unwrap_or(0.0);
    let (free, buffers, cached) = (or0("memFree"), or0("memBuffers"), or0("memCached"));
    let mut used = total - free - buffers - cached;
    if used < 0.0 {
        used = or0("memUsed");
    }
    vec![
        ("used", "Used", used.max(0.0)),
        ("buffers", "Buffers", buffers),
        ("cached", "Cached", cached),
        ("free", "Free", free),
    ]
}

/// `corePercents(sample)`.
pub fn core_percents(sample: Option<&Value>) -> Vec<f64> {
    sample.map_or(Vec::new(), |s| {
        list(s.get("cpus"))
            .iter()
            .map(|c| c.get("cpu").and_then(Value::as_f64).unwrap_or(0.0))
            .collect()
    })
}

/// `sparklinePoints(values, width, height, pad)`: each value's point, the lowest at the bottom.
pub fn sparkline_points(values: &[f64], width: f32, height: f32, pad: f32) -> Vec<(f32, f32)> {
    let pad = if pad.is_finite() && pad >= 0.0 {
        pad
    } else {
        0.0
    };
    let (w, h) = (width - pad * 2.0, height - pad * 2.0);
    if !(width > 0.0 && height > 0.0 && w > 0.0 && h > 0.0) || values.len() < 2 {
        return Vec::new();
    }
    let (min, max) = values
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(*v), hi.max(*v))
        });
    let span = max - min;
    let last = (values.len() - 1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = pad + w * i as f32 / last;
            let y = if span == 0.0 {
                pad + h / 2.0
            } else {
                pad + h - ((v - min) / span) as f32 * h
            };
            (x, y)
        })
        .collect()
}
