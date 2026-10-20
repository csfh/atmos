# 0005. The window shows what the backend reports and never a guess

Status: Accepted. Date: 2026-10-19.

## Context

After a write the window used to patch its own snapshot with the value it expected (`job.apply`). That made controls feel instant, and it was a second place that decided what was true. Its patches crossed a QML signal, which hands arrays back wrapped, and a second brightness change once emptied the monitor list. A patch could also say a change had happened that the machine then refused or finished differently.

## Decision

The window does not mutate its own state with a value it worked out. After a write it shows what the backend read back:

- a typed write answers with the keys it can change, as the machine reports them (`result.snapshot`), and `Ops.run` applies that;
- `settings.set` takes `readback`, a list of snapshot keys to read after the write, and answers with them;
- a write run by an interactive tool is followed by `snapshot.keys`, with `expect` where the machine finishes late;
- values come from the same producers as the full snapshot (`backend/src/snapshot/keys.rs`), so a key has the same shape either way.

Rules that keep this honest:

- Only the reply to the newest write of a key is applied. Overlapping replies for one key are collapsed by the backend, and the window ignores the rest.
- A full snapshot read that began before a write is not trusted for the keys the write set (`Ops.dropStale`), so a slow read cannot put back an old value.
- A reply that signals `settled: false` makes the window read the owning group again.
- A reply crosses a QML signal as JSON text or is copied with `plain()`, which restores real arrays.
- `runCommand` still takes an `argv`, but only as a label for the history and the Preview list. The backend builds what runs.

## Consequences

- A toggle changes when the answer arrives, usually 0.2 to 1 s, and longer for network writes. This is the accepted cost.
- The window carries no patch builders. `Snapshot.js` has none, and `job.apply` is gone.
- Two derived flags are still set by the window: `passwordlessSudo`, after the sudo-mode job exits cleanly, and the favorites list, which returns from its file's stamp.

## Alternatives considered

- **Keep the optimistic patch and correct it with the read-back.** It fixes the final state, not the window in between, and it keeps the second source of truth.
- **Read the whole snapshot group after every write.** It costs seconds and CPU per toggle; a keyed read is 0.3 s against 4.4 s.

## Enforced by

- `services/Ops.qml`, `services/Omarchy.qml` (`mutationAnswered`, `enqueueIo`)
- `tests/repo.test.js` (no `applyWritePatch`, no `job.apply`, no patch in `installTheme`, `dropStale` wired)
- `tests/writes-registry.test.js` (every setting carries a value and read-back keys the backend owns)
- `tests/settings.test.js`, `tests/snapshot-arraylike.test.js`
- `AGENTS.md` (the rule for a new write)
