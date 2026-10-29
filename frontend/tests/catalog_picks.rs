//! The rows whose button opens the file chooser (`catalog/picks.rs`) are their QML rows: the
//! button opens a `FileDialog` with the pick's title and name filters, and its `onAccepted` hands
//! the path chosen to the pick's setter. What the setter then sends is `catalog_setters.rs`'s.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::catalog::custom::Custom;
use atmos_ui::catalog::picks::{Kind, pick};
use atmos_ui::catalog::{self, Control};
use rows::{page_src, qml_row};

#[test]
fn each_pick_is_its_qml_dialog() {
    let mut seen = 0;
    for page in catalog::pages() {
        let src = page_src(page);
        let mut n = std::collections::HashMap::new();
        for line in page.lines() {
            let k = n.entry(line.label).or_insert(0);
            let nth = *k;
            *k += 1;
            let Control::Custom(Custom::Choose(id)) = line.control else {
                continue;
            };
            seen += 1;
            let p = pick(id);
            let row = qml_row(page, line.label, nth);
            let at = format!("{} / {}", page.id, line.label);
            assert!(
                row.contains(&format!("text: {}", serde_json::json!(p.button))),
                "{at}: {}",
                p.button
            );
            let dialog = row
                .split(".open()")
                .next()
                .and_then(|head| head.rsplit(|c: char| c.is_whitespace() || c == '{').next())
                .unwrap_or_else(|| panic!("{at}: opens a dialog"));
            let block = src
                .split(&format!("id: {dialog}\n"))
                .nth(1)
                .and_then(|rest| rest.split("\n  }\n").next())
                .unwrap_or_else(|| panic!("{at}: no dialog {dialog}"));
            assert!(
                block.contains(&format!("title: {}", serde_json::json!(p.title))),
                "{at}: title"
            );
            let filters: Vec<String> = p
                .filters
                .iter()
                .map(|(name, pats)| format!("{name} ({})", pats.join(" ")))
                .collect();
            assert!(
                block.contains(&format!("nameFilters: {}", serde_json::json!(filters))),
                "{at}: filters {filters:?}"
            );
            let selected = if p.kind == Kind::Folder {
                "selectedFolder"
            } else {
                "selectedFile"
            };
            let call = format!(
                "onAccepted: Omarchy.{}(RichUi.pathFromUrl({selected}))",
                p.setter.name
            );
            assert!(block.contains(&call), "{at}: {call}");
            if p.idle {
                assert!(
                    row.contains("enabled: !Omarchy.jobBusy"),
                    "{at}: waits for the job"
                );
            }
        }
    }
    assert!(seen >= 3, "every pick is drawn");
}
