//! The Speed test page's logic (`pages/network/SpeedtestPage.qml`): a run is a download sample,
//! then an upload one, each a `speedtest.net` request that is given five seconds; what each row
//! says while it runs and after; and the rates as `RichUi` writes them.
//! `tests/catalog_speedtest.rs` runs the page's code under Node against this.

use std::time::Duration;

use serde_json::{Value, json};

use crate::catalog::control::Field;
use crate::js;

pub const NET: Field = Field::Key("netKind");

/// How long one direction is given (`speedTimer.interval`).
pub const PHASE: Duration = Duration::from_secs(5);

pub const NO_ROUTE: &str = "No default route. Join a network on the Network page, then retry.";

/// The page's run, as its properties hold it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Run {
    pub running: bool,
    /// `down`, `up`, or "" between runs.
    pub phase: String,
    pub down: String,
    pub up: String,
    pub error: String,
    /// The answer a phase waits for; a newer one, or Cancel, makes an older answer stale.
    pub token: u64,
    pub in_flight: bool,
}

impl Run {
    /// What the page keeps of it.
    pub fn to_value(&self) -> Value {
        json!({
            "running": self.running, "phase": self.phase, "down": self.down, "up": self.up,
            "error": self.error, "token": self.token, "inFlight": self.in_flight,
        })
    }

    pub fn of(v: Option<&Value>) -> Run {
        let Some(v) = v else {
            return Run::default();
        };
        let s = |n: &str| v[n].as_str().unwrap_or_default().to_owned();
        Run {
            running: v["running"] == Value::Bool(true),
            phase: s("phase"),
            down: s("down"),
            up: s("up"),
            error: s("error"),
            token: v["token"].as_u64().unwrap_or(0),
            in_flight: v["inFlight"] == Value::Bool(true),
        }
    }

    fn ran(&self) -> bool {
        !self.down.is_empty() || !self.up.is_empty()
    }

    /// `startSpeedtest`: false when a phase is in flight already.
    pub fn start(&mut self, disconnected: bool) -> bool {
        if self.in_flight {
            return false;
        }
        if disconnected {
            self.error = NO_ROUTE.into();
            return false;
        }
        self.error.clear();
        self.down.clear();
        self.up.clear();
        self.running = true;
        self.begin("down");
        true
    }

    /// `startSpeedPhase(phase)`: the token its answer must carry.
    pub fn begin(&mut self, phase: &str) -> u64 {
        self.phase = phase.into();
        self.token += 1;
        self.in_flight = true;
        self.token
    }

    /// `stopSpeedtest`.
    pub fn stop(&mut self) {
        self.token += 1;
        self.in_flight = false;
        self.phase.clear();
        self.running = false;
    }

