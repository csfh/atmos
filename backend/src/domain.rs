//! Settings domains the Quickshell GUI can change, and the platform file each one uses.

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    String,
    Int,
    Number,
    Bool,
    List,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::String => "string",
            Ty::Int => "int",
            Ty::Number => "number",
            Ty::Bool => "bool",
            Ty::List => "list",
        }
    }

    pub fn accepts(self, value: &Value) -> bool {
        match self {
            Ty::String => value.is_string(),
            Ty::Int => value.as_i64().is_some(),
            Ty::Number => value.is_number(),
            Ty::Bool => value.is_boolean(),
            Ty::List => value.is_array(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub key: &'static str,
    pub group: &'static str,
    pub ty: Ty,
}

#[derive(Clone, Debug)]
pub struct Place {
    pub rel: String,
    pub kind: PlaceKind,
}

#[derive(Clone, Debug)]
pub enum PlaceKind {
    Map,
    /// Nested `shell.json` fields and bar widgets the shell already reads.
    Shell,
    /// One nested field inside an existing JSON object, leaving siblings in place.
    Nested {
        path: &'static [&'static str],
    },
    /// Hyprland sentinel whose body is `hl.config` / assignment Lua, not a private JSON line.
    Lua {
        begin: &'static str,
        end: &'static str,
    },
    Line {
        prefix: &'static str,
    },
    /// A file whose presence is the toggle. `true` writes the file; `false` removes it.
    Flag,
    /// Hyprland block written by `hypr-sentinel.py` (`o.bind`, `o.window`, and the rest).
    Hypr {
        kind: &'static str,
    },
    /// A document the existing shell script already writes (env, sunset, mime, audio, …).
    Doc,
    Items,
    Whole,
}

impl Place {
    pub fn encoding(&self) -> &'static str {
        match self.kind {
            PlaceKind::Map => "map",
            PlaceKind::Shell => "shell",
            PlaceKind::Nested { .. } => "nested",
            PlaceKind::Lua { .. } => "lua",
            PlaceKind::Line { .. } => "line",
            PlaceKind::Flag => "flag",
            PlaceKind::Hypr { .. } => "hypr",
            PlaceKind::Doc => "doc",
            PlaceKind::Items => "items",
            PlaceKind::Whole => "whole",
        }
    }

    pub fn prefix(&self) -> &str {
        match &self.kind {
            PlaceKind::Line { prefix } => prefix,
            _ => "",
        }
    }
}

macro_rules! specs {
    ($(($key:literal, $group:literal, $ty:ident)),* $(,)?) => {
        [ $(Spec { key: $key, group: $group, ty: Ty::$ty }),* ]
    };
}

