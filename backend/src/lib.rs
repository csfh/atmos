//! ratmos serves Atmos settings and display data.
//!
//! `main` calls [`run`]. Quickshell sends the same requests for every system
//! backend. Omarchy writes the Hyprland drop-ins, shell config, and the other
//! files those settings already use. The plain backend stores the same keys
//! in its own config tree.

mod agents;
mod apply;
mod display;
pub mod domain;
pub mod effect;
mod effects;
pub mod error;
mod fsutil;
mod host;
mod patch;
mod platform;
mod request;
mod runner;
mod scripts;
pub mod serve;
mod store;

use crate::error::{Error, Result};
use crate::platform::Backend;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::request::Request;
use crate::runner::Run;
use serde_json::{Map, Value};

pub const VERSION: &str = "0.1.0";

pub fn run(args: &[String], stdin: &str, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match dispatch(args, stdin, stdout, stderr) {
        Ok(code) => code,
        Err(err) => {
            let _ = writeln!(stderr, "ratmos: {err}");
            1
        }
    }
}

fn dispatch(
    args: &[String],
    stdin: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    let parsed = parse_args(args)?;
    match parsed.command.as_str() {
        "version" => {
            writeln!(stdout, "{VERSION}")?;
            stdout.flush()?;
            Ok(0)
        }
        "snapshot" => {
            let root = parsed.root.ok_or("snapshot needs --root")?;
            let doc = snapshot_document(parsed.backend, &root, parsed.sampler.as_deref())?;
            write_json(stdout, &doc)?;
            Ok(0)
        }
        "request" => {
            let request: Value = match serde_json::from_str(stdin.trim()) {
                Ok(request) => request,
                Err(err) => {
                    let err = Error::bad_request(format!("request: {err}"));
                    write_json(stdout, &error_envelope(parsed.backend, &err))?;
                    return Ok(1);
                }
            };
            let response = match handle(
                parsed.backend,
                parsed.root.as_deref(),
                &request,
                parsed.sampler.as_deref(),
            ) {
                Ok(response) => response,
                Err(err) => error_envelope(parsed.backend, &err),
            };
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
                parsed.backend,
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
                    return Err(format!("unknown display kind {kind}").into());
                }
            }
            let display = load_displays(
                parsed.backend,
                parsed.root.as_deref(),
                parsed.sampler.as_deref(),
                &kinds,
            );
            write_json(stdout, &display)?;
            Ok(0)
        }
        "apply" => Ok(apply::run(
            parsed.root.as_deref(),
            &parsed.rest,
            stdout,
            stderr,
        )),
        "" => {
            let _ = writeln!(stderr, "ratmos: missing command");
            usage(stderr);
            Ok(2)
        }
        other => {
            let _ = writeln!(stderr, "ratmos: unknown command {other}");
            usage(stderr);
            Ok(2)
        }
    }
}

/// `ratmos serve`: read requests from stdin until it closes. Not part of `run`
/// because it streams instead of reading one document.
pub fn run_serve(args: &[String]) -> i32 {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("ratmos: {err}");
            return 2;
        }
    };
    let stdin = std::io::stdin();
    serve::serve(
        serve::Config {
            backend: parsed.backend,
            root: parsed.root,
            sampler: parsed.sampler,
        },
        stdin.lock(),
        Box::new(std::io::stdout()),
    );
    0
}

struct Parsed {
    backend: Backend,
    root: Option<PathBuf>,
    sampler: Option<PathBuf>,
    command: String,
    rest: Vec<String>,
}

