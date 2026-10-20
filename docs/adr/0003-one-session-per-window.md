# 0003. One multiplexed session per window

Status: Accepted. Date: 2026-10-19.

## Context

The window used to poll with timers, each tick forking a process or asking a one-shot connection. Pushed state, ordered writes and "a read waits for the write before it" all need a connection that stays open and carries both directions.

## Decision

Every window keeps **one session** to the daemon, opened when `ratmos serve` starts, whether or not the window holds anything yet. It is a Unix socket carrying newline-delimited JSON frames:

- client to daemon: `sub` and `unsub` (display kinds held, each with its interval), `watch` (files, theme and accounts watched), `req` (any one-shot op, with an id);
- daemon to client: `event` (`{topic, sampledAt, tookMs, value}`, or `{topic, stale, error}`, and `stamp`, `chrome` and `accounts` for watched files) and `res` (the reply for an id).

`ratmos serve` is the bridge. It keeps the framing QML speaks (`docs/protocol.md`) and maps it onto the session. `client::Bridge` carries the window's commands and barriers as `req` and `res`, and falls back to the daemon's one-shot socket when the session is down.

Rules that keep the session sound:

- Every writer of a session goes through one locked writer that sends a frame as one line, so two frames never mix.
- A reply is written directly, never queued behind events. An event queue is bounded, and a window that falls behind is dropped, so one stalled window cannot hold up publishing.
- Each subscriber is throttled to its own interval by `sampledAt`, so a window that asked for two seconds is never fed one.
- A held topic that goes quiet for six intervals (and never less than 15 s) is reported as an error rather than freezing silently.
- With no daemon, or one that predates sessions, `serve` samples and looks at files itself on the same clock and sends the same events, so the window cannot tell. *Superseded by [0010](0010-nothing-works-without-the-daemon.md): a window says the backend is unavailable and keeps looking for a daemon; only a `--root` fixture still samples its own files.*

## Consequences

- The window has no polling timer for live data, files or accounts.
- The daemon must tell a session from a one-shot connection on the first line (`{"op":"session"}`), and a daemon older than a frame type refuses it. The client treats such a refusal as "nothing was queued" and runs the work itself. *A window now reports `unavailable` instead (0010); a terminal command still runs it.*
- File watching is a once-a-second poll in the daemon, shared by all windows (0007).

## Alternatives considered

- **One connection per subscription.** It multiplies sockets and loses ordering between a window's frames.
- **Server-sent events over HTTP.** It adds a server and a port for what a Unix socket already does.
- **A shared-memory ring.** It would be faster, and it is unnecessary at these rates.

## Enforced by

- `backend/src/sampler/session.rs`, `bus.rs`, `watch.rs`, `backend/src/serve_link.rs`, `backend/src/sampler/client.rs` (`Bridge`)
- `backend/tests/serve.rs` (live on the holder's clock, two windows at their own rates, a daemon that goes away, files over the daemon)
- `tests/serve-protocol.test.js`, `tests/golden.test.js` (the window's framing is unchanged)
- `tests/qml-isolation.test.js` (no `Timer` polls live data)
