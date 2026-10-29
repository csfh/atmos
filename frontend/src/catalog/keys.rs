//! The snapshot keys a mapped row of a catalog page reads (docs/adr/0021): the settings topic
//! that holds each (`tests/pins.rs` holds the pair to `backend/src/snapshot/keys.rs`), what the
//! page says when it cannot be read, and how `Snapshot.clampPatchedValue` reads it
//! (`tests/catalog_controls.rs` runs that under Node against these).

use serde_json::Value;

use crate::settings::Held;

const LOOK: &str = "settings.look";
const HOST: &str = "settings.host";
const NETWORK: &str = "settings.network";
const SYSTEM: &str = "settings.system";
const ACCOUNTS: &str = "settings.accounts";

/// How `clampPatchedValue` reads a key of its own.
#[derive(Debug, Clone, Copy)]
pub enum Reads {
    /// `value === true` (`BOOL_ON_KEYS`).
    On,
    /// `value !== false` (`BOOL_OFF_KEYS`).
    Off,
    /// `String(value || default)`: `""` for `STRING_KEYS`, `"top"` for `barPosition`.
    Text(&'static str),
    /// `Number(value) || default`.
    Number(f64),
    /// `isArray(value) ? value : default` (`ARRAY_KEYS`, with `[]`).
    List(&'static [&'static str]),
    /// A rule of the key's own (`clampVolume`, `spacerSize`, ...), in `special`.
    Own(fn(&Value) -> Value),
}

/// The keys `clampPatchedValue` reads by a rule of their own.
mod special {
    use serde_json::Value;

    use crate::js;

    /// `clampVolume`: a whole number from 0 to 100, 0 for one that is not.
    pub fn volume(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        let n = if n == 0.0 || n.is_nan() { 0.0 } else { n };
        js::json_number(n.clamp(0.0, 100.0))
    }

    /// `reminderCount`: a whole number, 0 for one that is not or is below 0.
    pub fn count(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        js::json_number(if n.is_nan() || n < 0.0 { 0.0 } else { n })
    }

