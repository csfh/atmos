//! Settings search. The catalog is the hub list the UI sends plus the rows
//! found by reading the page sources (labels, descriptions, hints, keywords);
//! a query is a case-insensitive substring match over each row's text.

use regex_lite::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct Hub {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub keywords: Vec<String>,
}

/// What the UI knows about its own pages: the hubs, which page file belongs to
/// which hub, the directory prefixes that decide the rest, and the titles.
#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct Context {
    #[serde(default)]
    pub hubs: Vec<Hub>,
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub prefixes: Vec<(String, String)>,
    #[serde(default)]
    pub titles: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct Row {
    id: String,
    hub: String,
    hub_title: String,
    label: String,
    description: String,
    hint: String,
    detail: String,
    keywords: Vec<String>,
    haystack: String,
}

fn haystack(parts: &[&str], keywords: &[String]) -> String {
    let mut text = String::new();
    for part in parts
        .iter()
        .copied()
        .chain(keywords.iter().map(String::as_str))
    {
        text.push(' ');
        text.push_str(part);
    }
    text.to_lowercase()
}

fn title_of(ctx: &Context, hub: &str) -> String {
    if let Some(t) = ctx.titles.get(hub) {
        return t.clone();
    }
    let root = hub.split('/').next().unwrap_or(hub);
    ctx.titles
        .get(root)
        .cloned()
        .unwrap_or_else(|| hub.to_string())
}

fn file_hub(ctx: &Context, rel: &str) -> Option<String> {
    let base = rel.replace('\\', "/");
    if let Some(hub) = ctx.files.get(&base) {
        return Some(hub.clone());
    }
    ctx.prefixes
        .iter()
        .find(|(prefix, _)| base.starts_with(prefix.as_str()))
        .map(|(_, hub)| hub.clone())
}

fn string_prop(block: &str, name: &str) -> String {
    let Ok(re) = Regex::new(&format!(
        r#"{}:\s*"((?:\\.|[^"\\])*)""#,
        regex_lite::escape(name)
    )) else {
        return String::new();
    };
    re.captures(block)
        .map(|m| unquote(&m[1]))
        .unwrap_or_default()
}

/// A JSON string body back to text, falling back to the raw text.
fn unquote(raw: &str) -> String {
    serde_json::from_str::<String>(&format!("\"{raw}\"")).unwrap_or_else(|_| raw.to_string())
}

fn keywords_prop(block: &str) -> Vec<String> {
    let Some(list) = Regex::new(r"keywords:\s*\[([^\]]*)\]")
        .ok()
        .and_then(|re| re.captures(block).map(|m| m[1].to_string()))
    else {
        return Vec::new();
    };
    Regex::new(r#""((?:\\.|[^"\\])*)""#)
        .map(|re| re.captures_iter(&list).map(|m| unquote(&m[1])).collect())
        .unwrap_or_default()
}

/// The `{ ... }` bodies of every `Name { ... }` in a QML source.
fn named_blocks<'a>(src: &'a str, name: &str) -> Vec<&'a str> {
    let needle = format!("{name} {{");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = src[from..].find(&needle) {
        let start = from + rel;
        let Some(brace) = src[start..].find('{').map(|b| start + b) else {
            break;
        };
        let (mut depth, mut end) = (0i32, brace);
        for (i, ch) in src[brace..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = brace + i + 1;
                        break;
                    }
                }
                _ => {}
            }
            end = brace + i + ch.len_utf8();
        }
        out.push(&src[brace..end]);
        from = start + needle.len();
    }
    out
}

fn slug(text: &str) -> String {
    let lowered = text.to_lowercase();
    let replaced = Regex::new(r"[^a-z0-9]+")
        .map(|re| re.replace_all(&lowered, "-").into_owned())
        .unwrap_or(lowered);
    let trimmed = replaced.trim_matches('-');
    if trimmed.is_empty() {
        "row".into()
    } else {
        trimmed.to_string()
    }
}

fn rows_from_qml(src: &str, hub: &str, title: &str) -> Vec<Row> {
    let mut out = Vec::new();
    let no_catalog = Regex::new(r"\bcatalog\s*:\s*false\b").expect("regex");
    for kind in [
        "CollectionRow",
        "SettingRow",
        "PrefsRow",
        "PrefsLink",
        "PrefsGroup",
    ] {
        for (i, block) in named_blocks(src, kind).into_iter().enumerate() {
            if no_catalog.is_match(block) {
                continue;
            }
            let label = string_prop(
                block,
                if kind == "PrefsGroup" {
                    "title"
                } else {
                    "label"
                },
            );
            if label.is_empty() {
                continue;
            }
            let (description, hint, detail) = (
                string_prop(block, "description"),
                string_prop(block, "hint"),
                string_prop(block, "detail"),
            );
            let keywords = keywords_prop(block);
            out.push(Row {
                id: format!("{hub}/{}-{kind}-{i}", slug(&label)),
                hub: hub.into(),
                hub_title: title.into(),
                haystack: haystack(&[&label, &description, &hint, &detail], &keywords),
                label,
                description,
                hint,
                detail,
                keywords,
            });
        }
    }
    out
}

fn walk_qml(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        let path = entry.path();
        if path.is_dir() {
            walk_qml(&path, &rel, out);
        } else if name.ends_with(".qml") {
            out.push((rel, path));
        }
    }
}

