//! The requests ratmos understands. One enum is the whole protocol: a
//! request names an `op` and carries the fields that op needs, and serde
//! rejects an unknown op or a missing or mistyped field before any handler
//! runs.

use serde::Deserialize;
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
#[serde(tag = "op")]
pub enum Request {
    #[serde(rename = "version")]
    Version,
    #[serde(rename = "platform")]
    Platform,
    #[serde(rename = "settings.list")]
    SettingsList,
    #[serde(rename = "settings.get")]
    SettingsGet { domain: String },
    #[serde(rename = "settings.set")]
    SettingsSet { domain: String, value: Value },
    #[serde(rename = "settings.snapshot")]
    SettingsSnapshot {
        #[serde(default = "all_groups")]
        group: String,
        /// Only read the domains whose top-level name is listed. Absent means
        /// every domain.
        #[serde(default)]
        keys: Option<Vec<String>>,
    },
    #[serde(rename = "display.get")]
    DisplayGet { kind: String },
    #[serde(rename = "display.snapshot")]
    DisplaySnapshot,
    #[serde(rename = "host.chrome")]
    HostChrome,
    #[serde(rename = "host.themePack")]
    HostThemePack(ThemePack),
    #[serde(rename = "host.accounts")]
    HostAccounts(Accounts),
    #[serde(rename = "host.stamp")]
    HostStamp(Paths),
    #[serde(rename = "host.read")]
    HostRead(Paths),
    #[serde(rename = "host.write")]
    HostWrite(WriteFile),
    #[serde(rename = "host.open")]
    HostOpen { path: String },
    #[serde(rename = "speedtest.disk")]
    SpeedtestDisk(SpeedDisk),
    #[serde(rename = "speedtest.net")]
    SpeedtestNet(SpeedNet),
    #[serde(rename = "unit.output")]
    UnitOutput(UnitOutput),
    #[serde(rename = "agents.mcp.list")]
    AgentsMcpList,
    #[serde(rename = "agents.mcp.set")]
    AgentsMcpSet {
        agent: String,
        on: bool,
        #[serde(default)]
        replace: bool,
    },
    #[serde(rename = "agents.mcp.check")]
    AgentsMcpCheck,
}

fn all_groups() -> String {
    "all".into()
}

fn down() -> String {
    "down".into()
}

fn system() -> String {
    "system".into()
}

#[derive(Debug, Deserialize)]
pub struct ThemePack {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub home: String,
    #[serde(default)]
    pub part: String,
}

#[derive(Debug, Deserialize)]
pub struct Accounts {
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub home: String,
}

#[derive(Debug, Deserialize)]
pub struct Paths {
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct WriteFile {
    pub path: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct SpeedDisk {
    #[serde(default)]
    pub dir: String,
}

#[derive(Debug, Deserialize)]
pub struct SpeedNet {
    #[serde(default = "down")]
    pub phase: String,
}

#[derive(Debug, Deserialize)]
pub struct UnitOutput {
    pub kind: String,
    #[serde(default = "system")]
    pub scope: String,
    pub unit: String,
}

impl Request {
    /// Parse a request. A bad one is a `bad_request` that names the problem.
    pub fn parse(value: Value) -> Result<Request> {
        serde_json::from_value(value).map_err(|err| Error::bad_request(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: Value) -> Result<Request> {
        Request::parse(value)
    }

    #[test]
    fn a_known_op_parses_with_its_fields() {
        match parse(json!({"op": "settings.get", "domain": "theme"})).unwrap() {
            Request::SettingsGet { domain } => assert_eq!(domain, "theme"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn snapshot_defaults_to_the_whole_snapshot() {
        match parse(json!({"op": "settings.snapshot"})).unwrap() {
            Request::SettingsSnapshot { group, keys } => {
                assert_eq!(group, "all");
                assert!(keys.is_none());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_missing_field_is_named() {
        let err = parse(json!({"op": "settings.get"})).unwrap_err();
        assert!(err.message.contains("domain"), "{err}");
    }

    #[test]
    fn an_unknown_op_is_rejected() {
        let err = parse(json!({"op": "nope"})).unwrap_err();
        assert!(err.message.contains("nope"), "{err}");
    }

    #[test]
    fn a_wrong_type_is_rejected() {
        assert!(parse(json!({"op": "host.read", "paths": "a"})).is_err());
        assert!(parse(json!({"op": "host.stamp", "paths": [1]})).is_err());
    }

    #[test]
    fn extra_fields_are_ignored() {
        assert!(parse(json!({"op": "host.chrome", "future": true})).is_ok());
    }

    #[test]
    fn optional_host_fields_default() {
        match parse(json!({"op": "speedtest.net"})).unwrap() {
            Request::SpeedtestNet(net) => assert_eq!(net.phase, "down"),
            other => panic!("{other:?}"),
        }
    }
}
