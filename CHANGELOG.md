# Changelog

Notable changes to Atmos. Each section is a git tag on `main` and `alpha`. Install and in-app Update follow the `alpha` branch.

## [v0.0.1-alpha.13] - 2026-09-07

### Added

- **General** hub. System, Tweaks, and Omafile live there. Admin is Accounts, Security, and Services.
- **Omafile**. A shareable Markdown settings document. Identity starts off. Security is never imported.
- **Kernel** under System. Running kernel, Direct EFI/UKI boot, and swappiness. Direct EFI boot left Boot Advanced.
- Unique Remix line icon for each hub.

### Changed

- Idle is titled Idle.
- Tweaks Pointer rows share one Pointer section. Swappiness is on Kernel.
- Services is a compact list with search, filters, and a context menu.
- Named workspaces: one Open on login select, a bar name with Set, and a monitor picker from connected outputs.
- Night light schedule is the two times. Use schedule is the on switch.

### Fixed

- Atmos did not start on alpha.12: `inputLuaView` was a bare child of a `QtObject`. ([#24](https://github.com/csfh/atmos/pull/24)) by Fred Nix ([@nixfred](https://github.com/nixfred)).
- Extra-theme selector vanished after snapshot load because SettingRow walked child visibility while the row was hidden.
- Reset rows hid their button when the setting was already default.
- Hyprland 0.56.2 has no `cursor.warp_on_focus_change`. Atmos no longer writes it. Cursor follows focus is gone from Windows.
- Named workspace cards repeated persistence copy and treated Open on login as ten independent toggles.

## [v0.0.1-alpha.12] - 2026-09-07

### Added

- **Diagnostics** under System. One page for Omarchy, Atmos, Hyprland, failed units, disk, memory, kernel, GPU, portals, PipeWire, network, pacman, and recent errors. Copy report puts an Atmos summary plus `omarchy-debug --no-sudo --print` on the clipboard. Save and Ask my Agent use the same text.
- **Workspaces** hub. Count (1–10), names, persistent rules, monitor assignment, default-on-login, scratch workspaces, wrap switching, and Super+mouse-wheel. Writes `-- atmos:workspaces` in `~/.config/hypr/atmos.lua`.
- **Keybindings** as a Controls hub. Search by category, add or reset an Atmos override, show the generated `hl.unbind` / `o.bind` text, and Record shortcut from a key press.
- **Bluetooth** as a Machine hub (Network still links to it). Radio, discoverable, pair, connect, disconnect, forget, trust, and battery when BlueZ reports it.
- **Startup** under Applications. Enable without delete, delay (`sleep N &&`), and failed units this boot. Still `o.launch_on_start` in the autostart sentinel.
- **Tweaks** (Admin). Overflow settings that name the file they write and have Reset: flat pointer accel, natural scroll, middle-click paste, Electron Wayland, XWayland zero scaling, swappiness.
- **Services** (Admin). Failed units always listed. Start, stop, restart, enable, disable only for an allowlist (PipeWire, portals, NetworkManager, Bluetooth, CUPS, Docker, Tailscale, TRIM timer). Others stay status and logs.
- **Environment** under System. Read-only session values plus a user overlay in `~/.config/environment.d/10-atmos.conf` (PATH prepend and extra vars).
- **Profiles**. Coding, Gaming, and Battery apply through `Settings.commandFor`. Your own profiles are Markdown import documents.
- **Presentation Mode** on Power. Stay awake, do-not-disturb, screensaver off for the session (`~/.local/state/omarchy/atmos-presentation.json`).
- Per-output monitor rules in `~/.config/hypr/monitors.lua`: mode, scale, rotation, disable without deleting, VRR, bit depth, color. Desk, Laptop, and Docked layouts apply from the live outputs.
- Network extras on Wi-Fi and Network: this-link gateway and DNS, saved-connection metered / priority / MAC / static IPv4, WireGuard import, hotspot when the adapter can AP.
- Power read-only CPU governor and energy preference. Charge limit when the kernel exposes `charge_control_end_threshold`.
- Window leftovers: swallow, swallow regex, cursor-follows-focus, focus-under-fullscreen. Window rules can match title and set pin, fullscreen, opacity.

### Changed

- Sidebar order puts Workspaces and Profiles with the desktop hubs, Keybindings with Controls, Bluetooth with Machine, Tweaks and Services with Admin.
- Displays scale and mode apply per output, not only the focused monitor.

### Fixed

- Shrinking the workspace count no longer refilled 4–10. `commandFor` and `hypr-sentinel.py` now carry `count` (or derive it from numbered ids).

## [v0.0.1-alpha.11] - 2026-09-06

### Changed

- Settings import and the live controls share one write dispatcher (`Settings.commandFor`). Import still runs the same `omarchy` and `set-*.sh` writers as the matching row. ([#22](https://github.com/csfh/atmos/pull/22))

### Fixed

- Installing or updating extra themes now refreshes the Installed themes list. Nested clones under `~/.config/omarchy/themes` are watched with inotifywait. ([#22](https://github.com/csfh/atmos/pull/22))
- Omarchy replacing `~/.local/state/omarchy/current/theme` is followed from `Theme.qml`. The 800ms theme poll is gone. ([#22](https://github.com/csfh/atmos/pull/22))

## [v0.0.1-alpha.10] - 2026-09-05

### Fixed

- Choosing an avatar from a user-only FUSE mount (rclone without `allow_other`) failed on the privileged AccountsService copy. The image is staged locally first. Based on work by Fred Nix ([@nixfred](https://github.com/nixfred)). ([#15](https://github.com/csfh/atmos/pull/15))
- Every `runGumJob` failed with `scripts/stubs: Is a directory`. The wrapper put the stub dir on `PATH` as `$1` and then `exec "$@"` still started at `$1`. It now `shift`s before exec. Fingerprint, FIDO2, sshd, sudoless docker, channel, `omarchy update`, firmware, orphans, and prune all go through this. Based on work by Fred Nix. ([#19](https://github.com/csfh/atmos/pull/19), [#17](https://github.com/csfh/atmos/pull/17))
- A GUI import hung and blocked the queue. `runJob` wrote the plan to stdin and never closed it, and `apply-settings.sh` waits for EOF. ([#20](https://github.com/csfh/atmos/pull/20), [#18](https://github.com/csfh/atmos/pull/18))
- A second export wrote an empty file and reported success. `writeProc` cleared `stdinEnabled` for `cat` and never restored it. ([#20](https://github.com/csfh/atmos/pull/20))
- Importing `barVisible = false` showed the bar. `omarchy toggle bar` takes the state being turned off. ([#20](https://github.com/csfh/atmos/pull/20))
- Muting a device and setting its volume in one import left it unmuted. `set-audio.sh` unmutes before a volume write, and mute now runs after. ([#20](https://github.com/csfh/atmos/pull/20))

## [v0.0.1-alpha.9] - 2026-09-05

### Added

- Shared setting layout. `SettingRow` is the row: hairline splits, trailing On/Off status for values you cannot change here, captions, and an optional leading checkbox. `PrefsRow` stays an alias. `CollectionRow` is the list row (Install, Enable, Edit, Remove). `PrefsCheck` is the export inclusion box. `PrefsSliderStepper` is the compact slider plus numeric stepper used for text size and cursor size.
- Visual tokens in `Theme.qml` (type scale, control sizes, motion, `copyInset`). Page title, section heading, and setting label share one left edge.
- `hypr.atmos_layout` wraps dwindle `layoutmsg` (`togglesplit`, `swapsplit`, `preselect`) so Super+J on a scrolling workspace does not throw. Install copies `packaging/hypr-atmos-layout.lua` to `~/.config/hypr/atmos_layout.lua` and requires it before Omarchy defaults.
- Groups **Manage…** opens a membership dialog. You add or remove logins there instead of a Select action on the group row.
- Import and export cover more Hypr look and input keys (blur, shadow, dim, animations, column width, scroll inertia, scroll speed, layouts, and related).
- Night light warmth is written into a live `hyprsunset.conf` night profile, so a restart keeps the Kelvin you set.
- Mouse button 8 (back) pops a nested page, the same as the header chevron and Escape. Error and sudo dialogs keep the button. ([#14](https://github.com/csfh/atmos/pull/14))

### Changed

- Read-only booleans such as Secure Boot and lid close show On/Off status instead of a disabled switch.
- Empty collections say so (no hidden/pinned bar icons, no reminders waiting, no scripts) instead of a blank control or a second create button.
- Toggle labels and descriptions describe the same direction. Show bar is "Keep the bar visible. Turn this off to hide it."
- Volume, brightness, and gaps keep a full-width slider. Text size and cursor size use the compact slider plus stepper. Tick labels on the wide slider are thinned so they do not collide.
- Generic Open buttons name the destination: Configure…, Manage…, Choose…, Test…, Open folder.
- Section `?` only appears when the group has extra copy beyond the row text.
- List Remove stays visible. It used to appear on hover when Edit was already on the row, which is why a flat `theme-set` hook flashed Remove… as the pointer crossed it.
- After a touchpad or high-resolution wheel throw, the pane coasts with the velocity from that gesture instead of stopping the moment the events stop. Notch wheels stay stepped and stop with the clicks. Based on work by Fred Nix ([@nixfred](https://github.com/nixfred)). ([#13](https://github.com/csfh/atmos/pull/13))

### Fixed

- SettingRow toggles could stay hidden after child controls landed. The row now tracks `children.length`.
- Sentinel parsers skip commented autostart, bind, and window examples, keep live calls with comments inside the table, and treat Lua `\n` in a string as a newline.
- Importing a clock format onto a bar on the left or right writes `verticalFormat`, which is the key the bar reads.
- Typed fields (hostname, weather, SSH key, Hyprland layouts, full name) validate before Set, and Set stays off when the value is invalid or unchanged.

## [v0.0.1-alpha.8] - 2026-09-04

### Added

- **Import and export** hub. Atmos writes this machine's settings to a Markdown file with fenced `atmos:` blocks, then reviews a file before applying it. Cosmetic and behavior settings travel. Hostname, timezone, locale, and keyboard layout are off unless you opt in. sshd, passwordless sudo, sudoless docker, Snapper, and TRIM are written as prose and have no importer. Apply goes through the existing `omarchy` and `set-*.sh` writers and sudo mode. Based on work by Fred Nix ([@nixfred](https://github.com/nixfred)). ([#9](https://github.com/csfh/atmos/pull/9), [#7](https://github.com/csfh/atmos/issues/7))

### Fixed

- Nearby Wi-Fi and Bluetooth lists stay empty when Quickshell's device objects appear after the page first loads. The Wi-Fi page keeps the scanner on while it is open. Bluetooth paired and nearby rows come from the live BlueZ list. Turning a radio on queues a network snapshot.
- Touchpad and mouse-wheel scrolling in prefs panes. One `PrefsFlickable` handles the wheel on the viewport. `WheelHandler` never received those events here. Based on work by Fred Nix. ([#8](https://github.com/csfh/atmos/pull/8), [#6](https://github.com/csfh/atmos/issues/6))
- The wheel `MouseArea` used `anchors.fill` on a Flickable child, which Qt left at 0×0. It now binds `width` and `height` to the viewport.
- `set-idle.sh` and `set-bar-widget.sh` default `OMARCHY_PATH` before sourcing `omarchy-shell-config`, so a write outside a desktop session still works. ([#10](https://github.com/csfh/atmos/issues/10))
- A failed `hyprctl reload` no longer fails a sentinel write that already landed. ([#11](https://github.com/csfh/atmos/issues/11))
- Failed `omarchy` jobs show stdout as well as stderr, because the CLI prints errors on stdout. ([#12](https://github.com/csfh/atmos/issues/12))

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

[v0.0.1-alpha.13]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.12...v0.0.1-alpha.13
[v0.0.1-alpha.12]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.11...v0.0.1-alpha.12
[v0.0.1-alpha.11]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.10...v0.0.1-alpha.11
[v0.0.1-alpha.10]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.9...v0.0.1-alpha.10
[v0.0.1-alpha.9]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.8...v0.0.1-alpha.9
[v0.0.1-alpha.8]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.7...v0.0.1-alpha.8
[v0.0.1-alpha.7]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.6...v0.0.1-alpha.7
[v0.0.1-alpha.6]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.5...v0.0.1-alpha.6
[v0.0.1-alpha.5]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.4...v0.0.1-alpha.5
[v0.0.1-alpha.4]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.3...v0.0.1-alpha.4
[v0.0.1-alpha.3]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.2...v0.0.1-alpha.3
[v0.0.1-alpha.2]: https://github.com/csfh/atmos/compare/v0.0.1-alpha.1...v0.0.1-alpha.2
[v0.0.1-alpha.1]: https://github.com/csfh/atmos/releases/tag/v0.0.1-alpha.1
