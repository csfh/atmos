//! The session user, avatar path, human logins, and groups. No sudo.

use serde_json::{json, Value};
use std::path::Path;

const ALWAYS_GROUPS: &[&str] = &["wheel", "docker"];

fn human_uid(uid: i64) -> bool {
    (1000..65534).contains(&uid)
}

fn removable_gid(gid: i64, name: &str) -> bool {
    !ALWAYS_GROUPS.contains(&name) && (1000..65534).contains(&gid)
}

struct User {
    name: String,
    uid: i64,
    gid: i64,
    full_name: String,
    home: String,
    shell: String,
    current: bool,
}

struct Group {
    name: String,
    gid: i64,
    members: Vec<String>,
}

fn current_login(passwd: &str) -> String {
    for var in ["USER", "LOGNAME"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() {
                return v;
            }
        }
    }
    use std::os::unix::fs::MetadataExt;
    let uid = std::fs::metadata("/proc/self").map_or(-1, |m| i64::from(m.uid()));
    passwd
        .lines()
        .filter_map(|l| {
            let parts: Vec<&str> = l.trim().split(':').collect();
            (parts.len() >= 3 && parts[2].parse::<i64>().ok() == Some(uid))
                .then(|| parts[0].to_string())
        })
        .next()
        .unwrap_or_default()
}

fn avatar_path(home: &str, username: &str) -> String {
    let mut candidates = Vec::new();
    if !username.is_empty() {
        candidates.push(format!("/var/lib/AccountsService/icons/{username}"));
    }
    if !home.is_empty() {
        candidates.push(format!("{home}/.face.icon"));
        candidates.push(format!("{home}/.face"));
    }
    candidates
        .into_iter()
        .find(|p| Path::new(p).is_file())
        .unwrap_or_default()
}

fn parse_passwd(text: &str, current: &str) -> Vec<User> {
    let mut users = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            continue;
        }
        let (Ok(uid), Ok(gid)) = (parts[2].parse::<i64>(), parts[3].parse::<i64>()) else {
            continue;
        };
        if parts[0] != current && !human_uid(uid) {
            continue;
        }
        users.push(User {
            name: parts[0].into(),
            uid,
            gid,
            full_name: parts[4].split(',').next().unwrap_or("").into(),
            home: parts[5].into(),
            shell: parts[6].into(),
            current: parts[0] == current,
        });
    }
    users
}

fn parse_group(text: &str) -> Vec<Group> {
    let mut groups = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 4 {
            continue;
        }
        let Ok(gid) = parts[2].parse::<i64>() else {
            continue;
        };
        groups.push(Group {
            name: parts[0].into(),
            gid,
            members: parts[3]
                .split(',')
                .filter(|m| !m.is_empty())
                .map(String::from)
                .collect(),
        });
    }
    groups
}

pub fn collect() -> Value {
    let passwd = std::fs::read_to_string("/etc/passwd").unwrap_or_default();
    let group = std::fs::read_to_string("/etc/group").unwrap_or_default();
    build(&passwd, &group, &current_login(&passwd))
}

fn build(passwd: &str, group: &str, current: &str) -> Value {
    let users = parse_passwd(passwd, current);
    let groups = parse_group(group);
    let wheel: Vec<&String> = groups
        .iter()
        .filter(|g| g.name == "wheel")
        .flat_map(|g| &g.members)
        .collect();
    let humans: Vec<&str> = users.iter().map(|u| u.name.as_str()).collect();
    let visible: Vec<Value> = groups
        .iter()
        .filter(|g| {
            let always = ALWAYS_GROUPS.contains(&g.name.as_str());
            if !always && users.iter().any(|u| u.name == g.name && u.gid == g.gid) {
                return false;
            }
            always
                || removable_gid(g.gid, &g.name)
                || g.members.iter().any(|m| humans.contains(&m.as_str()))
        })
        .map(|g| json!({"name": g.name, "gid": g.gid, "members": g.members}))
        .collect();
    let home = users
        .iter()
        .find(|u| u.current)
        .map(|u| u.home.clone())
        .or_else(|| std::env::var("HOME").ok())
        .unwrap_or_default();
    json!({
        "currentUser": current,
        "avatarPath": avatar_path(&home, current),
        "users": users.iter().map(|u| json!({
            "name": u.name, "uid": u.uid, "gid": u.gid, "fullName": u.full_name,
            "home": u.home, "shell": u.shell,
            "wheel": wheel.iter().any(|m| **m == u.name),
            "current": u.current,
        })).collect::<Vec<_>>(),
        "groups": visible,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humans_and_their_groups_are_listed() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\nann:x:1000:1000:Ann Lee,,,:/home/ann:/bin/zsh\nnobody:x:65534:65534::/:/bin/false\n";
        let group = "root:x:0:\nwheel:x:998:ann\nann:x:1000:\nvideo:x:985:ann\nstaff:x:1001:\n";
        let got = build(passwd, group, "ann");
        assert_eq!(got["users"].as_array().unwrap().len(), 1);
        assert_eq!(got["users"][0]["fullName"], "Ann Lee");
        assert_eq!(got["users"][0]["wheel"], true);
        let names: Vec<_> = got["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| g["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["wheel", "video", "staff"]);
    }
}
