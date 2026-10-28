//! The requests atmosd understands. One enum is the whole protocol: a
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
    SettingsSet {
        domain: String,
        value: Value,
        /// Snapshot keys to read back once the value is written, for the keys
        /// the setting changes besides its own.
        #[serde(default)]
        readback: Vec<String>,
    },
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
    #[serde(rename = "snapshot.keys")]
    SnapshotKeys {
        keys: Vec<String>,
        /// Values the keys must show before they are returned, for a change
        /// the machine finishes after its command exits. Waited for briefly.
        #[serde(default)]
        expect: serde_json::Map<String, Value>,
    },
    /// One settings topic, read from the machine now. The daemon starts this as
    /// a child with a window's environment to fill a topic (docs/adr/0011).
    #[serde(rename = "snapshot.topic")]
    SnapshotTopic { topic: String },
    /// Read some settings topics again; none named means all of them.
    #[serde(rename = "snapshot.invalidate")]
    SnapshotInvalidate {
        #[serde(default)]
        topics: Vec<String>,
    },
    #[serde(rename = "write")]
    Write {
        name: String,
        #[serde(default)]
        args: Value,
    },
    #[serde(rename = "display.snapshot")]
    DisplaySnapshot,
    #[serde(rename = "display.brightness")]
    DisplayBrightness { monitor: String, percent: u32 },
    #[serde(rename = "display.scale")]
    DisplayScale { scale: String },
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
    #[serde(rename = "host.image")]
    HostImage { which: String },
    #[serde(rename = "wifi.scan")]
    WifiScan,
    #[serde(rename = "bluetooth.scan")]
    BluetoothScan(BluetoothScan),
    #[serde(rename = "search.query")]
    SearchQuery {
        #[serde(default)]
        query: String,
        #[serde(flatten)]
        context: crate::search::Context,
    },
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
pub struct BluetoothScan {
    /// How long the discovery runs, 1 to 30 seconds.
    #[serde(default = "discovery_seconds")]
    pub seconds: u64,
}

