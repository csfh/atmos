# Agent notes

Atmos is a standalone Quickshell preferences app for Omarchy. Do not import `qs.Ui` or `qs.Commons` from `$OMARCHY_PATH/shell`.

## Run

- `./bin/atmos` — launch (each launch is its own window)
- `npm install` — oxlint and oxfmt; also sets `core.hooksPath` to `.githooks`
- `npm run lint` / `npm run fmt` — lint and format `services` and `tests`
- `./tests/run` — oxlint, oxfmt --check, `scripts/*.py` syntax, parser tests, plus a live snapshot check when `omarchy` is present
- pre-commit (`.githooks/pre-commit`) — oxlint, oxfmt --check, and `tests/compile-python`; skip with `git commit --no-verify`
- GitHub Actions (`.github/workflows/tests.yml`) — `npm ci` and `./tests/run` on pull requests and on `main` / `alpha`. Live snapshot stays skipped without `omarchy`.

## Rules

- Mutations go through `omarchy` commands, `scripts/set-idle.sh` which sources `omarchy-shell-config`, or the Hyprland sentinel writers `scripts/set-hypr-look.sh`, `scripts/set-hypr-input.sh`, `scripts/set-hypr-autostart.sh`, `scripts/set-hypr-bindings.sh`, `scripts/set-hypr-windows.sh`, and `scripts/set-hyprsunset.sh`.
- Do not write a private prefs store.
- Hyprland drop-in is `~/.config/hypr/atmos.lua` required as `hypr.atmos` next to `hypr.omafetch`, before `default.hypr.toggles`. `hypr.atmos_layout` wraps dwindle `layoutmsg` so scrolling workspaces do not throw. Sentinel blocks are `-- atmos:look|input|autostart|bindings|windows|workspaces begin/end` in `atmos.lua` / `looknfeel.lua` / `input.lua` / `autostart.lua` / `bindings.lua`, and `-- atmos:monitors begin/end` in `monitors.lua`.
- Theme colors come from `~/.local/state/omarchy/current/theme/{colors,shell}.toml` and `~/.config/omarchy/shell.toml`.
- Diagnostics inventory is `scripts/diag-inventory.py`. Copy/save/agent append `omarchy-debug --no-sudo --print` via `scripts/diag-report.sh`.
- Keep parsers in `services/*.js` so Node can test them without Quickshell.
- Do not import `qs.Ui`. Restyle Qt Quick Controls through `Prefs*` wrappers. Visual language lives in `services/Theme.qml`; use those tokens instead of one-off sizes.
- Do not launch floating terminals for settings work. Long jobs use `Omarchy.runJob` or an in-page `Process`.
- Concurrent Atmos windows are allowed. Writers serialize with flock + atomic rename (`hypr-sentinel.py` `_patch` merge, shared `shell.json` lock, and sidecar locks on env/hyprsunset/tweaks/avatar/presentation). Do not reintroduce `instance-lock.sh`, `lostInstanceLock`, `quickshell -n`, or launcher `focus_existing`.
