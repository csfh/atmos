//! The setter samples of the pages with the most of them (`Custom::samples` in `samples.rs`).

use serde_json::{Value, json};

use crate::software;

/// A setter, the snapshot it runs on, its arguments, and what it sends.
pub type Sample = (&'static str, Value, Value, Option<Value>);

/// What the System page's own rows call.
pub(super) fn system() -> Vec<Sample> {
    use crate::system::{self, Branding};
    let job = |j: Option<crate::jobs::Job>| j.map(|j| software::job_body(&j));
    let doc = json!({
        "hostname": "box", "omarchyChannel": "stable", "atmosChannel": "stable",
        "weatherAuto": false, "weatherLocation": "Oslo", "weatherCoords": "59.9,10.7",
        "aboutBranded": true, "printerSetup": true,
    });
    let auto = json!({ "weatherAuto": true, "weatherLocation": "", "aboutBranded": false, "printerSetup": false });
    let mut out = Vec::new();
    for l in ["de", "us", "fr,us", "Bad", " dk "] {
        let body = system::layout_write(l, "us").map(|(b, _)| b);
        out.push((
            "setKeyboardLayout",
            json!({ "keyboardLayout": "us" }),
            json!([l]),
            body,
        ));
    }
    for h in [" laptop ", "box", "-bad", "a..b", "my-host.lan", ""] {
        out.push((
            "setHostname",
            doc.clone(),
            json!([h]),
            system::hostname_write(h, "box"),
        ));
    }
    for c in ["rc", "stable", "dev", "nightly"] {
        out.push((
            "setOmarchyChannel",
            doc.clone(),
            json!([c]),
            job(system::channel_job(c, "stable")),
        ));
    }
    out.push((
        "checkOmarchyUpdate",
        doc.clone(),
        json!([]),
        job(Some(system::omarchy_check())),
    ));
    out.push((
        "runOmarchyUpdate",
        doc.clone(),
        json!([]),
        job(Some(system::omarchy_update())),
    ));
    out.push((
        "checkAtmosUpdate",
        doc.clone(),
        json!([]),
        job(Some(system::atmos_check())),
    ));
    out.push((
        "runAtmosUpdate",
        doc.clone(),
        json!([]),
        job(Some(system::atmos_update())),
    ));
    for c in ["stable", "alpha", "edge"] {
        out.push((
            "setAtmosChannel",
            doc.clone(),
            json!([c]),
            system::atmos_channel_write(c, "stable"),
        ));
    }
    out.push((
        "openPrinters",
        doc.clone(),
        json!([]),
        Some(system::printers_write(true)),
    ));
    out.push((
        "openPrinters",
        auto.clone(),
        json!([]),
        Some(system::printers_write(false)),
    ));
    out.push((
        "openCupsAdmin",
        doc.clone(),
        json!([]),
        Some(system::cups_write()),
    ));
    for (d, branded) in [(&doc, true), (&auto, false)] {
        for a in ["image", "text", "reset", "other"] {
            for (which, setter) in [
                ("about", "setAboutBranding"),
                ("screensaver", "setScreensaverBranding"),
            ] {
                let body = match system::branding(which, a, branded) {
                    Some(Branding::Job(j)) => Some(software::job_body(&j)),
                    Some(Branding::Write(w)) => Some(w),
                    None => None,
                };
                let mut d = d.clone();
                d["screensaverBranded"] = json!(branded);
                out.push((setter, d, json!([a]), body));
            }
        }
    }
    for (d, auto_on, cur) in [(&doc, false, "Oslo"), (&auto, true, "")] {
        for l in [" Bergen ", "Oslo", "-x", ""] {
            out.push((
                "setWeatherLocation",
                d.clone(),
                json!([l]),
                system::location_write(l, cur, auto_on),
            ));
        }
        out.push((
            "clearWeatherLocation",
            d.clone(),
            json!([]),
            system::auto_write(auto_on),
        ));
    }
    for c in ["60.4, 5.3", "+59.9,10.7", "91,0", "x"] {
        out.push((
            "setWeatherCoordinates",
            doc.clone(),
            json!([c]),
            system::coords_write(c, "Oslo", false, "59.9,10.7"),
        ));
    }
    out
}

/// What the Bar page's own rows call.
pub(super) fn bar() -> Vec<Sample> {
    use crate::bar;
    let doc = json!({
        "trayHidden": ["a/x", "b"], "trayPinned": [], "indicatorsItems": ["Dnd", "Reminder"],
        "agentsRefreshIntervalSec": 900, "agentsSyncFileName": "box.json", "agentsSyncDeviceId": "box",
        "clockFormat": "HH:mm", "clockWeekStart": "monday", "clockBirthYear": 1990,
        "clockLifeExpectancy": 0, "plugins": [{"id": "weather", "enabled": true}, {"id": "x.y", "enabled": false}],
    });
    let k = |n: &str| doc.get(n).cloned();
    let mut out = vec![
        (
            "clearTrayHidden",
            doc.clone(),
            json!([]),
            bar::clear_tray("trayHidden", k("trayHidden").as_ref()),
        ),
        (
            "clearTrayPinned",
            doc.clone(),
            json!([]),
            bar::clear_tray("trayPinned", k("trayPinned").as_ref()),
        ),
    ];
    for list in [
        json!(["StayAwake", "Dnd"]),
        json!(["Reminder", "Dnd"]),
        json!(bar::INDICATORS.iter().map(|(i, _)| *i).collect::<Vec<_>>()),
    ] {
        let names: Vec<String> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        out.push((
            "setIndicatorsItems",
            doc.clone(),
            json!([list]),
            bar::indicators_write(
                &names,
                Some(&json!(bar::indicator_items(k("indicatorsItems").as_ref()))),
            ),
        ));
    }
    for s in [600.0, 900.0, 20.0, 3600.0] {
        out.push((
            "setAgentsRefreshIntervalSec",
            doc.clone(),
            json!([s]),
            bar::seconds_write(s, k("agentsRefreshIntervalSec").as_ref()),
        ));
    }
    for raw in [" /home/u/Sync ", ""] {
        out.push((
            "setAgentsSyncDir",
            doc.clone(),
            json!([raw]),
            bar::sync_dir_write(raw, k("agentsSyncDir").as_ref()),
        ));
    }
    for raw in [" sync/laptop.json ", "box.json", ""] {
        out.push((
            "setAgentsSyncFileName",
            doc.clone(),
            json!([raw]),
            bar::sync_file_write(raw, k("agentsSyncFileName").as_ref()),
        ));
        out.push((
            "setAgentsSyncDeviceId",
            doc.clone(),
            json!([raw]),
            bar::device_write(raw, k("agentsSyncDeviceId").as_ref()),
        ));
    }
    for v in ["h:mm AP", "dd\nMMM"] {
        out.push((
            "set",
            doc.clone(),
            json!(["clockFormat", v]),
            Some(crate::settings::write("clockFormat", &json!(v))),
        ));
    }
    for d in ["Sunday", "monday", "funday", ""] {
        out.push((
            "setClockWeekStart",
            doc.clone(),
            json!([d]),
            bar::week_write(d, k("clockWeekStart").as_ref()),
        ));
    }
    let now = bar::this_year();
    for y in [
        json!(1985),
        json!("1990"),
        json!(""),
        json!(now - 121),
        json!(now + 1),
        json!("19x5"),
        json!(2000.4),
    ] {
        out.push((
            "setClockBirthYear",
            doc.clone(),
            json!([y]),
            bar::birth_write(&y, k("clockBirthYear").as_ref(), now),
        ));
    }
    for y in [json!(80), json!(""), json!(151), json!("90"), json!(0)] {
        out.push((
            "setClockLifeExpectancy",
            doc.clone(),
            json!([y]),
            bar::life_write(&y, k("clockLifeExpectancy").as_ref()),
        ));
    }
    for (id, on) in [
        ("weather", false),
        ("weather", true),
        ("x.y", true),
        ("bad id", true),
        ("new", true),
    ] {
        out.push((
            "setPluginEnabled",
            doc.clone(),
            json!([id, on]),
            bar::plugin_write(id, on, k("plugins").as_ref()),
        ));
    }
    out
}

/// What the Hooks page's buttons and dialog call.
pub(super) fn hooks() -> Vec<Sample> {
    use crate::hooks;
    let home = "/home/u/.config/omarchy/hooks/theme-set.d";
    let mut out = Vec::new();
    for (t, p) in [
        ("theme-set", "/home/u/notify.sh"),
        ("my-hook", "/tmp/x.sh"),
        ("Bad", "/x"),
        ("post-boot", "rel"),
        ("post-boot", "/a/../b"),
    ] {
        out.push((
            "installHook",
            json!({}),
            json!([t, p]),
            hooks::install_write(t, p),
        ));
    }
    for (t, n, c) in [
        ("font-set", "notify", "echo $1"),
        ("font-set", "a.py", "x"),
        ("font-set", "x.sample", "y"),
        ("font-set", "ok", ""),
    ] {
        out.push((
            "createHook",
            json!({}),
            json!([t, n, c]),
            hooks::create_write(t, n, c),
        ));
    }
    for (name, arg) in [("theme-set", " tokyo "), ("post-boot", ""), ("Bad", "")] {
        out.push((
            "runHook",
            json!({}),
            json!([name, arg]),
            hooks::run_write(name, arg),
        ));
    }
    for t in ["battery-low", "nope!"] {
        out.push((
            "openHookFolder",
            json!({}),
            json!([t]),
            hooks::folder_write(t),
        ));
    }
    // The QML keeps these to `$HOME/.config/omarchy/hooks/`; the test's `$HOME` is `/home/u`.
    for p in [format!("{home}/a.sh"), format!("{home}/a.sh.sample")] {
        out.push(("removeHook", json!({}), json!([p]), hooks::remove_write(&p)));
        out.push((
            "setHookSample",
            json!({}),
            json!([p, true]),
            hooks::sample_write(&p, true),
        ));
        out.push(("editHook", json!({}), json!([p]), hooks::edit_write(&p)));
    }
    out
}

/// Snapper, TRIM, rollback, hibernation and the passphrase, as the page sends them.
pub(super) fn disks() -> Vec<Sample> {
    use crate::disks;
    let doc = json!({"snapperNumberLimit": 5, "snapperTimeline": false, "fstrimEnabled": true});
    let (limit, timeline, trim) = (json!(5), json!(false), json!(true));
    let body = |j: Option<crate::jobs::Job>| j.map(|j| software::job_body(&j));
    let mut out = Vec::new();
    for n in [5.0, 12.4, 0.0, 51.0, 50.0] {
        out.push((
            "setSnapperNumberLimit",
            doc.clone(),
            json!([n]),
            disks::number_limit(n, Some(&limit)),
        ));
    }
    for on in [true, false] {
        out.push((
            "setSnapperTimeline",
            doc.clone(),
            json!([on]),
            disks::flag("snapper.timeline", on, Some(&timeline)),
        ));
        out.push((
            "setFstrim",
            doc.clone(),
            json!([on]),
            disks::flag("fstrim", on, Some(&trim)),
        ));
    }
    for (config, id) in [("root", "12"), ("home", "x"), ("a b", "3"), ("", "1")] {
        out.push((
            "restoreSnapshot",
            doc.clone(),
            json!([config, id]),
            body(disks::rollback(config, id)),
        ));
    }
    out.push((
        "setupHibernation",
        doc.clone(),
        json!([]),
        body(Some(disks::hibernation(false))),
    ));
    out.push((
        "removeHibernation",
        doc.clone(),
        json!([]),
        body(Some(disks::hibernation(true))),
    ));
    for (device, cur, new) in [
        ("/dev/nvme0n1p2", "old", "new"),
        ("sda", "a", "b"),
        ("/dev/x", "", "b"),
    ] {
        let job = disks::change_key(device, cur, new, new).ok().flatten();
        out.push((
            "changeDrivePassword",
            doc.clone(),
            json!([device, cur, new]),
            body(job),
        ));
    }
    out
}

/// Copy and Ask send the report the inventory makes.
pub(super) fn diagnostics() -> Vec<Sample> {
    let raw = crate::catalog::samples::DIAGNOSTICS.clone();
    let doc = json!({ "diagnostics": raw });
    let body =
        |action| crate::diagnostics::report_job(Some(&raw), action).map(|j| software::job_body(&j));
    let mut out = vec![
        ("copyDiagnosticReport", doc.clone(), json!([]), body("copy")),
        (
            "askAgentAboutDiagnostics",
            doc.clone(),
            json!([]),
            body("agent"),
        ),
    ];
    // Save hands the setter the path with `.txt` (the page's `onAccepted`).
    for path in ["/home/u/r.txt", "/home/u/r", "rel.txt"] {
        let saved = crate::diagnostics::save_job(Some(&raw), path);
        let arg = saved.as_ref().map_or(path.to_owned(), |(p, _)| p.clone());
        out.push((
            "saveDiagnosticReport",
            doc.clone(),
            json!([arg]),
            saved.map(|(_, j)| software::job_body(&j)),
        ));
    }
    out
}
