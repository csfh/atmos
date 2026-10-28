//! The user's images a window shows: the avatar and the current background. A window in its
//! sandbox cannot read them (they are outside what it is granted), and `host.read` answers with
//! text, so this answers with the file's bytes in base64. Only these two images, found here and
//! not named by the window, so it reads no path a window chose.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::error::{Error, Kind, Result};

/// The largest image sent: a wallpaper of a large display, compressed.
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Which image: the session user's avatar or the current background.
fn path(root: Option<&Path>, which: &str) -> Result<Option<PathBuf>> {
    let home = || std::env::var_os("HOME").map(PathBuf::from);
    let found = match (which, root) {
        ("avatar", Some(root)) => Some(root.join(".face")),
        ("avatar", None) => {
            let p = crate::inventory::accounts::current_avatar();
            (!p.is_empty()).then(|| PathBuf::from(p))
        }
        ("background", Some(root)) => Some(root.join(".local/state/omarchy/current/background")),
        ("background", None) => home().map(|h| h.join(".local/state/omarchy/current/background")),
        _ => {
            return Err(Error::bad_request("which is avatar or background").with_context("which"));
        }
    };
    Ok(found.filter(|p| p.is_file()))
}

/// The image's bytes in base64, with its type, or null when there is none.
pub fn host_image(root: Option<&Path>, which: &str) -> Result<Value> {
    let Some(file) = path(root, which)? else {
        return Ok(json!({ "which": which, "image": null }));
    };
    let size = std::fs::metadata(&file)?.len();
    if size > MAX_BYTES {
        return Err(Error::new(Kind::Failed, "the image is larger than 16 MiB").with_context(which));
    }
    let bytes = std::fs::read(&file)?;
    Ok(json!({
        "which": which,
        "image": {
            "type": kind(&bytes),
            "bytes": bytes.len(),
            "base64": base64(&bytes),
        },
    }))
}

/// The image's type from its first bytes, or null for one a window cannot draw.
fn kind(bytes: &[u8]) -> Option<&'static str> {
    Some(match bytes {
        [0x89, b'P', b'N', b'G', ..] => "image/png",
        [0xFF, 0xD8, 0xFF, ..] => "image/jpeg",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
        [b'G', b'I', b'F', b'8', ..] => "image/gif",
        [b'B', b'M', ..] => "image/bmp",
        _ => return None,
    })
}

/// Standard base64 with padding (RFC 4648). std only (docs/adr/0007).
fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(TABLE[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), encoded, "{plain}");
        }
        assert_eq!(base64(&[0xff, 0xfe, 0xfd]), "//79");
    }

    #[test]
    fn an_image_is_its_type_and_bytes_and_none_is_null() {
        let dir = std::env::temp_dir().join(format!("atmos-images-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".local/state/omarchy/current")).unwrap();
        std::fs::write(dir.join(".face"), b"\x89PNG\r\n\x1a\nrest").unwrap();
        let avatar = host_image(Some(&dir), "avatar").unwrap();
        assert_eq!(avatar["image"]["type"], "image/png");
        assert_eq!(avatar["image"]["bytes"], 12);
        assert_eq!(avatar["image"]["base64"], base64(b"\x89PNG\r\n\x1a\nrest"));
        let none = host_image(Some(&dir), "background").unwrap();
        assert_eq!(none["image"], Value::Null);
        assert!(
            host_image(Some(&dir), "/etc/passwd").is_err(),
            "no path from a window"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
