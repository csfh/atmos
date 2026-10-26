//! Machine facts: storage, power, hardware flags, identities, default apps,
//! installed software, and the Atmos install itself.

use super::sh::{
    self, alt, file_has, first_line, is_uint, jq_text, lines, ok, out, out_ok, out_within, present,
    read,
};
use regex_lite::Regex;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The starred rows. No file is an empty list; a file that is not a list of
/// rows is a failure, not an empty list.
pub fn favorites(f: &mut Map<String, Value>) {
    let file = crate::tasks::user::favorites_file();
    let items = match std::fs::read_to_string(&file) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => json!([]),
        Err(err) => {
            sh::fail(format!("{}: {err}", file.display()));
            return;
        }
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Array(rows)) => json!(rows),
            Ok(Value::Object(mut doc)) if doc.get("items").is_some_and(Value::is_array) => {
                doc.remove("items").unwrap_or(json!([]))
            }
            _ => {
                sh::fail(format!("{}: not a list of favorites", file.display()));
                return;
            }
        },
    };
    f.insert("favorites".into(), items);
}

pub fn snapper(f: &mut Map<String, Value>) {
    let present_snapper = present("snapper");
    let mut configs = json!([]);
    if present_snapper {
        let csv = out("snapper", &["--csvout", "list-configs"]);
        configs = json!(csv
            .lines()
            .skip(1)
            .filter_map(|l| {
                let mut cols = l.split(',');
                let name = cols.next().filter(|n| !n.is_empty())?;
                Some(json!({"name": name, "subvolume": cols.next().unwrap_or("")}))
            })
            .collect::<Vec<_>>());
    }
    f.insert("snapperPresent".into(), json!(present_snapper));
    f.insert("snapperConfigs".into(), configs);
    f.insert(
        "hibernationAvailable".into(),
        json!(ok("omarchy", &["hibernation", "available"])),
    );
    f.insert(
        "hibernationSupported".into(),
        json!(Path::new("/sys/power/image_size").is_file() && present("limine-mkinitcpio")),
    );
    f.insert(
        "hibernationConfigured".into(),
        json!(
            read(Path::new("/etc/mkinitcpio.conf.d/omarchy_resume.conf"))
                .lines()
                .any(|l| l == "HOOKS+=(resume)")
        ),
    );
    let (mut limit, mut timeline) = (5u64, false);
    let conf = read(Path::new("/etc/snapper/configs/root"));
    let value = |key: &str| {
        conf.lines().find_map(|l| {
            let t = l.trim_start();
            let rest = t.strip_prefix(key)?.strip_prefix('=')?;
            Some(rest.split('=').next().unwrap_or("").replace('"', ""))
        })
    };
    if let Some(raw) = value("NUMBER_LIMIT").filter(|v| is_uint(v)) {
        limit = raw.parse().unwrap_or(5);
    }
    if value("TIMELINE_CREATE").as_deref() == Some("yes") {
        timeline = true;
    }
    if !(1..=50).contains(&limit) {
        limit = 5;
    }
    f.insert("snapperNumberLimit".into(), json!(limit));
    f.insert("snapperTimeline".into(), json!(timeline));
    f.insert(
        "fstrimEnabled".into(),
        json!(
            present("systemctl") && out("systemctl", &["is-enabled", "fstrim.timer"]) == "enabled"
        ),
    );
}

pub fn branding(f: &mut Map<String, Value>, home: &str, omarchy_path: &str) {
    let differs = |custom: PathBuf, default: PathBuf| {
        custom.is_file()
            && !(default.is_file() && std::fs::read(&custom).ok() == std::fs::read(&default).ok())
    };
    let base = Path::new(home).join(".config/omarchy/branding");
    f.insert(
        "screensaverBranded".into(),
        json!(differs(
            base.join("screensaver.txt"),
            Path::new(omarchy_path).join("logo.txt")
        )),
    );
    f.insert(
        "aboutBranded".into(),
        json!(differs(
            base.join("about.txt"),
            Path::new(omarchy_path).join("icon.txt")
        )),
    );
}

