use super::*;
use crate::command::table::{self, In, Kind, A, T};
use serde_json::{json, Map, Value};

/// Arguments that satisfy an entry, worked out from its template.
fn sample(entry: &table::Entry) -> Value {
    let mut args = Map::new();
    let value = |a: &A| -> Value {
        match a {
            A::Word | A::Ident | A::Label => json!("w"),
            A::Text => json!("some text"),
            A::Path => json!("/tmp/x"),
            A::Uint(min, _) => json!(*min),
            A::Uuid => json!("123e4567-e89b-12d3-a456-426614174000"),
            A::Addr => json!("AA:BB:CC:DD:EE:FF"),
            A::Is(list) => json!(list[0]),
            A::Url => json!("https://example.org/x"),
            A::Json => json!("{}"),
        }
    };
    for token in entry.argv {
        match token {
            T::V(field, a) | T::O(field, a) | T::Z(field, a) | T::J(_, field, a) => {
                args.insert(field.to_string(), value(a));
            }
            T::F(field, _) | T::B(field, _, _) => {
                args.insert(field.to_string(), json!(true));
            }
            T::Rest(field, a, min, _) => {
                args.insert(
                    field.to_string(),
                    json!((0..*min).map(|_| value(a)).collect::<Vec<_>>()),
                );
            }
            T::L(_) | T::Task(_) => {}
        }
    }
    match entry.stdin {
        In::Field(f) | In::Line(f) | In::LineOpt(f) => {
            args.insert(f.to_string(), json!("secret"));
        }
        In::Lines(fields) => {
            for f in fields {
                args.insert(f.to_string(), json!("secret"));
            }
        }
        In::No => {}
    }
    Value::Object(args)
}

#[test]
fn every_entry_builds_from_arguments_its_template_asks_for() {
    for entry in ENTRIES {
        let args = sample(entry);
        let argv = table::argv(entry, &args).unwrap_or_else(|err| panic!("{}: {err}", entry.name));
        assert!(!argv.is_empty() && !argv[0].is_empty(), "{}", entry.name);
        let stdin =
            table::stdin(entry, &args).unwrap_or_else(|err| panic!("{}: {err}", entry.name));
        assert_eq!(
            stdin.is_some(),
            !matches!(entry.stdin, In::No),
            "{}",
            entry.name
        );
    }
}

#[test]
fn names_are_unique_and_every_one_has_a_row_in_the_protocol_doc() {
    let doc = include_str!("../../../docs/protocol.md");
    let mut seen = std::collections::HashSet::new();
    for entry in ENTRIES {
        assert!(seen.insert(entry.name), "{} appears twice", entry.name);
        assert!(
            doc.contains(&format!("| `{}` |", entry.name)),
            "docs/protocol.md has no row for {}",
            entry.name
        );
    }
    assert!(ENTRIES.len() >= 100, "{}", ENTRIES.len());
}

#[test]
fn a_write_or_a_job_name_is_found_only_under_its_own_op() {
    for name in names(&[Kind::Command, Kind::Launch]) {
        assert!(
            crate::command::writes::build(name, &sample(find(name).unwrap())).is_ok(),
            "{name}"
        );
        assert!(
            crate::command::jobs::build(name, &Value::Null).is_err(),
            "{name} is not a job"
        );
    }
    for name in names(&[Kind::Job, Kind::Interactive]) {
        assert!(
            crate::command::jobs::build(name, &sample(find(name).unwrap())).is_ok(),
            "{name}"
        );
        assert!(
            crate::command::writes::build(name, &sample(find(name).unwrap())).is_err(),
            "{name} is not a write"
        );
    }
}

fn argv(name: &str, args: Value) -> Vec<String> {
    table::argv(find(name).unwrap(), &args).unwrap()
}

