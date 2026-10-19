use super::*;

fn argv(name: &str, args: Value) -> Vec<String> {
    build(name, &args).unwrap().argv
}

#[test]
fn every_name_builds_with_valid_arguments() {
    let home = std::env::var("HOME").unwrap_or_default();
    let hook = format!("{home}/.config/omarchy/hooks/theme-set.d/x");
    let uuid = "123e4567-e89b-12d3-a456-426614174000";
    let cases: Vec<(&str, Value)> = vec![
        ("audio.output.mute-toggle", json!({})),
        ("audio.input.mute-toggle", json!({})),
        (
            "audio.output.default",
            json!({"id": "55", "name": "alsa_output.x"}),
        ),
        (
            "audio.input.default",
            json!({"id": "56", "name": "alsa_input.x"}),
        ),
        ("power.profile", json!({"name": "performance"})),
        ("wifi.band", json!({"band": "5"})),
        ("wifi.connection", json!({"action": "up", "uuid": uuid})),
        (
            "bluetooth.device",
            json!({"action": "connect", "address": "AA:BB:CC:DD:EE:FF"}),
        ),
        ("plugin.set", json!({"id": "omarchy.x", "on": true})),
        ("bar.spacer", json!({"on": true})),
        ("monitor.internal", json!({"on": false})),
        ("monitor.mirror", json!({"on": true})),
        ("keyboard.backlight", json!({"direction": "up"})),
        ("display.text-size-reset", json!({})),
        ("theme.remove", json!({"name": "tokyo"})),
        ("branding.reset", json!({"which": "about"})),
        ("hypr.reset", json!({"group": "look"})),
        ("hypr.workspace-layout-toggle", json!({})),
        ("hypr.apply", json!({"group": "input", "payload": "{}"})),
        ("atmos.channel", json!({"channel": "stable"})),
        (
            "weather.coordinates",
            json!({"name": "Oslo", "coords": "59.9,10.7"}),
        ),
        ("reminder.set", json!({"minutes": 5, "message": "tea"})),
        ("reminder.clear", json!({})),
        ("recording.stop", json!({})),
        (
            "hook.install",
            json!({"type": "theme-set", "path": "/tmp/x"}),
        ),
        (
            "hook.create",
            json!({"type": "theme-set", "name": "x", "command": "true"}),
        ),
        ("hook.remove", json!({"path": hook})),
        ("hook.sample", json!({"path": hook, "enabled": true})),
        ("tui.remove", json!({"name": "btop"})),
        ("webapp.remove", json!({"name": "Docs"})),
        (
            "app.remove-desktop",
            json!({"id": "x.desktop", "name": "X"}),
        ),
        ("avatar.set", json!({"user": "me", "path": "/tmp/a.png"})),
        ("avatar.clear", json!({"user": "me"})),
        ("snapper.number-limit", json!({"n": 10})),
        ("snapper.timeline", json!({"on": true})),
        ("fstrim", json!({"on": false})),
    ];
    let named: Vec<&str> = cases.iter().map(|(name, _)| *name).collect();
    for name in NAMES {
        assert!(named.contains(name), "no case for {name}");
    }
    assert_eq!(
        named.len(),
        NAMES.len(),
        "a case for a name that is not listed"
    );
    for (name, args) in cases {
        let built = build(name, &args).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert!(
            !built.argv.is_empty() && !built.readback.is_empty(),
            "{name}"
        );
        assert!(!built.lane.is_empty(), "{name}");
    }
}

#[test]
fn every_name_has_a_row_in_the_protocol_doc() {
    let doc = include_str!("../../../docs/protocol.md");
    for name in NAMES {
        assert!(
            doc.contains(&format!("| `{name}` |")),
            "docs/protocol.md has no row for {name}"
        );
    }
}

#[test]
fn argv_is_exactly_what_the_window_used_to_run() {
    assert_eq!(
        argv("audio.output.default", json!({"id": "55", "name": "n"})),
        ["omarchy", "audio", "output", "set", "default", "55", "n"]
    );
    assert_eq!(
        argv("plugin.set", json!({"id": "p", "on": false})),
        ["omarchy", "plugin", "disable", "p"]
    );
    assert_eq!(
        argv("monitor.mirror", json!({"on": true})),
        ["omarchy", "hyprland", "monitor", "internal", "mirror", "on"]
    );
    assert_eq!(
        argv("reminder.set", json!({"minutes": 5})),
        ["omarchy", "reminder", "5"]
    );
    assert_eq!(
        argv("reminder.set", json!({"minutes": 5, "message": "tea time"})),
        ["omarchy", "reminder", "5", "tea time"]
    );
    let spacer_off = argv("bar.spacer", json!({"on": false}));
    assert_eq!(
        spacer_off,
        ["omarchy", "plugin", "disable", "omarchy.spacer"]
    );
}

#[test]
fn a_task_write_runs_this_binary() {
    let line = argv("snapper.timeline", json!({"on": true}));
    assert_eq!(&line[1..], ["task", "set-snapper-policy", "timeline", "on"]);
    assert!(build("fstrim", &json!({"on": true})).unwrap().local);
    assert!(
        !build("plugin.set", &json!({"id": "p", "on": true}))
            .unwrap()
            .local
    );
}

