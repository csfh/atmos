//! The small shell-like helpers the snapshot reads the machine with: run a
//! tool and take its text, test a tool's exit status, find a tool on PATH.

use crate::runner::Run;
use serde_json::Value;
use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(30);

// A failed tool is not an empty answer (docs/adr/0006). A reader that wants to
// know whether a tool failed wraps its work in `watching`; the helpers below
// say so when a tool they ran did not.
thread_local! {
    static FAULTS: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Tools whose absence is a failure and not a machine without the feature.
const REQUIRED: &[&str] = &["omarchy", "omarchy-shell", "hyprctl"];

/// Run `work` and say which tools it called failed: could not start, timed
/// out, or (for `must`) did not answer.
pub fn watching<T>(work: impl FnOnce() -> T) -> (T, Vec<String>) {
    FAULTS.with(|cell| *cell.borrow_mut() = Some(Vec::new()));
    let made = work();
    let faults = FAULTS
        .with(|cell| cell.borrow_mut().take())
        .unwrap_or_default();
    (made, faults)
}

/// Record a failure for the `watching` reader on this thread, if there is one.
pub fn fail(message: impl Into<String>) {
    let message = message.into();
    FAULTS.with(|cell| {
        if let Some(list) = cell.borrow_mut().as_mut() {
            if !list.contains(&message) {
                list.push(message);
            }
        }
    });
}

/// A tool that could not run. One that is not installed is only a failure if
/// the machine cannot work without it.
fn did_not_run(program: &str, err: &crate::error::Error) {
    if err.kind != crate::error::Kind::NotFound || REQUIRED.contains(&program) {
        fail(format!("{program}: {}", err.message));
    }
}

/// The value is the tool's answer: it has to run and exit 0, or the reader is
/// told it failed and gets nothing.
pub fn must(program: &str, args: &[&str]) -> Option<String> {
    must_within(program, args, TIMEOUT)
}

pub fn must_within(program: &str, args: &[&str], limit: Duration) -> Option<String> {
    let shown = || format!("{program} {}", args.join(" "));
    match Run::new(program).args(args).timeout(limit).output() {
        Ok(o) if o.success() => Some(o.stdout_text().trim_end_matches('\n').to_string()),
        Ok(o) => {
            let why = o.stderr_text();
            fail(format!(
                "{}: exit {}{}",
                shown().trim_end(),
                o.code,
                why.lines()
                    .next()
                    .map(|line| format!(": {line}"))
                    .unwrap_or_default()
            ));
            None
        }
        Err(err) => {
            fail(format!("{}: {}", shown().trim_end(), err.message));
            None
        }
    }
}

/// `omarchy ARGS` as a value that must be there.
pub fn omarchy_must(args: &[&str]) -> Option<String> {
    must("omarchy", args)
}

/// A tool's whole result, for callers that must tell a failure from empty output.
pub fn capture(program: &str, args: &[&str]) -> crate::error::Result<crate::runner::Output> {
    Run::new(program).args(args).timeout(TIMEOUT).output()
}

/// `$(cmd 2>/dev/null || true)`: stdout whatever the exit code, trailing newlines gone.
pub fn out(program: &str, args: &[&str]) -> String {
    match Run::new(program).args(args).timeout(TIMEOUT).output() {
        Ok(o) => o.stdout_text().trim_end_matches('\n').to_string(),
        Err(err) => {
            did_not_run(program, &err);
            String::new()
        }
    }
}

/// Like `out` with a tighter limit, for tools that can hang (`timeout 2s ...`).
pub fn out_within(program: &str, args: &[&str], secs: u64) -> String {
    match Run::new(program)
        .args(args)
        .timeout(Duration::from_secs(secs))
        .output()
    {
        Ok(o) => o.stdout_text().trim_end_matches('\n').to_string(),
        Err(err) => {
            did_not_run(program, &err);
            String::new()
        }
    }
}

/// `$(cmd 2>/dev/null)` that only counts when the command exits 0.
pub fn out_ok(program: &str, args: &[&str]) -> Option<String> {
    let o = match Run::new(program).args(args).timeout(TIMEOUT).output() {
        Ok(o) => o,
        Err(err) => {
            did_not_run(program, &err);
            return None;
        }
    };
    o.success()
        .then(|| o.stdout_text().trim_end_matches('\n').to_string())
}

/// Whether the tool exits 0. A tool that exits otherwise is an honest "no";
/// one that cannot run is a failure.
pub fn ok(program: &str, args: &[&str]) -> bool {
    match Run::new(program).args(args).timeout(TIMEOUT).output() {
        Ok(o) => o.success(),
        Err(err) => {
            did_not_run(program, &err);
            false
        }
    }
}

pub fn omarchy(args: &[&str]) -> String {
    out("omarchy", args)
}

/// `command -v name`
pub fn present(name: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if name.contains('/') {
        return Path::new(name).is_file();
    }
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| {
            let path = Path::new(if dir.is_empty() { "." } else { dir }).join(name);
            path.metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

pub fn lines(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| Value::String(l.into()))
        .collect()
}

pub fn file_has(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path).is_ok_and(|t| t.lines().any(|l| l.contains(needle)))
}

pub fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// First line of a file with the trailing newline cut, or "".
pub fn first_line(path: &Path) -> String {
    read(path).lines().next().unwrap_or("").to_string()
}

/// `^[0-9]+$`
pub fn is_uint(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit())
}

