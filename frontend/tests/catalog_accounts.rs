//! The Accounts page (`pages/accounts.rs`): the names `Accounts.js` accepts, what each user's,
//! group's and member's row says, and the dialogs' checks are the QML page's own code, run under
//! Node; the lines the page module follows are pinned. What each setter sends is
//! `catalog_setters.rs`'s. Without `node` the test says so and stops.

mod common;

use atmos_ui::accounts::{self, NewUser};
use atmos_ui::software::job_body;
use common::qml;
use serde_json::{Value, json};

const NAMES: &[&str] = &[
    "ana",
    " bo ",
    "Root",
    "root",
    "_x",
    "1a",
    "a-",
    "a-b_c",
    "",
    "nogroup",
    "sddm",
    "toolongtoolongtoolongtoolongtoolong",
];
const FULL: &[&str] = &["Ana B", "a,b", "a:b", "-x", "", " x ", "a\nb"];

fn users() -> Value {
    json!([
        {"name": "me", "uid": 1000, "fullName": "Me Myself", "wheel": true, "current": true, "home": "/home/me"},
        {"name": "ana", "uid": 1001, "fullName": "", "wheel": false, "current": false},
        {"uid": 1002},
    ])
}

fn groups() -> Value {
    json!([
        {"name": "wheel", "gid": 998, "members": ["me"]},
        {"name": "docker", "gid": 1500, "members": []},
        {"name": "media", "gid": 1003, "members": ["ana"]},
        {"name": "video", "gid": 985, "members": ["me", "ana"]},
    ])
}

