use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

pub fn invoke(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_atmos-backend"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn atmos-backend");
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().expect("wait atmos-backend")
}

pub fn temp_root() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("atmos-backend-{nanos}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

#[allow(dead_code)]
pub fn request(root: &Path, backend: &str, body: &Value) -> Value {
    let output = invoke(
        &["--backend", backend, "--root", &root.display().to_string(), "request"],
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
    assert_eq!(value["version"], "0.1.0", "{value}");
    value
}

pub fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}
