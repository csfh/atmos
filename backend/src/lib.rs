//! Atmos settings and display-data backend.
//!
//! `main` calls [`run`]. Quickshell sends the same requests for every system
//! backend. Omarchy writes the Hyprland drop-ins, shell config, and the other
//! files those settings already use. The plain backend stores the same keys
//! in its own config tree.

mod display;
pub mod domain;
pub mod effect;
mod effects;
mod patch;
mod store;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Map, Value};

pub const VERSION: &str = "0.1.0";

pub fn run(args: &[String], stdin: &str, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match dispatch(args, stdin, stdout, stderr) {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(stderr, "atmos-backend: {err}");
            1
        }
    }
}

fn dispatch(
    args: &[String],
    stdin: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32, String> {
    let parsed = parse_args(args)?;
    match parsed.command.as_str() {
        "version" => {
            writeln!(stdout, "{VERSION}").map_err(|err| err.to_string())?;
            stdout.flush().map_err(|err| err.to_string())?;
            Ok(0)
        }
        "snapshot" => {
            let root = parsed.root.ok_or("snapshot needs --root")?;
            let doc = snapshot_document(&parsed.backend, &root, parsed.sampler.as_deref())?;
            write_json(stdout, &doc)?;
            Ok(0)
        }
        "request" => {
            let request: Value =
                serde_json::from_str(stdin.trim()).map_err(|err| format!("request: {err}"))?;
            let response = handle(
                &parsed.backend,
                parsed.root.as_deref(),
                &request,
                parsed.sampler.as_deref(),
            )?;
            let code = if response.get("ok").and_then(|v| v.as_bool()) == Some(true) {
                0
            } else {
                1
            };
            write_json(stdout, &response)?;
            Ok(code)
        }
        "display" => {
            let kind = parsed.rest.first().map(String::as_str).unwrap_or("");
            if kind.is_empty() {
                return Err("display needs a kind".into());
            }
            let value = display::load(
                parsed.root.as_deref(),
                &parsed.backend,
                kind,
                parsed.sampler.as_deref(),
            )?;
            write_json(stdout, &value)?;
            Ok(0)
        }
        "display-snapshot" => {
            let kinds: Vec<&str> = if parsed.rest.is_empty() {
                display::KINDS.to_vec()
            } else {
                parsed.rest.iter().map(String::as_str).collect()
            };
            for kind in &kinds {
                if !display::is_kind(kind) {
                    return Err(format!("unknown display kind {kind}"));
                }
            }
            let mut display = Map::new();
            for kind in kinds {
                let value = match display::load(
                    parsed.root.as_deref(),
                    &parsed.backend,
                    kind,
                    parsed.sampler.as_deref(),
                ) {
                    Ok(value) => value,
                    Err(err) => stamp_error(&parsed.backend, kind, &err),
                };
                display.insert(kind.to_string(), value);
            }
            write_json(stdout, &Value::Object(display))?;
            Ok(0)
        }
        "gui-snapshot" => {
            let group = parsed.rest.first().map(String::as_str).unwrap_or("all");
            Ok(gui_snapshot(group, stderr))
        }
        "apply" => Ok(apply(&parsed.rest, stderr)),
        "" => {
            let _ = writeln!(stderr, "atmos-backend: missing command");
            usage(stderr);
            Ok(2)
        }
        other => {
            let _ = writeln!(stderr, "atmos-backend: unknown command {other}");
            usage(stderr);
            Ok(2)
        }
    }
}

struct Parsed {
    backend: String,
    root: Option<PathBuf>,
    sampler: Option<PathBuf>,
    command: String,
    rest: Vec<String>,
}

fn parse_args(args: &[String]) -> Result<Parsed, String> {
    let mut backend = String::from("omarchy");
    let mut root = None;
    let mut sampler = None;
    let mut positionals = Vec::new();
    let mut index = 1;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            positionals.extend(args[index + 1..].iter().cloned());
            break;
        }
        if arg == "--backend" {
            index += 1;
            backend = args.get(index).cloned().ok_or("--backend needs a name")?;
            index += 1;
            continue;
        }
        if arg == "--root" {
            index += 1;
            let value = args.get(index).cloned().ok_or("--root needs a directory")?;
            root = Some(PathBuf::from(value));
            index += 1;
            continue;
        }
        if arg == "--sampler" {
            index += 1;
            let value = args.get(index).cloned().ok_or("--sampler needs a path")?;
            sampler = Some(PathBuf::from(value));
            index += 1;
            continue;
        }
        if arg.starts_with('-') {
            return Err(format!("unknown argument {arg}"));
        }
        positionals.push(arg.clone());
        index += 1;
    }
    platform(&backend)?;
    if let Some(dir) = &root {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    let command = positionals.first().cloned().unwrap_or_default();
    let rest = if positionals.is_empty() {
        Vec::new()
    } else {
        positionals[1..].to_vec()
    };
    Ok(Parsed {
        backend,
        root,
        sampler,
        command,
        rest,
    })
}

