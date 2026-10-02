use super::*;

pub fn read_hypr(path: &Path, kind: &str, key: &str) -> Result<Value> {
    if !path.is_file() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path)?;
    match key {
        "workspaceWrapSwitch" => Ok(flag_comment(&text, "wrapSwitch")),
        "workspaceWheelSwitch" => Ok(flag_comment(&text, "wheelSwitch")),
        _ => {
            let begin = sentinel_begin(kind);
            if !text.contains(begin) && !text_has_calls(&text, kind) {
                return Ok(Value::Null);
            }
            hypr_list(kind, path)
        }
    }
}

pub fn write_hypr(path: &Path, kind: &str, key: &str, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let payload = match key {
        "bindings" | "windowRules" | "autostart" | "monitorRules" => {
            serde_json::json!({ "items": keep_managed(value) })
        }
        "workspaces" => {
            let wrap = flag_comment(&text, "wrapSwitch").as_bool().unwrap_or(true);
            let wheel = flag_comment(&text, "wheelSwitch").as_bool().unwrap_or(true);
            serde_json::json!({ "items": keep_managed(value), "wrapSwitch": wrap, "wheelSwitch": wheel })
        }
        "workspaceWrapSwitch" | "workspaceWheelSwitch" => {
            let mut items = if text.contains("-- atmos:workspaces begin") {
                keep_managed(&hypr_list("workspaces", path)?)
            } else {
                Value::Array(Vec::new())
            };
            if !items.is_array() {
                items = Value::Array(Vec::new());
            }
            let wrap = if key == "workspaceWrapSwitch" {
                value.as_bool().unwrap_or(true)
            } else {
                flag_comment(&text, "wrapSwitch").as_bool().unwrap_or(true)
            };
            let wheel = if key == "workspaceWheelSwitch" {
                value.as_bool().unwrap_or(true)
            } else {
                flag_comment(&text, "wheelSwitch").as_bool().unwrap_or(true)
            };
            serde_json::json!({ "items": items, "wrapSwitch": wrap, "wheelSwitch": wheel })
        }
        other => return Err(format!("no hypr block for {other}").into()),
    };
    hypr_apply(kind, path, &payload)
}

pub(super) fn flag_comment(text: &str, name: &str) -> Value {
    let prefix = format!("-- atmos:{name} = ");
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix(&prefix) {
            return Value::Bool(rest == "true");
        }
    }
    Value::Null
}

pub(super) fn sentinel_begin(kind: &str) -> &'static str {
    match kind {
        "bindings" => "-- atmos:bindings begin",
        "windows" => "-- atmos:windows begin",
        "workspaces" => "-- atmos:workspaces begin",
        "autostart" => "-- atmos:autostart begin",
        "monitors" => "-- atmos:monitors begin",
        _ => "",
    }
}

pub(super) fn text_has_calls(text: &str, kind: &str) -> bool {
    match kind {
        "bindings" => text.contains("o.bind(") || text.contains("hl.unbind("),
        "windows" => text.contains("o.window("),
        "autostart" => text.contains("o.launch_on_start("),
        "workspaces" => text.contains("hl.workspace_rule("),
        "monitors" => text.contains("hl.monitor("),
        _ => false,
    }
}

pub(super) fn keep_managed(value: &Value) -> Value {
    let Some(items) = value.as_array() else {
        return value.clone();
    };
    Value::Array(
        items
            .iter()
            .filter(|item| item.get("managed").and_then(Value::as_bool) != Some(false))
            .cloned()
            .collect(),
    )
}

pub(super) fn hypr_apply(kind: &str, path: &Path, payload: &Value) -> Result<()> {
    let script = crate::scripts::repo_script("hypr-sentinel.py")?;
    let json = serde_json::to_string(payload)?;
    Run::new("python3")
        .arg(&script)
        .arg(kind)
        .arg("apply")
        .arg(path)
        // On stdin, not argv: a long bindings list would hit the argument limit.
        .input(json)
        .timeout(COMMAND_TIMEOUT)
        .checked()
        .map(|_| ())
        .map_err(|err| err.with_context(format!("hypr-sentinel.py {kind}")))
}

pub(super) fn hypr_list(kind: &str, path: &Path) -> Result<Value> {
    let script = crate::scripts::repo_script("hypr-sentinel.py")?;
    let output = Run::new("python3")
        .arg(&script)
        .arg(kind)
        .arg("list")
        .arg(path)
        .timeout(COMMAND_TIMEOUT)
        .checked()
        .map_err(|err| err.with_context(format!("hypr-sentinel.py {kind} list")))?;
    serde_json::from_slice(&output.stdout).map_err(Error::from)
}
