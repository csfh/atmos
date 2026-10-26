//! The Rust Tweaks page is the QML one until the QML one goes: `services/Tweaks.js` runs under Node
//! and must give what `tweaks.rs` gives for the catalog, its groups, whether each tweak is on for
//! every shape a snapshot value can take, and the `settings.set` a toggle and a Reset send.
//!
//! The JavaScript side of a write is the QML's own path to the backend, run for real where it can
//! be: `Tweaks.writeFor` (or `resetWrite` for Reset), then `HyprPrefs.clampInput` for a Hyprland field (`writeHyprInput`), then
//! `Settings.commandFor` for the value and the read-back keys, then `Requests.settingsSet`. The
//! glue between them, from `Omarchy.qml`, is restated here in three lines, and the QML page is
//! checked to hand its logic to `Tweaks.js`. Without `node` the test says so and stops.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use atmos_ui::tweaks::{self, CATALOG, Kind, Setting};
use serde_json::{Value, json};

const NODE: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const { load } = require(root + "/tests/harness.js");
const T = require(root + "/services/Tweaks.js");
const S = load("services/Settings.js");
const H = load("services/HyprPrefs.js");
const R = load("services/Requests.js");
const input = JSON.parse(fs.readFileSync(0, "utf8"));
const MISSING = input.missing;

// What `TweaksPage.writeTweak` reaches the backend with: Omarchy.writeHyprInput for a Hyprland
// field, Omarchy.setTweak (dispatchSetting) for the rest, then runSettingCommand.
function body(t, w) {
  if (!w) return null;
  let domain, value, snap;
  if (t.kind === "hypr-input") {
    const patch = {};
    patch[w.field] = w.value;
    const merged = Object.assign(H.defaultInput(), patch);
    const clamped = H.clampInput(merged);
    domain = "hyprInput." + w.field;
    value = clamped[w.field];
    snap = { hyprInput: clamped };
  } else {
    domain = "tweaks." + String(t.id || "");
    value = w.value === true;
    snap = {};
  }
  const cmd = S.commandFor(domain, value, snap, {});
  if (!cmd || cmd.skip) return { refused: domain };
  return R.settingsSet(domain, cmd.value, cmd.readback);
}

const catalog = T.catalog();
const out = {
  catalog,
  groups: T.groupedCatalog().map((g) => [g.group, g.items.map((i) => i.id)]),
  missing: T.byId("missing"),
  defaults: T.defaultState(),
  rows: catalog.map((t) => ({
    id: t.id,
    byId: T.byId(t.id) && T.byId(t.id).id,
    reset: T.resetValue(t),
    writeOff: T.writeFor(t, false),
    writeOn: T.writeFor(t, true),
    resetWrite: T.resetWrite(t),
    hyprDefault: t.kind === "hypr-input" ? H.defaultInput()[T.writeFor(t, true).field] : null,
    on: body(t, T.writeFor(t, true)),
    off: body(t, T.writeFor(t, false)),
    resetBody: body(t, T.resetWrite(t)),
    unset: T.currentOn(t, H.defaultInput(), {}),
    states: input.states.map((s) => {
      const v = s === MISSING ? undefined : s;
      return T.currentOn(t, v, v);
    }),
  })),
};
process.stdout.write(JSON.stringify(out));
"#;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// A string Node reads as `undefined`, which JSON cannot carry.
const MISSING: &str = "\u{0}missing";

/// What a snapshot key can be: absent, not an object, or an object whose field is any of the
/// values a file or a tool could leave there. Each tweak reads its own field of it.
fn states() -> Vec<Value> {
    let fields = [
        Value::Null,
        json!(true),
        json!(false),
        json!("true"),
        json!("false"),
        json!(1),
        json!(0),
        json!("flat"),
        json!("FLAT"),
        json!("adaptive"),
        json!(""),
        json!({}),
        json!([]),
    ];
    let mut out = vec![
        json!(MISSING),
        Value::Null,
        json!("x"),
        json!([]),
        json!({}),
    ];
    for t in CATALOG {
        for v in &fields {
            out.push(json!({ t.field(): v }));
        }
    }
    out
}

fn setting(s: Setting) -> Value {
    s.to_json()
}