pub fn weather(f: &mut Map<String, Value>, home: &str) {
    let file = Path::new(home).join(".local/state/omarchy/settings/weather.json");
    let (mut name, mut auto, mut coords) = (String::new(), true, String::new());
    if file.is_file() {
        let doc: Value = serde_json::from_str(&read(&file)).unwrap_or(Value::Null);
        name = alt(doc.get("name")).map(jq_text).unwrap_or_default();
        if !name.is_empty() {
            auto = false;
        }
        let num = Regex::new(r"^-?[0-9]+(\.[0-9]+)?$").expect("regex");
        let lat = alt(doc.get("latitude")).map(jq_text).unwrap_or_default();
        let lon = alt(doc.get("longitude")).map(jq_text).unwrap_or_default();
        if num.is_match(&lat) && num.is_match(&lon) {
            coords = format!("{lat},{lon}");
        }
    }
    f.insert("weatherLocation".into(), json!(name));
    f.insert("weatherAuto".into(), json!(auto));
    f.insert("weatherCoords".into(), json!(coords));
}

pub fn power(f: &mut Map<String, Value>) {
    let profiles = lines(&sh::omarchy_must(&["powerprofiles", "list"]).unwrap_or_default());
    let active = sh::omarchy_must(&["powerprofiles", "list", "--active-state"])
        .unwrap_or_default()
        .lines()
        .find_map(|l| {
            let mut cols = l.split('\t');
            let (name, state) = (cols.next()?, cols.next()?);
            (state == "1").then(|| name.to_string())
        })
        .unwrap_or_default();
    let state_dir = std::env::var("OMARCHY_POWERPROFILES_STATE_DIR")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            let base = std::env::var("XDG_STATE_HOME")
                .ok()
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| {
                    format!("{}/.local/state", std::env::var("HOME").unwrap_or_default())
                });
            format!("{base}/omarchy/powerprofiles")
        });
    let stored = |name: &str| first_line(&Path::new(&state_dir).join(name));
    let mut ac = stored("ac");
    if ac.is_empty() {
        ac = if profiles.iter().any(|p| p == "performance") {
            "performance".into()
        } else {
            "balanced".into()
        };
    }
    let mut battery = stored("battery");
    if battery.is_empty() {
        battery = "balanced".into();
    }
    f.insert("powerProfiles".into(), json!(profiles));
    f.insert("powerProfile".into(), json!(active));
    f.insert("powerProfileAc".into(), json!(ac));
    f.insert("powerProfileBattery".into(), json!(battery));
}

fn hw_flag(args: &[&str]) -> bool {
    let mut full = vec!["hw"];
    full.extend(args);
    ok("omarchy", &full)
}

