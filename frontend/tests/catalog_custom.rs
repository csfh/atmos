//! The rows a page module draws itself are their QML rows (docs/adr/0021): what they build from
//! what the window holds is the QML's own JavaScript, run under Node, and their texts and the
//! setters they call are the QML page's. What a setter then sends is `catalog_setters.rs`'s.
//! Without `node` the tests that need it say so and stop.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::catalog::{self, Control};
use atmos_ui::kernel;
use common::qml;
use rows::{nth, page_src, property, qml_row, qml_when};
use serde_json::{Value, json};

/// The Kernel page's rows.
#[test]
fn the_kernel_rows_are_the_qml_rows() {
    let page = catalog::page("system/kernel").unwrap();
    let src = page_src(page);
    assert!(src.contains("readonly property var diag: DiagJs.normalize(Omarchy.diagnostics)"));
    let row = |label: &str| {
        let line = page.lines().find(|l| l.label == label).unwrap();
        assert!(
            matches!(line.control, Control::Custom(_)),
            "{label} is drawn by its page"
        );
        qml_row(page, label, nth(page, line))
    };
    let running = row("Running kernel");
    let summary = format!(
        "DiagJs.kernelSummary(root.diag) || {}",
        json!(kernel::UNREAD)
    );
    assert!(
        running.contains(&format!("description: {summary}")),
        "{running}"
    );
    assert!(running.contains("onClicked: Omarchy.copyText(DiagJs.kernelSummary(root.diag))"));

    let direct = row("Direct EFI boot");
    let (on, off) = (kernel::REMOVE, kernel::SET_UP);
    let when = "Omarchy.directBoot";
    assert!(direct.contains(&format!(
        "text: {when} ? {} : {}",
        json!(on.0),
        json!(off.0)
    )));
    assert!(direct.contains(&format!("enabled: {}", qml_when(kernel::DIRECT_ENABLED))));
    assert!(direct.contains("onClicked: directBootConfirm.ask()"));
    let confirm = src.split("id: directBootConfirm").nth(1).unwrap();
    let confirm = confirm.split("\n  }").next().unwrap();
    let flat = confirm.split_whitespace().collect::<Vec<_>>().join(" ");
    for (name, a, b) in [
        ("title", on.1.title, off.1.title),
        ("message", on.1.message, off.1.message),
        ("confirmText", on.1.action, off.1.action),
    ] {
        let want = format!("{name}: {when} ? {} : {}", json!(a), json!(b));
        assert!(flat.contains(&want), "{want}");
    }
    assert!(flat.contains("onConfirmed: Omarchy.setupDirectBoot()"));

    let swap = row("Lower swappiness");
    let flat = swap.split_whitespace().collect::<Vec<_>>().join(" ");
    let on = "Omarchy.tweaks && Omarchy.tweaks.swappiness";
    assert!(flat.contains(&format!(
        "description: {on} ? {} : {}",
        json!(kernel::SWAPPINESS_ON),
        json!(kernel::SWAPPINESS_OFF)
    )));
    assert!(flat.contains(&format!("checked: !!({on})")));
    assert!(flat.contains(&format!(
        "onToggled: Omarchy.setTweak(\"swappiness\", !({on}))"
    )));
    assert!(flat.contains("onClicked: Omarchy.setTweak(\"swappiness\", false)"));
}

