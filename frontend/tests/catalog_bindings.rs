//! The Keybindings page (`pages/bindings.rs`): the chords, labels and commands `RowText.js` keeps,
//! the categories, conflicts and lines `Bindings.js` makes, the copy text `RichUi.js` makes, and
//! the page's own lists, words and Add, run under Node. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::bindings::{self, Draft};
use common::qml;
use serde_json::{Value, json};

fn bound() -> Value {
    json!([
        {"keys": "SUPER + Q", "action": "Close window"},
        {"keys": "SUPER + RETURN", "action": "Terminal"},
        {"keys": "SUPER + 1", "action": "Switch to workspace 1"},
        {"keys": "XF86AudioRaiseVolume", "action": "Volume up"},
        {"keys": "PRINT", "action": "Screenshot"},
        {"keys": "SUPER + L", "action": "Lock system"},
        {"keys": "SUPER + J", "action": "Toggle split"},
        {"keys": "SUPER + K"},
        {"action": "no chord"},
        null,
    ])
}

fn mine() -> Value {
    json!([
        {"keys": "SUPER + F", "label": "Files", "command": "nautilus", "managed": true},
        {"keys": "SUPER + U", "command": "", "unbind": true, "managed": true},
        {"keys": "SUPER + B", "command": "firefox"},
        {"label": "no chord"},
    ])
}

