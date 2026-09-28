//! Atmos settings and display-data backend.
//!
//! `main` calls [`run`]. Quickshell sends the same requests for every system
//! backend. Omarchy writes the Hyprland drop-ins, shell config, and the other
//! files those settings already use. The plain backend stores the same keys
//! in its own config tree.

mod display;
mod domain;
mod store;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn dispatch(args: &[String], stdin: &str, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32, String> {
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
            let root = parsed.root.ok_or("request needs --root")?;
            let request: Value = serde_json::from_str(stdin.trim()).map_err(|err| format!("request: {err}"))?;
            let response = handle(&parsed.backend, &root, &request, parsed.sampler.as_deref())?;
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
            let value = display::load(parsed.root.as_deref(), &parsed.backend, kind, parsed.sampler.as_deref())?;
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
                let value = match display::load(parsed.root.as_deref(), &parsed.backend, kind, parsed.sampler.as_deref()) {
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

fn handle(backend: &str, root: &Path, request: &Value, sampler: Option<&Path>) -> Result<Value, String> {
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
        "settings.snapshot" => Value::Object(settings_snapshot(backend, root)?),
        "display.get" => {
            let kind = field(request, "kind")?;
            display::load(Some(root), backend, kind, sampler)?
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

fn read_domain(backend: &str, root: &Path, key: &str) -> Result<Value, String> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    let place = domain::locate(backend, spec)?;
    store::read_place(root, &place, spec.key, spec.ty)
}

fn write_domain(backend: &str, root: &Path, key: &str, value: &Value) -> Result<(), String> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    let place = domain::locate(backend, spec)?;
    store::write_place(root, &place, spec.key, spec.ty, value)
}

fn place_for(backend: &str, key: &str) -> Result<domain::Place, String> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    domain::locate(backend, spec)
}

fn settings_snapshot(backend: &str, root: &Path) -> Result<Map<String, Value>, String> {
    let mut snapshot = Map::new();
    for spec in domain::specs() {
        snapshot.insert(spec.key.to_string(), read_domain(backend, root, spec.key)?);
    }
    Ok(snapshot)
}

fn display_snapshot(backend: &str, root: &Path, sampler: Option<&Path>) -> Result<Value, String> {
    let mut display = Map::new();
    for kind in display::KINDS {
        display.insert((*kind).to_string(), display::load(Some(root), backend, kind, sampler)?);
    }
    Ok(Value::Object(display))
}

fn snapshot_document(backend: &str, root: &Path, sampler: Option<&Path>) -> Result<Value, String> {
    Ok(serde_json::json!({
        "version": VERSION,
        "platform": platform(backend)?,
        "settings": Value::Object(settings_snapshot(backend, root)?),
        "display": display_snapshot(backend, root, sampler)?,
    }))
}

fn platform(backend: &str) -> Result<Value, String> {
    match backend {
        "omarchy" => Ok(serde_json::json!({"id": "omarchy", "compositor": "hyprland", "family": "arch"})),
        "plain" => Ok(serde_json::json!({"id": "plain", "compositor": "none", "family": "portable"})),
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
