//! What the "Atmos backend unavailable" screen says, from the state of the `serve` process and of
//! the daemon behind it. The words are those of `unavailableNotice` in `services/BackendProtocol.js`.

use crate::client::Link;
use crate::protocol::Daemon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unavailable {
    pub title: &'static str,
    pub detail: String,
    pub status: String,
}

impl Unavailable {
    /// The detail and the status as the sentences they are: a reason from the backend has no full
    /// stop of its own.
    pub fn description(&self) -> String {
        format!("{}. {}", self.detail.trim_end_matches('.'), self.status)
    }
}

/// The notice to show instead of a page, or `None` when the backend is there. A server that is
/// still starting is available, and so is a daemon that has not yet said (`None`).
pub fn unavailable(link: &Link, daemon: Option<&Daemon>) -> Option<Unavailable> {
    const TITLE: &str = "Atmos backend unavailable";
    if matches!(link, Link::Down { .. }) {
        return Some(Unavailable {
            title: TITLE,
            detail: "The backend process stopped.".into(),
            status: "Starting it again.".into(),
        });
    }
    let Some(Daemon::Down { why, retry_ms, .. }) = daemon else {
        return None;
    };
    let retry = retry_ms.div_ceil(1000);
    Some(Unavailable {
        title: TITLE,
        detail: if why.is_empty() {
            "The Atmos daemon does not answer.".into()
        } else {
            why.clone()
        },
        status: if retry > 0 {
            format!("Starting it and looking again in {retry}s.")
        } else {
            "Starting it.".into()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn down(why: &str, retry_ms: u64) -> Daemon {
        Daemon::Down {
            why: why.into(),
            attempt: 2,
            retry_ms,
        }
    }

    #[test]
    fn a_backend_that_is_there_shows_no_notice() {
        assert_eq!(unavailable(&Link::Starting, None), None);
        assert_eq!(unavailable(&Link::Up, None), None);
        assert_eq!(unavailable(&Link::Up, Some(&Daemon::Up)), None);
    }

    #[test]
    fn a_stopped_server_says_it_is_starting_again() {
        let link = Link::Down {
            failures: 1,
            retry_ms: 1000,
        };
        let notice = unavailable(&link, Some(&Daemon::Up)).unwrap();
        assert_eq!(notice.title, "Atmos backend unavailable");
        assert_eq!(notice.detail, "The backend process stopped.");
        assert_eq!(notice.status, "Starting it again.");
    }

    #[test]
    fn a_missing_daemon_says_why_and_when_it_looks_again() {
        let notice = unavailable(&Link::Up, Some(&down("no daemon (refused)", 2000))).unwrap();
        assert_eq!(notice.detail, "no daemon (refused)");
        assert_eq!(notice.status, "Starting it and looking again in 2s.");
    }

    #[test]
    fn the_description_reads_as_two_sentences() {
        let stopped = unavailable(
            &Link::Down {
                failures: 1,
                retry_ms: 1000,
            },
            None,
        )
        .unwrap();
        assert_eq!(
            stopped.description(),
            "The backend process stopped. Starting it again."
        );
        let reason = unavailable(&Link::Up, Some(&down("no daemon (refused)", 2000))).unwrap();
        assert_eq!(
            reason.description(),
            "no daemon (refused). Starting it and looking again in 2s."
        );
    }

    #[test]
    fn a_missing_reason_and_delay_still_read() {
        let notice = unavailable(&Link::Up, Some(&down("", 0))).unwrap();
        assert_eq!(notice.detail, "The Atmos daemon does not answer.");
        assert_eq!(notice.status, "Starting it.");
        assert_eq!(
            unavailable(&Link::Up, Some(&down("x", 1500)))
                .unwrap()
                .status,
            "Starting it and looking again in 2s.",
            "a part of a second rounds up"
        );
    }
}
