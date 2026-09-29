//! Locate the scripts Atmos already ships.

use std::path::{Path, PathBuf};

/// `ATMOS_ROOT/scripts` wins. Otherwise walk up from the binary so a
/// checkout build and an XDG install (`bin/ratmos` next to `scripts/`)
/// both work when the launcher did not export `ATMOS_ROOT`.
pub fn repo_script(name: &str) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Ok(root) = std::env::var("ATMOS_ROOT") {
        candidates.push(PathBuf::from(root).join("scripts").join(name));
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(Path::to_path_buf);
        for _ in 0..8 {
            if let Some(current) = dir.as_ref() {
                candidates.push(current.join("scripts").join(name));
                dir = current.parent().map(Path::to_path_buf);
            }
        }
    }
    candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| format!("missing scripts/{name}"))
}
