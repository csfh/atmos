//! The pages the window draws from the catalog are their QML pages (docs/adr/0021): every hub and
//! hub page that is not ported has one, its file is the one `Hubs.js` names, its description is
//! the QML page's own, its groups come in the QML's order, and its groups and rows are exactly
//! what `search.query` reads from that page, both ways. A QML page that gains, loses or rewords
//! a row fails here until its catalog page follows. Without a built `atmosd` or `node` the tests
//! that need one say so and stop.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use std::collections::{BTreeMap, BTreeSet};

use atmos_ui::app::PORTED;
use atmos_ui::catalog::{self, Field, Page, Shown, When};
use atmos_ui::hubs;
use common::qml;
use serde_json::{Value, json};

/// Every hub and hub page, by id.
fn every_page() -> Vec<&'static str> {
    hubs::HUBS
        .iter()
        .flat_map(|h| std::iter::once(h.id).chain(h.children.iter().map(|c| c.id)))
        .collect()
}

#[test]
fn every_page_that_is_not_ported_is_in_the_catalog_and_no_ported_one_is() {
    for id in every_page() {
        let root = id.split('/').next().unwrap();
        assert_eq!(catalog::page(id).is_some(), !PORTED.contains(&root), "{id}");
    }
}

/// A page's file and its own title and description, as the QML page has them.
#[test]
fn each_page_is_its_qml_file_and_says_what_it_says() {
    let Some(files) = qml::node(
        r#"
const H = require(process.env.REPO_DIR + "/services/Hubs.js");
process.stdout.write(JSON.stringify(H.searchContext().files));
"#,
    ) else {
        eprintln!("skip: node is not installed");
        return;
    };
    for page in catalog::pages() {
        assert_eq!(files[page.file], json!(page.id), "{}", page.file);
        let src = std::fs::read_to_string(qml::repo().join("pages").join(page.file)).unwrap();
        let own = |name: &str| {
            let head = format!("\n  {name}: \"");
            src.split(&head)
                .nth(1)
                .and_then(|rest| rest.split_once("\"\n"))
                .map(|(text, _)| text.replace("\\\"", "\""))
        };
        assert_eq!(
            own("description").unwrap_or_default(),
            page.description,
            "{}",
            page.id
        );
        if let Some(title) = own("title") {
            assert_eq!(title, page.title(), "{}", page.id);
        }
        groups_in_order(page, &src);
    }
}

/// The page's groups come in the order their titles do in the QML file.
fn groups_in_order(page: &Page, src: &str) {
    let mut last = 0;
    for group in page.groups.iter().filter(|g| !g.title.is_empty()) {
        let quoted = serde_json::to_string(group.title).unwrap();
        let at = src[last..]
            .find(&format!("title: {quoted}"))
            .map(|at| last + at)
            .unwrap_or_else(|| panic!("{}: the group {:?} is out of order", page.id, group.title));
        last = at + 1;
    }
}

fn row_words(hub: &str, label: &str, description: &str, hint: &str, keywords: &Value) -> String {
    json!([hub, label, description, hint, keywords]).to_string()
}

fn group_words(hub: &str, title: &str, detail: &str) -> String {
    json!([hub, title, detail]).to_string()
}

/// The text from `at`'s brace to the one that closes it.
fn span(src: &str, at: usize) -> (usize, usize) {
    let brace = at + src[at..].find('{').unwrap();
    let mut depth = 0;
    for (i, c) in src[brace..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return (at, brace + i);
                }
            }
            _ => {}
        }
    }
    (at, src.len())
}

/// The groups whose QML builds rows with a `Repeater` are the ones `MODEL_LISTS` names: a page
/// that draws only the literal rows of such a group is not whole.
#[test]
fn every_group_that_builds_rows_from_a_model_is_named() {
    let mut theirs = BTreeSet::new();
    for page in catalog::pages() {
        let src = std::fs::read_to_string(qml::repo().join("pages").join(page.file)).unwrap();
        let groups: Vec<(usize, usize, String)> = src
            .match_indices("PrefsGroup {")
            .map(|(at, _)| {
                let (s, e) = span(&src, at);
                let title = src[s..e]
                    .split("title: \"")
                    .nth(1)
                    .and_then(|t| t.split('"').next())
                    .unwrap_or_default()
                    .to_owned();
                (s, e, title)
            })
            .collect();
        for (at, _) in src.match_indices("Repeater {") {
            let group = groups
                .iter()
                .filter(|(s, e, _)| *s < at && at < *e)
                .max_by_key(|(s, _, _)| *s)
                .map_or(String::new(), |(_, _, t)| t.clone());
            theirs.insert((page.id.to_owned(), group));
        }
    }
    let ours: BTreeSet<(String, String)> = catalog::MODEL_LISTS
        .iter()
        .map(|(p, g)| ((*p).to_owned(), (*g).to_owned()))
        .collect();
    assert_eq!(ours, theirs);
}

