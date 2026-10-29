//! Every mapped row of a catalog page is its QML row (docs/adr/0021). For each one:
//!
//! - the QML row calls the path its control names, with the same range, choices, labels and
//!   conditions (`available:`, `enabled:`), read from the page's source;
//! - what it shows is what the QML shows: `HyprPrefs.clampLook`, `clampInput` and
//!   `Snapshot.clampPatchedValue` run under Node on the same snapshot values;
//! - what it writes is what the QML writes: the row's own rounding and no-op check, then
//!   `writeHyprLook`, `writeHyprInput` or `set` (`SettingTable.plan`), then `Settings.commandFor`
//!   and `Requests.settingsSet`, run under Node. The glue between them, from `Omarchy.qml`, is
//!   restated in a few lines, as in `tweaks_parity.rs`;
//! - a typed write is the one the setter the row calls sends, with the same arguments.
//!
//! Without `node` the tests that need it say so and stop.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use std::io::Write as _;
use std::process::{Command, Stdio};

use atmos_ui::catalog::control::{Range, When, on};
use atmos_ui::catalog::setter::Sends;
use atmos_ui::catalog::{self, Control, Field, Line, Page};
use atmos_ui::js;
use atmos_ui::settings::Settings;
use common::qml;
use rows::*;
use serde_json::{Value, json};

/// Every mapped row, with its page.
fn mapped() -> Vec<(&'static Page, &'static Line)> {
    catalog::pages()
        .flat_map(|p| p.lines().map(move |l| (p, l)))
        .filter(|(_, l)| l.mapped())
        .collect()
}

