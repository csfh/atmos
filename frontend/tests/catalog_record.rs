//! Record shortcut (`bindings::recorded`, `pages/bindings.rs`) and the Wi-Fi QR code on a tall
//! chart, both on what ODK gives (its listening `Button` and `ChartHeight`, ADR 0075 and 0076
//! there). A chord is `Bindings.js`'s: `recordChord` and `recordKeyEvent` run under Node for
//! each key ODK names, given as the Qt key the QML window reads. In the window, Record listens,
//! takes a chord and stops; Escape cancels; and the QR code is drawn far taller than a space.
//! Without `node` the Node test says so and stops.

mod common;

use atmos_ui::bindings;
use common::qml;
use common::window::{open, ordinary};
use odk::gpui::{TestAppContext, px, size};
use serde_json::json;

/// Each key as ODK names it, and the Qt key code and text the QML window reads for it.
const KEYS: &[(&str, u32, &str)] = &[
    ("a", 0x41, "a"),
    ("k", 0x4b, "k"),
    ("5", 0x35, "5"),
    ("space", 0x20, " "),
    (",", 0x2c, ","),
    (".", 0x2e, "."),
    ("tab", 0x0100_0001, "\t"),
    ("escape", 0x0100_0000, ""),
    ("enter", 0x0100_0004, "\r"),
    ("backspace", 0x0100_0003, ""),
    ("delete", 0x0100_0007, ""),
    ("left", 0x0100_0012, ""),
    ("up", 0x0100_0013, ""),
    ("right", 0x0100_0014, ""),
    ("down", 0x0100_0015, ""),
    ("f1", 0x0100_0030, ""),
    ("f12", 0x0100_003b, ""),
    ("f13", 0x0100_003c, ""),
    ("-", 0x2d, "-"),
    ("home", 0x0100_0010, ""),
];

/// Super, Ctrl, Alt and Shift as ODK reports them, and as Qt's modifier bits.
const MODS: &[(bool, bool, bool, bool, u32)] = &[
    (false, false, false, false, 0),
    (true, false, false, false, 0x1000_0000),
    (false, true, false, true, 0x0400_0000 | 0x0200_0000),
    (
        true,
        true,
        true,
        true,
        0x1000_0000 | 0x0400_0000 | 0x0800_0000 | 0x0200_0000,
    ),
];

#[test]
fn a_recorded_chord_is_bindings_js() {
    let parts = json!([
        ["Super_L", "x"],
        ["Ctrl", "Alt", ","],
        ["  a  b ", "Shift"],
        [],
        ["Meta", "."],
        ["SUPER", "bad|key"]
    ]);
    let mut events = Vec::new();
    for (_, code, text) in KEYS {
        for (.., bits) in MODS {
            events.push(json!({"key": code, "text": text, "modifiers": bits}));
        }
    }
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const B = load("services/Bindings.js");
process.stdout.write(JSON.stringify({{
  chords: {parts}.map((p) => B.recordChord(p)),
  events: {events}.map((e) => B.recordKeyEvent(e)),
}}));
"#,
        parts = parts,
        events = json!(events),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<String> = parts
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let p: Vec<&str> = p
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect();
            bindings::record_chord(&p)
        })
        .collect();
    assert_eq!(json!(ours), theirs["chords"]);
    let mut i = 0;
    for (key, ..) in KEYS {
        for (sup, ctrl, alt, shift, _) in MODS {
            assert_eq!(
                json!(bindings::recorded(key, *sup, *ctrl, *alt, *shift)),
                theirs["events"][i],
                "{key} {sup} {ctrl} {alt} {shift}"
            );
            i += 1;
        }
    }
    assert_eq!(
        bindings::recorded("k", true, false, false, true),
        "SUPER + SHIFT + K"
    );

    let src = std::fs::read_to_string(qml::repo().join("pages/windows/BindingsPage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "text: root.recordingKeys ? \"Listening…\" : \"Record shortcut\" primary: root.recordingKeys",
        "? \"Press the shortcut now. Super, Ctrl, Alt, and Shift count as modifiers.\" : \"Use the same chord form as bindings.lua, or press Record shortcut.\")",
        "placeholder: \"SUPER + F\" enabled: !root.recordingKeys",
        "if (!chord) { event.accepted = true return }",
    ] {
        assert!(flat.contains(want), "{want}");
    }
}

