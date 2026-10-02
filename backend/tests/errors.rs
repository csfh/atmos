mod common;

use common::{cleanup, request_error, temp_root};

fn code(value: &serde_json::Value) -> &str {
    value["error"]["code"].as_str().unwrap_or("")
}

#[test]
fn every_failure_is_a_json_envelope_with_a_code() {
    let root = temp_root();

    let unknown = request_error(&root, "omarchy", r#"{"op":"nope"}"#);
    assert_eq!(code(&unknown), "bad_request");
    assert!(unknown["error"]["message"]
        .as_str()
        .unwrap()
        .contains("nope"));

    let broken = request_error(&root, "omarchy", "{not json");
    assert_eq!(code(&broken), "bad_request");

    let missing = request_error(&root, "omarchy", r#"{"op":"settings.get"}"#);
    assert_eq!(code(&missing), "bad_request");
    assert!(missing["error"]["message"]
        .as_str()
        .unwrap()
        .contains("domain"));

    let wrong_type = request_error(
        &root,
        "omarchy",
        r#"{"op":"settings.set","domain":"theme","value":3}"#,
    );
    assert_eq!(code(&wrong_type), "failed");

    let no_domain = request_error(&root, "omarchy", r#"{"op":"settings.get","domain":"zzz"}"#);
    assert!(no_domain["error"]["message"]
        .as_str()
        .unwrap()
        .contains("zzz"));

    cleanup(&root);
}

#[test]
fn host_paths_stay_inside_the_root() {
    let root = temp_root();

    let escape = request_error(
        &root,
        "omarchy",
        r#"{"op":"host.write","path":"../outside.md","text":"x"}"#,
    );
    assert_eq!(code(&escape), "denied");
    assert!(!root.parent().unwrap().join("outside.md").exists());

    let read = request_error(
        &root,
        "omarchy",
        r#"{"op":"host.read","paths":["notes/../../etc/passwd"]}"#,
    );
    assert_eq!(code(&read), "denied");

    let open = request_error(
        &root,
        "omarchy",
        r#"{"op":"host.open","path":"missing.md"}"#,
    );
    assert_eq!(code(&open), "not_found");

    cleanup(&root);
}

/// A bindings list too long for one argv still lands, because the payload
/// travels on stdin to hypr-sentinel.py.
#[test]
fn a_huge_hypr_payload_does_not_hit_the_argument_limit() {
    use common::request;
    let root = temp_root();
    // Linux caps one argv string near 128 KiB; this is several times that.
    let rows: Vec<_> = (0..6000)
        .map(|i| {
            serde_json::json!({
                "keys": format!("SUPER+F{i}"),
                "description": "padding padding padding padding padding",
                "command": "true",
            })
        })
        .collect();
    assert!(serde_json::to_string(&rows).unwrap().len() > 300_000);
    request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.set", "domain": "bindings", "value": rows}),
    );
    let got = request(
        &root,
        "omarchy",
        &serde_json::json!({"op": "settings.get", "domain": "bindings"}),
    );
    assert_eq!(got["result"].as_array().unwrap().len(), 6000);
    cleanup(&root);
}

#[test]
fn apply_streams_the_command_and_ends_with_a_summary_line() {
    let output = common::invoke(
        &[
            "apply",
            "--",
            "sh",
            "-c",
            "echo out; echo 'warn: wayland.x' >&2; echo broke >&2; exit 3",
        ],
        "",
    );
    assert_eq!(
        output.status.code(),
        Some(3),
        "the exit code is the command's"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some("out"), "stdout passes through unchanged");
    let last = lines.next().expect("a summary line");
    let json = last.strip_prefix("@@ratmos-apply@@ ").expect("the marker");
    let summary: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(summary["ok"], false);
    assert_eq!(summary["code"], 3);
    assert_eq!(summary["message"], "broke");
    assert_eq!(summary["warnings"][0], "warn: wayland.x");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("broke") && stderr.contains("warn: wayland.x"));
}

#[test]
fn apply_reports_success_and_a_command_that_cannot_start() {
    let good = common::invoke(&["apply", "--", "true"], "");
    assert!(good.status.success());
    assert!(String::from_utf8_lossy(&good.stdout).contains("\"ok\":true"));

    let missing = common::invoke(&["apply", "--", "ratmos-no-such-command"], "");
    assert_eq!(missing.status.code(), Some(1));
    let text = String::from_utf8_lossy(&missing.stdout);
    assert!(
        text.contains("\"ok\":false") && text.contains("ratmos-no-such-command"),
        "{text}"
    );
}

#[test]
fn apply_passes_stdin_through_to_the_command() {
    let output = common::invoke(&["apply", "--", "cat"], "secret\n");
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("secret\n"));
}
