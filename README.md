# Atmos

A [Quickshell](https://quickshell.org) preferences window for [Omarchy](https://omarchy.org). It follows the active theme and writes through `omarchy` commands and Hyprland drop-ins. Settings live in `~/.config`.

## Run

```bash
./bin/atmos
```

A second launch focuses the window that is already open. Pass a page if you want to land somewhere specific:

```bash
./bin/atmos appearance
./bin/atmos windows/bindings
./bin/atmos network/wifi
```

Hubs include appearance, displays, hardware, windows, input, accessibility, sound, capture, disks, bar, notifications, defaults, applications, software, network, power, idle, security, hooks, and system.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/csfh/atmos/alpha/install.sh | bash
```

That copies the launcher into `~/.local/bin`, the QML app into `~/.local/share/atmos`, and the desktop file plus Hypr drop-in into the usual XDG config dirs. From a clone, `./install.sh` does the same thing. System → Atmos follows the **alpha** git branch for Check and Update.

## Tests

```bash
./tests/run
```

## License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 Christoffer Hallas.

If you send a pull request or other contribution, you agree to the [CLA](CLA.md).
