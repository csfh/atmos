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
        let served = [
            format!("Some(\"{op}\")"),
            format!("\"{op}\" =>"),
            format!("op == \"{op}\""),
        ];
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

/// Every hub's title, group, description and pages, as `Hubs.js` gives them under Node; the
/// window's titles for a page path, as `hubTitle` gives them. Without `node` it says so and stops.
#[test]
fn titles_groups_descriptions_and_pages_are_those_of_hubs_js() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    const NODE: &str = r#"
const H = require(process.env.REPO_DIR + "/services/Hubs.js");
const ids = ["home", "monitor/cpu", "system/kernel", "monitor/nothing", "nothing", "", "a/b/c"];
process.stdout.write(JSON.stringify({
  hubs: H.hubs().map((h) => [h.id, h.title, h.navGroup, h.description, h.keywords,
    (h.children || []).map((c) => [c.id, c.title])]),
  titles: ids.map((id) => [id, H.hubTitle(id), !!H.hubById(id)]),
}));
"#;
    let Ok(out) = std::process::Command::new("node")
        .args(["-e", NODE])
        .env("REPO_DIR", &root)
        .output()
    else {
        eprintln!("skip: node is not installed");
        return;
    };
    assert!(out.status.success(), "Hubs.js failed under node");
    let js: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let ours: Vec<serde_json::Value> = hubs::HUBS
        .iter()
        .map(|h| {
            let kids: Vec<_> = h.children.iter().map(|c| [c.id, c.title]).collect();
            serde_json::json!([h.id, h.title, h.group, h.description, h.keywords, kids])
        })
        .collect();
    assert_eq!(serde_json::Value::Array(ours), js["hubs"]);
    for row in js["titles"].as_array().unwrap() {
        let id = row[0].as_str().unwrap();
        assert_eq!(
            hubs::title(id),
            row[1].as_str().unwrap(),
            "the title of {id:?}"
        );
        assert_eq!(
            hubs::known(id),
            row[2].as_bool().unwrap(),
            "whether {id:?} is a hub"
        );
    }
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

/// A star writes the whole list as JSON, and the write reads the list back.
#[test]
fn the_favorites_write_takes_the_list_as_json_and_reads_it_back() {
    let e = entry("favorites.write");
    assert!(e.contains("V(\"json\", A::Json)"), "{e}");
    assert!(e.contains(".readback(&[\"favorites\"])"), "{e}");
}

/// A job the window runs is a registry entry, it waits for sudo mode exactly when the entry runs
/// as root, and the password goes to `sudo.enable` as the line it reads.
#[test]
fn the_jobs_are_registry_entries_that_need_root_when_the_window_asks_for_sudo() {
    use atmos_ui::drivers;
    for job in [drivers::switch_job(), drivers::firmware_job()] {
        let e = entry(job.name);
        assert_eq!(e.contains(".root()"), job.sudo, "{e}");
        let first = e.split("Entry::new(").next().unwrap_or(&e);
        assert!(
            first.contains(" Job,") || first.contains("Job,"),
            "{} is a job: {e}",
            job.name
        );
    }
    // And so does every job a catalog row starts.
    use atmos_ui::catalog::{self, Control, setter::Sends};
    for line in catalog::pages().flat_map(|p| p.lines()) {
        let setters = match line.control {
            Control::Press(_, s) | Control::Calls(_, _, s) => vec![s],
            Control::Buttons(all) => all.iter().map(|b| b.setter).collect(),
            Control::Custom(c) => c.setters(),
            _ => continue,
        };
        for s in setters {
            let Sends::Job(t) = s.sends else {
                continue;
            };
            let e = entry(t.name);
            assert_eq!(e.contains(".root()"), t.sudo, "{e}");
            let first = e.split("Entry::new(").next().unwrap_or(&e);
            assert!(
                first.contains("Job,") || first.contains("Interactive,"),
                "{} is a job: {e}",
                t.name
            );
            assert_eq!(first.contains("Interactive,"), t.interactive, "{e}");
        }
    }
    let sudo = entry("sudo.enable");
    assert!(sudo.contains("V(\"minutes\", A::Uint(1, 240))"), "{sudo}");
    assert!(sudo.contains(".stdin(In::Line(\"password\"))"), "{sudo}");
    const { assert!(atmos_ui::jobs::SUDO_MINUTES <= 240) };
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
fn oma_toml_builds_the_window_and_puts_atmosd_where_the_manifest_runs_the_daemon() {
    let dev = fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("oma.toml"))
        .expect("frontend/oma.toml");
    let manifest =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manifest.toml"))
            .expect("frontend/manifest.toml");
    assert!(
        dev.contains("executable = \"target/release/atmos-ui\""),
        "{dev}"
    );
    assert!(manifest.contains("executable = \"atmos-ui\""), "{manifest}");
    assert!(
        dev.contains("\"../backend/target/release/atmosd=bin/atmosd\""),
        "{dev}"
    );
    // The Bundle carries Atmos's licence and those of what its two programs contain.
    for licence in [
        "\"../LICENSE=share/licenses/LICENSE\"",
        "\"target/release/THIRD-PARTY-LICENSES-atmosd=share/licenses/THIRD-PARTY-LICENSES-atmosd\"",
        "\"target/release/THIRD-PARTY-LICENSES-atmos-ui=share/licenses/THIRD-PARTY-LICENSES-atmos-ui\"",
    ] {
        assert!(dev.contains(licence), "{licence} is not in the Bundle");
    }
    assert!(
        manifest.contains("executable = \"bin/atmosd\""),
        "{manifest}"
    );
    for build in [
        "[\"cargo\", \"build\", \"--release\", \"--manifest-path\", \"../backend/Cargo.toml\"]",
        "[\"cargo\", \"build\", \"--release\"]",
    ] {
        assert!(dev.contains(build), "{build} is not built by oma.toml");
    }
}