/// `[[ $x =~ ^[a-zA-Z0-9._-]+$ ]]`
pub fn is_ident(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

/// What `jq -r` prints for an unsigned integer, or None when the value is not one.
pub fn jq_uint(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(n) => {
            let f = n.as_f64()?;
            (f >= 0.0 && f.fract() == 0.0 && f < 1.8e19).then_some(f as u64)
        }
        Value::String(s) => is_uint(s).then(|| s.parse().ok()).flatten(),
        _ => None,
    }
}

/// jq's `x // default`: null and false fall through.
pub fn alt(value: Option<&Value>) -> Option<&Value> {
    value.filter(|v| !v.is_null() && **v != Value::Bool(false))
}

/// What `jq -r` prints for a scalar.
pub fn jq_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tool_that_exits_nonzero_is_a_failure_only_when_its_answer_is_the_value() {
        let ((answer, said, no), faults) = watching(|| {
            (
                must("sh", &["-c", "echo hi"]),
                must("sh", &["-c", "echo bad >&2; exit 3"]),
                ok("sh", &["-c", "exit 1"]),
            )
        });
        assert_eq!(answer.as_deref(), Some("hi"));
        assert_eq!(said, None);
        assert!(!no, "an honest no is not a failure");
        assert_eq!(faults.len(), 1, "{faults:?}");
        assert!(
            faults[0].contains("exit 3") && faults[0].contains("bad"),
            "{faults:?}"
        );
    }

    #[test]
    fn a_missing_tool_is_a_failure_only_if_the_machine_needs_it() {
        let (_, faults) = watching(|| {
            out("ratmos-no-such-tool", &[]);
            ok("ratmos-no-such-tool", &[]);
        });
        assert!(faults.is_empty(), "{faults:?}");
        let (_, faults) = watching(|| must("ratmos-no-such-tool", &[]));
        assert_eq!(faults.len(), 1, "{faults:?}");
        let (_, faults) = watching(|| out("omarchy-shell-not-here-hyprctl", &[]));
        assert!(faults.is_empty());
    }

    #[test]
    fn failures_are_only_kept_for_a_reader_that_asked() {
        fail("nobody is listening");
        let (_, faults) = watching(|| fail("one"));
        assert_eq!(faults, ["one"]);
        let (_, again) = watching(|| ());
        assert!(again.is_empty(), "the list does not carry over");
    }
}