/// Record listens once pressed, takes the next chord into the dialog and stops; a key no binding
/// can take leaves it listening; Escape stops it.
#[test]
fn record_takes_the_next_chord() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "keybindings", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(4000.0)));
    w.sample(
        "settings.host",
        1,
        json!({"keybindings": [], "keybindingsSource": "hyprctl", "bindingOverrides": [], "favorites": []}),
    );
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    catalog.update(w.cx, |p, cx| atmos_ui::pages::bindings::open_add(p, "", cx));
    w.until("the dialog", |w| w.drawn("bind-record/button"));
    w.settle();
    let chord = |w: &mut common::window::Win| {
        catalog.read_with(w.cx, |p, _| atmos_ui::pages::bindings::chord(p))
    };
    w.click("bind-record/button");
    assert_eq!(chord(&mut w), (String::new(), true), "it listens");
    w.cx.simulate_keystrokes("home");
    w.settle();
    assert_eq!(
        chord(&mut w),
        (String::new(), true),
        "no binding takes Home: still listening"
    );
    w.cx.simulate_keystrokes("ctrl-shift-k");
    w.settle();
    assert_eq!(chord(&mut w), ("CTRL + SHIFT + K".to_owned(), false));
    w.click("bind-record/button");
    w.cx.simulate_keystrokes("escape");
    w.settle();
    assert_eq!(
        chord(&mut w),
        ("CTRL + SHIFT + K".to_owned(), false),
        "Escape stops it"
    );
}

/// The Wi-Fi QR code is drawn on a tall chart, far taller than the largest space. Its colours
/// follow the theme, light on dark in a dark one; no phone has read it in a test.
#[test]
fn the_wifi_qr_code_is_drawn_on_a_tall_chart() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "network/wifi", ordinary);
    w.cx.simulate_resize(size(px(1400.0), px(9000.0)));
    w.sample(
        "settings.network",
        1,
        json!({"wifiHw": true, "wifiRadio": false, "wifiConnected": true, "wifiIface": "wlan0",
            "wifiConnections": [], "netSsid": "Home"}),
    );
    let show = "ctl-network/wifi/qr-code-show/button";
    w.until("Show QR", |w| w.drawn(show));
    w.settle();
    w.click(show);
    w.until("the job", |w| !w.daemon.of("job.start").is_empty());
    let cid = w.daemon.of("job.start")[0]["cid"].clone();
    let rows: Vec<String> = (0..25)
        .map(|i| {
            if i % 2 == 0 {
                "1".repeat(25)
            } else {
                "0".repeat(25)
            }
        })
        .collect();
    w.daemon
        .say(json!({"event": "job", "kind": "started", "cid": cid, "job": 7}));
    for (seq, line) in std::iter::once("meta\twlan0\tWPA\tHome".to_owned())
        .chain(rows)
        .enumerate()
    {
        w.daemon.say(
            json!({"event": "job", "kind": "stream", "cid": cid, "job": 7, "seq": seq,
            "fd": "out", "line": line}),
        );
    }
    w.daemon.say(
        json!({"event": "job", "kind": "exit", "cid": cid, "job": 7, "code": 0,
        "summary": {"ok": true, "message": "", "warnings": []}}),
    );
    let code = "ctl-network/wifi/qr-code-code/chart";
    w.until("the code", |w| w.drawn(code));
    let tall = w.bounds(code).unwrap().size.height;
    assert!(tall >= px(150.0), "{tall:?}");
}
