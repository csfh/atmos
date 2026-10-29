//! The Accounts page's logic (`pages/AccountsPage.qml`, `services/Accounts.js`): the names a
//! login, a group and a full name may be, what each user's and group's row says, the four
//! dialogs' checks, and what they send (`Omarchy.addAccountUser`, `setAccountPassword`,
//! `addAccountGroup`, `removeAccountUser`, `removeAccountGroup`, `setGroupMember`, `setFullName`,
//! `setAvatarPath`, `clearAvatar`). Every account change runs as root. `tests/catalog_accounts.rs`
//! runs `Accounts.js` and the page's code under Node against this.

use serde_json::{Value, json};

use crate::catalog::Field;
use crate::jobs::{Job, RootWrite};
use crate::js;
use crate::protocol;

pub const USERS: Field = Field::Key("users");
pub const GROUPS: Field = Field::Key("groups");
pub const CURRENT: Field = Field::Key("currentUser");
pub const AVATAR: Field = Field::Key("avatarPath");
pub const FULL_NAME: Field = Field::Key("fullName");

const RESERVED: &[&str] = &[
    "root",
    "bin",
    "daemon",
    "mail",
    "ftp",
    "http",
    "nobody",
    "dbus",
    "systemd-coredump",
    "systemd-network",
    "systemd-oom",
    "systemd-journal-remote",
    "systemd-resolve",
    "systemd-timesync",
    "tss",
    "uuidd",
    "alpm",
    "git",
    "avahi",
    "cups",
    "cups-browsed",
    "lp",
    "_talkd",
    "polkitd",
    "rtkit",
    "qemu",
    "brltty",
    "gluster",
    "rpc",
    "libvirt-qemu",
    "pcscd",
    "nvidia-persistenced",
    "sddm",
];

fn text(v: Option<&Value>) -> String {
    v.filter(|x| js::truthy(Some(x)))
        .map_or_else(String::new, js::string)
}

/// A login's or a group's name: lowercase, a letter or `_` first, at most 32, no `-` last.
fn name_shape(s: &str) -> bool {
    let mut chars = s.chars();
    (1..=32).contains(&s.len())
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        && !s.ends_with('-')
}

/// `parseUsername`: the name trimmed, or "" for one that is not a login's.
pub fn parse_username(raw: &str) -> String {
    let s = raw.trim();
    if name_shape(s) && !RESERVED.contains(&s) {
        s.to_owned()
    } else {
        String::new()
    }
}

/// `parseGroupName`.
pub fn parse_group_name(raw: &str) -> String {
    let s = raw.trim();
    if name_shape(s) && !matches!(s, "root" | "nobody" | "nogroup") {
        s.to_owned()
    } else {
        String::new()
    }
}

/// `isFullName`: at most 256, no `:`, `,` or line break, no `-` first.
pub fn is_full_name(raw: &str) -> bool {
    let s = raw.trim();
    s.encode_utf16().count() <= 256 && !s.starts_with('-') && !s.contains([':', '\n', '\r', ','])
}

/// `isRemovableGid(gid, name)`.
pub fn removable_gid(gid: Option<&Value>, name: &str) -> bool {
    if matches!(name, "wheel" | "docker") {
        return false;
    }
    let n = js::round(js::number(gid));
    n.is_finite() && (1000.0..65534.0).contains(&n)
}

/// `groupHasMember(group, user)`.
pub fn has_member(group: Option<&Value>, user: &str) -> bool {
    !user.is_empty()
        && group
            .and_then(|g| g.get("members"))
            .and_then(Value::as_array)
            .is_some_and(|m| m.iter().any(|x| x == user))
}

/// `groupByName(name)`.
pub fn group<'a>(groups: &'a [Value], name: &str) -> Option<&'a Value> {
    groups
        .iter()
        .find(|g| g.get("name").and_then(Value::as_str) == Some(name))
}