    /// `netIp`: digits, hex, `.` and `:`, else "".
    pub fn ip(v: &Value) -> Value {
        let s = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let ok = !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_hexdigit() || c == '.' || c == ':');
        Value::from(if ok { s } else { String::new() })
    }

    /// `clampIface`: letters, digits, `.`, `_` and `-`, else "".
    pub fn iface(v: &Value) -> Value {
        let s = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let ok = !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
        Value::from(if ok { s } else { String::new() })
    }

    /// `clampDigits`: digits, else "".
    pub fn digits(v: &Value) -> Value {
        let s = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        Value::from(if ok { s } else { String::new() })
    }

    /// `spacerSize`: 12 for one that is not a number or is below 0, and at most 64.
    pub fn spacer(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        js::json_number(if !n.is_finite() || n < 0.0 {
            12.0
        } else {
            n.min(64.0)
        })
    }

    /// `parallelDownloads`: 5 for one that is not a number or is below 1, and at most 20.
    pub fn downloads(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        let n = if n == 0.0 || n.is_nan() || n < 1.0 {
            5.0
        } else {
            n
        };
        js::json_number(n.min(20.0))
    }

    /// `weatherUnit`: `metric` or `imperial`, else `auto`.
    pub fn weather_unit(v: &Value) -> Value {
        let text = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        Value::from(if matches!(text.as_str(), "metric" | "imperial") {
            text.as_str()
        } else {
            "auto"
        })
    }

    /// `weatherRefreshMinutes`: 15 for one that is not a number or is below 1.
    pub fn weather_minutes(v: &Value) -> Value {
        let n = js::number(Some(v));
        js::json_number(if n == 0.0 || n.is_nan() || n < 1.0 {
            15.0
        } else {
            n
        })
    }

    /// `nightlightTemperature`: 0 for none, else a whole number from 3000 to 6500.
    pub fn kelvin(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        let n = if n == 0.0 || n.is_nan() {
            0.0
        } else {
            n.clamp(3000.0, 6500.0)
        };
        js::json_number(n)
    }

    /// `nightlightDay`, with the window's `parseTime`: 07:00 for a time it does not take.
    pub fn day(v: &Value) -> Value {
        time(v, "07:00")
    }

    /// `nightlightNight`, likewise: 20:00 for a time it does not take.
    pub fn night(v: &Value) -> Value {
        time(v, "20:00")
    }

    fn time(v: &Value, or: &str) -> Value {
        let text = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let parsed = crate::appearance::parse_time(&text);
        Value::from(if parsed.is_empty() {
            or.to_owned()
        } else {
            parsed
        })
    }

    /// `omarchyChannel`: one of the four channels, else "".
    pub fn omarchy_channel(v: &Value) -> Value {
        let c = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        Value::from(if matches!(c.as_str(), "stable" | "rc" | "edge" | "dev") {
            c
        } else {
            String::new()
        })
    }

    /// `atmosChannel`, with the window's `parseChannel`: stable is the only one.
    pub fn atmos_channel(_: &Value) -> Value {
        Value::from("stable")
    }

    /// `atmosRevision`: a short or full commit, else "".
    pub fn atmos_revision(v: &Value) -> Value {
        let r = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let ok =
            (4..=40).contains(&r.len()) && r.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'));
        Value::from(if ok { r } else { String::new() })
    }

    /// `atmosVersion`: a package version, else "".
    pub fn atmos_version(v: &Value) -> Value {
        let p = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let mut chars = p.chars();
        let ok = p.len() <= 40
            && chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
            && chars.all(|c| c.is_ascii_alphanumeric() || ".+_~-".contains(c));
        Value::from(if ok { p } else { String::new() })
    }

    /// `weatherCoords`, with the window's `parseWeatherCoords`.
    pub fn weather_coords(v: &Value) -> Value {
        let t = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        Value::from(crate::system::parse_coords(&t))
    }

    /// `keyboardLayout`: the first of a list, a short lower-case name, else "".
    pub fn keyboard_layout(v: &Value) -> Value {
        let t = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let t = t.split(',').next().unwrap_or_default();
        let ok = !t.is_empty()
            && t.len() <= 8
            && t.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        Value::from(if ok { t } else { "" })
    }

    /// `trayHidden`, `trayPinned`: `normalizedStringIds`.
    pub fn string_ids(v: &Value) -> Value {
        serde_json::json!(crate::bar::string_ids(Some(v)))
    }

    /// `indicatorsItems`: `normalizedIndicatorItems`.
    pub fn indicator_items(v: &Value) -> Value {
        serde_json::json!(crate::bar::indicator_items(Some(v)))
    }

    /// `agentsRefreshIntervalSec`: 900 for none, and at least 30.
    pub fn refresh_seconds(v: &Value) -> Value {
        let n = js::number(Some(v));
        let n = if n == 0.0 || n.is_nan() { 900.0 } else { n };
        js::json_number(n.max(30.0))
    }

    /// `clockWeekStart`: a day's name in lower case, else "".
    pub fn week_start(v: &Value) -> Value {
        let day = if js::truthy(Some(v)) {
            js::string(v).to_lowercase()
        } else {
            String::new()
        };
        let ok = crate::bar::DAYS.iter().any(|(d, _)| *d == day);
        Value::from(if ok { day } else { String::new() })
    }

    /// `clockBirthYear`: a whole number from 1, else 0.
    pub fn birth_year(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        js::json_number(if n.is_nan() || n < 1.0 { 0.0 } else { n })
    }

    /// `clockLifeExpectancy`: a whole number from 1 to 150, else 0.
    pub fn life_expectancy(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        js::json_number(if n.is_nan() || !(1.0..=150.0).contains(&n) {
            0.0
        } else {
            n
        })
    }

    /// `snapperNumberLimit`: a whole number from 1 to 50, 5 for one that is not.
    pub fn number_limit(v: &Value) -> Value {
        let n = js::round(js::number(Some(v)));
        let n = if n == 0.0 || n.is_nan() || n < 1.0 {
            5.0
        } else {
            n
        };
        js::json_number(n.min(50.0))
    }

    /// `netKind`: `ethernet` or `wifi`, else `disconnected`.
    pub fn net_kind(v: &Value) -> Value {
        let text = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        Value::from(if matches!(text.as_str(), "ethernet" | "wifi") {
            text.as_str()
        } else {
            "disconnected"
        })
    }

    /// `timezone`: a zone name of letters, digits and `/_+-`, with no `..`, else "".
    pub fn timezone(v: &Value) -> Value {
        let text = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        let ok = !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "/_+-".contains(c))
            && !text.contains("..");
        Value::from(if ok { text } else { String::new() })
    }

    /// `locale`: `C.UTF-8`, or `ll[_CC].UTF-8[@variant]`, else "".
    pub fn locale(v: &Value) -> Value {
        let text = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        Value::from(if text == "C.UTF-8" || is_locale(&text) {
            text
        } else {
            String::new()
        })
    }

    /// `^[a-z]{2,3}(_[A-Z]{2})?\.UTF-8(@[A-Za-z0-9]+)?$`
    fn is_locale(text: &str) -> bool {
        let (head, variant) = match text.split_once('@') {
            Some((head, v)) => (head, Some(v)),
            None => (text, None),
        };
        if variant.is_some_and(|v| v.is_empty() || !v.chars().all(|c| c.is_ascii_alphanumeric())) {
            return false;
        }
        let Some(name) = head.strip_suffix(".UTF-8") else {
            return false;
        };
        let (lang, country) = match name.split_once('_') {
            Some((l, c)) => (l, Some(c)),
            None => (name, None),
        };
        (2..=3).contains(&lang.len())
            && lang.chars().all(|c| c.is_ascii_lowercase())
            && country.is_none_or(|c| c.len() == 2 && c.chars().all(|c| c.is_ascii_uppercase()))
    }

    /// `hyprWorkspaceLayout`: `scrolling`, else `dwindle`.
    pub fn workspace_layout(v: &Value) -> Value {
        let text = if js::truthy(Some(v)) {
            js::string(v)
        } else {
            String::new()
        };
        Value::from(if text == "scrolling" {
            "scrolling"
        } else {
            "dwindle"
        })
    }

    /// `wifiBands`: the list, or `["auto"]`.
    pub fn wifi_bands(v: &Value) -> Value {
        if v.is_array() {
            v.clone()
        } else {
            Value::from(vec!["auto"])
        }
    }
}

