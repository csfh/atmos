//! `LiveStats.parse`: a live sample as the daemon sent it, each field checked, in the shape
//! `stats::push` keeps. `tests/stats_parity.rs` holds it to the JavaScript.

use serde_json::{Map, Value, json};

use crate::js;
use crate::stats::{finite, is_object, list, mine, non_neg, num, temp, text};

fn map_list(raw: Option<&Value>, f: impl Fn(&Value, usize) -> Option<Value>) -> Value {
    Value::Array(
        list(raw)
            .iter()
            .enumerate()
            .filter_map(|(i, r)| f(r, i))
            .collect(),
    )
}

fn parse_gpu(g: &Value, _: usize) -> Option<Value> {
    is_object(g).then(|| {
        json!({
            "card": text(g.get("card")), "pciId": text(g.get("pciId")).to_lowercase(),
            "name": text(g.get("name")), "vendor": text(g.get("vendor")), "driver": text(g.get("driver")),
            "integrated": g.get("integrated") == Some(&Value::Bool(true)), "temp": num(temp(g.get("temp"))),
            "busy": num(non_neg(g.get("busy"))), "vramUsed": num(non_neg(g.get("vramUsed"))),
            "vramTotal": num(non_neg(g.get("vramTotal"))),
        })
    })
}

fn parse_process(row: &Value) -> Option<Value> {
    if !is_object(row) {
        return None;
    }
    let pid = js::number(row.get("pid"));
    if !pid.is_finite() || pid <= 0.0 {
        return None;
    }
    let comm = text(row.get("comm")).trim().to_owned();
    if comm.is_empty() {
        return None;
    }
    let state: String = text(row.get("state")).trim().chars().take(1).collect();
    let ppid = js::number(row.get("ppid"));
    let ppid = if !ppid.is_finite() || ppid < 0.0 {
        0.0
    } else {
        ppid
    };
    let n = |k: &str| num(non_neg(row.get(k)));
    Some(json!({
        "pid": js::json_number(pid), "ppid": js::json_number(ppid), "comm": comm,
        "cmdline": text(row.get("cmdline")), "uid": num(finite(row.get("uid"))), "rssKb": n("rssKb"),
        "ticks": n("ticks"), "startTicks": n("startTicks"), "cpu": n("cpu"), "state": state,
        "threads": n("threads"), "nice": num(finite(row.get("nice"))),
        "kthread": row.get("kthread") == Some(&Value::Bool(true)), "mine": mine(row.get("mine")),
        "readBytes": n("readBytes"), "writeBytes": n("writeBytes"), "readBps": n("readBps"),
        "writeBps": n("writeBps"), "fds": n("fds"),
        "depth": js::json_number(non_neg(row.get("depth")).unwrap_or(0.0)),
    }))
}

fn parse_core(row: &Value, index: usize) -> Option<Value> {
    is_object(row).then(|| {
        let id = finite(row.get("id")).unwrap_or(index as f64);
        json!({
            "id": js::json_number(id), "idle": num(non_neg(row.get("idle"))),
            "total": num(non_neg(row.get("total"))), "freqMhz": num(non_neg(row.get("freqMhz"))),
            "governor": text(row.get("governor")), "cpu": num(non_neg(row.get("cpu"))),
        })
    })
}

fn named(row: &Value, key: &str) -> Option<String> {
    let name = text(row.get(key)).trim().to_owned();
    (is_object(row) && !name.is_empty()).then_some(name)
}

fn parse_iface(row: &Value, _: usize) -> Option<Value> {
    let name = named(row, "name")?;
    let mut o = Map::new();
    o.insert("name".into(), Value::from(name));
    for k in [
        "rx",
        "tx",
        "rxBps",
        "txBps",
        "rxPackets",
        "txPackets",
        "rxErr",
        "txErr",
        "rxDrop",
        "txDrop",
    ] {
        o.insert(k.into(), num(non_neg(row.get(k))));
    }
    Some(Value::Object(o))
}