fn handle(
    backend: &str,
    root: Option<&Path>,
    request: &Value,
    sampler: Option<&Path>,
) -> Result<Value, String> {
    let op = request.get("op").and_then(Value::as_str).unwrap_or("");
    let result = match op {
        "version" => Value::String(VERSION.into()),
        "platform" => platform(backend)?,
        "settings.list" => Value::Array(settings_list(backend)?),
        "settings.get" => {
            let domain = field(request, "domain")?;
            read_domain(backend, root, domain)?
        }
        "settings.set" => {
            let domain = field(request, "domain")?;
            let value = request.get("value").ok_or("missing value")?;
            write_domain(backend, root, domain, value)?;
            let place = place_for(backend, domain)?;
            serde_json::json!({
                "domain": domain,
                "value": read_domain(backend, root, domain)?,
                "file": place.rel,
                "encoding": place.encoding(),
            })
        }
        "settings.snapshot" => {
            let group = request
                .get("group")
                .and_then(Value::as_str)
                .unwrap_or("all");
            if root.is_none() && backend == "omarchy" {
                Value::Object(live_settings_snapshot(group)?)
            } else {
                Value::Object(settings_snapshot(backend, root)?)
            }
        }
        "display.get" => {
            let kind = field(request, "kind")?;
            display::load(root, backend, kind, sampler)?
        }
        "display.snapshot" => display_snapshot(backend, root, sampler)?,
        other => return Ok(error_envelope(backend, &format!("unknown op {other}"))),
    };
    Ok(ok_envelope(backend, result))
}

fn settings_list(backend: &str) -> Result<Vec<Value>, String> {
    let mut specs: Vec<_> = domain::specs().iter().collect();
    specs.sort_by_key(|spec| spec.key);
    let mut out = Vec::new();
    for spec in specs {
        let place = domain::locate(backend, spec)?;
        out.push(serde_json::json!({
            "domain": spec.key,
            "type": spec.ty.name(),
            "file": place.rel,
            "encoding": place.encoding(),
            "prefix": place.prefix(),
        }));
    }
    Ok(out)
}

fn read_domain(backend: &str, root: Option<&Path>, key: &str) -> Result<Value, String> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    if backend == "omarchy" {
        if let Some(effect::Effect::Command { .. }) = effect::get(key) {
            return effect::read_command(root, key);
        }
    }
    let place = domain::locate(backend, spec)?;
    store::read_place(root, &place, spec.key, spec.ty)
}

fn write_domain(
    backend: &str,
    root: Option<&Path>,
    key: &str,
    value: &Value,
) -> Result<(), String> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    if !spec.ty.accepts(value) {
        return Err(format!("{key} expects {}", spec.ty.name()));
    }
    if backend == "omarchy" {
        if let Some(effect::Effect::Command { argv }) = effect::get(key) {
            return effect::apply_command(root, key, argv, value);
        }
    }
    let place = domain::locate(backend, spec)?;
    match store::write_place(root, &place, spec.key, spec.ty, value) {
        Ok(()) => {}
        Err(err) if root.is_none() && backend == "omarchy" && permission_denied(&err) => {
            delegate_root_script(key, value)?;
        }
        Err(err) => return Err(err),
    }
    if backend == "omarchy" {
        effects::after_write(root, key, value)?;
    }
    if root.is_none() && backend == "omarchy" {
        if matches!(
            place.kind,
            domain::PlaceKind::Lua { .. }
                | domain::PlaceKind::Flag
                | domain::PlaceKind::Hypr { .. }
        ) {
            spawn_command("hyprctl", &["reload"]);
        }
        // Schedule and temperature land in hyprsunset.conf. The nightlight
        // switch is `omarchy toggle nightlight` and must not reload that file.
        if key.starts_with("nightlight") && key != "nightlight" {
            spawn_command("omarchy", &["restart", "hyprsunset"]);
        }
        run_live_command(key, value);
        if key == "tweaks.swappiness" {
            let flag = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            let _ = run_script("set-tweaks.sh", &["swappiness", flag]);
        }
    }
    Ok(())
}

