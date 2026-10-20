# 0012. `ratmos serve` is a bridge

Status: Accepted. Date: 2026-10-19.

## Context

After ADR 0010 a window did nothing without the daemon, but `serve` still answered some requests in its own process: it read settings files, `host.*`, ran `agents` checks and speed tests, built the settings snapshot and `snapshot.keys`, and made `search.query` and `platform` itself. Those ran with the window's environment, which was right, and with no order against the daemon's writes except a barrier, which was a patch over the real problem: the daemon was not the only thing that touched the machine. A frontend that talks to the daemon directly would have had to carry all of that.

## Decision

A window's `serve` hands every request to the daemon and relays the answer. The daemon runs it as a command in a child `ratmos request` that carries the window's environment (ADR 0004), so a daemon that outlives a login never runs a tool with a stale session.

- **A write** keeps the lane its entry names, as before.
- **A read** has a lane of its own, so reads never wait for each other. It still waits at the barrier for the writes ahead of it, which `serve` asks for first.
- **Three requests stay in `serve`** because they only ask the daemon for something it already holds: `display.get`, `display.snapshot` and `snapshot.invalidate`.
- **The daemon counts** the requests it ran for windows (`commandsRun` in `status`), so a test can tell who answered.
- **At most four read children run at once,** so a page that asks once per item (the theme grid) queues instead of starting a process for each. A speed test is a read that holds its child for seconds on purpose and takes no slot.
- **The window's app root travels with its environment** (`ATMOS_ROOT`, set from `platform::app_root()` when nothing set it), because the daemon may be another copy of the program, and `search.query` reads the app's own page files.
- **A daemon that cannot run reads is refused.** A session's `ready` frame carries `proto`, and `serve` reports `backend down` for a daemon that reports less than `sampler::PROTOCOL` or nothing, so the window shows its screen instead of empty pages.
- `ratmos request`, `ratmos display` and a `--root` fixture still run inline: they are the terminal's and the tests'.

## Consequences

- `serve` reads no file, runs no tool and collects nothing. What it owns is the framing QML speaks, the window's session to the daemon, and the fixtures.
- A frontend that speaks to the daemon directly gets every op, with the same order and the same environment handling.
- `version` and `platform` are the daemon's. Nothing in the window asks for them.
- A read costs a process (a few milliseconds), where it was a thread. `search.query` is the one a window sends often; it is sent once per change of the text.
- The child runs at the daemon's priority and with the 120 s limit of every command, so a read is slower on a saturated machine and a disk speed test that would need longer is cut off.
- A `snapshot.keys` or `settings.snapshot` that a window asks for is read by a child with the window's environment, the same as a topic.

## Alternatives considered

- **Run the cheap reads in the daemon's own process.** Settings files and `search.query` need no environment, but two paths for the same op is what ADR 0010 removed, and a read that runs a tool then differs from one that does not.
- **Leave the reads in `serve`.** It works, and it leaves the daemon as the authority for writes only.

## Enforced by

- `backend/tests/serve.rs` (every op `request.rs` accepts, sent over a window's `serve`, is run by the daemon, and the three that are not are named)
- `backend/src/sampler/ops.rs` (reads run on lanes of their own and a nonsense request is refused before anything is queued)
- `backend/src/sampler/ops.rs` (a burst of reads never runs more children than there are slots)
- `backend/src/sampler/client.rs` (a window's environment names its app root)
- `backend/tests/serve.rs` (a window refuses a daemon that reports no protocol)
- `backend/src/command/mod.rs` (`read_plan`)