/// `memberLocked(group, row)`: this session cannot leave wheel.
pub fn member_locked(groups: &[Value], group_name: &str, row: &Value) -> bool {
    group_name == "wheel"
        && row.get("current").is_some_and(|c| js::truthy(Some(c)))
        && has_member(group(groups, "wheel"), &text(row.get("name")))
}

/// `userBlurb(row)`.
pub fn user_says(row: &Value) -> String {
    let mut bits = Vec::new();
    let full = text(row.get("fullName"));
    if !full.is_empty() {
        bits.push(full);
    }
    bits.push(format!(
        "uid {}",
        row.get("uid").map_or("undefined".into(), js::string)
    ));
    if js::truthy(row.get("wheel")) {
        bits.push("admin".into());
    }
    if js::truthy(row.get("current")) {
        bits.push("this session".into());
    }
    bits.join(" · ")
}

/// `groupBlurb(row)`.
pub fn group_says(row: &Value) -> String {
    let n = row
        .get("members")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let gid = row
        .get("gid")
        .filter(|g| !g.is_null())
        .map_or_else(String::new, js::string);
    if n == 1 {
        format!("gid {gid} · 1 member")
    } else {
        format!("gid {gid} · {n} members")
    }
}

/// The member row's words in Manage's dialog.
pub fn member_says(groups: &[Value], group_name: &str, row: &Value) -> String {
    if member_locked(groups, group_name, row) {
        "This session has to stay in wheel.".into()
    } else {
        user_says(row)
    }
}

/// Manage's dialog's first line.
pub fn manage_says(group_name: &str) -> &'static str {
    if group_name == "wheel" {
        "Add or remove logins in wheel. This session has to stay."
    } else {
        "Add or remove logins in this group."
    }
}

/// `faceRowDescription(path)`.
pub fn face_says(path: &str) -> String {
    if path.is_empty() {
        "No face is set. Choose a PNG or JPEG. Portrait cards on the login screen draw it; AccountsService and ~/.face.icon keep the file.".into()
    } else {
        format!("{path}. Portrait cards on the login screen draw this face.")
    }
}

/// The Full name row's words, by what is typed.
pub fn full_name_says(draft: &str) -> &'static str {
    if draft.is_empty() || is_full_name(draft) {
        "The name on this account. Login screens and user lists show it."
    } else {
        "No colon, comma, or leading hyphen (GECOS splits on comma). Set stays off until the name is valid."
    }
}

/// `setFullName(name)`.
pub fn full_name_write(name: &str, current: &str) -> Option<Value> {
    let name = name.trim();
    (name != current && is_full_name(name))
        .then(|| crate::settings::write("fullName", &Value::from(name)))
}

/// What the Add a user dialog holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewUser {
    pub name: String,
    pub full: String,
    pub password: String,
    pub confirm: String,
    pub admin: bool,
}

impl NewUser {
    /// `submitAddUser()`: the job, or what is wrong.
    pub fn submit(&self, users: &[Value]) -> Result<Option<Job>, String> {
        let name = parse_username(&self.name);
        let full = self.full.trim();
        if name.is_empty() {
            return Err("Username must be lowercase, start with a letter or underscore, and stay under 32 characters.".into());
        }
        if users
            .iter()
            .any(|u| u.get("name").and_then(Value::as_str) == Some(name.as_str()))
        {
            return Err(format!("{name} already exists."));
        }
        if !is_full_name(full) {
            return Err("Full name cannot contain a colon or comma.".into());
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty.".into());
        }
        if self.password != self.confirm {
            return Err("Passwords do not match.".into());
        }
        Ok(add_user(&name, full, &self.password, self.admin))
    }
}

/// `addAccountUser(name, full, password, wheel)`.
pub fn add_user(name: &str, full: &str, password: &str, wheel: bool) -> Option<Job> {
    let name = parse_username(name);
    let full = full.trim();
    let ok =
        !name.is_empty() && !password.is_empty() && !password.contains('\n') && is_full_name(full);
    ok.then(|| {
        Job::new(
            "account.add-user",
            json!({ "name": name, "full": full, "wheel": wheel, "password": password }),
            "account-add",
            "The new login",
        )
        .sudo()
    })
}

