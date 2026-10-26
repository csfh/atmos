//! The sampling service. `atmosd daemon` collects every display kind slowly
//! and on repeat into a cache under the runtime dir; `display::load` reads
//! that cache and asks the daemon first when it holds nothing fresh.
//!
//! The settings snapshot is held too, as five topics (`settings`), but never
//! sampled on a clock: a topic is collected when a window first holds it and
//! when something invalidates it, and a write patches the cache with what it
//! read back, so a cache never shows a value from before a write (docs/adr/0011).

mod bus;
pub mod cache;
pub mod client;
mod commands;
pub mod daemon;
mod flight;
pub mod governor;
pub(crate) mod jobs;
pub mod launch;
mod ops;
mod owned;
pub mod policy;
mod schedule;
mod session;
mod settings;
mod topics;
pub(crate) mod watch;
pub mod worker;

/// What a session speaks. 2: the daemon runs reads for a window (docs/adr/0012);
/// a window refuses a daemon that reports less, or nothing.
pub const PROTOCOL: u64 = 2;

/// A name the daemon holds samples under: a display kind, or one of the
/// settings topics (docs/adr/0011). They are two lists, so what loops over the
/// inventories never collects settings by accident.
pub fn is_topic(name: &str) -> bool {
    crate::display::is_kind(name) || is_settings(name)
}

/// One of the settings topics: collected on demand, never on a clock.
pub fn is_settings(name: &str) -> bool {
    crate::snapshot::topics::is_topic(name)
}

/// Every name the daemon holds, for the status.
pub fn all_topics() -> impl Iterator<Item = &'static str> {
    crate::display::KINDS
        .iter()
        .chain(crate::snapshot::topics::NAMES.iter())
        .copied()
}

/// Inventories a task or an apply can change. Hardware is fixed for a boot and
/// `live` is never served old, so neither is dropped.
pub const AFTER_JOB: &[&str] = &["disks", "services", "software", "diagnostics"];

/// Drop cached kinds after something changed them. No daemon, no cache dir,
/// or a fixture root means there is nothing to drop.
pub fn drop_kinds(kinds: &[&str]) {
    if let Some(dir) = cache::dir_from_env() {
        cache::drop_kinds(&dir, kinds);
    }
}
