//! Which system ratmos is serving. A closed set, so a typo is a compile error
//! and a `match` has to cover every platform.

use serde_json::{json, Value};

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Omarchy,
    Plain,
}

impl Backend {
    pub fn parse(name: &str) -> Result<Backend> {
        match name {
            "omarchy" => Ok(Backend::Omarchy),
            "plain" => Ok(Backend::Plain),
            other => Err(Error::bad_request(format!("unknown backend {other}"))),
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Backend::Omarchy => "omarchy",
            Backend::Plain => "plain",
        }
    }

    pub fn info(self) -> Value {
        match self {
            Backend::Omarchy => {
                json!({"id": "omarchy", "compositor": "hyprland", "family": "arch"})
            }
            Backend::Plain => json!({"id": "plain", "compositor": "none", "family": "portable"}),
        }
    }

    /// Talking to the real machine: Omarchy, and not a fixture root. Only then
    /// do writes reload services and run live commands.
    pub fn is_live(self, root: Option<&std::path::Path>) -> bool {
        self == Backend::Omarchy && root.is_none()
    }
}

/// The app root: `ATMOS_ROOT`, else the nearest parent of the binary holding shell.qml.
pub fn app_root() -> std::path::PathBuf {
    use std::path::{Path, PathBuf};
    if let Ok(root) = std::env::var("ATMOS_ROOT") {
        if !root.is_empty() {
            return PathBuf::from(root);
        }
    }
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf));
    while let Some(current) = dir {
        if current.join("shell.qml").is_file() {
            return current;
        }
        dir = current.parent().map(Path::to_path_buf);
    }
    PathBuf::new()
}

/// A package manager owns the app when the packager left a PACKAGED marker in the app root.
pub fn packaged() -> bool {
    app_root().join("PACKAGED").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_names_and_rejects_others() {
        assert_eq!(Backend::parse("omarchy").unwrap(), Backend::Omarchy);
        assert_eq!(Backend::parse("plain").unwrap(), Backend::Plain);
        assert!(Backend::parse("windows").is_err());
    }

    #[test]
    fn only_omarchy_without_a_root_is_live() {
        let root = std::path::Path::new("/tmp/x");
        assert!(Backend::Omarchy.is_live(None));
        assert!(!Backend::Omarchy.is_live(Some(root)));
        assert!(!Backend::Plain.is_live(None));
    }

    #[test]
    fn info_names_the_platform() {
        assert_eq!(Backend::Plain.info()["id"], Backend::Plain.id());
    }
}
