# 0015. A cached settings topic is sent only when its files have not moved

Status: Accepted. Date: 2026-10-25.

## Context

ADR 0011 made the settings snapshot five topics the daemon holds: collected when a window first holds one and nothing is cached, and again when something invalidates it while it is held. One of the triggers is the daemon's file watcher, which looked once a second at the files the topics are read from, but only while a window held a topic. While nothing was held it forgot what it had seen, and its first look afterwards only remembered. A new hold was then sent the cached topic as it was ("as good as its last invalidation").

So a change made outside Atmos while no window held the topic was never seen: the next window was sent the old value, and kept it until something else invalidated the topic. The QML window holds all five topics for its whole life, so this rarely showed. The Rust window holds a topic only while a page that reads it is shown (ADR 0013), and a Bundle's window is often the only one (ADR 0014), so it showed every time a page was opened again after a change made elsewhere.

The watcher also skipped a change it saw in the second after a write the daemon ran, taking the write for the change. A change someone else made in that second, or at any time while nothing was held if the daemon had run any write meanwhile, was lost the same way.

The user's rule is that the daemon always has data that is up to date.

## Decision

A settings topic is never sent to a window from the cache unless the files it is read from are unchanged since the daemon last looked, and every collection of a topic begins after a look.

- **The daemon keeps the files' signatures.** They live in the daemon's settings state, not in the watcher's loop, and are kept while nothing holds a topic. The daemon takes them when it starts, before any topic is collected.
- **A hold looks first.** When a window's `sub` names a settings topic, the daemon looks at the files before it sends anything. A topic whose files moved, held or not, is invalidated: its cached sample is dropped, so it is not sent, and a held one is collected again and pushed. The window is sent the new sample, never the old one followed by a correction.
- **The watcher looks once a second while a topic is held,** and does nothing while none is, as before. It invalidates every topic whose files moved, whoever moved them: a write the daemon ran is read again too. The look and the invalidation happen under one lock, so a hold that looks a moment later finds the topic already dropped.
- **Nothing a previous daemon cached is trusted.** A daemon drops every cached settings topic when it starts, as it already did (ADR 0011); the cache dir outlives a restart, the daemon's signatures do not.

This amends ADR 0011's "Collected on demand" and "The backend owns the triggers", and its consequence that a change outside Atmos to a watched file shows at the next invalidation: it now shows at the next hold, or within a second while held. A change to something that is not a watched file is still seen only at the next invalidation.

## Consequences

- A page that is opened again shows what the files say now. Opening it after a change costs one collection of the topic, which a window waits on.
- Every hold of a settings topic costs one look at every watched file, one `stat` per path with directories walked to a bounded depth. A `sub` that names no settings topic costs nothing.
- A write the daemon runs to a watched file while its topic is held is followed, within a second, by a collection of that topic, where before the read-back patch was the only update. It costs a child process per topic per second of writes, at most. A tool that has not caught up with the file by then (Hyprland reloading its config) would make that collection show the value from before the write until the next change; a read-back carries the same risk today. This has not been seen in a live session.
- The rule covers files only. Keys whose truth is a running process (Hyprland's options, NetworkManager, BlueZ, PipeWire, systemd units) go stale between invalidations as before (ADR 0011). Keeping those up to date is a decision of its own.
- Signatures are a file's size and its modification time in whole seconds. Two changes of the same size within one second look like one.

## Alternatives considered

- **Store each file's signature with the cached sample and check it when the sample is served.** It makes a cached sample self-describing, which would let a restarted daemon keep it. A restarted daemon already drops the topics, and a collection reads more than files, so a sample kept across a restart would be trusted for its files and not for its tools.
- **Invalidate every settings topic on every new hold.** Always right, and a full collection of each topic every time a page is opened, whether anything changed or not.
- **Hold the settings topics for the window's whole life, as the QML window does.** It hides the gap for one window and leaves it for the first window after none, and keeps the daemon's watcher running for topics no page shows.
- **Keep skipping the change after the daemon's own write.** It saves a collection per write, and loses a change someone else made in the same second.

## Enforced by

- `backend/src/sampler/settings.rs` (the signatures are kept in the daemon's state; `look_at_files` invalidates whoever moved a file; the watcher only looks while a topic is held)
- `backend/src/sampler/session.rs` (a `sub` that names a settings topic looks at the files before anything is sent)
- `backend/src/sampler/daemon.rs` (the signatures are taken at start; cached settings topics are dropped at start)
- `backend/src/sampler/daemon_tests.rs` (a file that moved while nothing held its topic is read again before the topic is sent; a topic an earlier daemon cached is never sent; a change while held is pushed)
- `backend/tests/serve.rs` (the same through `ratmos serve` and a real daemon, across a restart)
