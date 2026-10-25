# 0013. The window is Rust, built on ODK's components alone

Status: Accepted. Date: 2026-10-24.

## Context

The window is QML on Quickshell, and everything else is Rust. ADRs 0005, 0008, 0010 and 0012 made the window a thin client: it holds what the backend reports, sends typed requests, and does nothing without the daemon. That left the window with no logic a toolkit change would lose, which is the condition for changing the toolkit.

ODK (`../odk`) is a Rust design system on GPUI with its own controls kit, theme layer and bundle format. It follows the user's Omarchy theme by itself, and an application built on it can be packed as an `.oma` bundle. `docs/odk-primitives.md` lists what Atmos needs from it, and `docs/odk-components-handoff.md` the components that were missing. Most are built now; Resizable and Data Table are not.

## Decision

The new window is a Rust program in `frontend/` (`atmos-ui`), a Cargo root of its own that depends on ODK by path.

- **It uses ODK's components and nothing else.** Every view is built from `odk::prelude`: no `odk::gpui`, no hand-set colour or size, no bare element, no component modified in place. What a view needs and ODK lacks is added to ODK, as `sidebar_frame`, `content_column` and `scroll_y` were. `frontend/tests/purity.rs` reads `frontend/src` and fails on any of it.
- **It reaches the backend one way.** `frontend/src/client.rs` keeps one `ratmos serve` per window, as `services/Backend.qml` does, and is the only file that starts a process or reads from one. A request returns a future that resolves once; what the server pushes arrives on one channel that a task awaits, so nothing polls. The way to the server is a `Transport`, and only `ServeProcess` knows it is a child process.
- **It copies nothing it cannot check.** The ops it sends, the hubs and their order, titles and groups are pinned to `request.rs`, `serve.rs`, `launch.rs`, `Hubs.js` and `Layout.js` by `frontend/tests/pins.rs`.
- **It replaces the QML window page by page.** This record covers the shell and the client. A page is ported by giving the hub a view; when every hub has one, the QML window, its tests and its launcher are removed in a record of their own. Until then both run, against the same `ratmos serve`, and nothing in `bin/atmos` or `ratmos launch` changes.

## Consequences

- Building the window needs ODK checked out beside Atmos (`../odk`), and GPUI's first build is long. `tests/run` and the pre-commit hook skip the frontend, and say so, when `../odk` is absent; GitHub Actions therefore does not build it.
- The window could not at first be packed as an `.oma` bundle: a sandboxed application has no permission to run a program outside `/app` or to reach `ratmos.sock`, and it sees only a private home. ADR 0014 settles it, with a daemon the manifest declares (ODK's ADR 0050).
- `WindowOptions` carries its title and minimum size as GPUI types the purity rule forbids, so the window sets its `app_id` and nothing else. ODK has to wrap them.
- The Rust window lists the hubs from a copy of `Hubs.js`. Search and the page index stay in `Hubs.js` until the QML window goes.

## Alternatives considered

- **Keep QML.** It works, but Quickshell's Bluetooth, Networking and dialog modules, the Qt platform theme and the per-window QML engine are what Atmos carries to avoid owning a toolkit, and ODK now is that toolkit.
- **Put the window in ODK's repository.** ODK is the design system, and OMA keeps no application's code. Atmos's pages, its backend and its protocol belong with Atmos.
- **Speak `ratmos.sock` directly and drop `serve`.** It removes a process, and ADR 0012 already makes `serve` a bridge that reads nothing itself. It would give the window the daemon's framing and session handling to carry, and nothing is gained until a sandbox forbids the child process.
- **Allow `odk::gpui` for what the prelude lacks.** Every gap would then be filled in the application, and the next application would fill it again.

## Enforced by

- `frontend/tests/purity.rs`
- `frontend/tests/pins.rs`
- `frontend/tests/client.rs`
- `frontend/src/client.rs`