#[test]
fn the_names_and_rows_are_accounts_js_and_the_page_s() {
    let script = format!(
        r#"{root}
const A = load("services/Accounts.js");
const O = {{ accountUsers: {users}, accountGroups: {groups} }};
const r = pageRoot("AccountsPage.qml", O, {{}});
process.stdout.write(JSON.stringify({{
  users: {names}.map((n) => A.parseUsername(n)),
  groups: {names}.map((n) => A.parseGroupName(n)),
  full: {full}.map((n) => A.isFullName(n)),
  gids: [[1000, "a"], [999, "a"], [65534, "a"], [1500, "docker"], ["1200", "x"], [null, "x"]].map(([g, n]) => A.isRemovableGid(g, n)),
  faces: ["", "/home/me/.face.icon"].map((p) => A.faceRowDescription(p)),
  userRows: O.accountUsers.map((u) => r.userBlurb(u)),
  groupRows: O.accountGroups.map((g) => r.groupBlurb(g)),
  locked: ["wheel", "video", "docker"].map((g) => O.accountUsers.map((u) => r.memberLocked(g, u))),
  inside: ["wheel", "video", "nope"].map((g) => O.accountUsers.map((u) => r.userInGroup(g, u.name))),
}}));
"#,
        root = qml::PAGE_ROOT,
        users = users(),
        groups = groups(),
        names = json!(NAMES),
        full = json!(FULL),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let (u, g) = (users(), groups());
    let (u, g) = (u.as_array().unwrap(), g.as_array().unwrap());
    let gids: Vec<bool> = [
        (json!(1000), "a"),
        (json!(999), "a"),
        (json!(65534), "a"),
        (json!(1500), "docker"),
        (json!("1200"), "x"),
        (Value::Null, "x"),
    ]
    .iter()
    .map(|(gid, n)| accounts::removable_gid(Some(gid), n))
    .collect::<Vec<_>>();
    let locked: Vec<Vec<bool>> = ["wheel", "video", "docker"]
        .iter()
        .map(|grp| {
            u.iter()
                .map(|x| accounts::member_locked(g, grp, x))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let inside: Vec<Vec<bool>> = ["wheel", "video", "nope"]
        .iter()
        .map(|grp| {
            u.iter()
                .map(|x| {
                    accounts::has_member(
                        accounts::group(g, grp),
                        x["name"].as_str().unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let faces: Vec<String> = ["", "/home/me/.face.icon"]
        .iter()
        .map(|p| accounts::face_says(p))
        .collect();
    let ours = json!({
        "users": NAMES.iter().map(|n| accounts::parse_username(n)).collect::<Vec<_>>(),
        "groups": NAMES.iter().map(|n| accounts::parse_group_name(n)).collect::<Vec<_>>(),
        "full": FULL.iter().map(|n| accounts::is_full_name(n)).collect::<Vec<_>>(),
        "gids": gids,
        "faces": faces,
        "userRows": u.iter().map(accounts::user_says).collect::<Vec<_>>(),
        "groupRows": g.iter().map(accounts::group_says).collect::<Vec<_>>(),
        "locked": locked,
        "inside": inside,
    });
    assert_eq!(ours, theirs);
}

/// Add a user, Set password and Add a group check what is typed as the QML page does, and hand
/// the setter what it would.
#[test]
fn the_dialogs_check_what_the_qml_page_checks() {
    let adds = json!([
        ["ana", "", "pw", "pw", false],
        ["newbie", "New B", "pw", "pw", true],
        ["Bad", "", "pw", "pw", false],
        ["neo", "a,b", "pw", "pw", false],
        ["neo", "", "", "", false],
        ["neo", "", "pw", "pq", false],
    ]);
    let passwords = json!([["pw", "pw"], ["", ""], ["pw", "x"]]);
    let groups_typed = json!(["media", "newgrp", "Bad!", ""]);
    let script = format!(
        r#"{root}
const O = {{ accountUsers: {users}, accountGroups: {groups}, calls: [],
  addAccountUser: (...a) => O.calls.push(["user", ...a]), setAccountPassword: (...a) => O.calls.push(["password", ...a]),
  addAccountGroup: (...a) => O.calls.push(["group", ...a]) }};
const r = pageRoot("AccountsPage.qml", O, {{}});
const field = (t) => ({{ currentText: () => t, clear: () => {{}} }});
for (const d of ["addUserDialog", "passwordDialog", "addGroupDialog"]) globalThis[d] = {{ close: () => {{}}, open: () => {{}} }};
const out = {{ users: [], passwords: [], groups: [] }};
for (const [n, f, p, c, admin] of {adds}) {{
  Object.assign(globalThis, {{ addUserName: field(n), addUserFull: field(f), addUserPassword: field(p), addUserConfirm: field(c) }});
  r.addUserAdmin = admin; r.addError = ""; O.calls = [];
  r.submitAddUser();
  out.users.push([r.addError, O.calls]);
}}
for (const [p, c] of {passwords}) {{
  Object.assign(globalThis, {{ passwordField: field(p), passwordConfirm: field(c) }});
  r.passwordUser = "me"; r.passwordError = ""; O.calls = [];
  r.submitPassword();
  out.passwords.push([r.passwordError, O.calls]);
}}
for (const n of {groups_typed}) {{
  globalThis.addGroupName = field(n);
  r.addGroupError = ""; O.calls = [];
  r.submitAddGroup();
  out.groups.push([r.addGroupError, O.calls]);
}}
process.stdout.write(JSON.stringify(out));
"#,
        root = qml::PAGE_ROOT,
        users = users(),
        groups = groups(),
    );
    let Some(theirs) = qml::node(&script) else {
        eprintln!("skip: node is not installed");
        return;
    };
    let (u, g) = (users(), groups());
    let call = |kind: &str, args: Value| -> Value {
        let mut all = vec![json!(kind)];
        all.extend(args.as_array().unwrap().iter().cloned());
        json!([all])
    };
    let outcome = |r: Result<Option<atmos_ui::jobs::Job>, String>, kind: &str, args: Value| match r
    {
        Err(why) => json!([why, []]),
        Ok(job) => {
            assert!(
                job.is_some_and(|j| job_body(&j)["sudo"] == json!(true)),
                "{kind} runs as root"
            );
            json!(["", call(kind, args)])
        }
    };
    let ours = json!({
        "users": adds.as_array().unwrap().iter().map(|a| {
            let s = |i: usize| a[i].as_str().unwrap().to_owned();
            let d = NewUser { name: s(0), full: s(1), password: s(2), confirm: s(3), admin: a[4] == json!(true) };
            let args = json!([accounts::parse_username(&s(0)), s(1).trim(), s(2), a[4]]);
            outcome(d.submit(u.as_array().unwrap()), "user", args)
        }).collect::<Vec<_>>(),
        "passwords": passwords.as_array().unwrap().iter().map(|p| {
            let (pw, c) = (p[0].as_str().unwrap(), p[1].as_str().unwrap());
            outcome(accounts::submit_password("me", pw, c).map_err(String::from), "password", json!(["me", pw]))
        }).collect::<Vec<_>>(),
        "groups": groups_typed.as_array().unwrap().iter().map(|n| {
            let n = n.as_str().unwrap();
            outcome(accounts::submit_group(n, g.as_array().unwrap()), "group", json!([accounts::parse_group_name(n)]))
        }).collect::<Vec<_>>(),
    });
    assert_eq!(ours, theirs);

    let src = std::fs::read_to_string(qml::repo().join("pages/AccountsPage.qml")).unwrap();
    let flat = src
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace(" query: root.query", "");
    for want in [
        "title: \"Choose a face\" nameFilters: [\"Images (*.png *.jpg *.jpeg)\"] onAccepted: Omarchy.setAvatarPath(RichUi.pathFromUrl(selectedFile))",
        "title: \"Clear face\" message: \"Remove the face icon for this login.\" confirmText: \"Clear\" onConfirmed: Omarchy.clearAvatar()",
        "message: \"Remove \" + root.pendingUser + \" and their home directory.\" confirmText: \"Remove\" onConfirmed: Omarchy.removeAccountUser(root.pendingUser)",
        "message: \"Remove the \" + root.pendingGroup + \" group. Logins stay.\" confirmText: \"Remove\" onConfirmed: Omarchy.removeAccountGroup(root.pendingGroup)",
        "? \"Creating the login…\" : \"A local login with a home from /etc/skel.\"",
        "text: \"New password for \" + root.passwordUser + \".\"",
        "? \"Add or remove logins in wheel. This session has to stay.\" : \"Add or remove logins in this group.\"",
        "? \"This session has to stay in wheel.\" : root.userBlurb(modelData)",
        "Omarchy.setGroupMember(root.selectedGroup, name, !root.userInGroup(root.selectedGroup, name))",
        "label: Omarchy.currentUser.length ? Omarchy.currentUser : \"Face\" description: AccountsJs.faceRowDescription(Omarchy.avatarPath)",
        "enabled: root.fullNameValid && root.fullNameParsed !== Omarchy.fullName onClicked: Omarchy.setFullName(root.fullNameParsed)",
        "dangerAction: modelData && modelData.current ? \"\" : \"Remove…\"",
        "dangerAction: modelData && AccountsJs.isRemovableGid(modelData.gid, modelData.name) ? \"Remove…\" : \"\"",
    ] {
        assert!(flat.contains(want), "{want}");
    }
    let ok = accounts::full_name_says("");
    let bad = accounts::full_name_says("a,b");
    assert!(flat.contains(&format!("? {} : {}", json!(ok), json!(bad))));
}