fn parse_args(args: &[String]) -> Result<Parsed> {
    let mut backend = Backend::Omarchy;
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
            backend = Backend::parse(args.get(index).ok_or("--backend needs a name")?)?;
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
            return Err(format!("unknown argument {arg}").into());
        }
        positionals.push(arg.clone());
        index += 1;
    }
    if let Some(dir) = &root {
        std::fs::create_dir_all(dir)?;
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

pub(crate) fn handle(
    backend: Backend,
    root: Option<&Path>,
    request: &Value,
    sampler: Option<&Path>,
) -> Result<Value> {
    let request = Request::parse(request.clone())?;
    let result = match request {
        Request::Version => Value::String(VERSION.into()),
        Request::Platform => platform(backend)?,
        Request::SettingsList => Value::Array(settings_list(backend)?),
        Request::SettingsGet { domain } => read_domain(backend, root, &domain)?,
        Request::SettingsSet { domain, value } => {
            write_domain(backend, root, &domain, &value)?;
            let place = place_for(backend, &domain)?;
            serde_json::json!({
                "domain": domain,
                "value": read_domain(backend, root, &domain)?,
                "file": place.rel,
                "encoding": place.encoding(),
            })
        }
        Request::SettingsSnapshot { group, keys } => {
            let mut doc = if backend.is_live(root) {
                live_settings_snapshot(&group, keys.as_deref())?
            } else {
                nested_snapshot(backend, root, keys.as_deref())?
            };
            // An unset value is absent, not null.
            doc.retain(|_, value| !value.is_null());
            Value::Object(doc)
        }
        Request::DisplayGet { kind } => display::load(root, backend, &kind, sampler)?,
        Request::DisplaySnapshot => load_displays(backend, root, sampler, display::KINDS),
        Request::HostChrome => host::chrome(backend, root)?,
        Request::HostThemePack(args) => host::theme_pack(backend, root, &args)?,
        Request::HostAccounts(args) => host::accounts(backend, root, &args)?,
        Request::HostStamp(args) => host::stamp(root, &args)?,
        Request::HostRead(args) => host::read_files(root, &args)?,
        Request::HostWrite(args) => host::write_file(root, &args)?,
        Request::HostOpen { path } => host::open_file(root, &path)?,
        Request::SpeedtestDisk(args) => host::speed_disk(backend, root, &args)?,
        Request::SpeedtestNet(args) => host::speed_net(backend, root, &args)?,
        Request::UnitOutput(args) => host::unit_output(backend, root, &args)?,
        Request::AgentsMcpList => agents::list(root)?,
        Request::AgentsMcpSet {
            agent,
            on,
            replace,
        } => agents::set(root, &agent, on, replace)?,
        Request::AgentsMcpCheck => agents::check(root)?,
    };
    Ok(ok_envelope(backend, result))
}

fn settings_list(backend: Backend) -> Result<Vec<Value>> {
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

fn read_domain(backend: Backend, root: Option<&Path>, key: &str) -> Result<Value> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    if backend == Backend::Omarchy {
        if let Some(effect::Effect::Command(_)) = effect::get(key) {
            return effect::read_command(root, key);
        }
    }
    let place = domain::locate(backend, spec)?;
    store::read_place(root, &place, spec.key, spec.ty)
}

fn write_domain(backend: Backend, root: Option<&Path>, key: &str, value: &Value) -> Result<()> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    if !spec.ty.accepts(value) {
        return Err(format!("{key} expects {}", spec.ty.name()).into());
    }
    if backend == Backend::Omarchy {
        if let Some(effect::Effect::Command(form)) = effect::get(key) {
            return effect::apply_command(root, key, form, value);
        }
    }
    let place = domain::locate(backend, spec)?;
    match store::write_place(root, &place, spec.key, spec.ty, value) {
        Ok(()) => {}
        Err(err) if backend.is_live(root) && err.is_denied() => {
            delegate_root_script(key, value)?;
        }
        Err(err) => return Err(err),
    }
    if backend == Backend::Omarchy {
        effects::after_write(root, key, value)?;
    }
    if backend.is_live(root) {
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

fn delegate_root_script(key: &str, value: &Value) -> Result<()> {
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
        _ => Err(format!("cannot write {key} without permission").into()),
    }
}

fn scalar_arg(value: &Value) -> Result<String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Bool(true) => Ok("true".into()),
        Value::Bool(false) => Ok("false".into()),
        Value::Number(number) => Ok(number.to_string()),
        _ => Err("live writer needs a scalar".into()),
    }
}

/// A repo script that may need the user (polkit, a password), so it has no
/// time limit.
fn run_script(name: &str, args: &[&str]) -> Result<()> {
    let path = scripts::repo_script(name)?;
    Run::new("bash")
        .arg(&path)
        .args(args)
        .checked()
        .map(|_| ())
        .map_err(|err| err.with_context(name))
}

fn run_command(program: &str, args: &[&str]) -> Result<()> {
    Run::new(program).args(args).checked().map(|_| ())
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
    // Fire and forget, but a command that cannot start is not silent.
    if let Err(err) = Run::new(program).args(args).detach() {
        eprintln!("ratmos: {err}");
    }
}

fn place_for(backend: Backend, key: &str) -> Result<domain::Place> {
    let spec = domain::find(key).ok_or_else(|| format!("unknown domain {key}"))?;
    domain::locate(backend, spec)
}

fn nested_snapshot(
    backend: Backend,
    root: Option<&Path>,
    keys: Option<&[String]>,
) -> Result<Map<String, Value>> {
    let mut doc = Map::new();
    for spec in domain::specs() {
        if !wanted(spec.key, keys) {
            continue;
        }
        let value = read_domain(backend, root, spec.key)?;
        if !value.is_null() {
            nest_domain(&mut doc, spec.key, value);
        }
    }
    Ok(doc)
}

