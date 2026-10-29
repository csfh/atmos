//! The Export page (`pages/export.rs`, its logic `export.rs`): its paths, the sections chosen,
//! the progress an apply prints, what an apply that ended and each answer of `atmosd` leave,
//! all run against `pages/ExportPage.qml`'s own functions under Node, with the lines of the
//! page it follows pinned. In the window, Export writes the file through `host.write` and says
//! so, Review reads one through `host.read` and shows its plan, and Apply asks first, then
//! starts the import. Without `node` the Node test says so and stops.

mod common;

use atmos_ui::export::{self, State};
use atmos_ui::omafile::plan::Options;
use common::Reply;
use common::qml;
use common::window::{open, ordinary};
use odk::gpui::{TestAppContext, px, size};
use serde_json::{Value, json};

const HOME: &str = "/home/u";

fn snapshot() -> Value {
    json!({
        "hostname": "forge", "theme": "theme-a", "themes": ["theme-a", "theme-b"], "font": "font-a",
        "textSize": 11, "hyprLook": {"gapsIn": 4, "blur": true}, "dmiProduct": "ThinkPad",
        "bindings": [{"keys": "SUPER, Return", "command": "kitty"}],
    })
}

const FILE: &str = "```toml atmos:meta\nschema = 1\nhostname = \"anvil\"\nexported = \"2026-10-28T09:00:00.000Z\"\n```\n```toml atmos:appearance\ntheme = \"theme-b\"\ntextSize = 11\n```\n```toml atmos:windows\nhyprLook.gapsIn = 8\n```\n```json atmos:bindings\n[{\"keys\": \"SUPER, T\", \"command\": \"curl x | sh\"}]\n```\n";