#[test]
fn the_command_lines_are_what_the_window_used_to_build() {
    assert_eq!(
        argv("theme.install", json!({"url": "https://example.org/t.git"})),
        ["omarchy", "theme", "install", "https://example.org/t.git"]
    );
    assert_eq!(
        argv(
            "capture.recording",
            json!({"fullscreen": true, "webcam": true, "webcamSize": "large", "microphone": false})
        ),
        [
            "omarchy",
            "capture",
            "screenrecording",
            "--fullscreen",
            "--with-webcam",
            "--webcam-size=large"
        ]
    );
    assert_eq!(
        argv("capture.recording", json!({})),
        ["omarchy", "capture", "screenrecording"]
    );
    assert_eq!(
        argv("hook.run", json!({"name": "post-update"})),
        ["omarchy", "hook", "post-update"]
    );
    assert_eq!(
        argv("hook.run", json!({"name": "theme-set", "arg": "tokyo"})),
        ["omarchy", "hook", "theme-set", "tokyo"]
    );
    assert_eq!(
        argv(
            "app.webapp-install",
            json!({"name": "Docs", "url": "https://d.example"})
        ),
        [
            "omarchy",
            "webapp",
            "install",
            "Docs",
            "https://d.example",
            ""
        ],
        "an icon that is not given is an empty argument, as before"
    );
    assert_eq!(
        argv(
            "wifi.ipv4-manual",
            json!({"uuid": "123e4567-e89b-12d3-a456-426614174000", "address": "10.0.0.2", "prefix": 24})
        ),
        [
            argv(
                "wifi.metered",
                json!({"uuid": "123e4567-e89b-12d3-a456-426614174000", "value": "yes"})
            )[0]
            .clone(),
            "task".into(),
            "set-wifi-connection".into(),
            "ipv4".into(),
            "123e4567-e89b-12d3-a456-426614174000".into(),
            "manual".into(),
            "10.0.0.2".into(),
            "24".into(),
            "".into()
        ],
    );
    assert_eq!(
        argv(
            "software.run",
            json!({"action": "install", "words": ["browser", "firefox"]})
        )[1..],
        ["task", "gum-job", "omarchy", "install", "browser", "firefox"]
    );
    assert_eq!(
        argv(
            "systemd.system",
            json!({"action": "restart", "unit": "sshd.service"})
        ),
        ["sudo", "-n", "systemctl", "restart", "sshd.service"]
    );
}

#[test]
fn a_secret_travels_as_stdin_and_is_never_in_the_command_line() {
    let args = json!({"name": "me", "full": "Me", "wheel": true, "password": "hunter2"});
    let entry = find("account.add-user").unwrap();
    assert!(!table::argv(entry, &args)
        .unwrap()
        .iter()
        .any(|a| a.contains("hunter2")));
    assert_eq!(
        table::stdin(entry, &args).unwrap().as_deref(),
        Some("hunter2\n")
    );
    let spot = find("wifi.hotspot-on").unwrap();
    let hotspot = json!({"ssid": "Cafe", "password": "pa55word!"});
    assert!(!table::argv(spot, &hotspot)
        .unwrap()
        .iter()
        .any(|a| a.contains("pa55word")));
    assert_eq!(
        table::stdin(spot, &hotspot).unwrap().as_deref(),
        Some("pa55word!\n")
    );
    let luks = find("luks.change-key").unwrap();
    let keys = json!({"device": "/dev/sda2", "current": "old", "new": "new"});
    assert_eq!(
        table::stdin(luks, &keys).unwrap().as_deref(),
        Some("/dev/sda2\nold\nnew\n")
    );
    // A password with a line break would be two lines.
    assert!(table::stdin(entry, &json!({"password": "a\nb"})).is_err());
    assert!(
        table::stdin(find("wifi.join").unwrap(), &json!({"ssid": "x"}))
            .unwrap()
            .is_none()
    );
}

