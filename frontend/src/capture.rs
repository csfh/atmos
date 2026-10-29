//! The Capture page's logic (`pages/CapturePage.qml`): the choices its screenshot and recording
//! rows hold for the page alone, and what Capture, Start, Stop, the webcam's buttons and the
//! save folders send, as `Omarchy.captureScreenshot`, `startScreenrecording`,
//! `stopScreenrecording`, `resizeWebcam`, `openUserDir` and `copyText` make them.
//! `tests/catalog_setters.rs` runs each of those against this.

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::protocol;

pub const RECORDING: Field = Field::Key("recordingActive");
pub const OVERLAY: Field = Field::Key("webcamOverlay");

pub const MODES: &[(&str, &str)] = &[
    ("smart", "Smart"),
    ("region", "Region"),
    ("windows", "Window"),
    ("fullscreen", "Fullscreen"),
];
pub const DESTS: &[(&str, &str)] = &[
    ("slurp", "Copy and save"),
    ("copy", "Copy"),
    ("save", "Save"),
];
pub const SIZES: &[(&str, &str)] = &[("small", "Small"), ("medium", "Medium"), ("large", "Large")];

/// The recording switches: the page's name for each, and where it starts.
pub const DESKTOP_AUDIO: (&str, bool) = ("recDesktopAudio", true);
pub const MICROPHONE: (&str, bool) = ("recMic", false);
pub const WEBCAM: (&str, bool) = ("recWebcam", false);
pub const FULLSCREEN: (&str, bool) = ("recFullscreen", false);

/// The webcam overlay's buttons, and what each asks for.
pub const RESIZE: &[(&str, &str)] = &[
    ("Smaller", "smaller"),
    ("Larger", "larger"),
    ("Reset", "reset"),
];

fn one_of(value: &str, list: &[(&str, &str)]) -> bool {
    list.iter().any(|(v, _)| *v == value)
}

/// `captureScreenshot(mode, dest)`: the job's arguments, none for a mode or place it does not
/// take.
pub fn screenshot(mode: &str, dest: &str) -> Option<Value> {
    (one_of(mode, MODES) && one_of(dest, DESTS)).then(|| json!({ "mode": mode, "dest": dest }))
}

/// Capture, or Capture… when a picker opens.
pub fn capture_label(mode: &str) -> &'static str {
    if mode == "fullscreen" {
        "Capture"
    } else {
        "Capture…"
    }
}

/// `startScreenrecording(...)`: the job's arguments, with the webcam's size only with the webcam.
pub fn recording(desktop: bool, mic: bool, webcam: bool, size: &str, fullscreen: bool) -> Value {
    let size = if one_of(size, SIZES) { size } else { "medium" };
    let mut args = json!({
        "fullscreen": fullscreen, "desktopAudio": desktop, "microphone": mic, "webcam": webcam,
    });
    if webcam {
        args["webcamSize"] = Value::from(size);
    }
    args
}

/// The recording row: its label and what it says, by whether a recording runs.
pub fn recording_label(on: bool) -> &'static str {
    if on { "Recording" } else { "Start recording" }
}

pub fn recording_says(on: bool) -> &'static str {
    if on {
        "gpu-screen-recorder is running. Stop writes the file under Videos."
    } else {
        "Start a recording with the switches above."
    }
}

/// Start, or Start… when a picker opens.
pub fn start_label(fullscreen: bool) -> &'static str {
    if fullscreen { "Start" } else { "Start…" }
}

/// `stopScreenrecording()` once the recorder runs.
pub fn stop() -> Value {
    protocol::write("recording.stop", &json!({}))
}

/// `resizeWebcam(action)`.
pub fn resize(action: &str) -> Option<Value> {
    let ok = matches!(
        action,
        "smaller" | "larger" | "reset" | "small" | "medium" | "large"
    );
    ok.then(|| protocol::write("webcam.resize", &json!({ "action": action })))
}

/// `openUserDir(dir)`: an absolute path of one line, with no `..`.
pub fn open_dir(dir: &str) -> Option<Value> {
    let ok = !dir.is_empty()
        && dir.encode_utf16().count() <= 1024
        && dir.starts_with('/')
        && !dir.contains("..")
        && !dir.contains(['\r', '\n', '\0']);
    ok.then(|| protocol::write("open.path", &json!({ "path": dir })))
}

/// A save folder's description: the folder, or why its buttons are off.
pub fn folder_says(dir: &str, which: &str) -> String {
    if dir.is_empty() {
        format!(
            "XDG {which} is not set. Open folder and Copy stay disabled until that folder exists."
        )
    } else {
        dir.to_owned()
    }
}
