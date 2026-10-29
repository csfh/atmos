//! A row that calls a setter of `Omarchy.qml` of its own sends what the setter sends
//! (docs/adr/0021). The setter is not restated here: every function of `Omarchy.qml` runs under
//! Node, with the JavaScript libraries it imports, its properties as `Snapshot.js` reads them
//! from a snapshot, and the four ways it reaches the backend (`runSetting`, `runCommand`,
//! `runJob`, `runInteractive`) recording what they would send. The row's own check of the value
//! it shows comes first, as the QML row makes it (`catalog_controls.rs` holds the row's text to
//! it). For every value the row can give, `Setter::body` must be what the setter sent, or
//! nothing when it sent nothing. Without `node` the test says so and stops.

mod common;

use std::io::Write as _;
use std::process::{Command, Stdio};

use atmos_ui::catalog::control::{Options, Shape};
use atmos_ui::catalog::setter::Setter;
use atmos_ui::catalog::{self, Control, Field};
use atmos_ui::js;
use atmos_ui::settings::Settings;
use common::qml;
use serde_json::{Value, json};

const OMARCHY: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const { load } = require(root + "/tests/harness.js");
const src = fs.readFileSync(root + "/services/Omarchy.qml", "utf8");
const cases = JSON.parse(fs.readFileSync(0, "utf8"));

const libs = {};
for (const m of src.matchAll(/^import "(\w+\.js)" as (\w+)$/gm)) {
  try { libs[m[2]] = load("services/" + m[1]); } catch (e) { libs[m[2]] = {}; }
}
const H = libs.HyprPrefs, Snap = libs.SnapshotJs, R = libs.Requests;

// Every function of the root object, by name: the text from its brace to the one that closes it.
const fns = {};
for (const m of src.matchAll(/\n  function (\w+)\(([^)]*)\) \{/g)) {
  let i = m.index + m[0].length, depth = 1;
  while (depth > 0 && i < src.length) {
    const c = src[i++];
    if (c === "{") depth++;
    else if (c === "}") depth--;
  }
  fns[m[1]] = { args: m[2], body: src.slice(m.index + m[0].length, i - 1) };
}

let sent = [];
function scope(state) {
  return new Proxy(state, {
    has: (t, k) => k !== Symbol.unscopables,
    get: (t, k) => (k in t ? t[k] : globalThis[k]),
    set: (t, k, v) => { t[k] = v; return true; },
  });
}

function omarchy(doc) {
  const state = Object.assign({}, libs);
  const s = scope(state);
  for (const [name, f] of Object.entries(fns)) {
    try {
      state[name] = new Function("__s", `with (__s) { return function ${name}(${f.args}) {${f.body}\n} }`)(s);
    } catch (e) {}
  }
  Object.assign(state, {
    root: s, reverting: false, shellDir: "", lastError: "",
    Backend: { bin: "atmosd" }, Qt: { callLater: (f) => f() },
    // The user's home, for the setters that keep a path inside it.
    Quickshell: { env: (k) => (k === "HOME" ? "/home/u" : "") },
    // The window's own paint (`Theme.applyNamedTheme`, `discardPreview`), which sends nothing.
    Theme: new Proxy({}, { get: () => () => {} }),
    runSetting: (label, opts) => {
      if (!opts || opts.hasValue !== true) return;
      sent.push(R.settingsSet(String(opts.domain || opts.key || ""), opts.value, opts.readback));
    },
    runCommand: (op) => { if (op && op.op) sent.push(op); },
    // A job is listed by its kind, and waits for sudo mode when the setter says so.
    runJob: (op, kind, opts) => {
      if (op) sent.push(Object.assign({}, op, { kind: String(kind || ""), sudo: !!(opts && opts.sudo), interactive: false }));
    },
    runInteractive: (op, opts) => {
      if (op) sent.push(Object.assign({}, op, { kind: String((opts && opts.kind) || ""), sudo: false, interactive: true }));
    },
  });
  for (const [key, raw] of Object.entries(doc)) {
    if (key === "hyprLook") state.hyprLook = H.clampLook(raw);
    else if (key === "hyprInput") state.hyprInput = H.clampInput(raw);
    else state[key] = Snap.clampPatchedValue(key, raw, {
      parseTime: libs.HyprSunset.parseTime,
      parseChannel: libs.AtmosUpdate.parseChannel,
      parseWeatherCoords: libs.RichUi.parseWeatherCoords,
    });
  }
  // What `Snapshot.js` derives from a list it adopts.
  if ("audioSinks" in doc) state.audioSink = Snap.defaultAudioName(state.audioSinks);
  if ("audioSources" in doc) state.audioSource = Snap.defaultAudioName(state.audioSources);
  state.snapshotData = doc;
  return state;
}