pub fn hardware_flags(f: &mut Map<String, Value>) {
    f.insert("hwNvidia".into(), json!(hw_flag(&["nvidia"])));
    f.insert("hwNvidiaGsp".into(), json!(hw_flag(&["nvidia", "gsp"])));
    f.insert(
        "hwNvidiaWithoutGsp".into(),
        json!(hw_flag(&["nvidia", "without", "gsp"])),
    );
    f.insert("hwVulkan".into(), json!(hw_flag(&["vulkan"])));
    f.insert("hwIntel".into(), json!(hw_flag(&["intel"])));
    f.insert("hwIntelPtl".into(), json!(hw_flag(&["intel", "ptl"])));
    f.insert("hwWebcam".into(), json!(hw_flag(&["webcam"])));
    f.insert("hwFramework16".into(), json!(hw_flag(&["framework16"])));
    f.insert("hwAsusRog".into(), json!(hw_flag(&["asus", "rog"])));
    f.insert("hwSurface".into(), json!(hw_flag(&["surface"])));
    let dmi = |name: &str| {
        let raw = read(&Path::new("/sys/class/dmi/id").join(name));
        raw.lines().next().unwrap_or("").replace('\r', "")
    };
    let mut product = dmi("product_name");
    if product.is_empty() {
        product = read(Path::new("/proc/device-tree/model")).replace('\0', "");
    }
    f.insert("dmiVendor".into(), json!(dmi("sys_vendor")));
    f.insert("dmiProduct".into(), json!(product));
    f.insert("dmiFamily".into(), json!(dmi("product_family")));
    let (mut cpu, mut mem) = (String::new(), String::new());
    if let Some(stats) = out_ok("omarchy", &["system", "stats"]) {
        for line in stats.lines() {
            if let Some(rest) = line.strip_prefix("cpu") {
                cpu = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("memory") {
                mem = rest.to_string();
            }
        }
    }
    f.insert("cpuStat".into(), json!(cpu));
    f.insert("memoryStat".into(), json!(mem));
    f.insert("cpuIdentity".into(), json!(cpu_identity()));
    f.insert("gpuIdentity".into(), json!(pci_identity(true)));
    f.insert("npuIdentity".into(), json!(pci_identity(false)));
}

fn cpu_identity() -> String {
    let info = read(Path::new("/proc/cpuinfo"));
    let find = |keys: &[&str]| {
        info.lines().find_map(|l| {
            let (key, value) = l.split_once(':')?;
            keys.iter()
                .any(|k| key.trim().eq_ignore_ascii_case(k))
                .then(|| value.trim().to_string())
        })
    };
    find(&["model name"])
        .or_else(|| find(&["Hardware", "cpu model"]))
        .unwrap_or_default()
}

fn pci_identity(gpu_kind: bool) -> String {
    if !present("lspci") {
        return String::new();
    }
    let raw = out("lspci", &["-nn"]);
    let id_tail = Regex::new(r" \[[0-9a-fA-F]{4}:[0-9a-fA-F]{4}\].*").expect("regex");
    let rev = Regex::new(r" \(rev [^)]+\)$").expect("regex");
    for line in raw.lines() {
        let lower = line.to_lowercase();
        let is_gpu = [
            "vga compatible controller",
            "3d controller",
            "display controller",
        ]
        .iter()
        .any(|k| lower.contains(k));
        let is_npu = [
            "neural processing",
            "npu",
            "xdna",
            "tpu",
            "habana",
            "hailo",
            "coral",
            "ai accelerator",
        ]
        .iter()
        .any(|k| lower.contains(k));
        if (gpu_kind && !is_gpu) || (!gpu_kind && !is_npu) {
            continue;
        }
        let Some(at) = line.find(": ") else {
            continue;
        };
        let rest = &line[at + 2..];
        let rest = id_tail.replace(rest, "").to_string();
        let rest = rev.replace(&rest, "").trim().to_string();
        if !rest.is_empty() {
            return rest;
        }
    }
    String::new()
}

pub fn security(f: &mut Map<String, Value>, user: &str) {
    let sudo = read(Path::new("/etc/pam.d/sudo"));
    f.insert(
        "fingerprintAvailable".into(),
        json!(present("omarchy-hw-fingerprint") && ok("omarchy-hw-fingerprint", &[])),
    );
    f.insert(
        "fingerprintConfigured".into(),
        json!(sudo.contains("pam_fprintd.so")),
    );
    f.insert("fido2Configured".into(), json!(sudo.contains("pam_u2f.so")));
    let (mut enabled, mut active) = (false, false);
    if present("systemctl") {
        enabled = out("systemctl", &["is-enabled", "sshd.service"]) == "enabled";
        active = out("systemctl", &["is-active", "sshd.service"]) == "active";
    }
    f.insert("sshdEnabled".into(), json!(enabled));
    f.insert("sshdActive".into(), json!(active));
    // `/etc/sudoers.d` is root-only, so the file cannot be seen from here. The
    // `sudo-on` task also starts a transient timer that removes it, and the
    // timer is visible to every user without a `sudo` probe in the journal.
    let timed = present("systemctl")
        && out(
            "systemctl",
            &[
                "is-active",
                &format!("omarchy-nopasswd-expire-{user}.timer"),
            ],
        ) == "active";
    f.insert(
        "passwordlessSudo".into(),
        json!(timed || Path::new(&format!("/etc/sudoers.d/99-omarchy-nopasswd-{user}")).is_file()),
    );
    let groups = out("id", &["-nG"]);
    f.insert(
        "sudolessDocker".into(),
        json!(groups.split_whitespace().any(|g| g == "docker")),
    );
}

pub fn omarchy_info(f: &mut Map<String, Value>) {
    let head = |args: &[&str]| {
        sh::omarchy_must(args)
            .unwrap_or_default()
            .lines()
            .next()
            .unwrap_or("")
            .to_string()
    };
    f.insert("omarchyVersion".into(), json!(head(&["version"])));
    let channel = head(&["channel", "current"]);
    f.insert(
        "omarchyChannel".into(),
        json!(
            if ["stable", "rc", "edge", "dev"].contains(&channel.as_str()) {
                channel
            } else {
                String::new()
            }
        ),
    );
    let update = crate::runner::Run::new("omarchy")
        .args(["update", "available"])
        .output()
        .ok();
    let (available, text) = match update {
        Some(o) => (o.success(), o.stdout_text()),
        None => (false, String::new()),
    };
    let summary = text.lines().next().unwrap_or("").to_string();
    f.insert("updateAvailable".into(), json!(available));
    f.insert(
        "updateSummary".into(),
        json!(if summary.is_empty() {
            "Omarchy is up to date".to_string()
        } else {
            summary
        }),
    );
}

pub fn atmos(f: &mut Map<String, Value>, home: &str) {
    let data = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{home}/.local/share"));
    let data = Path::new(&data).join("atmos");
    let root = crate::platform::app_root();
    let packaged = root.join("PACKAGED").is_file();
    let mut version = String::new();
    if packaged {
        let v = first_line(&root.join("PACKAGED"));
        if Regex::new(r"^[0-9A-Za-z][0-9A-Za-z.+_~-]{0,39}$").is_ok_and(|re| re.is_match(&v)) {
            version = v;
        }
    }
    let mut revision = first_line(&data.join("REVISION"));
    if !Regex::new(r"^[0-9a-f]{4,40}$").is_ok_and(|re| re.is_match(&revision)) {
        revision.clear();
    }
    // Only `stable` exists; an old `alpha` and anything unknown read as stable.
    let channel = "stable";
    f.insert("atmosRevision".into(), json!(revision));
    f.insert("atmosChannel".into(), json!(channel));
    f.insert(
        "atmosInstalled".into(),
        json!(packaged || data.join("bin/atmos").is_file()),
    );
    f.insert("atmosPackaged".into(), json!(packaged));
    f.insert("atmosVersion".into(), json!(version));
}

pub fn gpu_extras(f: &mut Map<String, Value>) {
    f.insert("voxtypeInstalled".into(), json!(present("voxtype")));
    f.insert(
        "hybridGpuAvailable".into(),
        json!(present("omarchy-hw-hybrid-gpu") && ok("omarchy-hw-hybrid-gpu", &[])),
    );
    let mut mode = String::new();
    if present("supergfxctl") {
        mode = out_within("supergfxctl", &["-g"], 2);
        if mode != "Integrated" && mode != "Hybrid" {
            mode.clear();
        }
    }
    f.insert("hybridGpuMode".into(), json!(mode));
}

pub fn boot_and_defaults(f: &mut Map<String, Value>, home: &str) {
    let (mut available, mut enabled) = (false, false);
    if Path::new("/sys/firmware/efi").is_dir() && present("efibootmgr") {
        available = true;
        let list = out("efibootmgr", &[]);
        let boot = Regex::new(r"^Boot[0-9A-Fa-f]+\*? Omarchy(\s|$)").expect("regex");
        enabled = list.lines().any(|l| boot.is_match(l));
    }
    f.insert("directBootAvailable".into(), json!(available));
    f.insert("directBoot".into(), json!(enabled));
    let desktop = Regex::new(r"^[A-Za-z0-9._-]+\.desktop$").expect("regex");
    let query = |mime: &str| {
        let v = out("xdg-mime", &["query", "default", mime]);
        if desktop.is_match(&v) {
            v
        } else {
            String::new()
        }
    };
    f.insert("mimePdf".into(), json!(query("application/pdf")));
    f.insert("mimeImage".into(), json!(query("image/png")));
    f.insert("mimeVideo".into(), json!(query("video/mp4")));
    let data = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{home}/.local/share"));
    let dirs = [
        format!("{data}/applications"),
        "/usr/local/share/applications".into(),
        "/usr/share/applications".into(),
    ];
    let options = |items: &[(&str, &str)]| {
        json!(items
            .iter()
            .filter(|(id, _)| dirs.iter().any(|d| Path::new(d).join(id).is_file()))
            .map(|(id, label)| json!({"value": id, "label": label}))
            .collect::<Vec<_>>())
    };
    f.insert(
        "mimePdfOptions".into(),
        options(&[
            ("org.gnome.Evince.desktop", "Evince"),
            ("org.gnome.Papers.desktop", "Papers"),
            ("org.pwmt.zathura.desktop", "Zathura"),
            ("org.kde.okular.desktop", "Okular"),
            ("firefox.desktop", "Firefox"),
            ("chromium.desktop", "Chromium"),
        ]),
    );
    f.insert(
        "mimeImageOptions".into(),
        options(&[
            ("imv.desktop", "imv"),
            ("org.gnome.Loupe.desktop", "Loupe"),
            ("org.gnome.eog.desktop", "Image Viewer"),
            ("firefox.desktop", "Firefox"),
            ("chromium.desktop", "Chromium"),
        ]),
    );
    f.insert(
        "mimeVideoOptions".into(),
        options(&[
            ("mpv.desktop", "mpv"),
            ("vlc.desktop", "VLC"),
            ("org.gnome.Totem.desktop", "Videos"),
            ("firefox.desktop", "Firefox"),
            ("chromium.desktop", "Chromium"),
        ]),
    );
    user_dirs(f, home);
}

fn user_dirs(f: &mut Map<String, Value>, home: &str) {
    let config = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{home}/.config"));
    let dirs = read(&Path::new(&config).join("user-dirs.dirs"));
    let lookup = |key: &str| {
        dirs.lines().find_map(|l| {
            let value = l.trim().strip_prefix(key)?.strip_prefix('=')?;
            Some(value.trim_matches('"').replace("$HOME", home))
        })
    };
    // The file is sourced, so what it sets wins over the environment.
    let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let mut pictures = lookup("XDG_PICTURES_DIR")
        .or_else(|| env("XDG_PICTURES_DIR"))
        .unwrap_or_else(|| format!("{home}/Pictures"));
    let mut videos = lookup("XDG_VIDEOS_DIR")
        .or_else(|| env("XDG_VIDEOS_DIR"))
        .unwrap_or_else(|| format!("{home}/Videos"));
    if present("xdg-user-dir") {
        if let Some(v) = out_ok("xdg-user-dir", &["PICTURES"]) {
            pictures = v;
        }
        if let Some(v) = out_ok("xdg-user-dir", &["VIDEOS"]) {
            videos = v;
        }
    }
    f.insert("picturesDir".into(), json!(pictures));
    f.insert("videosDir".into(), json!(videos));
}

pub fn software(f: &mut Map<String, Value>, home: &str, tailscale_installed: bool) {
    f.insert(
        "recordingActive".into(),
        json!(sh::ok("pgrep", &["-f", "^gpu-screen-recorder"])),
    );
    let mut overlay = false;
    if present("hyprctl") {
        let clients: Value =
            serde_json::from_str(&out("hyprctl", &["clients", "-j"])).unwrap_or(Value::Null);
        overlay = clients
            .as_array()
            .is_some_and(|c| c.iter().any(|w| w["title"] == "WebcamOverlay"));
    }
    f.insert("webcamOverlay".into(), json!(overlay));
    let any = |names: &[&str]| names.iter().any(|n| present(n));
    f.insert(
        "services".into(),
        json!({
            "onepassword": present("1password"),
            "dropbox": any(&["dropbox", "dropbox-cli"]),
            "nordvpn": present("nordvpn"),
            "signal": present("signal-desktop"),
            "spotify": present("spotify"),
            "sunshine": present("sunshine"),
            "tailscale": tailscale_installed,
        }),
    );
    let apps = Path::new(home).join(".local/share/applications");
    let xbox_cloud = std::fs::read_dir(&apps)
        .map(|rd| {
            rd.flatten().any(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".desktop")
                    && name
                        .find("xbox")
                        .is_some_and(|i| name[i + 4..].contains("cloud"))
            })
        })
        .unwrap_or(false);
    f.insert(
        "gaming".into(),
        json!({
            "steam": present("steam"),
            "heroic": present("heroic"),
            "lutris": present("lutris"),
            "retroarch": present("retroarch"),
            "battlenet": Path::new(home).join("Games/battlenet/drive_c/Program Files (x86)/Battle.net/Battle.net Launcher.exe").is_file(),
            "geforceNow": any(&["nvidia-geforcenow", "geforcenow", "GeForceNOW"]),
            "xboxCloud": xbox_cloud,
            "xboxControllers": Path::new("/etc/modules-load.d/xpadneo.conf").is_file(),
        }),
    );
    f.insert(
        "extras".into(),
        json!({
            "chatgpt": present("chatgpt"),
            "docker": present("docker"),
            "herdr": present("herdr"),
            "localsend": present("localsend"),
        }),
    );
    f.insert("hasAether".into(), json!(present("aether")));
    f.insert(
        "browsers".into(),
        json!({
            "chromium": present("chromium"),
            "chrome": present("google-chrome-stable"),
            "brave": present("brave"),
            "brave-origin": present("brave-origin"),
            "edge": present("microsoft-edge-stable"),
            "firefox": present("firefox"),
            "zen": present("zen-browser"),
        }),
    );
    f.insert("terminals".into(), json!({"alacritty": present("alacritty"), "foot": present("foot"), "ghostty": present("ghostty"), "kitty": present("kitty")}));
    f.insert(
        "editors".into(),
        json!({
            "nvim": present("nvim"), "code": present("code"), "cursor": present("cursor"),
            "zeditor": present("zeditor"), "sublime_text": present("sublime_text"),
            "helix": present("helix"), "vim": present("vim"), "emacs": present("emacs"),
        }),
    );
    let plugins = sh::omarchy_must(&["plugin", "list", "--json"])
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(Value::is_array)
        .unwrap_or(json!([]));
    f.insert("plugins".into(), plugins);
}