const SPECS: &[Spec] = &specs![
    ("theme", "theme", String),
    ("background", "theme", String),
    ("font", "theme", String),
    ("textSize", "theme", Int),
    ("plymouth", "theme", String),
    ("hyprLook.cursorSize", "look", Int),
    ("hyprLook.gapsIn", "look", Int),
    ("hyprLook.gapsOut", "look", Int),
    ("hyprLook.rounding", "look", Int),
    ("hyprLook.borderSize", "look", Int),
    ("hyprLook.activeOpacity", "look", Number),
    ("hyprLook.inactiveOpacity", "look", Number),
    ("hyprLook.blur", "look", Bool),
    ("hyprLook.shadow", "look", Bool),
    ("hyprLook.dimInactive", "look", Bool),
    ("hyprLook.dimStrength", "look", Number),
    ("hyprLook.animations", "look", Bool),
    ("hyprLook.columnWidth", "look", Number),
    ("hyprLook.cursorHideOnKey", "look", Bool),
    ("hyprLook.cursorWarp", "look", Bool),
    ("hyprLook.resizeOnBorder", "look", Bool),
    ("hyprLook.allowTearing", "look", Bool),
    ("hyprLook.layout", "look", String),
    ("hyprLook.preserveSplit", "look", Bool),
    ("hyprLook.enableSwallow", "look", Bool),
    ("hyprLook.swallowRegex", "look", String),
    ("hyprLook.onFocusUnderFullscreen", "look", Int),
    ("hyprLook.focusOnActivate", "look", Bool),
    ("hyprNoGaps", "look", Bool),
    ("hyprSquareAspect", "look", Bool),
    ("barPosition", "shell", String),
    ("barTransparent", "shell", Bool),
    ("barVisible", "shell", Bool),
    ("clockFormat", "shell", String),
    ("clockFormatAlt", "shell", String),
    ("clockWeekStart", "shell", String),
    ("clockBirthYear", "shell", Int),
    ("clockLifeExpectancy", "shell", Int),
    ("indicatorsAlwaysShow", "shell", Bool),
    ("indicatorsItems", "shell", List),
    ("powerShowPercentage", "shell", Bool),
    ("spacerSize", "shell", Int),
    ("weatherLocation", "shell", String),
    ("weatherUnit", "shell", String),
    ("weatherRefreshMinutes", "shell", Int),
    ("agentsRefreshIntervalSec", "shell", Int),
    ("agentsSync", "shell", Bool),
    ("agentsSyncDir", "shell", String),
    ("agentsSyncFileName", "shell", String),
    ("agentsSyncDeviceId", "shell", String),
    ("trayHidden", "shell", List),
    ("trayPinned", "shell", List),
    ("idleScreensaver", "shell", Int),
    ("idleLock", "shell", Int),
    ("stayAwake", "shell", Bool),
    ("screensaverEnabled", "shell", Bool),
    ("doNotDisturb", "shell", Bool),
    ("workspaceBarNames", "shell", Bool),
    ("workspaceBarCount", "shell", Int),
    ("browser", "defaults", String),
    ("terminal", "defaults", String),
    ("editor", "defaults", String),
    ("agent", "defaults", String),
    ("mimePdf", "defaults", String),
    ("mimeImage", "defaults", String),
    ("mimeVideo", "defaults", String),
    ("nightlight", "hyprsunset", Bool),
    ("nightlightTemperature", "hyprsunset", Int),
    ("nightlightDay", "hyprsunset", String),
    ("nightlightNight", "hyprsunset", String),
    ("nightlightNightOn", "hyprsunset", Bool),
    ("hyprInput.sensitivity", "input", Number),
    ("hyprInput.accelProfile", "input", String),
    ("hyprInput.emulateDiscreteScroll", "input", Int),
    ("hyprInput.naturalScroll", "input", Bool),
    ("hyprInput.scrollFactor", "input", Number),
    ("hyprInput.clickfinger", "input", Bool),
    ("hyprInput.disableWhileTyping", "input", Bool),
    ("hyprInput.drag3fg", "input", Int),
    ("hyprInput.repeatRate", "input", Int),
    ("hyprInput.repeatDelay", "input", Int),
    ("hyprInput.numlock", "input", Bool),
    ("hyprInput.followMouse", "input", Int),
    ("hyprInput.keyPressDpms", "input", Bool),
    ("hyprInput.mouseMoveDpms", "input", Bool),
    ("hyprInput.kbLayoutOverride", "input", String),
    ("hyprInput.kbVariantOverride", "input", String),
    ("hyprInput.kbGroupToggle", "input", Bool),
    ("hyprInput.workspaceGesture", "input", Bool),
    ("touchpadEnabled", "input", Bool),
    ("touchscreenEnabled", "input", Bool),
    ("hostname", "hostname", String),
    ("timezone", "timezone", String),
    ("locale", "locale", String),
    ("keyboardLayout", "keyboard", String),
    ("ntp", "ntp", Bool),
    ("fullName", "fullname", String),
    ("parallelDownloads", "downloads", Int),
    ("dns", "network", String),
    ("customDns", "network", String),
    ("bluetooth", "network", Bool),
    ("wifiRadio", "network", Bool),
    ("wifiBand", "wifiband", String),
    ("audioOutputVolume", "audio", Int),
    ("audioInputVolume", "audio", Int),
    ("audioOutputMuted", "audio", Bool),
    ("audioInputMuted", "audio", Bool),
    ("audioTuningOn", "audio", Bool),
    ("powerProfileAc", "power", String),
    ("powerProfileBattery", "power", String),
    ("powerProfile", "power", String),
    ("suspendEnabled", "power", Bool),
    ("crashCapture", "power", Bool),
    ("presentationMode", "power", Bool),
    ("chargeLimit", "power", Int),
    ("bindings", "bindings", List),
    ("windowRules", "windows", List),
    ("workspaces", "workspaces", List),
    ("workspaceWrapSwitch", "workspaces", Bool),
    ("workspaceWheelSwitch", "workspaces", Bool),
    ("monitorRules", "monitors", List),
    ("monitorScale", "monitors", Number),
    ("internalDisplay", "monitors", Bool),
    ("internalMirror", "monitors", Bool),
    ("displayBrightness", "backlight", Int),
    ("autostart", "autostart", List),
    ("envVars", "env", List),
    ("envPathPrepend", "env", String),
    ("tweaks.middlePaste", "tweaks", Bool),
    ("tweaks.electronWayland", "tweaks", Bool),
    ("tweaks.forceZeroScaling", "tweaks", Bool),
    ("tweaks.swappiness", "tweaks", Bool),
    ("sshdEnabled", "security", Bool),
    ("passwordlessSudo", "security", Bool),
    ("sudolessDocker", "security", Bool),
    ("fingerprintConfigured", "security", Bool),
    ("fido2Configured", "security", Bool),
    ("snapperNumberLimit", "security", Int),
    ("snapperTimeline", "security", Bool),
    ("fstrimEnabled", "security", Bool),
    ("directBoot", "security", Bool),
    ("omarchyChannel", "omarchy-channel", String),
    ("atmosChannel", "atmos-channel", String),
    ("favorites", "favorites", List),
    ("plugins", "plugins", List),
    ("avatarPath", "avatar", String),
];

