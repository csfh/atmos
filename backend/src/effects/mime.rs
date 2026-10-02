use super::*;

pub(super) fn mime_types(key: &str) -> &'static [&'static str] {
    match key {
        "mimePdf" => &["application/pdf"],
        "mimeImage" => &["image/png", "image/jpeg", "image/webp", "image/gif"],
        "mimeVideo" => &["video/mp4", "video/webm", "video/x-matroska"],
        _ => &[],
    }
}

pub(super) fn mime_map(text: &str) -> Map<String, Value> {
    let mut inside = false;
    let mut map = Map::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[Default Applications]" {
            inside = true;
            continue;
        }
        if trimmed.starts_with('[') {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some((mime, desktop)) = trimmed.split_once('=') {
            map.insert(mime.to_string(), Value::String(desktop.to_string()));
        }
    }
    map
}

pub(super) fn mime_desktop(text: &str, mime: &str) -> Value {
    mime_map(text)
        .get(mime)
        .cloned()
        .unwrap_or(Value::String(String::new()))
}

pub(super) fn write_mime(path: &Path, key: &str, value: &Value) -> Result<()> {
    let desktop = value
        .as_str()
        .ok_or_else(|| format!("{key} expects a string"))?;
    let existing = if path.is_file() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed == "[Default Applications]" {
            skipping = true;
            continue;
        }
        if skipping && trimmed.starts_with('[') {
            skipping = false;
        }
        if !skipping {
            kept.push(line.to_string());
        }
    }
    let mut map = mime_map(&existing);
    for mime in mime_types(key) {
        map.insert((*mime).to_string(), Value::String(desktop.to_string()));
    }
    if !kept.is_empty() && !kept.last().is_some_and(|line| line.is_empty()) {
        kept.push(String::new());
    }
    kept.push("[Default Applications]".to_string());
    let mut names: Vec<_> = map.keys().cloned().collect();
    names.sort();
    for name in names {
        if let Some(desktop) = map.get(&name).and_then(Value::as_str) {
            kept.push(format!("{name}={desktop}"));
        }
    }
    atomic_text(path, &format!("{}\n", kept.join("\n")))
}

pub(super) fn mime_script(kind: &str, value: &Value) -> Result<()> {
    let desktop = value.as_str().unwrap_or("");
    if desktop.is_empty() {
        return Ok(());
    }
    let _ = run_bash(false, "set-mime-default.sh", &[kind, desktop]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_map_reads_only_default_applications() {
        let text = "[Added Associations]\nimage/png=other.desktop\n[Default Applications]\nimage/png=imv.desktop\ntext/html=firefox.desktop\n[Removed Associations]\nx/y=z.desktop\n";
        let map = mime_map(text);
        assert_eq!(map.len(), 2);
        assert_eq!(map["image/png"], "imv.desktop");
        assert_eq!(map["text/html"], "firefox.desktop");
    }

    #[test]
    fn an_empty_file_has_no_defaults() {
        assert!(mime_map("").is_empty());
    }
}