fn discovery_seconds() -> u64 {
    crate::radios::DISCOVERY_SECONDS
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
    use crate::error::Kind;
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

    /// Exhaustive on purpose: a new `Request` variant stops compiling here
    /// until it gets an op name, and `every_variant_has_a_round_trip_case`
    /// then demands a body for it.
    fn op_name(request: &Request) -> &'static str {
        match request {
            Request::Version => "version",
            Request::Platform => "platform",
            Request::SettingsList => "settings.list",
            Request::SettingsGet { .. } => "settings.get",
            Request::SettingsSet { .. } => "settings.set",
            Request::SettingsSnapshot { .. } => "settings.snapshot",
            Request::DisplayGet { .. } => "display.get",
            Request::DisplaySnapshot => "display.snapshot",
            Request::SnapshotKeys { .. } => "snapshot.keys",
            Request::SnapshotTopic { .. } => "snapshot.topic",
            Request::SnapshotInvalidate { .. } => "snapshot.invalidate",
            Request::Write { .. } => "write",
            Request::DisplayBrightness { .. } => "display.brightness",
            Request::DisplayScale { .. } => "display.scale",
            Request::SearchQuery { .. } => "search.query",
            Request::HostChrome => "host.chrome",
            Request::HostThemePack(_) => "host.themePack",
            Request::HostAccounts(_) => "host.accounts",
            Request::HostStamp(_) => "host.stamp",
            Request::HostRead(_) => "host.read",
            Request::HostWrite(_) => "host.write",
            Request::HostOpen { .. } => "host.open",
            Request::SpeedtestDisk(_) => "speedtest.disk",
            Request::SpeedtestNet(_) => "speedtest.net",
            Request::UnitOutput(_) => "unit.output",
            Request::HostImage { .. } => "host.image",
            Request::WifiScan => "wifi.scan",
            Request::BluetoothScan(_) => "bluetooth.scan",
            Request::AgentsMcpList => "agents.mcp.list",
            Request::AgentsMcpSet { .. } => "agents.mcp.set",
            Request::AgentsMcpCheck => "agents.mcp.check",
        }
    }

    /// The smallest valid body for every op.
    fn minimal_bodies() -> Vec<Value> {
        vec![
            json!({"op": "version"}),
            json!({"op": "platform"}),
            json!({"op": "settings.list"}),
            json!({"op": "settings.get", "domain": "theme"}),
            json!({"op": "settings.set", "domain": "theme", "value": "x"}),
            json!({"op": "settings.snapshot"}),
            json!({"op": "display.get", "kind": "monitors"}),
            json!({"op": "display.snapshot"}),
            json!({"op": "snapshot.keys", "keys": ["theme"]}),
            json!({"op": "snapshot.topic", "topic": "settings.look"}),
            json!({"op": "snapshot.invalidate"}),
            json!({"op": "write", "name": "plugin.set", "args": {"id": "p", "on": true}}),
            json!({"op": "display.brightness", "monitor": "DP-3", "percent": 40}),
            json!({"op": "display.scale", "scale": "1.5"}),
            json!({"op": "search.query", "query": "x"}),
            json!({"op": "host.chrome"}),
            json!({"op": "host.themePack"}),
            json!({"op": "host.accounts"}),
            json!({"op": "host.stamp", "paths": []}),
            json!({"op": "host.read", "paths": ["a"]}),
            json!({"op": "host.write", "path": "a"}),
            json!({"op": "host.open", "path": "a"}),
            json!({"op": "speedtest.disk"}),
            json!({"op": "speedtest.net"}),
            json!({"op": "unit.output", "kind": "status", "unit": "sshd.service"}),
            json!({"op": "host.image", "which": "avatar"}),
            json!({"op": "wifi.scan"}),
            json!({"op": "bluetooth.scan"}),
            json!({"op": "agents.mcp.list"}),
            json!({"op": "agents.mcp.set", "agent": "claude", "on": true}),
            json!({"op": "agents.mcp.check"}),
        ]
    }

    #[test]
    fn every_variant_round_trips_through_its_op_name() {
        for body in minimal_bodies() {
            let want = body["op"].as_str().unwrap().to_string();
            let request = parse(body.clone()).unwrap_or_else(|e| panic!("{body}: {e}"));
            assert_eq!(op_name(&request), want, "{body}");
        }
    }

    #[test]
    fn every_variant_has_a_round_trip_case() {
        // Count the `rename` attributes above this module and compare with the
        // table, so a new variant cannot ship without a body here.
        let source = include_str!("request.rs");
        let (declared, _) = source.split_once("#[cfg(test)]").unwrap();
        let marker = ["rename = ", "\""].concat();
        assert_eq!(declared.matches(&marker).count(), minimal_bodies().len());
        let mut ops: Vec<_> = minimal_bodies()
            .iter()
            .map(|b| b["op"].as_str().unwrap().to_string())
            .collect();
        ops.sort();
        ops.dedup();
        assert_eq!(ops.len(), minimal_bodies().len(), "duplicate op in table");
    }

    #[test]
    fn every_op_that_needs_a_field_names_it_when_missing() {
        let needs = [
            ("settings.get", "domain"),
            ("settings.set", "domain"),
            ("display.get", "kind"),
            ("host.stamp", "paths"),
            ("host.read", "paths"),
            ("host.write", "path"),
            ("host.open", "path"),
            ("unit.output", "kind"),
            ("host.image", "which"),
            ("agents.mcp.set", "agent"),
        ];
        for (op, field) in needs {
            let err = parse(json!({ "op": op })).unwrap_err();
            assert_eq!(err.kind, Kind::BadRequest, "{op}");
            assert!(err.message.contains(field), "{op}: {err}");
        }
    }

    #[test]
    fn every_parse_failure_is_a_bad_request() {
        let bad = [
            json!({}),
            json!({"op": 1}),
            json!({"op": null}),
            json!("settings.list"),
            json!([]),
            json!(null),
            json!({"op": "nope"}),
            json!({"op": "settings.get", "domain": 7}),
            json!({"op": "agents.mcp.set", "agent": "a", "on": "yes"}),
        ];
        for body in bad {
            let err = parse(body.clone()).unwrap_err();
            assert_eq!(err.kind, Kind::BadRequest, "{body}");
            assert_eq!(err.kind.code(), "bad_request");
        }
    }

    #[test]
    fn op_names_are_case_sensitive() {
        assert!(parse(json!({"op": "Settings.List"})).is_err());
        assert!(parse(json!({"op": "host.themepack"})).is_err());
    }

    #[test]
    fn field_defaults_match_the_wire_contract() {
        match parse(json!({"op": "unit.output", "kind": "k", "unit": "u"})).unwrap() {
            Request::UnitOutput(u) => assert_eq!(u.scope, "system"),
            other => panic!("{other:?}"),
        }
        match parse(json!({"op": "agents.mcp.set", "agent": "a", "on": false})).unwrap() {
            Request::AgentsMcpSet { on, replace, .. } => assert!(!on && !replace),
            other => panic!("{other:?}"),
        }
        match parse(json!({"op": "host.write", "path": "p"})).unwrap() {
            Request::HostWrite(w) => assert_eq!((w.path.as_str(), w.text.as_str()), ("p", "")),
            other => panic!("{other:?}"),
        }
        match parse(json!({"op": "host.accounts"})).unwrap() {
            Request::HostAccounts(a) => assert!(a.user.is_empty() && a.home.is_empty()),
            other => panic!("{other:?}"),
        }
        match parse(json!({"op": "host.themePack"})).unwrap() {
            Request::HostThemePack(t) => assert!(t.name.is_empty() && t.part.is_empty()),
            other => panic!("{other:?}"),
        }
        match parse(json!({"op": "speedtest.disk"})).unwrap() {
            Request::SpeedtestDisk(d) => assert!(d.dir.is_empty()),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn snapshot_keys_are_carried_through() {
        let body = json!({"op": "settings.snapshot", "group": "look", "keys": ["a", "b"]});
        match parse(body).unwrap() {
            Request::SettingsSnapshot { group, keys } => {
                assert_eq!(group, "look");
                assert_eq!(keys.unwrap(), vec!["a", "b"]);
            }
            other => panic!("{other:?}"),
        }
    }
}