#[test]
fn each_mapped_row_is_the_qml_row_it_stands_for() {
    let omarchy = std::fs::read_to_string(qml::repo().join("services/Omarchy.qml")).unwrap();
    assert!(mapped().len() >= 50, "the mapped rows are found");
    for (page, line) in mapped() {
        let row = qml_row(page, line.label, nth(page, line));
        let at = format!("{} / {}", page.id, line.label);
        // Each condition the QML row has, the Rust row has, and no other. `enabled: true` is
        // none, a shared row's `!requirePresent ||` holds unless the page sets it, and so does
        // a navigator, which this window always has.
        let qml = |name: &str| -> Vec<String> {
            let head = format!("{name}: ");
            row.lines()
                .filter_map(|l| l.trim().strip_prefix(&head).map(str::to_owned))
                .filter(|c| {
                    c != "true"
                        && !c.starts_with("!requirePresent ||")
                        && c != "!!(root.navigator && root.navigator.go)"
                })
                // A choice with no options cannot be used, so `options.length > 0` is the
                // control's own.
                .map(|c| {
                    let listed = c
                        .find("root.")
                        .is_some_and(|at| c[at..].contains("Options.length > 0"));
                    if !listed {
                        return c;
                    }
                    let kept: Vec<&str> = c
                        .split(" && ")
                        .filter(|part| {
                            !(part.starts_with("root.") && part.ends_with("Options.length > 0"))
                        })
                        .collect();
                    kept.join(" && ")
                })
                .filter(|c| !c.is_empty())
                .collect()
        };
        // A row of buttons has its conditions on each button, held below.
        let row_enabled = if matches!(line.control, Control::Buttons(_) | Control::Custom(_)) {
            None
        } else {
            Some(("enabled", line.enabled))
        };
        // A custom row decides when it shows too, and `catalog_custom.rs` holds that.
        let row_available =
            (!matches!(line.control, Control::Custom(_))).then_some(("available", line.available));
        for (name, when) in row_available.into_iter().chain(row_enabled) {
            let want: Vec<String> = match when {
                When::Always => Vec::new(),
                w => vec![qml_when(w)],
            };
            let have = qml(name);
            assert!(
                have.iter().all(|c| want.contains(c)) && want.iter().all(|c| have.contains(c)),
                "{at}: {name} is {have:?} in the QML and {want:?} here\n{row}"
            );
        }
        match line.control {
            Control::Pending => unreachable!(),
            Control::Switch(f) => {
                assert!(row.contains("PrefsToggle"), "{at}");
                assert!(row.contains(&qml_field(f)), "{at}: reads {}", qml_field(f));
                assert!(row.contains(&qml_writer(f)), "{at}: {}", qml_writer(f));
            }
            Control::Number(f, range) => number_row(&at, &row, f, range),
            Control::Choice(f, choices) => {
                assert!(row.contains(&qml_writer(f)), "{at}: {}", qml_writer(f));
                let mut last = 0;
                for (value, label) in choices {
                    let item = format!("{{ value: {}, label: {} }}", json!(value), json!(label));
                    let found = row[last..].find(&item);
                    assert!(found.is_some(), "{at}: {item} in order");
                    last += found.unwrap();
                }
            }
            Control::Text(f, placeholder) => {
                assert!(row.contains(&format!("placeholder: {}", json!(placeholder))));
                assert!(row.contains(&qml_writer(f)), "{at}: {}", qml_writer(f));
            }
            Control::Acts(acts) => {
                for act in acts {
                    assert!(row.contains(&format!("text: {}", json!(act.label))), "{at}");
                    assert!(
                        row.contains(&format!("Omarchy.{}()", act.setter)),
                        "{at}: calls {}",
                        act.setter
                    );
                    let body = omarchy
                        .split(&format!("  function {}(", act.setter))
                        .nth(1)
                        .and_then(|rest| rest.split("\n  }\n").next())
                        .unwrap_or_else(|| panic!("{at}: Omarchy has no {}", act.setter));
                    let write = format!("Requests.write({}, {})", json!(act.name), js_args(act));
                    assert!(body.contains(&write), "{at}: {} sends {write}", act.setter);
                }
            }
            Control::Go(label, target) => {
                assert!(row.contains(&format!("text: {}", json!(label))), "{at}");
                assert!(atmos_ui::hubs::known(target), "{at}: {target}");
                // The QML goes there by name, or by a page of its hub (`openSubpage`), where
                // Windows' `bindings` is Keybindings.
                let tail = target.rsplit('/').next().unwrap_or(target);
                let sub = if target == "keybindings" {
                    "bindings"
                } else {
                    tail
                };
                assert!(
                    row.contains(&format!("go({})", json!(target)))
                        || row.contains(&format!("openSubpage({})", json!(sub))),
                    "{at}: opens {target}"
                );
            }
            Control::Says(text) => {
                assert!(row.contains(&format!("valueText: {}", json!(text))), "{at}");
            }
            Control::Shows(f) => {
                assert!(
                    row.contains(&format!("valueText: {}", qml_field(f))),
                    "{at}"
                );
            }
            Control::Picks(f, options) => {
                assert!(row.contains("PrefsSelect"), "{at}");
                assert!(row.contains(&format!("value: {}", qml_field(f))), "{at}");
                assert!(row.contains(&qml_writer(f)), "{at}: {}", qml_writer(f));
                options_row(&at, &row, &page_src(page), options);
            }
            Control::Calls(f, shape, setter) => {
                calls_row(&at, &row, &page_src(page), f, shape, setter);
            }
            Control::Buttons(all) => {
                let buttons = row.matches("PrefsButton {").count();
                assert_eq!(buttons, all.len(), "{at}: every button of the row");
                let page = page_src(page);
                for b in all {
                    buttons_row(&at, &row, &page, &omarchy, b);
                }
            }
            // A custom row is held to its QML by `catalog_custom.rs`.
            Control::Custom(_) => {}
            Control::Bare => {
                assert!(
                    !row.contains("Prefs") && !row.contains("valueText"),
                    "{at}: no control"
                );
            }
            Control::Minutes(range) => {
                let value = "value: Omarchy.sudoMinutes";
                assert!(row.contains(value), "{at}: {value}");
                for (name, n) in [
                    ("from", range.min),
                    ("to", range.max),
                    ("stepSize", range.step),
                ] {
                    let want = format!("{name}: {}", qml_number(n));
                    assert!(row.contains(&want), "{at}: {want}");
                }
                let set = "Omarchy.sudoMinutes = Math.round(value)";
                assert!(row.contains(set), "{at}: {set}");
            }
            Control::Press(label, setter) => {
                assert!(row.contains(&format!("text: {}", json!(label))), "{at}");
                let call = format!("Omarchy.{}()", setter.name);
                assert!(row.contains(&call), "{at}: calls {call}");
                if let Sends::Job(t) = setter.sends {
                    // The label the job is listed by is the one the setter gives it.
                    let body = omarchy
                        .split(&format!("  function {}(", setter.name))
                        .nth(1)
                        .and_then(|rest| rest.split("\n  }\n").next())
                        .unwrap_or_default();
                    let kind = json!(t.kind).to_string();
                    assert!(
                        body.contains(&kind),
                        "{at}: {} lists the job as {kind}",
                        setter.name
                    );
                }
            }
        }
    }
}

