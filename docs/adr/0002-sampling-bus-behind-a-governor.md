# 0002. A sampling bus behind a CPU governor, with background work in its own idle process

Status: Accepted. Date: 2026-10-18.

## Context

Atmos shows a lot of host state: live CPU and network, hardware, disks, services, software, diagnostics. Collecting it costs processes and CPU. The ask was to sample it eagerly and repeatedly, so pages open on data that is already there, and never to take CPU from what the user is doing.

## Decision

State a window shows is a **topic**: one inventory, with its last good value, a sequence number, `sampledAt`, and whether the latest collection failed. The daemon keeps the topics warm on a schedule (`sampler/policy.rs`) and pushes changes to the windows that hold them.

Background collection never runs in the daemon process, which answers windows. It runs in `ratmos sampler`, a separate unit under `CPUSchedulingPolicy=idle`, `IOSchedulingClass=idle`, `CPUWeight=1` and a `CPUQuota`, and the daemon sends it one job at a time over its socket. With no sampler connected nothing is collected in the background, and topics fill on demand.

The daemon's governor decides whether a background job may start. All of these must hold: CPU pressure is low (`/proc/pressure/cpu`, or load per core without it), the duty budget allows it (about one percent of a core averaged over a window, counting the sampler's children), no foreground collection is running, and no other background job is running. Pressure backs a topic off exponentially. The `CPUQuota` is the hard backstop.

A read is answered from the topic when it is fresh enough. A cold or too-old topic is collected at once in the daemon, with interactive priority, and an interactive request never joins a background run. `live` is the exception to scheduled sampling: the daemon samples it itself, in the foreground, only while a window holds it and no faster than the quickest holder asks, because a window is waiting on it.

## Consequences

- "Never contends" means this bound, not a promise about every instant: under sustained load background sampling stops and foreground reads carry the page.
- The idle IO class does nothing under an IO scheduler that ignores priorities, such as the `none` scheduler most NVMe drives use. The CPU limits are the guarantee, and the documentation says so.
- A dev checkout needs the sampler unit, or `ratmos sampler`, to sample in the background.
- Rates are computed from `sampledAt`, the time the daemon began collecting, never from the window's clock. A repeat sample must not be served twice, so `live` has a one-second maximum age.

## Alternatives considered

- **Idle scheduling for the whole daemon.** It would make a window wait behind a starved process.
- **A foreign call for `SCHED_IDLE` inside one process.** See 0007; a separate unit gives the same scheduling declaratively.
- **Sampling only on demand.** It is never wrong but always slow on the first page.

## Enforced by

- `backend/src/sampler/governor.rs`, `worker.rs`, `topics.rs`, `policy.rs`
- `packaging/atmosd-sampler.service` (idle classes and quota), `backend/src/tasks/sampler_unit.rs` tests
- `backend/src/sampler/daemon_tests.rs` (the governor, a dead sampler, last good kept on failure)
- `backend/src/sampler/client.rs` tests (a reader never joins a background run)