#[test]
fn the_texts_are_row_text_bindings_and_rich_ui() {
    let keys = [
        "SUPER + F",
        "  super  +  f ",
        "SUPER + ä",
        "a\nb",
        "",
        &"X".repeat(65),
    ];
    let labels = ["Files", "  x ", "a\rb", "", &"y".repeat(65)];
    let commands = [" nautilus ", "a\nb", "", &"z".repeat(257)];
    let actions = [
        "Switch to workspace 3",
        "Launch browser",
        "Mute audio",
        "Screenshot region",
        "Lock",
        "Close window",
        "Something",
        "",
    ];
    let rows = [
        ("SUPER + F", "Files", "nautilus", false),
        ("SUPER + Q", "", "kitty", true),
        ("SUPER + U", "", "", true),
        ("bad!", "", "x", false),
        ("SUPER + X", "", "", false),
        ("SUPER + \"", "a\"b", "echo \\ \t", false),
    ];
    let copies = json!([{"keys": "SUPER + Q", "action": "Close\nwindow"}, {"keys": "A", "command": "A"}, {"label": "only"}, {"keys": "K"}, {}]);
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const B = load("services/Bindings.js");
const R = load("services/RichUi.js");
process.stdout.write(JSON.stringify({{
  keys: {keys}.map((k) => B.sanitizeKeys(k)),
  labels: {labels}.map((k) => B.sanitizeLabel(k)),
  commands: {commands}.map((k) => B.sanitizeCommand(k)),
  categories: {actions}.map((a) => B.categoryFromAction(a)),
  conflicts: {keys}.map((k) => B.catalogConflict({bound}, k)).concat([B.catalogConflict({bound}, " super + q "), B.catalogConflict(null, "SUPER + Q")]),
  lines: {rows}.map(([keys, label, command, unbind]) => B.generatedBindText({{ keys, label, command, unbind }})),
  copies: {copies}.map((r) => R.bindingCopyText(r)),
}}));
"#,
        keys = json!(keys),
        labels = json!(labels),
        commands = json!(commands),
        actions = json!(actions),
        bound = bound(),
        rows = json!(rows),
        copies = copies,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let b = bound();
    let mut conflicts: Vec<String> = keys
        .iter()
        .map(|k| bindings::conflict(Some(&b), k))
        .collect();
    conflicts.push(bindings::conflict(Some(&b), " super + q "));
    conflicts.push(bindings::conflict(None, "SUPER + Q"));
    let ours = json!({
        "keys": keys.iter().map(|k| bindings::sanitize_keys(k)).collect::<Vec<_>>(),
        "labels": labels.iter().map(|k| bindings::sanitize_label(k)).collect::<Vec<_>>(),
        "commands": commands.iter().map(|k| bindings::sanitize_command(k)).collect::<Vec<_>>(),
        "categories": actions.iter().map(|a| bindings::category(a)).collect::<Vec<_>>(),
        "conflicts": conflicts,
        "lines": rows.iter().map(|(k, l, c, u)| bindings::generated(k, l, c, *u)).collect::<Vec<_>>(),
        "copies": copies.as_array().unwrap().iter().map(bindings::copy_text).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);
}

/// The page's lists, what each row says, and what Add hands `addBinding`.
#[test]
fn the_lists_and_add_are_the_qml_page_s() {
    let filters = ["", "super + q", "WORKSPACE", "media", "nothing"];
    let adds = [
        ("SUPER + T", "Term", "alacritty", false),
        ("SUPER + Q", "", "kitty", false),
        ("", "", "x", false),
        ("SUPER + X", "", "", false),
        ("SUPER + X", "  ", "", true),
        ("bad!", "", "x", false),
    ];
    let script = format!(
        r#"{root}
const O = {{ keybindings: {bound}, bindings: {mine}, added: [], addBinding: (...a) => O.added.push(a) }};
const lists = {filters}.map((f) => pageRoot("windows/BindingsPage.qml", O, {{ catalogFilter: f }}).catalogRows);
const r = pageRoot("windows/BindingsPage.qml", O, {{}});
const describe = r.overrideRows.map((row) => r.describeOverride(row));
const submits = {adds}.map(([k, l, c, u]) => {{
  globalThis.keysField = {{ currentText: () => k }};
  globalThis.labelField = {{ currentText: () => l }};
  globalThis.commandField = {{ currentText: () => c }};
  globalThis.addDialog = {{ close: () => {{}} }};
  r.unbindOnly = u;
  r.addError = "";
  O.added = [];
  r.submitAdd();
  return [r.addError, O.added];
}});
process.stdout.write(JSON.stringify({{ lists, overrides: r.overrideRows, describe, submits }}));
"#,
        root = qml::PAGE_ROOT,
        bound = bound(),
        mine = mine(),
        filters = json!(filters),
        adds = json!(adds),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let (b, m) = (bound(), mine());
    let overrides = bindings::overrides(Some(&m));
    let submits: Vec<Value> = adds
        .iter()
        .map(|(k, l, c, u)| {
            let d = Draft {
                keys: (*k).into(),
                label: (*l).into(),
                command: (*c).into(),
                unbind: *u,
            };
            match d.submit(Some(&b)) {
                Ok((k, l, c, u)) => json!(["", [[k, l, c, u]]]),
                Err(why) => json!([why, []]),
            }
        })
        .collect();
    let ours = json!({
        "lists": filters.iter().map(|f| bindings::catalog(Some(&b), f)).collect::<Vec<_>>(),
        "overrides": overrides,
        "describe": overrides.iter().map(bindings::describe).collect::<Vec<_>>(),
        "submits": submits,
    });
    assert_eq!(ours, theirs);

    let src = std::fs::read_to_string(qml::repo().join("pages/windows/BindingsPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        format!("property int catalogLimit: {}", bindings::LIMIT),
        "description: (modelData && modelData.category ? modelData.category + \" · \" : \"\") + (modelData && modelData.action ? modelData.action : \"\")".into(),
        "description: root.catalogRows.length + \" of \" + Omarchy.keybindings.length + \" bindings.\"".into(),
        "description: \"Showing \" + Math.min(root.catalogLimit, root.catalogRows.length) + \" of \" + root.catalogRows.length + \" bindings. Refine the filter, or show them all.\"".into(),
        "text: \"Remove…\" danger: true enabled: modelData && modelData.managed".into(),
        "message: \"Remove the Atmos override for \" + root.pendingKeys + \"?\" confirmText: \"Remove\" onConfirmed: Omarchy.removeBinding(root.pendingKeys)".into(),
        "text: \"Copy\" enabled: !!(modelData && modelData.keys) onClicked: Omarchy.copyText(RichUi.bindingCopyText(modelData))".into(),
        "text: \"Override…\" enabled: !!(modelData && modelData.keys) onClicked: root.openAdd(modelData.keys)".into(),
        "available: root.overrideRows.length === 0".into(),
        "available: Omarchy.keybindings.length === 0".into(),
        "available: Omarchy.keybindings.length > 0 && root.catalogRows.length === 0".into(),
        "(root.keysDraft + \" already runs “\" + root.conflictText() + \"”. Add will unbind that first.\")".into(),
        ": \"Use the same chord form as bindings.lua, or press Record shortcut.\")".into(),
        "unbind: root.unbindOnly || !!root.conflictText() }) || \"The Hyprland line appears here once the chord is valid.\"".into(),
        "enabled: root.keysDraft.length > 0 && (root.unbindOnly || root.commandDraft.length > 0)".into(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }
    assert_eq!(bindings::filter_says(3, 8), "3 of 8 bindings.");
    assert_eq!(
        bindings::more_says(100, 250),
        "Showing 100 of 250 bindings. Refine the filter, or show them all."
    );
    let d = Draft {
        keys: "SUPER + Q".into(),
        ..Draft::default()
    };
    assert_eq!(
        d.says(Some(&b), false),
        "SUPER + Q already runs “Close window”. Add will unbind that first."
    );
    assert_eq!(d.line(Some(&b)), "hl.unbind(\"SUPER + Q\")");
}
