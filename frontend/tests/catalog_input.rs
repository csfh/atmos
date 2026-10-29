//! Input's Hyprland layouts rows (`pages/keyboard.rs`): the lists as `HyprPrefs.js` sanitizes
//! them, the input block as it clamps it, the page's drafts and Set as `InputPage.qml` reads
//! them, and the revert `kbOverridePlan` hands the guard, run under Node. A revert writes back
//! through `writeHyprKbOverride`, as `dispatchRevert` does. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::guard;
use atmos_ui::keyboard::{self, Draft};
use common::qml;
use serde_json::{Value, json};

const LAYOUTS: &[&str] = &[
    "us",
    " US , dk ",
    "us,,dk",
    "toolongname",
    "us,dé",
    "",
    "  ",
    "a1,b2,c3",
];
const VARIANTS: &[(&str, usize)] = &[
    (",nodeadkeys", 2),
    ("intl", 1),
    ("intl", 2),
    (" , ", 2),
    ("bad variant", 1),
    ("", 2),
    ("x", 0),
];

fn raws() -> Vec<Value> {
    vec![
        json!({"sensitivity": 2, "kbLayoutOverride": "US,dk", "kbVariantOverride": ",nodeadkeys", "kbGroupToggle": "on", "workspaceGesture": 1}),
        json!({"kbLayoutOverride": "bad!", "kbVariantOverride": "x"}),
        json!({"kbLayoutOverride": "us", "kbVariantOverride": "a,b"}),
        json!(null),
    ]
}

