# Atmos

[![Tests](https://github.com/csfh/atmos/actions/workflows/tests.yml/badge.svg)](https://github.com/csfh/atmos/actions/workflows/tests.yml)
[![Latest release](https://img.shields.io/github/v/release/csfh/atmos?include_prereleases)](https://github.com/csfh/atmos/releases)
[![License: MIT](https://img.shields.io/github/license/csfh/atmos)](LICENSE)

**Preferences for Omarchy — themes, the bar, network, power, and the rest of this machine.**

A standalone [Quickshell](https://quickshell.org) preferences window for [Omarchy](https://omarchy.org). It follows the active theme and writes through `omarchy` commands and Hyprland drop-ins. There is no private prefs store: settings live in `~/.config`.

[![Atmos Home hub with live processor, memory, network, and temperature](docs/screenshots/home.png)](https://atmos.csfh.dev)

Site: [atmos.csfh.dev](https://atmos.csfh.dev) · source for it in [`site/`](site/).

<p>
  <img src="docs/screenshots/monitor.png" alt="Atmos Monitor hub with live cores, memory, traffic, disk I/O, and sensors" width="360" />
  <img src="docs/screenshots/appearance.png" alt="Atmos Appearance page with the installed Omarchy theme grid" width="360" />
</p>

## Features

- **Theme-aware chrome** from the current Omarchy theme. Hover a theme to preview it in this window.
- **Writes through `omarchy` and Hyprland sentinels**, not a second store.
- **Home and Monitor** — live load, cores, memory, disk I/O, traffic, sensors, processes. Samples stay in the window.
- **Search and keyboard** — `/` or `Ctrl+F` finds a setting; `Ctrl+/` opens the shortcut sheet.
- **Concurrent windows** stay in sync through the files they share. Writes serialize on disk.
- **Diagnostics, Omafile, agents** — copy a machine report, share settings as Markdown, install `atmos mcp`.

Hubs: Home, Monitor, Favorites, Appearance, Displays, Windows, Workspaces, Bar, Notifications, Profiles, Input, Keybindings, Accessibility, Sound, Capture, Hardware, Drivers, Disks, Network, Bluetooth, Power, Idle, Defaults, Agentic, Applications, Software, Hooks, System, Tweaks, Omafile, Accounts, Security, Services.

## Requirements

- An [Omarchy](https://omarchy.org) system (Hyprland).
- `git`, `python3`, `cargo`, and `quickshell` (`install.sh` checks these).
- `~/.local/bin` on `PATH`.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/csfh/atmos/stable/install.sh | bash
```

That copies the launcher into `~/.local/bin`, the app into `~/.local/share/atmos`, a desktop file, and the Hyprland drop-in. From a clone, `./install.sh` does the same thing.

Then run `atmos`, or open it from the app launcher.

### From the Omarchy package repo

```bash
sudo pacman -S omarchy-atmos
atmos --setup
```

The package installs under `/usr/lib/atmos`. Hyprland integration (the window seed and the `hypr.atmos` requires) is opt-in: `atmos --setup` adds it, `atmos --setup off` removes it, and `atmos --setup status` prints `on` or `off`. A package install updates with `omarchy update`, and System → Atmos hides Channel and Update. If you installed with `install.sh` first, remove `~/.local/share/atmos`, `~/.local/bin/atmos` and `~/.local/share/applications/atmos.desktop` so they do not shadow the package.

## Update

**System → Atmos** Check / Update follows the **stable** git branch. Re-running the install command does the same fetch-and-stage. If the window looks the same afterward, quit Atmos and open it again. Update also repairs an install that is missing the launcher, backend, or shell.

## Usage

```bash
atmos
atmos appearance
atmos windows/bindings
atmos network/wifi
```

Every launch opens its own window. Pass a hub or child page to land there. From a source checkout, `./bin/atmos` is the same launcher.

## Configuration

Atmos does not write a private prefs store. It reads and writes Omarchy and Hyprland files under `~/.config`.

- Hyprland drop-in: `~/.config/hypr/atmos.lua`, required as `hypr.atmos` from `hyprland.lua` next to `hypr.omafetch`, before `default.hypr.toggles`. `hypr.atmos_layout` wraps dwindle `layoutmsg` so scrolling workspaces do not throw.
- Sentinel blocks: `-- atmos:look|input|autostart|bindings|windows|workspaces begin/end` in the matching Lua files, and `-- atmos:monitors begin/end` in `monitors.lua`.
- Theme colors: `~/.local/state/omarchy/current/theme/{colors,shell}.toml` and `~/.config/omarchy/shell.toml`.
- Install channel: `~/.config/atmos/channel` (only `stable` exists; an old `alpha` reads as `stable`).
- **System → Atmos → Reset** strips Atmos-managed Hypr overrides. Theme, wallpaper, and `shell.json` stay as they are.

## Troubleshooting

**`atmos` is not found.** The wrapper lives in `~/.local/bin`. Add that directory to `PATH`, or run `~/.local/share/atmos/bin/atmos`.

**Hyprland config-reload warnings.** After a look, input, monitor, or workspace write, Atmos runs `hyprctl reload`. A failed reload does not undo the write. If Hyprland already has config errors, those writers print `hyprctl configerrors` and the apply can look failed even though the file changed. **System → Diagnostics** lists the errors. A dirty config still runs; the errors are why a bind or monitor rule did not apply.

**`hypr.atmos` is missing / the window does not tile.** Install writes the drop-in and the require. Diagnostics reports when the require is absent. Re-run `install.sh`, or restore Hyprland defaults from System (the Atmos require is written back).

**Display or input change snapped back.** Mode, scale, rotation, refresh, keyboard layout, and turning the touchpad off revert after 12 seconds unless you press Keep. Escape reverts too. Quitting during those 12 seconds leaves the new value.

**Install says a tool is missing.** `install.sh` needs `git`, `python3`, `cargo`, and `quickshell`. `atmos mcp` also needs `node`.

**Need a report.** **System → Diagnostics → Copy report** puts an Atmos summary plus `omarchy-debug --no-sudo --print` on the clipboard.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). A pull request agrees to the [CLA](CLA.md).

```bash
npm install
./tests/run
```

`./tests/run` runs oxlint, oxfmt --check, `scripts/*.py` syntax, and parser tests. `npm install` points Git at `.githooks/`. Pull requests and pushes to `main` and `stable` run the same suite on GitHub Actions.

## License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 Christoffer Hallas.

UI glyphs in `icons/` are from [Remix Icon](https://remixicon.com), Remix Icon License v1.0. See [icons/NOTICE](icons/NOTICE).
