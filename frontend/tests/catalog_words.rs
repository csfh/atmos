//! Each row whose words the QML builds from what the window holds (`Line::words`) says what its
//! QML `description:` says, run under Node in its page's scope, for snapshots, running jobs and
//! diagnostics that take every branch. Without `node` the test says so and stops.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::catalog::{self, words::Said};
use atmos_ui::settings::Settings;
use common::qml;
use serde_json::{Value, json};

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

fn docs() -> Vec<Value> {
    let ours = |n: usize| -> Vec<Value> {
        (0..n)
            .map(|i| json!({"keys": format!("SUPER + {i}"), "managed": true}))
            .collect()
    };
    vec![
        json!({"bluetooth": true, "bluetoothDevices": [{}, {}], "tailscaleRunning": true, "tailscaleInstalled": true, "wifiBand": "5", "wifiHw": true, "wifiRadio": true, "netKind": "wifi", "netSsid": "Home", "background": "/home/u/.config/omarchy/backgrounds/x/1-a.jpg", "plymouth": "omarchy",
            "nightlightTemperature": 4000, "bindings": ours(2), "keybindings": [{"keys": "A"}, {"keys": "B"}, {"keys": "C"}],
            "windowRules": [{"match": "a", "managed": true}], "reminderCount": 1, "autostart": [{"command": "a"}],
            "sshdActive": true, "sshdEnabled": true}),
        json!({"bluetooth": true, "bluetoothDevices": [{}], "tailscaleRunning": false, "tailscaleInstalled": true, "wifiBand": "", "wifiHw": true, "wifiRadio": true, "netKind": "ethernet", "netSsid": "", "background": "", "plymouth": "", "nightlightTemperature": 0, "bindings": ours(1), "keybindings": [],
            "windowRules": [], "reminderCount": 3, "autostart": [], "sshdActive": false, "sshdEnabled": true}),
        json!({"bluetooth": true, "bluetoothDevices": [], "tailscaleRunning": false, "tailscaleInstalled": false, "wifiBand": "2.4", "wifiHw": true, "wifiRadio": false, "netKind": "wifi", "netSsid": "", "background": "plain.png", "plymouth": "x", "nightlightTemperature": 6500, "bindings": [],
            "keybindings": [{"keys": "A"}], "windowRules": [{"match": "a", "managed": true}, {"match": "b", "managed": true}, {"match": "c"}],
            "reminderCount": 0, "autostart": [{"command": "a"}, {"command": "b"}], "sshdActive": false, "sshdEnabled": false}),
        json!({"bluetooth": false, "bluetoothDevices": [], "tailscaleRunning": false, "tailscaleInstalled": false, "wifiBand": "", "wifiHw": false, "wifiRadio": false, "netKind": "", "netSsid": "x", "background": "a/b/", "plymouth": "", "nightlightTemperature": -5, "bindings": [{"keys": "X"}],
            "keybindings": [{"keys": "A"}, {"keys": "B"}], "windowRules": [{"match": "a"}], "reminderCount": "2",
            "autostart": [], "sshdActive": false, "sshdEnabled": false}),
    ]
}

const JOBS: &[Option<&str>] = &[
    None,
    Some("snapshot-create"),
    Some("reset-atmos"),
    Some("refresh-hyprland"),
    Some("theme-install"),
];

fn diagnostics() -> Vec<Value> {
    vec![
        json!({"failedUnits": [{"unit": "a"}]}),
        json!({"failedUnits": [{"unit": "a"}, {"unit": "b"}]}),
        json!({"hyprland": {"configErrors": ["bad"]}}),
        json!({}),
    ]
}

#[test]
fn each_row_s_words_are_its_qml_description() {
    let mut rows_ = Vec::new();
    let mut ours = Vec::new();
    for page in catalog::pages() {
        for line in page.lines() {
            let Some(words) = line.words else { continue };
            let row = rows::qml_row(page, line.label, rows::nth(page, line));
            let expr = rows::property(&row, "description");
            rows_.push(json!({"at": format!("{} / {}", page.id, line.label), "file": page.file, "expr": expr}));
            for doc in docs() {
                let s = settings(&doc);
                for job in JOBS {
                    for diag in diagnostics() {
                        let said = Said {
                            settings: &s,
                            job: *job,
                            inventory: Some(&diag),
                        };
                        ours.push(json!(words.say(&said)));
                    }
                }
            }
        }
    }
    assert!(rows_.len() >= 13, "every row with words is found");
    let script = format!(
        r#"{root}
const Snap = load("services/Snapshot.js");
const RichUi = load("services/RichUi.js");
const HyprSunset = load("services/HyprSunset.js");
const AtmosUpdate = load("services/AtmosUpdate.js");
const out = [];
for (const row of {rows})
  for (const doc of {docs})
    for (const job of {jobs})
      for (const diag of {diags}) {{
        const O = {{ jobKind: job || "", jobBusy: !!job, diagnostics: diag }};
        for (const [key, raw] of Object.entries(doc))
          O[key] = Snap.clampPatchedValue(key, raw, {{
            parseTime: HyprSunset.parseTime, parseChannel: AtmosUpdate.parseChannel,
            parseWeatherCoords: RichUi.parseWeatherCoords,
          }});
        const r = pageRoot(row.file, O, {{}});
        out.push([row.at, job, new Function("root", "Omarchy", "RichUi", "return (" + row.expr + ");")(r, O, RichUi)]);
      }}
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        rows = json!(rows_),
        docs = json!(docs()),
        jobs = json!(JOBS),
        diags = json!(diagnostics()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let theirs = theirs.as_array().unwrap();
    assert_eq!(ours.len(), theirs.len());
    for (ours, theirs) in ours.iter().zip(theirs) {
        assert_eq!(ours, &theirs[2], "{} while {} runs", theirs[0], theirs[1]);
    }
}