#[test]
fn the_lists_and_the_block_are_hypr_prefs_js() {
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const H = load("services/HyprPrefs.js");
process.stdout.write(JSON.stringify({{
  layouts: {layouts}.map((l) => H.sanitizeLayoutList(l)),
  variants: {variants}.map(([v, n]) => H.sanitizeVariantList(v, n)),
  inputs: {raws}.map((r) => H.clampInput(r)),
  reverts: {raws}.map((r) => H.kbOverridePlan(H.clampInput(r), "fr", "", true, (a, b) => Object.assign({{}}, a, b)).revert),
}}));
"#,
        layouts = json!(LAYOUTS),
        variants = json!(VARIANTS),
        raws = json!(raws()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours = json!({
        "layouts": LAYOUTS.iter().map(|l| keyboard::sanitize_layouts(l)).collect::<Vec<_>>(),
        "variants": VARIANTS.iter().map(|(v, n)| keyboard::sanitize_variants(v, *n)).collect::<Vec<_>>(),
        "inputs": raws().iter().map(|r| keyboard::input(Some(r))).collect::<Vec<_>>(),
        "reverts": raws().iter().map(|r| keyboard::write(&keyboard::input(Some(r)), ("fr", "", true), false).1).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);
}

/// The page's drafts, what its two rows say, and what Set and Enter hand the setter.
#[test]
fn the_drafts_are_the_qml_page_s() {
    let drafts = [
        ("us,dk", ",nodeadkeys"),
        ("US", ""),
        ("us,dk", "x"),
        ("bad layout", ""),
        ("", "intl"),
        ("us,dk", ""),
        ("", ""),
    ];
    let raw = json!({"kbLayoutOverride": "us,dk", "kbVariantOverride": ",nodeadkeys", "kbGroupToggle": true});
    let script = format!(
        r#"{root}
const H = load("services/HyprPrefs.js");
const out = {drafts}.map(([l, v]) => {{
  const O = {{ hyprInput: H.clampInput({raw}), calls: [], setHyprKbOverride: (...a) => O.calls.push(a) }};
  const r = pageRoot("InputPage.qml", O, {{ kbLayoutDraft: l, kbVariantDraft: v }});
  r.applyKbOverride();
  return [r.kbLayoutParsed, r.kbVariantParsed, r.kbLayoutValid, r.kbVariantValid, r.kbOverrideValid, r.kbOverrideDirty, O.calls];
}});
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        drafts = json!(drafts),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let input = keyboard::input(Some(&raw));
    let ours: Vec<Value> = drafts
        .iter()
        .map(|(l, v)| {
            let d = Draft::of(&input, l, v);
            let calls = if d.valid() {
                json!([[d.layouts, d.variants, true]])
            } else {
                json!([])
            };
            json!([
                d.layouts,
                d.variants,
                d.layouts_ok,
                d.variants_ok,
                d.valid(),
                d.dirty,
                calls
            ])
        })
        .collect();
    assert_eq!(json!(ours), theirs);

    let src = std::fs::read_to_string(qml::repo().join("pages/InputPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    let (ok, bad) = (Draft::of(&input, "", ""), Draft::of(&input, "!", "!"));
    for want in [
        format!(
            "description: root.kbLayoutValid ? {} : {}",
            json!(ok.layouts_says()),
            json!(bad.layouts_says())
        ),
        format!(
            "description: root.kbVariantValid ? {} : {}",
            json!(ok.variants_says()),
            json!(bad.variants_says())
        ),
        "placeholder: \"us,dk\" invalid: !root.kbLayoutValid onEdited: function(value) { root.kbLayoutDraft = value } onSubmitted: function(value) { root.kbLayoutDraft = value root.applyKbOverride() }".into(),
        "placeholder: \",nodeadkeys\" invalid: !root.kbVariantValid onEdited: function(value) { root.kbVariantDraft = value } onSubmitted: function(value) { root.kbVariantDraft = value root.applyKbOverride() }".into(),
        "text: \"Set\" primary: true enabled: root.kbOverrideValid && root.kbOverrideDirty onClicked: root.applyKbOverride()".into(),
        "checked: Omarchy.hyprInput.kbGroupToggle onToggled: { if (root.kbOverrideValid) Omarchy.setHyprKbOverride(root.kbLayoutParsed, root.kbVariantParsed, Omarchy.hyprInput.kbGroupToggle !== true) else Omarchy.setHyprKbOverride(Omarchy.hyprInput.kbLayoutOverride, Omarchy.hyprInput.kbVariantOverride, Omarchy.hyprInput.kbGroupToggle !== true) }".into(),
        "function onHyprInputChanged() { root.kbLayoutDraft = Omarchy.hyprInput.kbLayoutOverride || \"\" root.kbVariantDraft = Omarchy.hyprInput.kbVariantOverride || \"\" }".into(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }

    // Alt+Alt flips the switch with the drafts while they are valid, else with what is set.
    let set = |l: &str, v: &str, t: bool| keyboard::set(&input, (l, v, t), false);
    assert_eq!(
        Draft::of(&input, "fr", "").toggle(&input, false),
        set("fr", "", false)
    );
    assert_eq!(
        Draft::of(&input, "!", "").toggle(&input, false),
        set("us,dk", ",nodeadkeys", false)
    );
}

/// The bar's revert is `kbOverridePlan`'s, it opens as `kbRunOptions` says, and Revert writes
/// what `dispatchRevert` writes.
#[test]
fn a_revert_writes_the_lists_back() {
    let om = std::fs::read_to_string(qml::repo().join("services/Omarchy.qml")).unwrap();
    let prefs = std::fs::read_to_string(qml::repo().join("services/HyprPrefs.js")).unwrap();
    for (src, want) in [
        (
            &prefs,
            "guard: reverting ? null : { id: \"kbOverride\", revert: revert },",
        ),
        (
            &om,
            "root.writeHyprKbOverride(desc.layouts, desc.variants, desc.groupToggle === true)",
        ),
    ] {
        assert!(src.contains(want), "{want}");
    }
    let raw = json!({"kbLayoutOverride": "us", "kbVariantOverride": "", "kbGroupToggle": false});
    let input = keyboard::input(Some(&raw));
    let (_, revert) = keyboard::set(&input, ("fr,us", "azerty,", true), false).unwrap();
    let values = json!({"hyprInput": {"kbLayoutOverride": "fr,us", "kbVariantOverride": "azerty,", "kbGroupToggle": true}});
    let shown = |k: &str| values.get(k).cloned();
    let (key, body) = guard::revert_write(&revert, &shown).unwrap();
    let now = keyboard::input(values.get("hyprInput"));
    assert_eq!(key, "hyprInput");
    assert_eq!(body, keyboard::write(&now, ("us", "", false), false).0);
}