/// `omarchy menu keybindings --print` lines as {keys, action}.
pub fn keybindings() -> Value {
    if !present("omarchy") {
        return json!([]);
    }
    let raw = sh::must_within(
        "omarchy",
        &["menu", "keybindings", "--print"],
        Duration::from_secs(10),
    )
    .unwrap_or_default();
    let rows: Vec<Value> = raw
        .lines()
        .filter_map(|line| {
            let (at, arrow) = match line.find(" \u{2192} ") {
                Some(i) => (i, " \u{2192} ".len()),
                None => (line.find(" -> ")?, 4),
            };
            let keys = line[..at].trim_end();
            let action = line[at + arrow..].trim();
            (!keys.is_empty() && !action.is_empty())
                .then(|| json!({"keys": keys, "action": action}))
        })
        .collect();
    json!(rows)
}

pub fn focus_and_print(f: &mut Map<String, Value>) {
    let mut class = String::new();
    if present("hyprctl") {
        let raw = out("hyprctl", &["activewindow", "-j"]);
        class = serde_json::from_str::<Value>(&raw)
            .ok()
            .and_then(|v| alt(v.get("class")).map(jq_text))
            .unwrap_or_default();
    }
    f.insert("focusedClass".into(), json!(class));
    f.insert(
        "cupsActive".into(),
        json!(ok("systemctl", &["is-active", "--quiet", "cups.service"])),
    );
    f.insert(
        "printerSetup".into(),
        json!(present("system-config-printer")),
    );
}