/// The snapshot keys a mapped row reads, the topic that holds each (`tests/pins.rs` holds the
/// pair to `keys.rs`), how a key of its own is read, and what the page says when it cannot be.
#[rustfmt::skip]
pub const KEYS: &[(Held, Option<Reads>)] = &[
    (held("hyprLook", LOOK, "The window look could not be read"), None),
    (held("hyprInput", LOOK, "The input settings could not be read"), None),
    (held("hyprLookManaged", LOOK, "Whether Atmos manages the look could not be read"), Some(Reads::On)),
    (held("hyprInputManaged", LOOK, "Whether Atmos manages input could not be read"), Some(Reads::On)),
    (held("hyprNoGaps", LOOK, "Tight windows could not be read"), Some(Reads::On)),
    (held("hyprSquareAspect", LOOK, "Square single window could not be read"), Some(Reads::On)),
    (held("windowOpaque", HOST, "Opaque windows could not be read"), Some(Reads::On)),
    (held("textSize", LOOK, "The text size could not be read"), Some(Reads::Number(12.0))),
    (held("touchscreenEnabled", LOOK, "The touchscreen switch could not be read"), Some(Reads::Off)),
    (held("touchscreenPresent", LOOK, "Whether there is a touchscreen could not be read"), Some(Reads::On)),
    (held("stayAwake", LOOK, "Stay awake could not be read"), Some(Reads::On)),
    (held("screensaverEnabled", LOOK, "The screensaver switch could not be read"), Some(Reads::Off)),
    (held("suspendEnabled", LOOK, "The suspend menu switch could not be read"), Some(Reads::Off)),
    (held("powerShowPercentage", LOOK, "Battery percentage could not be read"), Some(Reads::On)),
    (held("isLaptop", LOOK, "Whether this is a laptop could not be read"), Some(Reads::On)),
    (held("batteryPresent", LOOK, "Whether there is a battery could not be read"), Some(Reads::On)),
    (held("powerPresent", LOOK, "Whether there is a power supply could not be read"), Some(Reads::On)),
    (held("nightlight", LOOK, "Night light could not be read"), Some(Reads::On)),
    (held("barPosition", LOOK, "Where the bar sits could not be read"), Some(Reads::Text("top"))),
    (held("barTransparent", LOOK, "Whether the bar is transparent could not be read"), Some(Reads::On)),
    (held("barVisible", LOOK, "Whether the bar is shown could not be read"), Some(Reads::Off)),
    (held("indicatorsAlwaysShow", LOOK, "Always show could not be read"), Some(Reads::On)),
    (held("indicatorsPresent", LOOK, "Whether the bar has indicators could not be read"), Some(Reads::On)),
    (held("agentsSync", LOOK, "Sync usage could not be read"), Some(Reads::On)),
    (held("agentsPresent", LOOK, "Whether the bar has the agents widget could not be read"), Some(Reads::On)),
    (held("doNotDisturb", LOOK, "Do not disturb could not be read"), Some(Reads::On)),
    (held("audioTuningOn", NETWORK, "Speaker tuning could not be read"), Some(Reads::On)),
    (held("audioTuningMatch", NETWORK, "Whether these speakers can be tuned could not be read"), Some(Reads::On)),
    (held("bluetooth", NETWORK, "The Bluetooth radio could not be read"), Some(Reads::On)),
    (held("ntp", SYSTEM, "Network time could not be read"), Some(Reads::On)),
    (held("ntpAvailable", SYSTEM, "Whether network time is available could not be read"), Some(Reads::On)),
    (held("weatherPresent", LOOK, "Whether the bar has the weather widget could not be read"), Some(Reads::On)),
    (held("reminderActive", LOOK, "Whether a reminder is set could not be read"), Some(Reads::On)),
    (held("wifiHw", NETWORK, "Whether there is a Wi-Fi radio could not be read"), Some(Reads::On)),
    (held("crashCapture", SYSTEM, "Crash capture could not be read"), Some(Reads::Off)),
    (held("powerProfile", HOST, "The power profile could not be read"), Some(Reads::Text(""))),
    (held("powerProfileAc", HOST, "The profile on AC could not be read"), Some(Reads::Text(""))),
    (held("powerProfileBattery", HOST, "The profile on battery could not be read"), Some(Reads::Text(""))),
    (held("powerProfiles", HOST, "The power profiles could not be read"), Some(Reads::List(&[]))),
    (held("presentationMode", SYSTEM, "Presentation Mode could not be read"), None),
    (held("powerGovernor", SYSTEM, "The CPU governor could not be read"), None),
    (held("amdPstate", SYSTEM, "The energy preference could not be read"), None),
    (held("chargeLimit", SYSTEM, "The charge limit could not be read"), None),
    (held("chargeLimitAvailable", SYSTEM, "Whether there is a charge limit could not be read"), None),
    (held("idleScreensaver", LOOK, "The screensaver timeout could not be read"), Some(Reads::Number(0.0))),
    (held("idleLock", LOOK, "The lock timeout could not be read"), Some(Reads::Number(0.0))),
    (held("audioOutputVolume", NETWORK, "The volume could not be read"), Some(Reads::Own(special::volume))),
    (held("audioInputVolume", NETWORK, "The capture volume could not be read"), Some(Reads::Own(special::volume))),
    (held("audioOutputMuted", NETWORK, "Whether the output is muted could not be read"), Some(Reads::On)),
    (held("audioInputMuted", NETWORK, "Whether the microphone is muted could not be read"), Some(Reads::On)),
    (held("audioSinks", NETWORK, "The outputs could not be read"), Some(Reads::List(&[]))),
    (held("audioSources", NETWORK, "The inputs could not be read"), Some(Reads::List(&[]))),
    (held("spacerPresent", LOOK, "Whether the bar has a spacer could not be read"), Some(Reads::On)),
    (held("spacerSize", LOOK, "The spacer width could not be read"), Some(Reads::Own(special::spacer))),
    (held("wifiRadio", NETWORK, "The Wi-Fi radio could not be read"), Some(Reads::On)),
    (held("wifiConnected", NETWORK, "Whether Wi-Fi is connected could not be read"), Some(Reads::On)),
    (held("wifiBands", NETWORK, "The Wi-Fi bands could not be read"), Some(Reads::Own(special::wifi_bands))),
    (held("wifiBandSelected", NETWORK, "The Wi-Fi band could not be read"), Some(Reads::Text("auto"))),
    (held("parallelDownloads", SYSTEM, "Parallel downloads could not be read"), Some(Reads::Own(special::downloads))),
    (held("weatherUnit", LOOK, "The weather units could not be read"), Some(Reads::Own(special::weather_unit))),
    (held("weatherRefreshMinutes", LOOK, "The weather refresh could not be read"), Some(Reads::Own(special::weather_minutes))),
    (held("internalPresent", LOOK, "Whether there is a laptop screen could not be read"), Some(Reads::On)),
    (held("externalPresent", LOOK, "Whether there is an external display could not be read"), Some(Reads::On)),
    (held("internalEnabled", LOOK, "Whether the laptop screen is on could not be read"), Some(Reads::On)),
    (held("mirroring", LOOK, "Whether the laptop screen is mirrored could not be read"), Some(Reads::On)),
    (held("hyprWorkspaceGesture", LOOK, "The workspace swipe could not be read"), Some(Reads::On)),
    (held("hyprWorkspaceGestureUnmanaged", LOOK, "Whether the workspace swipe is managed could not be read"), Some(Reads::On)),
    (held("font", LOOK, "The font could not be read"), Some(Reads::Text(""))),
    (held("fonts", LOOK, "The installed fonts could not be read"), Some(Reads::List(&[]))),
    (held("plymouth", LOOK, "The unlock theme could not be read"), Some(Reads::Text(""))),
    (held("plymouthThemes", LOOK, "The unlock themes could not be read"), Some(Reads::List(&[]))),
    (held("locale", SYSTEM, "The locale could not be read"), Some(Reads::Own(special::locale))),
    (held("locales", SYSTEM, "The locales could not be read"), Some(Reads::List(&[]))),
    (held("timezone", SYSTEM, "The timezone could not be read"), Some(Reads::Own(special::timezone))),
    (held("timezones", SYSTEM, "The timezones could not be read"), Some(Reads::List(&[]))),
    (held("nightlightTemperature", LOOK, "The night light warmth could not be read"), Some(Reads::Own(special::kelvin))),
    (held("tailscaleInstalled", NETWORK, "Whether Tailscale is installed could not be read"), Some(Reads::On)),
    (held("theme", LOOK, "The theme could not be read"), Some(Reads::Text(""))),
    (held("hasAether", HOST, "Whether Aether is installed could not be read"), Some(Reads::On)),
    (held("snapperPresent", HOST, "Whether Snapper is set up could not be read"), Some(Reads::On)),
    (held("aboutBranded", LOOK, "Whether the about logo is custom could not be read"), Some(Reads::On)),
    (held("screensaverBranded", LOOK, "Whether the screensaver logo is custom could not be read"), Some(Reads::On)),
    (held("cupsActive", HOST, "Whether printing is on could not be read"), Some(Reads::On)),
    (held("directBoot", SYSTEM, "Whether direct boot is set up could not be read"), Some(Reads::On)),
    (held("fido2Configured", HOST, "Whether a security key is set up could not be read"), Some(Reads::On)),
    (held("fingerprintConfigured", HOST, "Whether a fingerprint is set up could not be read"), Some(Reads::On)),
    (held("passwordlessSudo", HOST, "Whether sudo mode is on could not be read"), Some(Reads::On)),
    (held("sudolessDocker", HOST, "Whether sudoless Docker is set up could not be read"), Some(Reads::On)),
    (held("voxtypeInstalled", HOST, "Whether dictation is installed could not be read"), Some(Reads::On)),
    (held("hyprWorkspaceLayout", LOOK, "This workspace's layout could not be read"), Some(Reads::Own(special::workspace_layout))),
    (held("fingerprintAvailable", HOST, "Whether there is a fingerprint reader could not be read"), Some(Reads::On)),
    (held("sshdEnabled", HOST, "Whether SSH is enabled could not be read"), Some(Reads::On)),
    (held("sshdActive", HOST, "Whether SSH is running could not be read"), Some(Reads::On)),
    (held("directBootAvailable", SYSTEM, "Whether direct boot is possible could not be read"), Some(Reads::On)),
    (held("tweaks", SYSTEM, "The tweaks could not be read"), None),
    (held("reminders", LOOK, "The reminders could not be read"), Some(Reads::List(&[]))),
    (held("agent", SYSTEM, "The default agent could not be read"), Some(Reads::Text(""))),
    (held("envDetected", SYSTEM, "What the session uses could not be read"), None),
    (held("envVars", SYSTEM, "The variables could not be read"), None),
    (held("envPathPrepend", SYSTEM, "The PATH prepend could not be read"), None),
    (held("autostart", HOST, "The startup programs could not be read"), Some(Reads::List(&[]))),
    (held("windowRules", HOST, "The window rules could not be read"), Some(Reads::List(&[]))),
    (held("focusedClass", HOST, "The focused window could not be read"), Some(Reads::Text(""))),
    (held("extras", HOST, "The installed extras could not be read"), None),
    (held("browsers", HOST, "The installed browsers could not be read"), None),
    (held("terminals", HOST, "The installed terminals could not be read"), None),
    (held("editors", HOST, "The installed editors could not be read"), None),
    (held("services", HOST, "The installed services could not be read"), None),
    (held("gaming", HOST, "The installed games could not be read"), None),
    (held("themes", LOOK, "The themes could not be read"), Some(Reads::List(&[]))),
    (held("extraThemes", LOOK, "The extra themes could not be read"), Some(Reads::List(&[]))),
    (held("nightlightDay", LOOK, "The night light schedule could not be read"), Some(Reads::Own(special::day))),
    (held("nightlightNight", LOOK, "The night light schedule could not be read"), Some(Reads::Own(special::night))),
    (held("nightlightNightOn", LOOK, "Whether the schedule is used could not be read"), Some(Reads::On)),
    (held("recordingActive", HOST, "Whether a recording runs could not be read"), Some(Reads::On)),
    (held("webcamOverlay", HOST, "Whether the webcam overlay is up could not be read"), Some(Reads::On)),
    (held("picturesDir", SYSTEM, "The Pictures folder could not be read"), Some(Reads::Text(""))),
    (held("videosDir", SYSTEM, "The Videos folder could not be read"), Some(Reads::Text(""))),
    (held("netKind", NETWORK, "The connection could not be read"), Some(Reads::Own(special::net_kind))),
    (held("desktopApps", HOST, "The desktop launchers could not be read"), Some(Reads::List(&[]))),
    (held("tuiApps", HOST, "The terminal launchers could not be read"), Some(Reads::List(&[]))),
    (held("webApps", HOST, "The web apps could not be read"), Some(Reads::List(&[]))),
    (held("snapperConfigs", HOST, "Snapper's configs could not be read"), Some(Reads::List(&[]))),
    (held("hibernationConfigured", HOST, "Hibernation could not be read"), Some(Reads::On)),
    (held("hibernationSupported", HOST, "Hibernation could not be read"), Some(Reads::On)),
    (held("snapperNumberLimit", HOST, "The snapshot limit could not be read"), Some(Reads::Own(special::number_limit))),
    (held("snapperTimeline", HOST, "Timeline snapshots could not be read"), Some(Reads::On)),
    (held("fstrimEnabled", HOST, "Weekly TRIM could not be read"), Some(Reads::On)),
    (held("omarchyVersion", HOST, "The Omarchy version could not be read"), Some(Reads::Text(""))),
    (held("atmosVersion", HOST, "The Atmos version could not be read"), Some(Reads::Own(special::atmos_version))),
    (held("atmosRevision", HOST, "The Atmos revision could not be read"), Some(Reads::Own(special::atmos_revision))),
    (held("atmosPackaged", HOST, "Whether Atmos is packaged could not be read"), Some(Reads::On)),
    (held("trayPresent", LOOK, "Whether the tray is on the bar could not be read"), Some(Reads::On)),
    (held("trayHidden", LOOK, "The hidden tray icons could not be read"), Some(Reads::Own(special::string_ids))),
    (held("trayPinned", LOOK, "The pinned tray icons could not be read"), Some(Reads::Own(special::string_ids))),
    (held("indicatorsItems", LOOK, "The shown indicators could not be read"), Some(Reads::Own(special::indicator_items))),
    (held("agentsRefreshIntervalSec", LOOK, "The usage refresh could not be read"), Some(Reads::Own(special::refresh_seconds))),
    (held("agentsSyncDir", LOOK, "The sync folder could not be read"), Some(Reads::Text(""))),
    (held("agentsSyncFileName", LOOK, "The snapshot file could not be read"), Some(Reads::Text(""))),
    (held("agentsSyncDeviceId", LOOK, "The device id could not be read"), Some(Reads::Text(""))),
    (held("clockPresent", LOOK, "Whether the clock is on the bar could not be read"), Some(Reads::On)),
    (held("clockFormat", LOOK, "The clock format could not be read"), Some(Reads::Text(""))),
    (held("clockFormatAlt", LOOK, "The alternate format could not be read"), Some(Reads::Text(""))),
    (held("clockWeekStart", LOOK, "The week start could not be read"), Some(Reads::Own(special::week_start))),
    (held("clockBirthYear", LOOK, "The birth year could not be read"), Some(Reads::Own(special::birth_year))),
    (held("clockLifeExpectancy", LOOK, "The life expectancy could not be read"), Some(Reads::Own(special::life_expectancy))),
    (held("plugins", HOST, "The plugins could not be read"), Some(Reads::List(&[]))),
    (held("hostname", SYSTEM, "The hostname could not be read"), None),
    (held("omarchyChannel", HOST, "The Omarchy channel could not be read"), Some(Reads::Own(special::omarchy_channel))),
    (held("updateAvailable", HOST, "Whether an update is ready could not be read"), Some(Reads::On)),
    (held("updateSummary", HOST, "The update summary could not be read"), Some(Reads::Text(""))),
    (held("atmosChannel", HOST, "The Atmos channel could not be read"), Some(Reads::Own(special::atmos_channel))),
    (held("atmosInstalled", HOST, "Whether Atmos is installed could not be read"), Some(Reads::On)),
    (held("printerSetup", HOST, "Whether the printer window is installed could not be read"), Some(Reads::On)),
    (held("weatherAuto", HOST, "The weather location could not be read"), Some(Reads::Off)),
    (held("weatherLocation", HOST, "The weather location could not be read"), Some(Reads::Text(""))),
    (held("weatherCoords", HOST, "The weather coordinates could not be read"), Some(Reads::Own(special::weather_coords))),
    (held("hooks", HOST, "The hooks could not be read"), None),
    (held("keyboardLayout", SYSTEM, "The keyboard layout could not be read"), Some(Reads::Own(special::keyboard_layout))),
    (held("keyboardLayouts", SYSTEM, "The keyboard layouts could not be read"), Some(Reads::List(&[]))),
    (held("systemdUnits", SYSTEM, "The services could not be read"), None),
    (held("workspaces", SYSTEM, "The workspaces could not be read"), None),
    (held("workspaceWrapSwitch", SYSTEM, "Wrap switching could not be read"), None),
    (held("workspaceWheelSwitch", SYSTEM, "Super + mouse wheel could not be read"), None),
    (held("workspaceBarCount", LOOK, "The bar's workspaces could not be read"), None),
    (held("workspaceBarNames", LOOK, "Show names in the bar could not be read"), None),
    (held("monitors", LOOK, "The monitors could not be read"), Some(Reads::List(&[]))),
    (held("currentUser", ACCOUNTS, "This login could not be read"), None),
    (held("avatarPath", ACCOUNTS, "The face could not be read"), None),
    (held("fullName", ACCOUNTS, "The full name could not be read"), None),
    (held("users", ACCOUNTS, "The logins could not be read"), Some(Reads::List(&[]))),
    (held("groups", ACCOUNTS, "The groups could not be read"), Some(Reads::List(&[]))),
    (held("wifiIface", NETWORK, "The Wi-Fi interface could not be read"), Some(Reads::Own(special::iface))),
    (held("wifiConnections", NETWORK, "The saved networks could not be read"), Some(Reads::List(&[]))),
    (held("wifiBand", NETWORK, "The Wi-Fi band could not be read"), Some(Reads::Text(""))),
    (held("tailscaleRunning", NETWORK, "Whether Tailscale is running could not be read"), Some(Reads::On)),
    (held("netIp", NETWORK, "The address could not be read"), Some(Reads::Own(special::ip))),
    (held("netGateway", NETWORK, "The gateway could not be read"), None),
    (held("netDnsServers", NETWORK, "The resolvers could not be read"), None),
    (held("netSsid", NETWORK, "The network name could not be read"), Some(Reads::Text(""))),
    (held("netIface", NETWORK, "The interface could not be read"), Some(Reads::Own(special::iface))),
    (held("netSpeed", NETWORK, "The link speed could not be read"), Some(Reads::Own(special::digits))),
    (held("netSignal", NETWORK, "The signal could not be read"), Some(Reads::Own(special::digits))),
    (held("dns", NETWORK, "The DNS provider could not be read"), Some(Reads::Text(""))),
    (held("bluetoothDevices", NETWORK, "The paired devices could not be read"), Some(Reads::List(&[]))),
    (held("bluetoothDiscoverable", NETWORK, "Whether others can find this adapter could not be read"), None),
    (held("background", LOOK, "The wallpaper could not be read"), Some(Reads::Text(""))),
    (held("reminderCount", LOOK, "The reminders could not be read"), Some(Reads::Own(special::count))),
    (held("ntpSynchronized", SYSTEM, "Whether the clock is in sync could not be read"), Some(Reads::On)),
    (held("bindings", HOST, "Your keybindings could not be read"), Some(Reads::List(&[]))),
    (held("keybindings", HOST, "What is bound could not be read"), Some(Reads::List(&[]))),
    (held("monitorRules", SYSTEM, "The monitor rules could not be read"), None),
    (held("touchpadPresent", LOOK, "Whether there is a touchpad could not be read"), Some(Reads::On)),
    (held("touchpadEnabled", LOOK, "Whether the touchpad is on could not be read"), Some(Reads::Off)),
    (held("keyboardBacklightPresent", LOOK, "Whether the keyboard has a backlight could not be read"), Some(Reads::On)),
    (held("keyboardBrightness", LOOK, "The keyboard backlight could not be read"), Some(Reads::Own(special::volume))),
];

const fn held(key: &'static str, topic: &'static str, unread: &'static str) -> Held {
    Held { key, topic, unread }
}

pub(super) fn entry(key: &str) -> &'static (Held, Option<Reads>) {
    KEYS.iter()
        .find(|(h, _)| h.key == key)
        .unwrap_or_else(|| panic!("`{key}` is not a key a mapped row reads"))
}