process.stdout.write(JSON.stringify(cases.map((c) => {
  const state = omarchy(c.doc);
  // The row's own check: `if (next !== Omarchy.x)`, or `|| Omarchy.muted` for a volume.
  if (c.rowSame && c.value === state[c.shown] && !(c.unless && state[c.unless] === true)) return null;
  sent = [];
  state[c.setter](...c.args);
  if (c.all) return sent;
  if (sent.length > 1) return { many: sent };
  return sent[0] || null;
})));
"#;

/// What a snapshot's value of a key can be.
fn samples() -> Vec<Value> {
    vec![
        Value::Null,
        json!(true),
        json!(false),
        json!(0),
        json!(1),
        json!(5),
        json!(50),
        json!(75),
        json!(100),
        json!(101),
        json!(-5),
        json!(1800),
        json!(""),
        json!("balanced"),
        json!("performance"),
        json!("auto"),
        json!("5"),
        json!("metric"),
        json!(["balanced", "performance"]),
        json!("en_US.UTF-8"),
        json!("Europe/Copenhagen"),
    ]
}

/// The window's settings with every topic carrying `doc`.
fn settings(doc: &Value) -> Settings {
    let mut s = Settings::default();
    for topic in [
        "settings.look",
        "settings.host",
        "settings.network",
        "settings.system",
    ] {
        s.adopt(topic, 1, doc);
    }
    s
}

/// The values a row can give its setter while its field shows `current`.
fn values(field: Field, shape: Shape, current: &Value, s: &Settings) -> Vec<Value> {
    match shape {
        Shape::Toggle => vec![Control::flip(field, current)],
        Shape::Button(_) => vec![current.clone()],
        Shape::Count(range) => {
            let mid = (range.min + range.max) / 2.0;
            [
                range.min,
                range.max,
                mid,
                range.min - 1.0,
                range.max + 1.0,
                js::number(Some(current)),
            ]
            .into_iter()
            .filter(|n| n.is_finite())
            .map(|n| js::json_number(range.round(n)))
            .collect()
        }
        Shape::Pick(options) => {
            let mut out: Vec<Value> = options
                .list(s)
                .into_iter()
                .map(|(v, _)| Value::from(v))
                .collect();
            if let Options::Fixed(_) = options {
                out.push(json!("bogus"));
            }
            out.extend([json!(""), current.clone()]);
            out
        }
    }
}

/// Every key the setter's row reads, besides its own field.
fn others(setter: &Setter, options: Option<Options>) -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    if let atmos_ui::catalog::setter::Args::GivenAnd(f) = setter.args {
        out.push(f);
    }
    out.extend(setter.check.unless);
    out.extend(setter.check.blocked);
    out.extend(setter.check.needs);
    out.extend(options.and_then(Options::field));
    out
}

