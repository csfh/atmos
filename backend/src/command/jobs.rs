//! The registry of jobs: work that streams, reads input or needs the user's
//! session, named and checked like a write (`writes.rs`), and run by the daemon
//! (`sampler/jobs.rs`). The entries are in `entries.rs`.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

use super::entries;
use super::table::{self, Kind};
use crate::error::{Error, Kind as ErrorKind, Result};
use crate::sampler::jobs::Spec;

/// Check the arguments of job `name` and say what it runs.
pub fn build(name: &str, args: &Value) -> Result<Spec> {
    #[cfg(test)]
    if let Some(script) = name.strip_prefix("test.sh:") {
        return Ok(Spec {
            argv: vec!["sh".into(), "-c".into(), script.into()],
            lane: format!("test:{script}"),
            affects: Vec::new(),
            keep_stdin: true,
            stdin: args
                .get("stdin")
                .and_then(Value::as_str)
                .map(str::to_string),
        });
    }
    static RUNS: AtomicU64 = AtomicU64::new(0);
    let entry = entries::find(name)
        .filter(|entry| matches!(entry.kind, Kind::Job | Kind::Interactive))
        .ok_or_else(|| {
            Error::new(ErrorKind::BadRequest, format!("unknown job {name}")).with_context("name")
        })?;
    // A lane the entry names serialises its jobs (two package installs); with
    // none, every run has a lane of its own and nothing waits.
    let lane = if entry.lane.is_empty() {
        format!("job:{name}:{}", RUNS.fetch_add(1, Ordering::SeqCst))
    } else {
        entry.lane.to_string()
    };
    let mut affects: Vec<String> = crate::sampler::AFTER_JOB
        .iter()
        .map(|k| k.to_string())
        .collect();
    affects.push(super::SETTINGS.to_string());
    // A job can change anything the snapshot shows. An interactive tool
    // changes what its entry names (a theme picker the look, a recorder the
    // host), and a tool that only asks the user something changes nothing;
    // either way the topics are read again when it ends (docs/adr/0016).
    if entry.kind == Kind::Job {
        affects.extend(
            crate::snapshot::topics::NAMES
                .iter()
                .map(|topic| topic.to_string()),
        );
    } else {
        affects.extend(entry.affects.iter().map(|topic| topic.to_string()));
    }
    Ok(Spec {
        argv: table::argv(entry, args)?,
        lane,
        affects,
        keep_stdin: false,
        stdin: table::stdin(entry, args)?,
    })
}
