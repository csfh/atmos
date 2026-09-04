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

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for every alpha tag.

## Tests

```bash
npm install
./tests/run
```

`./tests/run` lints and checks formatting with [oxlint](https://oxc.rs/docs/guide/usage/linter) and [oxfmt](https://oxc.rs/docs/guide/usage/formatter), then runs the parser tests. `npm run lint` and `npm run fmt` run those tools on their own.

`npm install` points Git at `.githooks/`, so commits run oxlint and `oxfmt --check` first. `git commit --no-verify` skips that. From a clone that already has `node_modules`, run `scripts/install-git-hooks.sh`.

## License

MIT. See [LICENSE](LICENSE). Copyright (c) 2026 Christoffer Hallas.

UI glyphs in `icons/` are from [Remix Icon](https://remixicon.com), Remix Icon License v1.0. See [icons/NOTICE](icons/NOTICE).

If you send a pull request or other contribution, you agree to the [CLA](CLA.md).
