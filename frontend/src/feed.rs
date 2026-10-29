//! The live samples the window keeps for the catalog's Monitor and Home pages, as
//! `LiveStatsStore` keeps them: the last minute of rows (`stats::push`), the interval the daemon
//! samples at, and Pause, which stops the feed and keeps what was taken. The window holds `live`
//! at that interval while a page that reads it is shown (`app.rs`).

use serde_json::Value;

use crate::stats;

/// `intervalChips()`: each interval's id, label and milliseconds.
pub const INTERVALS: &[(&str, &str, u64)] = &[
    ("100", "100ms", 100),
    ("250", "250ms", 250),
    ("500", "0.5s", 500),
    ("1000", "1s", 1000),
    ("2000", "2s", 2000),
    ("5000", "5s", 5000),
];

#[derive(Debug, Clone)]
pub struct Live {
    pub history: Vec<Value>,
    /// When the newest sample taken in was begun.
    last_at: f64,
    pub interval_ms: u64,
    pub paused: bool,
    pub error: String,
}

impl Default for Live {
    fn default() -> Self {
        Live {
            history: Vec::new(),
            last_at: 0.0,
            interval_ms: 2000,
            paused: false,
            error: String::new(),
        }
    }
}

impl Live {
    /// `adoptFrame`: a sample begun at `at`, taken in unless it is not newer than the last (the
    /// same sample twice would read as a rate of zero). Whether anything changed.
    pub fn adopt(&mut self, at: f64, result: &Value) -> bool {
        if at.is_nan() || at <= self.last_at {
            return false;
        }
        match stats::parse(result) {
            None => self.error = "Could not parse a live sample.".into(),
            Some(sample) => {
                self.error.clear();
                self.last_at = at;
                self.history = stats::push(&self.history, &sample, at);
            }
        }
        true
    }

    /// `latest`: the newest row.
    pub fn latest(&self) -> Option<&Value> {
        self.history.last()
    }

    /// The interval to hold `live` at, or none while paused.
    pub fn held(&self) -> Option<u64> {
        (!self.paused).then_some(self.interval_ms)
    }

    /// `setIntervalId(id)`: one of the intervals, two seconds for any other. Whether it changed.
    pub fn set_interval(&mut self, id: &str) -> bool {
        let ms = INTERVALS
            .iter()
            .find(|(i, _, _)| *i == id)
            .map_or(2000, |(_, _, ms)| *ms);
        let changed = ms != self.interval_ms;
        self.interval_ms = ms;
        changed
    }
}