/// Runs `script` under Node on `input`, and reads its JSON output.
fn node(script: &str, input: &Value) -> Option<Value> {
    let mut child = Command::new("node")
        .args(["-e", script])
        .env("REPO_DIR", qml::repo())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "node failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

const READS: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const { load } = require(root + "/tests/harness.js");
const H = load("services/HyprPrefs.js");
const Snap = load("services/Snapshot.js");
// The adapter the window reads the schedule's times with (`Omarchy.snapshotAdapters`).
const adapters = {
  parseTime: load("services/HyprSunset.js").parseTime,
  parseChannel: load("services/AtmosUpdate.js").parseChannel,
  parseWeatherCoords: load("services/RichUi.js").parseWeatherCoords,
};
const cases = JSON.parse(fs.readFileSync(0, "utf8"));
const MISSING = "\u0000missing";
const v = (x) => (x === MISSING ? undefined : x);
process.stdout.write(JSON.stringify(cases.map((c) => {
  if (c.kind === "look") return H.clampLook(v(c.raw))[c.field];
  if (c.kind === "input") return H.clampInput(v(c.raw))[c.field];
  if (c.kind === "derived") return Snap.defaultAudioName(v(c.raw));
  return Snap.clampPatchedValue(c.field, v(c.raw), adapters);
})));
"#;

/// What a snapshot's value of a field can be.
fn samples() -> Vec<Value> {
    vec![
        Value::Null,
        json!(true),
        json!(false),
        json!("true"),
        json!("false"),
        json!("on"),
        json!("off"),
        json!("1"),
        json!("0"),
        json!(0),
        json!(1),
        json!(2),
        json!(-1),
        json!(0.5),
        json!(0.123456),
        json!(7),
        json!(100),
        json!(1e9),
        json!("7"),
        json!(" 12 "),
        json!("abc"),
        json!(""),
        json!("0x10"),
        json!("Infinity"),
        json!("scrolling"),
        json!("flat"),
        json!("adaptive"),
        json!("kitty|alacritty"),
        json!("a\nb"),
        json!("x".repeat(130)),
        json!([]),
        json!([3]),
        json!({}),
    ]
}

/// The fields the mapped rows read, each once.
fn fields() -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    for (_, line) in mapped() {
        let gates = line
            .available
            .fields()
            .into_iter()
            .chain(line.enabled.fields());
        for f in line.control.fields().into_iter().chain(gates) {
            if !out.contains(&f) && f != Field::Key("extras") {
                out.push(f);
            }
        }
    }
    out
}

fn kind(f: Field) -> (&'static str, &'static str) {
    match f {
        Field::Look(f) => ("look", f),
        Field::Input(f) => ("input", f),
        Field::Key(k) => ("key", k),
        Field::Derived(list, _) => ("derived", list),
    }
}

/// The snapshot value of the field's key that holds `v`.
fn raw(f: Field, v: &Value) -> Value {
    match f {
        Field::Look(name) | Field::Input(name) => json!({ name: v }),
        Field::Key(_) | Field::Derived(..) => v.clone(),
    }
}

