use super::*;

pub(super) fn env_state(text: &str) -> (String, Vec<(String, String)>) {
    let mut inside = false;
    let mut prepend = String::new();
    let mut vars = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "# atmos:env begin" {
            inside = true;
            continue;
        }
        if trimmed == "# atmos:env end" {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("PATH=") {
            if let Some(path) = rest.strip_suffix(":$PATH") {
                prepend = path.to_string();
            }
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            if !key.is_empty() && key != "PATH" {
                vars.push((key.to_string(), value.to_string()));
            }
        }
    }
    (prepend, vars)
}

pub(super) fn write_env(path: &Path, key: &str, value: &Value) -> Result<()> {
    let existing = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let (mut prepend, mut vars) = env_state(&existing);
    match key {
        "envPathPrepend" => {
            prepend = value
                .as_str()
                .ok_or_else(|| "envPathPrepend expects a string".to_string())?
                .to_string();
        }
        "envVars" => {
            let items = value
                .as_array()
                .ok_or_else(|| "envVars expects a list".to_string())?;
            vars.clear();
            for item in items {
                let Some(obj) = item.as_object() else {
                    continue;
                };
                let Some(name) = obj.get("key").and_then(Value::as_str) else {
                    continue;
                };
                if name.is_empty() || name == "PATH" {
                    continue;
                }
                let stored = obj.get("value").and_then(Value::as_str).unwrap_or("");
                vars.push((name.to_string(), stored.to_string()));
            }
        }
        _ => return Err(format!("not an env field {key}").into()),
    }
    let mut lines = vec!["# atmos:env begin".to_string()];
    if !prepend.is_empty() {
        lines.push(format!("PATH={prepend}:$PATH"));
    }
    for (name, stored) in vars {
        lines.push(format!("{name}={stored}"));
    }
    lines.push("# atmos:env end".to_string());
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ratmos-env-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join("10-atmos.conf")
    }

    #[test]
    fn env_state_reads_only_the_managed_block() {
        let text = "OUTSIDE=1\n# atmos:env begin\nPATH=/opt/bin:$PATH\nEDITOR=nvim\n# atmos:env end\nAFTER=2\n";
        let (prepend, vars) = env_state(text);
        assert_eq!(prepend, "/opt/bin");
        assert_eq!(vars, vec![("EDITOR".to_string(), "nvim".to_string())]);
    }

    #[test]
    fn writing_vars_keeps_the_path_prepend_and_the_other_way_round() {
        let path = scratch("both");
        write_env(&path, "envPathPrepend", &json!("/opt/bin")).unwrap();
        write_env(
            &path,
            "envVars",
            &json!([{"key": "EDITOR", "value": "nvim"}]),
        )
        .unwrap();
        let (prepend, vars) = env_state(&fs::read_to_string(&path).unwrap());
        assert_eq!(prepend, "/opt/bin");
        assert_eq!(vars, vec![("EDITOR".to_string(), "nvim".to_string())]);
        write_env(&path, "envPathPrepend", &json!("")).unwrap();
        let (prepend, vars) = env_state(&fs::read_to_string(&path).unwrap());
        assert_eq!(prepend, "");
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn a_path_entry_in_the_list_and_a_bad_field_are_refused() {
        let path = scratch("bad");
        write_env(&path, "envVars", &json!([{"key": "PATH", "value": "x"}])).unwrap();
        assert!(env_state(&fs::read_to_string(&path).unwrap()).1.is_empty());
        assert!(write_env(&path, "nope", &json!("x")).is_err());
        assert!(write_env(&path, "envVars", &json!("not a list")).is_err());
    }
}
