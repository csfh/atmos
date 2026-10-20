# 0011. The settings snapshot is held as topics

Status: Accepted. Date: 2026-10-19.

## Context

The five inventories were topics (ADR 0002), but the settings snapshot was not. The window read it in groups (`look`, `rest`, `all`, `network`, `disks`, `accounts`, `system`) whenever it thought something had changed: at startup, after a write, after a job, when a watched file moved, when a Refresh button was pressed. Each read ran `serve`'s own copy of every producer, `rest` and `all` overlapped `look`, a producer that could not run a tool wrote a default in its place (an empty list, `false`, a made-up `Disconnected`), and which group a key belonged to was decided in QML. A new frontend would have to copy all of that.

## Decision

The snapshot is held by the daemon as topics, and a window is pushed them like any other.

- **Five topics, disjoint, owned by the backend.** `settings.look`, `settings.network`, `settings.accounts`, `settings.system` and `settings.host`. Each producer in `snapshot/keys.rs` belongs to one topic, so each key does, and `SnapshotGroups.js` decides nothing. The list of topic names is separate from `display::KINDS`, so nothing that loops over the inventories collects settings by accident.
- **A failure is a missing key, never a default.** A producer whose tool did not run, timed out, or answered with an error leaves its keys out of the topic and names them in the topic's `failed` map with the reason. The window keeps the value it had (the rule monitors already followed, ADR 0006). A tool that exits non-zero as an honest answer (`omarchy toggle enabled` for "off") is still an answer.
- **Collected on demand, not on a clock.** A topic is collected when a window first holds it and nothing is cached, and again when something invalidates it while it is held. Nothing polls a whole group on an interval.
- **Collected with a current environment.** Producers call `hyprctl` and `omarchy`, which need the session's environment, and a daemon can outlive a session (ADR 0004). A window's `serve` sends its environment as the first frame of its session, and the daemon collects a topic in a child `ratmos request` that carries the newest one.
- **The backend owns the triggers.** A command or job names the topics it affects (`affects`). A typed write also patches the cached topic with the values it read back, so the window is not made to wait for a second collection of a topic for one key. The daemon watches the files the keys are read from. A `snapshot.invalidate` op is what a Refresh button sends.
- **A sample taken before a write never lands after it.** Every invalidation bumps the topic's generation, and a collection that started under an older one is neither cached nor published and is run again. This lives in the daemon so every frontend gets it.
- **Invalidating is one op for the window.** `snapshot.invalidate` names settings topics or inventories (not `live`); the window sends it for a Refresh button, and the daemon does the rest.
- **The window applies what is pushed.** `Backend.subscribe` holds the topics; a pushed sample is applied except for keys the window wrote after the sample was taken (ADR 0005). The window no longer reads the snapshot, runs a read queue, or turns a file stamp into a refresh.

`settings.snapshot` and `snapshot.keys` stay as ops, for the terminal, the golden tests and the read-back of a write; both are built from the same producers, and a window's request for either is run by the daemon (ADR 0012).

## Consequences

- One implementation of the snapshot, one place its failures show, and the window has no idea what a group is.
- A key a producer cannot read goes stale visibly instead of silently showing a made-up value.
- A change made outside Atmos to a file the daemon does not watch is seen at the next invalidation, as before.
- The daemon spawns a child to collect a topic, so a collection costs a process; it is the cost the write path already pays for the same reason.

## Alternatives considered

- **Poll each group on an interval.** It hides every trigger problem behind a timer and spends CPU on a machine that is not changing.
- **Collect in `ratmos sampler`.** The sampler has no window's environment and its collections are background work behind the governor, which is wrong for something a window is waiting on.
- **Keep the groups the window knows.** They overlap, and a stale `rest` would overwrite a newer `look`.

## Enforced by

- `backend/src/snapshot/topics.rs` (every key has one topic; topics are disjoint; the files the daemon watches)
- `backend/src/snapshot/keys.rs` (a producer without a topic does not compile)
- `backend/src/snapshot/sh.rs` (a tool that fails is a failure only where its answer is the value)
- `backend/src/sampler/settings.rs` (a sample that started before an invalidation is dropped; a write's read-back patches the cached topic; an unsettled one invalidates it; a held topic is collected again; the files that moved name the topics)
- `backend/src/sampler/session.rs` (a held topic that is not cached is collected when it is held)
- `backend/tests/snapshot_failure.rs` (a tool that fails leaves its keys out and says why)
- `backend/tests/serve.rs` (a window is pushed a settings topic and the next one after an invalidation, and sees a failure as a missing key)
- `backend/tests/command.rs` (a setting and a file write go through lanes of the daemon)
- `tests/bus-conformance.baseline` (`displayReads`, `settingsSnapshotReads` and `enqueueReads` are zero)
- `tests/snapshot-groups.test.js` (the window holds five topics and no group owns a key)
