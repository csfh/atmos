//! `manifest.toml` makes a Bundle the real `oma` accepts, with the daemon declared and `ratmos`
//! beside the window (docs/adr/0014, ODK's ADR 0050). It runs `oma bundle` and `oma inspect` on a
//! stand-in executable, in a private home: no window opens and nothing is installed.
//!
//! `oma` is the one `tests/frontend` builds (`OMA`), or ODK's debug build beside this
//! repository; without either the test says so and stops.

use std::path::{Path, PathBuf};
use std::process::Command;

fn oma() -> Option<PathBuf> {
    let beside = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../odk/target/debug/oma");
    let path = std::env::var_os("OMA").map_or(beside, PathBuf::from);
    if path.is_file() {
        Some(path)
    } else {
        eprintln!("skipped: no oma (cargo build -p oma, in ODK)");
        None
    }
}

/// An x86_64 ELF to stand in for the window and for `ratmos`: the check is of the Bundle, not
/// of what is in it.
const ELF: &str = "/usr/bin/true";

struct Home(PathBuf);

impl Home {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("atmos-ui-bundle-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    /// `oma` with nothing of the real session: its own home, and every XDG directory in it.
    fn oma(&self, oma: &Path, args: &[&str]) -> std::process::Output {
        Command::new(oma)
            .args(args)
            .env_clear()
            .env("HOME", &self.0)
            .env("XDG_DATA_HOME", self.0.join("data"))
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .env("XDG_STATE_HOME", self.0.join("state"))
            .env("XDG_CACHE_HOME", self.0.join("cache"))
            .env("PATH", "/usr/bin")
            .output()
            .expect("run oma")
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_manifest_makes_a_bundle_with_the_daemon_declared_and_ratmos_beside_the_window() {
    let Some(oma) = oma() else { return };
    let home = Home::new();
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("manifest.toml");
    let out = home.0.join("out");
    let include = format!("{ELF}=bin/ratmos");
    let made = home.oma(
        &oma,
        &[
            "bundle",
            ELF,
            "--manifest",
            manifest.to_str().unwrap(),
            "--include",
            &include,
            "-o",
            out.to_str().unwrap(),
        ],
    );
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let bundle = out.join("Atmos.oma");
    assert!(
        bundle.join("bin/ratmos").is_file(),
        "ratmos is in the Bundle"
    );
    assert!(bundle.join("atmos-ui").is_file(), "so is the window");

    let inspected = home.oma(&oma, &["inspect", bundle.to_str().unwrap()]);
    assert!(inspected.status.success());
    let text = String::from_utf8_lossy(&inspected.stdout);
    assert!(text.contains("dev.csfh.atmos"), "{text}");
    assert!(
        text.contains("Runs its own program (bin/ratmos) OUTSIDE the sandbox"),
        "the consent line says what the daemon is: {text}"
    );
    assert!(text.contains("graphics card"), "{text}");
    // Nothing else is asked for: no network, no folders, no sound.
    assert_eq!(text.matches("Can ").count(), 1, "{text}");
}

/// The Bundle needs `ratmos` in it: without, the daemon's program does not exist and `oma`
/// refuses the Bundle instead of making one that cannot start.
#[test]
fn a_bundle_without_ratmos_is_refused() {
    let Some(oma) = oma() else { return };
    let home = Home::new();
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("manifest.toml");
    let made = home.oma(
        &oma,
        &[
            "bundle",
            ELF,
            "--manifest",
            manifest.to_str().unwrap(),
            "-o",
            home.0.join("out").to_str().unwrap(),
        ],
    );
    assert!(!made.status.success());
    let err = String::from_utf8_lossy(&made.stderr);
    assert!(err.contains("daemon.executable"), "{err}");
    assert!(
        !home.0.join("out/Atmos.oma").exists(),
        "nothing is left behind"
    );
}