    /// `finishSpeedPhase`: the next phase to begin, if there is one.
    pub fn finish(&mut self) -> Option<&'static str> {
        if self.phase == "down" {
            return Some("up");
        }
        self.phase.clear();
        self.running = false;
        None
    }

    /// The timer ran out before the answer (`speedTimer.onTriggered`).
    pub fn timed_out(&mut self) -> Option<&'static str> {
        self.token += 1;
        self.in_flight = false;
        self.finish()
    }

    /// `speedAnswered`: the reply's result (`stdout`, `stderr`, `exit`), or why there is none.
    pub fn answered(&mut self, reply: Result<&Value, &str>) -> Option<&'static str> {
        self.in_flight = false;
        if let Ok(result) = reply {
            self.adopt(result.get("stdout"));
        }
        let failed = match reply {
            Ok(result) => js::number(result.get("exit")) != 0.0,
            Err(_) => true,
        };
        if failed {
            let why = match reply {
                Ok(result) if js::truthy(result.get("stderr")) => js::string(&result["stderr"]),
                Ok(_) => String::new(),
                Err(why) => why.to_owned(),
            };
            let why = why.trim();
            self.error = if why.is_empty() {
                "Speed test failed".into()
            } else {
                why.into()
            };
            self.running = false;
            self.phase.clear();
            return None;
        }
        self.finish()
    }

    /// `adoptSpeed(stdout)`: each line that is a rate is the phase's.
    pub fn adopt(&mut self, stdout: Option<&Value>) {
        let text = stdout
            .filter(|v| js::truthy(Some(v)))
            .map_or_else(String::new, js::string);
        for line in text.split('\n') {
            let n = parse_mbps(line);
            if !n.is_finite() {
                continue;
            }
            let n = js::string(&js::json_number(n));
            match self.phase.as_str() {
                "down" => self.down = n,
                "up" => self.up = n,
                _ => {}
            }
        }
    }

    /// `phaseLabel()`.
    pub fn phase_label(&self, disconnected: bool) -> &'static str {
        if disconnected {
            "Run stays disabled until this machine has a default route. Join a network on the Network page."
        } else if self.running && self.phase == "up" {
            "Measuring upload."
        } else if self.running {
            "Measuring download."
        } else if self.ran() {
            "Last run on this page."
        } else {
            "Download, then upload. About five seconds each."
        }
    }

    /// `progressDescription()`.
    pub fn progress_says(&self) -> &'static str {
        if self.running && self.phase == "up" {
            "Measuring outbound on the default route."
        } else if self.running {
            "Measuring inbound on the default route."
        } else if self.ran() {
            "Last run finished. Copy a rate from Results, or Run again."
        } else {
            "About five seconds each direction. Opening this page starts a run."
        }
    }

    /// `progressValueText()`.
    pub fn progress_text(&self) -> String {
        let or = |label: String, dots: &str| {
            if label.is_empty() {
                dots.to_owned()
            } else {
                label
            }
        };
        if self.running && self.phase == "up" {
            format!("Upload {}", or(mbps_label(&self.up), "…"))
        } else if self.running {
            format!("Download {}", or(mbps_label(&self.down), "…"))
        } else if self.ran() {
            format!("Down {}   Up {}", rate(&self.down), rate(&self.up))
        } else {
            "Waiting".into()
        }
    }

    /// The bar's value, out of 100.
    pub fn progress(&self) -> f64 {
        match self.phase.as_str() {
            "down" => 40.0,
            "up" => 80.0,
            _ if !self.down.is_empty() => 100.0,
            _ => 0.0,
        }
    }

    /// `downloadDescription()`.
    pub fn down_says(&self, disconnected: bool) -> String {
        if !self.down.is_empty() {
            format!("{} inbound on the last sample.", mbps_label(&self.down))
        } else if self.running && self.phase == "down" {
            "Measuring inbound on the default route.".into()
        } else if disconnected {
            OFFLINE.into()
        } else {
            "No download sample yet. Run above measures inbound first.".into()
        }
    }

    /// `uploadDescription()`.
    pub fn up_says(&self, disconnected: bool) -> String {
        if !self.up.is_empty() {
            format!("{} outbound on the last sample.", mbps_label(&self.up))
        } else if self.running && self.phase == "up" {
            "Measuring outbound on the default route.".into()
        } else if self.running && self.phase == "down" {
            "Upload starts after the download sample. Copy stays disabled until that finishes."
                .into()
        } else if disconnected {
            OFFLINE.into()
        } else {
            "No upload sample yet. Run above measures outbound second.".into()
        }
    }
}

const OFFLINE: &str =
    "Copy stays disabled until a run finishes. Join a network on the Network page, then Run.";

/// `parseFloat`: the number at the start of the trimmed text, NaN when there is none.
fn parse_float(text: &str) -> f64 {
    let t = text.trim();
    let b = t.as_bytes();
    let mut end = 0;
    if end < b.len() && matches!(b[end], b'+' | b'-') {
        end += 1;
    }
    if t[end..].starts_with("Infinity") {
        return if t.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let digits = |i: &mut usize| {
        let at = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        *i > at
    };
    let mut any = digits(&mut end);
    if end < b.len() && b[end] == b'.' {
        let mut i = end + 1;
        if digits(&mut i) || any {
            any = true;
            end = i;
        }
    }
    if !any {
        return f64::NAN;
    }
    if end < b.len() && matches!(b[end], b'e' | b'E') {
        let mut i = end + 1;
        if i < b.len() && matches!(b[i], b'+' | b'-') {
            i += 1;
        }
        if digits(&mut i) {
            end = i;
        }
    }
    t[..end].parse().unwrap_or(f64::NAN)
}

/// `RichUi.parseMbpsLine`.
pub fn parse_mbps(line: &str) -> f64 {
    let n = parse_float(line);
    if n.is_finite() && n >= 0.0 {
        n
    } else {
        f64::NAN
    }
}

/// `RichUi.formatMbps`.
fn format_mbps(raw: &str) -> String {
    let n = parse_float(raw);
    if !n.is_finite() || n < 0.0 {
        String::new()
    } else if n == 0.0 {
        "0.0".into()
    } else if n < 10.0 {
        format!("{:.1}", js::round(n * 10.0) / 10.0)
    } else {
        js::string(&js::json_number(js::round(n)))
    }
}

/// `RichUi.mbpsLabel`.
pub fn mbps_label(raw: &str) -> String {
    match format_mbps(raw) {
        n if n.is_empty() => n,
        n => format!("{n} Mbps"),
    }
}

/// `rateText`.
pub fn rate(raw: &str) -> String {
    match mbps_label(raw) {
        l if l.is_empty() => "—".into(),
        l => l,
    }
}

/// `RichUi.mbpsCopyText(direction, raw)`.
pub fn copy_text(direction: &str, raw: &str) -> String {
    match mbps_label(raw) {
        l if l.is_empty() => l,
        l => format!("{} {l}", direction.trim()),
    }
}

/// The request of a phase.
pub fn request(phase: &str) -> Value {
    json!({ "op": "speedtest.net", "phase": phase })
}