/// The text of a QML block that is its own, outside the blocks nested in it.
fn own(block: &str) -> String {
    let mut depth = 0;
    let mut out = String::new();
    for c in block.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ if depth == 1 => out.push(c),
            _ => {}
        }
    }
    out
}

/// The groups whose QML says `wide: true` are the ones `WIDE` names, and a group the QML leaves
/// without a title is wide: the window gives every group with no title a row of its own. A group
/// whose title is worked out (one per monitor, from a model) is in a list the window draws.
#[test]
fn every_group_that_takes_the_whole_width_is_named() {
    let mut theirs = BTreeSet::new();
    for page in catalog::pages() {
        let src = std::fs::read_to_string(qml::repo().join("pages").join(page.file)).unwrap();
        for (at, _) in src.match_indices("PrefsGroup {") {
            let (s, e) = span(&src, at);
            let own = own(&src[s..=e]);
            let wide = own.lines().any(|l| l.trim() == "wide: true");
            let title = own
                .split("title: \"")
                .nth(1)
                .and_then(|t| t.split('"').next())
                .unwrap_or_default();
            if !own.contains("title:") {
                assert!(wide, "{}: a group with no title is not wide", page.id);
            } else if wide && !title.is_empty() {
                theirs.insert((page.id.to_owned(), title.to_owned()));
            }
        }
    }
    let ours: BTreeSet<(String, String)> = catalog::WIDE
        .iter()
        .map(|(p, g)| ((*p).to_owned(), (*g).to_owned()))
        .collect();
    assert_eq!(ours, theirs);
}

/// The values a field of a condition can read, as the condition reads it: a switch's booleans,
/// or a text that is empty or not.
fn domains(w: When, out: &mut BTreeMap<String, Value>) {
    match w {
        When::On(Field::Key(k)) | When::Off(Field::Key(k)) => {
            out.insert(k.to_owned(), json!([false, true]));
        }
        When::Has(Field::Key(k)) => {
            out.insert(k.to_owned(), json!(["", "x"]));
        }
        When::All(all) | When::Any(all) => all.iter().for_each(|w| domains(*w, out)),
        other => panic!("no values for {other:?}"),
    }
}

/// The groups a QML page hides while a condition is false (`query: cond ? root.query : "."`)
/// are the ones `SHOWN` names, with that condition; each `When` holds where the QML's condition
/// does, run under Node over every value its fields can read.
#[test]
fn every_group_the_qml_hides_is_named_with_its_condition() {
    let mut theirs = BTreeSet::new();
    for page in catalog::pages() {
        let src = std::fs::read_to_string(qml::repo().join("pages").join(page.file)).unwrap();
        for (at, _) in src.match_indices("PrefsGroup {") {
            let (s, e) = span(&src, at);
            let block = &src[s..e];
            let title = block
                .split("title: \"")
                .nth(1)
                .and_then(|t| t.split('"').next())
                .unwrap_or_default();
            let cond = block.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("query: ")?
                    .strip_suffix(" ? root.query : \".\"")
            });
            if let Some(cond) = cond {
                theirs.insert((page.id.to_owned(), title.to_owned(), cond.to_owned()));
            }
        }
    }
    let ours: BTreeSet<(String, String, String)> = catalog::SHOWN
        .iter()
        .map(|(p, g, c, _)| ((*p).to_owned(), (*g).to_owned(), (*c).to_owned()))
        .collect();
    assert_eq!(ours, theirs);
    let cases: Vec<Value> = catalog::SHOWN
        .iter()
        .filter_map(|(p, g, qml, shown)| match shown {
            Shown::When(w) => {
                let mut fields = BTreeMap::new();
                domains(*w, &mut fields);
                Some(json!({"at": format!("{p} / {g}"), "qml": qml, "ours": rows::qml_when(*w), "fields": fields}))
            }
            Shown::Page(_) => None,
        })
        .collect();
    let script = format!(
        r#"
const cases = {cases};
const wrong = [];
for (const c of cases) {{
  const names = Object.keys(c.fields);
  const test = (expr) => new Function("Omarchy", "return !!(" + expr + ");");
  const [qml, ours] = [test(c.qml), test(c.ours)];
  const each = (i, Omarchy) => {{
    if (i === names.length) {{
      if (qml(Omarchy) !== ours(Omarchy)) wrong.push([c.at, Omarchy]);
      return;
    }}
    for (const v of c.fields[names[i]]) each(i + 1, Object.assign({{}}, Omarchy, {{ [names[i]]: v }}));
  }};
  each(0, {{}});
}}
process.stdout.write(JSON.stringify(wrong));
"#,
        cases = json!(cases)
    );
    let Some(wrong) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    assert_eq!(wrong, json!([]));
}

