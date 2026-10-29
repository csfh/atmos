# 0021. A page not yet ported is drawn from its QML rows

Status: Accepted. Date: 2026-10-28.

## Context

ADR 0013 replaces the QML window page by page, and until now a hub without a port showed its name and "not in the new window yet". Five hubs were ported, each with a logic module, a parity test against its `services/*.js`, and a page test. 29 hubs and their 18 pages were left, and the Rust window could not show those pages at all: `show` resolved every page to its hub.

Most of what those pages hold is settings rows, and most of those rows write through a few generic paths in `Omarchy.qml`: `writeHyprLook` and `writeHyprInput` for a field of the Hyprland look or input, `set` for a key of its own, and a typed write with fixed arguments. ODK has a switch, a select, a number field, a text field and buttons, which is everything those rows need. What ODK lacked was the rest: charts and sparklines, a data table, a file chooser, a resizable panel; its `main` has since added a file chooser, a data table, a resizable panel, an image, and a chart that is a canvas the application draws in named inks. The QML also builds some rows from models at run time (a Wi-Fi network, a process) and has rows with setters of their own.

The words of every row are already copied once and checked: `search.query` reads each QML page's literal labels, descriptions, hints and keywords, and ADR 0020 makes the Rust window search the rows its pages declare.

## Decision

Every hub and hub page that is not ported is drawn from its QML rows, and a row gets its control when the window can do what the QML row does.