fn permission_denied(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    lower.contains("permission denied") || lower.contains("os error 13")
}

fn delegate_root_script(key: &str, value: &Value) -> Result<(), String> {
    let rendered = scalar_arg(value)?;
    match key {
        "hostname" => run_script("set-hostname.sh", &[&rendered]),
        "timezone" => run_script("set-timezone.sh", &[&rendered]),
        "locale" => run_script("set-locale.sh", &[&rendered]),
        "keyboardLayout" => run_script("set-keyboard-layout.sh", &[&rendered]),
        "ntp" => run_script("set-ntp.sh", &[&rendered]),
        "fullName" => run_script("set-full-name.sh", &[&rendered]),
        "parallelDownloads" => run_script("set-parallel-downloads.sh", &[&rendered]),
        "plymouth" => run_command("omarchy", &["plymouth", "set", "by", "theme", &rendered]),
        _ => Err(format!("cannot write {key} without permission")),
    }
}

fn scalar_arg(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Bool(true) => Ok("true".into()),
        Value::Bool(false) => Ok("false".into()),
        Value::Number(number) => Ok(number.to_string()),
        _ => Err("live writer needs a scalar".into()),
    }
}

fn run_script(name: &str, args: &[&str]) -> Result<(), String> {
    let root = std::env::var("ATMOS_ROOT").map_err(|_| "ATMOS_ROOT is not set".to_string())?;
    let path = PathBuf::from(root).join("scripts").join(name);
    let status = Command::new("bash")
        .arg(&path)
        .args(args)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{name} exited {}", status.code().unwrap_or(1)))
    }
}

fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited {}", status.code().unwrap_or(1)))
    }
}

fn run_live_command(key: &str, value: &Value) {
    let Ok(rendered) = scalar_arg(value) else {
        return;
    };
    let args: &[&str] = match key {
        "theme" => &["theme", "set"],
        "background" => &["theme", "bg", "set"],
        "barPosition" => &["bar", "position"],
        "barTransparent" => &["bar", "transparent"],
        "font" => &["font", "set"],
        "textSize" => &["display", "text", "size"],
        "browser" => &["default", "browser"],
        "terminal" => &["default", "terminal"],
        "editor" => &["default", "editor"],
        "agent" => &["default", "agent"],
        "dns" => &["dns"],
        "powerProfile" => &["powerprofiles", "set", "autodetect"],
        "powerProfileAc" => &["powerprofiles", "set", "ac"],
        "powerProfileBattery" => &["powerprofiles", "set", "battery"],
        "audioTuningOn" => {
            let flag = if value.as_bool() == Some(true) {
                "on"
            } else {
                "off"
            };
            spawn_command("omarchy", &["audio", "tuning", flag]);
            return;
        }
        _ => return,
    };
    let mut owned: Vec<&str> = args.to_vec();
    owned.push(&rendered);
    spawn_command("omarchy", &owned);
}

fn spawn_command(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn place_for(backend: &str, key: &str) -> Result<domain::Place, String> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    domain::locate(backend, spec)
}

fn settings_snapshot(backend: &str, root: Option<&Path>) -> Result<Map<String, Value>, String> {
    let mut snapshot = Map::new();
    for spec in domain::specs() {
        snapshot.insert(spec.key.to_string(), read_domain(backend, root, spec.key)?);
    }
    Ok(snapshot)
}

/// Live Omarchy reads merge `snapshot.sh` with the files settings.set writes.
/// A null file value does not erase a snapshot field. Dotted domains fold into
/// the nested objects the Quickshell snapshot already uses. `--root` never
/// reaches this path, so fixture launches stay file-only and deterministic.
fn live_settings_snapshot(group: &str) -> Result<Map<String, Value>, String> {
    let mut snapshot = Map::new();
    if let Ok(text) = capture_snapshot_sh(group) {
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text) {
            snapshot = map;
        }
    }
    let group_name = if group.is_empty() { "all" } else { group };
    snapshot.insert("group".into(), Value::String(group_name.into()));
    for spec in domain::specs() {
        // One unreadable system file must not drop the rest of the page.
        if let Ok(value) = read_domain("omarchy", None, spec.key) {
            overlay_domain(&mut snapshot, spec.key, value);
        }
    }
    Ok(snapshot)
}

