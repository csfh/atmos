# 0008. Every read and every write goes through the backend

Status: Accepted. Date: 2026-10-18.

## Context

The window is QML. QML could run any host command, read any file and keep any value it likes. When it did, the logic that decided what was true lived in two languages with different tests, and the bugs that cost the most (the empty monitor list, a stale read overwriting a write) were seams between them.

## Decision

**Every read and every write runs through the backend.** QML reaches `ratmos` only through `services/Backend.qml`, with bodies from `services/Requests.js`. It does not run host commands itself, read host files, poll with a timer, or decide what the machine's state is.

- Reads are a request, a held topic (`sub`), or a watch (`watch`).
- Writes are typed (0004), and so are jobs and interactive tools, which run in the daemon (0009). The window builds no command line and starts no process.
- Rust is the owner of what ratmos does: errors are `crate::error::Error`, child processes go through `crate::runner::Run`, files are replaced with `crate::fsutil::atomic_write`, and a path the user chose goes through `host::confine`. Parsers stay in `services/*.js` so Node can test them without Quickshell.
- Privileged steps re-exec as `ratmos root <task>` through `sudo` (the window asks for sudo mode first), as part of a daemon job.

## Consequences

- One language owns what is true, one set of tests covers it, and the window can be restyled or replaced without touching it.
- A new feature that reads or writes the host needs a backend op before it needs a page.
- `tests/bus-conformance.test.js` counts what still bypasses the buses and fails if a count rises; every count is zero for commands, jobs, interactive tools and processes outside `Backend.qml`. The settings snapshot is held as topics and the inventories are held too, so the window reads neither (ADR 0011; `displayReads`, `settingsSnapshotReads` and `enqueueReads` are zero). `settings.set`, `host.write`, `host.open` and `agents.mcp.set` are commands on lanes of the daemon like every other write.

## Alternatives considered

- **Let QML run the safe commands.** The line between safe and not moves with every feature.
- **A thin QML layer over a script per action.** It is the shape this replaced.

## Enforced by

- `services/Backend.qml` (the only place a `Process` starts the backend besides `IoQueue`)
- `tests/qml-isolation.test.js`, `tests/backend.test.js` (QML reaches the backend only through its seam)
- `tests/requests.test.js`, `tests/protocol-doc.test.js` (the op lists agree)
- `AGENTS.md` (the rules a change must follow)
