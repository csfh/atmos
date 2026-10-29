//! The System page's rows Atmos draws itself (`pages/system.rs`): the hostname, location and
//! coordinate checks, the version rows, the channel confirm and the Atmos check's summary are the
//! QML page's and its helpers' own code, run under Node. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::system;
use common::qml;
use serde_json::json;

#[test]
fn the_system_rows_are_the_qml_rows() {
    let src = std::fs::read_to_string(qml::repo().join("pages/SystemPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        format!("message: root.pendingChannel === \"dev\" ? {} : \"Switch the package channel to \" + root.pendingChannel + \". That rewrites pacman mirrors and upgrades the system.\"", json!(system::channel_message("dev"))),
        "onConfirmed: Omarchy.setOmarchyChannel(root.pendingChannel)".into(),
        "enabled: root.hostnameValid && root.hostnameParsed !== Omarchy.hostname onClicked: Omarchy.setHostname(root.hostnameParsed)".into(),
        "available: !Omarchy.atmosPackaged label: \"Channel\"".into(),
        "onClicked: Omarchy.checkOmarchyUpdate()".into(),
        "enabled: !Omarchy.jobBusy && Omarchy.atmosUpdateAvailable onClicked: atmosUpdateConfirm.ask()".into(),
        "onClicked: Omarchy.openPrinters()".into(),
        "onClicked: Omarchy.openCupsAdmin()".into(),
        "onClicked: Omarchy.setAboutBranding(\"image\")".into(),
        "onClicked: Omarchy.setAboutBranding(\"reset\")".into(),
        "enabled: !Omarchy.weatherAuto onClicked: Omarchy.clearWeatherLocation()".into(),
        "available: !Omarchy.weatherAuto && Omarchy.weatherLocation.length > 0".into(),
        "if (root.weatherCoordsValid) Omarchy.setWeatherCoordinates(root.weatherCoordsParsed)".into(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }
    let hosts = json!([
        "laptop",
        " my-host.lan ",
        "-x",
        "a..b",
        "x-",
        "a_b",
        "",
        "ab.c-d.e",
        "a".repeat(64)
    ]);
    let places = json!(["Oslo", " Bergen ", "-x", "", "a\nb", "x".repeat(129)]);
    let coords = json!([
        "59.9,10.7",
        " +59.9 , -10.70 ",
        "91,0",
        "0,181",
        "1.,2",
        "x",
        "1, 2, 3",
        "-0.5,+180"
    ]);
    let checks = json!([
        "status behind\nsummary 3 commits behind\n",
        "status current\n",
        "status fetch-failed\nsummary a/b\n",
        "status packaged\n",
        "summary -x\nstatus weird\n",
        ""
    ]);
    let versions = json!([["3.1.0", "stable"], ["3.1.0", ""], ["", "rc"]]);
    let atmos = json!([
        [true, "1.0.0-1", "abc1", true],
        [false, "", "abc1", true],
        [false, "", "", true],
        [false, "", "", false]
    ]);
    let script = format!(
        r#"{root}
const R = load("services/RichUi.js");
const A = load("services/AtmosUpdate.js");
const O = {{ omarchyVersion: "", omarchyChannel: "", atmosPackaged: false, atmosVersion: "", atmosRevision: "", atmosInstalled: false }};
const r = pageRoot("SystemPage.qml", O, {{}});
const src = require("fs").readFileSync(process.env.REPO_DIR + "/pages/SystemPage.qml", "utf8");
const after = (anchor) => src.split(anchor)[1];
const expr = (block, name) => block.split(name + ": ")[1].split(/\n\s+[a-zA-Z]+: /)[0];
const orow = after("title: \"Updates\"");
const arow = after("title: \"Atmos\"").split("SettingRow {{")[1];
const ev = (e) => new Function("Omarchy", "return (" + e + ");")(O);
process.stdout.write(JSON.stringify({{
  hosts: {hosts}.map(R.parseHostname),
  places: {places}.map(R.parseWeatherLocation),
  coords: {coords}.map(R.parseWeatherCoords),
  checks: {checks}.map((t) => {{ const p = A.parseCheckOutput(t); return [p.status, p.summary]; }}),
  versions: {versions}.map(([v, c]) => {{ Object.assign(O, {{ omarchyVersion: v, omarchyChannel: c }}); return [ev(expr(orow, "label")), ev(expr(orow, "description"))]; }}),
  atmos: {atmos}.map(([p, v, rev, inst]) => {{ Object.assign(O, {{ atmosPackaged: p, atmosVersion: v, atmosRevision: rev, atmosInstalled: inst }}); return [ev(expr(arow, "label")), ev(expr(arow, "description"))]; }}),
}}));
"#,
        root = qml::PAGE_ROOT,
        hosts = hosts,
        places = places,
        coords = coords,
        checks = checks,
        versions = versions,
        atmos = atmos,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let strs = |v: &serde_json::Value| {
        v.as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    let ours = json!({
        "hosts": strs(&hosts).iter().map(|h| system::parse_hostname(h)).collect::<Vec<_>>(),
        "places": strs(&places).iter().map(|p| system::parse_location(p)).collect::<Vec<_>>(),
        "coords": strs(&coords).iter().map(|c| system::parse_coords(c)).collect::<Vec<_>>(),
        "checks": strs(&checks).iter().map(|t| { let (s, m) = system::parse_check(t); json!([s, m]) }).collect::<Vec<_>>(),
        "versions": versions.as_array().unwrap().iter().map(|v| {
            let (l, s, _) = system::omarchy_row(v[0].as_str().unwrap(), v[1].as_str().unwrap());
            json!([l, s])
        }).collect::<Vec<_>>(),
        "atmos": atmos.as_array().unwrap().iter().map(|a| {
            let (l, s, _, _) = system::atmos_row(a[0].as_bool().unwrap(), a[1].as_str().unwrap(), a[2].as_str().unwrap(), a[3].as_bool().unwrap());
            json!([l, s])
        }).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);
}

/// Idle's Screensaver logo is the About logo's buttons, for the screensaver.
#[test]
fn the_screensaver_logo_row_is_the_qml_row() {
    let src = std::fs::read_to_string(qml::repo().join("pages/IdlePage.qml")).unwrap();
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        format!(
            "description: Omarchy.screensaverBranded ? {} : {}",
            json!(atmos_ui::system::screensaver_says(true)),
            json!(atmos_ui::system::screensaver_says(false))
        ),
        "text: \"Choose…\" onClicked: Omarchy.setScreensaverBranding(\"image\")".into(),
        "text: \"Edit\" onClicked: Omarchy.setScreensaverBranding(\"text\")".into(),
        "visible: Omarchy.screensaverBranded text: \"Reset\" danger: true enabled: Omarchy.screensaverBranded onClicked: Omarchy.setScreensaverBranding(\"reset\")".into(),
    ] {
        assert!(flat.contains(&want), "{want}");
    }
}