#[test]
fn each_field_reads_what_the_qml_reads() {
    let mut cases = Vec::new();
    let mut ours = Vec::new();
    for f in fields() {
        let (kind, field) = kind(f);
        let mut raws: Vec<Value> = samples().iter().map(|v| raw(f, v)).collect();
        if !matches!(f, Field::Key(_)) {
            raws.extend([Value::Null, json!("x"), json!([]), json!({})]);
        }
        if let Field::Derived(..) = f {
            raws.extend([
                json!([{"name": "a", "default": true}, {"name": "b"}]),
                json!([{"name": "a"}, {"name": "b", "default": 1}]),
                json!([{"default": true}, {"name": 5, "default": "yes"}]),
                json!([null, {"name": "", "default": true}]),
            ]);
        }
        for raw in raws {
            ours.push(f.normalize(&raw));
            cases.push(json!({"kind": kind, "field": field, "raw": raw}));
        }
    }
    let Some(theirs) = node(READS, &Value::Array(cases.clone())) else {
        eprintln!("skip: node is not installed");
        return;
    };
    for ((case, ours), theirs) in cases.iter().zip(ours).zip(theirs.as_array().unwrap()) {
        assert_eq!(&ours, theirs, "{case}");
    }
}

const WRITES: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const { load } = require(root + "/tests/harness.js");
const H = load("services/HyprPrefs.js");
const Snap = load("services/Snapshot.js");
const S = load("services/Settings.js");
const R = load("services/Requests.js");
const T = load("services/SettingTable.js");
const cases = JSON.parse(fs.readFileSync(0, "utf8"));

// What the row does with what it was given, then Omarchy's path to the backend.
function body(c) {
  let value = c.value;
  if (c.places !== null) {
    const scale = Math.pow(10, c.places);
    value = Math.round(c.typed * scale) / scale;
  }
  if (c.kind === "look") {
    const current = H.clampLook(c.raw);
    if (c.skip && value === current[c.field]) return null;
    const patch = {};
    patch[c.field] = value;
    const look = H.clampLook(Snap.mergeSnapshot(current, patch));
    return send("hyprLook." + c.field, look[c.field], { hyprLook: look });
  }
  if (c.kind === "input") {
    const current = H.clampInput(c.raw);
    if (c.skip && value === current[c.field]) return null;
    const patch = {};
    patch[c.field] = value;
    const input = H.clampInput(Snap.mergeSnapshot(current, patch));
    input.kbLayout = input.kbLayoutOverride;
    return send("hyprInput." + c.field, input[c.field], { hyprInput: input });
  }
  const current = Snap.clampPatchedValue(c.field, c.raw, {});
  const step = T.plan(c.field, value, current);
  if (step.known) {
    if (!step.dispatch) return null;
    value = step.value;
  } else if (value === current) {
    return null;
  }
  const snap = {};
  snap[c.field] = current;
  return send(c.field, value, snap);
}

function send(domain, value, snap) {
  const cmd = S.commandFor(domain, value, snap, {});
  if (!cmd || cmd.skip) return { refused: domain };
  return R.settingsSet(domain, cmd.value, cmd.readback);
}

process.stdout.write(JSON.stringify(cases.map(body)));
"#;

/// The window's settings, with `key` read as `raw`.
fn settings(f: Field, raw: &Value) -> Settings {
    let mut s = Settings::default();
    let doc = json!({ f.key(): raw });
    s.adopt(f.held().topic, 1, &doc);
    s
}

