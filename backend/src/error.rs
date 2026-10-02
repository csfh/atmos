//! One error type for the whole backend. Every failure carries a kind that
//! callers can branch on and a message a person can read, so nothing has to
//! sniff text such as "permission denied" out of a string.

use std::fmt;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The request was malformed or named something that does not exist.
    BadRequest,
    NotFound,
    /// The OS or the path rules refused the access.
    Denied,
    Io,
    /// A child process failed or could not start.
    Command,
    Timeout,
    /// Anything that does not fit a more specific kind.
    Failed,
}

impl Kind {
    pub fn code(self) -> &'static str {
        match self {
            Kind::BadRequest => "bad_request",
            Kind::NotFound => "not_found",
            Kind::Denied => "denied",
            Kind::Io => "io",
            Kind::Command => "command",
            Kind::Timeout => "timeout",
            Kind::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: Kind,
    pub message: String,
    /// The file, command or field the failure is about.
    pub context: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(kind: Kind, message: impl Into<String>) -> Self {
        Error {
            kind,
            message: message.into(),
            context: None,
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Error::new(Kind::BadRequest, message)
    }

    pub fn denied(message: impl Into<String>) -> Self {
        Error::new(Kind::Denied, message)
    }

    pub fn command(message: impl Into<String>) -> Self {
        Error::new(Kind::Command, message)
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// An I/O failure that remembers which path it was about.
    pub fn io(err: io::Error, path: &Path) -> Self {
        let mut out = Error::from(err);
        out.context = Some(path.display().to_string());
        out
    }

    pub fn is_denied(&self) -> bool {
        self.kind == Kind::Denied
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.context {
            Some(context) => write!(f, "{}: {}", context, self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        let kind = match err.kind() {
            io::ErrorKind::PermissionDenied => Kind::Denied,
            io::ErrorKind::NotFound => Kind::NotFound,
            io::ErrorKind::TimedOut => Kind::Timeout,
            _ => Kind::Io,
        };
        Error::new(kind, err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::bad_request(err.to_string())
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::new(Kind::Failed, message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error::new(Kind::Failed, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_kinds_map_to_error_kinds() {
        let denied = Error::from(io::Error::from(io::ErrorKind::PermissionDenied));
        assert!(denied.is_denied());
        let missing = Error::from(io::Error::from(io::ErrorKind::NotFound));
        assert_eq!(missing.kind, Kind::NotFound);
    }

    #[test]
    fn context_leads_the_message() {
        let err = Error::from("boom").with_context("/etc/x");
        assert_eq!(err.to_string(), "/etc/x: boom");
    }
}
