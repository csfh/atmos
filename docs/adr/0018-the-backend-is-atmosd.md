# 0018. The backend is `atmosd`

Status: Accepted. Date: 2026-10-26.

## Context

The backend binary, its crate and its units were called `ratmos`: "r" for Rust, from when the QML window was the app and Rust was the helper. The Rust window now ships beside it, the daemon is the authority (ADR 0007), and the name no longer says what the program is.

The user decided on 2026-10-25 to rename it `atmosd`, and that v1 is not compatible with older versions (ADR 0017).

## Decision

The binary, the crate, the unit and socket files and every command line are `atmosd`.

- `atmosd daemon`, `atmosd sampler`, `atmosd serve`, `atmosd task`, `atmosd request`, `atmosd install`, `atmosd launch` and `atmosd sentinel` are the old commands under the new name.
- The units are `atmosd.service`, `atmosd-sampler.service`, `atmosd.socket` and the transient `atmosd-adhoc`. The socket is `$XDG_RUNTIME_DIR/atmos/atmosd.sock`. The Bundle runs `bin/atmosd`.
- `RATMOS_DIRECT` is `ATMOSD_DIRECT`.
- No shim is kept: no `ratmos` binary, no alias unit, no old socket. Records written before this one keep the old name as history.

## Consequences

- A machine that ran an earlier build keeps its `ratmos.service` and `ratmos-sampler.service` user units, and a running `ratmos daemon`, until the user stops and removes them. `atmosd install` writes the new units and does not touch those. Two daemons on the same machine do not share a socket, so the old one is harmless but wasteful.
- An older install that updates in place cannot find `ratmos` any more; this is the failure ADR 0017 already accepts.
- The golden replies in `tests/golden/` carry no name, so none was re-recorded.

## Alternatives considered

- **Keep `ratmos` and add `atmosd` as an alias.** It leaves two names for one program in every doc and unit, for installs that v1 does not serve anyway.
- **Rename the units and leave the crate and binary alone.** It splits the name between what the user sees in `systemctl` and what they type.

## Enforced by

- `tests/consistency.test.js` fails if the tree names `ratmos` outside past ADRs and the changelog.
