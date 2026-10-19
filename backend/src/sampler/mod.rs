//! The sampling service. `ratmos daemon` collects every display kind slowly
//! and on repeat into a cache under the runtime dir; `display::load` reads
//! that cache and asks the daemon first when it holds nothing fresh.
//!
//! Settings, theme, accounts and stamps stay live reads: they are cheap, and
//! a cache would show stale values right after a write.

mod bus;
pub mod cache;
pub mod client;
mod commands;
pub mod daemon;
mod flight;
pub mod governor;
pub mod launch;
mod ops;
pub mod policy;
mod schedule;
mod session;
mod topics;
pub(crate) mod watch;
pub mod worker;

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
