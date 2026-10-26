//! A tool a document cannot do without failing is an error, never an empty
//! document: the page keeps what it had and says why.

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
        let dir = PathBuf::from(format!("/tmp/rt-inv-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("bin")).unwrap();
        Fakes { dir }
    }

    fn tool(&self, name: &str, body: &str) {
        let path = self.dir.join("bin").join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn get(&self, kind: &str) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_atmosd"))
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
                json!({"op": "display.get", "kind": kind})
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

#[test]
fn a_failing_systemctl_is_an_error_not_a_machine_with_no_services() {
    let fakes = Fakes::new("systemctl");
    fakes.tool("systemctl", "echo 'Failed to connect to bus' >&2; exit 1");
    let reply = fakes.get("services");
    assert_eq!(reply["ok"], false, "{reply}");
    assert_eq!(reply["error"]["code"], "command");
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("systemctl"),
        "{reply}"
    );
    assert!(reply["result"].is_null());
}

#[test]
fn a_systemctl_that_works_is_a_document() {
    let fakes = Fakes::new("systemctl-ok");
    fakes.tool("systemctl", "exit 0");
    let reply = fakes.get("services");
    assert_eq!(reply["ok"], true, "{reply}");
}

#[test]
fn a_failing_lsblk_is_an_error_not_a_machine_with_no_disks() {
    let fakes = Fakes::new("lsblk");
    fakes.tool("lsblk", "echo 'lsblk: not permitted' >&2; exit 32");
    let reply = fakes.get("disks");
    assert_eq!(reply["ok"], false, "{reply}");
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("lsblk"),
        "{reply}"
    );
}

#[test]
fn an_lsblk_that_prints_nonsense_is_an_error_too() {
    let fakes = Fakes::new("lsblk-junk");
    fakes.tool("lsblk", "echo 'not json'");
    let reply = fakes.get("disks");
    assert_eq!(reply["ok"], false, "{reply}");
    assert!(
        reply["error"]["message"]
            .as_str()
            .unwrap()
            .contains("device list"),
        "{reply}"
    );
}

#[test]
fn a_working_lsblk_and_df_make_a_document() {
    let fakes = Fakes::new("disks-ok");
    fakes.tool("lsblk", r#"echo '{"blockdevices": []}'"#);
    fakes.tool(
        "df",
        "echo 'Filesystem 1-blocks Used Available Capacity Mounted on'",
    );
    let reply = fakes.get("disks");
    assert_eq!(reply["ok"], true, "{reply}");
    assert_eq!(reply["result"]["disks"], json!([]));
}