/// A page that is whole draws the words its QML builds from what the window holds: each of its
/// rows whose QML `description:` is not a literal has a `varies`, `words`, or a custom control
/// that speaks, and a row whose label is not a literal is drawn by its group's list.
#[test]
fn a_whole_page_draws_the_words_its_qml_builds() {
    let literal = |t: &str| {
        t.is_empty()
            || (t.starts_with('"') && t.ends_with('"') && t[1..t.len() - 1].find('"').is_none())
    };
    let mut wrong = Vec::new();
    for page in catalog::pages().filter(|p| p.drawn()) {
        let pages = qml::repo().join("pages");
        let mut src = std::fs::read_to_string(pages.join(page.file)).unwrap();
        for entry in std::fs::read_dir(pages.join("rows")).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            if src.contains(&format!("{name} {{")) {
                src.push_str(&std::fs::read_to_string(&path).unwrap());
            }
        }
        let spans = |what: &str| -> Vec<(usize, usize)> {
            src.match_indices(what)
                .map(|(at, _)| span(&src, at))
                .collect()
        };
        let (repeaters, groups) = (spans("Repeater {"), spans("PrefsGroup {"));
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for (at, _) in src.match_indices("SettingRow {") {
            if repeaters.iter().any(|(s, e)| *s < at && at < *e) {
                continue;
            }
            let (s, e) = span(&src, at);
            let block = &src[s..e];
            let (label, says) = (
                rows::property(block, "label"),
                rows::property(block, "description"),
            );
            if !literal(&label) {
                let group = groups
                    .iter()
                    .filter(|(gs, ge)| *gs < at && at < *ge)
                    .max_by_key(|(gs, _)| *gs)
                    .map_or(String::new(), |(gs, ge)| {
                        src[*gs..*ge]
                            .split("title: \"")
                            .nth(1)
                            .and_then(|t| t.split('"').next())
                            .unwrap_or_default()
                            .to_owned()
                    });
                if catalog::list(page.id, &group).is_none() {
                    wrong.push(format!("{} / {label}: no list draws it", page.id));
                }
                continue;
            }
            let name: String = serde_json::from_str(&label).unwrap();
            let n = seen.entry(name.clone()).or_default();
            let line = page.lines().filter(|l| l.label == name).nth(*n);
            *n += 1;
            if literal(&says) {
                continue;
            }
            let Some(line) = line else {
                wrong.push(format!("{} / {name}: not in the catalog", page.id));
                continue;
            };
            let drawn = line.varies.is_some()
                || line.words.is_some()
                || matches!(line.control, atmos_ui::catalog::Control::Custom(c) if c.speaks());
            if !drawn {
                wrong.push(format!("{} / {name}: {says}", page.id));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A group a QML page keeps out of the catalog (`catalog: false`) on a page that is drawn is one
/// `UNLISTED` draws after the group it follows, with its title and words, or one the window has
/// no need of, said here.
#[test]
fn every_group_kept_out_of_the_catalog_is_drawn_or_said_not_to_be() {
    let needless: &[(&str, &str, &str)] = &[
        (
            "system",
            "Debug",
            "it shows the QML window's own error dialog; this window toasts",
        ),
        (
            "services",
            "",
            "its summary is the services list's (List::Units)",
        ),
    ];
    for page in catalog::pages().filter(|p| p.drawn()) {
        let src = std::fs::read_to_string(qml::repo().join("pages").join(page.file)).unwrap();
        let mut before = String::new();
        for (at, _) in src.match_indices("PrefsGroup {") {
            let (s, e) = span(&src, at);
            let block = &src[s..e];
            let head: String = block
                .lines()
                .take_while(|l| {
                    !l.trim_start().starts_with("SettingRow {")
                        && !l.contains("Column {")
                        && !l.contains("Repeater {")
                })
                .collect::<Vec<_>>()
                .join("\n");
            let title = block
                .split("title: \"")
                .nth(1)
                .and_then(|t| t.split('"').next())
                .unwrap_or_default()
                .to_owned();
            if head.contains("catalog: false") {
                let detail = block
                    .split("detail: \"")
                    .nth(1)
                    .and_then(|t| t.split('"').next())
                    .unwrap_or_default();
                let drawn = catalog::UNLISTED.iter().any(|(p, after, t, d, _)| {
                    *p == page.id && *after == before && *t == title && *d == detail
                });
                let said = needless
                    .iter()
                    .any(|(p, t, _)| *p == page.id && *t == title);
                assert!(drawn || said, "{} / {title:?}", page.id);
            } else {
                before = title;
            }
        }
    }
    for (p, after, title, ..) in catalog::UNLISTED {
        assert!(
            catalog::page(p).is_some_and(|page| page.groups.iter().any(|g| g.title == *after)),
            "{p} / {title}"
        );
    }
}

/// The built-in profiles a list draws are `Profiles.builtins()`.
#[test]
fn the_profiles_are_those_of_profiles_js() {
    let Some(js) = qml::node(
        r#"
const P = require(process.env.REPO_DIR + "/services/Profiles.js");
process.stdout.write(JSON.stringify(P.builtins()));
"#,
    ) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<Value> = atmos_ui::profiles::PROFILES
        .iter()
        .map(|p| {
            let values: serde_json::Map<String, Value> = p
                .values()
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect();
            json!({"id": p.id, "title": p.title, "description": p.description, "values": values})
        })
        .collect();
    assert_eq!(Value::Array(ours), js);
}

#[test]
fn the_catalog_has_the_rows_and_groups_search_reads_from_the_qml_and_no_others() {
    let Some(hits) = qml::hits() else {
        return;
    };
    let (mut rows, mut groups) = (BTreeSet::new(), BTreeSet::new());
    for hit in &hits {
        let hub = hit["hub"].as_str().unwrap();
        if hit["id"] == hit["hub"] || catalog::page(hub).is_none() {
            continue;
        }
        let s = |k: &str| hit[k].as_str().unwrap().to_owned();
        if hit["id"].as_str().unwrap().contains("-PrefsGroup-") {
            groups.insert(group_words(hub, &s("label"), &s("detail")));
        } else {
            rows.insert(row_words(
                hub,
                &s("label"),
                &s("description"),
                &s("hint"),
                &hit["keywords"],
            ));
        }
    }
    let (mut our_rows, mut our_groups) = (BTreeSet::new(), BTreeSet::new());
    for page in catalog::pages() {
        for group in page.groups {
            if !group.title.is_empty() {
                our_groups.insert(group_words(page.id, group.title, group.detail));
            }
            for l in group.rows {
                our_rows.insert(row_words(
                    page.id,
                    l.label,
                    l.description,
                    l.hint,
                    &json!(l.keywords),
                ));
            }
        }
    }
    let missing: Vec<_> = rows.difference(&our_rows).collect();
    let extra: Vec<_> = our_rows.difference(&rows).collect();
    assert!(missing.is_empty(), "rows the catalog lacks: {missing:#?}");
    assert!(extra.is_empty(), "rows the QML does not have: {extra:#?}");
    let missing: Vec<_> = groups.difference(&our_groups).collect();
    let extra: Vec<_> = our_groups.difference(&groups).collect();
    assert!(missing.is_empty(), "groups the catalog lacks: {missing:#?}");
    assert!(extra.is_empty(), "groups the QML does not have: {extra:#?}");
}

/// The SSH key check is `RichUi.parseSshPublicKey`.
#[test]
fn the_ssh_key_check_is_richui_s() {
    let blob = "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";
    let samples = vec![
        format!("ssh-ed25519 {blob}"),
        format!("ssh-ed25519 {blob} me@host"),
        format!("  ssh-rsa {blob}   two words  "),
        format!("sk-ssh-ed25519@openssh.com {blob}"),
        format!("ecdsa-sha2-nistp256 {blob}=="),
        format!("ecdsa-sha2-nistp999 {blob}"),
        format!("ssh-ed25519-cert-v01@openssh.com {blob}"),
        format!("ssh-ED25519 {blob}"),
        format!("ssh-ed25519 {blob}==="),
        format!("-ssh-ed25519 {blob}"),
        format!("ssh-ed25519 {blob}\nssh-rsa {blob}"),
        "ssh-ed25519 short".to_owned(),
        "-----BEGIN OPENSSH PRIVATE KEY-----".to_owned(),
        "ssh-ed25519".to_owned(),
        String::new(),
    ];
    let script = format!(
        r#"
const R = require(process.env.REPO_DIR + "/tests/harness.js").load("services/RichUi.js");
process.stdout.write(JSON.stringify({}.map((s) => R.parseSshPublicKey(s))));
"#,
        json!(samples)
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let ours: Vec<String> = samples
        .iter()
        .map(|s| atmos_ui::security::ssh_public_key(s))
        .collect();
    assert_eq!(json!(ours), theirs);
}

/// A blocked page is drawn but for what it is blocked on.
#[test]
fn a_blocked_page_is_drawn_but_for_its_block() {
    for (id, why) in catalog::BLOCKED {
        let page = catalog::page(id).unwrap_or_else(|| panic!("{id} is a catalog page"));
        assert!(page.drawn() && !page.mapped(), "{id}");
        assert!(!why.is_empty());
    }
}

/// Says which pages are whole, for whoever runs the tests with `--nocapture`.
#[test]
fn which_pages_are_whole() {
    let whole: Vec<_> = catalog::pages()
        .filter(|p| p.mapped())
        .map(|p| p.id)
        .collect();
    eprintln!("whole: {} {whole:?}", whole.len());
}
