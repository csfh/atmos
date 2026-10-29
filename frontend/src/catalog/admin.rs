//! The Admin pages' groups and rows, as the QML pages have them in order. Search reads
//! the same words (`tests/catalog_parity.rs` holds them to `search.query`).

use super::control::Control::{Buttons, Custom, Minutes};
use super::control::Field::Key;
use super::control::When::{All, Any, Idle, Off, On};
use super::control::range;
use super::custom::Custom::Acct;
use super::setter::Args::{Given, Nothing};
use super::setter::Parse::SshKey;
use super::setter::Sends::{Job, SudoMode};
use super::setter::{ANY, ask, btn, setter, task};
use super::{Group, Page, line};

#[rustfmt::skip]
pub const PAGES: &[Page] = &[
    Page {
        id: "accounts",
        file: "AccountsPage.qml",
        description: "This login's name and face, other local users, and groups. Fingerprint, SSH, and sudoless Docker stay on Security.",
        groups: &[
            Group { title: "This account", detail: "Face is a PNG or JPEG copied to ~/.face.icon and AccountsService. Full name is the GECOS real name. Password changes this login.", rows: &[
                line("Full name", "", "chfn --full-name", &["user", "gecos", "real name", "display name", "account"]).with(Custom(Acct("fullname"))),
                line("Password", "Change the password for this login.", "chpasswd", &["password", "passwd", "login", "chpasswd"]).with(Custom(Acct("password"))),
            ] },
            Group { title: "Users", detail: "Human logins (UID 1000 and up) plus this session. Add copies /etc/skel. Remove deletes the home directory. You cannot remove the login you are using.", rows: &[
                line("Add a user", "Create a local login. Admin adds them to wheel.", "useradd -m", &["useradd", "add", "create", "login", "wheel", "admin"]).with(Custom(Acct("adduser"))),
                line("Logins", "No human logins.", "", &["user", "empty"]).with(Custom(Acct("nologins"))),
            ] },
            Group { title: "Groups", detail: "wheel and docker always show. Extra groups are ones with a human member or a GID of 1000 or more. Manage opens a group so you can add or remove logins. You cannot drop this session from wheel or remove wheel and docker.", rows: &[
                line("Add a group", "Create a local group, then add members from Manage…", "groupadd", &["groupadd", "add", "create", "group"]).with(Custom(Acct("addgroup"))),
                line("Groups", "No groups.", "", &["group", "empty"]).with(Custom(Acct("nogroups"))),
            ] },
        ],
    },
    Page {
        id: "security",
        file: "SecurityPage.qml",
        description: "Fingerprint, a security key, and whether this machine accepts SSH. Passwordless sudo is under Advanced.",
        groups: &[
            Group { title: "Login", detail: "Fingerprint and FIDO2 change PAM for sudo, polkit, and the lock screen. Set up talks to the reader or key.", rows: &[
                line("Fingerprint", "", "omarchy setup security fingerprint", &["fingerprint", "fprintd", "biometric", "lock"]).varies(On(Key("fingerprintConfigured")), "The reader can unlock sudo, polkit, and the lock screen.", "This machine has a reader. Set up enrolls a print and turns it on for sudo, polkit, and lock.").with(Buttons(&[btn("Set up…", setter("setupFingerprint", Nothing, ANY, Job(task("security.fingerprint-setup", "security-fingerprint", "The fingerprint setup").sudo()))).when(All(&[Idle, On(Key("fingerprintAvailable")), Off(Key("fingerprintConfigured"))])).confirm("Set up fingerprint", "Enroll a print for sudo, polkit, and the lock screen. You will need to touch the reader during setup.", "Set up"), btn("Remove…", setter("removeFingerprint", Nothing, ANY.needs(Key("fingerprintConfigured")), Job(task("security.fingerprint-remove", "security-fingerprint-remove", "The fingerprint removal").sudo()))).when(All(&[Idle, On(Key("fingerprintConfigured"))])).confirm("Remove fingerprint", "Stop using the fingerprint reader for sudo, polkit, and the lock screen.", "Remove")])).shown(On(Key("fingerprintAvailable"))),
                line("Security key", "", "omarchy setup security fido2", &["fido2", "yubikey", "webauthn", "u2f"]).varies(On(Key("fido2Configured")), "A FIDO2 key can stand in for a password on sudo and polkit.", "Register a FIDO2 key for sudo and polkit.").with(Buttons(&[btn("Set up…", setter("setupFido2", Nothing, ANY, Job(task("security.fido2-setup", "security-fido2", "The security key setup").sudo()))).when(All(&[Idle, Off(Key("fido2Configured"))])).confirm("Set up a security key", "Register a FIDO2 key for sudo and polkit. Have the key ready.", "Set up"), btn("Remove…", setter("removeFido2", Nothing, ANY.needs(Key("fido2Configured")), Job(task("security.fido2-remove", "security-fido2-remove", "The security key removal").sudo()))).when(All(&[Idle, On(Key("fido2Configured"))])).confirm("Remove security key", "Stop using FIDO2 keys for sudo and polkit.", "Remove")])),
            ] },
            Group { title: "Remote", detail: "OpenSSH on this machine. Turning it on needs a public key. Turning it off leaves authorized_keys alone.", rows: &[
                line("SSH server", "", "omarchy setup security sshd", &["ssh", "sshd", "openssh", "remote"]).with(Buttons(&[btn("Turn on…", setter("setupSshd", Given, ANY.parse(SshKey), Job(task("security.sshd-setup", "security-sshd", "The SSH setup").sudo().with_arg("key")))).when(All(&[Idle, Off(Key("sshdEnabled")), Off(Key("sshdActive"))])).prompt(ask("Turn on SSH", "ssh-ed25519 AAAA… comment", "Turn on", "Paste a public key. Omarchy starts sshd, opens the firewall, and adds that key to authorized_keys.", "Paste one public key line (ssh-ed25519 or ssh-rsa). Turn on stays off until it looks valid.", "Installing and starting OpenSSH…", SshKey)), btn("Turn off…", setter("disableSshd", Nothing, ANY.only(Any(&[On(Key("sshdEnabled")), On(Key("sshdActive"))])), Job(task("sshd.disable", "security-sshd-disable", "Turning off SSH").sudo()))).when(All(&[Idle, Any(&[On(Key("sshdEnabled")), On(Key("sshdActive"))])])).confirm("Turn off SSH", "Stop the OpenSSH server and close the firewall port. Authorized keys on this account stay.", "Turn off")])).words(super::words::Words::Ssh),
            ] },
            Group { title: "Advanced", detail: "Passwordless sudo is timed. Sudoless Docker puts this account in the docker group.", rows: &[
                line("Passwordless minutes", "How long passwordless sudo stays on when you enable it.", "omarchy sudo passwordless", &["sudo", "nopasswd", "passwordless", "minutes"]).with(Minutes(range(5.0, 120.0, 5.0, 0))),
                line("Passwordless sudo", "", "omarchy sudo passwordless", &["sudo", "nopasswd", "passwordless", "agent"]).varies(On(Key("passwordlessSudo")), "Sudo is not asking for a password right now. Turn it off early if you are done.", "Let this account run sudo without a password for the minutes above. Agents use this. So can anything else running as you.").with(Buttons(&[btn("Turn on…", setter("requestSudoMode", Nothing, ANY, SudoMode)).when(All(&[Idle, Off(Key("passwordlessSudo"))])), btn("Turn off…", setter("disablePasswordlessSudo", Nothing, ANY.needs(Key("passwordlessSudo")), Job(task("sudo.disable", "passwordless-sudo-off", "Turning off sudo mode").sudo()))).when(All(&[Idle, On(Key("passwordlessSudo"))])).confirm("Turn off passwordless sudo", "Sudo will ask for a password again.", "Turn off")])),
                line("Sudoless Docker", "", "omarchy setup security sudoless docker", &["docker", "group", "sudoless"]).varies(On(Key("sudolessDocker")), "This account is in the docker group.", "Add this account to the docker group so Docker runs without sudo.").with(Buttons(&[btn("Set up…", setter("setupSudolessDocker", Nothing, ANY, Job(task("security.docker-setup", "security-docker", "The Docker setup").sudo()))).when(All(&[Idle, Off(Key("sudolessDocker"))])).confirm("Sudoless Docker", "Add this account to the docker group so Docker runs without sudo. You may need to log in again.", "Set up"), btn("Remove…", setter("removeSudolessDocker", Nothing, ANY.needs(Key("sudolessDocker")), Job(task("security.docker-remove", "security-docker-remove", "The Docker removal").sudo()))).when(All(&[Idle, On(Key("sudolessDocker"))])).confirm("Remove sudoless Docker", "Take this account out of the docker group. Docker will need sudo again.", "Remove")])),
            ] },
        ],
    },
    Page {
        id: "services",
        file: "ServicesPage.qml",
        description: "Start, stop, and enable stay on the allowlist. Other units are status and logs.",
        groups: &[
            // The QML page's one group, kept out of search (`catalog: false`): its list draws it.
            Group { title: "", detail: "", rows: &[] },
        ],
    },
];
