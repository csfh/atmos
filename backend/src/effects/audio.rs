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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn each_audio_key_has_its_own_prefix() {
        assert_eq!(audio_prefix("audioOutputVolume"), "output-volume ");
        assert_eq!(audio_prefix("audioInputVolume"), "input-volume ");
        assert_eq!(audio_prefix("audioTuningOn"), "tuning ");
        assert_eq!(audio_prefix("theme"), "");
        assert_eq!(audio_prefix(""), "");
    }

    #[test]
    fn a_number_is_read_from_its_prefixed_line() {
        let text = "tuning true\noutput-volume 40\n  input-volume 75  \n";
        assert_eq!(audio_number(text, "output-volume ").unwrap(), json!(40));
        assert_eq!(audio_number(text, "input-volume ").unwrap(), json!(75));
    }

    #[test]
    fn a_missing_number_line_is_null() {
        assert_eq!(audio_number("", "output-volume ").unwrap(), Value::Null);
        assert_eq!(
            audio_number("tuning true\n", "output-volume ").unwrap(),
            Value::Null
        );
    }

    #[test]
    fn a_malformed_number_is_an_error_not_zero() {
        assert!(audio_number("output-volume loud\n", "output-volume ").is_err());
        // Trimming drops the trailing space, so a bare key is not a match.
        assert_eq!(
            audio_number("output-volume \n", "output-volume ").unwrap(),
            Value::Null
        );
        assert!(audio_number("output-volume 4.5\n", "output-volume ").is_err());
    }

    #[test]
    fn negative_numbers_round_trip() {
        assert_eq!(
            audio_number("output-volume -3\n", "output-volume ").unwrap(),
            json!(-3)
        );
    }

    #[test]
    fn a_bool_is_true_only_for_the_word_true() {
        assert_eq!(audio_bool("tuning true\n", "tuning ").unwrap(), json!(true));
        assert_eq!(
            audio_bool("tuning false\n", "tuning ").unwrap(),
            json!(false)
        );
        assert_eq!(audio_bool("tuning yes\n", "tuning ").unwrap(), json!(false));
        assert_eq!(
            audio_bool("tuning True\n", "tuning ").unwrap(),
            json!(false)
        );
        assert_eq!(audio_bool("other 1\n", "tuning ").unwrap(), Value::Null);
        assert_eq!(audio_bool("", "tuning ").unwrap(), Value::Null);
    }

    #[test]
    fn the_first_matching_line_wins() {
        let text = "output-volume 10\noutput-volume 90\n";
        assert_eq!(audio_number(text, "output-volume ").unwrap(), json!(10));
    }
}
