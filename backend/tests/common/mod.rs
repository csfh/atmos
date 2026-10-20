use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

pub fn invoke(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ratmos"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ratmos");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().expect("wait ratmos")
}

/// A directory no other test in this process shares. The clock alone is not
/// enough: tests that start together can read the same nanosecond, and then one
/// test's cleanup deletes the other's files.
pub fn temp_root() -> PathBuf {
    static MADE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = MADE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("ratmos-{nanos}-{}-{n}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

#[allow(dead_code)]
pub fn request(root: &Path, backend: &str, body: &Value) -> Value {
    let output = invoke(
        &[
            "--backend",
            backend,
            "--root",
            &root.display().to_string(),
            "request",
        ],
        &serde_json::to_string(body).unwrap(),
    );
    assert!(
        output.status.success(),
        "request failed\nstdout {}\nstderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("response json");
    assert_eq!(value["ok"], Value::Bool(true), "{value}");
    assert_eq!(value["version"], "0.4.0", "{value}");
    value
}

pub fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

/// A request that is expected to fail. Returns the parsed error envelope.
#[allow(dead_code)]
pub fn request_error(root: &Path, backend: &str, stdin: &str) -> Value {
    let output = invoke(
        &[
            "--backend",
            backend,
            "--root",
            &root.display().to_string(),
            "request",
        ],
        stdin,
    );
    assert!(!output.status.success(), "expected a failing exit");
    let value: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout must be an envelope ({err})\nstdout {}\nstderr {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert_eq!(value["ok"], Value::Bool(false), "{value}");
    value
}
