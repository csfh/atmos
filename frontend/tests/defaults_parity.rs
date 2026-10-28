//! The Rust Defaults page is the QML one until the QML one goes: for every snapshot the test makes
//! up, `services/Defaults.js` and the QML page's own steps run under Node and must give what
//! `defaults.rs` gives: the programs each picker offers, the one it shows, and the `settings.set`
//! each choice sends, or that it sends nothing.
//!
//! The JavaScript side of a choice is the QML's own path to the backend, run for real where it can
//! be: the page's `onChanged` check, then `SettingTable.plan` (`Omarchy.set`) or the handler check
//! of `setMimeDefault`, then `Settings.commandFor` for the value and the read-back keys, then
//! `Requests.settingsSet`. The glue between them, from the QML, is restated here, and the QML page
//! is checked to take its lists from `Defaults.js`. Without `node` the test says so and stops.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use atmos_ui::defaults::{self, PICKERS, Picker, Source};
use atmos_ui::settings::Settings;
use serde_json::{Value, json};

const NODE: &str = r#"
const fs = require("fs");
const root = process.env.REPO_DIR;
const { load } = require(root + "/tests/harness.js");
const D = require(root + "/services/Defaults.js");
const U = load("services/RichUi.js");
const T = load("services/SettingTable.js");
const S = load("services/Settings.js");
const R = load("services/Requests.js");
const input = JSON.parse(fs.readFileSync(0, "utf8"));

// What Omarchy.qml holds before anything is read.
const EMPTY = {
  browser: "", terminal: "", editor: "", browsers: {}, terminals: {}, editors: {},
  mimePdf: "", mimeImage: "", mimeVideo: "",
  mimePdfOptions: [], mimeImageOptions: [], mimeVideoOptions: [],
};

function send(domain, value, snap) {
  const cmd = S.commandFor(domain, value, snap, {});
  if (!cmd || cmd.skip) return null;
  return R.settingsSet(domain, cmd.value, cmd.readback);
}

// One picker of DefaultsPage.qml: its value, its options and its onChanged.
function picker(id, O) {
  const set = (key) => (v) => {
    const step = T.plan(key, v, O[key]);
    return step.dispatch ? send(key, step.value, O) : null;
  };
  const mime = (key) => (v) => {
    if (v === O[key]) return null;
    if (!/^[A-Za-z0-9._-]+\.desktop$/.test(String(v || ""))) return null;
    return send(key, v, O);
  };
  switch (id) {
    case "browser":
      return { value: O.browser, options: U.installedOptions(D.browserOptions(), O.browsers),
        change: (v) => (v !== O.browser ? set("browser")(v) : null) };
    case "terminal":
      return { value: O.terminal, options: U.installedOptions(D.terminalOptions(), O.terminals),
        change: (v) => (v !== O.terminal ? set("terminal")(v) : null) };
    case "editor":
      return { value: D.editorValue(O.editor),
        options: U.installedOptions(D.editorOptions(), D.editorsInstalled(O.editors)),
        change: (v) => (v !== D.editorValue(O.editor) ? set("editor")(v) : null) };
    case "pdf":
      return { value: O.mimePdf, options: D.mimeOptions(O.mimePdf, O.mimePdfOptions), change: mime("mimePdf") };
    case "image":
      return { value: O.mimeImage, options: D.mimeOptions(O.mimeImage, O.mimeImageOptions), change: mime("mimeImage") };
    case "video":
      return { value: O.mimeVideo, options: D.mimeOptions(O.mimeVideo, O.mimeVideoOptions), change: mime("mimeVideo") };
  }
}

const out = {
  lists: { browsers: D.browserOptions(), terminals: D.terminalOptions(), editors: D.editorOptions() },
  states: input.states.map((state) => {
    const O = Object.assign({}, EMPTY, state);
    return input.pickers.map((id) => {
      const p = picker(id, O);
      return {
        value: p.value,
        options: p.options.map((o) => [String(o.value), String(o.label)]),
        sends: input.choices.map((c) => p.change(c)),
      };
    });
  }),
};
process.stdout.write(JSON.stringify(out));
"#;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn node(input: &Value) -> Option<Value> {
    let mut child = match Command::new("node")
        .arg("-e")
        .arg(NODE)
        .env("REPO_DIR", repo())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!("skip: node is not installed");
            return None;
        }
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

const PDF: &str = "org.gnome.Evince.desktop";