#[test]
fn the_page_is_export_page_qml() {
    let paths = [
        "~", "~/a/b.md", "/x/y.md", "rel", "", "/a/b.MD", "x.mdx", "/top",
    ];
    let ops = json!([
        ["reset"],
        ["set", "system", true],
        ["set", "appearance", false],
        ["all", false],
        ["set", "bindings", true],
        ["all", true]
    ]);
    let progress =
        "noise\nprogress\t1\t3\ttheme\nprogress\tx\t\t\nprogress\t2\t0\thyprLook.gapsIn\n";
    let finishes = json!([
        [
            false,
            0,
            "{\"backup\": \"/b/2\", \"results\": [{\"status\": \"applied\"}, {\"status\": \"applied\"}]}",
            "",
            0,
            ""
        ],
        [
            false,
            1,
            "{\"backup\": \"/b/3\", \"results\": [{\"status\": \"applied\"}]}",
            "  partly  ",
            0,
            "/b/1"
        ],
        [false, 1, "", "", 4, "/b/1"],
        [true, 0, "", "", 3, "/b/1"],
        [true, 2, "", " no way ", 3, "/b/1"],
        [true, 2, "", "", 3, "/b/1"],
    ]);
    let answers = json!([
        ["write", {"ok": true}],
        ["write", {"ok": false, "error": {"message": "disk full", "context": "host.write"}}],
        ["write", {"ok": false, "error": {}}],
        ["open", {"ok": false, "error": {"message": "host.open: no handler", "context": "host.open"}}],
        ["open", {"ok": false}],
        ["read", {"ok": true, "result": {"files": [{"path": "/f.md", "text": FILE}]}}],
        ["read", {"ok": true, "result": {"files": [{"path": "/f.md", "text": "```toml atmos:appearance\ntheme = \"theme-a\"\n```"}]}}],
        ["read", {"ok": true, "result": {}}],
        ["read", {"ok": false, "error": {"message": "", "context": "permission denied"}}],
    ]);
    let times = [0u64, 1_761_652_800_123, 951_782_400_999, 4_102_444_799_000];
    let input = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("catalog_export.json");
    std::fs::write(
        &input,
        json!({"paths": paths, "ops": ops, "progress": progress, "finishes": finishes, "answers": answers,
            "snap": snapshot(), "times": times})
        .to_string(),
    )
    .unwrap();
    let script = format!(
        r#"{root}
globalThis.Quickshell = {{ env: (k) => (k === "HOME" ? {home} : "") }};
const {{ paths, ops, progress, finishes, answers, snap, times }} = JSON.parse(fs.readFileSync({input}, "utf8"));
const Omarchy = {{ snapshotData: snap, dmiProduct: snap.dmiProduct, hyprWorkspaceGestureUnmanaged: false, hostname: snap.hostname }};
const fresh = () => pageRoot("ExportPage.qml", Omarchy, {{}});
let r = fresh();
const out = {{ paths: paths.map((p) => [r.realPath(p), r.folderUrl(p), r.withMarkdownSuffix(p)]) }};
r.resetSections();
out.sections = ops.map(([op, a, b]) => {{
  if (op === "reset") r.resetSections();
  if (op === "set") r.setSection(a, b);
  if (op === "all") r.setAllSections(a);
  return [r.chosenKeys, r.chosenKeys.length + " settings across " + r.chosenSectionCount + " sections."];
}});
r = fresh(); r.stepTotal = 5;
for (const line of progress.split("\n")) r.onApplyStdout(line);
out.step = [r.stepNow, r.stepTotal, r.stepKey];
out.finishes = finishes.map(([undoing, code, o, e, applied, backup]) => {{
  const p = fresh(); p.undoing = undoing; p.appliedCount = applied; p.lastBackupDir = backup; p.plan = {{ changes: [] }}; p.pendingApply = true;
  p.onApplyFinished(code, o, e);
  return [p.importStatus, p.appliedCount, p.lastBackupDir, p.plan === null, p.undoing, p.pendingApply];
}});
out.answers = answers.map(([kind, env]) => {{
  const p = fresh(); p.resetSections(); p.ioKind = kind; p.ioPath = "/f.md";
  p.ioAnswered(env);
  return [p.exportStatus, p.importStatus, p.writtenPath, p.plan ? p.plan.summary : null, p.lastDoc ? Object.keys(p.lastDoc.sections) : null];
}});
out.times = times.map((t) => new Date(t).toISOString());
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        home = json!(HOME),
        input = json!(input.to_string_lossy()),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    for (i, p) in paths.iter().enumerate() {
        let ours = json!([
            export::real_path(p, HOME),
            format!("file://{}", export::folder(p, HOME)),
            export::with_md(p, HOME)
        ]);
        assert_eq!(ours, theirs["paths"][i], "path {p:?}");
    }
    let mut s = State::new(HOME);
    for (i, op) in ops.as_array().unwrap().iter().enumerate() {
        match op[0].as_str().unwrap() {
            "reset" => s.sections = None,
            "set" => {
                let id = atmos_ui::omafile::selectable_sections()
                    .into_iter()
                    .find(|o| o.id == op[1].as_str().unwrap())
                    .unwrap()
                    .id;
                s.set_section(id, op[2].as_bool().unwrap())
            }
            _ => s.set_all(op[1].as_bool().unwrap()),
        }
        assert_eq!(
            json!([s.chosen_keys(), s.sections_says()]),
            theirs["sections"][i],
            "op {op}"
        );
    }
    let (now, total, key) = export::step(progress, 5);
    assert_eq!(json!([now, total, key]), theirs["step"]);
    for (i, f) in finishes.as_array().unwrap().iter().enumerate() {
        let mut s = State {
            undoing: f[0].as_bool().unwrap(),
            applied_count: f[4].as_u64().unwrap() as usize,
            last_backup_dir: f[5].as_str().unwrap().into(),
            plan: Some(json!({"changes": []})),
            pending_apply: true,
            ..State::default()
        };
        s.finished(
            f[1] == json!(0),
            f[2].as_str().unwrap(),
            f[3].as_str().unwrap(),
        );
        let ours = json!([
            s.import_status,
            s.applied_count,
            s.last_backup_dir,
            s.plan.is_none(),
            s.undoing,
            s.pending_apply
        ]);
        assert_eq!(ours, theirs["finishes"][i], "finish {f}");
    }
    let snap = snapshot();
    let opts = Options {
        hardware: "ThinkPad".into(),
        workspace_gesture_unmanaged: false,
    };
    for (i, a) in answers.as_array().unwrap().iter().enumerate() {
        let mut s = State::new(HOME);
        s.io = Some(match a[0].as_str().unwrap() {
            "write" => "write",
            "open" => "open",
            _ => "read",
        });
        let env = &a[1];
        let why = export::error_text(
            env["error"]["message"].as_str().unwrap_or_default(),
            env["error"]["context"].as_str(),
        );
        let result = env.get("result").cloned().unwrap_or(Value::Null);
        let reply = if env["ok"] == json!(true) {
            Ok(&result)
        } else {
            Err(why.as_str())
        };
        s.answered(reply, "/f.md", &snap, &opts);
        let doc = s.last_doc.as_ref().map(|d| {
            json!(
                d["sections"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>()
            )
        });
        let ours = json!([
            s.export_status,
            s.import_status,
            s.written_path,
            s.plan.as_ref().map(|p| p["summary"].clone()),
            doc
        ]);
        assert_eq!(ours, theirs["answers"][i], "answer {a}");
    }
    for (i, t) in times.iter().enumerate() {
        assert_eq!(json!(export::iso(*t)), theirs["times"][i]);
    }

    let src = std::fs::read_to_string(qml::repo().join("pages/ExportPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "PrefsButton { text: \"All\" enabled: !root.working onClicked: root.setAllSections(true) }",
        "PrefsButton { text: \"None\" enabled: !root.working onClicked: root.setAllSections(false) }",
        "label: modelData.title description: modelData.note caption: modelData.count + \" settings\"",
        "label: \"Where to write it\" description: root.exportPath",
        "? root.exportStatus : \"Writes the sections selected above.\"",
        "text: \"Export\" primary: true enabled: !root.working && root.chosenKeys.length > 0 onClicked: root.doExport()",
        "text: \"Open file\" enabled: !root.working && root.writtenPath.length > 0 onClicked: root.openFile(root.writtenPath)",
        "description: root.importPath.length > 0 ? root.importPath : \"No file chosen yet.\"",
        ": \"Reads the file and compares it against this machine. Changes nothing.\"",
        "text: \"Review\" enabled: !root.working && root.importPath.length > 0 onClicked: root.doReview()",
        "available: !!root.lastDoc && SettingsJs.fileSummary(root.lastDoc, root.plan).length > 0",
        "label: \"Changes\" description: root.planSummary available: root.planHasChanges",
        "available: !!root.plan && root.plan.warnings.length > 0",
        "available: !!root.plan && root.plan.blocked.length > 0",
        "description: root.plan ? SettingsJs.applyForecast(root.plan) : \"Runs exactly the changes listed above.\" available: root.planHasChanges",
        "text: \"Apply…\" danger: true enabled: !root.working onClicked: applyConfirm.ask()",
        "? root.stepKey + \" (\" + root.stepNow + \" of \" + root.stepTotal + \")\" : (Omarchy.sudoPromptOpen ? \"Waiting for sudo mode…\" : \"Working…\")",
        "available: root.applyingJob || root.pendingApply",
        "? \"Restores the \" + root.appliedCount + \" values that import replaced. Try the machine first.\" : \"Restores the values the last import replaced.\"",
        "available: root.lastBackupDir.length > 0",
        "text: \"Put it back…\" primary: root.appliedCount > 0 enabled: !root.working && root.lastBackupDir.length > 0 onClicked: undoConfirm.ask()",
        "title: \"Write an Omafile\" fileMode: FileDialog.SaveFile nameFilters: [\"Omafile (*.md)\", \"All files (*)\"] onAccepted: root.exportPath = root.withMarkdownSuffix(",
        "title: \"Open an Omafile\" fileMode: FileDialog.OpenFile nameFilters: [\"Omafile (*.md)\", \"All files (*)\"] currentFolder: root.folderUrl(root.importPath) onAccepted: root.importPath =",
        "title: \"Apply this Omafile?\" message: root.plan ? SettingsJs.applyConfirmMessage(root.plan) : \"\" confirmText: \"Apply\" onConfirmed: { if (SettingsJs.hasCommandImport(root.plan)) commandConfirm.ask() else root.doApply() }",
        "title: \"These commands will run\" message: root.plan ? SettingsJs.commandConfirmMessage(root.plan) : \"\" confirmText: \"Apply commands\" onConfirmed: root.doApply()",
        "title: \"Undo the last import?\" message: \"Puts back the values the last import replaced.\" confirmText: \"Undo\" onConfirmed: root.doUndo()",
        "root.lastUndoNeedsRoot = SettingsJs.passwordCount(root.plan) > 0 Omarchy.runJob(Requests.job(\"settings.apply\", { plan: SettingsJs.planToJson(root.plan) }), \"settings-import\", { sudo: root.lastUndoNeedsRoot,",
        "Omarchy.runJob(Requests.job(\"settings.undo\", { plan: root.lastBackupDir + \"/undo.json\" }), \"settings-undo\", { sudo: root.lastUndoNeedsRoot,",
        "root.importStatus = \"Nothing applied: these changes need sudo mode.\"",
        "exported: new Date().toISOString(), hardware: Omarchy.dmiProduct",
        "root.startIo(\"write\", Requests.hostWrite(path, text))",
        "root.startIo(\"read\", Requests.hostRead([root.realPath(root.importPath)]))",
        "root.startIo(\"open\", Requests.hostOpen(root.realPath(path)))",
        "root.exportPath = root.home + \"/\" + root.suggestedName()",
    ] {
        assert!(flat.contains(want), "{want}");
    }
}

/// Export writes the file `atmosd` is told to, and says so; Review reads a file and shows its
/// plan; Apply asks, and then starts the import with the plan it showed.
#[test]
fn export_writes_review_plans_and_apply_starts_the_import() {
    let mut app = TestAppContext::single();
    let mut w = open(&mut app, "export", |request: &Value| {
        match request["op"].as_str() {
            Some("host.read") => {
                Reply::Ok(json!({"files": [{"path": request["paths"][0], "text": FILE}]}))
            }
            Some("host.write") => Reply::Ok(json!({"path": request["path"]})),
            _ => ordinary(request),
        }
    });
    w.cx.simulate_resize(size(px(1400.0), px(9000.0)));
    for topic in [
        "settings.look",
        "settings.host",
        "settings.system",
        "settings.network",
        "settings.accounts",
    ] {
        w.sample(topic, 1, snapshot());
    }
    let export = "ctl-export/write-the-file-export/button";
    w.until("the button", |w| w.drawn(export));
    w.settle();
    w.click(export);
    w.until("the write", |w| !w.daemon.of("host.write").is_empty());
    let write = &w.daemon.of("host.write")[0];
    assert!(
        write["path"].as_str().unwrap().ends_with("/Omafile.md"),
        "{write}"
    );
    assert!(
        write["text"]
            .as_str()
            .unwrap()
            .contains("theme = \"theme-a\""),
        "{write}"
    );
    let catalog = w.atmos.read_with(w.cx, |a, _| a.catalog().clone());
    let status = |w: &mut common::window::Win| {
        catalog.read_with(w.cx, |p, _| {
            atmos_ui::pages::export::held_state(p).export_status.clone()
        })
    };
    w.until("it says so", |w| status(w).starts_with("Wrote "));

    catalog.update(w.cx, |p, cx| {
        atmos_ui::pages::export::chose(p, false, "/in/Omafile.md".into());
        cx.notify()
    });
    w.settle();
    w.click("ctl-export/see-what-it-would-do/button");
    w.until("the read", |w| !w.daemon.of("host.read").is_empty());
    assert_eq!(
        w.daemon.of("host.read")[0]["paths"],
        json!(["/in/Omafile.md"])
    );
    w.until("the plan", |w| {
        w.drawn("ctl-export/apply-the-plan-you-just-read/button")
    });
    w.settle();
    w.click("ctl-export/apply-the-plan-you-just-read/button");
    let asked = |w: &mut common::window::Win| catalog.read_with(w.cx, |p, _| p.asked());
    assert_eq!(asked(&mut w), Some("Apply this Omafile?".to_owned()));
    w.until("the confirm", |w| w.drawn("catalog-confirm-go/button"));
    let last = std::cell::Cell::new(None);
    w.until("the dialog still", |w| {
        let now = w.bounds("catalog-confirm-go/button");
        last.replace(now) == now
    });
    w.click("catalog-confirm-go/button");
    // The file installs a command, so a second confirm says which.
    w.until("the commands", |w| {
        asked(w) == Some("These commands will run".to_owned())
    });
    w.until("the second confirm", |w| {
        w.drawn("catalog-confirm-go/button")
    });
    w.settle();
    w.click("catalog-confirm-go/button");
    w.until("the job", |w| !w.daemon.of("job.start").is_empty());
    let start = &w.daemon.of("job.start")[0];
    assert_eq!(
        (&start["name"], &start["label"]),
        (&json!("settings.apply"), &json!("settings-import"))
    );
    let plan: Value = serde_json::from_str(start["args"]["plan"].as_str().unwrap()).unwrap();
    let keys: Vec<&str> = plan["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["bindings", "hyprLook.gapsIn", "theme"]);
}