/// The producer whose keys include `key`, from the `OWNERS` table of `keys.rs`.
fn producer_of<'a>(keys: &'a str, key: &str) -> Option<&'a str> {
    let owners = &keys[keys.find("const OWNERS")?..];
    let owners = &owners[..owners.find("\n];")?];
    let quoted = format!("\"{key}\"");
    let at = owners
        .match_indices(&quoted)
        .map(|(at, _)| at)
        .find(|at| matches!(owners[at + quoted.len()..].chars().next(), Some(',' | ']')))?;
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

/// An import waits for sudo mode exactly when a change it applies needs root, as the QML page's
/// `doApply` asks (`passwordCount(plan) > 0`): `settings.apply` is no root entry, as its task
/// raises the privileged steps itself; and the undo asks as the import did.
#[test]
fn an_import_asks_for_sudo_mode_when_a_change_needs_root() {
    use atmos_ui::export::State;
    use atmos_ui::omafile::CATALOG;
    for name in ["settings.apply", "settings.undo"] {
        let e = entry(name);
        assert!(!e.contains(".root()"), "{e}");
        assert!(e.contains("Job,"), "{e}");
    }
    let root = CATALOG
        .iter()
        .find(|i| i.needs_root)
        .expect("a setting needs root");
    let plain = CATALOG
        .iter()
        .find(|i| !i.needs_root && i.importable)
        .unwrap();
    for (item, sudo) in [(root, true), (plain, false)] {
        let mut s = State {
            plan: Some(serde_json::json!({"changes": [{"key": item.key, "to": 1, "from": 0}]})),
            ..State::default()
        };
        let job = s.apply((0, 0)).unwrap();
        assert_eq!(
            (job.name, job.kind, job.sudo),
            ("settings.apply", "settings-import", sudo)
        );
        s.last_backup_dir = "/b".into();
        let undo = s.undo((0, 0)).unwrap();
        assert_eq!(
            (undo.name, undo.sudo, undo.args["plan"].as_str()),
            ("settings.undo", sudo, Some("/b/undo.json"))
        );
    }
}

/// Each settings page holds the topics its keys are in, and reads back keys a producer owns: a
/// key that moved topic would leave the page waiting for ever, and no page test would see it.
#[test]
fn every_settings_page_holds_the_topics_that_own_its_keys() {
    use atmos_ui::catalog::control::KEYS;
    use atmos_ui::{catalog, defaults, drivers, favorites, jobs, settings, tweaks};
    let keys = repo("backend/src/snapshot/keys.rs");
    let topics = repo("backend/src/snapshot/topics.rs");
    for held in tweaks::HELD
        .iter()
        .chain(defaults::HELD)
        .chain(drivers::HELD)
        .chain(KEYS.iter().map(|(held, _)| held))
        .chain([&favorites::HELD, &jobs::SUDO])
        .chain(atmos_ui::export::held().iter())
    {
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
        .map(|t| (t.key, tweaks::HELD.to_vec()))
        .chain(
            defaults::PICKERS
                .iter()
                .map(|p| (p.domain, defaults::HELD.to_vec())),
        )
        .chain(catalog::pages().flat_map(|p| {
            p.lines()
                .filter_map(|l| l.control.field())
                .map(move |f| (f.key(), p.held()))
        }));
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