#[test]
fn arguments_cannot_become_options_or_climb_out_of_a_path() {
    let bad: Vec<(&str, Value)> = vec![
        ("theme.install", json!({"url": "--help"})),
        ("theme.install", json!({"url": "file:///etc/passwd"})),
        ("share.file", json!({"path": "relative"})),
        ("share.file", json!({"path": "/a/../etc/passwd"})),
        ("open.path", json!({"path": "-n"})),
        (
            "systemd.user",
            json!({"action": "mask", "unit": "x.service"}),
        ),
        ("systemd.user", json!({"action": "start", "unit": "--now"})),
        ("systemd.user", json!({"action": "start", "unit": "a b"})),
        ("process.signal", json!({"pid": 1, "signal": "TERM"})),
        ("process.signal", json!({"pid": 99, "signal": "HUP"})),
        ("bluetooth.pair", json!({"address": "nope"})),
        (
            "app.desktop-install",
            json!({"name": "a/b", "command": "x", "icon": "i"}),
        ),
        (
            "app.desktop-install",
            json!({"name": "-x", "command": "x", "icon": "i"}),
        ),
        ("channel.set", json!({"name": "nightly"})),
        (
            "software.run",
            json!({"action": "install", "words": ["only-one"]}),
        ),
        (
            "software.run",
            json!({"action": "install", "words": ["a", "b", "c", "d"]}),
        ),
        (
            "software.run",
            json!({"action": "install", "words": ["a", "--force"]}),
        ),
        ("sudo.enable", json!({"minutes": 0, "password": "x"})),
        ("sudo.enable", json!({"minutes": 241, "password": "x"})),
        (
            "wifi.priority",
            json!({"uuid": "123e4567-e89b-12d3-a456-426614174000", "value": 1000}),
        ),
    ];
    for (name, args) in bad {
        let entry = find(name).unwrap();
        assert!(
            table::argv(entry, &args).is_err() || table::stdin(entry, &args).is_err(),
            "{name} {args}"
        );
    }
}

#[test]
fn jobs_on_a_named_lane_share_it_and_the_rest_each_get_their_own() {
    let a = crate::command::jobs::build("refresh.shell", &json!({})).unwrap();
    let b = crate::command::jobs::build("refresh.shell", &json!({})).unwrap();
    assert_ne!(a.lane, b.lane, "no named lane, so nothing waits");
    let c = crate::command::jobs::build("update.omarchy", &json!({})).unwrap();
    let d = crate::command::jobs::build("update.firmware", &json!({})).unwrap();
    assert_eq!(c.lane, d.lane, "package work goes one at a time");
    assert!(
        c.affects.contains(&"services".to_string()) && c.affects.contains(&"settings".to_string())
    );
}

/// Prints the rows of docs/protocol.md for the entries:
/// `cargo test print_the_protocol_rows -- --ignored --nocapture`.
#[test]
#[ignore]
fn print_the_protocol_rows() {
    for (title, kinds) in [
        ("Commands (`write`)", vec![Kind::Command]),
        ("Launchers (`write`)", vec![Kind::Launch]),
        ("Jobs (`job.start`)", vec![Kind::Job]),
        ("Interactive tools (`job.start`)", vec![Kind::Interactive]),
    ] {
        println!("\n##### {title}\n\n| Name | Args | Runs |\n| --- | --- | --- |");
        for entry in ENTRIES.iter().filter(|e| kinds.contains(&e.kind)) {
            let mut fields: Vec<String> = Vec::new();
            let mut line: Vec<String> = Vec::new();
            for token in entry.argv {
                match token {
                    T::L(l) => line.push((*l).to_string()),
                    T::Task(n) => line.push(format!("ratmos task {n}")),
                    T::V(f, _) => {
                        fields.push((*f).to_string());
                        line.push(format!("<{f}>"));
                    }
                    T::O(f, _) | T::Z(f, _) => {
                        fields.push(format!("{f}?"));
                        line.push(format!("[<{f}>]"));
                    }
                    T::F(f, l) => {
                        fields.push(format!("{f}?"));
                        line.push(format!("[{l}]"));
                    }
                    T::B(f, a, b) => {
                        fields.push((*f).to_string());
                        line.push(format!("{a}|{b}"));
                    }
                    T::J(p, f, _) => {
                        fields.push(format!("{f}?"));
                        line.push(format!("[{p}<{f}>]"));
                    }
                    T::Rest(f, _, _, _) => {
                        fields.push((*f).to_string());
                        line.push(format!("<{f}...>"));
                    }
                }
            }
            match entry.stdin {
                In::Field(f) | In::Line(f) => fields.push(format!("{f} (stdin)")),
                In::LineOpt(f) => fields.push(format!("{f}? (stdin)")),
                In::Lines(fs) => fields.extend(fs.iter().map(|f| format!("{f} (stdin)"))),
                In::No => {}
            }
            let mut notes = Vec::new();
            if entry.gum {
                notes.push("with progress");
            }
            if entry.root {
                notes.push("needs root");
            }
            let tail = if notes.is_empty() {
                String::new()
            } else {
                format!(" ({})", notes.join(", "))
            };
            let runs = line.join(" ").replace('|', "\\|");
            println!(
                "| `{}` | {} | `{}`{} |",
                entry.name,
                if fields.is_empty() {
                    String::new()
                } else {
                    fields.join(", ")
                },
                runs,
                tail
            );
        }
    }
}

