//! One effect row per settings domain.
//!
//! Omarchy `settings.set` dispatches through this table. `Command` argv is the
//! Settings.js writer. Under `--root` that argv is appended to `commands.log`
//! and the platform file is left alone. A live set spawns the same argv.
//! Live reads of those keys stay null so `snapshot.sh` keeps the status it
//! already collected from `omarchy toggle nightlight --status` and pactl.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Patch the platform file this domain already uses.
    Document,
    /// Hyprland sentinel. Apply keeps rows whose `managed` flag is not false.
    Sentinel { kind: &'static str },
    /// Settings.js argv. Fixture logs it. Live spawns it.
    Command { argv: &'static [&'static str] },
}

const SENTINELS: &[(&str, &str)] = &[
    ("bindings", "bindings"),
    ("windowRules", "windows"),
    ("autostart", "autostart"),
    ("monitorRules", "monitors"),
    ("workspaces", "workspaces"),
    ("workspaceWrapSwitch", "workspaces"),
    ("workspaceWheelSwitch", "workspaces"),
];

const COMMANDS: &[(&str, &[&str])] = &[
    ("nightlight", &["omarchy", "toggle", "nightlight"]),
    (
        "audioOutputMuted",
        &["omarchy", "audio", "output", "volume", "mute-toggle"],
    ),
    ("audioInputMuted", &["omarchy", "audio", "input", "mute"]),
];

const DOCUMENTS: &[&str] = &[
    "theme",
    "background",
    "font",
    "textSize",
    "plymouth",
    "hyprLook.cursorSize",
    "hyprLook.gapsIn",
    "hyprLook.gapsOut",
    "hyprLook.rounding",
    "hyprLook.borderSize",
    "hyprLook.activeOpacity",
    "hyprLook.inactiveOpacity",
    "hyprLook.blur",
    "hyprLook.shadow",
    "hyprLook.dimInactive",
    "hyprLook.dimStrength",
    "hyprLook.animations",
    "hyprLook.columnWidth",
    "hyprLook.cursorHideOnKey",
    "hyprLook.cursorWarp",
    "hyprLook.resizeOnBorder",
    "hyprLook.allowTearing",
    "hyprLook.layout",
    "hyprLook.preserveSplit",
    "hyprLook.enableSwallow",
    "hyprLook.swallowRegex",
    "hyprLook.onFocusUnderFullscreen",
    "hyprLook.focusOnActivate",
    "hyprNoGaps",
    "hyprSquareAspect",
    "barPosition",
    "barTransparent",
    "barVisible",
    "clockFormat",
    "clockFormatAlt",
    "clockWeekStart",
    "clockBirthYear",
    "clockLifeExpectancy",
    "indicatorsAlwaysShow",
    "indicatorsItems",
    "powerShowPercentage",
    "spacerSize",
    "weatherLocation",
    "weatherUnit",
    "weatherRefreshMinutes",
    "agentsRefreshIntervalSec",
    "agentsSync",
    "agentsSyncDir",
    "agentsSyncFileName",
    "agentsSyncDeviceId",
    "trayHidden",
    "trayPinned",
    "idleScreensaver",
    "idleLock",
    "stayAwake",
    "screensaverEnabled",
    "doNotDisturb",
    "workspaceBarNames",
    "workspaceBarCount",
    "browser",
    "terminal",
    "editor",
    "agent",
    "mimePdf",
    "mimeImage",
    "mimeVideo",
    "nightlightTemperature",
    "nightlightDay",
    "nightlightNight",
    "nightlightNightOn",
    "hyprInput.sensitivity",
    "hyprInput.accelProfile",
    "hyprInput.emulateDiscreteScroll",
    "hyprInput.naturalScroll",
    "hyprInput.scrollFactor",
    "hyprInput.clickfinger",
    "hyprInput.disableWhileTyping",
    "hyprInput.drag3fg",
    "hyprInput.repeatRate",
    "hyprInput.repeatDelay",
    "hyprInput.numlock",
    "hyprInput.followMouse",
    "hyprInput.keyPressDpms",
    "hyprInput.mouseMoveDpms",
    "hyprInput.kbLayoutOverride",
    "hyprInput.kbVariantOverride",
    "hyprInput.kbGroupToggle",
    "hyprInput.workspaceGesture",
    "touchpadEnabled",
    "touchscreenEnabled",
    "hostname",
    "timezone",
    "locale",
    "keyboardLayout",
    "ntp",
    "fullName",
    "parallelDownloads",
    "dns",
    "customDns",
    "bluetooth",
    "wifiRadio",
    "wifiBand",
    "audioOutputVolume",
    "audioInputVolume",
    "audioTuningOn",
    "powerProfileAc",
    "powerProfileBattery",
    "powerProfile",
    "suspendEnabled",
    "crashCapture",
    "presentationMode",
    "chargeLimit",
    "monitorScale",
    "internalDisplay",
    "internalMirror",
    "displayBrightness",
    "envVars",
    "envPathPrepend",
    "tweaks.middlePaste",
    "tweaks.electronWayland",
    "tweaks.forceZeroScaling",
    "tweaks.swappiness",
    "sshdEnabled",
    "passwordlessSudo",
    "sudolessDocker",
    "fingerprintConfigured",
    "fido2Configured",
    "snapperNumberLimit",
    "snapperTimeline",
    "fstrimEnabled",
    "directBoot",
    "omarchyChannel",
    "atmosChannel",
    "favorites",
    "plugins",
    "avatarPath",
];

