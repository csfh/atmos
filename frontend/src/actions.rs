//! What a window may ask the daemon to do to a process or to the clipboard, and what it checks
//! first. The daemon checks again (`backend/src/command/entries.rs`, `tasks/user.rs`): it refuses
//! pid 1, itself, its parent and what the user does not own. These checks are what the page can
//! tell a person before it asks (`services/Processes.js`, `services/RichUi.js`).

use serde_json::{Value, json};

use crate::live::Process;
use crate::protocol;

/// The least and the greatest pid `process.signal` takes.
pub const PID_RANGE: (u64, u64) = (2, 4_194_304);
/// The longest text `clipboard.copy` is sent, in UTF-16 units as the QML window counts them.
pub const CLIPBOARD_MAX: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// Asks the process to exit.
    Term,
    /// Ends it without a chance to clean up.
    Kill,
}

impl Signal {
    pub fn name(self) -> &'static str {
        match self {
            Signal::Term => "TERM",
            Signal::Kill => "KILL",
        }
    }
}

/// `canSignal`: a kernel thread is not a process one ends, and a process the backend says the
/// user does not own is not theirs to end. One whose owner it did not say is tried.
pub fn can_signal(p: &Process) -> bool {
    !p.kthread && p.mine != Some(false)
}

/// The `process.signal` write, or `None` for a pid the entry refuses.
pub fn signal_write(pid: u64, signal: Signal) -> Option<Value> {
    (PID_RANGE.0..=PID_RANGE.1).contains(&pid).then(|| {
        protocol::write(
            "process.signal",
            &json!({ "pid": pid, "signal": signal.name() }),
        )
    })
}

/// `clipboardPayload` with `singleLine` and a limit of [`CLIPBOARD_MAX`]: nothing for text that
/// is empty, holds a NUL or breaks a line, and the first 1024 UTF-16 units of the rest.
pub fn clipboard_payload(text: &str) -> Option<String> {
    if text.is_empty() || text.contains('\0') || text.contains(['\r', '\n']) {
        return None;
    }
    let mut units = 0;
    let mut kept = String::new();
    for c in text.chars() {
        units += c.len_utf16();
        if units > CLIPBOARD_MAX {
            break;
        }
        kept.push(c);
    }
    Some(kept)
}

/// The `clipboard.copy` write for `text`, or `None` for text that is not copied.
pub fn copy_write(text: &str) -> Option<Value> {
    clipboard_payload(text).map(|text| protocol::write("clipboard.copy", &json!({ "text": text })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(kthread: bool, mine: Option<bool>) -> Process {
        Process {
            pid: 10,
            comm: "p".into(),
            cmdline: String::new(),
            uid: None,
            rss_kb: None,
            ticks: None,
            start_ticks: None,
            cpu: None,
            kthread,
            mine,
        }
    }

    #[test]
    fn a_process_is_ended_only_if_it_is_a_process_and_not_known_to_be_anothers() {
        assert!(can_signal(&process(false, Some(true))));
        assert!(can_signal(&process(false, None)));
        assert!(!can_signal(&process(false, Some(false))));
        assert!(!can_signal(&process(true, Some(true))));
    }

    #[test]
    fn a_signal_is_a_write_of_a_pid_the_entry_takes() {
        let w = signal_write(4242, Signal::Kill).unwrap();
        assert_eq!(w["name"], "process.signal");
        assert_eq!(w["args"], json!({"pid": 4242, "signal": "KILL"}));
        assert_eq!(signal_write(1, Signal::Term), None, "never init");
        assert_eq!(signal_write(0, Signal::Term), None);
        assert!(signal_write(4_194_304, Signal::Term).is_some());
        assert_eq!(signal_write(4_194_305, Signal::Term), None);
    }

    #[test]
    fn text_that_cannot_be_copied_is_not_and_the_rest_is_cut_at_1024_units() {
        assert_eq!(clipboard_payload(""), None);
        assert_eq!(clipboard_payload("a\0b"), None);
        assert_eq!(clipboard_payload("one\ntwo"), None);
        assert_eq!(clipboard_payload("one\rtwo"), None);
        assert_eq!(
            clipboard_payload("/bin/x --y").as_deref(),
            Some("/bin/x --y")
        );
        let long = "x".repeat(2000);
        assert_eq!(clipboard_payload(&long).unwrap().len(), 1024);
        // A character outside the basic plane is two units, and is not split.
        let wide = "😀".repeat(600);
        assert_eq!(clipboard_payload(&wide).unwrap().chars().count(), 512);
        let w = copy_write("1234").unwrap();
        assert_eq!(w["args"], json!({"text": "1234"}));
        assert_eq!(copy_write("a\nb"), None);
    }
}