#[test]
fn each_row_that_calls_a_setter_sends_what_the_setter_sends() {
    let mut cases = Vec::new();
    let mut ours = Vec::new();
    let mut rows = 0;
    for page in catalog::pages() {
        for line in page.lines() {
            let Control::Calls(field, shape, setter) = line.control else {
                continue;
            };
            rows += 1;
            let options = match shape {
                Shape::Pick(o) => Some(o),
                _ => None,
            };
            for raw in samples() {
                for flag in [false, true] {
                    // A device picker's field is derived from a list of devices.
                    let raw = match field {
                        Field::Derived(..) => json!([
                            {"name": "alsa.a", "id": 41, "description": "Speakers", "default": flag},
                            {"name": "alsa.b", "id": "42"},
                            {"name": "", "id": 3},
                        ]),
                        _ => raw.clone(),
                    };
                    let mut doc = json!({ field.key(): raw });
                    for other in others(&setter, options)
                        .into_iter()
                        .filter(|o| o.key() != field.key())
                    {
                        let v = match other {
                            Field::Key(k) if k.ends_with('s') => {
                                json!(["balanced", "performance", "power-saver", "auto", "5",
                                    "en_US.UTF-8", "C.UTF-8", "Europe/Copenhagen", "bad..zone",
                                    {"value": "da_DK.UTF-8", "label": "Danish"}])
                            }
                            _ => json!(flag),
                        };
                        doc[other.key()] = v;
                    }
                    if let atmos_ui::catalog::setter::Sends::HyprInput(_) = setter.sends {
                        doc["hyprInput"] = json!({});
                    }
                    let s = settings(&doc);
                    let Some(current) = field.read(&s).value().cloned() else {
                        continue;
                    };
                    for value in values(field, shape, &current, &s) {
                        ours.push(setter.body(Some(field), value.clone(), &s));
                        cases.push(json!({
                            "doc": doc, "setter": setter.name, "args": setter.call_args(&value, &s),
                            "value": value, "rowSame": setter.check.row_same, "shown": field.key(),
                            "unless": setter.check.unless.map(Field::key),
                            "at": format!("{} / {}", page.id, line.label),
                        }));
                    }
                }
            }
        }
    }
    // A button calls its setter with nothing, and has no field of its own.
    for page in catalog::pages() {
        for line in page.lines() {
            let setters: Vec<Setter> = match line.control {
                Control::Press(_, setter) => vec![setter],
                Control::Buttons(all) => all.iter().map(|b| b.setter).collect(),
                Control::Custom(c) => {
                    for (name, doc, args, body) in c.samples() {
                        // A sample that sends several requests is held to all of them.
                        let all = body.as_ref().is_some_and(Value::is_array);
                        ours.push(body);
                        cases.push(json!({
                            "doc": doc, "setter": name, "args": args, "value": null,
                            "rowSame": false, "shown": null, "unless": null, "all": all,
                            "at": format!("{} / {}", page.id, line.label),
                        }));
                    }
                    // A custom row's setter that takes a value is given both booleans, with its
                    // field read.
                    for setter in c.setters() {
                        if setter.args == atmos_ui::catalog::setter::Args::Nothing {
                            continue;
                        }
                        for (raw, value) in
                            [(json!({}), true), (json!({"swappiness": true}), false)]
                        {
                            let doc = json!({ "tweaks": raw });
                            let s = settings(&doc);
                            let field = Some(Field::Key("tweaks"));
                            ours.push(setter.body(field, json!(value), &s));
                            cases.push(json!({
                                "doc": doc, "setter": setter.name,
                                "args": setter.call_args(&json!(value), &s), "value": value,
                                "rowSame": false, "shown": null, "unless": null,
                                "at": format!("{} / {}", page.id, line.label),
                            }));
                        }
                    }
                    c.setters()
                        .into_iter()
                        .filter(|s| s.args == atmos_ui::catalog::setter::Args::Nothing)
                        .collect()
                }
                _ => continue,
            };
            rows += 1;
            for setter in setters {
                // A setter given a line of text (a prompt's) is given keys and junk.
                if setter.args == atmos_ui::catalog::setter::Args::Given {
                    let key =
                        "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";
                    for text in [
                        format!("ssh-ed25519 {key} me@host"),
                        format!("  ssh-rsa {key}  "),
                        "ssh-ed25519 short".to_owned(),
                        "-----BEGIN".to_owned(),
                        String::new(),
                    ] {
                        let s = settings(&json!({}));
                        ours.push(setter.body(None, json!(text), &s));
                        cases.push(json!({
                            "doc": {}, "setter": setter.name, "args": [text], "value": text,
                            "rowSame": false, "shown": null, "unless": null,
                            "at": format!("{} / {}", page.id, line.label),
                        }));
                    }
                    continue;
                }
                for flag in [false, true] {
                    let mut doc = json!({});
                    for other in others(&setter, None) {
                        doc[other.key()] = json!(flag);
                    }
                    let s = settings(&doc);
                    ours.push(setter.body(None, Value::Null, &s));
                    cases.push(json!({
                        "doc": doc, "setter": setter.name, "args": [], "value": null,
                        "rowSame": false, "shown": null, "unless": null,
                        "at": format!("{} / {}", page.id, line.label),
                    }));
                }
            }
        }
    }
    // A built-in profile's Apply: `applyProfileValues` with its values, each sent.
    for profile in atmos_ui::profiles::PROFILES {
        for doc in [
            json!({}),
            json!({"hyprLook": {"gapsIn": 7}, "powerProfile": "balanced"}),
        ] {
            let values: serde_json::Map<String, Value> = profile
                .values()
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect();
            let bodies: Vec<Value> = profile.writes().into_iter().map(|(_, b)| b).collect();
            ours.push(Some(Value::Array(bodies)));
            cases.push(json!({
                "doc": doc, "setter": "applyProfileValues", "args": [values], "value": null,
                "rowSame": false, "shown": null, "unless": null, "all": true,
                "at": format!("profiles / {}", profile.title),
            }));
        }
    }
    assert!(rows >= 15, "the rows that call setters are found: {rows}");
    let mut child = match Command::new("node")
        .args(["-e", OMARCHY])
        .env("REPO_DIR", qml::repo())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!("skip: node is not installed");
            return;
        }
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(Value::Array(cases.clone()).to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let theirs: Value = serde_json::from_slice(&out.stdout).unwrap();
    let theirs = theirs.as_array().unwrap();
    assert_eq!(theirs.len(), cases.len());
    let sent = theirs.iter().filter(|t| !t.is_null()).count();
    assert!(
        sent > 100,
        "the setters sent something: {sent} of {}",
        cases.len()
    );
    for ((case, ours), theirs) in cases.iter().zip(ours).zip(theirs) {
        let mut theirs = theirs.clone();
        if let Some(o) = theirs.as_object_mut() {
            o.remove("id");
        }
        for item in theirs.as_array_mut().into_iter().flatten() {
            if let Some(o) = item.as_object_mut() {
                o.remove("id");
            }
        }
        assert_eq!(
            ours.unwrap_or(Value::Null),
            theirs,
            "{}: {}",
            case["at"],
            json!({"doc": case["doc"], "value": case["value"]})
        );
    }
}