/// The running kernel is `kernelSummary(normalize(diagnostics))`.
#[test]
fn the_kernel_summary_is_the_qml_s() {
    let docs = vec![
        json!({"kernel": {"sysname": "Linux", "release": "6.17.1-arch1-1", "machine": "x86_64"}}),
        json!({"kernel": {"sysname": " Linux\t", "release": "6.1\r\nx", "machine": null}}),
        json!({"kernel": {"release": "x".repeat(80)}}),
        json!({"kernel": {"sysname": "my secret"}}),
        json!({"kernel": "Linux"}),
        json!({}),
        Value::Null,
    ];
    let script = format!(
        r#"
const D = require(process.env.REPO_DIR + "/tests/harness.js").load("services/Diagnostics.js");
process.stdout.write(JSON.stringify({}.map((d) => D.kernelSummary(D.normalize(d)))));
"#,
        json!(docs)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<String> = docs
        .iter()
        .map(|d| kernel::kernel_summary(Some(d)))
        .collect();
    assert_eq!(json!(ours), theirs);
}

/// The Notifications page's form and its list of reminders.
#[test]
fn the_notifications_rows_are_the_qml_rows() {
    use atmos_ui::notifications;
    let page = catalog::page("notifications").unwrap();
    let src = page_src(page);
    assert!(src.contains(&format!(
        "property int reminderMinutes: {}",
        notifications::MINUTES.0
    )));
    let line = page.lines().find(|l| l.label == "New reminder").unwrap();
    let form = qml_row(page, "New reminder", nth(page, line));
    assert!(form.contains(&format!("from: {}", notifications::MINUTES.1)));
    assert!(form.contains(&format!("to: {}", notifications::MINUTES.2)));
    assert!(form.contains("placeholder: \"Message\""));
    assert!(form.contains(
        "onClicked: Omarchy.setReminder(String(root.reminderMinutes), root.reminderMessage)"
    ));
    // The list: the `Repeater` over the reminders, its row's words evaluated under Node.
    let list = src
        .split("model: Omarchy.reminders")
        .nth(1)
        .expect("a Repeater over the reminders");
    let label = property(list, "label");
    let description = property(list, "description");
    assert_eq!(
        property(list, "hint"),
        json!(notifications::HINT).to_string()
    );
    assert!(list.contains("onClicked: Omarchy.copyText(RichUi.reminderCopyText(modelData))"));
    assert!(list.contains("onClicked: Omarchy.showReminders()"));
    let items = vec![
        json!({"label": "Tea", "remaining": "4m", "atTime": "10:30"}),
        json!({"label": "", "remaining": "1h", "message": "Stand\nup"}),
        json!({"message": "Call", "atTime": "9:00"}),
        json!({}),
    ];
    let script = format!(
        r#"
const RichUi = require(process.env.REPO_DIR + "/tests/harness.js").load("services/RichUi.js");
const label = new Function("modelData", "return (" + {label} + ");");
const description = new Function("modelData", "return (" + {description} + ");");
process.stdout.write(JSON.stringify({items}.map((m) => [label(m), description(m), RichUi.reminderCopyText(m)])));
"#,
        items = json!(items),
        label = json!(label),
        description = json!(description),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<Value> = items
        .iter()
        .map(|m| {
            json!([
                notifications::label(m),
                notifications::description(m),
                notifications::copy_text(m)
            ])
        })
        .collect();
    assert_eq!(json!(ours), theirs);
}

/// Agentic and MCP: what they say of the agents is what the pages' own functions and
/// expressions say, run under Node on the same lists.
#[test]
fn the_agent_pages_say_what_their_qml_says() {
    use atmos_ui::agentic;
    let agents = json!([
        {"id": "claude", "name": "Claude Code", "installed": true, "writer": true, "state": "ours", "on": true, "path": "/h/.claude.json"},
        {"id": "grok", "installed": true, "writer": true, "state": "absent"},
        {"id": "cursor", "name": "Cursor", "installed": true, "writer": true, "state": "custom"},
        {"id": "codex", "installed": true, "writer": false, "state": "stale"},
        {"id": "pi", "name": "Pi", "installed": false},
    ]);
    let list = agents.as_array().unwrap().clone();
    let agentic_src = page_src(catalog::page("agentic").unwrap());
    let mcp_src = page_src(catalog::page("agentic/mcp").unwrap());
    let group = |src: &str, title: &str| {
        let block = src
            .split(&format!("title: \"{title}\""))
            .nth(1)
            .unwrap()
            .to_owned();
        property(&block, "detail")
    };
    let repeater = |src: &str, model: &str| {
        src.split(&format!("model: {model}"))
            .nth(1)
            .unwrap()
            .to_owned()
    };
    let agent_rows = repeater(&agentic_src, "root.agents");
    let mcp_rows = repeater(&mcp_src, "root.installed");
    assert!(agent_rows.contains("text: modelData.installed ? \"Use\" : \"Install\""));
    assert!(agent_rows.contains("visible: Omarchy.agent !== modelData.id"));
    assert!(agentic_src.contains("Omarchy.set(\"agent\", id)"));
    let mcp_toggle = "enabled: modelData.writer === true && (modelData.state === \"absent\" || modelData.state === \"ours\")";
    assert!(mcp_rows.contains(mcp_toggle));
    assert!(mcp_rows.contains("visible: modelData.writer === true && (modelData.state === \"custom\" || modelData.state === \"stale\")"));
    assert!(
        mcp_src.contains(
            "root.checkText = names.length + (names.length === 1 ? \" tool\" : \" tools\")"
        )
    );
    let exprs = json!({
        "agentsDetail": group(&agentic_src, "Agents"),
        "mcpDetail": group(&mcp_src, "Installed agents"),
        "mcpHint": property(&mcp_rows, "hint"),
        "mcpCaption": property(&mcp_rows, "caption"),
    });
    let script = format!(
        r#"{root}
const agents = {agents};
const e = {exprs};
const at = (r, expr, m) => new Function("root", "Omarchy", "modelData", "return (" + expr + ");")(r, r.__o, m);
const out = [];
for (const [agent, error] of [["claude", ""], ["nobody", ""], ["", "Could not read agents"]]) {{
  const O = {{ agent }};
  const r = pageRoot("AgenticPage.qml", O, {{ agents, loadError: error }});
  r.__o = O;
  out.push([r.installedCount, r.knownAgent, at(r, e.agentsDetail), agents.map((a) => r.rowDescription(a))]);
}}
const m = pageRoot("agentic/McpPage.qml", {{}}, {{ agents }});
m.__o = {{}};
out.push([m.installed.map((a) => a.id), m.installable, at(m, e.mcpDetail),
  m.installed.map((a) => [m.rowDescription(a), at(m, e.mcpHint, a), at(m, e.mcpCaption, a)])]);
const R = load("services/Requests.js");
out.push([R.agentsMcpSet("grok", true, false), R.agentsMcpSet("cursor", false, true)]);
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        agents = agents,
        exprs = exprs
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let mut ours = Vec::new();
    for (agent, error) in [
        ("claude", ""),
        ("nobody", ""),
        ("", "Could not read agents"),
    ] {
        ours.push(json!([
            agentic::installed_count(&list),
            agentic::known_agent(&list, agent),
            agentic::agents_detail(&list, error),
            list.iter()
                .map(|a| agentic::agent_description(a, agent))
                .collect::<Vec<_>>()
        ]));
    }
    let installed = agentic::installed(&list);
    ours.push(json!([
        installed
            .iter()
            .map(|a| a["id"].clone())
            .collect::<Vec<_>>(),
        agentic::installable(&list).len(),
        agentic::mcp_detail(&list),
        installed
            .iter()
            .map(|a| json!([
                agentic::mcp_description(a),
                agentic::mcp_hint(a),
                agentic::mcp_caption(a)
            ]))
            .collect::<Vec<_>>()
    ]));
    ours.push(json!([
        atmos_ui::protocol::agents_mcp_set("grok", true, false),
        atmos_ui::protocol::agents_mcp_set("cursor", false, true)
    ]));
    assert_eq!(json!(ours), theirs);
}

/// Environment: what it says of the session and what Add and Remove send are the page's own
/// code, run under Node.
#[test]
fn the_environment_rows_are_the_qml_rows() {
    use atmos_ui::environment as env;
    let page = catalog::page("system/environment").unwrap();
    let src = page_src(page);
    let row = |label: &str| {
        let line = page.lines().find(|l| l.label == label).unwrap();
        qml_row(page, label, nth(page, line))
    };
    let exprs = json!({
        "session": property(&row("Session"), "valueText"),
        "shell": property(&row("Shell"), "valueText"),
        "tools": property(&row("Editor / terminal / browser"), "valueText"),
        "path": property(&row("PATH"), "description"),
        "add": property(&row("Add a variable"), "description"),
    });
    let prepend = row("PATH prepend");
    assert!(prepend.contains("placeholder: \"/opt/bin\""));
    assert!(prepend.contains("enabled: root.pathPrependDraft !== Omarchy.envPathPrepend"));
    assert!(
        prepend.contains("onClicked: Omarchy.setEnvVars(Omarchy.envVars, root.pathPrependDraft)")
    );
    assert!(src.contains(
        "function onEnvPathPrependChanged() { root.pathPrependDraft = Omarchy.envPathPrepend }"
    ));
    let remove = src.split("model: Omarchy.envVars").nth(1).unwrap();
    assert!(remove.contains("if (list[i] && list[i].key !== modelData.key) vars.push(list[i])"));
    assert!(remove.contains("Omarchy.setEnvVars(vars, Omarchy.envPathPrepend)"));
    assert_eq!(
        property(remove, "label"),
        "modelData && modelData.key ? modelData.key : \"VAR\""
    );
    assert_eq!(
        property(remove, "description"),
        "modelData && modelData.value ? modelData.value : \"\""
    );
    let detected = json!([
        {"sessionType": "wayland", "desktop": "Hyprland", "shell": "/bin/zsh", "editor": "nvim", "browser": "zen", "path": "/usr/bin:/bin"},
        {"sessionType": "", "terminal": "ghostty\nx", "path": "x".repeat(1100)},
        {},
        "junk",
    ]);
    let adds = json!([
        ["EDITOR", "nvim"],
        [" PAGER ", "less\n"],
        ["PATH", "/x"],
        ["9x", "y"],
        ["EDITOR", "vim"]
    ]);
    let vars = json!([{"key": "EDITOR", "value": "code"}, null, {"key": "LANG", "value": "C"}]);
    let script = format!(
        r#"{root}
const e = {exprs};
const at = (r, expr) => new Function("root", "Omarchy", "return (" + expr + ");")(r, r.__o);
const out = [];
for (const d of {detected}) {{
  const O = {{ envDetected: d }};
  const r = pageRoot("system/EnvironmentPage.qml", O, {{}});
  r.__o = O;
  out.push([at(r, e.session), at(r, e.shell), at(r, e.tools), at(r, e.path)]);
}}
for (const [k, v] of {adds}) {{
  let sent = null;
  const O = {{ envVars: {vars}, envPathPrepend: "/opt", setEnvVars: (a, b) => {{ sent = [a, b]; }} }};
  const r = pageRoot("system/EnvironmentPage.qml", O, {{ varKeyDraft: k, varValueDraft: v, varError: "" }});
  r.__o = O;
  r.addVar();
  out.push([sent, r.varError, at(r, e.add)]);
}}
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        exprs = exprs,
        detected = detected,
        adds = adds,
        vars = vars
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let mut ours = Vec::new();
    for d in detected.as_array().unwrap() {
        let d = Some(d);
        ours.push(json!([
            env::session(d),
            env::shell(d),
            env::tools(d),
            env::path(d)
        ]));
    }
    for pair in adds.as_array().unwrap() {
        let (k, v) = (pair[0].as_str().unwrap(), pair[1].as_str().unwrap());
        ours.push(match env::add_var(&vars, k, v) {
            Ok(next) => json!([[next, "/opt"], "", env::ADD]),
            Err(why) => json!([null, why, why]),
        });
    }
    assert_eq!(json!(ours), theirs);
    // Remove, as its `onClicked` filters the list.
    let gone = env::remove_var(&vars, Some(&json!("EDITOR")));
    assert_eq!(json!(gone), json!([{"key": "LANG", "value": "C"}]));
}

/// Startup: its lists and what each line says are the page's own code, run under Node, and its
/// form, switch and Remove… call what the QML calls.
#[test]
fn the_startup_rows_are_the_qml_rows() {
    use atmos_ui::startup;
    let page = catalog::page("applications/startup").unwrap();
    let src = page_src(page);
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    for want in [
        "placeholder: \"hyprsunset\"",
        "from: 0 to: 600 value: root.delayDraft",
        "onClicked: Omarchy.addAutostart(root.autostartDraft, root.delayDraft)",
        "onSubmitted: function(value) { Omarchy.addAutostart(value, root.delayDraft) }",
        "checked: modelData && modelData.enabled !== false",
        "enabled: modelData && modelData.managed",
        "onToggled: Omarchy.setAutostartEnabled(modelData.command, !(modelData && modelData.enabled !== false))",
        "title: \"Remove startup command\"",
        "message: \"Remove \" + root.pendingAutostart + \" from the Atmos autostart block?\"",
        "onConfirmed: Omarchy.removeAutostart(root.pendingAutostart)",
        "hint: \"~/.config/hypr/autostart.lua\"",
        "available: root.failed.length === 0 label: \"Startup failures\"",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    assert_eq!(
        startup::remove_message("mako"),
        "Remove mako from the Atmos autostart block?"
    );
    let lines = src.split("model: root.rows").nth(1).unwrap();
    let failed = src.split("model: root.failed").nth(1).unwrap();
    let exprs = json!({
        "label": property(lines, "label"),
        "description": property(lines, "description"),
        "unit": property(failed, "label"),
        "why": property(failed, "description"),
    });
    let autostart = json!([
        {"command": "waybar", "managed": false},
        {"command": "hyprsunset", "delay": 5, "enabled": false, "managed": true},
        {"command": "", "managed": true},
        {"command": "mako", "managed": true},
        null,
    ]);
    let units = json!([
        {"unit": "a.service", "active": "failed"},
        {"unit": "b.service", "active": "active", "sub": "failed", "description": "B"},
        {"active": "active", "sub": "running"},
        {"active": "failed"},
    ]);
    let script = format!(
        r#"{root}
const e = {exprs};
const at = (expr, m) => new Function("modelData", "return (" + expr + ");")(m);
const r = pageRoot("applications/StartupPage.qml", {{ autostart: {autostart}, systemdUnits: {units} }}, {{}});
process.stdout.write(JSON.stringify([
  r.rows.map((m) => [at(e.label, m), at(e.description, m)]),
  r.failed.map((m) => [at(e.unit, m), at(e.why, m)]),
]));
"#,
        root = qml::PAGE_ROOT,
        exprs = exprs,
        autostart = autostart,
        units = units
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours = json!([
        startup::rows(Some(&autostart))
            .iter()
            .map(|r| json!([atmos_ui::js::string(&r["command"]), startup::description(r)]))
            .collect::<Vec<_>>(),
        startup::failed(Some(&units))
            .iter()
            .map(|u| {
                let or = |name: &str, fallback: &str| {
                    u.get(name)
                        .filter(|v| atmos_ui::js::truthy(Some(v)))
                        .map_or_else(|| fallback.to_owned(), atmos_ui::js::string)
                };
                json!([or("unit", "unit"), or("description", "Failed this boot.")])
            })
            .collect::<Vec<_>>(),
    ]);
    assert_eq!(ours, theirs);
}
