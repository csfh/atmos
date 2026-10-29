//! The Capture page's rows the page module draws (`pages/capture.rs`): the choices it holds and
//! where they start, its texts, and the calls each control makes are the QML page's. What each
//! call then sends is `catalog_setters.rs`'s.

mod common;
#[path = "catalog/rows.rs"]
mod rows;

use atmos_ui::capture;
use atmos_ui::catalog;
use rows::page_src;
use serde_json::json;

#[test]
fn the_capture_rows_are_the_qml_rows() {
    let page = catalog::page("capture").unwrap();
    let src = page_src(page);
    let flat = src.split_whitespace().collect::<Vec<_>>().join(" ");
    let options = |list: &[(&str, &str)]| {
        list.iter()
            .map(|(v, l)| format!("{{ value: {}, label: {} }}", json!(v), json!(l)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut wants = vec![
        format!("readonly property var screenshotModes: [ {} ]", options(capture::MODES)),
        format!("readonly property var screenshotDests: [ {} ]", options(capture::DESTS)),
        format!("readonly property var webcamSizes: [ {} ]", options(capture::SIZES)),
        "property string shotMode: \"smart\" property string shotDest: \"slurp\"".to_owned(),
        "property string recWebcamSize: \"medium\"".to_owned(),
        "text: root.shotMode === \"fullscreen\" ? \"Capture\" : \"Capture…\" primary: true onClicked: Omarchy.captureScreenshot(root.shotMode, root.shotDest)".to_owned(),
        "available: root.recWebcam label: \"Webcam size\"".to_owned(),
        "label: Omarchy.recordingActive ? \"Recording\" : \"Start recording\"".to_owned(),
        format!("description: Omarchy.recordingActive ? {} : {}", json!(capture::recording_says(true)), json!(capture::recording_says(false))),
        "visible: !Omarchy.recordingActive text: root.recFullscreen ? \"Start\" : \"Start…\"".to_owned(),
        "onClicked: Omarchy.startScreenrecording(root.recDesktopAudio, root.recMic, root.recWebcam, root.recWebcamSize, root.recFullscreen)".to_owned(),
        "visible: Omarchy.recordingActive text: \"Stop\"".to_owned(),
        "onClicked: Omarchy.stopScreenrecording()".to_owned(),
        "available: Omarchy.webcamOverlay label: \"Resize webcam\"".to_owned(),
    ];
    for (key, on) in [
        capture::DESKTOP_AUDIO,
        capture::MICROPHONE,
        capture::WEBCAM,
        capture::FULLSCREEN,
    ] {
        wants.push(format!("property bool {key}: {on}"));
        wants.push(format!("checked: root.{key} enabled: !Omarchy.recordingActive onToggled: root.{key} = !root.{key}"));
    }
    for (label, action) in capture::RESIZE {
        wants.push(format!(
            "text: {} enabled: Omarchy.webcamOverlay onClicked: Omarchy.resizeWebcam({})",
            json!(label),
            json!(action)
        ));
    }
    for (dir, which) in [("picturesDir", "Pictures"), ("videosDir", "Videos")] {
        wants.push(format!(
            "description: Omarchy.{dir}.length ? Omarchy.{dir} : {}",
            json!(capture::folder_says("", which))
        ));
        wants.push(format!("text: \"Open folder\" enabled: Omarchy.{dir}.length > 0 onClicked: Omarchy.openUserDir(Omarchy.{dir})"));
        wants.push(format!("text: \"Copy\" enabled: Omarchy.{dir}.length > 0 onClicked: Omarchy.copyText(Omarchy.{dir})"));
    }
    for want in wants {
        assert!(flat.contains(&want), "{want}");
    }
}
