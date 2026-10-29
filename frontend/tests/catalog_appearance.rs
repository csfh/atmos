//! The Appearance page's rows the page module draws (`pages/appearance.rs`): their texts and
//! what they call are the QML page's, and what they build from what the window holds is its own
//! JavaScript, run under Node. What a setter then sends is `catalog_setters.rs`'s. Without
//! `node` the tests say so and stop.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::appearance as look;
use atmos_ui::catalog;
use common::qml;
use rows::page_src;
use serde_json::{Value, json};

#[test]
fn the_appearance_rows_are_the_qml_rows() {
    let page = catalog::page("appearance").unwrap();
    let src = page_src(page);
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    let says = |valid: bool| {
        format!(
            "description: root.nightTimesValid ? {} : {}",
            json!(look::schedule_says(true)),
            json!(look::schedule_says(false))
        )
        .replace("{valid}", &valid.to_string())
    };
    let use_says = format!(
        "description: root.nightTimesValid ? {} : {}",
        json!(look::use_schedule_says(true)),
        json!(look::use_schedule_says(false))
    );
    for want in [
        "available: Omarchy.themes.length > 0".to_owned(),
        "if (modelData !== Omarchy.theme) Omarchy.setTheme(modelData)".to_owned(),
        "available: Omarchy.extraThemes.length === 0 label: \"Installed themes\" description: \"No extra themes installed.\"".to_owned(),
        "available: Omarchy.extraThemes.length > 0 label: \"Installed themes\" description: root.extraCountText() + \" Pick one to apply or remove, or pull the latest commit on all of them. Hover a name to preview its colors on this window.\"".to_owned(),
        "value: root.extraToRemove options: Omarchy.extraThemes enabled: Omarchy.extraThemes.length > 0 onChanged: function(value) { root.extraToRemove = value if (value !== Omarchy.theme) Omarchy.setTheme(value)".to_owned(),
        "text: Omarchy.jobKind === \"theme-update\" && Omarchy.jobBusy ? \"Updating…\" : \"Update all\" enabled: !Omarchy.jobBusy && Omarchy.extraThemes.length > 0 onClicked: Omarchy.updateThemes()".to_owned(),
        "text: \"Remove…\" danger: true enabled: root.extraToRemove.length > 0".to_owned(),
        "title: \"Remove this theme\"".to_owned(),
        "message: \"Delete \" + root.extraToRemove + \" from your extra themes? The files under ~/.config/omarchy/themes go with it.\"".to_owned(),
        "onConfirmed: Omarchy.removeTheme(root.extraToRemove)".to_owned(),
        format!("description: Omarchy.jobKind === \"theme-install\" && Omarchy.jobBusy ? {} : {}", json!(look::INSTALLING), json!(look::ADD_SAYS)),
        "text: \"Add…\" primary: true enabled: !Omarchy.jobBusy onClicked: root.openAddTheme()".to_owned(),
        "title: \"Add a theme\"".to_owned(),
        format!("text: Omarchy.jobKind === \"theme-install\" && Omarchy.jobBusy ? {} : (root.themeUrlDraft.length > 0 && !root.themeUrlValid ? {} : {})", json!(look::INSTALLING), json!(look::BAD_URL), json!(look::ADD_INTRO)),
        "placeholder: \"https://github.com/org/omarchy-theme.git\" enabled: !Omarchy.jobBusy invalid: root.themeUrlDraft.length > 0 && !root.themeUrlValid".to_owned(),
        "text: \"Install\" primary: true enabled: !Omarchy.jobBusy && root.themeUrlValid onClicked: root.submitAddTheme()".to_owned(),
        says(true),
        "placeholder: \"07:00\" invalid: root.dayDraft.length > 0 && root.dayParsed.length === 0".to_owned(),
        "placeholder: \"20:00\" invalid: root.nightDraft.length > 0 && root.nightParsed.length === 0".to_owned(),
        "text: \"Set\" enabled: root.nightTimesValid onClicked: root.applyNightSchedule(Omarchy.nightlightNightOn)".to_owned(),
        use_says,
        "checked: Omarchy.nightlightNightOn enabled: root.nightTimesValid onToggled: root.applyNightSchedule(!Omarchy.nightlightNightOn)".to_owned(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }
    assert_eq!(
        look::remove_message("tokyo"),
        "Delete tokyo from your extra themes? The files under ~/.config/omarchy/themes go with it."
    );

    let urls = json!([
        "https://github.com/org/omarchy-tokyo-theme.git",
        "  HTTPS://github.com/org/Nord.git ",
        "git@github.com:org/omarchy-dune-theme",
        "ssh://git@host/x/rose-pine",
        "ext::sh -c touch",
        "-oProxyCommand=x",
        "github.com/org/repo",
        "/local/path",
        "foo:bar/.git",
        "https://h/a b",
        "nope://host/x",
        "file:///srv/themes/sand.git",
        "c:theme",
        "",
    ]);
    let times = json!([
        "7:00", "07:5", "23:59", "24:00", " 9:30 ", "19:60", "1:2:3", "", "08:05"
    ]);
    let extras = json!([[], ["tokyo"], ["tokyo", "nord", "dune"]]);
    let picks = json!(["nord", "gone", ""]);
    let script = format!(
        r#"{root}
const RichUi = load("services/RichUi.js");
const Sunset = load("services/HyprSunset.js");
let installed = null;
const O = {{ theme: "tokyo", themes: [], extraThemes: [], jobBusy: false, jobKind: "",
  nightlightDay: "07:00", nightlightNight: "20:00", nightlightNightOn: false,
  installTheme: (u) => {{ installed = u; }} }};
const r = pageRoot("AppearancePage.qml", O, {{}});
globalThis.addThemeDialog = {{ open() {{}}, close() {{}} }};
globalThis.themeUrlField = {{ setText() {{}} }};
process.stdout.write(JSON.stringify([
  {urls}.map((u) => RichUi.parseGitUrl(u)),
  {urls}.map((u) => {{ installed = null; r.themeUrlDraft = u; r.submitAddTheme(); return installed; }}),
  {times}.map((t) => Sunset.parseTime(t)),
  {extras}.map((list) => {{ O.extraThemes = list; return r.extraCountText(); }}),
  {extras}.map((list) => {{
    O.extraThemes = list;
    return {picks}.map((p) => {{ r.extraToRemove = p; r.syncExtraToRemove(); return r.extraToRemove; }});
  }}),
]));
"#,
        root = qml::PAGE_ROOT,
        urls = urls,
        times = times,
        extras = extras,
        picks = picks
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let strs = |v: &Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect()
    };
    let ours = json!([
        strs(&urls)
            .iter()
            .map(|u| look::parse_git_url(u))
            .collect::<Vec<_>>(),
        strs(&urls)
            .iter()
            .map(|u| {
                let p = look::parse_git_url(u);
                if p.is_empty() { Value::Null } else { json!(p) }
            })
            .collect::<Vec<_>>(),
        strs(&times)
            .iter()
            .map(|t| look::parse_time(t))
            .collect::<Vec<_>>(),
        extras
            .as_array()
            .unwrap()
            .iter()
            .map(|l| look::extra_count(l.as_array().unwrap().len()))
            .collect::<Vec<_>>(),
        extras
            .as_array()
            .unwrap()
            .iter()
            .map(|l| {
                let list = look::names(Some(l));
                strs(&picks)
                    .iter()
                    .map(|p| look::picked(&list, p))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    ]);
    assert_eq!(ours, theirs);
}

/// Background's current wallpaper row is the QML row's label and words.
#[test]
fn the_wallpaper_row_is_the_qml_row() {
    let src =
        std::fs::read_to_string(qml::repo().join("pages/appearance/BackgroundPage.qml")).unwrap();
    let row = src.split("SettingRow {").nth(1).unwrap();
    let (label, says) = (
        rows::property(row, "label"),
        rows::property(row, "description"),
    );
    let paths = [
        "",
        "/home/u/.config/omarchy/current/theme/backgrounds/1-a.jpg",
        "plain.png",
    ];
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const RichUi = load("services/RichUi.js");
process.stdout.write(JSON.stringify({paths}.map((background) => {{
  const Omarchy = {{ background }};
  return [{label}, {says}];
}})));
"#,
        paths = json!(paths),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<_> = paths
        .iter()
        .map(|p| atmos_ui::appearance::wallpaper_row(p))
        .collect();
    assert_eq!(json!(ours), theirs);
}
