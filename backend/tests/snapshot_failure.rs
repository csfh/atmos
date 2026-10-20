//! A tool the snapshot cannot do without failing is a missing key and a named
//! reason, never a default: the window keeps what it showed (docs/adr/0006).

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::{json, Value};

struct Fakes {
    dir: PathBuf,
}

impl Fakes {
    fn new(name: &str) -> Fakes {
        let dir = PathBuf::from(format!("/tmp/rt-snapfail-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("bin")).unwrap();
        Fakes { dir }
    }

    fn tool(&self, name: &str, body: &str) {
        let path = self.dir.join("bin").join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn keys(&self, keys: &[&str]) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
            .args(["--backend", "omarchy", "request"])
            .env_clear()
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.dir.join("bin").display()),
            )
            .env("HOME", &self.dir)
            .env("XDG_RUNTIME_DIR", &self.dir)
            .env("ATMOS_NO_AUTOSTART", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                json!({"op": "snapshot.keys", "keys": keys})
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();
        let out = child.wait_with_output().unwrap();
        serde_json::from_slice(&out.stdout).expect("an envelope")
    }
}

impl Drop for Fakes {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

const THEME: &str = r#"case "$*" in
  "theme current") echo tokyo-night ;;
  "theme bg current") echo /x.png ;;
  "font current") echo Mono ;;
  *list) printf 'a\nb\n' ;;
  "theme extras") echo /x/extra ;;
  "plymouth current") echo omarchy ;;
  *) echo ok ;;
esac"#;

#[test]
fn a_tool_that_answers_gives_its_keys() {
    let fakes = Fakes::new("healthy");
    fakes.tool("omarchy", THEME);
    let reply = fakes.keys(&["theme", "font"]);
    assert_eq!(reply["result"]["values"]["theme"], "tokyo-night", "{reply}");
    assert_eq!(reply["result"]["failed"], json!({}), "{reply}");
}

#[test]
fn a_tool_that_fails_leaves_its_keys_out_and_says_why() {
    let fakes = Fakes::new("failing");
    fakes.tool(
        "omarchy",
        &format!(
            r#"if [ "$*" = "theme current" ]; then echo 'no such theme state' >&2; exit 3; fi
{THEME}"#
        ),
    );
    let reply = fakes.keys(&["theme", "font", "hostname"]);
    let values = &reply["result"]["values"];
    assert!(
        values.get("theme").is_none(),
        "no value was made up: {reply}"
    );
    assert!(
        values.get("font").is_none(),
        "the producer's keys go together: {reply}"
    );
    let why = reply["result"]["failed"]["theme"]
        .as_str()
        .expect("a reason");
    assert!(
        why.contains("theme current") && why.contains("exit 3"),
        "{why}"
    );
    assert!(
        reply["result"]["failed"].get("hostname").is_none(),
        "{reply}"
    );
}

#[test]
fn a_machine_without_omarchy_fails_its_keys_instead_of_showing_empty_ones() {
    let fakes = Fakes::new("absent");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
        .args(["--backend", "omarchy", "request"])
        .env_clear()
        .env("PATH", fakes.dir.join("bin"))
        .env("HOME", &fakes.dir)
        .env("ATMOS_NO_AUTOSTART", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            json!({"op": "snapshot.keys", "keys": ["theme", "dns", "doNotDisturb"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let reply: Value = serde_json::from_slice(&out.stdout).unwrap();
    let failed = reply["result"]["failed"].as_object().unwrap();
    assert!(
        failed.contains_key("theme") && failed.contains_key("dns"),
        "{reply}"
    );
    assert!(reply["result"]["values"].get("theme").is_none(), "{reply}");
}
