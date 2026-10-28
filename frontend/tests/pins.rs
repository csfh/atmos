//! What the window copies from the app it replaces is pinned to its source, so a change on one
//! side fails here instead of showing up as a missing page or a `bad_request`.

use atmos_ui::{hubs, protocol};
use std::fs;
use std::path::PathBuf;

fn repo(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn every_op_is_one_the_backend_answers() {
    let request = repo("backend/src/request.rs");
    let serve = repo("backend/src/serve.rs");
    for op in protocol::OPS {
        let variant = format!("#[serde(rename = \"{op}\")]");
        // `serve` answers some ops itself: as a comparison or as an arm of its `match`.
        let served = [format!("Some(\"{op}\")"), format!("\"{op}\" =>")];
        assert!(
            request.contains(&variant) || served.iter().any(|s| serve.contains(s)),
            "`{op}` is neither a Request variant nor an op serve answers"
        );
    }
}

#[test]
fn the_hubs_are_the_launchers_in_the_same_order() {
    let launch = repo("backend/src/launch.rs");
    let list = launch
        .split("const HUBS: &[&str] = &[")
        .nth(1)
        .and_then(|rest| rest.split("];").next())
        .expect("launch.rs has a HUBS list");
    let launcher: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
    let ours: Vec<&str> = hubs::HUBS.iter().map(|h| h.id).collect();
    assert_eq!(ours, launcher);
}

/// The value of `key: "..."` on a line, if it has that key.
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.trim()
        .strip_prefix(key)?
        .strip_prefix(": \"")?
        .split('"')
        .next()
}

#[test]
fn titles_and_groups_are_those_of_hubs_js() {
    let source = repo("services/Hubs.js");
    // A hub's own fields sit at six spaces; its children's are deeper.
    let own = |line: &&str| line.starts_with("      ") && !line.starts_with("       ");
    let mut found: Vec<(String, String, String)> = Vec::new();
    for line in source.lines().filter(own) {
        if let Some(id) = field(line, "id") {
            found.push((id.into(), String::new(), String::new()));
        } else if let (Some(title), Some(last)) = (field(line, "title"), found.last_mut()) {
            last.1 = title.into();
        } else if let (Some(group), Some(last)) = (field(line, "navGroup"), found.last_mut()) {
            last.2 = group.into();
        }
    }
    let ours: Vec<_> = hubs::HUBS
        .iter()
        .map(|h| (h.id.to_string(), h.title.to_string(), h.group.to_string()))
        .collect();
    assert_eq!(ours, found);
}

#[test]
fn group_headings_are_those_of_layout_js() {
    let source = repo("services/Layout.js");
    let table = source
        .split("var NAV_GROUP_LABELS = {")
        .nth(1)
        .and_then(|rest| rest.split("};").next())
        .expect("Layout.js has NAV_GROUP_LABELS");
    let mut theirs: Vec<(String, String)> = vec![("home".into(), String::new())];
    for line in table.lines() {
        let line = line.trim().trim_end_matches(',');
        if let Some((key, label)) = line.split_once(": \"") {
            theirs.push((key.into(), label.trim_end_matches('"').into()));
        }
    }
    let ours: Vec<_> = hubs::GROUPS
        .iter()
        .map(|(id, label)| (id.to_string(), label.to_string()))
        .collect();
    assert_eq!(ours, theirs);
}

/// The text of the registry entry named `name` in `entries.rs`: from its name to the next entry.
fn entry(name: &str) -> String {
    let source = repo("backend/src/command/entries.rs");
    let at = source
        .find(&format!("\"{name}\""))
        .unwrap_or_else(|| panic!("the registry has no `{name}`"));
    let rest = &source[at..];
    rest[..rest[1..].find("Entry::new(").map_or(rest.len(), |n| n + 1)].to_owned()
}

#[test]
fn the_signal_write_is_the_registry_entry_the_page_assumes() {
    let e = entry("process.signal");
    let (low, high) = atmos_ui::actions::PID_RANGE;
    let range = format!("V(\"pid\", A::Uint({low}, {}))", "4_194_304");
    assert_eq!(high, 4_194_304);
    assert!(e.contains(&range), "the pid range moved: {e}");
    assert!(
        e.contains("V(\"signal\", A::Is(&[\"TERM\", \"KILL\"]))"),
        "{e}"
    );
}

#[test]
fn the_clipboard_write_takes_its_text_on_stdin_under_the_name_the_page_sends() {
    let e = entry("clipboard.copy");
    assert!(e.contains("In::Field(\"text\")"), "{e}");
}