pub fn specs() -> &'static [Spec] {
    SPECS
}

pub fn find(key: &str) -> Option<&'static Spec> {
    SPECS.iter().find(|spec| spec.key == key)
}

#[derive(Clone, Copy)]
pub enum LuaForm {
    /// `name = <scalar>`
    Value,
    /// `name = { enabled = <bool> }`
    Enabled,
    /// `hl.env("HYPRCURSOR_SIZE", "<int>")`
    Cursor,
    /// A `hl.gesture` whose action is `workspace`.
    Gesture,
    /// `kb_options` contains `grp:alts_toggle`.
    KbToggle,
}

pub struct LuaBind {
    pub name: &'static str,
    pub form: LuaForm,
}

pub fn lua_bind(key: &str) -> Option<LuaBind> {
    let (name, form) = match key {
        "hyprLook.cursorSize" => ("HYPRCURSOR_SIZE", LuaForm::Cursor),
        "hyprLook.gapsIn" => ("gaps_in", LuaForm::Value),
        "hyprLook.gapsOut" => ("gaps_out", LuaForm::Value),
        "hyprLook.rounding" => ("rounding", LuaForm::Value),
        "hyprLook.borderSize" => ("border_size", LuaForm::Value),
        "hyprLook.activeOpacity" => ("active_opacity", LuaForm::Value),
        "hyprLook.inactiveOpacity" => ("inactive_opacity", LuaForm::Value),
        "hyprLook.blur" => ("blur", LuaForm::Enabled),
        "hyprLook.shadow" => ("shadow", LuaForm::Enabled),
        "hyprLook.dimInactive" => ("dim_inactive", LuaForm::Value),
        "hyprLook.dimStrength" => ("dim_strength", LuaForm::Value),
        "hyprLook.animations" => ("animations", LuaForm::Enabled),
        "hyprLook.columnWidth" => ("column_width", LuaForm::Value),
        "hyprLook.cursorHideOnKey" => ("hide_on_key_press", LuaForm::Value),
        "hyprLook.cursorWarp" => ("warp_on_change_workspace", LuaForm::Value),
        "hyprLook.resizeOnBorder" => ("resize_on_border", LuaForm::Value),
        "hyprLook.allowTearing" => ("allow_tearing", LuaForm::Value),
        "hyprLook.layout" => ("layout", LuaForm::Value),
        "hyprLook.preserveSplit" => ("preserve_split", LuaForm::Value),
        "hyprLook.enableSwallow" => ("enable_swallow", LuaForm::Value),
        "hyprLook.swallowRegex" => ("swallow_regex", LuaForm::Value),
        "hyprLook.onFocusUnderFullscreen" => ("on_focus_under_fullscreen", LuaForm::Value),
        "hyprLook.focusOnActivate" => ("focus_on_activate", LuaForm::Value),
        "hyprInput.sensitivity" => ("sensitivity", LuaForm::Value),
        "hyprInput.accelProfile" => ("accel_profile", LuaForm::Value),
        "hyprInput.emulateDiscreteScroll" => ("emulate_discrete_scroll", LuaForm::Value),
        "hyprInput.naturalScroll" => ("natural_scroll", LuaForm::Value),
        "hyprInput.scrollFactor" => ("scroll_factor", LuaForm::Value),
        "hyprInput.clickfinger" => ("clickfinger_behavior", LuaForm::Value),
        "hyprInput.disableWhileTyping" => ("disable_while_typing", LuaForm::Value),
        "hyprInput.drag3fg" => ("drag_3fg", LuaForm::Value),
        "hyprInput.repeatRate" => ("repeat_rate", LuaForm::Value),
        "hyprInput.repeatDelay" => ("repeat_delay", LuaForm::Value),
        "hyprInput.numlock" => ("numlock_by_default", LuaForm::Value),
        "hyprInput.followMouse" => ("follow_mouse", LuaForm::Value),
        "hyprInput.keyPressDpms" => ("key_press_enables_dpms", LuaForm::Value),
        "hyprInput.mouseMoveDpms" => ("mouse_move_enables_dpms", LuaForm::Value),
        "hyprInput.kbLayoutOverride" => ("kb_layout", LuaForm::Value),
        "hyprInput.kbVariantOverride" => ("kb_variant", LuaForm::Value),
        "hyprInput.kbGroupToggle" => ("kb_options", LuaForm::KbToggle),
        "hyprInput.workspaceGesture" => ("workspace", LuaForm::Gesture),
        "touchpadEnabled" => ("touchpad_enabled", LuaForm::Value),
        "touchscreenEnabled" => ("touchscreen_enabled", LuaForm::Value),
        _ => return None,
    };
    Some(LuaBind { name, form })
}