#[test]
fn the_rust_tweaks_give_what_tweaks_js_gives() {
    let states = states();
    let input = json!({ "missing": MISSING, "states": states });
    let mut node = match Command::new("node")
        .args(["-e", NODE])
        .env("REPO_DIR", repo())
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
    let js: Value = serde_json::from_slice(&out.stdout).unwrap();

    // The catalog, field by field and in order.
    let theirs = js["catalog"].as_array().unwrap();
    assert_eq!(theirs.len(), CATALOG.len(), "the same number of tweaks");
    for (t, j) in CATALOG.iter().zip(theirs) {
        let ours = json!({
            "id": t.id, "group": t.group, "label": t.label, "description": t.description,
            "modifies": t.modifies, "key": t.key, "onValue": setting(t.on),
            "offValue": setting(t.off), "defaultValue": setting(t.default), "kind": t.kind.id(),
        });
        assert_eq!(&ours, j, "{}", t.id);
    }
    let groups: Vec<Value> = tweaks::groups()
        .into_iter()
        .map(|(name, items)| json!([name, items.iter().map(|t| t.id).collect::<Vec<_>>()]))
        .collect();
    assert_eq!(Value::Array(groups), js["groups"], "groupedCatalog");
    assert_eq!(js["missing"], Value::Null);
    assert!(tweaks::by_id("missing").is_none());

    let mut compared = 0;
    for (t, row) in CATALOG.iter().zip(js["rows"].as_array().unwrap()) {
        assert_eq!(row["id"], t.id);
        assert_eq!(row["byId"], t.id, "byId finds it");
        assert_eq!(tweaks::by_id(t.id).map(|b| b.id), Some(t.id));
        assert_eq!(row["reset"], t.reset_value(), "{} resetValue", t.id);
        assert_eq!(
            row["writeOff"]["value"],
            t.value(false),
            "{} writeFor off",
            t.id
        );
        assert_eq!(
            row["writeOn"]["value"],
            t.value(true),
            "{} writeFor on",
            t.id
        );
        assert_eq!(row["writeOn"]["domain"], t.key);
        assert_eq!(row["writeOn"]["field"], t.field());
        // What is sent, after the QML's own clamping and commandFor.
        assert_eq!(row["on"], t.write(true), "{} toggled on", t.id);
        assert_eq!(row["off"], t.write(false), "{} toggled off", t.id);
        assert_eq!(row["resetBody"], t.reset(), "{} Reset", t.id);
        assert_eq!(row["resetWrite"]["domain"], t.key);
        assert_eq!(
            row["resetBody"]["value"], row["reset"],
            "{} Reset writes resetValue",
            t.id
        );

        // The default is where Atmos models what Omarchy sets: `defaultState` for a tweak file,
        // `HyprPrefs.defaultInput` for a Hyprland field. An unset value reads as it.
        assert_eq!(row["unset"], t.default_on(), "{} unset", t.id);
        match t.kind {
            Kind::Tweak => assert_eq!(
                js["defaults"][t.id],
                t.reset_value(),
                "{} defaultState",
                t.id
            ),
            Kind::HyprInput => {
                assert_eq!(row["hyprDefault"], t.reset_value(), "{} defaultInput", t.id)
            }
        }
        for (state, theirs) in states.iter().zip(row["states"].as_array().unwrap()) {
            let snapshot = if state == MISSING {
                &Value::Null
            } else {
                state
            };
            assert_eq!(
                json!(t.is_on(snapshot)),
                *theirs,
                "{} currentOn of {state}",
                t.id
            );
            compared += 1;
        }
    }
    assert!(compared > 300, "{compared} states compared");
    // The edges were reached: both tweaks Omarchy turns on read on for a value that is not false.
    for id in ["electronWayland", "forceZeroScaling"] {
        let t = tweaks::by_id(id).unwrap();
        assert!(t.is_on(&json!({ t.field(): "false" })), "{id}");
    }
}

/// The QML page asks `Tweaks.js`, so holding the Rust to `Tweaks.js` holds it to the page.
#[test]
fn the_qml_page_hands_its_logic_to_tweaks_js() {
    let page = read("pages/TweaksPage.qml");
    for call in [
        "TweaksJs.currentOn(tweak, Omarchy.hyprInput, Omarchy.tweaks)",
        "TweaksJs.writeFor(tweak, on)",
        "TweaksJs.resetWrite(tweak)",
        "Omarchy.writeHyprInput(patch)",
        "Omarchy.setTweak(tweak.id, write.value)",
    ] {
        assert!(
            page.contains(call),
            "TweaksPage.qml no longer says `{call}`"
        );
    }
    let omarchy = read("services/Omarchy.qml");
    assert!(
        omarchy.contains("dispatchSetting(\"tweaks.\" + String(id || \"\"), on === true)"),
        "Omarchy.setTweak changed: the glue in this test restates it"
    );
    for line in [
        "var input = HyprPrefs.clampInput(SnapshotJs.mergeSnapshot(hyprInput, patch))",
        "runSettingCommand(SettingsJs.commandFor(\"hyprInput.\" + field, input[field], snap, scriptOpts()), \"hyprInput.\" + field)",
    ] {
        assert!(
            omarchy.contains(line),
            "Omarchy.writeHyprInput changed: the glue in this test restates it"
        );
    }
}

/// Every domain the page writes is a setting the backend stores, of the type the page sends.
#[test]
fn every_written_domain_is_registered() {
    let domains = read("backend/src/domain.rs");
    for t in CATALOG {
        let store = match t.kind {
            Kind::HyprInput => "input",
            Kind::Tweak => "tweaks",
        };
        let ty = match t.on {
            Setting::Bool(_) => "Bool",
            Setting::Text(_) => "String",
        };
        let row = format!("(\"{}\", \"{store}\", {ty})", t.key);
        assert!(domains.contains(&row), "{row} is not a domain");
    }
}