- **The catalog is the QML pages' rows, in order.** `frontend/src/catalog/` holds each unported page (by its `Hubs.js` id and file): its groups in order, and in each group the rows whose words are literals in the QML, the rows `search.query` reads. A page draws them under its own description (`pages/catalog.rs`), with a star on every row. A hub with pages shows them as tabs, so `monitor/cpu` opens the CPU page and Favorites' Open goes to the page a row is on.
- **A row is mapped when what it sends can be checked under Node.** A row's `Control` names the path its QML row takes: a field of the look or input, a key of its own, a typed write, a link to another page, or a setter of `Omarchy.qml` of its own (`setChargeLimit`, `toggleAudioOutputMute`, `captureText`) with the checks it makes and what it sends: a setting, a typed write, or a job for the window's one queue, after a confirm or a prompt for one checked line of text if the QML asks one. `available:` and `enabled:` are conditions on what the snapshot says and on whether the window runs a job (`!Omarchy.jobBusy`). A QML slider is a number field, as on Sentinel.
- **Page logic is the page module's.** What a generic control cannot hold (a dialog, a draft the page validates, a confirm, options or words the page builds, page-local state, a row built from a model) is a `Custom` row or a list (`LISTS`) that the page's module draws (`pages/<page>.rs`, its logic in `src/<page>.rs`). A group whose QML builds rows from a model (`MODEL_LISTS`, held to the QML) draws them only through a list of its own, before the group's rows, after them, or after the row the QML puts them after (`List::follows`), and a page is whole only when every such group has one. A write the QML guards with the revert bar opens the window's bar (`guard.rs`, `Settings::send_guarded`), a write that needs sudo waits for sudo mode (`Jobs::write_as_root`), Preview holds writes in the window's history (`Settings` log), a file the QML asks for comes from ODK's file chooser (`pages/files.rs`), and a page that takes in what its job printed takes each end, or each cancel at the sudo prompt, once (`Jobs::ended`, `Jobs::cancelled`). A chart the QML draws on its canvas is laid out as `Charts.js` lays it out (`charts.rs`, `layouts.rs`, `cpu_charts.rs`, `memory_charts.rs`, `treemap.rs`) and drawn on ODK's `Chart`. What the QML window reads from Quickshell, the nearby Wi-Fi networks and Bluetooth devices, this one reads from `atmosd`'s scans, and the face and the wallpaper from `host.image` (`radios.rs`, held to `RichUi.js`, the QML pages' own functions and the recorded replies). The Export page's Omafile is `Settings.js` itself (`omafile/`, its table a copy of `settingsCatalog()`).
- **Words the QML builds are drawn.** One the QML picks by a condition is `Line::varies`; a count, a value inside the text, or a job's running text is `Line::words` (`catalog/words.rs`); a custom row that builds its own says so (`Custom::speaks`). A group the QML hides while a condition is false (`query: cond ? root.query : "."`) is in `catalog::SHOWN` and drawn only while one of its rows is or the condition holds.
- **What waits is said.** A row the window cannot do yet says "Not yet" where its control would be, and its page says its rows marked Not yet are changed in the QML window for now. A page drawn whole but for a part ODK or the backend cannot give yet (a key capture, an image read) is in `catalog::BLOCKED` with what it waits on, and says that instead; it is not whole.
- **A mapped row is the QML row.** `frontend/tests/catalog_controls.rs` reads each mapped row's QML block (its path, range, step, rounding, choices, labels and conditions, both ways), runs `clampLook`, `clampInput` and `Snapshot.clampPatchedValue` under Node on the same snapshot values, and runs the QML row's path to the backend (its rounding and no-op check, then `writeHyprLook`, `writeHyprInput` or `set`, then `Settings.commandFor` and `Requests.settingsSet`) for every value it can send. A row that calls a setter of its own is held by `frontend/tests/catalog_setters.rs`, which runs the setter itself: every function of `Omarchy.qml` is evaluated under Node with its imports, its properties read from a snapshot, and its ways to the backend recording what they send. A custom row's calls are samples it names (`Custom::samples`, `catalog/sample_pages.rs`) that the same test runs. Each page module is held by its own `frontend/tests/catalog_<page>.rs`, which runs the QML page's own properties and functions (`common::qml::PAGE_ROOT`) and its `services/*.js` under Node against the module, and pins the QML lines it follows.
- **A catalog page holds what it reads.** The page holds the topics of the keys its rows, lists, words and hidden groups read, the inventories its lists and custom rows read, and the favorites list for its stars. A page that draws the live samples (Home, Monitor) holds `live` too, at the interval its Sampling chips pick, and not while paused; the window keeps the last minute of samples as `LiveStatsStore` does (`feed.rs`, `stats.rs`). A group the QML keeps out of the catalog (`catalog: false`) that the window draws all the same is in `catalog::UNLISTED`.
- **The catalog is not a port.** `app::PORTED` stays the hubs with a page of their own, and a hub is in one or the other. A port replaces the hub's catalog page with its own page, as ADR 0013 has it, and the QML window goes only when `PORTED` holds every hub.

## Consequences

- The Rust window shows every page, and every row on a generic path works in it. Which pages are whole is the catalog's to say (`Page::mapped`), not this record's.
- Search in the Rust window finds every row the QML search finds, since the catalog's rows are in its catalog (ADR 0020).
- A row's words are copied a third time, into the catalog, and held to the QML both ways by `frontend/tests/catalog_parity.rs`: a QML page that gains, loses or rewords a row fails until its catalog page follows.
- A description the QML builds at run time comes out of `search.query` empty, and so it is in the catalog. The window draws it once what it is built from is read: `Line::varies` is held to the QML expression by `catalog_controls.rs`, `Line::words` by `catalog_words.rs`, which runs each row's own QML description in its page's scope, and `catalog_parity.rs` fails a whole page with a QML description none of them draws. A page whose own description is built so shows its hub's.
- `settings::readback` names the extra keys a script setting reads back (`ntp` reads `ntpSynchronized` too), as `Settings.commandFor` does; the controls test found the one that was missing.

## Alternatives considered

- **Port each page as the first five were.** It is the end state, but each port is a logic module and parity tests of its own. The catalog shows every page now, and a port can still replace one.
- **Leave the unported hubs as their names.** The Rust window could not be used to look at, or find, most of the app.
- **Restate each setter in the test, as `tweaks_parity.rs` restates its glue.** It checks the restatement, not the setter; running `Omarchy.qml`'s own functions checks the code the QML window runs.
- **Draw a row the QML builds from a model, with its model.** That is the page's logic and data, which is a port.

## Enforced by

- `frontend/tests/catalog_parity.rs`
- `frontend/tests/catalog_controls.rs`
- `frontend/tests/catalog_setters.rs`
- `frontend/tests/catalog_words.rs`
- `frontend/tests/catalog_accounts.rs`
- `frontend/tests/catalog_appearance.rs`
- `frontend/tests/catalog_bar.rs`
- `frontend/tests/catalog_bindings.rs`
- `frontend/tests/catalog_bluetooth.rs`
- `frontend/tests/catalog_capture.rs`
- `frontend/tests/catalog_custom.rs`
- `frontend/tests/catalog_diagnostics.rs`
- `frontend/tests/catalog_disks.rs`
- `frontend/tests/catalog_display.rs`
- `frontend/tests/catalog_export.rs`
- `frontend/tests/catalog_guard.rs`
- `frontend/tests/catalog_hardware.rs`
- `frontend/tests/catalog_history.rs`
- `frontend/tests/catalog_hooks.rs`
- `frontend/tests/catalog_input.rs`
- `frontend/tests/catalog_launchers.rs`
- `frontend/tests/catalog_lists.rs`
- `frontend/tests/catalog_network.rs`
- `frontend/tests/catalog_picks.rs`
- `frontend/tests/catalog_radios.rs`
- `frontend/tests/catalog_record.rs`
- `frontend/tests/catalog_software.rs`
- `frontend/tests/catalog_speedtest.rs`
- `frontend/tests/catalog_system.rs`
- `frontend/tests/catalog_units.rs`
- `frontend/tests/catalog_wifi.rs`
- `frontend/tests/catalog_page.rs`
- `frontend/tests/stats_parity.rs`
- `frontend/tests/processes_parity.rs`
- `frontend/tests/monitor_parity.rs`
- `frontend/tests/meters_parity.rs`
- `frontend/tests/charts_parity.rs`
- `frontend/tests/cpu_charts_parity.rs`
- `frontend/tests/memory_charts_parity.rs`
- `frontend/tests/omafile_parity.rs`
- `frontend/tests/pins.rs`
- `frontend/src/catalog/mod.rs`
- `frontend/src/catalog/control.rs`
- `frontend/src/catalog/setter.rs`
- `frontend/src/catalog/words.rs`