fn settings_snapshot(backend: Backend, root: Option<&Path>) -> Result<Map<String, Value>> {
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
fn live_settings_snapshot(group: &str, keys: Option<&[String]>) -> Result<Map<String, Value>> {
    let mut snapshot = Map::new();
    if let Ok(text) = capture_snapshot_sh(group) {
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&text) {
            snapshot = map;
        }
    }
    let group_name = if group.is_empty() { "all" } else { group };
    snapshot.insert("group".into(), Value::String(group_name.into()));
    for spec in domain::specs() {
        if !wanted(spec.key, keys) {
            continue;
        }
        // One unreadable system file must not drop the rest of the page.
        if let Ok(value) = read_domain(Backend::Omarchy, None, spec.key) {
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
    if matches!(
        key,
        "nightlight"
            | "audioOutputMuted"
            | "audioInputMuted"
            | "barVisible"
            | "screensaverEnabled"
            | "stayAwake"
            | "touchpadEnabled"
            | "touchscreenEnabled"
            | "doNotDisturb"
    ) {
        return;
    }
    nest_domain(doc, key, value);
}

/// A dotted domain such as `hyprLook.gaps` lands inside the object `hyprLook`.
fn nest_domain(doc: &mut Map<String, Value>, key: &str, value: Value) {
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

/// Whether a domain is in the requested key list. The list names top-level
/// keys, so `hyprLook.gaps` is wanted when `hyprLook` is.
fn wanted(key: &str, keys: Option<&[String]>) -> bool {
    let Some(keys) = keys else { return true };
    let head = key.split_once('.').map_or(key, |(head, _)| head);
    keys.iter().any(|name| name == head)
}

fn capture_snapshot_sh(group: &str) -> Result<String> {
    let script = scripts::repo_script("snapshot.sh")?;
    let output = Run::new("bash")
        .arg(&script)
        .arg(group)
        .timeout(std::time::Duration::from_secs(60))
        .checked()?;
    Ok(output.stdout_text())
}

/// One failing inventory stays on its own kind. The other documents still return.
fn load_displays(
    backend: Backend,
    root: Option<&Path>,
    sampler: Option<&Path>,
    kinds: &[&str],
) -> Value {
    // The inventories are independent scripts, so run them side by side.
    let loaded: Vec<(String, Value)> = std::thread::scope(|scope| {
        let handles: Vec<_> = kinds
            .iter()
            .map(|kind| {
                scope.spawn(move || {
                    let value = match display::load(root, backend, kind, sampler) {
                        Ok(value) => value,
                        Err(err) => stamp_error(backend, kind, &err),
                    };
                    ((*kind).to_string(), value)
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|handle| handle.join().ok())
            .collect()
    });
    let mut display = Map::new();
    for (kind, value) in loaded {
        display.insert(kind, value);
    }
    Value::Object(display)
}

fn snapshot_document(backend: Backend, root: &Path, sampler: Option<&Path>) -> Result<Value> {
    Ok(serde_json::json!({
        "version": VERSION,
        "platform": platform(backend)?,
        "settings": Value::Object(settings_snapshot(backend, Some(root))?),
        "display": load_displays(backend, Some(root), sampler, display::KINDS),
    }))
}

fn platform(backend: Backend) -> Result<Value> {
    Ok(backend.info())
}

pub(crate) fn ok_envelope(backend: Backend, result: Value) -> Value {
    serde_json::json!({
        "ok": true,
        "version": VERSION,
        "platform": platform(backend).unwrap_or(Value::Null),
        "result": result,
    })
}

pub(crate) fn error_envelope(backend: Backend, err: &Error) -> Value {
    serde_json::json!({
        "ok": false,
        "version": VERSION,
        "platform": platform(backend).unwrap_or(Value::Null),
        "error": {
            "code": err.kind.code(),
            "message": err.message,
            "context": err.context,
        },
        "result": Value::Null,
    })
}

fn stamp_error(backend: Backend, kind: &str, err: &Error) -> Value {
    serde_json::json!({
        "platform": backend.id(),
        "collector": display::collector(backend, kind),
        "error": {
            "code": err.kind.code(),
            "message": err.message,
            "context": err.context,
        },
    })
}

fn write_json(stdout: &mut dyn Write, value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    writeln!(stdout, "{text}")?;
    stdout.flush().map_err(Error::from)
}

fn usage(stderr: &mut dyn Write) {
    let _ = writeln!(
        stderr,
        "usage: ratmos [--backend omarchy|plain] [--root DIR] <snapshot|request|display KIND|display-snapshot|apply -- CMD|version>"
    );
}
