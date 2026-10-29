//! The Diagnostics page (`pages/diagnostics.rs`): `Diagnostics.js`'s `normalize`, its summaries and
//! report, and the page's stale-settings row, run under Node on the same inventories as
//! `diagnostics.rs`. Without `node` the test says so and stops.

mod common;

use atmos_ui::catalog::samples::DIAGNOSTICS;
use atmos_ui::diagnostics as diag;
use common::qml;
use serde_json::{Value, json};

#[test]
fn the_diagnostics_rows_are_the_qml_rows() {
    let inventories = json!([*DIAGNOSTICS, {"disk": {"percent": 140}, "network": {"online": true, "kind": "lte"}, "pacman": {"syncOk": true}}, null]);
    let failed = json!([
        [],
        [["a", "tool failed"]],
        (0..14)
            .map(|i| json!([format!("k{i:02}"), "x".repeat(150)]))
            .collect::<Vec<_>>()
    ]);
    let script = format!(
        r#"process.env.TZ = "UTC";
{root}
const D = load("services/Diagnostics.js");
process.stdout.write(JSON.stringify({{
  each: {inventories}.map((raw) => {{
    const d = D.normalize(raw);
    return {{ d, said: [D.omarchySummary(d), D.atmosSummary(d), D.hyprSummary(d), D.kernelSummary(d), D.gpuSummary(d),
      D.networkSummary(d), D.pacmanSummary(d), D.serviceMapSummary(d.portals), D.serviceMapSummary(d.pipewire)],
      ok: [D.hyprOk(d), D.servicesOk(d.portals), D.servicesOk(d.pipewire), D.diskPressure(d), D.memoryPressure(d), D.atmosOk(d)],
      report: D.reportText(raw) }};
  }}),
  names: [["box", 1793174400000], ["my host!", 1793230140000], ["", 0]].map(([h, t]) => D.reportFileName(h, new Date(t))),
  stale: {failed}.map((list) => {{
    const failedMap = {{}};
    for (const [k, v] of list) failedMap[k] = v;
    const r = pageRoot("system/DiagnosticsPage.qml", {{ snapshotFailed: failedMap, diagnostics: {{}}, jobKind: "", jobBusy: false }}, {{}});
    const n = r.failedKeys.length;
    return [n + (n === 1 ? " setting is stale" : " settings are stale"), r.failedText];
  }}),
}}));
"#,
        root = qml::PAGE_ROOT,
        inventories = inventories,
        failed = failed,
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let names: Vec<String> = [
        ("box", 1_793_174_400u64),
        ("my host!", 1_793_230_140),
        ("", 0),
    ]
    .iter()
    .map(|(h, t)| diag::report_file_name(h, *t))
    .collect();
    let ours = json!({
        "names": names,
        "each": inventories.as_array().unwrap().iter().map(|raw| {
            let d = diag::normalize(Some(raw));
            json!({
                "said": [diag::omarchy_summary(&d), diag::atmos_summary(&d), diag::hypr_summary(&d), diag::kernel_summary(&d),
                    diag::gpu_summary(&d), diag::network_summary(&d), diag::pacman_summary(&d),
                    diag::service_map_summary(&d["portals"]), diag::service_map_summary(&d["pipewire"])],
                "ok": [diag::hypr_ok(&d), diag::services_ok(&d["portals"]), diag::services_ok(&d["pipewire"]),
                    diag::disk_pressure(&d), diag::memory_pressure(&d), diag::atmos_ok(&d)],
                "report": diag::report_text(Some(raw)),
                "d": d,
            })
        }).collect::<Vec<_>>(),
        "stale": failed.as_array().unwrap().iter().map(|list| {
            let pairs: Vec<(String, String)> = list.as_array().unwrap().iter()
                .map(|p| (p[0].as_str().unwrap().to_owned(), p[1].as_str().unwrap().to_owned())).collect();
            let (label, text) = diag::stale(&pairs);
            json!([label, text])
        }).collect::<Vec<Value>>(),
    });
    assert_eq!(ours, theirs);
}