/// Snapshots a machine could report: nothing read, programs installed and missing, Zed under its
/// Omarchy name, a default that is not in the list, handlers listed and not.
fn states() -> Vec<Value> {
    vec![
        json!({}),
        json!({"browser": "chromium", "browsers": {"chromium": true, "chrome": false, "zen": false}}),
        json!({"browser": "firefox", "browsers": {"firefox": true, "edge": 0, "brave": null}}),
        json!({"terminal": "ghostty", "terminals": {"alacritty": false, "foot": false}}),
        json!({"editor": "zeditor", "editors": {"zeditor": true, "nvim": false}}),
        json!({"editor": "zeditor", "editors": {"zeditor": false}}),
        json!({"editor": "nvim", "editors": {"nvim": true, "code": false, "emacs": false}}),
        json!({"editor": "zed"}),
        json!({"browser": "netscape", "browsers": {"netscape": false}}),
        json!({"mimePdf": PDF, "mimePdfOptions": [{"value": PDF, "label": "Evince"},
            {"value": "org.pwmt.zathura.desktop", "label": "Zathura"}]}),
        json!({"mimePdf": "foo.desktop", "mimePdfOptions": [{"value": PDF, "label": "Evince"}]}),
        json!({"mimeImage": "imv.desktop", "mimeImageOptions": []}),
        json!({"mimeVideo": "", "mimeVideoOptions": [{"value": "mpv.desktop", "label": "mpv"}]}),
    ]
}

/// What a picker could be asked to choose: every program it lists, an empty choice, Zed by both
/// names, and handlers good and bad.
fn choices() -> Vec<String> {
    let mut out: Vec<String> = ["", "zeditor", "zed", PDF, "foo.desktop", "mpv.desktop"]
        .into_iter()
        .chain([
            "imv.desktop",
            ".desktop",
            "rm -rf /",
            "a b.desktop",
            "x.desktop.sh",
        ])
        .map(str::to_owned)
        .collect();
    for list in [defaults::BROWSERS, defaults::TERMINALS, defaults::EDITORS] {
        out.extend(list.iter().map(|(v, _)| (*v).to_owned()));
    }
    out
}

fn settings(state: &Value) -> Settings {
    let mut s = Settings::default();
    // Each key goes to the topic that holds it, as the daemon sends them.
    for topic in ["settings.system", "settings.host"] {
        let doc: serde_json::Map<String, Value> = state
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _)| {
                defaults::HELD
                    .iter()
                    .any(|h| h.key == key.as_str() && h.topic == topic)
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        s.adopt(topic, 1, &Value::Object(doc));
    }
    s
}

fn rust(p: &Picker, s: &Settings, choices: &[String]) -> Value {
    json!({
        "value": p.current(s).unwrap_or_default(),
        "options": p.options(s),
        "sends": choices.iter().map(|c| p.choose(c, s)).collect::<Vec<_>>(),
    })
}

#[test]
fn the_rust_pickers_give_what_the_qml_page_gives() {
    let states = states();
    let choices = choices();
    let ids: Vec<_> = PICKERS.iter().map(|p| p.id).collect();
    let Some(js) = node(&json!({"states": states, "choices": choices, "pickers": ids})) else {
        return;
    };
    for (state, js_rows) in states.iter().zip(js["states"].as_array().unwrap()) {
        let s = settings(state);
        for (p, js_row) in PICKERS.iter().zip(js_rows.as_array().unwrap()) {
            assert_eq!(&rust(p, &s, &choices), js_row, "{} with {state}", p.id);
        }
    }
    for (all, list) in [
        (defaults::BROWSERS, "browsers"),
        (defaults::TERMINALS, "terminals"),
        (defaults::EDITORS, "editors"),
    ] {
        let js: Vec<(String, String)> = js["lists"][list]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| {
                (
                    o["value"].as_str().unwrap().to_owned(),
                    o["label"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        let rs: Vec<_> = all
            .iter()
            .map(|(v, l)| ((*v).to_owned(), (*l).to_owned()))
            .collect();
        assert_eq!(rs, js, "{list}");
    }
}

/// The installed-program keys are the ones the QML page reads for each picker.
#[test]
fn each_picker_reads_the_keys_the_qml_page_reads() {
    let qml = read("pages/DefaultsPage.qml");
    for p in PICKERS {
        let key = match p.source {
            Source::Installed { key, .. } => key,
            Source::Handlers { key } => key,
        };
        assert!(
            qml.contains(&format!("Omarchy.{key}")),
            "{} reads {key}",
            p.id
        );
        assert!(
            qml.contains(&format!("Omarchy.{}", p.domain)),
            "{} shows {}",
            p.id,
            p.domain
        );
        assert!(qml.contains(p.label) && qml.contains(p.description) && qml.contains(p.hint));
    }
    for list in [
        "browserOptions",
        "terminalOptions",
        "editorOptions",
        "mimeOptions",
    ] {
        assert!(
            qml.contains(&format!("DefaultsJs.{list}(")),
            "the page takes {list} from Defaults.js"
        );
    }
}
