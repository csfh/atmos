//! `omafile` is `services/Settings.js`: the catalog it generates from, the sections offered, the
//! Markdown an export writes, what reading a file back gives, the plan an import makes against
//! another machine, and every line the page shows of it, from two machines' snapshots built
//! from the catalog itself and from files written by hand to go wrong. Without `node` the test
//! says so and stops.

mod common;

use atmos_ui::omafile::{self, CATALOG, SECTIONS, canon, lines, markdown, plan, toml};
use common::qml;
use serde_json::{Map, Value, json};

/// Sets `key` in `snap`, a dot walking into (and making) a nested object.
fn put(snap: &mut Value, key: &str, value: Value) {
    let mut node = snap;
    let parts: Vec<&str> = key.split('.').collect();
    for p in &parts[..parts.len() - 1] {
        node = node
            .as_object_mut()
            .unwrap()
            .entry(*p)
            .or_insert_with(|| json!({}));
    }
    node[parts[parts.len() - 1]] = value;
}

fn lists(which: u8) -> Map<String, Value> {
    let a = which == 0;
    let mut m = Map::new();
    m.insert(
        "bindings".into(),
        json!([
            {"keys": "SUPER, Return", "command": "kitty", "label": "Terminal", "managed": true},
            {"keys": "SUPER, B", "command": if a { "firefox" } else { "chromium" }, "label": ""},
            {"keys": "SUPER, Q", "unbind": true, "command": ""},
            {"keys": "SUPER  SHIFT, E", "command": "nautilus", "managed": false},
        ]),
    );
    m.insert("autostart".into(), json!([
        {"command": "waybar", "delay": 0, "enabled": true},
        {"command": "  mako  ", "delay": if a { "2" } else { "3" }, "enabled": true, "managed": false},
    ]));
    m.insert(
        "windowRules".into(),
        json!([
            {"match": "pavucontrol", "float": true, "size": [800, 600], "center": true},
            {"match": "steam", "workspace": if a { "5" } else { "6" }, "opacity": "0.9 0.8"},
        ]),
    );
    m.insert(
        "workspaces".into(),
        json!([
            {"id": 1, "name": "web", "monitor": "DP-1", "default": true},
            {"workspace": "special:scratch", "on_created_empty": "kitty"},
        ]),
    );
    m.insert("monitorRules".into(), json!([
        {"output": "DP-1", "mode": "2560x1440@165", "scale": if a { 1.25 } else { 1.5 }, "position": "0x0", "vrr": "1"},
        {"output": "eDP-1", "disabled": true, "bitdepth": 10, "cm": "hdr", "transform": 9},
    ]));
    m.insert(
        "envVars".into(),
        json!([{"key": "EDITOR", "value": if a { "nvim" } else { "helix" }}]),
    );
    m.insert(
        "indicatorsItems".into(),
        json!(if a {
            json!(["Dnd", "Dictation"])
        } else {
            json!([
                "Dictation",
                "Reminder",
                "NightLight",
                "Dnd",
                "StayAwake",
                "ScreenRecording"
            ])
        }),
    );
    m.insert("trayHidden".into(), json!(["nm-applet"]));
    m.insert(
        "trayPinned".into(),
        json!(if a { json!(["blueman"]) } else { json!([]) }),
    );
    m
}

/// A machine's snapshot: every key the catalog names, of its type, the two machines apart on
/// most of them, with the lists a key's `options` reads.
fn machine(which: u8) -> Value {
    let mut snap = json!({});
    let rows = lists(which);
    for (i, item) in CATALOG.iter().enumerate() {
        let i = i as f64;
        let v = match item.ty {
            "boolean" => json!((i as u64 + u64::from(which)).is_multiple_of(2)),
            "integer" => json!(10 + i as u64 * 3 + u64::from(which) * (i as u64 % 3)),
            "number" => json!(0.5 + i / 100.0 + f64::from(which) * 0.125),
            "list" => rows.get(item.key).cloned().unwrap_or(json!([])),
            _ => match (item.choices, item.format) {
                (Some(c), _) => json!(c[(i as usize + usize::from(which)) % c.len()]),
                (_, "time") => json!(if which == 0 { "07:30" } else { "21:05" }),
                _ => json!(format!(
                    "{}-{}",
                    item.key,
                    if which == 0 && (i as u64).is_multiple_of(4) {
                        "b"
                    } else {
                        "a"
                    }
                )),
            },
        };
        put(&mut snap, item.key, v);
    }
    snap["hostname"] = json!(if which == 0 { "forge" } else { "anvil" });
    snap["omarchyVersion"] = json!("3.1.0");
    snap["atmosVersion"] = json!("1.0.0-rc");
    // Options this machine lists: the file's theme and font are installed here, its locale is not,
    // and it does not say which keyboard layouts it has.
    snap["themes"] = json!([{"value": "theme-a", "label": "A"}, "theme-b", "theme-a"]);
    snap["fonts"] = json!(["font-a", "font-b"]);
    snap["locales"] = json!(["en_US.UTF-8"]);
    snap["timezones"] = json!([]);
    snap["hyprWorkspaceGestureUnmanaged"] = json!(which == 1);
    snap
}

