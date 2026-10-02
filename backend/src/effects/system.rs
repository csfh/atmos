use super::*;

pub(super) fn write_snapper(path: &Path, key: &str, value: &Value) -> Result<()> {
    let existing = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut number = snapper_number(&existing);
    let mut timeline = snapper_timeline(&existing);
    match key {
        "snapperNumberLimit" => {
            number = value
                .as_i64()
                .ok_or_else(|| "snapperNumberLimit expects an int".to_string())?;
        }
        "snapperTimeline" => {
            timeline = value
                .as_bool()
                .ok_or_else(|| "snapperTimeline expects a bool".to_string())?;
        }
        _ => return Err(format!("not a snapper field {key}").into()),
    }
    let want = if timeline { "yes" } else { "no" };
    let mut lines = Vec::new();
    let mut saw_number = false;
    let mut saw_timeline = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("NUMBER_LIMIT=") {
            lines.push(format!("NUMBER_LIMIT=\"{number}\""));
            saw_number = true;
        } else if trimmed.starts_with("TIMELINE_CREATE=") {
            lines.push(format!("TIMELINE_CREATE=\"{want}\""));
            saw_timeline = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !saw_number {
        lines.push(format!("NUMBER_LIMIT=\"{number}\""));
    }
    if !saw_timeline {
        lines.push(format!("TIMELINE_CREATE=\"{want}\""));
    }
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

pub(super) fn snapper_number(text: &str) -> i64 {
    for line in text.lines() {
        let trimmed = line.trim().trim_start_matches("NUMBER_LIMIT=").trim();
        if line.trim().starts_with("NUMBER_LIMIT=") {
            let bare = trimmed.trim_matches('"');
            if let Ok(number) = bare.parse() {
                return number;
            }
        }
    }
    5
}

pub(super) fn snapper_timeline(text: &str) -> bool {
    for line in text.lines() {
        if line.trim().starts_with("TIMELINE_CREATE=") {
            return line.contains("yes");
        }
    }
    false
}

pub(super) fn write_pam(path: &Path, key: &str, value: &Value) -> Result<()> {
    let on = value
        .as_bool()
        .ok_or_else(|| format!("{key} expects a bool"))?;
    let needle = if key == "fingerprintConfigured" {
        "pam_fprintd.so"
    } else {
        "pam_u2f.so"
    };
    let existing = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();
    let present = lines.iter().any(|line| line.contains(needle));
    if on && !present {
        lines.push(format!("auth sufficient {needle}"));
    }
    if !on {
        lines.retain(|line| !line.contains(needle));
    }
    let body = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    atomic_text(path, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapper_number_reads_quoted_and_bare_values_and_defaults_to_five() {
        assert_eq!(snapper_number("NUMBER_LIMIT=\"12\"\n"), 12);
        assert_eq!(snapper_number("NUMBER_LIMIT=7\n"), 7);
        assert_eq!(snapper_number("OTHER=1\n"), 5);
        assert_eq!(snapper_number("NUMBER_LIMIT=\"many\"\n"), 5);
    }

    #[test]
    fn snapper_timeline_follows_the_yes() {
        assert!(snapper_timeline("TIMELINE_CREATE=\"yes\"\n"));
        assert!(!snapper_timeline("TIMELINE_CREATE=\"no\"\n"));
        assert!(!snapper_timeline(""));
    }
}
