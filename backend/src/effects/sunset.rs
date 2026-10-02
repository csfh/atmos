use super::*;

pub(super) struct Sunset {
    pub(super) day: String,
    pub(super) night: String,
    pub(super) temp: i64,
    pub(super) night_on: bool,
    pub(super) saw_day: bool,
    pub(super) saw_night: bool,
    pub(super) saw_temp: bool,
}

pub(super) fn sunset_state(text: &str) -> Sunset {
    let mut day = "07:00".to_string();
    let mut night = "20:00".to_string();
    let mut temp = 4000_i64;
    let mut saw_day = false;
    let mut saw_night = false;
    let mut saw_temp = false;
    let mut profiles: Vec<Vec<String>> = Vec::new();
    let mut current: Option<Vec<String>> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# atmos:night = ") {
            night = rest.to_string();
            saw_night = true;
        }
        if let Some(rest) = trimmed.strip_prefix("# atmos:temperature = ") {
            if let Ok(number) = rest.parse() {
                temp = number;
                saw_temp = true;
            }
        }
        if trimmed.starts_with("profile") {
            current = Some(Vec::new());
            continue;
        }
        if trimmed == "}" {
            if let Some(body) = current.take() {
                profiles.push(body);
            }
            continue;
        }
        if let Some(body) = current.as_mut() {
            body.push(trimmed.to_string());
        }
    }
    if let Some(first) = profiles.first() {
        for line in first {
            if let Some(rest) = line.strip_prefix("time = ") {
                day = rest.to_string();
                saw_day = true;
            }
        }
    }
    let night_on = profiles.len() > 1;
    if night_on {
        for line in &profiles[1] {
            if let Some(rest) = line.strip_prefix("time = ") {
                night = rest.to_string();
                saw_night = true;
            }
            if let Some(rest) = line.strip_prefix("temperature = ") {
                if let Ok(number) = rest.parse() {
                    temp = number;
                    saw_temp = true;
                }
            }
        }
    }
    Sunset {
        day,
        night,
        temp,
        night_on,
        saw_day,
        saw_night,
        saw_temp,
    }
}

pub(super) fn write_sunset(path: &Path, key: &str, value: &Value) -> Result<()> {
    let existing = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut state = sunset_state(&existing);
    match key {
        "nightlightNightOn" => {
            state.night_on = value
                .as_bool()
                .ok_or_else(|| "nightlightNightOn expects a bool".to_string())?;
        }
        "nightlightTemperature" => {
            state.temp = value
                .as_i64()
                .ok_or_else(|| "nightlightTemperature expects an int".to_string())?;
            state.saw_temp = true;
        }
        "nightlightDay" => {
            state.day = clock(
                value
                    .as_str()
                    .ok_or_else(|| "nightlightDay expects a time".to_string())?,
            )?;
            state.saw_day = true;
        }
        "nightlightNight" => {
            state.night = clock(
                value
                    .as_str()
                    .ok_or_else(|| "nightlightNight expects a time".to_string())?,
            )?;
            state.saw_night = true;
        }
        _ => return Err(format!("not a sunset field {key}").into()),
    }
    // Profiles only. The on/off switch is `omarchy toggle nightlight`.
    let mut lines = vec!["# Written by atmos. Day leaves the screen untinted.".to_string()];
    if state.saw_night {
        lines.push(format!("# atmos:night = {}", state.night));
    }
    if state.saw_temp {
        lines.push(format!("# atmos:temperature = {}", state.temp));
    }
    lines.extend([
        "profile {".to_string(),
        format!("    time = {}", state.day),
        "    identity = true".to_string(),
        "}".to_string(),
    ]);
    if state.night_on {
        lines.push(String::new());
        lines.push("profile {".to_string());
        lines.push(format!("    time = {}", state.night));
        lines.push(format!("    temperature = {}", state.temp));
        lines.push("}".to_string());
    }
    atomic_text(path, &format!("{}\n", lines.join("\n")))
}

pub(super) fn clock(raw: &str) -> Result<String> {
    let Some((hour, minute)) = raw.trim().split_once(':') else {
        return Err(format!("{raw} is not HH:MM").into());
    };
    let hour: u32 = hour.parse().map_err(|_| format!("{raw} is not HH:MM"))?;
    let minute: u32 = minute.parse().map_err(|_| format!("{raw} is not HH:MM"))?;
    if hour > 23 || minute > 59 {
        return Err(format!("{raw} is not HH:MM").into());
    }
    Ok(format!("{hour:02}:{minute:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_pads_and_rejects() {
        assert_eq!(clock("7:5").unwrap(), "07:05");
        assert_eq!(clock(" 23:59 ").unwrap(), "23:59");
        assert!(clock("24:00").is_err());
        assert!(clock("12:60").is_err());
        assert!(clock("noon").is_err());
    }

    #[test]
    fn an_empty_file_reads_as_the_defaults() {
        let state = sunset_state("");
        assert_eq!(state.day, "07:00");
        assert_eq!(state.night, "20:00");
        assert_eq!(state.temp, 4000);
        assert!(!state.saw_day && !state.saw_night && !state.saw_temp);
    }

    #[test]
    fn marker_comments_set_the_night_and_temperature() {
        let state = sunset_state("# atmos:night = 21:30\n# atmos:temperature = 3500\n");
        assert_eq!(state.night, "21:30");
        assert!(state.saw_night);
        assert_eq!(state.temp, 3500);
        assert!(state.saw_temp);
    }
}
