# 0010. Nothing works without the daemon

Status: Accepted. Date: 2026-10-19.

## Context

A window with no daemon used to carry on: `display::load` collected inline, `serve` ran a local sample pump and its own file watcher, a typed write ran in the caller's process (`route_command`), and `Backend.qml` fell back to one `ratmos request` process per call. Each fallback is a second implementation of something the daemon does, with different ordering, no governor and no shared cache, and each one hid a daemon that was not running. A jobs-in-the-daemon design (ADR 0009) cannot have such a fallback at all.

## Decision

The daemon is required. A window never collects, watches, or runs a write or job itself. When the daemon cannot be reached the window says so on one screen, "Atmos backend unavailable", keeps starting it (the ladder in `sampler/launch.rs`, again after one, two, four and then five seconds) and reconnects by itself when it answers. `--root` fixtures and terminal subcommands (`ratmos display`, `snapshot`, `request`, `task`, `apply`) may still run inline: they are for tests and for the terminal, not for a window.

How it is told apart: `ratmos serve` without `--root` on Omarchy is a window's server, and it calls `client::require`. From then on a typed write that finds no daemon, and a `display.get` that finds none, fail with the error kind `unavailable` instead of running in the process. `serve` says what it knows as an event, `{"event":"backend","state":"up"|"down","why","attempt","retryMs"}`, and `Backend.qml` turns it into `available`, which `components/BackendUnavailable.qml` shows. A request made while the server itself is not running is refused the same way.

## Consequences

- One implementation of every read and write, and one place its failures show.
- A machine where the daemon cannot start has a window that explains why and does nothing else.
- Deleted: the inline collection in `display::load` for a window, `serve`'s local sample pump and file loop (a fixture root keeps them), `route_command`'s inline run for a window, and `Backend.qml`'s one-shot process, poll timer and `parseEnvelope`. Jobs and interactive tools already needed the daemon (ADR 0009).
- A window whose daemon dies is told within a moment, shows the screen, and carries on by itself when a daemon answers: the server puts the held topics and watches to the new session, and the window attaches the jobs it follows again.
- The server and the daemon are two processes. The server hands every request to the daemon, which runs it in a child that carries the window's environment (ADR 0012); the three it answers itself only ask the daemon for what it holds. It answers only while a session to the daemon is up: a request that arrives while the window is still looking for one waits up to eight seconds for it, and with none it is refused with `unavailable`, so a setting cannot be changed behind the screen either.
- When the backend comes back, `Backend` raises `reconnected` and `Omarchy` attaches the jobs it follows again; the feeds hold their topics again by themselves and the daemon pushes what they show.
- Supersedes the fallback consequence of ADR 0001 and the "no daemon" consequence of ADR 0003.

## Alternatives considered

- **Keep the fallbacks as a safety net.** They are the code that is never exercised until the day it is needed, and they hide a broken daemon.
- **Make the window own a private daemon.** It breaks one authority across windows (ADR 0001).

## Enforced by

- `backend/tests/jobs.rs` (no daemon: a job is refused with `unavailable`)
- `backend/tests/serve.rs` (a window with no daemon refuses a read and a write and samples nothing; it says when the daemon goes, and carries on when one returns)
- `backend/tests/command.rs` (a window whose daemon dies refuses the write, then reconnects)
- `tests/qml-isolation.test.js` (no one-shot in `Backend.qml`; the unavailable screen is in the window)
