# Changelog

Notable changes to Atmos. Each section is a git tag on `main` and `alpha`. Install and in-app Update follow the `alpha` branch.

## [v0.0.1-alpha.7] - 2026-09-04

### Added

- Failed settings work opens a centered **Error** dialog. Copy puts the message on the clipboard. Dismiss clears it. **Ask my Agent to work on this** launches the default coding agent with that error, the same path as Omarchy's crash toast (`omarchy agent prompt`).
- **Debug** at the bottom of System. Show error paints a sample `lastError` so you can try the dialog. Find a setting does not list it.
- GitHub Actions runs `./tests/run` on pull requests and on pushes to `main` and `alpha`. The live Omarchy snapshot step stays skipped on those runners.
- `tests/compile-python` parses `scripts/*.py` (ast and tabnanny) from pre-commit and `./tests/run`.

### Changed

- Appearance and the other look hubs (Displays, Windows, Bar, Notifications, Idle) read a look snapshot first, then `rest`. The look dump now includes bar, idle, Hypr look, monitors, DND, and reminders. Network, Disks, Accounts, and System each have their own snapshot group before `rest`.
- File watches enqueue `look`, `rest`, or `all` by path. A face or hostname change does not rerun hardware and disk inventory.
- Account name, host, face, and user lists live in `AccountsStore`. Omarchy still exposes the same properties. Those files are watched on the store, not through a full snapshot.
- Find a setting keeps one `SearchIndex.js serve` process for the pane and sends queries on stdin.
- Accessibility text size, animations, cursor hide/size, and touchscreen are shared `pages/rows` components used from Appearance, Windows, and Displays.
- Parser tests are split along `services/*.js`. `./tests/run` loops `tests/*.test.js`.
- The Face row always says Omarchy's greeter does not draw the file. The sidebar name and `user@host` lines keep a fixed height while snapshot data arrives.

## [v0.0.1-alpha.6] - 2026-09-04

### Added

- **Accounts** hub after Security. This account has a PNG or JPEG face (`~/.face.icon` and AccountsService), full name, and password. Users lists human logins (UID 1000 and up). Add copies `/etc/skel` and can put the login in wheel. Remove deletes the home directory. You cannot remove the session you are in. Groups shows wheel, docker, and extra groups. Select a group and toggle members. You cannot drop this session from wheel or delete wheel and docker. Omarchy's greeter still does not draw faces.
- Sidebar clusters with muted labels: Desktop, Controls, Machine, Apps, Admin. Remix line icons on each hub. Search flattens the list.
- Sidebar header: a round face, full name, and `user@hostname`. An empty face opens the image picker. The name opens Accounts.

### Changed

- Full name lives on Accounts. System stays the machine (hostname, locale, clock, updates).
- Root work asks for your password in an Atmos dialog and runs `sudo -S`. After that, sudo stays unlocked for the minutes on Security. pkexec is still the fallback when stdin cannot supply a password.

### Fixed

- Enabling sudo mode opened Polkit with the raw `bash -c` line from `set-passwordless-sudo.sh`.
- The sidebar photo stayed square because `clip` plus `radius` does not round an Image. It is masked with `MultiEffect` the same way Omarchy's image picker is.

## [v0.0.1-alpha.5] - 2026-09-03

### Added

- **Reset Atmos** on System. It strips Atmos-managed Hyprland look, input, autostart, bindings, and extra window-rule sentinels, keeps the floating window, and leaves theme, wallpaper, and `shell.json` alone. It also deletes `~/.cache/atmos/search.sqlite`.
- Remix Icon line SVGs for About, back, and row chevrons. They fill with the current theme (muted, foreground, or accent).
- **Scroll inertia** on Input → Pointer. Writes Hyprland `input.emulate_discrete_scroll` into the Atmos input sentinel: Smooth (`0`) keeps high-resolution wheel motion, Default (`1`) is Hyprland’s usual handling, Stepped (`2`) turns the wheel into clicks.
- A sudo-mode prompt before commands that need root (hostname, locale, timezone, NTP, layout, full name, pacman downloads, SSH, Snapper, TRIM, LUKS, snapshots, hibernation, updates, firmware, channel, Plymouth, fingerprint/FIDO2, Docker, Tailscale, Voxtype, hybrid GPU, software installs, and the other `pkexec` / `omarchy` root jobs). Confirming enables Omarchy passwordless sudo for the minutes set on Security. Scripts run through `scripts/as-root.sh` (passwordless `sudo` when that drop-in is on, otherwise `pkexec`).

### Changed

- Switching theme in Atmos paints from the theme’s `colors.toml` and `shell.toml` immediately, then starts `omarchy theme set` in the background the same way Omarchy’s Switch theme action does.
- Current theme maps `theme.name` slugs (`miasma`) onto the display names in the select (`Miasma`).
- Volume and display brightness sliders follow the pointer while dragged, keep the `%` caption, and write at most every 100ms, then again on release.
- An outside Omarchy Switch theme updates Atmos chrome and Current theme. FileView cannot follow `omarchy-theme-set` replacing `~/.local/state/omarchy/current/theme`, so Atmos watches that directory with `inotifywait` and re-reads `theme.name` on a short poll.

### Fixed