fn parse_disk(row: &Value, _: usize) -> Option<Value> {
    let name = named(row, "name")?;
    let n = |k: &str| num(non_neg(row.get(k)));
    Some(json!({
        "name": name, "readSectors": n("readSectors"), "writeSectors": n("writeSectors"),
        "readBps": n("readBps"), "writeBps": n("writeBps"),
    }))
}

fn parse_sensor(row: &Value, _: usize) -> Option<Value> {
    if !is_object(row) {
        return None;
    }
    let kind = text(row.get("kind"));
    let value = match kind.as_str() {
        "temp" => temp(row.get("value")),
        "fan" => non_neg(row.get("value")),
        _ => return None,
    }?;
    let (chip, label) = (text(row.get("chip")), text(row.get("label")));
    let id = match text(row.get("id")).trim() {
        "" => format!("{chip}:{label}"),
        id => id.to_owned(),
    };
    Some(
        json!({ "id": id, "chip": chip, "label": label, "kind": kind, "value": js::json_number(value) }),
    )
}

fn obj(raw: Option<&Value>) -> &Value {
    static EMPTY: Value = Value::Null;
    raw.filter(|v| is_object(v)).unwrap_or(&EMPTY)
}

fn parse_psi(raw: Option<&Value>) -> Value {
    let src = obj(raw);
    json!({ "cpu": num(non_neg(src.get("cpu"))), "memory": num(non_neg(src.get("memory"))), "io": num(non_neg(src.get("io"))) })
}

fn parse_tcp(raw: Option<&Value>) -> Value {
    let src = obj(raw);
    let n = |k: &str| js::json_number(non_neg(src.get(k)).filter(|n| *n != 0.0).unwrap_or(0.0));
    json!({ "established": n("established"), "listen": n("listen"), "timeWait": n("timeWait"), "closeWait": n("closeWait"), "total": n("total") })
}

fn parse_irq(raw: Option<&Value>) -> Value {
    let Some(raw) = raw.filter(|v| is_object(v)) else {
        return Value::Null;
    };
    let rows: Vec<Value> = list(raw.get("rows"))
        .iter()
        .filter(|r| js::truthy(Some(r)) && r.get("id").is_some_and(|id| !id.is_null()))
        .filter_map(|r| {
            let vals: Vec<Value> = list(r.get("values"))
                .iter()
                .map(|c| num(non_neg(Some(c))))
                .collect();
            (!vals.is_empty()).then(|| json!({ "id": js::string(&r["id"]), "values": vals }))
        })
        .collect();
    if rows.is_empty() {
        return Value::Null;
    }
    json!({ "cpus": num(non_neg(raw.get("cpus"))), "rows": rows })
}

fn parse_sockets(raw: Option<&Value>) -> Value {
    fn present(v: Option<&Value>) -> Option<&Value> {
        v.filter(|x| !x.is_null())
    }
    Value::Array(
        list(raw)
            .iter()
            .enumerate()
            .filter(|(_, s)| js::truthy(Some(s)) && js::truthy(s.get("state")))
            .map(|(i, s)| {
                json!({
                    "inode": present(s.get("inode")).map_or_else(|| i.to_string(), js::string),
                    "state": js::string(&s["state"]),
                    "addr": present(s.get("addr")).map_or(Value::Null, |a| Value::from(js::string(a))),
                    "port": num(non_neg(s.get("port"))), "uid": num(finite(s.get("uid"))),
                    "pid": num(non_neg(s.get("pid"))),
                })
            })
            .collect(),
    )
}

fn parse_buddy(raw: Option<&Value>) -> Value {
    let mut out = Vec::new();
    for n in list(raw) {
        match non_neg(Some(n)) {
            Some(n) => out.push(js::json_number(n)),
            None => return Value::Null,
        }
    }
    if out.is_empty() {
        Value::Null
    } else {
        Value::Array(out)
    }
}