/// The limits are stored as integers: a number from a field that is sent as `80.0` is refused.
#[test]
fn the_three_limits_are_integer_settings_in_the_shell_file() {
    let domains = repo("backend/src/domain.rs");
    for limit in atmos_ui::sentinel::Limit::ALL {
        let row = format!("(\"{}\", \"shell\", Int)", limit.domain());
        assert!(domains.contains(&row), "{row} is not a domain");
    }
}

/// `odk oma dev frontend` builds what `docs/oma.md` says it does: the include lands where the manifest
/// declares the daemon, and both builds are the ones the packaging commands run.
#[test]
fn dev_toml_builds_the_window_and_puts_atmosd_where_the_manifest_runs_the_daemon() {
    let dev = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dev.toml"))
        .expect("frontend/dev.toml");
    let manifest =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manifest.toml"))
            .expect("frontend/manifest.toml");
    assert!(
        dev.contains("executable = \"target/release/atmos-ui\""),
        "{dev}"
    );
    assert!(manifest.contains("executable = \"atmos-ui\""), "{manifest}");
    assert!(
        dev.contains("include = [\"../backend/target/release/atmosd=bin/atmosd\"]"),
        "{dev}"
    );
    assert!(
        manifest.contains("executable = \"bin/atmosd\""),
        "{manifest}"
    );
    for build in [
        "[\"cargo\", \"build\", \"--release\", \"--manifest-path\", \"../backend/Cargo.toml\"]",
        "[\"cargo\", \"build\", \"--release\"]",
    ] {
        assert!(dev.contains(build), "{build} is not built by dev.toml");
    }
}

/// The producer whose keys include `key`, from the `OWNERS` table of `keys.rs`.
fn producer_of<'a>(keys: &'a str, key: &str) -> Option<&'a str> {
    let owners = &keys[keys.find("const OWNERS")?..];
    let owners = &owners[..owners.find("\n];")?];
    let at = owners.find(&format!("\"{key}\","))?;
    let before = &owners[..at];
    let name = &before[before.rfind("Producer::")? + "Producer::".len()..];
    name.split(|c: char| !c.is_alphanumeric()).next()
}

/// The settings topic `fn topic` puts a producer's keys in, as `topics.rs` spells it.
fn topic_of_producer(keys: &str, topics: &str, producer: &str) -> Option<String> {
    let body = keys.split("fn topic(producer: Producer)").nth(1)?;
    let body = body.split("match producer {").nth(1)?;
    let body = &body[..body.find("\n}\n")?];
    let arms: Vec<&str> = body.split("=>").collect();
    let words = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty())
            .map(str::to_owned)
            .collect()
    };
    for i in 0..arms.len() - 1 {
        // The names of an arm follow the previous arm's target and its comma.
        let names = if i == 0 {
            arms[0]
        } else {
            arms[i].split_once(',').map_or("", |(_, rest)| rest)
        };
        if words(names).iter().any(|w| w == producer) {
            let target = arms[i + 1].split("super::topics::").nth(1)?;
            let constant = words(target).into_iter().next()?;
            let line = format!("pub const {constant}: &str = \"");
            let value = topics.split(&line).nth(1)?.split('"').next()?;
            return Some(value.to_owned());
        }
    }
    None
}

/// Each settings page holds the topics its keys are in, and reads back keys a producer owns: a
/// key that moved topic would leave the page waiting for ever, and no page test would see it.
#[test]
fn every_settings_page_holds_the_topics_that_own_its_keys() {
    use atmos_ui::{defaults, settings, tweaks};
    let keys = repo("backend/src/snapshot/keys.rs");
    let topics = repo("backend/src/snapshot/topics.rs");
    for held in tweaks::HELD.iter().chain(defaults::HELD) {
        let producer = producer_of(&keys, held.key)
            .unwrap_or_else(|| panic!("no producer owns `{}`", held.key));
        assert_eq!(
            topic_of_producer(&keys, &topics, producer).as_deref(),
            Some(held.topic),
            "`{}` is {producer}'s",
            held.key
        );
    }
    let written = tweaks::CATALOG
        .iter()
        .map(|t| (t.key, tweaks::HELD))
        .chain(defaults::PICKERS.iter().map(|p| (p.domain, defaults::HELD)));
    for (domain, held) in written {
        assert!(
            held.iter().any(|h| h.key == settings::head(domain)),
            "{domain} is a key its page does not hold"
        );
        for key in settings::readback(domain) {
            assert!(
                producer_of(&keys, key).is_some(),
                "no producer owns `{key}`"
            );
        }
    }
}
