use super::*;

pub fn read_hypr(path: &Path, kind: &str, key: &str) -> Result<Value> {
    if !path.is_file() {
        return Ok(Value::Null);
    }
    let text = fs::read_to_string(path)?;
    match key {
        "workspaceWrapSwitch" => Ok(flag_comment(&text, "wrapSwitch")),
        "workspaceWheelSwitch" => Ok(flag_comment(&text, "wheelSwitch")),
        "windowOpaque" => Ok(Value::Bool(
            flag_comment(&text, "windowOpaque") == Value::Bool(true),
        )),
        _ => {
            let Some(parsed) = crate::hyprland::Kind::parse(kind) else {
                return hypr_list(kind, path);
            };
            if !text.contains(parsed.begin()) && !parsed.has_calls(&text) {
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
            let mut items = if text.contains(crate::hyprland::Kind::Workspaces.begin()) {
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
        // The writer keeps the rules and merges the flag into the block.
        "windowOpaque" => serde_json::json!({ "windowOpaque": value }),
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
    let kind =
        crate::hyprland::Kind::parse(kind).ok_or_else(|| format!("no hypr block for {kind}"))?;
    crate::hyprland::apply_file(kind, path, payload, false)
}

pub(super) fn hypr_list(kind: &str, path: &Path) -> Result<Value> {
    let kind =
        crate::hyprland::Kind::parse(kind).ok_or_else(|| format!("no hypr block for {kind}"))?;
    crate::hyprland::list_file(kind, path)
}