fn overlay_domain(doc: &mut Map<String, Value>, key: &str, value: Value) {
    if value.is_null() {
        return;
    }
    // These switches are live command status. A file value must not cover them.
    if matches!(key, "nightlight" | "audioOutputMuted" | "audioInputMuted") {
        return;
    }
    if let Some((head, tail)) = key.split_once('.') {
        let entry = doc
            .entry(head.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        if let Some(map) = entry.as_object_mut() {
            map.insert(tail.to_string(), value);
        }
        return;
    }
    doc.insert(key.to_string(), value);
}

fn capture_snapshot_sh(group: &str) -> Result<String, String> {
    let root = std::env::var("ATMOS_ROOT").map_err(|_| "ATMOS_ROOT is not set".to_string())?;
    let script = PathBuf::from(root).join("scripts").join("snapshot.sh");
    let output = Command::new("bash")
        .arg(&script)
        .arg(group)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn display_snapshot(
    backend: &str,
    root: Option<&Path>,
    sampler: Option<&Path>,
) -> Result<Value, String> {
    let mut display = Map::new();
    for kind in display::KINDS {
        display.insert(
            (*kind).to_string(),
            display::load(root, backend, kind, sampler)?,
        );
    }
    Ok(Value::Object(display))
}

fn snapshot_document(backend: &str, root: &Path, sampler: Option<&Path>) -> Result<Value, String> {
    Ok(serde_json::json!({
        "version": VERSION,
        "platform": platform(backend)?,
        "settings": Value::Object(settings_snapshot(backend, Some(root))?),
        "display": display_snapshot(backend, Some(root), sampler)?,
    }))
}

fn platform(backend: &str) -> Result<Value, String> {
    match backend {
        "omarchy" => {
            Ok(serde_json::json!({"id": "omarchy", "compositor": "hyprland", "family": "arch"}))
        }
        "plain" => {
            Ok(serde_json::json!({"id": "plain", "compositor": "none", "family": "portable"}))
        }
        other => Err(format!("unknown backend {other}")),
    }
}

fn ok_envelope(backend: &str, result: Value) -> Value {
    serde_json::json!({
        "ok": true,
        "version": VERSION,
        "platform": platform(backend).unwrap_or(Value::Null),
        "result": result,
    })
}

fn error_envelope(backend: &str, message: &str) -> Value {
    serde_json::json!({
        "ok": false,
        "version": VERSION,
        "platform": platform(backend).unwrap_or(Value::Null),
        "error": message,
        "result": Value::Null,
    })
}

fn stamp_error(backend: &str, kind: &str, err: &str) -> Value {
    serde_json::json!({
        "platform": backend,
        "collector": display::collector(backend, kind),
        "error": err,
    })
}

fn field<'a>(request: &'a Value, name: &str) -> Result<&'a str, String> {
    request
        .get(name)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| format!("missing {name}"))
}

fn write_json(stdout: &mut dyn Write, value: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    writeln!(stdout, "{text}").map_err(|err| err.to_string())?;
    stdout.flush().map_err(|err| err.to_string())
}

fn gui_snapshot(group: &str, stderr: &mut dyn Write) -> i32 {
    let root = match std::env::var("ATMOS_ROOT") {
        Ok(root) => root,
        Err(_) => {
            let _ = writeln!(stderr, "atmos-backend: ATMOS_ROOT is not set");
            return 1;
        }
    };
    let script = PathBuf::from(root).join("scripts").join("snapshot.sh");
    if !script.is_file() {
        let _ = writeln!(stderr, "atmos-backend: missing {}", script.display());
        return 1;
    }
    match Command::new("bash").arg(&script).arg(group).status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            let _ = writeln!(stderr, "atmos-backend: {err}");
            1
        }
    }
}

fn apply(argv: &[String], stderr: &mut dyn Write) -> i32 {
    let Some(program) = argv.first() else {
        let _ = writeln!(stderr, "atmos-backend: apply needs a command");
        return 2;
    };
    match Command::new(program).args(&argv[1..]).status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(err) => {
            let _ = writeln!(stderr, "atmos-backend: {err}");
            1
        }
    }
}

fn usage(stderr: &mut dyn Write) {
    let _ = writeln!(
        stderr,
        "usage: atmos-backend [--backend omarchy|plain] [--root DIR] <snapshot|request|display KIND|display-snapshot|gui-snapshot|apply -- CMD|version>"
    );
}