#[test]
fn each_mapped_row_writes_what_the_qml_row_writes() {
    let mut cases = Vec::new();
    let mut ours = Vec::new();
    let mut push =
        |f: Field, raw: &Value, value: Value, typed: Option<(f64, Range)>, skip: bool| {
            let s = settings(f, raw);
            let (kind, field) = kind(f);
            ours.push(if skip {
                f.write(value.clone(), &s)
            } else {
                f.send(value.clone(), &s)
            });
            cases.push(json!({
                "kind": kind, "field": field, "raw": raw, "value": value, "skip": skip,
                "typed": typed.map(|(t, _)| t), "places": typed.map(|(_, r)| r.places),
            }));
        };
    for (_, line) in mapped() {
        let Some(f) = line.control.field() else {
            continue;
        };
        for v in samples() {
            let raw = raw(f, &v);
            let Some(current) = f.read(&settings(f, &raw)).value().cloned() else {
                continue;
            };
            match line.control {
                // A switch writes the other value, with no check of its own.
                Control::Switch(_) => {
                    let next = Control::flip(f, &current);
                    assert_ne!(on(&next), on(&current));
                    push(f, &raw, next, None, false);
                }
                Control::Number(_, range) => {
                    let mid = (range.min + range.max) / 2.0;
                    for typed in [
                        range.min,
                        range.max,
                        mid,
                        mid + range.step / 3.0,
                        range.min - 1.0,
                    ] {
                        let value = js::json_number(range.round(typed));
                        push(f, &raw, value, Some((typed, range)), true);
                    }
                }
                Control::Choice(_, choices) => {
                    for (value, _) in choices {
                        push(f, &raw, Control::chosen(f, value), None, true);
                    }
                }
                Control::Picks(..) => {
                    for chosen in ["balanced", "performance", "power-saver", ""] {
                        push(f, &raw, json!(chosen), None, true);
                    }
                }
                // Set writes what the field holds, with no check of its own.
                Control::Text(..) => {
                    for text in ["kitty|alacritty", "", "  x  ", "a\nb"] {
                        push(f, &raw, json!(text), None, false);
                    }
                    push(f, &raw, current.clone(), None, false);
                }
                _ => {}
            }
        }
    }
    let Some(theirs) = node(WRITES, &Value::Array(cases.clone())) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let theirs = theirs.as_array().unwrap();
    assert!(theirs.len() > 500, "the cases ran: {}", theirs.len());
    for ((case, ours), theirs) in cases.iter().zip(ours).zip(theirs) {
        assert_eq!(&ours.unwrap_or(Value::Null), theirs, "{case}");
    }
}

/// A typed write a row sends is one the backend has: a typed write or a registry entry.
#[test]
fn each_typed_write_is_one_the_backend_has() {
    let read = |p: &str| std::fs::read_to_string(qml::repo().join(p)).unwrap();
    let registry = read("backend/src/command/writes.rs") + &read("backend/src/command/entries.rs");
    for (page, line) in mapped() {
        if let Control::Acts(acts) = line.control {
            for act in acts {
                assert!(
                    registry.contains(&format!("\"{}\"", act.name)),
                    "{}: {}",
                    page.id,
                    act.name
                );
                assert_eq!(act.body()["op"], "write");
            }
        }
    }
}

/// A description picked by a condition is the QML row's own: its `description:` is that
/// condition and the two texts, as the QML writes them.
#[test]
fn each_varying_description_is_the_qml_rows_condition_and_texts() {
    let mut found = 0;
    for page in catalog::pages() {
        for line in page.lines() {
            let Some(v) = line.varies else {
                continue;
            };
            found += 1;
            let row = qml_row(page, line.label, nth(page, line));
            // From the row's `description:` to its `hint:`, however the file is indented.
            let expr: Vec<&str> = row
                .lines()
                .map(str::trim)
                .skip_while(|l| !l.starts_with("description: "))
                .take_while(|l| !l.starts_with("hint:"))
                .collect();
            assert!(
                !expr.is_empty(),
                "{} / {}: no description",
                page.id,
                line.label
            );
            let expr = expr.join(" ");
            let expr = expr["description: ".len()..]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let want = format!(
                "{} ? {} : {}",
                qml_when(v.when),
                json!(v.then),
                json!(v.otherwise)
            );
            assert_eq!(expr, want, "{} / {}", page.id, line.label);
        }
    }
    assert!(found >= 15, "the varying descriptions are found: {found}");
}
