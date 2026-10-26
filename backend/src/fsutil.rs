//! The one way Atmos replaces a file: write a uniquely named sibling, sync it,
//! then rename it over the target, so a reader never sees half a file and two
//! writers never share a temp name.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|err| Error::io(err, parent))?;
    // Two threads writing one path in the same nanosecond must not share a
    // temporary file, so the clock is not enough: a counter makes the name unique.
    static WRITES: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mut tmp_name = path.file_name().unwrap_or_default().to_os_string();
    tmp_name.push(format!(
        ".tmp-{}-{nanos}-{}",
        std::process::id(),
        WRITES.fetch_add(1, Ordering::SeqCst)
    ));
    let tmp = parent.join(tmp_name);
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|err| Error::io(err, &tmp))?;
        file.write_all(bytes).map_err(|err| Error::io(err, &tmp))?;
        file.sync_all().map_err(|err| Error::io(err, &tmp))?;
    }
    // Keep the mode of the file being replaced, so a 0600 file stays 0600.
    if let Ok(meta) = fs::metadata(path) {
        let _ = fs::set_permissions(&tmp, meta.permissions());
    }
    fs::rename(&tmp, path).map_err(|err| {
        let _ = fs::remove_file(&tmp);
        Error::io(err, path)
    })
}

pub fn atomic_write_text(path: &Path, text: &str) -> Result<()> {
    atomic_write(path, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("atmosd-fsutil-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn writers_racing_on_one_path_never_share_a_temporary_file() {
        let dir = scratch("race");
        let target = dir.join("state.json");
        let writers: Vec<_> = (0..8)
            .map(|n| {
                let target = target.clone();
                std::thread::spawn(move || {
                    for round in 0..50 {
                        atomic_write(&target, format!("writer {n} round {round}\n").as_bytes())
                            .unwrap();
                    }
                })
            })
            .collect();
        for writer in writers {
            writer.join().unwrap();
        }
        let text = fs::read_to_string(&target).unwrap();
        assert!(
            text.starts_with("writer ") && text.ends_with('\n'),
            "{text:?}"
        );
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[test]
    fn siblings_with_different_extensions_do_not_collide() {
        let dir = scratch("siblings");
        atomic_write(&dir.join("a.toml"), b"toml").unwrap();
        atomic_write(&dir.join("a.json"), b"json").unwrap();
        assert_eq!(fs::read(dir.join("a.toml")).unwrap(), b"toml");
        assert_eq!(fs::read(dir.join("a.json")).unwrap(), b"json");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    #[test]
    fn keeps_the_mode_of_the_replaced_file() {
        let dir = scratch("mode");
        let file = dir.join("secret");
        fs::write(&file, b"old").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        atomic_write(&file, b"new").unwrap();
        assert_eq!(
            fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(fs::read(&file).unwrap(), b"new");
    }

    #[test]
    fn creates_missing_parents() {
        let dir = scratch("parents");
        atomic_write(&dir.join("x/y/z.txt"), b"ok").unwrap();
        assert_eq!(fs::read(dir.join("x/y/z.txt")).unwrap(), b"ok");
    }
}
