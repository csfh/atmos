# 0009. Every job and interactive tool runs in the daemon

Status: Accepted. Date: 2026-10-19.

## Context

Registry writes (ADR 0004) were typed, but anything that streamed output, read input or needed the user's session ran as `ratmos apply -- <argv>` on a process the window started. About ninety call sites built a command line in QML. Those jobs died with the window, were invisible to every other window, took no part in lanes, and the window could not tell a package install from a picker. A new frontend would have had to copy each command line.

## Decision

A job is a registry entry, like a write, with a kind: **Job** (streams output, may take long), **Interactive** (needs the user's session: a picker, a region tool, a recorder) or **Launch** (starts something that stays open). The window sends `job.start {cid, name, args, label}` over its session and gets `job` events back (`started`, `stream`, `exit`, `gone`). `job.input`, `job.cancel`, `job.attach` and `job.list` complete the set. Passwords, plans and reports are arguments that become the job's stdin; they are never in a command line, a label or a job list.

The daemon runs every job:

- As a **transient user unit** (`systemd-run --user --pipe --wait --collect`), in a cgroup of its own, at the priority of an ordinary program of yours and not the daemon's idle one, with the window's allow-listed environment. `KillMode=process`: a tool that starts a recorder, a clipboard holder or a switcher and exits does not take it down with the unit. A cancel signals the whole unit (`systemctl --user kill --kill-whom=all`).
- On a **lane**, so two package installs go one at a time and a job holds back reads of what it changes until it ends.
- Past the window. A job outlives the window that started it and the daemon that is replaced. A window that opens while one runs lists the jobs, attaches, and replays what it missed; a daemon that restarts adopts the units a dead one left.
- With no systemd user manager (tests), as a child of the daemon in its own process group.

`ratmos apply --` and `ratmos task` stay as terminal subcommands. The window does not use them.

## Consequences

- No QML builds a command line. `tests/bus-conformance.baseline` is zero for `runCommand`, `runJob` and `runInteractive` without a `Requests.*` op and for a `Process` outside `Backend.qml`.
- The in-app sudo prompt, the file picker, the region picker and the recorder were each run from a real Quickshell window against a real daemon. The recorder needed `KillMode=process`: the recording script starts `gpu-screen-recorder` in the background and exits, and the default kills it with the unit.
- A job started with no daemon fails with `unavailable`. See ADR 0010.
- A job's output is kept in the daemon (bounded) for a window that attaches late.
- A cancel needs the user manager that started the unit. A daemon in another runtime directory cannot stop it.

## Alternatives considered

- **Keep `apply` in the window and lease a lane.** It dies with the window, and the window keeps the command lines.
- **Run the tool as a child of the daemon.** One hung tool, or a restart, would take every job with it, and the tool would run at idle priority.
- **Polkit for privileged steps.** The window's flows use `sudo -n` or `sudo -S` with a password on stdin, so a job needs no agent in the session.

## Enforced by

- `backend/src/command/entries.rs`, `table.rs`, `jobs.rs` (the registry and its checks)
- `backend/src/sampler/jobs.rs`, `session.rs`, `daemon.rs` (units, lanes, adoption, frames)
- `backend/tests/jobs.rs` (follow, refuse, outlive a window, reattach, cancel, no daemon, deferred replacement)
- `tests/bus-conformance.test.js`, `tests/writes-registry.test.js` (every `Requests.job` name and literal argument is accepted by `ratmos job-check`)
- `docs/protocol.md` (a row for every job and interactive tool)