- Binding loop on `errorBanner` height (`errorText` no longer `anchors.fill`s the parent whose height it also drives). ([#4](https://github.com/csfh/atmos/pull/4), [#3](https://github.com/csfh/atmos/issues/3))
- `scripts/snapshot.sh` aborted under `set -euo pipefail` when bluetooth jq indexed `.address` against the connected-address array, or when a best-effort text-size, brightness, wifi-iface, or timezone probe failed. ([#5](https://github.com/csfh/atmos/pull/5), [#2](https://github.com/csfh/atmos/issues/2))
- Remix glyphs stayed off-theme because `Image` plus `MultiEffect` never saw `currentColor`. `PrefsIcon` draws the SVG paths as Qt Shapes and binds `fillColor`.
- Scroll inertia did not land in `~/.config/hypr/input.lua`. The live writer is `scripts/hypr-sentinel.py`, not the JS serializer used in tests.

## [v0.0.1-alpha.4] - 2026-09-03

### Added

- Settings search through a SQLite index of hub and `PrefsRow` haystacks (`node:sqlite` under `~/.cache/atmos/search.sqlite`, or `ATMOS_SEARCH_INDEX`). Find a setting no longer loads every page. Snapshot JSON can fill derived state in the same cache. The file is a cache only; settings still write through `omarchy` and Hyprland sentinels.
- oxlint and oxfmt on `services/` and `tests/`, run from `./tests/run`. Commits fail those checks via `.githooks/pre-commit` (`npm prepare` and `scripts/install-git-hooks.sh` set `core.hooksPath`).

### Changed

- Appearance (and other look hubs) queue a fast look snapshot first, then the full read, on the same lock as mutations. Partial JSON merges so a later read does not blank earlier fields. First paint no longer sets a window-wide busy flag.
- Every setting write goes through one I/O queue. Waiting writes coalesce by key. Toggles, selects, and spin boxes hold the chosen value until the worker applies it.

### Removed

- The global Updating banner. Controls enable from their own data and `jobBusy`.

### Fixed

- Brightness writes no longer kick a full snapshot that held the lock and overwrote the slider. Same-monitor writes coalesce; a snapshot that started before a later write is dropped.
- Sliders snapped back to the old model value on release before the queued write applied. The thumb holds the dropped value until `root.value` matches it.
- Theme/app removes, keyboard backlight, hooks, reminders, Wi-Fi QR, and reconstructed autostart, bindings, and windows lists were still assigning Omarchy properties off-queue.

## [v0.0.1-alpha.3] - 2026-09-03

### Fixed

- Super+Space (and the desktop file) opened nothing useful: `~/.local/bin/atmos` was a symlink, so the launcher treated `~/.local` as the app root. Install now writes a wrapper that `exec`s `~/.local/share/atmos/bin/atmos`, and the desktop `Exec`/`TryExec` point at that staged binary. `bin/atmos` resolves through `readlink -f` so a remaining symlink still finds `shell.qml`.

## [v0.0.1-alpha.2] - 2026-09-03

### Fixed

- A leftover `omarchy-prefs:input` sentinel could leave a second three-finger workspace swipe that shadowed the Atmos one. Applying or resetting the Atmos input block now strips those legacy look/input markers.

### Removed

- Injecting Atmos into Omarchy menu slots (`packaging/omarchy-menu.jsonc`). Install only stages the desktop launcher.

## [v0.0.1-alpha.1] - 2026-09-03

First public alpha. Standalone [Quickshell](https://quickshell.org) preferences window for [Omarchy](https://omarchy.org). Theme colors come from the current Omarchy theme. Mutations go through `omarchy` commands and Hyprland drop-ins (`~/.config/hypr/atmos.lua` with `-- atmos:* begin/end` sentinels). There is no private prefs store.

### Added

- Hubs: Appearance, Displays, Hardware, Windows, Input, Accessibility, Sound, Capture, Disks, Bar, Notifications, Defaults, Applications, Software, Network, Power, Idle and lock, Security, Hooks, System. Search is a Find-a-setting overlay over those hubs. Some hubs have subpages (background, boot, wifi, bluetooth, bindings, rules).
- Hardware inventory for GPU, CPU, NPU, memory, chipset, DMI, USB, NIC, and related units (`scripts/hw-inventory.py`). Disk inventory, Snapper, LUKS, and hibernation on Disks. Autostart on Applications.
- XDG install: `curl -fsSL https://raw.githubusercontent.com/csfh/atmos/alpha/install.sh | bash` (or `./install.sh` from a clone). Stages the app under `~/.local/share/atmos`, a launcher under `~/.local/bin`, a desktop file, and the Hyprland drop-in. System → Atmos Check/Update follows the **alpha** git branch.
- Keyboard use on controls (tab focus and activation). File watching of Omarchy/Hyprland paths so outside changes can refresh the snapshot. Shared page routing and content-column layout.
- MIT license. Contributions assign copyright to Christoffer Hallas ([CLA](CLA.md)).

[v0.0.1-alpha.7]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.6...v0.0.1-alpha.7
[v0.0.1-alpha.6]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.5...v0.0.1-alpha.6
[v0.0.1-alpha.5]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.4...v0.0.1-alpha.5
[v0.0.1-alpha.4]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.3...v0.0.1-alpha.4
[v0.0.1-alpha.3]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.2...v0.0.1-alpha.3
[v0.0.1-alpha.2]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.1...v0.0.1-alpha.2
[v0.0.1-alpha.1]: https://github.com/csfh/atmos/releases/tag/v0.0.1-alpha.1
