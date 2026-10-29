//! What the Security page checks before it turns SSH on: one public key line, as
//! `RichUi.parseSshPublicKey` takes it, so junk never reaches the setup (which starts sshd before
//! it looks at the key). `tests/catalog_parity.rs` runs the JavaScript under Node against this.

/// The key line to write, or "" when `raw` is not one: a key type, a base64 blob, and an optional
/// comment, on one line.
pub fn ssh_public_key(raw: &str) -> String {
    let text = raw.trim();
    if text.is_empty() || text.encode_utf16().count() > 8192 {
        return String::new();
    }
    if text.contains(['\r', '\n']) || text.contains("-----") || text.starts_with('-') {
        return String::new();
    }
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 2 || !is_key_type(parts[0]) || !is_blob(parts[1]) {
        return String::new();
    }
    let at = text.find(parts[1]).unwrap_or(0) + parts[1].len();
    let comment = text[at..].trim_start();
    if comment.is_empty() {
        format!("{} {}", parts[0], parts[1])
    } else {
        format!("{} {} {comment}", parts[0], parts[1])
    }
}

/// `^(?:sk-)?(?:ssh-[a-z0-9]+|ecdsa-sha2-nistp(?:256|384|521))(?:-cert-v01@openssh\.com|@openssh\.com)?$`
fn is_key_type(text: &str) -> bool {
    if text.is_empty() || text.len() > 80 || text.starts_with('-') {
        return false;
    }
    let text = text.strip_prefix("sk-").unwrap_or(text);
    let core = text
        .strip_suffix("-cert-v01@openssh.com")
        .or_else(|| text.strip_suffix("@openssh.com"))
        .unwrap_or(text);
    let ssh = core.strip_prefix("ssh-").is_some_and(|name| {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    });
    ssh || matches!(
        core,
        "ecdsa-sha2-nistp256" | "ecdsa-sha2-nistp384" | "ecdsa-sha2-nistp521"
    )
}

/// `^[A-Za-z0-9+/]{32,}={0,2}$`
fn is_blob(text: &str) -> bool {
    let body = text.trim_end_matches('=');
    text.len() - body.len() <= 2
        && body.len() >= 32
        && body
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_line_is_kept_and_junk_is_not() {
        let blob = "AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl";
        assert_eq!(
            ssh_public_key(&format!("  ssh-ed25519 {blob}   me@host ")),
            format!("ssh-ed25519 {blob} me@host")
        );
        assert_eq!(ssh_public_key("ssh-ed25519 short"), "");
        assert_eq!(ssh_public_key("-----BEGIN"), "");
        assert!(is_key_type("sk-ssh-ed25519@openssh.com"));
        assert!(!is_key_type("ssh-"));
    }
}