fn shared_rows(src: &str, hub: &str, title: &str, pages: &Path) -> Vec<Row> {
    let mut out = Vec::new();
    let re = Regex::new(r"\b([A-Z][A-Za-z0-9]*Row)\s*\{").expect("regex");
    for m in re.captures_iter(src) {
        let name = &m[1];
        if matches!(name, "PrefsRow" | "SettingRow" | "CollectionRow") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(pages.join("rows").join(format!("{name}.qml"))) {
            out.extend(rows_from_qml(&text, hub, title));
        }
    }
    out
}

fn build(ctx: &Context, root: &Path) -> Vec<Row> {
    let mut rows: Vec<Row> = ctx
        .hubs
        .iter()
        .map(|hub| Row {
            id: hub.id.clone(),
            hub: hub.id.clone(),
            hub_title: hub.title.clone(),
            label: hub.title.clone(),
            description: hub.description.clone(),
            hint: String::new(),
            detail: String::new(),
            keywords: hub.keywords.clone(),
            haystack: haystack(&[&hub.title, &hub.description, "", ""], &hub.keywords),
        })
        .collect();
    let pages = root.join("pages");
    let mut files = Vec::new();
    walk_qml(&pages, "", &mut files);
    for (rel, path) in files {
        let Some(hub) = file_hub(ctx, &rel) else {
            continue;
        };
        let title = title_of(ctx, &hub);
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        rows.extend(rows_from_qml(&src, &hub, &title));
        rows.extend(shared_rows(&src, &hub, &title, &pages));
    }
    // One row per id: the last one wins, like an upsert.
    let mut by_id: BTreeMap<String, Row> = BTreeMap::new();
    for row in rows {
        by_id.insert(row.id.clone(), row);
    }
    by_id.into_values().collect()
}

type Cache = Option<(String, Vec<Row>)>;
static CACHE: Mutex<Cache> = Mutex::new(None);

fn catalog(ctx: &Context, root: &Path) -> Vec<Row> {
    let key = format!(
        "{}\u{0}{}",
        root.display(),
        serde_json::to_string(&json!({"h": ctx.hubs.len(), "f": ctx.files, "p": ctx.prefixes}))
            .unwrap_or_default()
    );
    let mut guard = CACHE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((cached, rows)) = guard.as_ref() {
        if *cached == key {
            return rows.clone();
        }
    }
    let rows = build(ctx, root);
    *guard = Some((key, rows.clone()));
    rows
}

fn row_json(row: &Row) -> Value {
    json!({
        "id": row.id,
        "hub": row.hub,
        "hubTitle": row.hub_title,
        "label": row.label,
        "description": row.description,
        "hint": row.hint,
        "detail": row.detail,
        "keywords": row.keywords,
    })
}

/// Matching rows ordered by hub, then label (byte order), as SQLite did.
pub fn query(ctx: &Context, root: &Path, query: &str) -> Value {
    let needle = query.to_lowercase();
    let needle = needle.trim();
    let mut hits: Vec<Row> = catalog(ctx, root)
        .into_iter()
        .filter(|r| needle.is_empty() || r.haystack.contains(needle))
        .collect();
    hits.sort_by(|a, b| a.hub.cmp(&b.hub).then_with(|| a.label.cmp(&b.label)));
    json!({"query": query, "hits": hits.iter().map(row_json).collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context {
            hubs: vec![Hub {
                id: "windows".into(),
                title: "Windows".into(),
                description: "Gaps and borders".into(),
                keywords: vec!["tiling".into()],
            }],
            files: BTreeMap::from([("WindowsPage.qml".to_string(), "windows".to_string())]),
            prefixes: vec![("windows/".to_string(), "windows".to_string())],
            titles: BTreeMap::from([("windows".to_string(), "Windows".to_string())]),
        }
    }

    #[test]
    fn page_sources_become_searchable_rows() {
        let dir = std::env::temp_dir().join(format!("ratmos-search-{}", std::process::id()));
        let pages = dir.join("pages");
        std::fs::create_dir_all(pages.join("windows")).unwrap();
        std::fs::write(
            pages.join("WindowsPage.qml"),
            "SettingRow {\n  label: \"Dim others\"\n  description: \"Darken windows\"\n  keywords: [\"dim\", \"focus\"]\n}\nSettingRow { label: \"Hidden\"\n catalog: false }\n",
        )
        .unwrap();
        std::fs::write(
            pages.join("windows/RulesPage.qml"),
            "PrefsGroup { title: \"Rules\" }",
        )
        .unwrap();
        let got = query(&ctx(), &dir, "FOCUS");
        let labels: Vec<_> = got["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["label"].as_str().unwrap())
            .collect();
        assert_eq!(labels, ["Dim others"]);
        assert_eq!(got["hits"][0]["hubTitle"], "Windows");
        let all = query(&ctx(), &dir, "");
        assert_eq!(
            all["hits"].as_array().unwrap().len(),
            3,
            "hub, row and subpage group"
        );
        assert!(query(&ctx(), &dir, "nope")["hits"]
            .as_array()
            .unwrap()
            .is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn slugs_and_blocks_follow_the_old_scanner() {
        assert_eq!(slug("Dim others!"), "dim-others");
        assert_eq!(slug("!!"), "row");
        let blocks = named_blocks("A { x: { y: 1 } } B { } A { z }", "A");
        assert_eq!(blocks, ["{ x: { y: 1 } }", "{ z }"]);
    }
}
