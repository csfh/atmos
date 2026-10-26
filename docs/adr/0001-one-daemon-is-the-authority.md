# 0001. One daemon is the authority

Status: Accepted. Date: 2026-10-18.

## Context

Each Atmos window used to own its state. Every window collected the inventories it showed, ran its own writes and looked at the files it cared about, so two windows could disagree, the machine did the same work twice, and nothing could order a write in one window against a read in another. The first version of the sampling service was a cache that a window could ignore.

## Decision

`ratmos daemon` is the one authority on state. Windows are clients of it. It owns the topics (the sampled inventories), runs the command bus, watches files for every window and answers reads, and it is what a window talks to for everything it shows or changes.

The daemon is **mandatory on an installed system** and **reachable without anyone starting it**:

- `ratmos.socket` listens on `$XDG_RUNTIME_DIR/atmos/ratmos.sock` from login. systemd starts the daemon on the first connection and passes it the open socket (`LISTEN_FDS`), so a daemon that left is started again by the next connection.
- Without the units (a checkout run from a terminal) `ratmos serve` makes sure one exists: the installed units first, then a transient user unit, then a bare detached process.
- A daemon leaves when its runtime directory is gone or the binary it runs has been replaced, since `VERSION` does not change between builds.
- One instance per runtime directory, by an exclusive lock.

The daemon is a user unit, not a system unit: it serves one user's windows and runs their tools with their environment.

## Consequences

- There is one place to look for what Atmos believes about the machine, and one place that decides the order of work.
- A window that cannot reach a daemon still works. It collects inline and runs writes in its own process, without ordering across windows. This fallback is deliberate, so Atmos is usable on a machine with no systemd user session, and it is the degraded mode, not the design. *Superseded by [0010](0010-nothing-works-without-the-daemon.md): a window without a daemon says so and waits for one.*
- Tests must never see the daemon of the session they run in. `tests/run` exports a scratch `XDG_RUNTIME_DIR` and `ATMOS_NO_AUTOSTART=1`.
- A daemon started by a window (`ratmos-adhoc.service`) holds the socket, so `ratmos install` stops it before starting the installed units.

## Alternatives considered

- **A system unit.** It would serve every user, but it would run a user's tools as another identity and could not see their session environment.
- **No daemon, a shared cache file.** It cannot order writes, push events or hold a window back until a write is visible.
- **A daemon the user must start.** It leaves windows with no authority in the common case, which is the state this decision removes.

## Enforced by

- `backend/src/sampler/daemon.rs` (single-instance lock, inherited socket)
- `backend/src/sampler/launch.rs` (the autostart ladder)
- `packaging/atmosd.socket`, `packaging/atmosd.service`
- `backend/tests/serve.rs` (a daemon leaves when its binary is replaced or its directory is gone)
- `tests/install.test.js`, `tests/packaged.test.js` (the three units and their order)
