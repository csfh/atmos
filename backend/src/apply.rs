//! `ratmos apply -- CMD ARGS...`: run a host command with its stdin and stdout
//! left connected, so a job can stream lines and read a password.
//!
//! stderr is copied through as it arrives, and also read here. When the
//! command ends, one more line goes to stdout:
//!
//! `@@ratmos-apply@@ {"ok":false,"code":3,"message":"...","warnings":[...]}`
//!
//! `ok` is the exit code being zero. `message` is the stderr that is not
//! known noise, so the frontend shows a reason without guessing which lines
//! matter. `warnings` is the noise that was filtered out. The exit code is the
//! command's own.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use serde_json::json;

use crate::error::Result;

pub const MARKER: &str = "@@ratmos-apply@@ ";

/// Lines that programs print on a healthy run. They are not failures.
const NOISE: &[&str] = &[
    "warn: wayland.",
    "warn: terminal.",
    "xdg-toplevel-icon",
    "slave exited with signal",
];

const MESSAGE_CAP: usize = 4000;

pub fn is_noise(line: &str) -> bool {
    NOISE.iter().any(|pattern| {
        if pattern.ends_with('.') {
            line.starts_with(pattern)
        } else {
            line.contains(pattern)
        }
    })
}

/// Split stderr lines into the failure text and the noise.
pub fn classify(lines: &[String]) -> (String, Vec<String>) {
    let mut message: Vec<&str> = Vec::new();
    let mut warnings = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_noise(trimmed) {
            warnings.push(trimmed.to_string());
        } else {
            message.push(trimmed);
        }
    }
    let mut text = message.join("\n");
    if text.len() > MESSAGE_CAP {
        let mut cut = MESSAGE_CAP;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
    }
    (text, warnings)
}

pub fn summary(code: i32, stderr_lines: &[String]) -> String {
    let (message, warnings) = classify(stderr_lines);
    let value = json!({
        "ok": code == 0,
        "code": code,
        "message": message,
        "warnings": warnings,
    });
    format!("{MARKER}{value}")
}

pub fn run(
    root: Option<&Path>,
    argv: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let Some(program) = argv.first() else {
        let _ = writeln!(stderr, "ratmos: apply needs a command");
        return 2;
    };
    if let Some(dir) = root {
        let code = match log_command(dir, argv) {
            Ok(()) => 0,
            Err(err) => {
                let _ = writeln!(stderr, "ratmos: {err}");
                1
            }
        };
        finish(stdout, code, &[]);
        return code;
    }
    let mut child = match Command::new(program)
        .args(&argv[1..])
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            let line = format!("{program}: {err}");
            let _ = writeln!(stderr, "ratmos: {line}");
            finish(stdout, 1, &[line]);
            return 1;
        }
    };
    let pipe = child.stderr.take();
    let reader = thread::spawn(move || {
        let mut seen = Vec::new();
        if let Some(pipe) = pipe {
            let mut out = std::io::stderr();
            for line in BufReader::new(pipe).lines().map_while(|l| l.ok()) {
                let _ = writeln!(out, "{line}");
                seen.push(line);
            }
        }
        seen
    });
    let code = match child.wait() {
        Ok(status) => status.code().unwrap_or(1),
        Err(_) => 1,
    };
    let seen = reader.join().unwrap_or_default();
    finish(stdout, code, &seen);
    code
}

fn finish(stdout: &mut dyn Write, code: i32, lines: &[String]) {
    let _ = writeln!(stdout, "{}", summary(code, lines));
    let _ = stdout.flush();
}

fn log_command(dir: &Path, argv: &[String]) -> Result<()> {
    let path = dir.join("commands.log");
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::json!({ "argv": argv });
    writeln!(file, "{line}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn known_noise_is_a_warning_not_a_message() {
        let (message, warnings) = classify(&lines(&[
            "warn: wayland.foo is odd",
            "xdg-toplevel-icon missing",
            "slave exited with signal 15",
            "warn: terminal.bar",
        ]));
        assert_eq!(message, "");
        assert_eq!(warnings.len(), 4);
    }

    #[test]
    fn anything_else_is_the_message() {
        let (message, warnings) =
            classify(&lines(&["warn: wayland.x", "", "disk full", "  oh no  "]));
        assert_eq!(message, "disk full\noh no");
        assert_eq!(warnings, vec!["warn: wayland.x".to_string()]);
    }

    #[test]
    fn a_prefix_pattern_does_not_match_mid_line() {
        assert!(!is_noise("error: warn: wayland. later"));
        assert!(is_noise("xdg-toplevel-icon: not supported"));
    }

    #[test]
    fn a_long_message_is_cut_on_a_char_boundary() {
        let long = "é".repeat(MESSAGE_CAP);
        let (message, _) = classify(&[long]);
        assert!(message.len() <= MESSAGE_CAP);
        assert!(message.chars().all(|c| c == 'é'));
    }

    #[test]
    fn the_summary_line_is_a_marker_and_json() {
        let line = summary(3, &lines(&["bad"]));
        let json = line.strip_prefix(MARKER).unwrap();
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["code"], 3);
        assert_eq!(value["message"], "bad");
        let ok: serde_json::Value =
            serde_json::from_str(summary(0, &[]).strip_prefix(MARKER).unwrap()).unwrap();
        assert_eq!(ok["ok"], true);
    }
}