#[test]
fn arguments_cannot_become_options_or_extra_lines() {
    for bad in [
        ("plugin.set", json!({"id": "--help", "on": true})),
        ("plugin.set", json!({"id": "a\nb", "on": true})),
        ("plugin.set", json!({"id": "", "on": true})),
        ("plugin.set", json!({"on": true})),
        ("plugin.set", json!({"id": "p", "on": "yes"})),
        (
            "bluetooth.device",
            json!({"action": "pair", "address": "AA:BB:CC:DD:EE:FF"}),
        ),
        (
            "bluetooth.device",
            json!({"action": "connect", "address": "nope"}),
        ),
        ("wifi.connection", json!({"action": "up", "uuid": "1234"})),
        ("wifi.band", json!({"band": "7"})),
        ("reminder.set", json!({"minutes": 0})),
        ("reminder.set", json!({"minutes": 3, "message": "-rf"})),
        ("snapper.number-limit", json!({"n": 51})),
        ("theme.remove", json!({"name": "../x"})),
        ("theme.remove", json!({"name": ".hidden"})),
        ("hook.remove", json!({"path": "/etc/passwd"})),
        (
            "hook.remove",
            json!({"path": "/home/x/.config/omarchy/hooks/../../x"}),
        ),
        ("avatar.set", json!({"user": "me", "path": "relative.png"})),
        ("nope", json!({})),
    ] {
        assert!(build(bad.0, &bad.1).is_err(), "{} {}", bad.0, bad.1);
    }
}

#[test]
fn hook_paths_are_confined_to_the_hooks_directory() {
    let home = std::env::var("HOME").unwrap_or_default();
    let inside = format!("{home}/.config/omarchy/hooks/post-update.d/x");
    assert!(build("hook.remove", &json!({"path": inside})).is_ok());
    let sample = format!("{home}/.config/omarchy/hooks/post-update.d/x.sample");
    assert!(build("hook.remove", &json!({"path": sample})).is_err());
    assert!(build("hook.remove", &json!({"path": format!("{home}/x")})).is_err());
}

fn snap(pairs: &[(&str, Value)]) -> Snap {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[test]
fn a_flipped_toggle_settles_only_once_the_value_changes() {
    let built = build("audio.output.mute-toggle", &json!({})).unwrap();
    let check = built.settle.as_ref().unwrap();
    let before = snap(&[("audioOutputMuted", json!(false))]);
    assert!(!check(
        &before,
        &snap(&[("audioOutputMuted", json!(false))])
    ));
    assert!(check(&before, &snap(&[("audioOutputMuted", json!(true))])));
    assert!(built.before);
}

#[test]
fn a_row_write_settles_on_the_row_not_on_the_list() {
    let built = build(
        "bluetooth.device",
        &json!({"action": "connect", "address": "AA:BB:CC:DD:EE:FF"}),
    )
    .unwrap();
    let check = built.settle.as_ref().unwrap();
    let none = Snap::new();
    let off = snap(&[(
        "bluetoothDevices",
        json!([
            {"address": "AA:BB:CC:DD:EE:FF", "connected": false},
            {"address": "11:22:33:44:55:66", "connected": true}]),
    )]);
    let on = snap(&[(
        "bluetoothDevices",
        json!([
            {"address": "AA:BB:CC:DD:EE:FF", "connected": true}]),
    )]);
    assert!(
        !check(&none, &off),
        "another device being connected is not it"
    );
    assert!(check(&none, &on));
    let forget = build(
        "bluetooth.device",
        &json!({"action": "forget", "address": "AA:BB:CC:DD:EE:FF"}),
    )
    .unwrap();
    let gone = forget.settle.as_ref().unwrap();
    assert!(!gone(&none, &off));
    assert!(!gone(&none, &none), "no list at all is not a removal");
    assert!(gone(&none, &snap(&[("bluetoothDevices", json!([]))])));
}

#[test]
fn a_removal_by_name_matches_strings_ids_and_names() {
    let check = names_gone("tuiApps", "btop".into());
    let none = Snap::new();
    assert!(!check(
        &none,
        &snap(&[("tuiApps", json!([{"id": "btop"}]))])
    ));
    assert!(!check(
        &none,
        &snap(&[("tuiApps", json!([{"name": "btop"}]))])
    ));
    assert!(!check(&none, &snap(&[("tuiApps", json!(["btop"]))])));
    assert!(check(&none, &snap(&[("tuiApps", json!([{"id": "htop"}]))])));
}

#[test]
fn a_fixture_root_only_reports_the_command() {
    let built = build("plugin.set", &json!({"id": "p", "on": true})).unwrap();
    let out = execute(Some(Path::new("/nonexistent")), &built).unwrap();
    assert_eq!(out["argv"], json!(["omarchy", "plugin", "enable", "p"]));
    assert_eq!(out["settled"], true);
}