fn files() -> Vec<String> {
    vec![
        String::new(),
        "no blocks at all".into(),
        "```toml atmos:meta\nschema = 2\n```\n```toml atmos:appearance\ntheme = \"x\"\n```".into(),
        "```toml atmos:meta\nschema = 1\nhardware = \"ThinkPad\"\nexported = \"2026-10-28T12:34:56.789Z\"\nhostname = \"h\"\n```\n\n```toml atmos:appearance\n# a comment\ntheme = \"theme-a\" # trailing\nfont = \"nope\"\ntextSize = 12.5\nbad line\n2key = 1\nhyprLook.cursorSize = [1, \"a,b\", \"q\\\"uote\"] # c\nmystery = true\n```\n```toml atmos:windows\nhyprLook.gapsIn = -.5\nhyprLook.gapsOut = 7\ntheme = \"wrong section\"\nhyprLook.blur = \"yes\"\nhyprLook.rounding = [1,,2]\nhyprLook.borderSize = \"unterminated\n```\n```json atmos:bindings\n[{\"keys\": \"SUPER, T\", \"command\": \"kitty\"}, {\"keys\": \"bad\\nkeys\", \"command\": \"x\"}]\n```\n```json atmos:autostart\n{\"not\": \"a list\"}\n```\n```json atmos:trayHidden\nnot json\n```\n```toml atmos:security\nsshd = true\n```\n```toml atmos:never".into(),
        "  ```toml   atmos:bar  \nclockFormat = \"%H\\t%M\\n\" \n```\n```tomlx atmos:bar\nx = 1\n```\n```json atmos:9bad\n[]\n```".into(),
    ]
}

/// A plan's change list with every list row made bad one way and another.
fn bad_lists() -> Vec<(&'static str, Value)> {
    vec![
        ("bindings", json!([{"keys": "SUPER, X"}])),
        ("bindings", json!([{"keys": "SUPER, X", "unbind": "yes"}])),
        ("bindings", json!([{"keys": "SUPER, X", "command": 5}])),
        (
            "bindings",
            json!([{"keys": "SUPER, X", "command": "a", "label": "  "}]),
        ),
        ("bindings", json!(["not a row"])),
        ("autostart", json!([{"command": "x", "delay": 601}])),
        ("autostart", json!([{"command": "x", "enabled": "no"}])),
        (
            "windowRules",
            json!([{"match": "a", "placement": "sideways"}]),
        ),
        (
            "windowRules",
            json!([{"match": "a", "width": 50, "height": 300}]),
        ),
        ("windowRules", json!([{"match": "a"}])),
        ("windowRules", json!([{"match": "a", "opacity": "1  2"}])),
        ("windowRules", json!([{"match": "a]]"}])),
        ("workspaces", json!([{"id": 11}])),
        ("workspaces", json!([{"id": 2, "name": "bad/name"}])),
        ("monitorRules", json!([{"output": "DP 1"}])),
        (
            "monitorRules",
            json!([{"output": "DP-1", "mode": "1920x1080@"}]),
        ),
        (
            "monitorRules",
            json!([{"output": "DP-1", "mode": "highrr", "position": "1x-2", "scale": -1, "mirror": "eDP-1"}]),
        ),
        ("envVars", json!([{"key": "PATH", "value": "x"}])),
        ("envVars", json!([{"key": "A", "value": "x\ny"}])),
        ("indicatorsItems", json!(["Bogus"])),
        ("trayPinned", json!([""])),
        ("trayPinned", json!("not a list")),
    ]
}

