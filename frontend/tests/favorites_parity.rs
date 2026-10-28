//! The Rust favorites are the QML ones until the QML window goes: `services/Favorites.js` runs
//! under Node and must give what `favorites.rs` gives for a row's id, the list read from the
//! snapshot, a star's list and its write (`Omarchy.toggleFavorite`), and the Favorites hub's
//! groups, whose hubs `Hubs.hubById` knows. Without `node` the test says so and stops.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use atmos_ui::favorites::{self, Item};
use serde_json::{Value, json};

const NODE: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const F = require(root + "/services/Favorites.js");
const H = require(root + "/services/Hubs.js");
const R = require(root + "/tests/harness.js").load("services/Requests.js");
const input = JSON.parse(fs.readFileSync(0, "utf8"));
const known = (hub) => !!H.hubById(hub);
const parse = (body) => body && Object.assign({}, body, { args: { json: JSON.parse(body.args.json) } });
const out = {
  keys: input.labels.flatMap((l) => input.hubs.map((h) => [F.slug(l), F.rowKey(h, l)])),
  lists: input.lists.map((list) => {
    const items = F.parseDocument(list);
    return {
      items,
      groups: F.groups(items, known).map((g) => [g.title, g.items.map((i) => i.id)]),
      favorite: input.rows.map((r) => F.isFavorite(items, r.hub, r.label)),
      toggles: input.rows.map((r) =>
        [undefined, true, false].map((want) => {
          // What SettingRow.toggleStar passes.
          const row = { hub: r.hub, hubTitle: H.hubTitle(r.hub), label: r.label, description: r.description };
          const next = F.toggleItem(items, row, want);
          return { next, write: parse(R.write("favorites.write", { json: F.serialize(next) })) };
        }),
      ),
    };
  }),
};
process.stdout.write(JSON.stringify(out));
"#;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn node(input: &Value) -> Option<Value> {
    let Ok(mut child) = Command::new("node")
        .arg("-e")
        .arg(NODE)
        .env("REPO_DIR", repo())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    else {
        eprintln!("skip: node is not installed");
        return None;
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "the JavaScript side failed");
    Some(serde_json::from_slice(&out.stdout).unwrap())
}

fn item_json(item: &Item) -> Value {
    json!({"id": item.id, "hub": item.hub, "hubTitle": item.hub_title, "label": item.label,
        "description": item.description})
}

fn items_json(items: &[Item]) -> Value {
    Value::Array(items.iter().map(item_json).collect())
}

/// Rows a star could be pressed on, and some it could not.
fn rows() -> Vec<Value> {
    vec![
        json!({"hub": "defaults", "hubTitle": "Defaults", "label": "Browser", "description": "Links"}),
        json!({"hub": "tweaks", "hubTitle": "Tweaks", "label": "Natural scrolling", "description": ""}),
        json!({"hub": "system/kernel", "hubTitle": "Kernel", "label": "Direct boot", "description": "x"}),
        json!({"hub": "nothing", "hubTitle": "", "label": "A row", "description": ""}),
        json!({"hub": "", "label": "No hub"}),
        json!({"hub": "defaults", "label": "!!!"}),
    ]
}

fn lists() -> Vec<Value> {
    let mut out = vec![
        json!([]),
        json!(null),
        json!("junk"),
        json!({"items": "no"}),
        json!({"items": [{"hub": "defaults", "label": "Browser"}]}),
        json!([
            {"hub": "defaults", "hubTitle": "Defaults", "label": "Browser"},
            {"hub": "defaults", "label": "browser", "description": "second of the same id"},
            {"hub": "/tweaks/", "label": "Natural scrolling", "hubTitle": "Tweaks"},
            {"hub": "Tweaks", "label": "Upper case hub"},
            {"hub": "nothing", "label": "Unknown hub"},
            {"hub": "system/kernel", "label": "Direct boot"},
            {"hub": "monitor", "hubTitle": "", "label": "CPU"},
            {"hub": 7, "label": 3.0},
            {"hub": "home", "label": true},
            {"hub": "home", "label": false},
            {"label": "No hub"},
            [1, 2],
            "junk",
            null,
        ]),
    ];
    out.push(Value::Array(rows()));
    out
}

#[test]
fn the_rust_favorites_give_what_favorites_js_gives() {
    let labels = [
        "Browser",
        "  Disable mouse-acceleration! ",
        "Wi‑Fi / 5 GHz",
        "ÉCRAN",
        "",
        "!!",
        "a--b",
        "Ünïcödé 2",
    ];
    let hubs = [
        "defaults",
        "/system/kernel/",
        "System",
        "a//b",
        "",
        "x/y-z",
        "x/Y",
    ];
    let rows = rows();
    let lists = lists();
    let Some(js) = node(&json!({"labels": labels, "hubs": hubs, "rows": rows, "lists": lists}))
    else {
        return;
    };

    let ours: Vec<Value> = labels
        .iter()
        .flat_map(|l| {
            hubs.iter().map(move |h| {
                json!([
                    favorites::slug(l),
                    favorites::row_key(h, l).unwrap_or_default()
                ])
            })
        })
        .collect();
    assert_eq!(Value::Array(ours), js["keys"]);

    for (list, js) in lists.iter().zip(js["lists"].as_array().unwrap()) {
        let items = favorites::items(list);
        assert_eq!(items_json(&items), js["items"], "the list of {list}");
        let groups: Vec<Value> = favorites::groups(&items)
            .into_iter()
            .map(|(title, rows)| json!([title, rows.iter().map(|r| &r.id).collect::<Vec<_>>()]))
            .collect();
        assert_eq!(Value::Array(groups), js["groups"], "the groups of {list}");
        for (n, row) in rows.iter().enumerate() {
            let (hub, label) = (row["hub"].as_str().unwrap(), row["label"].as_str().unwrap());
            assert_eq!(
                json!(favorites::is_favorite(&items, hub, label)),
                js["favorite"][n],
                "{row} in {list}"
            );
            let star = Item::of_row(hub, label, row["description"].as_str().unwrap_or_default());
            for (w, want) in [None, Some(true), Some(false)].into_iter().enumerate() {
                let next = star
                    .as_ref()
                    .map_or_else(|| items.clone(), |s| favorites::toggle(&items, s, want));
                let js = &js["toggles"][n][w];
                assert_eq!(items_json(&next), js["next"], "{row} {want:?} in {list}");
                let mut write = favorites::write(&next);
                let text = write["args"]["json"].as_str().unwrap().to_owned();
                write["args"]["json"] = serde_json::from_str(&text).unwrap();
                assert_eq!(write, js["write"], "the write of {row} {want:?}");
            }
        }
    }
}