pub fn locate(backend: &str, spec: &Spec) -> Result<Place, String> {
    if backend == "plain" {
        return Ok(Place {
            rel: format!(".config/plain/{}.json", spec.group),
            kind: PlaceKind::Map,
        });
    }
    if backend != "omarchy" {
        return Err(format!("unknown backend {backend}"));
    }
    if let Some(place) = omarchy_key(spec.key) {
        return Ok(place);
    }
    Ok(match spec.group {
        "theme" => map(".config/omarchy/theme.json"),
        "defaults" => map(".config/omarchy/defaults.json"),
        "network" => map(".config/omarchy/network.json"),
        "audio" => doc(".local/state/omarchy/audio-level"),
        "power" => map(".config/omarchy/power.json"),
        "env" => doc(".config/environment.d/10-atmos.conf"),
        "tweaks" => map(".config/omarchy/tweaks.json"),
        "security" => map(".config/omarchy/security.json"),
        "shell" => Place {
            rel: ".config/omarchy/shell.json".into(),
            kind: PlaceKind::Shell,
        },
        "look" => lua(
            ".config/hypr/looknfeel.lua",
            "-- atmos:look begin",
            "-- atmos:look end",
        ),
        "input" => lua(
            ".config/hypr/input.lua",
            "-- atmos:input begin",
            "-- atmos:input end",
        ),
        "bindings" => hypr("bindings", ".config/hypr/bindings.lua"),
        "windows" => hypr("windows", ".config/hypr/atmos.lua"),
        "workspaces" => hypr("workspaces", ".config/hypr/atmos.lua"),
        "autostart" => hypr("autostart", ".config/hypr/autostart.lua"),
        "monitors" => hypr("monitors", ".config/hypr/monitors.lua"),
        "hyprsunset" => doc(".config/hypr/hyprsunset.conf"),
        "hostname" => line("etc/hostname", ""),
        "timezone" => line("etc/timezone", ""),
        "locale" => line("etc/locale.conf", "LANG="),
        "keyboard" => line("etc/vconsole.conf", "XKBLAYOUT="),
        "ntp" => line("etc/systemd/timesyncd.conf.d/atmos-ntp.conf", "NTP="),
        "fullname" => line("var/lib/AccountsService/users/atmos", "FullName="),
        "downloads" => line("etc/pacman.conf", "ParallelDownloads = "),
        "wifiband" => line(".config/omarchy/wifi-band", ""),
        "backlight" => line("sys/class/backlight/acpi_video0/brightness", ""),
        "omarchy-channel" => line(".config/omarchy/channel", ""),
        "atmos-channel" => line(".config/atmos/channel", ""),
        "avatar" => line(".config/omarchy/avatar.path", ""),
        "favorites" => Place {
            rel: ".local/state/omarchy/atmos-favorites.json".into(),
            kind: PlaceKind::Items,
        },
        "plugins" => Place {
            rel: ".config/omarchy/plugins.json".into(),
            kind: PlaceKind::Whole,
        },
        other => return Err(format!("no omarchy file for group {other}")),
    })
}

