//! Sudo mode is read from the expiry timer the `sudo-on` task starts, since
//! `/etc/sudoers.d` is root-only (#101). The snapshot and `settings.get` read
//! it the same way, so a window and a terminal never disagree.

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
    /// A `systemctl` that answers `state` for ann's expiry timer and nothing
    /// for anything else the security producer asks it.
    fn new(name: &str, state: &str) -> Fakes {
        let dir = PathBuf::from(format!("/tmp/rt-sudo-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("bin")).unwrap();
        let path = dir.join("bin").join("systemctl");
        fs::write(
            &path,
            format!(
                r#"#!/bin/sh
case "$*" in
  "show -p SubState --value omarchy-nopasswd-expire-ann.timer") echo {state} ;;
esac
"#
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        Fakes { dir }
    }

    fn request(&self, user: &str, body: Value) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_atmosd"))
            .args(["--backend", "omarchy", "request"])
            .env_clear()
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.dir.join("bin").display()),
            )
            .env("HOME", &self.dir)
            .env("USER", user)
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
            .write_all(body.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        serde_json::from_slice(&out.stdout).expect("an envelope")
    }

    /// Sudo mode as the snapshot reads it and as `settings.get` reads it.
    fn both(&self, user: &str) -> (Value, Value) {
        let snapshot = self.request(
            user,
            json!({"op": "snapshot.keys", "keys": ["passwordlessSudo"]}),
        );
        let get = self.request(
            user,
            json!({"op": "settings.get", "domain": "passwordlessSudo"}),
        );
        assert_eq!(get["ok"], true, "{get}");
        (
            snapshot["result"]["values"]["passwordlessSudo"].clone(),
            get["result"].clone(),
        )
    }
}

impl Drop for Fakes {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn only_a_timer_still_waiting_is_sudo_mode_on() {
    // `running` is a timer that has fired while its `rm` runs, `elapsed` one
    // kept loaded after it fired, and `dead` one never armed or already gone.
    for (state, on) in [
        ("waiting", true),
        ("running", false),
        ("elapsed", false),
        ("dead", false),
    ] {
        let fakes = Fakes::new(state, state);
        let (snapshot, get) = fakes.both("ann");
        assert_eq!(snapshot, json!(on), "the snapshot, timer {state}");
        assert_eq!(get, json!(on), "settings.get, timer {state}");
    }
}

#[test]
fn another_users_timer_is_not_this_users_sudo_mode() {
    let fakes = Fakes::new("other-user", "waiting");
    let (snapshot, get) = fakes.both("bob");
    assert_eq!(snapshot, json!(false));
    assert_eq!(get, json!(false));
}
