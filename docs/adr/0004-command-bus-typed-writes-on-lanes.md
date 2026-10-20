# 0004. A command bus: typed writes on lanes, with barriers and leases

Status: Accepted. Date: 2026-10-19.

## Context

Writes ran as `ratmos apply -- <argv>` jobs that the window queued for itself. Nothing ordered a write against a read in the same window, let alone another window, a dragged slider needed its own coalescing, and the backend never learned what a write meant. A DDC/CI monitor answers a read right after a write with the old level, so reading "after" was not enough either.

## Decision

A write is a **typed command**. Two shapes exist today: `display.brightness` and `display.scale`, and `write {name, args}`, whose registry (`backend/src/command/writes.rs`) names every other write. An entry checks its arguments (nothing that could pass as an option, a path confined where it must be), builds the command line, names its **lane**, says which snapshot keys to read back, and says what a settled read-back looks like. The window sends a name and arguments, never a command line.

The command bus (`backend/src/sampler/commands.rs`):

- Commands on one lane run in order. Different lanes run in parallel.
- A lane may coalesce: a command queued behind only commands of its own key is replaced by the newest, and every caller still gets an answer, flagged `coalesced`. Registry writes never coalesce, because two toggles are two toggles. Brightness and scale do.
- A command names the topics it affects. When it finishes the daemon runs the invalidation hook, and only then lets readers go.
- A **barrier** holds a read of an affected topic until nothing pending touches it, for at most three seconds. `settings.snapshot` waits on `settings`, and `display.get` on its kind.
- A **lease** is a lane held by a caller that runs its own process tree (a sudo step, a stream, a terminal). It is a connection that stays open, so a holder that dies releases the lane. `ratmos task` leases a lane of its own, and `ratmos apply --` leases the shared `generic` lane for up to 300 ms, so writes the daemon has no typed op for still go one at a time and still hold back reads of what they change.

The daemon does not run the tool itself. It starts `ratmos request` as a child with the **caller's environment** and a `RATMOS_DIRECT` marker, so a daemon that outlives a Hyprland session never runs `hyprctl` with a stale `HYPRLAND_INSTANCE_SIGNATURE`. A privileged write asks for sudo mode first, so `sudo -n` in the child needs no session. A command the daemon took and then did not answer is a failure for the caller and is never run a second time.

After the command exits, the backend reads the touched keys back, and a write the machine finishes late is waited for: up to 1.5 s by default, 10 s for a radio or a device, until the read-back shows what the entry expects (`settled`). If it never does, the reply says `settled: false` and carries what the machine does show, and the window reads the owning group again.

## Consequences

- A read after a write sees the write, across windows.
- Jobs and interactive tools are typed too and run in the daemon on the same lanes (ADR 0009). `apply -- <argv>` and `ratmos task` remain for the terminal; they lease a lane while they run.
- A write is slower on a saturated machine than the same write inline, because the child runs at the daemon's priority.
- Adding a write means adding a registry entry and its read-back keys. `tests/writes-registry.test.js` fails on a name or key the backend does not own, and on arguments the window sends that the entry refuses.

## Alternatives considered

- **Ordering in the window's own job queue.** It cannot order across windows.
- **Running every command inside the daemon process.** It runs tools with a stale environment, and one hung tool would hold the daemon.
- **A generic "run this argv on a lane" op.** It is the passthrough again, with no validation and no read-back.

## Enforced by

- `backend/src/sampler/commands.rs` and `ops.rs` (lanes, coalescing, barriers, leases)
- `backend/src/command/mod.rs`, `args.rs`, `writes.rs`, `writes_tests.rs` (registry, validation, settle)
- `backend/tests/command.rs` (end to end with fake tools, including a panel that lags and writes racing on one monitor)
- `tests/writes-registry.test.js`, `tests/golden.test.js`
- `docs/protocol.md` (a row for every write name, checked by `writes_tests.rs`)