#[test]
fn the_omafile_is_settings_js() {
    let (a, b) = (machine(0), machine(1));
    let sections: Vec<&str> = SECTIONS.iter().map(|s| s.0).collect();
    let chosen: Vec<Vec<&str>> = vec![
        sections.clone(),
        vec!["appearance", "bindings"],
        vec![],
        vec!["system", "nope"],
    ];
    let opts = [
        plan::Options::default(),
        plan::Options {
            hardware: "ThinkPad".into(),
            workspace_gesture_unmanaged: false,
        },
        plan::Options {
            hardware: "Framework".into(),
            workspace_gesture_unmanaged: true,
        },
    ];
    let outputs = [
        "progress\t1\t3\ttheme\nnoise\n{\"backup\": \"/b/1\", \"results\": [{\"status\": \"applied\"}, {\"status\": \"failed\"}, null]}\n\n",
        "{not json}\n[1]\n",
        "",
    ];
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("omafile_parity.json");
    let opts_json: Vec<Value> = opts
        .iter()
        .map(|o| json!({"hardware": o.hardware, "workspaceGestureUnmanaged": o.workspace_gesture_unmanaged}))
        .collect();
    let bad: Vec<Value> = bad_lists()
        .into_iter()
        .map(|(k, v)| json!([k, v]))
        .collect();
    std::fs::write(
        &input,
        json!({"a": a, "b": b, "chosen": chosen, "files": files(), "opts": opts_json, "outputs": outputs, "bad": bad})
            .to_string(),
    )
    .unwrap();
    let script = format!(
        r#"
const {{ load }} = require(process.env.REPO_DIR + "/tests/harness.js");
const S = load("services/Settings.js");
const {{ a, b, chosen, files, opts, outputs, bad }} = JSON.parse(require("fs").readFileSync({input}, "utf8"));
const meta = {{ exported: "2026-10-28T09:00:00.000Z", hardware: "ThinkPad" }};
const keys = chosen.map((ids) => S.keysForSections(ids));
const exported = keys.map((k) => S.exportMarkdown(a, k, meta));
const docs = exported.concat(files).map((t) => S.parseSettingsMarkdown(t));
const plans = docs.map((d) => opts.map((o) => S.planImport(d, b, null, o)));
const words = (p, d) => ({{
  summary: S.fileSummary(d, p), changes: S.changeLines(p), warnings: S.warningLines(p), blocked: S.blockedLines(p),
  forecast: S.applyForecast(p), confirm: S.applyConfirmMessage(p), commands: S.hasCommandImport(p),
  commandConfirm: S.commandConfirmMessage(p), passwords: S.passwordCount(p), json: S.planToJson(p),
}});
const byKey = S.catalogByKey();
process.stdout.write(JSON.stringify({{
  catalog: S.settingsCatalog(), sections: S.settingsSections(), offered: S.selectableSections(), keys, exported, docs, plans,
  words: plans.map((ps, i) => ps.map((p) => words(p, docs[i]))),
  noPlan: docs.map((d) => S.fileSummary(d, null)),
  results: outputs.map((o) => {{ const r = S.parseApplyResult(o); return [r, S.appliedCountFromResult(r), S.backupDirFromResult(r)]; }}),
  canon: bad.map(([k, v]) => S.canonicalizeList(byKey[k], v)),
  live: ["bindings", "autostart", "windowRules", "workspaces", "monitorRules", "envVars", "indicatorsItems", "trayHidden", "trayPinned"]
    .map((k) => [S.canonicalizeList(byKey[k], a[k]), S.canonicalizeList(byKey[k], b[k])]),
  toml: ["1", "-0", "1.", ".5", "-.25", "true # c", "\"a\" x", "[]", "[1, [2]]", "[1,]", "[\"a\" , 2.5 ]  # c", "\"\\q\"", "nan", "\"tab\\there\""].map((t) => {{ const v = S.parseTomlValue(t); return v === undefined ? "UNDEFINED" : v; }}),
  tomlText: [[1, 2.5, "q\"\\\n\r\t", true, null], 1e21, 1e-7, -0, 0.1 + 0.2].map((v) => S.tomlValue(v)),
}}));
"#,
        input = json!(input.to_string_lossy()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };

    // The catalog and sections are the JavaScript's.
    let ours: Vec<Value> = CATALOG
        .iter()
        .map(|i| json!({"key": i.key, "section": i.section, "label": i.label, "tier": i.tier, "kind": i.kind,
            "type": i.ty, "options": i.options, "choices": i.choices, "format": i.format, "hostBound": i.host_bound,
            "needsRoot": i.needs_root, "extraConfirm": i.extra_confirm, "consequence": i.consequence,
            "importable": i.importable}))
        .collect();
    assert_eq!(json!(ours), theirs["catalog"]);
    let ours: Vec<Value> = SECTIONS
        .iter()
        .map(|(id, t, n)| json!({"id": id, "title": t, "note": n}))
        .collect();
    assert_eq!(json!(ours), theirs["sections"]);
    let ours: Vec<Value> = omafile::selectable_sections()
        .iter()
        .map(|s| json!({"id": s.id, "title": s.title, "note": s.note, "count": s.count, "byDefault": s.by_default}))
        .collect();
    assert_eq!(json!(ours), theirs["offered"]);

    let meta = markdown::Meta {
        exported: "2026-10-28T09:00:00.000Z".into(),
        hardware: "ThinkPad".into(),
    };
    let mut texts = Vec::new();
    for (i, ids) in chosen.iter().enumerate() {
        let keys = omafile::keys_for_sections(ids);
        assert_eq!(json!(keys), theirs["keys"][i], "keys {ids:?}");
        let text = markdown::export(&a, &keys, &meta);
        assert_eq!(json!(text), theirs["exported"][i], "export {ids:?}");
        texts.push(text);
    }
    assert!(texts[0].contains("```json atmos:bindings"), "{}", texts[0]);
    texts.extend(files());
    for (i, text) in texts.iter().enumerate() {
        let doc = markdown::parse(text);
        assert_eq!(doc, theirs["docs"][i], "doc {i}: {text}");
        assert_eq!(
            json!(lines::file_summary(&doc, None)),
            theirs["noPlan"][i],
            "summary {i}"
        );
        for (j, o) in opts.iter().enumerate() {
            let p = plan::plan(&doc, &b, o);
            assert_eq!(p, theirs["plans"][i][j], "plan {i} {j}");
            let w = &theirs["words"][i][j];
            let at = format!("words {i} {j}");
            assert_eq!(
                json!(lines::file_summary(&doc, Some(&p))),
                w["summary"],
                "{at}"
            );
            assert_eq!(json!(lines::change_lines(&p)), w["changes"], "{at}");
            assert_eq!(json!(lines::warning_lines(&p)), w["warnings"], "{at}");
            assert_eq!(json!(lines::blocked_lines(&p)), w["blocked"], "{at}");
            assert_eq!(json!(lines::forecast(&p)), w["forecast"], "{at}");
            assert_eq!(json!(lines::apply_confirm(&p)), w["confirm"], "{at}");
            assert_eq!(json!(lines::has_commands(&p)), w["commands"], "{at}");
            assert_eq!(
                json!(lines::command_confirm(&p)),
                w["commandConfirm"],
                "{at}"
            );
            assert_eq!(json!(lines::password_count(&p)), w["passwords"], "{at}");
            assert_eq!(json!(lines::plan_json(&p)), w["json"], "{at}");
        }
    }
    assert!(
        !theirs["plans"][0][0]["changes"]
            .as_array()
            .unwrap()
            .is_empty(),
        "a plan changes something"
    );
    assert!(
        !theirs["plans"][6][1]["blocked"]
            .as_array()
            .unwrap()
            .is_empty(),
        "a bad file blocks"
    );

    for (i, o) in outputs.iter().enumerate() {
        let r = lines::apply_result(o);
        assert_eq!(
            json!([r, lines::applied_count(&r), lines::backup_dir(&r)]),
            theirs["results"][i],
            "result {i}"
        );
    }
    let canon_json = |item: &omafile::Item, v: &Value| match canon::list(item, v) {
        Ok(rows) => json!({"value": rows}),
        Err(e) => json!({"error": e}),
    };
    for (i, (k, v)) in bad_lists().iter().enumerate() {
        let item = omafile::item(k).unwrap();
        assert_eq!(canon_json(item, v), theirs["canon"][i], "canon {k} {v}");
    }
    for (i, k) in [
        "bindings",
        "autostart",
        "windowRules",
        "workspaces",
        "monitorRules",
        "envVars",
        "indicatorsItems",
        "trayHidden",
        "trayPinned",
    ]
    .iter()
    .enumerate()
    {
        let item = omafile::item(k).unwrap();
        assert_eq!(
            json!([canon_json(item, &a[k]), canon_json(item, &b[k])]),
            theirs["live"][i],
            "live {k}"
        );
    }
    let tomls = [
        "1",
        "-0",
        "1.",
        ".5",
        "-.25",
        "true # c",
        "\"a\" x",
        "[]",
        "[1, [2]]",
        "[1,]",
        "[\"a\" , 2.5 ]  # c",
        "\"\\q\"",
        "nan",
        "\"tab\\there\"",
    ];
    for (i, t) in tomls.iter().enumerate() {
        let ours = toml::value(t).unwrap_or(json!("UNDEFINED"));
        assert_eq!(ours, theirs["toml"][i], "toml {t}");
    }
    let values = [
        json!([1, 2.5, "q\"\\\n\r\t", true, null]),
        json!(1e21),
        json!(1e-7),
        json!(-0.0),
        json!(0.1 + 0.2),
    ];
    for (i, v) in values.iter().enumerate() {
        assert_eq!(
            json!(toml::value_text(v)),
            theirs["tomlText"][i],
            "toml text {v}"
        );
    }
}