pub fn marker_flags(f: &mut Map<String, Value>, home: &str) {
    let autostart = crate::host::env_path(
        "ATMOS_AUTOSTART_FILE",
        format!("{home}/.config/hypr/autostart.lua"),
    );
    let bindings = crate::host::env_path(
        "ATMOS_BINDINGS_FILE",
        format!("{home}/.config/hypr/bindings.lua"),
    );
    let windows = crate::host::env_path(
        "ATMOS_WINDOWS_FILE",
        format!("{home}/.config/hypr/atmos.lua"),
    );
    let list = |kind, file: &Path| crate::hyprland::list_file(kind, file).unwrap_or(json!([]));
    use crate::hyprland::Kind;
    f.insert("autostart".into(), list(Kind::Autostart, &autostart));
    f.insert(
        "autostartManaged".into(),
        json!(file_has(&autostart, Kind::Autostart.begin())),
    );
    f.insert("bindings".into(), list(Kind::Bindings, &bindings));
    f.insert(
        "bindingsManaged".into(),
        json!(file_has(&bindings, Kind::Bindings.begin())),
    );
    f.insert("windowRules".into(), list(Kind::Windows, &windows));
    f.insert(
        "windowRulesManaged".into(),
        json!(file_has(&windows, Kind::Windows.begin())),
    );
    f.insert(
        "windowOpaque".into(),
        json!(file_has(&windows, "-- atmos:windowOpaque = true")),
    );
}
