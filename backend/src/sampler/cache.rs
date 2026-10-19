//! The sample cache: one JSON file per display kind under the runtime dir.
//!
//! The runtime dir is tmpfs, so frequent samples never touch the disk and a
//! reboot starts empty. A sample records when its collection began. Dropping a
//! kind writes a `.dirty` marker with the time of the change, and any sample
//! that began at or before it is a miss, even one a running collection writes
//! a moment later.

use std::fs::DirBuilder;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::display;
use crate::error::{Error, Result};
use crate::fsutil::{atomic_write, atomic_write_text};

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    /// Unix milliseconds when the collection began.
    pub at_ms: u64,
    pub took_ms: u64,
    pub value: Value,
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Sample {
    pub fn age(&self) -> Duration {
        Duration::from_millis(now_ms().saturating_sub(self.at_ms))
    }

    pub fn to_json(&self) -> Value {
        json!({
            "version": crate::VERSION,
            "sampledAt": self.at_ms,
            "tookMs": self.took_ms,
            "value": self.value,
        })
    }

    /// A sample from another ratmos version is a miss, so an update never
    /// serves documents shaped by the old collectors.
    pub fn from_json(doc: &Value) -> Option<Sample> {
        if doc.get("version")?.as_str()? != crate::VERSION {
            return None;
        }
        Some(Sample {
            at_ms: doc.get("sampledAt")?.as_u64()?,
            took_ms: doc.get("tookMs")?.as_u64()?,
            value: doc.get("value")?.clone(),
        })
    }
}

/// `$XDG_RUNTIME_DIR/atmos`; never `/run/user/UID`, so a sandbox stays inside itself.
pub fn dir_from_env() -> Option<PathBuf> {
    let run = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .filter(|v| !v.is_empty())?;
    Some(PathBuf::from(run).join("atmos"))
}

pub fn socket(dir: &Path) -> PathBuf {
    dir.join("ratmos.sock")
}

pub fn ensure_dir(dir: &Path) -> Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .map_err(|err| Error::io(err, dir))
}

fn sample_path(dir: &Path, kind: &str) -> Option<PathBuf> {
    display::is_kind(kind).then(|| dir.join(format!("{kind}.json")))
}

fn dirty_path(dir: &Path, kind: &str) -> Option<PathBuf> {
    display::is_kind(kind).then(|| dir.join(format!("{kind}.dirty")))
}

fn dirty_ms(dir: &Path, kind: &str) -> Option<u64> {
    let text = std::fs::read_to_string(dirty_path(dir, kind)?).ok()?;
    text.trim().parse().ok()
}

/// The cached sample for a kind, unless it is unreadable, from another
/// version, or older than the last time the kind was dropped.
pub fn read(dir: &Path, kind: &str) -> Option<Sample> {
    let text = std::fs::read_to_string(sample_path(dir, kind)?).ok()?;
    let sample = Sample::from_json(&serde_json::from_str(&text).ok()?)?;
    match dirty_ms(dir, kind) {
        Some(dirty) if dirty >= sample.at_ms => None,
        _ => Some(sample),
    }
}

/// A kind with no valid sample that was dropped, and so wants one soon.
pub fn is_dirty(dir: &Path, kind: &str) -> bool {
    dirty_ms(dir, kind).is_some() && read(dir, kind).is_none()
}

pub fn write(dir: &Path, kind: &str, sample: &Sample) -> Result<()> {
    let path = sample_path(dir, kind).ok_or_else(|| Error::from(format!("unknown kind {kind}")))?;
    ensure_dir(dir)?;
    atomic_write(&path, sample.to_json().to_string().as_bytes())
}

/// Mark kinds as changed by a mutation. Readers miss until a newer sample lands.
pub fn drop_kinds(dir: &Path, kinds: &[&str]) {
    if !dir.is_dir() {
        return;
    }
    let now = now_ms().to_string();
    for kind in kinds {
        if let Some(path) = dirty_path(dir, kind) {
            let _ = atomic_write_text(&path, &now);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ratmos-sampler-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample(at_ms: u64) -> Sample {
        Sample {
            at_ms,
            took_ms: 3,
            value: json!({"a": 1}),
        }
    }

    #[test]
    fn a_written_sample_reads_back() {
        let dir = scratch("roundtrip");
        write(&dir, "services", &sample(now_ms())).unwrap();
        let back = read(&dir, "services").unwrap();
        assert_eq!(back.value, json!({"a": 1}));
        assert_eq!(back.took_ms, 3);
        assert!(read(&dir, "disks").is_none());
    }

    #[test]
    fn only_display_kinds_name_a_file() {
        let dir = scratch("kinds");
        assert!(write(&dir, "../escape", &sample(1)).is_err());
        assert!(read(&dir, "../escape").is_none());
    }

    #[test]
    fn another_version_is_a_miss() {
        let dir = scratch("version");
        let mut doc = sample(now_ms()).to_json();
        doc["version"] = json!("0.0.0-old");
        std::fs::write(dir.join("services.json"), doc.to_string()).unwrap();
        assert!(read(&dir, "services").is_none());
    }

    #[test]
    fn a_corrupt_file_is_a_miss() {
        let dir = scratch("corrupt");
        std::fs::write(dir.join("services.json"), "{not json").unwrap();
        assert!(read(&dir, "services").is_none());
    }

    #[test]
    fn dropping_a_kind_hides_samples_that_began_before_it() {
        let dir = scratch("drop");
        let before = now_ms();
        write(&dir, "services", &sample(before)).unwrap();
        drop_kinds(&dir, &["services"]);
        assert!(read(&dir, "services").is_none());
        assert!(is_dirty(&dir, "services"));
        // A collection that began after the drop is valid again.
        write(&dir, "services", &sample(now_ms() + 5)).unwrap();
        assert!(read(&dir, "services").is_some());
        assert!(!is_dirty(&dir, "services"));
    }

    #[test]
    fn a_collection_in_flight_across_a_drop_stays_a_miss() {
        let dir = scratch("race");
        let began = now_ms();
        drop_kinds(&dir, &["disks"]);
        write(&dir, "disks", &sample(began)).unwrap();
        assert!(read(&dir, "disks").is_none());
    }

    #[test]
    fn dropping_in_a_missing_dir_creates_nothing() {
        let dir = scratch("missing").join("absent");
        drop_kinds(&dir, &["services"]);
        assert!(!dir.exists());
    }
}