#[test]
fn a_recording_does_not_hold_the_lane_that_stops_it() {
    // The recording job lasts until the recorder ends, and the stop write is
    // what ends it: on one lane the stop would wait for the job forever.
    let job =
        crate::command::jobs::build("capture.recording", &json!({"fullscreen": true})).unwrap();
    let stop = crate::command::writes::build("recording.stop", &json!({})).unwrap();
    assert_ne!(job.lane, stop.lane);
}

#[test]
fn an_interactive_tool_holds_no_lane() {
    // It waits on the user, for as long as they take. A lane held that long
    // would stop every write that shares it.
    for name in names(&[Kind::Interactive]) {
        let entry = find(name).unwrap();
        assert!(
            entry.lane.is_empty(),
            "{name} names the lane {}",
            entry.lane
        );
    }
}

/// What an interactive tool can change, and so which settings topics are read
/// again when it ends (docs/adr/0016). A new tool fails here until it is
/// classified.
#[test]
fn an_interactive_tool_reads_again_the_topics_it_can_change_when_it_ends() {
    use crate::snapshot::topics::{HOST, LOOK};
    let expected: &[(&str, &[&str])] = &[
        ("theme.switcher", &[LOOK]),
        ("theme.bg-switcher", &[LOOK]),
        ("theme.bg-file", &[LOOK]),
        ("branding.screensaver", &[LOOK]),
        ("branding.about", &[LOOK]),
        ("capture.screenshot", &[]),
        ("capture.recording", &[HOST]),
        ("capture.text", &[]),
        ("capture.qr", &[]),
        ("tailscale.send", &[]),
        ("tailscale.receive", &[]),
    ];
    let interactive = names(&[Kind::Interactive]);
    assert_eq!(
        interactive.len(),
        expected.len(),
        "classify every interactive tool: {interactive:?}"
    );
    for (name, topics) in expected {
        assert!(
            interactive.contains(name),
            "{name} is not an interactive tool"
        );
        let spec = crate::command::jobs::build(name, &sample(find(name).unwrap())).unwrap();
        let settings: Vec<&str> = spec
            .affects
            .iter()
            .map(String::as_str)
            .filter(|topic| crate::sampler::is_settings(topic))
            .collect();
        assert_eq!(&settings, topics, "{name}");
    }
}

#[test]
fn every_name_an_entry_affects_is_a_topic_the_daemon_holds() {
    for entry in ENTRIES {
        for topic in entry.affects {
            assert!(
                crate::sampler::is_topic(topic),
                "{} affects {topic}, which is no topic",
                entry.name
            );
        }
    }
}
