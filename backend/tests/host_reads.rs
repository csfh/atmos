mod common;

use std::fs;

use common::{cleanup, request, temp_root};

#[test]
fn host_reads_return_fixture_values_and_round_trip_a_file() {
    let root = temp_root();
    fs::create_dir_all(root.join(".local/state/omarchy/current/theme")).unwrap();
    fs::create_dir_all(root.join(".config/omarchy")).unwrap();
    fs::write(
        root.join(".local/state/omarchy/current/theme.name"),
        "fixture-theme-name\n",
    )
    .unwrap();
    fs::write(
        root.join(".local/state/omarchy/current/theme/colors.toml"),
        "foreground = \"#fixture-color\"\n",
    )
    .unwrap();
    fs::write(
        root.join(".config/omarchy/shell.toml"),
        "fixture-user-shell = true\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("etc")).unwrap();
    fs::write(root.join("etc/hostname"), "fixture-hostname\n").unwrap();
    fs::write(
        root.join("etc/passwd"),
        "fixtureuser:x:1000:1000:Fixture User:/home/fixture:/bin/bash\n",
    )
    .unwrap();
    fs::write(root.join("etc/group"), "fixturegroup:x:1000:fixtureuser\n").unwrap();
    fs::create_dir_all(root.join("home/fixture")).unwrap();
    fs::write(root.join("home/fixture/.face.icon"), "icon").unwrap();
    fs::create_dir_all(root.join(".local/state/omarchy/speed")).unwrap();
    fs::write(
        root.join(".local/state/omarchy/speed/disk.txt"),
        "fixture-disk 120 MB/s\n",
    )
    .unwrap();
    fs::write(
        root.join(".local/state/omarchy/speed/net-down.txt"),
        "fixture-down 80 Mbps\n",
    )
    .unwrap();
    fs::create_dir_all(root.join(".local/state/omarchy/units")).unwrap();
    fs::write(
        root.join(".local/state/omarchy/units/status-fixture.service.txt"),
        "fixture-unit-status active\n",
    )
    .unwrap();
    fs::create_dir_all(root.join(".config/omarchy/themes/fixture-theme")).unwrap();
    fs::write(
        root.join(".config/omarchy/themes/fixture-theme/colors.toml"),
        "accent = \"#fixture-pack\"\n",
    )
    .unwrap();

    let chrome = request(&root, "omarchy", &serde_json::json!({"op": "host.chrome"}));
    let chrome_text = chrome["result"].to_string();
    assert!(chrome_text.contains("fixture-theme-name"), "{chrome}");
    assert!(chrome_text.contains("#fixture-color"), "{chrome}");
    assert!(chrome_text.contains("fixture-user-shell"), "{chrome}");
    println!("ok chrome {chrome_text}");

    let accounts = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "host.accounts", "user": "fixture", "home": "/home/fixture"}),
    );
    let accounts_text = accounts["result"].to_string();
    assert!(accounts_text.contains("fixture-hostname"), "{accounts}");
    assert!(accounts_text.contains("Fixture User"), "{accounts}");
    assert_eq!(
        accounts["result"]["exists"]["/home/fixture/.face.icon"], true,
        "{accounts}"
    );
    println!("ok accounts {accounts_text}");

    let pack = request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "host.themePack",
            "name": "Fixture Theme",
            "home": ""
        }),
    );
    // home empty still searches /usr/share, which the fixture roots at
    // root/usr/share. The planted file is under .config, so pass home as the
    // fixture home path the UI would send. Re-request with that home.
    let _ = pack;
    let pack = request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "host.themePack",
            "name": "Fixture Theme",
            "home": "/home/fixture"
        }),
    );
    // Theme pack looks in {home}/.config/omarchy/themes. Plant it there too.
    let _ = pack;
    fs::create_dir_all(root.join("home/fixture/.config/omarchy/themes/fixture-theme")).unwrap();
    fs::write(
        root.join("home/fixture/.config/omarchy/themes/fixture-theme/colors.toml"),
        "accent = \"#fixture-pack\"\n",
    )
    .unwrap();
    let pack = request(
        &root,
        "omarchy",
        &serde_json::json!({
            "op": "host.themePack",
            "name": "Fixture Theme",
            "home": "/home/fixture"
        }),
    );
    assert!(
        pack["result"]["colors"]
            .as_str()
            .unwrap_or("")
            .contains("#fixture-pack"),
        "{pack}"
    );
    println!("ok themePack {}", pack["result"]);

    let disk = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "speedtest.disk", "dir": ""}),
    );
    assert!(
        disk["result"]["stdout"]
            .as_str()
            .unwrap_or("")
            .contains("fixture-disk"),
        "{disk}"
    );
    println!("ok disk {}", disk["result"]["stdout"]);

    let net = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "speedtest.net", "phase": "down"}),
    );
    assert!(
        net["result"]["stdout"]
            .as_str()
            .unwrap_or("")
            .contains("fixture-down"),
        "{net}"
    );
    println!("ok net {}", net["result"]["stdout"]);

    let unit = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "unit.output", "kind": "status", "scope": "system", "unit": "fixture.service"}),
    );
    assert!(
        unit["result"]["text"]
            .as_str()
            .unwrap_or("")
            .contains("fixture-unit-status"),
        "{unit}"
    );
    println!("ok unit {}", unit["result"]["text"]);

    let export_path = "/home/fixture/export.md";
    let written = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "host.write", "path": export_path, "text": "fixture-export-body\n"}),
    );
    assert_eq!(written["result"]["path"], export_path, "{written}");
    let file = fs::read_to_string(root.join("home/fixture/export.md")).unwrap();
    assert!(file.contains("fixture-export-body"), "{file}");
    let read = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "host.read", "paths": [export_path]}),
    );
    assert!(
        read["result"]["files"][0]["text"]
            .as_str()
            .unwrap_or("")
            .contains("fixture-export-body"),
        "{read}"
    );
    println!("ok export {}", read["result"]["files"][0]["text"]);

    let opened = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "host.open", "path": export_path}),
    );
    assert_eq!(opened["result"]["opened"], export_path, "{opened}");
    println!("ok open {}", opened["result"]["opened"]);

    let plain = request(&root, "plain", &serde_json::json!({"op": "host.chrome"}));
    assert_eq!(plain["platform"]["id"], "plain", "{plain}");
    assert_ne!(
        plain["result"]["themeName"], chrome["result"]["themeName"],
        "{plain}"
    );
    println!(
        "ok plain chrome {} vs {}",
        plain["result"]["themeName"], chrome["result"]["themeName"]
    );
    let plain_disk = request(
        &root,
        "plain",
        &serde_json::json!({"op": "speedtest.disk", "dir": ""}),
    );
    assert_eq!(plain_disk["platform"]["id"], "plain");
    assert_ne!(plain_disk["result"]["stdout"], disk["result"]["stdout"]);
    println!("ok plain disk {}", plain_disk["result"]["stdout"]);

    cleanup(&root);
}
