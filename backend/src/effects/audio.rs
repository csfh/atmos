use super::*;

pub(super) fn audio_prefix(key: &str) -> &'static str {
    match key {
        "audioOutputVolume" => "output-volume ",
        "audioInputVolume" => "input-volume ",
        "audioTuningOn" => "tuning ",
        _ => "",
    }
}

pub(super) fn audio_number(text: &str, prefix: &str) -> Result<Value> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix(prefix) {
            let number: i64 = rest
                .parse()
                .map_err(|err: std::num::ParseIntError| err.to_string())?;
            return Ok(Value::from(number));
        }
    }
    Ok(Value::Null)
}

pub(super) fn audio_bool(text: &str, prefix: &str) -> Result<Value> {
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix(prefix) {
            return Ok(Value::Bool(rest == "true"));
        }
    }
    Ok(Value::Null)
}

pub(super) fn write_audio_line(path: &Path, key: &str, value: &Value) -> Result<()> {
    let prefix = audio_prefix(key);
    let line = match key {
        "audioOutputVolume" | "audioInputVolume" => {
            let number = value
                .as_i64()
                .ok_or_else(|| format!("{key} expects an int"))?;
            format!("{prefix}{number}")
        }
        _ => {
            let flag = value
                .as_bool()
                .ok_or_else(|| format!("{key} expects a bool"))?;
            format!("{prefix}{flag}")
        }
    };
    let existing = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut found = false;
    let mut lines = Vec::new();
    for current in existing.lines() {
        if current.trim().starts_with(prefix) {
            lines.push(line.clone());
            found = true;
        } else if !current.trim().is_empty() {
            lines.push(current.to_string());
        }
    }
    if !found {
        lines.push(line);
    }
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

pub(super) fn audio_script(fixture: bool, action: &str, value: &Value) -> Result<()> {
    let number = value
        .as_i64()
        .ok_or_else(|| format!("{action} expects an int"))?
        .to_string();
    run_bash(fixture, "set-audio.sh", &[action, &number])
}
