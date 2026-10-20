# 0006. A failure is never an empty value

Status: Accepted. Date: 2026-10-18.

## Context

A collector that ran `hyprctl`, `systemctl` or `lsblk` and got an error used to return an empty document, and the window drew it as a machine with no monitors, no services or no disks. "No monitors reported" appeared after a brightness change when `hyprctl` failed. The failure was invisible and the last good data was lost.

## Decision

A collector that fails says so, and the last good value stays.

- A topic keeps its last good sample. A failed collection marks it `stale` with the last error and never replaces the value.
- A collector whose main tool fails returns an error naming the tool (`inventory::require`), not an empty document. This holds for `services` (`systemctl`) and `disks` (`lsblk`, `df`), and for `hypr::monitors()`, which returns no list when `hyprctl` fails, so the window keeps the list it has.
- A tool that is only one source among several (`hardware` falls back from `lspci` to sysfs) may degrade, and a collector that reports each check's own failure (`diagnostics`) reports it as a result.
- A background job that times out, errors or panics is an error for its topic, and the sampler carries on.
- A typed write that does not run is an error reply. One that runs and does not show the expected state says `settled: false`.

## Consequences

- A page can show a failure and the last good data together.
- On a machine without `systemctl` the services page is an error every time, which is true.
- The settings snapshot is covered by ADR 0011: a producer whose tool did not run, timed out or answered with an error leaves its keys out and names them, with the reason, under `failed` (`_failed` in a topic).

## Alternatives considered

- **Return partial documents with an `errors` list.** The window would have to merge it everywhere, and an empty list still looks like an answer.
- **Retry until it works.** It hides a real failure and costs the CPU the governor protects.

## Enforced by

- `backend/src/inventory/mod.rs` (`require`), `services.rs`, `disks.rs`, `backend/src/display.rs`
- `backend/tests/inventory_failure.rs` (a failing tool is an error and a working one is a document)
- `backend/src/sampler/daemon_tests.rs` (a failed run marks the topic stale and keeps the last good sample)
- `tests/monitors-failure.test.js` (a failed `hyprctl` leaves the monitor list alone)