/// `submitPassword()`: the job, or what is wrong.
pub fn submit_password(
    user: &str,
    password: &str,
    confirm: &str,
) -> Result<Option<Job>, &'static str> {
    if password.is_empty() {
        return Err("Password cannot be empty.");
    }
    if password != confirm {
        return Err("Passwords do not match.");
    }
    Ok(set_password(user, password))
}

/// `setAccountPassword(name, password)`.
pub fn set_password(name: &str, password: &str) -> Option<Job> {
    let name = parse_username(name);
    (!name.is_empty() && !password.is_empty() && !password.contains('\n')).then(|| {
        Job::new(
            "account.set-password",
            json!({ "name": name, "password": password }),
            "account-password",
            "The password change",
        )
        .sudo()
    })
}

/// `submitAddGroup()`: the job, or what is wrong.
pub fn submit_group(raw: &str, groups: &[Value]) -> Result<Option<Job>, String> {
    let name = parse_group_name(raw);
    if name.is_empty() {
        return Err("Group name must be lowercase, start with a letter or underscore, and stay under 32 characters.".into());
    }
    if group(groups, &name).is_some() {
        return Err(format!("{name} already exists."));
    }
    Ok(add_group(&name))
}

/// `addAccountGroup(name)`.
pub fn add_group(name: &str) -> Option<Job> {
    let name = parse_group_name(name);
    (!name.is_empty()).then(|| {
        Job::new(
            "account.add-group",
            json!({ "name": name }),
            "account-group-add",
            "The new group",
        )
        .sudo()
    })
}

/// `removeAccountUser(name)`.
pub fn remove_user(name: &str, current: &str) -> Option<Job> {
    let name = parse_username(name);
    (!name.is_empty() && name != current).then(|| {
        Job::new(
            "account.remove-user",
            json!({ "name": name }),
            "account-remove",
            "The login removal",
        )
        .sudo()
    })
}

/// `removeAccountGroup(name)`.
pub fn remove_group(name: &str) -> Option<Job> {
    let name = parse_group_name(name);
    (!name.is_empty() && name != "wheel" && name != "docker").then(|| {
        Job::new(
            "account.remove-group",
            json!({ "name": name }),
            "account-group-remove",
            "The group removal",
        )
        .sudo()
    })
}

fn root_write(key: &'static str, body: Value, refused: &str) -> RootWrite {
    RootWrite {
        key,
        body,
        refused: refused.into(),
    }
}

/// `setGroupMember(group, name, on)`: a write that runs as root.
pub fn member_write(group: &str, name: &str, on: bool, current: &str) -> Option<RootWrite> {
    let (group, name) = (parse_group_name(group), parse_username(name));
    if group.is_empty() || name.is_empty() || (!on && group == "wheel" && name == current) {
        return None;
    }
    let body = protocol::write(
        "account.member",
        &json!({ "group": group, "user": name, "on": on }),
    );
    Some(root_write("groups", body, "The group was not changed"))
}

/// `setAvatarPath(path)`.
pub fn avatar_write(path: &str, current: &str) -> Option<RootWrite> {
    let ok = !current.is_empty() && path.starts_with('/') && !path.contains("..");
    ok.then(|| {
        let body = protocol::write("avatar.set", &json!({ "user": current, "path": path }));
        root_write("avatarPath", body, "The face was not changed")
    })
}

/// `clearAvatar()`.
pub fn avatar_clear(current: &str) -> Option<RootWrite> {
    (!current.is_empty()).then(|| {
        let body = protocol::write("avatar.clear", &json!({ "user": current }));
        root_write("avatarPath", body, "The face was not cleared")
    })
}