pub fn get(key: &str) -> Option<Effect> {
    if let Some((_, kind)) = SENTINELS.iter().copied().find(|(name, _)| *name == key) {
        return Some(Effect::Sentinel { kind });
    }
    if let Some((_, argv)) = COMMANDS.iter().copied().find(|(name, _)| *name == key) {
        return Some(Effect::Command { argv });
    }
    if DOCUMENTS.contains(&key) {
        return Some(Effect::Document);
    }
    None
}

pub fn row_keys() -> Vec<&'static str> {
    let mut keys = Vec::with_capacity(DOCUMENTS.len() + SENTINELS.len() + COMMANDS.len());
    keys.extend(DOCUMENTS.iter().copied());
    keys.extend(SENTINELS.iter().map(|(key, _)| *key));
    keys.extend(COMMANDS.iter().map(|(key, _)| *key));
    keys
}

pub fn apply_command(
    root: Option<&Path>,
    key: &str,
    argv: &[&str],
    value: &Value,
) -> Result<(), String> {
    if argv.is_empty() {
        return Err(format!("{key} command is empty"));
    }
    if let Some(dir) = root {
        return append_command_log(dir, key, argv, value);
    }
    let status = Command::new(argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| format!("{}: {err}", argv[0]))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited {}", argv[0], status.code().unwrap_or(1)))
    }
}

/// Fixture reads return the value recorded with the argv. Live reads stay null
/// so a file or this log cannot cover `snapshot.sh`.
pub fn read_command(root: Option<&Path>, key: &str) -> Result<Value, String> {
    let Some(dir) = root else {
        return Ok(Value::Null);
    };
    let path = dir.join("commands.log");
    if !path.is_file() {
        return Ok(Value::Null);
    }
    let text = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let mut found = Value::Null;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: Value = serde_json::from_str(trimmed).map_err(|err| err.to_string())?;
        if parsed.get("domain").and_then(Value::as_str) == Some(key) {
            found = parsed.get("value").cloned().unwrap_or(Value::Null);
        }
    }
    Ok(found)
}

fn append_command_log(dir: &Path, key: &str, argv: &[&str], value: &Value) -> Result<(), String> {
    let line = serde_json::json!({
        "domain": key,
        "argv": argv,
        "value": value,
    });
    let text = serde_json::to_string(&line).map_err(|err| err.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("commands.log"))
        .map_err(|err| err.to_string())?;
    writeln!(file, "{text}").map_err(|err| err.to_string())
}
