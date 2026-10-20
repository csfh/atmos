# 0007. System integration uses std only

Status: Accepted. Date: 2026-10-18.

## Context

Several things in this design want a system call: running a process at idle priority, watching files with inotify, inheriting a systemd socket, detecting a process's CPU use. Each can be done with a foreign call into libc or a crate that wraps one. Atmos has three dependencies and a single static binary, and the backend is the part that has to be trusted with a user's files.

## Decision

The backend makes no foreign call and adds no crate for these. Each need is met another way:

- **Idle scheduling** is declared in a separate unit (`CPUSchedulingPolicy=idle`, 0002), not set by the process.
- **Socket activation** takes the inherited descriptor with `UnixListener::from_raw_fd` (std, one reviewed `unsafe` block) after checking `LISTEN_PID` and `LISTEN_FDS`, and duplicates it so the copy is close-on-exec.
- **File watching** is a once-a-second `stat` in the daemon, shared by all windows, reading a file only when its signature moves.
- **CPU pressure and per-process CPU** come from `/proc/pressure/cpu` and `/proc/PID/stat`.
- **A process's environment, session and cgroup** are handled by starting it through `systemd-run` or `Command`, never by calling into the kernel.

## Consequences

- File changes can be seen up to a second late, and a watched path costs a `stat` a second.
- A feature that would need a foreign call (a latency-sensitive notification, a signal-safe watcher) needs a new decision.
- The one `unsafe` block in the backend is the inherited descriptor.

## Alternatives considered

- **`libc` or `nix` for `sched_setscheduler`, `inotify` and `fcntl`.** It buys precision and costs a dependency and a class of mistakes.
- **A `notify` crate.** It pulls a tree of dependencies for a one-second saving.

## Enforced by

- `backend/Cargo.toml` (the dependency list)
- `backend/src/sampler/daemon.rs` (`inherited_listener`, `listen_fds` and its test)
- `backend/src/sampler/watch.rs`, `backend/src/sampler/governor.rs`
- `packaging/ratmos-sampler.service`