fn parse_cgroup(raw: Option<&Value>) -> Value {
    let Some(raw) = raw.filter(|v| is_object(v)) else {
        return Value::Null;
    };
    let children: Vec<Value> = list(raw.get("children"))
        .iter()
        .map(|c| parse_cgroup(Some(c)))
        .filter(|c| !c.is_null())
        .collect();
    let value = non_neg(raw.get("value"));
    if value.is_none() && children.is_empty() {
        return Value::Null;
    }
    let id = match text(raw.get("id")) {
        id if id.is_empty() => text(raw.get("path")),
        id => id,
    };
    let mut node = Map::new();
    node.insert("id".into(), Value::from(id));
    if let Some(v) = value {
        node.insert("value".into(), js::json_number(v));
    }
    if !children.is_empty() {
        node.insert("children".into(), Value::Array(children));
    }
    Value::Object(node)
}

fn parse_rapl(raw: Option<&Value>) -> Value {
    Value::Array(
        list(raw)
            .iter()
            .filter(|r| js::truthy(Some(r)) && r.get("id").is_some_and(|id| !id.is_null()))
            .filter_map(|r| {
                let uj = non_neg(r.get("uj"))?;
                let id = js::string(&r["id"]);
                let name = match text(r.get("name")) {
                    n if n.is_empty() => id.clone(),
                    n => n,
                };
                Some(json!({ "id": id, "name": name, "uj": js::json_number(uj) }))
            })
            .collect(),
    )
}

fn parse_slabs(raw: Option<&Value>) -> Value {
    Value::Array(
        list(raw)
            .iter()
            .filter_map(|s| {
                let name = text(s.get("name"));
                let active = non_neg(s.get("active"))?;
                (!name.is_empty()).then(|| json!({ "name": name, "active": js::json_number(active), "num": num(non_neg(s.get("num"))) }))
            })
            .collect(),
    )
}

const SCALARS: &[&str] = &[
    "cpuIdle",
    "cpuTotal",
    "load1",
    "load5",
    "load15",
    "memUsed",
    "memTotal",
    "memAvail",
    "memFree",
    "memBuffers",
    "memCached",
    "memShared",
    "memSReclaimable",
    "memAnon",
    "memDirty",
    "memSlab",
    "memPageTables",
    "memKernelStack",
    "memFile",
    "swapUsed",
    "swapTotal",
    "netRx",
    "netTx",
    "uptimeSec",
];

/// `parse(raw)`: a sample as the daemon sent it, each field checked; none for one that is not.
pub fn parse(data: &Value) -> Option<Value> {
    if !is_object(data) {
        return None;
    }
    let mut o = Map::new();
    o.insert("uid".into(), num(finite(data.get("uid"))));
    for k in SCALARS {
        o.insert((*k).into(), num(non_neg(data.get(*k))));
    }
    o.insert("cpus".into(), map_list(data.get("cpus"), parse_core));
    o.insert("ifaces".into(), map_list(data.get("ifaces"), parse_iface));
    o.insert("disks".into(), map_list(data.get("disks"), parse_disk));
    o.insert("psi".into(), parse_psi(data.get("psi")));
    o.insert("tcp".into(), parse_tcp(data.get("tcp")));
    o.insert("tcpSockets".into(), parse_sockets(data.get("tcpSockets")));
    o.insert("interrupts".into(), parse_irq(data.get("interrupts")));
    o.insert("softirqs".into(), parse_irq(data.get("softirqs")));
    o.insert("buddy".into(), parse_buddy(data.get("buddy")));
    o.insert("cgroups".into(), parse_cgroup(data.get("cgroups")));
    o.insert("rapl".into(), parse_rapl(data.get("rapl")));
    o.insert("slabs".into(), parse_slabs(data.get("slabs")));
    let clk = non_neg(data.get("clkTck"))
        .filter(|n| *n != 0.0)
        .unwrap_or(100.0);
    o.insert("clkTck".into(), js::json_number(clk));
    o.insert("cpuTemp".into(), num(temp(data.get("cpuTemp"))));
    o.insert("gpus".into(), map_list(data.get("gpus"), parse_gpu));
    o.insert(
        "sensors".into(),
        map_list(data.get("sensors"), parse_sensor),
    );
    o.insert(
        "processes".into(),
        map_list(data.get("processes"), |r, _| parse_process(r)),
    );
    Some(Value::Object(o))
}
