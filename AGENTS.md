# Agent notes

Atmos is a standalone Quickshell preferences app for Omarchy. Do not import `qs.Ui` or `qs.Commons` from `$OMARCHY_PATH/shell`.

## Run

- `./bin/atmos` — launch (each launch is its own window)
- `npm install` — oxlint and oxfmt; also sets `core.hooksPath` to `.githooks`
- `npm run lint` / `npm run fmt` — lint and format `services` and `tests`
- `./tests/run` — oxlint, oxfmt --check, `scripts/*.py` syntax, parser tests, plus a live snapshot check when `omarchy` is present
- `tests/sandbox.js` — any test that runs a script or `ratmos` goes through `createSandbox()`: throwaway HOME/XDG, a PATH of fakes that record argv (`box.calls()`), and a guard that throws on the real home or a real tool. An unscripted fake exits 97. `backendWrapper()` gives an `ATMOS_BACKEND` pinned to a fixture `--root` (ratmos only; run the app itself under `box.env()` too).
- `tests/golden/` — recorded `ratmos` replies for every op, replayed over `request` and `serve` with bodies from `Requests.js`. After an intended protocol change: `ATMOS_GOLDEN_RECORD=1 node tests/golden.test.js`, then review the diff.
- `./tests/smoke [seconds]` — opt-in: launches the real app, checks the log is clean and that one `ratmos serve` runs with no idle one-shot requests. Needs a Wayland session; run it before tagging a release.
- pre-commit (`.githooks/pre-commit`) — oxlint, oxfmt --check, and `tests/compile-python`; skip with `git commit --no-verify`
- GitHub Actions (`.github/workflows/tests.yml`) — `npm ci` and `./tests/run` on pull requests and on `main` / `stable`. Live snapshot stays skipped without `omarchy`.

## Rules

- Mutations go through `omarchy` commands, `scripts/set-idle.sh` which sources `omarchy-shell-config`, or the Hyprland sentinel writers `scripts/set-hypr-look.sh`, `scripts/set-hypr-input.sh`, `scripts/set-hypr-autostart.sh`, `scripts/set-hypr-bindings.sh`, `scripts/set-hypr-windows.sh`, and `scripts/set-hyprsunset.sh`.
- Do not write a private prefs store.
- Hyprland drop-in is `~/.config/hypr/atmos.lua` required as `hypr.atmos` next to `hypr.omafetch`, before `default.hypr.toggles`. `hypr.atmos_layout` wraps dwindle `layoutmsg` so scrolling workspaces do not throw. Sentinel blocks are `-- atmos:look|input|autostart|bindings|windows|workspaces begin/end` in `atmos.lua` / `looknfeel.lua` / `input.lua` / `autostart.lua` / `bindings.lua`, and `-- atmos:monitors begin/end` in `monitors.lua`.
- A `PACKAGED` file in the app root marks a package-manager install: no Check/Update, no writes under the app root. Per-user Hyprland setup is opt-in through `atmos --setup` (`scripts/setup-atmos.sh`); never run it implicitly from the app.
- Theme colors come from `~/.local/state/omarchy/current/theme/{colors,shell}.toml` and `~/.config/omarchy/shell.toml`.
- Diagnostics inventory is `scripts/diag-inventory.py`. Copy/save/agent append `omarchy-debug --no-sudo --print` via `scripts/diag-report.sh`.
- Keep parsers in `services/*.js` so Node can test them without Quickshell.
- Do not import `qs.Ui`. Restyle Qt Quick Controls through `Prefs*` wrappers. Visual language lives in `services/Theme.qml`; use those tokens instead of one-off sizes.
- QML reaches `ratmos` only through `services/Backend.qml` with bodies from `services/Requests.js`. Do not add a `Process` that runs `ratmos request`, and do not poll with a `Timer`: watch through `Backend.watch` and listen for `Backend.stamp|chrome|accounts`. Long jobs that stream output stay on `ratmos apply`. A new op needs its `Request` variant in `backend/src/request.rs` and its builder in `Requests.js` (`tests/requests.test.js` checks the two lists match).
- Rust errors are `crate::error::Error` (kind + message + context), never `String`. Child processes go through `crate::runner::Run`, file replacement through `crate::fsutil::atomic_write`, and user-chosen paths through `host::confine`.
- Do not launch floating terminals for settings work. Long jobs use `Omarchy.runJob` or an in-page `Process`.
- Concurrent Atmos windows are allowed. Writers serialize with flock + atomic rename (`hypr-sentinel.py` `_patch` merge, shared `shell.json` lock, and sidecar locks on env/hyprsunset/tweaks/avatar/presentation). Do not reintroduce `instance-lock.sh`, `lostInstanceLock`, `quickshell -n`, or launcher `focus_existing`.
