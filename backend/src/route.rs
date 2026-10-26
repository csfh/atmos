//! Where a request runs: the daemon's lanes for a write, a lease around a
//! terminal job, and the read-again ask for the settings topics. A window's
//! `serve` has no other way to run a write; a terminal or a fixture root runs
//! it in its own process (docs/adr/0010).

use std::path::Path;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::platform::Backend;
use crate::{command, error, place_for, read_domain, sampler, snapshot, write_domain};

/// `snapshot.invalidate`: the daemon reads these topics again. A window has
/// nothing else to ask; a terminal or a fixture has no daemon cache to refresh.
pub(crate) fn invalidate_topics(root: Option<&Path>, topics: &[String]) -> Result<Value> {
    if root.is_some() {
        return Ok(serde_json::json!({"invalidated": []}));
    }
    let ask = serde_json::json!({"op": "invalidate", "topics": topics});
    match sampler::client::call(&ask, std::time::Duration::from_secs(5)) {
        Some(reply) if reply["ok"].as_bool() == Some(true) => {
            Ok(serde_json::json!({"invalidated": topics}))
        }
        Some(reply) => Err(Error::new(
            error::Kind::Failed,
            reply["message"].as_str().unwrap_or("not invalidated"),
        )),
        None if sampler::client::required() => Err(Error::unavailable("refresh")),
        None => Ok(serde_json::json!({"invalidated": []})),
    }
}

/// `settings.set`: the value is written, then read back with whatever else the
/// setting changed, as the machine now reports it.
pub(crate) fn set_domain(
    backend: Backend,
    root: Option<&Path>,
    domain: &str,
    value: &Value,
    readback: &[String],
) -> Result<Value> {
    write_domain(backend, root, domain, value)?;
    // A setting can start or stop a unit; the other inventories do not follow settings.
    if root.is_none() {
        sampler::drop_kinds(&["services"]);
    }
    let place = place_for(backend, domain)?;
    let mut reply = serde_json::json!({
        "domain": domain,
        "value": read_domain(backend, root, domain)?,
        "file": place.rel,
        "encoding": place.encoding(),
    });
    if root.is_none() && !readback.is_empty() {
        // Read as the topic is collected, so the reply and the next pushed sample agree.
        let read = snapshot::topics::read_keys(readback);
        reply["snapshot"] = Value::Object(read.values);
        if !read.failed.is_empty() {
            reply["failed"] = serde_json::json!(read.failed);
        }
    }
    Ok(reply)
}

/// `ratmos task` run from a terminal holds a lane of its own while it runs, so
/// a read of what it changes waits for it instead of racing it. (The window's
/// jobs run in the daemon and are not leased.) Its own lane: jobs never queue behind each other.
/// Without a daemon the job just runs.
pub(crate) fn job_lease() -> Option<sampler::client::Lease> {
    sampler::client::lease(
        &format!("job:{}", std::process::id()),
        sampler::AFTER_JOB,
        std::time::Duration::from_millis(500),
    )
}

/// The `generic` lane: `apply -- <argv>` is a write the daemon knows nothing
/// about, so writes through it go one at a time across windows and, like any
/// job, hold back reads of what a job can change. A lane that stays busy for
/// longer than a moment (a long streamed job holds it) is not waited for: the
/// write runs unordered rather than stall behind it.
pub(crate) fn generic_lease() -> Option<sampler::client::Lease> {
    sampler::client::lease(
        "generic",
        sampler::AFTER_JOB,
        std::time::Duration::from_millis(300),
    )
}

/// A typed command goes to the daemon's lane for it and comes back with the
/// answer of the command that ran (see `command`). A window's `serve` has no
/// other way to run it; a terminal, a fixture root, or the child the daemon
/// started for it runs it here.
pub(crate) fn route_command(
    backend: Backend,
    root: Option<&Path>,
    direct: bool,
    body: &Value,
    inline: impl FnOnce() -> Result<Value>,
) -> Result<Value> {
    if root.is_some() || direct {
        return inline();
    }
    match ask_daemon(backend, request_of(body), body) {
        Some(answer) => answer,
        None if sampler::client::required() => Err(Error::unavailable("run the write")),
        None => inline(),
    }
}

fn request_of(body: &Value) -> Option<crate::request::Request> {
    crate::request::Request::parse(body.clone()).ok()
}

/// Ask the daemon to run `body` in a child that carries this process's
/// environment. None means nothing is listening, so nothing was queued. A write
/// comes back flagged `coalesced`; a read is the answer and nothing else.
fn ask_daemon(
    backend: Backend,
    request: Option<crate::request::Request>,
    body: &Value,
) -> Option<Result<Value>> {
    Some(match sampler::client::command(body, backend.id()) {
        sampler::client::Command::Unavailable => return None,
        sampler::client::Command::Lost(message) => Err(Error::new(error::Kind::Failed, message)),
        sampler::client::Command::Done {
            envelope,
            coalesced,
        } => {
            if envelope["ok"].as_bool() != Some(true) {
                return Some(Err(command::error_of(&envelope)));
            }
            let mut result = envelope["result"].clone();
            let is_write = request.as_ref().is_some_and(|r| command::plan(r).is_some());
            if let (true, Some(map)) = (is_write, result.as_object_mut()) {
                map.insert("coalesced".into(), Value::Bool(coalesced));
            }
            Ok(result)
        }
    })
}

/// What a window's `serve` does with a request it has parsed (docs/adr/0012):
/// the daemon answers it, in a child with the window's environment, except for
/// the few that only ask the daemon something it already holds.
pub(crate) fn for_the_daemon(request: &crate::request::Request) -> bool {
    use crate::request::Request;
    !matches!(
        request,
        Request::DisplayGet { .. } | Request::DisplaySnapshot | Request::SnapshotInvalidate { .. }
    )
}

/// A window's `serve` hands the request to the daemon and relays its answer.
pub(crate) fn forward(
    backend: Backend,
    request: crate::request::Request,
    body: &Value,
) -> Result<Value> {
    ask_daemon(backend, Some(request), body)
        .unwrap_or_else(|| Err(Error::unavailable("answer the request")))
}