fn map(rel: &str) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Map,
    }
}

fn line(rel: &str, prefix: &'static str) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Line { prefix },
    }
}

fn hypr(kind: &'static str, rel: &str) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Hypr { kind },
    }
}

fn doc(rel: &str) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Doc,
    }
}

fn lua(rel: &str, begin: &'static str, end: &'static str) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Lua { begin, end },
    }
}

fn flag(rel: &str) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Flag,
    }
}

fn nested(rel: &str, path: &'static [&'static str]) -> Place {
    Place {
        rel: rel.into(),
        kind: PlaceKind::Nested { path },
    }
}

fn omarchy_key(key: &str) -> Option<Place> {
    Some(match key {
        "hyprNoGaps" => flag(".local/state/omarchy/toggles/hypr/window-no-gaps.lua"),
        "hyprSquareAspect" => {
            flag(".local/state/omarchy/toggles/hypr/single-window-aspect-ratio.lua")
        }
        "doNotDisturb" => nested(".local/state/omarchy/notifications.json", &["dnd"]),
        "mimePdf" | "mimeImage" | "mimeVideo" => doc(".config/mimeapps.list"),
        "customDns" => line(".config/omarchy/custom-dns", ""),
        "bluetooth" => doc(".local/state/omarchy/bluetooth-power"),
        "wifiRadio" => doc(".local/state/omarchy/wifi-radio"),
        "suspendEnabled" => doc(".local/state/omarchy/suspend"),
        "crashCapture" => doc(".local/state/omarchy/crash-capture"),
        "presentationMode" => doc(".local/state/omarchy/atmos-presentation.json"),
        "chargeLimit" => doc("sys/class/power_supply/BAT0/charge_control_end_threshold"),
        "snapperNumberLimit" | "snapperTimeline" => doc("etc/snapper/configs/root"),
        "passwordlessSudo" => doc("etc/sudoers.d/99-omarchy-nopasswd"),
        "fingerprintConfigured" | "fido2Configured" => doc("etc/pam.d/sudo"),
        "sshdEnabled" => doc("etc/systemd/system-preset/atmos-sshd.preset"),
        "fstrimEnabled" => doc("etc/systemd/system/timers.target.wants/fstrim.timer"),
        "directBoot" => doc("sys/firmware/efi/omarchy-entry"),
        "sudolessDocker" => doc("etc/group.d/atmos-docker"),
        "monitorScale" | "internalDisplay" | "internalMirror" => {
            map(".config/omarchy/monitor-prefs.json")
        }
        _ => return None,
    })
}
